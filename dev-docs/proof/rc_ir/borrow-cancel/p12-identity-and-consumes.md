# P5, P6, P7 -- identity とオブジェクト、`acted_references`、消費の網羅性

この文書は `README.md` の P5 (a)、P5 (b)、P5 (c)、P6、P7 を証明する。立つのは `README.md` の定義 D1-D21、
仮定 A1-A15、および命題 P1-P4 の**言明**である。P1-P4 の証明は `p10-leaves-and-units.md` と
`p11-origin-soundness.md` にあり、この文書はその言明だけを使う。

これに加えて、この文書は `README.md` に無い前提 **H1 (`Match` の網羅性)** を第 1 節で置く。H1 が要るのは
L1b、L1 の E3 の場合、L4、P5 (a)、P6 (b) である。第 9 節が、H1 を仮定として `README.md` へ足すことを提案する。

読んだコードは作業ツリーの版である。README の対象コミット `a924f115` との差分は、生成される `// PROOF:` の
注釈行だけであり、この文書が引用する記号の本文は対象コミットと一致する。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P5 (a) 対の健全性 | 証明した (H1 の下で) |
| P5 (b) 対の有効性 | 証明した |
| P5 (c) 被覆 | 証明した |
| P6 (`acted_references` は静的な上位近似である) | 証明した (H1 の下で) |
| P7 (消費の網羅性) | 証明した |
| L6 (報告しない箇所は D9 の消費ではない) | 証明した (P7 に添える補題) |

P5 (a) の要は L4 である。**スロットの path は boxed leaf であり、`origin` の再帰は boxed leaf の path から
出ると boxed leaf の path しか訪れない。** `origin_from_leaves_under` が `truncate_to_unit` で path を
unit へ切り詰める枝は、leaf でない path でだけ通る。1 つの unit の下に複数の leaf を持つ値
(`Std::Option (a, b)` の payload) が 2 つのオブジェクトに届いても、その 2 つの leaf の `identity` が
1 つに潰れることは無い。第 9 節に、この結論がどの仮定に載っているかを書く。

P6 (b) の要は、`identity` が付ける名前とオブジェクトの間の写像 `ν` である。L4 の (ii) が
`ν(id(v, λ)) = obj(v, λ)` を与え、これ 1 つから、名前づけが 2 つのオブジェクトを 1 つに潰さないこと
(= P5 (a)) と、P6 (b) の等号の両方が出る。逆向き -- 1 つのオブジェクトが 2 つの名前を持つこと -- は
実際に起きる (R1) が、P6 (b) の等号を壊さない。P6 の節がこれを述べる。

## 1. 記法と前提

1 つの関数 (またはグローバル初期化子) の本体を固定し、その本体から作られる `VarTable` を `vars`、プログラムの
`TypeEnv` を `type_env` と書く (`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`)。この
2 つは本体ごとに 1 つなので、以下では `origin` と `acted_references` の第 1・第 2 引数を落として書く。

- `origin(x, π)` は `origin(vars, type_env, x, π)`。この記法が定まること (同じ `(x, π)` について値が 1 つで
  あること) は L0 が示す。
- `id(x, π)` は `origin(x, π).identity()`、`cand(x, π)` は `origin(x, π).candidates()` を集合とみなした
  もの、`act(x, π)` は `origin(x, π).acted_on()` を集合とみなしたもの
  (`CODE src/rc_ir/ownership.rs: Origin::identity`, `Origin::candidates`, `Origin::acted_on`)。
- `ty(x)` は `x` が得る値の型である。A12 より同じ名前の `RcVar` が持つ型は一致するので、これは `x` の
  出現によらない。

**DEF `L`**
`L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合である。D4 より、
これが「`v` の `π` の下の boxed leaf」の全体である。inhabited (D16) でないものを含む。

**DEF `Linh`**
実行路の位置 `p` において、`L(v, π)` の元のうち `v` の値で inhabited (D16) なものの集合を `Linh(v, π, p)` と
書く。

`ActRefs(v, π)` は D15 の `acted_references(v, π)` である。`VarPath` は対 `(FullName, FieldPath)` である
(`CODE src/rc_ir/ast.rs: VarPath`)。等号はこの対の等号である。

この文書は補題を `L0`、`L1b`、`L1`、`L1a`、`L2`、`L3`、`L4`、`L5`、`L6` (この順に並べる)、反例を
`R1` と呼ぶ。**`BY` の行で
引用してよいのは、それぞれの言明だけである。** 言明が複数の主張からなる補題は主張に (a)、(b)、… の名札を
付け、`L5 (c)` のように引用する。同じ規則を P6 (a)、P6 (b)、P7 (a)、P7 (b) にも使う -- これらは
この文書が P6 と P7 を分けた主張であり、引用してよいのはその言明である。

### この文書が置く前提

**H1 (`Match` の網羅性)** -- 果たす者: lowering (`CODE src/rc_ir/lower.rs: Lowerer::lower_match`,
`Lowerer::lower_if`) と、アームの列を保つ後段のパス。検査: 無し。
すべての `Match(s, arms)` について、次のどちらかが成り立つ。`arms` が catch-all アーム (`tag` が `None`) を
持つか、`s` の値が取りうる実行時のタグがいずれかのアームの `tag` である。

`README.md` はこの前提を持たない。`validate` が見るのは、アームが 1 つ以上あること、catch-all アームが
最後にあること、2 つのアームが同じ変位を担わないことだけである
(`CODE src/rc_ir/validate.rs: Validator::check_rhs`)。網羅しない `Match` -- 3 変位の union に対する
`[tag 0, tag 1]` -- では、実行時のタグが 2 のとき、コード生成は `tag = Some(1)` のアームへ入る
(最後のアームが switch の default である)。そのとき D9 の移動の表の「unbox union の変位アームの payload
束縛」の行が名指す活性変位と、`origin` が辿る静的な変位番号が食い違い、L1b、L4、P5 (a)、P6 (b) が偽に
なる。
第 9 節が、この前提を仮定として `README.md` へ足すことを提案する。

### DEF 路のスロット

1 つの活性化と、その中の 1 回の実行 (D21) を固定し、その実行が辿る実行路を `ρ` とする。名前 `x` が
`ρ` の上で**値を得る**とは、次のいずれかをいう。

- **(S1)** `x` がこの本体のパラメータか capture である。
- **(S2)** `Let(x, rhs, k)` の形の節点、または `Destructure(c, fs, s, k)` で `(i, x) ∈ fs` である節点が、
  `ρ` の上にある。
- **(S3)** `Let(m, Match(s, arms), k)` の形の節点が `ρ` の上にあり、`ρ` を辿る実行がその `Match` で選ぶ
  アーム `a` の `payload` が `x` である。
- **(S4)** `x` が `vars.bindings` に無い名前である。`VarTable::of` はパラメータ・capture と、
  `collect_bindings` が歩く本体のすべての束縛を記録するので、この場合の `x` は本体が束縛しない名前、
  すなわち D1 の `globals` の記号か `funcs` の記号であり (A11、`CODE src/rc_ir/ownership.rs: VarTable::of`,
  `collect_bindings`)、その値はプログラムが定める 1 つの値である。

D2 より、本体が変数を束縛する節点は `Let`、`Destructure`、`Match` のアームの payload の 3 つだけなので、
(S1)-(S3) は本体の束縛をすべて尽くす。

`x` が `ρ` の上で値を得るとき、その値の inhabited (D16) な boxed leaf `λ` との対 `(x, λ)` を **`ρ` の
スロット**と呼び、その leaf が指すオブジェクトを `obj(x, λ)`、その leaf が持つ参照 (A5 よりちょうど 1 つ) を
`ref(x, λ)` と書く。

D2 より本体は木であり、D3 の実行路は各節点を高々 1 度通り、A6 より束縛名は相異なるので、1 つの活性化・
1 つの路について `x` が得る値は高々 1 つである。よって `obj(x, λ)` と `ref(x, λ)` は路の上の位置に依らない。
位置 `p` のスロット (D6) は、`p` までに値を得た名前についての `ρ` のスロットであり、D6 の `obj(x, λ)` は
この `obj(x, λ)` である。`ρ` のスロットは D6 のスロットを路全体へ広げたものであり、L4 の (i) はこの広い側で
述べる。

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
D9 の移動の表の各行は、構文の粒度で「どの値の参照がどの値へ移るか」を述べる。A5 より、値が保持する参照は
その値の inhabited な boxed leaf にちょうど 1 つずつ在るので、行が述べる移動は、始点の値の leaf と終点の
値の leaf の間の対応である。**E1 から E6 は、その対応を leaf ごとに書き下したものである。** 対応は上の
一覧のとおりであり、D17 が `origin` の辿る辺について書く leaf の写り方 -- `Move`・catch-all・`Join` は
`λ` を変えず、`Destructure` のフィールドと変位アームの payload は先頭に添字を足し、`Llvm` は宣言の `σ` へ
置き換える -- と一致する。辺が在るのは両端がスロットであるときに限る。とくに E2 の `[i] ++ λ` は、`i` が
穴のとき `ty(c)` の boxed leaf ではない (D4 の規則 5) ので、その辺は無い。

**DEF `ρ` の上で実行された辺**
辺が **`ρ` の上で実行された**とは、次をいう。

- E1、E2、E5 の辺: その辺を定める節点 (`Let(x, Var(y), k)`、`Destructure(c, fs, s, k)`、
  `Let(x, Llvm(gen, args), k)`) が `ρ` の上にある。
- E3、E4 の辺: その辺を定める `Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、`ρ` を辿る実行が
  その `Match` で選ぶアーム (D21) が、その辺の属するアームである。
- E6 の辺: その辺を定めるアーム本体の終端の `Ret(x)` が `ρ` の上にある。D3 より、これはその `Match` の
  節点が `ρ` の上にあり、`ρ` がそのアームを通ることと同じである。

## L0 (`origin` は `origin_inner` の値を返す)

**言明**。`vars` と `type_env` を固定すると、`origin(vars, type_env, x, π)` は `(x, π)` の関数として定まり、
その値は `origin_inner(vars, type_env, x, π)` の 1 回の呼び出しが返した値である。

<1>1. `origin` は、`vars.origins` に鍵 `(x, π)` の記録があればその値を返し、無ければ
      `grow_stack(|| origin_inner(vars, type_env, x, π))` の値を鍵 `(x, π)` で記録して返す。
  BY CODE src/rc_ir/ownership.rs: origin

<1>2. `grow_stack(f)` の値は `f()` の値である。
  BY A15

<1>3. `vars.origins` に書き込むのは `<1>1` の記録だけである。`VarTable::empty` がこの欄を空に初期化し、
      `ownership.rs` の他の箇所はこの欄を `<1>1` の読み出し以外に触らない。
  BY CODE src/rc_ir/ownership.rs: VarTable, VarTable::empty, origin

<1>4. QED
  `<1>1` より返り値は、記録の値か、いま行った `grow_stack(|| origin_inner(..))` の値である。`<1>2` より
  後者は `origin_inner(vars, type_env, x, π)` の値であり、`<1>3` より前者もある時点の同じ呼び出しの値で
  ある。鍵 `(x, π)` について記録は 1 度しか書かれないので (`<1>1` は記録が無いときにだけ書く)、同じ
  `(x, π)` についてどの呼び出しも同じ値を返す。
  BY <1>1, <1>2, <1>3

## L1b (変位アームは scrutinee の活性変位のアームである)

**言明**。`Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、`ρ` を辿る実行がその `Match` で
`tag = Some(t)` のアームを選ぶとする。このとき、その位置での `s` の値の実行時のタグは `t` である。

<1>1. `ρ` を辿る実行がこの `Match` で選ぶアームは、`s` の値の実行時のタグに `tag` が等しいアームであり、
      そのようなアームが無ければ、コード生成の振る舞いが決めるアームである。
  BY D21

<1>2. CASE `arms` に、`s` の値の実行時のタグ `t*` に `tag` が等しいアームがある。
  <2>1. `<1>1` より、選ばれるのはそのアームである。前提より選ばれたアームの `tag` は `Some(t)` なので、
        `Some(t) = Some(t*)` すなわち `t = t*` である。
    BY <1>1
  <2>2. QED
    BY <2>1

<1>3. CASE `arms` に、`s` の値の実行時のタグに `tag` が等しいアームが無い。
  <2>1. `arms` は catch-all アームを持つ。
    H1 より、`arms` が catch-all アームを持つか、`s` の値の実行時のタグがいずれかのアームの `tag` で
    ある。後者はこの CASE の前提に反する。
    BY H1
  <2>2. catch-all アームは `arms` の最後である。
    コード生成は、最後のアームを除く各アームについて `arm.tag.expect("a non-final match arm must be a
    variant arm")` を評価する。`ρ` を辿る実行が在るのだからコード生成は panic しておらず、最後のアーム
    以外はすべて `tag` を持つ。`<2>1` の catch-all アームは `tag` を持たないので、最後のアームである。
    BY <2>1, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
  <2>3. QED
    コード生成は、最後のアームのブロックを switch の default とし、それ以外の各アームをその `tag` の
    case とする。よって `<1>1` の第 2 の場合に実行が入るのは最後のアームであり、`<2>1` と `<2>2` より
    それは catch-all アーム、すなわち `tag` が `None` のアームである。これは前提の `Some(t)` に反するので、
    この CASE は起きない。
    BY <1>1, <2>1, <2>2, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match

<1>4. QED
  `<1>2` と `<1>3` は「`s` の値の実行時のタグに `tag` が等しいアームがあるか無いか」の 2 つの場合であり、
  排中律より尽きている。
  BY <1>2, <1>3

## L1 (実行された辺の両端は同じオブジェクトを指す)

**言明**。`ρ` の上で実行された (DEF `ρ` の上で実行された辺) E1 から E6 の辺の両端のスロットは、同じ参照を
持ち、したがって同じオブジェクトを指す。

<1>1. 1 つの値の inhabited な各 boxed leaf には参照がちょうど 1 つ在り、同じ値の同じ leaf は同じ参照を
      持つ。また、同じ参照は 1 つのオブジェクトに対する義務の 1 単位なので、同じ参照を持つ 2 つの
      スロットは同じオブジェクトを指す。
  BY A5, D8

<1>2. E1 の辺 `(y, λ)`-`(x, λ)`。
  <2>1. D2 の `Let` の行より `x` は `rhs` すなわち `Var(y)` の値に束縛され、その値は `y` の値である。
        A12 の move-bind の行より `ty(x) = ty(y)` である。
    BY A12, D2
  <2>2. QED
    `<2>1` より `(x, λ)` と `(y, λ)` は同じ値の同じ leaf である。D9 の移動の表の第 1 行がこの移動を
    述べ、DEF 辺の leaf 対応 がそれを leaf ごとに `λ` ↔ `λ` と読む。
    BY D9, DEF 辺の leaf 対応, <1>1, <2>1

<1>3. E2 の辺 `(c, [i] ++ λ)`-`(f, λ)`。
  <2>1. D2 の `Destructure` の行より `f` は `c` の値の第 `i` フィールドに束縛される。A12 の
        `Destructure` の行より `ty(f)` は `ty(c)` の第 `i` フィールドの型である。
    BY A12, D2
  <2>2. `ty(c)` の boxed leaf のうち `[i]` を前置に持つものは、`ty(f)` の boxed leaf に `[i]` を前置した
        ものの全体である。
    D4 の規則 5 より、unbox 集約の leaf は `unpunched_field_types` が返す各フィールドの添字を前置した
    そのフィールドの型の leaf である。E2 の辺が在るのは `[i] ++ λ` が `ty(c)` の boxed leaf であるとき、
    すなわち `i` が穴でないときである (DEF 辺の leaf 対応)。
    BY D4, DEF 辺の leaf 対応, <2>1
  <2>3. QED
    `<2>1` と `<2>2` より、`c` の値の位置 `[i] ++ λ` と `f` の値の位置 `λ` は同じ位置である。D9 の移動の
    表の unbox 容器の `Destructure` の行がこの移動を述べ、DEF 辺の leaf 対応 がそれを leaf ごとに
    `[i] ++ λ` ↔ `λ` と読む。
    BY D9, DEF 辺の leaf 対応, <1>1, <2>1, <2>2

<1>4. E3 の辺 `(s, [t] ++ λ)`-`(p, λ)`。
  <2>1. この辺が `ρ` の上で実行されたとき、`ρ` を辿る実行はこの `Match` で `tag = Some(t)` のアームを
        選んでおり、この位置での `s` の値の実行時のタグは `t` である。
    BY L1b, DEF `ρ` の上で実行された辺
  <2>2. A12 の payload と変位の行より `ty(p)` は `ty(s)` の変位 `t` の型であり、D4 の規則 5 より
        `ty(s)` の boxed leaf のうち `[t]` を前置に持つものは、`ty(p)` の boxed leaf に `[t]` を前置した
        ものの全体である。
    BY A12, D4
  <2>3. QED
    `<2>1` より `t` は活性変位であり、D9 の移動の表の unbox union の変位アームの行は「scrutinee の活性
    変位の参照が payload 変数へ」と述べる。`<2>2` より `s` の値の位置 `[t] ++ λ` と `p` の値の位置 `λ` は
    同じ位置であり、DEF 辺の leaf 対応 がこの移動を leaf ごとに `[t] ++ λ` ↔ `λ` と読む。
    BY D9, DEF 辺の leaf 対応, <1>1, <2>1, <2>2

<1>5. E4 の辺 `(s, λ)`-`(p, λ)`。
  <2>1. A12 の catch-all アームの行より `ty(p) = ty(s)` である。
    BY A12
  <2>2. QED
    D9 の移動の表の catch-all の行は「scrutinee の参照が payload 変数へ」と述べ、`s` のすべての参照が
    移る。`<2>1` より両者の boxed leaf は同じであり、DEF 辺の leaf 対応 がこの移動を leaf ごとに
    `λ` ↔ `λ` と読む。
    BY D9, DEF 辺の leaf 対応, <1>1, <2>1

<1>6. E5 の辺 `(args[j], σ)`-`(x, λ)`。
  A3 の「単一の `Arg(j, σ)`」の行は、生成コードが結果の leaf `λ` に第 `j` オペランドの leaf `σ` と
  **同じ参照**を置き、新しい参照を作らないと述べる。D9 の移動の表の `Llvm` の素通し leaf の行がこれを
  移動とし、DEF 辺の leaf 対応 がこの行の leaf の対応を `σ` ↔ `λ` と読む。
  BY A3, D9, DEF 辺の leaf 対応, <1>1

<1>7. E6 の辺 `(x, λ)`-`(m, λ)`。
  <2>1. D2 の `Ret` の行より アーム本体の値は `x` の値であり、D2 の `Let` の行より `m` はその `Match` の
        値、すなわち通ったアームの値に束縛される。A12 のアームの結果と `Match` の束縛変数の行より
        `ty(m) = ty(x)` である。
    BY A12, D2
  <2>2. QED
    `<2>1` より `(m, λ)` と `(x, λ)` は同じ値の同じ leaf である。D9 の移動の表の `Match` のアーム本体の
    `Ret(x)` の行がこの移動を述べ、DEF 辺の leaf 対応 がそれを leaf ごとに `λ` ↔ `λ` と読む。
    BY D9, DEF 辺の leaf 対応, <1>1, <2>1

<1>8. QED
  E1 から E6 は D20 (すなわち D9 の移動の表) の 6 行に 1 対 1 で対応し、`<1>2` から `<1>7` がその 6 つで
  ある。
  BY D9, D20, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7

## L1a (`Binding` の形は `ρ` の上の束縛節点の形である)

**言明**。`(x, λ)` を `ρ` のスロットとし、`vars.bindings.get(x)` が `Some(b)` であるとする。このとき
次が成り立つ。

- **(a)** `b = Param` であることと、`x` がこの本体のパラメータか capture であることは同値である。
- **(b)** `b ≠ Param` のとき、`x` を束縛する節点が `ρ` の上にちょうど 1 つあり、その形は `b` の構成子に
  応じて次のとおりである。さらに、その節点が定める辺は `ρ` の上で実行された辺である。

| `b` | `ρ` の上の節点 | 定める辺 |
|---|---|---|
| `Move(y)` | `Let(x, Var(y), k)` | E1 の `(y, λ)`-`(x, λ)` |
| `Producer` | `Let(x, App(callee, args), k)` または `Let(x, Closure(f, caps), k)` | 無し |
| `Field(c, i)` | `Destructure(c, fs, s, k)` で `(i, x) ∈ fs` | `c` が unbox のとき E2 の `(c, [i] ++ λ)`-`(x, λ)` |
| `Payload(s, Some(t))` | `Let(m, Match(s, arms), k)` で、`ρ` が選ぶアームが `tag = Some(t)`、`payload = x` のもの | `s` が unbox のとき E3 の `(s, [t] ++ λ)`-`(x, λ)` |
| `Payload(s, None)` | `Let(m, Match(s, arms), k)` で、`ρ` が選ぶアームが catch-all、`payload = x` のもの | E4 の `(s, λ)`-`(x, λ)` |
| `Llvm(gen, args, ty)` | `Let(x, Llvm(gen, args), k)` で `ty = ty(x)` | 宣言が単一の `Arg(j, σ)` のとき E5 の `(args[j], σ)`-`(x, λ)` |
| `Join(rs)` | `Let(x, Match(s, arms), k)` | `ρ` が選ぶアーム `a` について `r_0 := returned_var(a.body)` とおくと、`r_0 ∈ rs` であり、E6 の `(r_0, λ)`-`(x, λ)` |

<1>1. `VarTable::of` は、パラメータと capture のそれぞれについて `Binding::Param` を記録し、その後
      `collect_bindings` を呼ぶ。`collect_bindings` は `Param` を記録しない。
  BY CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings

<1>2. `collect_bindings` が `x` について記録を作るのは、次の 3 つの節点でだけであり、記録される構成子は
      次のとおりである。
      `Let(x, rhs, k)`: `rhs` が `Var(y)` なら `Move(y)`、`Llvm(gen, args)` なら
      `Llvm(gen, args, x.ty)`、`Closure(..)` または `App(..)` なら `Producer`、`Match(s, arms)` なら
      `Join(arm_results)` で `arm_results` は各アームの `returned_var(&arm.body)` の列。
      `Destructure(c, fs, _, k)` の各 `(i, fv)`: `fv` について `Field(c, i)`。
      `Let(m, Match(s, arms), k)` の各アーム: `arm.payload` について `Payload(s, arm.tag)`。
  BY CODE src/rc_ir/ownership.rs: collect_bindings, returned_var

<1>3. (a) が成り立つ。
  `x` がパラメータか capture であるとき、`<1>1` が `x` について `Param` を記録し、A6 より本体はその名前を
  束縛し直さないので、`<1>2` の記録がそれを上書きすることはない。よって `b = Param` である。逆に
  `b = Param` であるとき、`<1>2` より `collect_bindings` が作る構成子は `Move`、`Llvm`、`Producer`、
  `Join`、`Field`、`Payload` の 6 つで `Param` を含まないので、この記録は `<1>1` が作ったものであり、
  `x` はパラメータか capture である。
  BY A6, <1>1, <1>2

<1>4. `b ≠ Param` のとき、`x` を束縛する節点が `ρ` の上にある。
  DEF 路のスロット より `x` は `ρ` の上で値を得る。(S4) は `x` が `vars.bindings` に無いことを言うので
  前提に反する。(S1) は `<1>3` より `b = Param` を与えるので前提に反する。残るのは (S2) と (S3) であり、
  どちらも `x` を束縛する節点が `ρ` の上にあることを言う。
  BY DEF 路のスロット, <1>3

<1>5. `x` を束縛する節点は高々 1 つであり、`b` はその節点が `<1>2` の規則で作る構成子である。
  BY A6, <1>2

<1>6. (b) の表の節点の形が成り立つ。
  `<1>4` と `<1>5` より、`x` を束縛する `ρ` の上の節点が 1 つあり、`<1>2` の対応を逆に読むと、記録された
  構成子ごとにその節点の形が表のとおりに定まる。`Payload(s, tag)` の場合、`x` を束縛するのは (S3) の
  意味であり、`ρ` が選ぶアームの `payload` が `x` である。`Join(rs)` の場合、`rs` は各アームの
  `returned_var(&arm.body)` の列なので、`ρ` が選ぶアーム `a` の `returned_var(a.body)` は `rs` の元で
  ある。
  BY DEF 路のスロット, <1>2, <1>4, <1>5

<1>7. (b) の表の辺が `ρ` の上で実行された辺である。
  <2>1. `Move(y)`、`Llvm(gen, args, ty)`、`Field(c, i)` の場合、`<1>6` の節点はそれぞれ
        `Let(x, Var(y), k)`、`Let(x, Llvm(gen, args), k)`、`Destructure(c, fs, s, k)` であって `ρ` の
        上にある。DEF `ρ` の上で実行された辺 の第 1 の場合より、E1・E5・E2 の辺は `ρ` の上で実行された。
    BY DEF `ρ` の上で実行された辺, <1>6
  <2>2. `Payload(s, Some(t))` と `Payload(s, None)` の場合、`<1>6` の節点は `Let(m, Match(s, arms), k)`
        であって `ρ` の上にあり、`ρ` が選ぶアームが `x` を payload とするアームである。DEF `ρ` の上で
        実行された辺 の第 2 の場合より、E3・E4 の辺は `ρ` の上で実行された。
    BY DEF `ρ` の上で実行された辺, <1>6
  <2>3. `Join(rs)` の場合、`<1>6` の節点は `Let(x, Match(s, arms), k)` であって `ρ` の上にあり、`ρ` は
        アーム `a` を通る。D3 より `ρ` は `a.body` の実行路を辿り、その終端は `a.body` の終端の `Ret` で
        ある。D2 より `Ret` は唯一の終端子であり `Ret` 以外の 5 種は継続を 1 つ持つので、`returned_var`
        が着く `Ret` はその終端の `Ret` であり、それが名指す変数が `r_0` である。DEF `ρ` の上で実行された辺
        の第 3 の場合より、E6 の辺は `ρ` の上で実行された。
    BY D2, D3, DEF `ρ` の上で実行された辺, CODE src/rc_ir/ownership.rs: returned_var, <1>6
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>8. QED
  BY <1>3, <1>6, <1>7

## L2 (E1-E5 の辺は `origin` を保つ)

**言明**。次の 5 つが成り立つ。

- **(E1)** E1 の辺 `(y, λ)`-`(x, λ)` について `origin(x, λ) = origin(y, λ)`。
- **(E2)** E2 の辺 `(c, [i] ++ λ)`-`(f, λ)` について `origin(f, λ) = origin(c, [i] ++ λ)`。
- **(E3)** E3 の辺 `(s, [t] ++ λ)`-`(p, λ)` について `origin(p, λ) = origin(s, [t] ++ λ)`。
- **(E4)** E4 の辺 `(s, λ)`-`(p, λ)` について `origin(p, λ) = origin(s, λ)`。
- **(E5)** E5 の辺 `(args[j], σ)`-`(x, λ)` について `origin(x, λ) = origin(args[j], σ)`。

<1>1. `collect_bindings` は、`Let(x, Var(y), k)` に対し `x` の `Binding` を `Move(y)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Var(y)` の場合

<1>2. `collect_bindings` は、`Destructure(container, fields, _state, k)` の各 `(idx, fv)` に対し `fv` の
      `Binding` を `Field(container, idx)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Destructure` の腕

<1>3. `collect_bindings` は、`Let(x, Match(scrut, arms), k)` の各アームの `payload` に対し `Binding` を
      `Payload(scrut, arm.tag)` とする。`arm.tag` は catch-all のとき `None`、変位アームのとき
      `Some(t)` である。
  BY D2, CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Match` の場合

<1>4. `collect_bindings` は、`Let(x, Llvm(llvm_gen, args), k)` に対し `x` の `Binding` を
      `Llvm(llvm_gen, args, x.ty)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Llvm` の場合

<1>4a. `origin(x, π)` の値は `origin_inner(vars, type_env, x, π)` の 1 回の呼び出しが返した値である。
      よって `origin_inner` の腕が返す式を読めば `origin` の値が決まる。
  BY L0

<1>5. (E1) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Move(y))` の腕は `origin(vars, type_env, &y.name, path)` を
        そのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕
  <2>2. QED
    BY <1>1, <1>4a, <2>1

<1>6. (E2) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Field(container, idx))` の腕は、`container.ty.is_box(type_env)`
        が偽のとき `container_path` を `[*idx] ++ path` として作り
        `origin(vars, type_env, &container.name, &container_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕
  <2>2. E2 の辺は `c` が unbox のときにだけ引かれるので、`container.ty.is_box(type_env)` は偽である。
    BY D9 (移動の表の unbox 容器の `Destructure` の行)
  <2>3. QED
    BY <1>2, <1>4a, <2>1, <2>2

<1>7. (E3) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の
        `Some(tag) if !scrut.ty.is_box(type_env)` の場合は、`scrut_path` を `[*tag] ++ path` として作り
        `origin(vars, type_env, &scrut.name, &scrut_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. E3 の辺は `s` が unbox union のときにだけ引かれるので、`scrut.ty.is_box(type_env)` は偽であり、
        `arm.tag` は `Some(t)` である。
    BY D9 (移動の表の unbox union の変位アームの行), <1>3
  <2>3. QED
    BY <1>3, <1>4a, <2>1, <2>2

<1>8. (E4) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の `None` の場合は
        `origin(vars, type_env, &scrut.name, path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. E4 の辺は catch-all アームについてだけ引かれるので、`arm.tag` は `None` である。
    BY D9 (移動の表の catch-all の行), <1>3
  <2>3. QED
    BY <1>3, <1>4a, <2>1, <2>2

<1>9. (E5) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕は、`decl` を
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` として作り、
        `decl.leaf_origins_at(path).and_then(as_arg_projection)` が `Some((j, p))` のとき
        `origin(vars, type_env, &args[j].name, &p)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕
  <2>2. `as_arg_projection(sources)` が `Some((j, p))` を返すのは、`sources` がちょうど 1 元からなり、
        その元が `LeafOrigin::Arg(j, p)` のときに限る。
    BY CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>3. E5 の辺は結果の leaf `λ` の宣言が単一の `Arg(j, σ)` のときにだけ引かれるので、
        `decl.leaf_origins_at(λ)` は 1 元集合 `{Arg(j, σ)}` であり、`as_arg_projection` はそれを
        `Some((j, σ))` に写す。
    BY D9 (移動の表の `Llvm` の素通し leaf の行), <2>2
  <2>4. QED
    BY <1>4, <1>4a, <2>1, <2>3

<1>10. QED
  BY <1>5, <1>6, <1>7, <1>8, <1>9

**補足 (E1-E5 に入らない `origin_inner` の腕)**。`origin_inner` が `here()` を返して値自身を origin と
するのは、`Binding` が `None` / `Param` / `Producer` のとき、`Field` で容器が boxed のとき、`Payload` で
scrutinee が boxed のとき、`Llvm` で `as_arg_projection` も `origin_from_leaves_under` も答えを出さない
ときである (`CODE src/rc_ir/ownership.rs: origin_inner`)。このうち `Param` は D10 の初期値、`Producer` は
D10 の生成の表の `App` と `Closure` の行 (`CODE src/rc_ir/ownership.rs: collect_bindings` がこの 2 つに
`Producer` を付ける)、boxed の `Field` と boxed の `Payload` は D10 の生成の表の対応する行、`Llvm` は
D10 の生成の表の `Llvm` の行に当たる。残る `Binding::Join` の腕は E6 の辺であり、L2 の言明はそれを外して
いる。

## L3 (boxed leaf は互いに前置にならない)

**言明**。型 `τ` について、`boxed_leaf_paths(τ, type_env)` の相異なる 2 元の一方が他方の前置になることは
ない。とくに `π` が leaf であるとき `L(v, π)` は `{π}` である。

<1>1. `boxed_leaf_paths` の内部の走査 `go(ty, type_env, path, out)` は、3 つの腕で `out` に path を積んだ
      直後に `return` する。`ty.is_closure()` の腕は `path ++ [CLOSURE_CAPTURE_IDX]` を積み、
      `ty.is_box(type_env)` の腕と `ty.is_array()` の腕は `path` を積む。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `go` が path を積むのはその 3 つの腕だけであり、`ty.is_fully_unboxed(type_env)` の腕は積まずに
      `return` し、最後の腕は `unpunched_field_types` が返す各 `(i, fty)` について `path ++ [i]` を
      引数として `go` を呼ぶ (他の再帰呼び出しは無い)。
  BY D4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2a. 引数 `path` が `q` である `go` の 1 回の呼び出しが、その再帰呼び出しを含めて `out` に積む path の
       列を `Push(q)` と書く。以下、path `a` が path `b` の**前置**であるとは `b` が `a` で始まることを
       いい、`a = b` の場合を含む。`<1>1` と `<1>2` より `Push(q)` は次のいずれかである。
       空の列 (`is_fully_unboxed` の腕)、`[q ++ [CLOSURE_CAPTURE_IDX]]` (`is_closure` の腕)、
       `[q]` (`is_box` の腕と `is_array` の腕)、`Push(q ++ [i])` を `i` について連ねた列 (最後の腕、
       `i` は `unpunched_field_types` が返す添字を渡る)。
  BY <1>1, <1>2

<1>2b. `Push(q)` の各成分は `q` を前置に持つ。
  A10 より型の in-place の降下は有限なので、`<1>2a` の再帰は整礎であり、それについての帰納法が使える。
  空の列には成分が無い。`[q ++ [CLOSURE_CAPTURE_IDX]]` と `[q]` の成分は `q` を前置に持つ。最後の場合、
  帰納法の仮定より `Push(q ++ [i])` の各成分は `q ++ [i]` を前置に持ち、`q` はその前置である。
  BY A10, <1>2a

<1>3. `Push(q)` の異なる 2 つの位置にある成分 `P`、`P'` について、`P` は `P'` の前置ではない。
      とくに (前置が等号を含むので) `P ≠ P'` であり、`Push(q)` の成分は互いに相異なる。
  `<1>2b` と同じ整礎な関係についての帰納法による。空の列と長さ 1 の列には異なる 2 つの位置が無い。
  最後の場合、同じ `i` の `Push(q ++ [i])` から来た 2 つについては帰納法の仮定であり、相異なる
  `i ≠ i'` から来た 2 つについては、`<1>2b` より一方は位置 `|q|` に `i` を、他方は `i'` を持つので、
  どちらも他方の前置ではない。
  BY A10, <1>2a, <1>2b

<1>3a. `boxed_leaf_paths(τ, type_env)` が返す列は `Push([])` である。
  `boxed_leaf_paths` は空の `path` と空の `out` で `go` を 1 度呼び、`out` を返す。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>4. QED
  `<1>3` と `<1>3a` より `boxed_leaf_paths(τ, type_env)` の成分は互いに相異なり、どの 2 つも一方が他方の
  前置ではない。よって `π` が leaf であるとき、`π` を前置に持つ leaf は `π` 自身だけであり、DEF `L` より
  `L(v, π) = {π}` である。
  BY DEF `L`, <1>3, <1>3a

## L4 (identity のスロット)

**言明**。`ρ` を実行路、`(x, λ)` を `ρ` のスロット (DEF 路のスロット) とする。`id(x, λ) = (w, σ)` と
おくと、次が成り立つ。

- (i) `σ` は `ty(w)` の boxed leaf であり、`(w, σ)` は `ρ` のスロットである。
- (ii) `obj(x, λ) = obj(w, σ)`。

証明は、`origin` が `(x, λ)` から行う再帰呼び出しの関係の上の帰納法による。P2 より `origin` は停止するので、
無限に降りる呼び出しの列は無く、この関係は整礎である。以下、帰納法の仮定を「IH」と書く。

<1>1. `origin(x, λ)` の値は `origin_inner(vars, type_env, x, λ)` の 1 回の呼び出しが返した値である。
  BY L0

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
        `Join` の元は各アーム本体の `returned_var` から取る。
    BY L1a, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var
  <2>2. 節点に現れる変数の使用は、その節点でスコープに入っている束縛に解決する。
    BY A11
  <2>3. D2 より、`Let` と `Destructure` が束縛する変数のスコープはその継続、`Match` のアームの payload の
        スコープはそのアーム本体であり、どちらも束縛の節点の子孫である。よって節点でスコープに入っている
        束縛は、その節点の祖先が束縛したものか、パラメータ・capture である。
    BY D2
  <2>4. D3 より実行路は根から辿るので、`ρ` が着いた節点の祖先はすべて `ρ` の上にある。パラメータと
        capture は本体の実行の前に値を得ている。
    BY D3, D10 (`Obl` の初期値はパラメータ・capture の leaf である)
  <2>5. `Join(rs)` の元のうち `ρ` が通ったアームのものは、L1a (b) の `Join` の行より、そのアーム本体の
        終端の `Ret` が名指す変数である。その `Ret` は `ρ` の上の節点なので、`<2>2` から `<2>4` をその
        位置に適用できる。
    BY D3, L1a, <2>1, <2>2, <2>3, <2>4
  <2>6. `vars.bindings` に無い名前は DEF 路のスロット の (S4) であり、路によらず値を得ている。
    BY DEF 路のスロット
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>4. `x` の `Binding` が `Llvm(gen, args, result_ty)` であるとき、`result_ty` は `ty(x)` であり、
      `decl := gen.result_prov(result_ty, &arg_tys, type_env)` について `decl.leaf_origins_at(λ)` は
      `Some(S)` である。
  <2>1. `collect_bindings` は `Let(x, Llvm(llvm_gen, args), k)` に `Binding::Llvm(llvm_gen, args, x.ty)` を
        作る。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Llvm` の場合
  <2>2. `result_prov` は結果の型の boxed leaf ごとに `LeafOrigins` を 1 つ宣言する。すなわち `decl` の
        鍵の全体は `boxed_leaf_paths(result_ty, type_env)` である。
    BY A3
  <2>3. `leaf_origins_at(π)` は、`π` に記録がある場合に `Some`、無い場合に `None` を返す。
    BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>4. QED
    `λ` は `ty(x)` の boxed leaf である (DEF 路のスロット)。`<2>1` より `result_ty = ty(x)` なので、
    `<2>2` より `λ` は `decl` の鍵であり、`<2>3` より `leaf_origins_at(λ)` は `Some` を返す。
    BY DEF 路のスロット, <2>1, <2>2, <2>3

<1>5. `<1>4` の `S` の元数は 0 か 1 である。
  A3 は「複数の元を宣言する op は、このコミットのプログラムには存在しない」と述べ、`result_prov` を
  override する 29 個が leaf に置く集合はすべて要素数 0 か 1 であるとする。`origin_inner` が読む `decl` は
  `llvm_gen.result_prov(..)` の返り値そのものであって、`Provenance::join` や `compose` を通していない。
  BY A3, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕

<1>5a. `id(x, λ) = (x, λ)` であるとき、(i) と (ii) が成り立つ。
  (i): 前提より `(x, λ)` は `ρ` のスロットであり、DEF 路のスロット より `λ` は `ty(x)` の boxed leaf で
  ある。(ii): `obj(x, λ) = obj(x, λ)` は等号の反射律である。
  BY DEF 路のスロット

<1>6. CASE `None | Some(Param) | Some(Producer)`。
  <2>1. 答えは `here()` すなわち `Exactly((x, λ))` であり、`id(x, λ) = (x, λ)` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner の
       `None | Some(Binding::Param) | Some(Binding::Producer)` の腕, Origin::identity
  <2>2. QED
    BY <1>5a, <2>1

<1>7. CASE `Some(Move(y))`。
  <2>1. `origin(x, λ) = origin(y, λ)` であり、よって `id(x, λ) = id(y, λ)` である。
    BY L2 (E1), <1>1
  <2>2. `(y, λ)` は `ρ` のスロットである。
    A12 の move-bind の行より `ty(y) = ty(x)` なので `λ` は `ty(y)` の boxed leaf である。D2 の `Let` の
    行より `x` が束縛される値は `Var(y)` の値、すなわち `y` の値であり、D16 の inhabited は値で決まるので
    `λ` は `y` の値でも inhabited である。`y` が `ρ` の上で値を得ていることは `<1>3` である。
    BY A12, D2, D16, <1>3
  <2>3. `obj(x, λ) = obj(y, λ)`。
    L1a (b) の `Move(y)` の行より、E1 の辺 `(y, λ)`-`(x, λ)` は `ρ` の上で実行された辺である。
    BY L1, L1a
  <2>4. QED
    IH を `(y, λ)` に適用すると、`id(y, λ) = (w, σ)` について (i) と `obj(y, λ) = obj(w, σ)` が出る。
    BY <2>1, <2>2, <2>3, IH

<1>8. CASE `Some(Field(c, i))` で `c` が boxed。
  <2>1. 答えは `here()` であり `id(x, λ) = (x, λ)` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕の
       `container.ty.is_box(type_env)` が真の枝, Origin::identity
  <2>2. QED
    BY <1>5a, <2>1

<1>9. CASE `Some(Field(c, i))` で `c` が unbox。
  <2>1. `origin(x, λ) = origin(c, [i] ++ λ)` であり、よって `id(x, λ) = id(c, [i] ++ λ)` である。
    BY L2 (E2), <1>1
  <2>2. フィールド `i` は `c` の値が保持するフィールドである。
    D9 の移動の表の unbox 容器の `Destructure` の行より、スロット `(x, λ)` が持つ参照は `c` のフィールド
    `i` の参照である。A5 より `c` の値が持つ参照は `ty(c)` の inhabited な boxed leaf にあり、D4 の
    規則 5 は穴のフィールドの下へ降りないので、`i` が穴なら `c` はフィールド `i` に参照を持たない。
    BY A5, D4, D9
  <2>3. `[i] ++ λ` は `ty(c)` の boxed leaf である。
    A12 の `Destructure` の行より `ty(x)` は `c` のフィールド `i` の型であり、D4 の規則 5 より unbox
    集約の leaf は `unpunched_field_types` が返す各フィールドの添字を前置した leaf である。`<2>2` より
    フィールド `i` はその中にある。
    BY A12, D4, <2>2
  <2>4. `[i] ++ λ` は `c` の値で inhabited である。
    D2 の `Destructure` の行より `x` の値は `c` の値の第 `i` フィールドなので、`c` の値の位置
    `[i] ++ λ` は `x` の値の位置 `λ` である。前提より `λ` は `x` の値で inhabited である。A12 の
    `Destructure` の行より `c` は構造体なので `[i]` は unbox union の節を通らず、`[i] ++ λ` が通る
    unbox union の節は `λ` が通る節と同じである。
    BY A12, D2, D16, <2>3
  <2>5. `(c, [i] ++ λ)` は `ρ` のスロットである。
    BY <1>3, <2>3, <2>4
  <2>6. `obj(x, λ) = obj(c, [i] ++ λ)`。
    L1a (b) の `Field(c, i)` の行より、E2 の辺 `(c, [i] ++ λ)`-`(x, λ)` は `ρ` の上で実行された辺である。
    BY L1, L1a
  <2>7. QED
    IH を `(c, [i] ++ λ)` に適用する。
    BY <2>1, <2>5, <2>6, IH

<1>10. CASE `Some(Payload(s, None))` (catch-all)。
  <2>1. `origin(x, λ) = origin(s, λ)` であり、よって `id(x, λ) = id(s, λ)` である。
    BY L2 (E4), <1>1
  <2>2. `(s, λ)` は `ρ` のスロットである。
    A12 の catch-all アームの payload と scrutinee の行より `ty(s) = ty(x)` なので `λ` は `ty(s)` の
    boxed leaf である。D9 の移動の表の catch-all の行より、payload 変数 `x` が得る値は scrutinee `s` の
    値である。よって `λ` は `s` の値でも inhabited である (D16)。`s` が値を得ていることは `<1>3` である。
    BY A12, D9, D16, <1>3
  <2>3. `obj(x, λ) = obj(s, λ)`。
    L1a (b) の `Payload(s, None)` の行より、E4 の辺 `(s, λ)`-`(x, λ)` は `ρ` の上で実行された辺である。
    BY L1, L1a
  <2>4. QED
    BY <2>1, <2>2, <2>3, IH

<1>11. CASE `Some(Payload(s, Some(t)))` で `s` が boxed。
  <2>1. 答えは `here()` であり `id(x, λ) = (x, λ)` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕の
       `Some(_)` の枝, Origin::identity
  <2>2. QED
    BY <1>5a, <2>1

<1>12. CASE `Some(Payload(s, Some(t)))` で `s` が unbox。
  <2>1. `origin(x, λ) = origin(s, [t] ++ λ)` であり、よって `id(x, λ) = id(s, [t] ++ λ)` である。
    BY L2 (E3), <1>1
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf である。
    A12 の payload と変位の行より `ty(s)` は union であり `ty(x)` はその変位 `t` の型である。D4 の
    規則 5 より union の leaf は各変位の payload の leaf に変位番号を前置したものである。
    BY A12, D4
  <2>3. この位置で `s` の値の実行時のタグは `t` である。
    L1a (b) の `Payload(s, Some(t))` の行より、`Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、
    `ρ` を辿る実行が選ぶアームの `tag` は `Some(t)` である。L1b がその位置での `s` の値の実行時のタグを
    `t` と定める。
    BY L1a, L1b
  <2>4. `[t] ++ λ` は `s` の値で inhabited である。
    `[t] ++ λ` が通る unbox union の節は、`ty(s)` の根の節 (`<2>3` よりタグは `t`) と、`λ` が通る節
    (前提より `λ` は inhabited) である。
    BY D16, <2>2, <2>3
  <2>5. `(s, [t] ++ λ)` は `ρ` のスロットである。
    BY <1>3, <2>2, <2>4
  <2>6. `obj(x, λ) = obj(s, [t] ++ λ)`。
    L1a (b) の `Payload(s, Some(t))` の行より、E3 の辺 `(s, [t] ++ λ)`-`(x, λ)` は `ρ` の上で実行された
    辺である。
    BY L1, L1a
  <2>7. QED
    IH を `(s, [t] ++ λ)` に適用する。
    BY <2>1, <2>5, <2>6, IH

<1>13. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` が単一の `Arg(j, σ')`。
  <2>1. `origin(x, λ) = origin(args[j], σ')` であり、よって `id(x, λ) = id(args[j], σ')` である。
    BY L2 (E5), <1>1
  <2>2. `σ'` は `ty(args[j])` の boxed leaf であり、`args[j]` の値で inhabited である。
    A3 の「単一の `Arg(j, σ)`」の行は、宣言が第 `j` オペランドの leaf `σ` を名指すこと、および結果の
    その leaf が inhabited であることと第 `j` オペランドの leaf `σ` が inhabited であることが同値である
    ことを述べる。前提より `λ` は inhabited である。
    BY A3
  <2>3. `(args[j], σ')` は `ρ` のスロットである。
    BY <1>3, <2>2
  <2>4. `obj(x, λ) = obj(args[j], σ')`。
    L1a (b) の `Llvm(gen, args, ty)` の行より、E5 の辺 `(args[j], σ')`-`(x, λ)` は `ρ` の上で実行された
    辺である。
    BY L1, L1a
  <2>5. QED
    BY <2>1, <2>3, <2>4, IH

<1>14. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` が空集合。
  A3 の空集合の行より、結果のその leaf は inhabited にならない。前提より `λ` は inhabited なので、この
  場合は起きない。
  BY A3, DEF 路のスロット

<1>15. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` が単一の `Fresh` または単一の `Unknown`。
  <2>1. `as_arg_projection(S)` は `None` を返すので、腕は
        `origin_from_leaves_under(vars, type_env, &decl, args, λ, &here_identity)` に進む。
        `here_identity` は `(x, λ)` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: as_arg_projection, origin_inner の
       `Some(Binding::Llvm(..))` の腕の `None =>` の枝
  <2>2. `decl.leaf_origins_under(λ)` が返すのは `S` 1 つだけである。
    `leaves_under(path)` は写像の鍵のうち `path` を前置に持つものの値を返す。`<1>4` より `decl` の鍵の
    全体は `boxed_leaf_paths(ty(x), type_env)` であり、`λ` はその 1 つで、その値は `S` である。L3 より
    `ty(x)` の boxed leaf のうち `λ` を前置に持つものは `λ` 自身だけである。
    BY L3, <1>4, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
  <2>3. `origin_from_leaves_under` は `Some(Origin::Exactly((x, λ)))` を返す。
    `S` の元は `Fresh` か `Unknown` なので、ループは `operand_units` に何も入れず `produced_here` を
    立てる。よって `reached` は `Exactly((x, λ))` 1 つだけからなり、`reached.iter().all(..)` の枝が
    その値を返す。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>4. QED
    `<2>3` と `Origin::identity` より `id(x, λ) = (x, λ)` である。
    BY <1>5a, <2>3, CODE src/rc_ir/ownership.rs: Origin::identity

<1>16. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` の元数が 2 以上。
  `<1>5` よりこの場合は起きない。
  BY <1>5

<1>17. CASE `Some(Join(arm_results))`。
  <2>1. 答えは `Origin::of_candidates(C, (x, λ))` である。ここで `C` は各 `r ∈ arm_results` についての
        `act(r, λ)` の合併である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕
  <2>2. `r_0` を `ρ` が通ったアームの結果とすると、`(r_0, λ)` は `ρ` のスロットであり
        `obj(x, λ) = obj(r_0, λ)` である。
    A12 のアームの結果と `Match` の束縛変数の行より両者の型は一致するので `λ` は `ty(r_0)` の boxed leaf
    である。D9 の移動の表の `Match` のアーム本体の `Ret(x)` の行より `x` が得る値は `r_0` の値なので、
    `λ` は `r_0` の値でも inhabited である (D16)。`r_0` が値を得ていることは `<1>3` である。L1a (b) の
    `Join(rs)` の行より E6 の辺 `(r_0, λ)`-`(x, λ)` は `ρ` の上で実行された辺なので、L1 よりオブジェクトが
    一致する。
    BY A12, D9, D16, L1, L1a, <1>3
  <2>2a. 任意の変数 `u` と path `μ` について `act(u, μ)` は `id(u, μ)` を含み、したがって空でない。
    `Origin::acted_on()` は `identity()` を先頭に置く列である。
    BY D15, CODE src/rc_ir/ownership.rs: Origin::acted_on
  <2>3. `C` は空でない。
    A9 よりアームは 1 つ以上あり、`<2>2a` より各 `act(r, λ)` は空でない。
    BY A9, <2>1, <2>2a
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
      BY L1a, <2>1, <2>2a
    <3>3. QED
      IH を `(r_0, λ)` に適用すると、(i) は `c` について成り立ち、`obj(r_0, λ) = obj(c)` が出る。
      `<2>2` と合わせて `obj(x, λ) = obj(c)` である。
      BY <2>2, <3>1, <3>2, IH
  <2>6. QED
    `<2>3` より `|C| ≥ 1` であり、`<2>4` と `<2>5` がその 2 つの場合を尽くす。
    BY <2>3, <2>4, <2>5

<1>18. QED
  `<1>2` の 6 本の腕のうち、`Field` は容器の boxed / unbox で `<1>8` と `<1>9` に、`Payload` は
  `variant` が `None` か `Some` か、`Some` のときの scrutinee の boxed / unbox で `<1>10`、`<1>11`、
  `<1>12` に分かれる。`Llvm` は `<1>4` の `S` の形で `<1>13` から `<1>16` に分かれる -- `S` は
  `LeafOrigin` の集合であり、`LeafOrigin` の構成子は `Fresh`、`Unknown`、`Arg(j, σ)` の 3 つなので、
  `<1>5` が「元数 2 以上」を消した後に残るのは、空集合、単一の `Arg`、単一の `Fresh`、単一の `Unknown` の
  4 つである。残りは `<1>6`、`<1>7`、`<1>17` である。
  BY <1>1, <1>2, <1>5, <1>6, <1>7, <1>8, <1>9, <1>10, <1>11, <1>12, <1>13, <1>14, <1>15, <1>16, <1>17,
     CODE src/rc_ir/provenance.rs: LeafOrigin

**補足 (切り詰めを通る枝に入らないこと)**。`origin_from_leaves_under` が `truncate_to_unit` を呼ぶのは
`LeafOrigin::Arg(j, leaf)` の元を `operand_units` に入れるときである
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。`<1>13` から `<1>16` より、`λ` が boxed leaf で
あるときにこの関数へ入るのは `S` が空集合か単一の `Fresh` / `Unknown` のときだけである。L3 より `ty(x)` の
boxed leaf のうち `λ` を前置に持つものは `λ` 自身だけなので、`decl.leaf_origins_under(λ)` が返す宣言は `S`
だけであり、どの場合も `Arg` の元を持たない。よって leaf の path から出た再帰は切り詰めを 1 度も通らない。

## P5 (a) -- 対の健全性

**言明** (README の P5 (a))。1 つの関数の 1 回の活性化について、1 つの実行路の 1 つの位置において `origin` の
`identity` が等しい 2 つの leaf のスロットは、同じオブジェクトを指す。

<1>1. 位置 `p` の 2 つのスロットを `(x, λ)`、`(y, μ)` とし、`id(x, λ) = id(y, μ) = (w, σ)` とする。この
      2 つはこの実行路 `ρ` のスロットである。
  BY D6, DEF 路のスロット

<1>2. `obj(x, λ) = obj(w, σ)` であり、`obj(y, μ) = obj(w, σ)` である。
  BY L4, <1>1

<1>3. QED
  BY <1>2

**補足 (この主張がどこで危ういか)**。`identity` が 2 つの leaf の間で潰れるのは、`origin` の再帰が
`origin_from_leaves_under` の切り詰めを通るときである。`Std::Option (a, b)` のように 1 つの unit
(unbox union) の下に 2 つの boxed leaf を持つ値では、その 2 つの leaf が別々のオブジェクトに届きながら、
切り詰めた先が同じ unit path になりうる。`L4 の補足` が、スロットの path が boxed leaf である限りこの枝に
入らないことを述べる。この結論は A3 の「複数の元を宣言する op は存在しない」に載っている。第 9 節に
書き出す。

## P5 (b) -- 対の有効性

**言明** (README の P5 (b))。同じオブジェクトを指す 2 つの leaf のスロットで、一方から他方への別名の道
(D20) が `Match` のアーム本体の `Ret` の辺を含まないならば、両者の `identity` は等しい。

<1>1. 道の各辺は E1 から E5 のいずれかである。
  `Match` のアーム本体の `Ret` の辺は E6 であり、D20 の別名の辺は E1 から E6 である。
  BY D20, 前提

<1>2. 道の各辺の両端のスロットは同じ `origin` を持つ。
  BY L2, <1>1

<1>3. 道の両端のスロットは同じ `origin` を持つ。
  BY <1>2 (等号の推移律)

<1>4. QED
  `Origin::identity` は `Origin` の関数なので、等しい `origin` は等しい `identity` を持つ。
  BY <1>3, CODE src/rc_ir/ownership.rs: Origin::identity

**補足 (前提のうち使っていないもの)**。この証明は前提の「同じオブジェクトを指す」を使わない。E1 から E5 の
別名の道で結ばれていれば、オブジェクトが同じかどうかを問わず `identity` は等しい。

## R1 (E6 を除く限定が外せないこと)

**言明**。同じオブジェクトを指す 2 つのスロットで、`identity` が異なるものがある。さらに、その 2 つは
別名の道 (D20) で結ばれており、結ぶ道はすべて E6 の辺を含む。よって P5 (b) の「別名の道が `Match` の
アーム本体の `Ret` の辺を含まない」という限定は外せない。

<1>1. 次の関数 `f` を考える。`T` は boxed な型、`Bool` は 2 つの変位を持つ unbox union で、その 2 つの
      変位の payload の型はどちらも `is_fully_unboxed` が真であるとする。パラメータは `c : Bool`、
      `x : T`、`y : T` の 3 つで、`borrowed_units` は空 (A1) である。本体は次のとおりで、`m : T`、
      `p0`・`p1` は `Bool` の変位 0・1 の payload の型を持つ。

      ```
      Let(m, Match(c, [ arm(tag=0, payload=p0, body=B0), arm(tag=1, payload=p1, body=B1) ]), Ret(m))
      B0 = Release(y, [], s, Ret(x))
      B1 = Release(x, [], s, Ret(y))
      ```

      これは D2 の形の本体であり、A12 (アームの結果と `Match` の束縛変数の型、payload と変位の型) と
      H1 (アームが `Bool` の 2 変位を尽くす) を満たす。
  BY A12, D2, H1

<1>2. `boxed_leaf_paths(T, type_env)` は `{[]}` であり、`[]` は inhabited である。`rc_units(T, type_env)` は
      `{[]}` である。`p0` と `p1` の型の `boxed_leaf_paths` は空であり、`boxed_leaf_paths(Bool, type_env)`
      も空である。
  <2>1. `T` は boxed なので、`boxed_leaf_paths` の 3 番目の規則により自分自身の位置 1 つが leaf である。
    BY D4
  <2>2. `T` は unbox union を通らないので `[]` は inhabited である。
    BY D16
  <2>3. `T` は boxed なので `unit_step` は `Unit` を返し、`rc_units` は `[]` を積む。
    BY D5
  <2>4. `<1>1` より `Bool` の 2 つの変位の payload の型はどちらも `is_fully_unboxed` が真なので、D4 の
        規則 1 よりどちらも leaf を持たない。D4 の規則 5 より union の leaf は各変位の payload の leaf に
        変位番号を前置したものなので、`boxed_leaf_paths(Bool, type_env)` も空である。
    BY D4, <1>1
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4

<1>3. `f` の活性化の `Obl` の初期値は `{r_x, r_y}` である。ここで `r_x` は `obj(x, [])` への参照、`r_y` は
      `obj(y, [])` への参照である。
  D14 と A1 よりパラメータの unit はすべて所有され、`<1>2` より `x` と `y` の inhabited な leaf は `[]`
  だけで、`c` は leaf を持たない。
  BY A1, D10, D14, <1>2

<1>4. 変位 0 のアームを選ぶ実行路について、アーム本体の `Ret(x)` の直後の位置 (関数本体の終端の `Ret(m)` の
      位置) で `Obl` は `{r_x}` である。
  <2>1. `Release(y, [], s, ...)` は `Obl` から `obj(y, [])` への参照を 1 つ取り除く。`<1>3` の `Obl` が持つ
        `obj(y, [])` への参照は `r_y` だけなので、取り除かれるのは `r_y` である。
    BY D10, <1>2, <1>3
  <2>2. アーム本体の `Ret(x)` は移動なので `Obl` を変えない。
    BY D9, D10
  <2>3. QED
    BY <1>3, <2>1, <2>2

<1>5. 変位 1 のアームを選ぶ実行路について、同じ位置で `Obl` は `{r_y}` である。
  <2>1. `Release(x, [], s, ...)` は `Obl` から `obj(x, [])` への参照を 1 つ取り除く。`<1>3` の `Obl` が持つ
        `obj(x, [])` への参照は `r_x` だけなので、取り除かれるのは `r_x` である。
    BY D10, <1>2, <1>3
  <2>2. アーム本体の `Ret(y)` は移動なので `Obl` を変えない。
    BY D9, D10
  <2>3. QED
    BY <1>3, <2>1, <2>2

<1>6. 変位 0 のアームを選ぶ実行路について、`(x, [])` と `(m, [])` はどちらもその路のスロットであり、
      同じオブジェクトを指す。
  <2>1. `x` はパラメータなので DEF 路のスロット の (S1) で値を得ており、`m` は `ρ` の上の節点
        `Let(m, Match(c, ...), Ret(m))` が束縛するので (S2) で値を得ている。`<1>1` より `ty(x) = ty(m) = T`
        であり、`<1>2` より `[]` は `T` の inhabited な boxed leaf なので、どちらも `ρ` のスロットである。
    BY DEF 路のスロット, <1>1, <1>2
  <2>2. アーム本体の `Ret(x)` は `ρ` の上にあるので、E6 の辺 `(x, [])`-`(m, [])` は `ρ` の上で実行された
        辺である。
    BY DEF `ρ` の上で実行された辺, <1>1
  <2>3. QED
    BY L1, <2>1, <2>2

<1>7. `f` の本体は D11 の意味で RC 規律を満たす。
  <2>1. CASE 変位 0 のアームを選ぶ実行路。
    <3>1. 終端の `Ret(m)` が消費するのは `m` の leaf `[]` の参照であり、それは `r_x` である。
      `<1>6` より `(m, [])` は `(x, [])` と同じオブジェクトを指し、`<1>4` の `Obl` が持つそのオブジェクトへの
      参照は `r_x` だけである。A5 よりスロットの参照は 1 つである。
      BY A5, D9, <1>4, <1>6
    <3>2. (S-a) が成り立つ。参照を取り除く操作は `Release(y, [], s, ...)` と終端の `Ret(m)` の消費の
          2 つである。前者が取り除く `r_y` は `<1>3` の `Obl` にあり、後者が取り除く `r_x` は `<1>4` の
          `Obl` にある。
      BY D9, D10, D11, <1>3, <1>4, <3>1
    <3>3. (S-b) が成り立つ。終端の `Ret(m)` の消費が `<1>4` の `Obl` から `r_x` を取り除くと、`Obl` は
          空になる。
      BY D9, D10, D11, <1>4, <3>1
    <3>4. (S-c) が成り立つ。この実行路にある読む構文は `Let(m, Match(c, ...), Ret(m))` の `Match` だけで、
          読まれるのは `c` である。`c` は leaf を持たないので読まれるオブジェクトは無い。
      BY D7, D11, <1>2
    <3>5. QED
      BY <3>2, <3>3, <3>4
  <2>2. CASE 変位 1 のアームを選ぶ実行路。
    <3>1. アーム本体の `Ret(y)` は `ρ` の上にあるので E6 の辺 `(y, [])`-`(m, [])` は実行された辺であり、
          L1 より `(m, [])` は `(y, [])` と同じオブジェクトを指す。よって終端の `Ret(m)` が消費するのは
          `r_y` である。
      BY A5, D9, L1, DEF 路のスロット, DEF `ρ` の上で実行された辺, <1>1, <1>2, <1>5
    <3>2. (S-a) が成り立つ。参照を取り除く操作は `Release(x, [], s, ...)` と終端の `Ret(m)` の消費の
          2 つである。前者が取り除く `r_x` は `<1>3` の `Obl` にあり、後者が取り除く `r_y` は `<1>5` の
          `Obl` にある。
      BY D9, D10, D11, <1>3, <1>5, <3>1
    <3>3. (S-b) が成り立つ。終端の `Ret(m)` の消費が `<1>5` の `Obl` から `r_y` を取り除くと、`Obl` は
          空になる。
      BY D9, D10, D11, <1>5, <3>1
    <3>4. (S-c) が成り立つ。読む構文は `Match` だけで、`c` は leaf を持たない。
      BY D7, D11, <1>2
    <3>5. QED
      BY <3>2, <3>3, <3>4
  <2>3. QED
    D3 より実行路はアームの選び方で尽くされ、アームは 2 つである。
    BY D3, <2>1, <2>2

<1>7a. `origin(x, π)` の値は `origin_inner(vars, type_env, x, π)` の 1 回の呼び出しが返した値である。
  BY L0

<1>8. パラメータ `q` と path `μ` について `origin(q, μ) = Exactly((q, μ))` であり `id(q, μ) = (q, μ)` で
      ある。とくに `id(x, []) = (x, [])` である。
  `VarTable::of` はパラメータの `Binding` を `Param` とし、`origin_inner` の
  `None | Some(Binding::Param) | Some(Binding::Producer)` の腕は `here()` すなわち `Exactly((q, μ))` を
  返す。
  BY <1>7a, CODE src/rc_ir/ownership.rs: VarTable::of, origin_inner, Origin::identity

<1>9. `id(m, []) = (m, [])` である。
  <2>1. `m` の `Binding` は `Join([x, y])` である。`x` と `y` はそれぞれのアーム本体の `returned_var` で
        ある。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Match` の場合,
       returned_var
  <2>2. `act(x, []) = {(x, [])}`、`act(y, []) = {(y, [])}` である。
    `x` と `y` はどちらもパラメータなので、`<1>8` より `origin` は `Exactly((x, []))`、`Exactly((y, []))`
    を返す。`Exactly(p)` の `acted_on()` は `[p]` である。
    BY <1>8, CODE src/rc_ir/ownership.rs: Origin::acted_on, Origin::candidates
  <2>3. `origin_inner` の `Join` の腕が集める候補集合は `{(x, []), (y, [])}` であり、A6 より 2 元である。
        よって `of_candidates` は `Join { identity: (m, []), .. }` を返す。
    BY A6, <1>7a, <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_inner の
       `Some(Binding::Join(arm_results))` の腕, Origin::of_candidates
  <2>4. QED
    BY <2>3, CODE src/rc_ir/ownership.rs: Origin::identity

<1>9a. `f` の本体が持つ別名の辺は、E6 の辺 `(x, [])`-`(m, [])` と `(y, [])`-`(m, [])` の 2 本だけである。
  <2>1. E1 の辺は `Let(_, Var(_), _)` の節点を要し、E2 の辺は `Destructure` の節点を要し、E5 の辺は
        `Let(_, Llvm(..), _)` の節点を要する。`<1>1` の本体はこの 3 種の節点を持たない。
    BY D20, <1>1
  <2>2. E4 の辺は catch-all アームを要する。`<1>1` の 2 つのアームはどちらも `tag` を持つので、E4 の辺は
        無い。E3 の辺は payload 変数 `p0` / `p1` のスロットを端に持つが、`<1>2` より `p0` と `p1` の型は
        boxed leaf を持たないので、この 2 つを端とするスロットは無く、辺も無い。
    BY D20, DEF 辺の leaf 対応, <1>1, <1>2
  <2>3. E6 の辺は 2 つのアーム本体の終端の `Ret` から来る。`<1>2` より `T` の boxed leaf は `[]` だけ
        なので、その辺は `(x, [])`-`(m, [])` と `(y, [])`-`(m, [])` の 2 本である。
    BY D20, <1>1, <1>2
  <2>4. QED
    BY D20, <2>1, <2>2, <2>3

<1>10. QED
  `<1>7` より `f` の本体は RC 規律を満たし、`<1>6` より変位 0 の実行路の `Ret(m)` の位置において
  `(x, [])` と `(m, [])` は同じオブジェクトを指す。`<1>8` と `<1>9` より `id(x, []) = (x, [])`、
  `id(m, []) = (m, [])` であって、A6 よりこの 2 つの `VarPath` は異なる。`<1>9a` より本体の別名の辺は
  E6 の 2 本だけなので、`(x, [])` と `(m, [])` は別名の道 (E6 の辺 1 本) で結ばれており、この 2 つを
  結ぶどの道も E6 の辺だけからなる。
  BY A6, D20, <1>6, <1>7, <1>8, <1>9, <1>9a

## P5 (c) -- 被覆

**言明** (README の P5 (c))。`Release(v, π)` の走査が `un_bump` と `consume_objects` に渡すオブジェクトの
和 -- すなわち `ActRefs(v, π).objects()` と `other_objects(v, π)` の和 -- は、`π` の下の各 boxed leaf `λ` に
ついて `origin(v, λ).acted_on()` をすべて含む。

<1>1. `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕は、`other_objects(v, path)` を
      `consume_objects` に渡し、`acted_references(v, path)` を `un_bump` に渡す。`un_bump` が読むのは
      その `References` が名指すオブジェクト、すなわち `ActRefs(v, π).objects()` である。
  <2>1. 腕は `let others = self.other_objects(v, path); self.consume_objects(&mut pending, &others);` の
        のち `let un_bumped = self.acted_references(v, path);` を `un_bump(&mut pending, &un_bumped)` に
        渡す。`UnBump::OutsideBracket` の枝はさらに `un_bumped.objects()` を `consume_objects` に渡す。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕
  <2>2. `CancelAnalysis::acted_references(v, path)` は `ownership::acted_references(vars, type_env, v, path)`
        の値である (空でないことを表明するほかに何もしない)。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references
  <2>3. `un_bump(pending, un_bumped)` が `un_bumped` から読むのは `shares_an_object`、`covers`、
        `subtract` であり、いずれも `un_bumped` が名指すオブジェクトについての演算である。
    BY D15, CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/ownership.rs: References
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>1a. `boxed_leaf_paths(ty(v), type_env)` の要素のうち `leaf.starts_with(π)` を満たすものの全体は
       `L(v, π)` である。
  BY D4, DEF `L`

<1>2. `ActRefs(v, π).objects()` は `{ id(v, λ) : λ ∈ L(v, π) }` である。
  <2>1. `acted_references(v, π)` は、`boxed_leaf_paths(ty(v), type_env)` の要素 `leaf` のうち
        `leaf.starts_with(π)` を満たすものについて、`origin(v, leaf).identity()` をキーとする計数を
        1 ずつ増やした `Map<VarPath, usize>` を `References` に包んで返す。
    BY D15, CODE src/rc_ir/ownership.rs: acted_references
  <2>2. `References::objects()` はその `Map` のキーの列を返す。
    BY D15, CODE src/rc_ir/ownership.rs: References::objects
  <2>3. QED
    BY <1>1a, <2>1, <2>2

<1>3. `other_objects(v, π)` は `∪_{λ ∈ L(v, π)} (cand(v, λ) \ {id(v, λ)})` を含む。
  `other_objects` は `boxed_leaf_paths(ty(v), type_env)` のうち `leaf.starts_with(path)` を満たす各 `leaf`
  について `where_from = origin(v, leaf)` を取り、その `candidates()` のうち `identity()` と異なるものを
  すべて `out` に積む。回る `leaf` の全体は `<1>1a` より `L(v, π)` である。
  BY <1>1a, CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects

<1>4. 各 `λ` について `act(v, λ) = {id(v, λ)} ∪ (cand(v, λ) \ {id(v, λ)})` である。
  `Origin::acted_on()` は `identity()` を先頭に、`candidates()` のうち `identity()` と異なるものを続けた
  列である。
  BY D15, CODE src/rc_ir/ownership.rs: Origin::acted_on

<1>5. QED
  `<1>4` より `∪_{λ ∈ L(v, π)} act(v, λ) = { id(v, λ) : λ ∈ L(v, π) } ∪
  ∪_{λ ∈ L(v, π)} (cand(v, λ) \ {id(v, λ)})` であり、第 1 項は `<1>2` より `ActRefs(v, π).objects()` に
  等しく、第 2 項は `<1>3` より `other_objects(v, π)` に含まれる。`<1>1` より、この 2 つが走査の
  `un_bump` と `consume_objects` に渡るオブジェクトである。
  BY <1>1, <1>2, <1>3, <1>4

**補足 (逆向きも成り立つ)**。`<1>2` と `<1>3` はどちらも等号で成り立つので、和はちょうど
`∪_{λ ∈ L(v, π)} act(v, λ)` である。P5 (c) が包含だけを述べるのは、読み手 (P7c、P19) が要るのが包含の
向きだからである。

## P6 (`acted_references` は静的な上位近似である)

P6 の 2 つの主張を次のように書く。

- **(a)** `acted_references(v, π)` が返す `Map` は、`L(v, π)` のすべての元 `λ` を `id(v, λ)` で名付けて
  数えた多重集合である。
- **(b)** 位置 `p` において `Retain(v, π)` が実行時に作る参照の多重集合は、(a) の数え上げを
  `Linh(v, π, p)` に制限したものに等しい。`Release(v, π)` が実行時に処分する参照の多重集合も同じものに
  等しい。

(b) の 2 つの辺は住む集合が違う。D8 より参照の多重集合は**オブジェクトごと**の個数であり、(a) の数え上げは
`VarPath` ごとの個数である。この 2 つを比べる写像を先に定める。

**DEF 名前の指すオブジェクト**
`ρ` のスロット `(w, σ)` に対し `ν(w, σ) := obj(w, σ)` と置く。DEF 路のスロット より `obj(w, σ)` は `ρ` の
上の位置に依らないので、`ν` は「`ρ` のスロットである `VarPath`」の集合からオブジェクトの集合への写像として
定まる。`VarPath` の多重集合 `M` の各元が `ρ` のスロットであるとき、`ν` による**押し出し** `ν_*M` を、
各オブジェクト `o` について `(ν_*M)(o) := Σ_{n : ν(n) = o} M(n)` で定める。

(b) の「等しい」は、右辺 (`VarPath` の多重集合) を `ν` で押し出したものが、左辺 (オブジェクトごとの個数) に
等しいことである。

### P6 (a)

<1>1. `acted_references(v, path)` は、`boxed_leaf_paths(&v.ty, type_env)` の要素 `leaf` のうち
      `leaf.starts_with(path)` を満たすものについて、`origin(vars, type_env, &v.name, &leaf).identity()` を
      キーとする計数を 1 ずつ増やした `Map<VarPath, usize>` を返す。
  BY CODE src/rc_ir/ownership.rs: acted_references

<1>2. `<1>1` の走査が回る `leaf` の全体は `L(v, π)` である。inhabited かどうかは判定していない。
  BY D4, D15, DEF `L`, <1>1

<1>3. QED
  `<1>1` と `<1>2` より、返り値は `L(v, π)` の各元 `λ` を `id(v, λ)` で名付けて数えた多重集合である。
  BY <1>1, <1>2

### P6 (b)

<1>1. `Retain(v, π)` が `Obl` に加える参照は、`Linh(v, π, p)` の各 `λ` について `obj(v, λ)` への参照
      1 つずつであり、それが全部である。
  <2>1. D10 の `Retain` の行は「`π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ
        加える」である。
    BY D10
  <2>2. 「`π` の下の inhabited な leaf」の全体は `Linh(v, π, p)` である。
    BY D4, D16, DEF `L`, DEF `Linh`
  <2>3. QED
    BY <2>1, <2>2

<1>2. `Release(v, π)` が `Obl` から取り除く参照は、`Linh(v, π, p)` の各 `λ` について `obj(v, λ)` への
      参照 1 つずつであり、それが全部である。
  <2>1. D10 の `Release` の行は「`π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ
        取り除く」である。
    BY D10
  <2>2. 「`π` の下の inhabited な leaf」の全体は `Linh(v, π, p)` である。
    BY D4, D16, DEF `L`, DEF `Linh`
  <2>3. QED
    BY <2>1, <2>2

<1>2a. `Linh(v, π, p)` の各 `λ` について `(v, λ)` は `ρ` のスロットである。
  DEF `Linh` より `λ` は `ty(v)` の boxed leaf であって `v` の値で inhabited であり、`v` は位置 `p` までに
  値を得ているので `ρ` の上で値を得ている。
  BY DEF 路のスロット, DEF `Linh`

<1>3. `Retain(v, π)` が作る参照の、オブジェクトごとの多重集合は、各オブジェクト `o` について
      `#{λ ∈ Linh(v, π, p) : obj(v, λ) = o}` である。`Release(v, π)` が処分する参照の多重集合も同じで
      ある。
  D8 より、同じオブジェクトへの参照どうしは互いに区別されず、参照の多重集合はオブジェクトごとの個数と
  して読む。`<1>1` と `<1>2` は `λ` ごとに `obj(v, λ)` への参照を 1 つずつ数えている。
  BY D8, <1>1, <1>2

<1>4. 各 `λ ∈ Linh(v, π, p)` について、`id(v, λ)` は `ρ` のスロットであり、`ν(id(v, λ)) = obj(v, λ)` で
      ある。
  `<1>2a` より `(v, λ)` は `ρ` のスロットなので L4 が使える。L4 の (i) が `id(v, λ)` が `ρ` のスロットで
  あることを、(ii) が `obj(v, λ) = obj(id(v, λ))` を与え、DEF 名前の指すオブジェクト より
  `obj(id(v, λ)) = ν(id(v, λ))` である。
  BY L4, DEF 名前の指すオブジェクト, <1>2a

<1>5. P6 (a) の数え上げを `Linh(v, π, p)` に制限した多重集合を `M := Σ_{λ ∈ Linh(v, π, p)} [id(v, λ)]` と
      おくと、その押し出しは各オブジェクト `o` について
      `(ν_*M)(o) = #{λ ∈ Linh(v, π, p) : obj(v, λ) = o}` である。
  P6 (a) より `M` は `Linh(v, π, p)` の各元 `λ` を `id(v, λ)` で名付けて数えた多重集合である。`<1>4` より
  `M` の各元は `ρ` のスロットなので押し出しが定まり、その定義より
  `(ν_*M)(o) = #{λ ∈ Linh(v, π, p) : ν(id(v, λ)) = o}` である。`<1>4` の
  `ν(id(v, λ)) = obj(v, λ)` を代入する。
  BY P6 (a), DEF 名前の指すオブジェクト, <1>4

<1>6. QED
  `<1>3` の 2 つの多重集合と `<1>5` の `ν_*M` は、どのオブジェクト `o` についても同じ値
  `#{λ ∈ Linh(v, π, p) : obj(v, λ) = o}` を取る。
  BY <1>3, <1>5

### P6 の結論

<1>1. (a) は P6 (a) の言明である。
  BY P6 (a)
<1>2. (b) は P6 (b) の言明である。
  BY P6 (b)
<1>3. QED
  BY <1>1, <1>2

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
比べる読み手 -- `un_bump` の `covers` -- が R1 の形をどう扱うかは P16 から P19 が扱う。

**補足 3 (2 つの leaf が 1 つの名前を持つとき)**。`L(v, π)` の相異なる 2 つの leaf が同じ `id` を持つことが
ある。`Map` はそのとき計数を 2 にする (P6 (a))。これは 1 つのオブジェクトへの参照を 2 つ持つ値に
対応し、A5 の下で参照は inhabited な leaf ごとに 1 つなので、参照としても 2 つある。

**補足 4 (上位近似のずれは片側だけである)**。`acted_references` は `L(v, π)` を数え、実行時に触れるのは
`Linh(v, π, p)` である。`Linh(v, π, p)` は `L(v, π)` の部分集合なので、`References` の数はつねに実行時に
触れる参照の数以上である。この差を読むのは `un_bump` の `covers` と `subtract`、`consume_objects` の
`names`、`merge` の `References` の等号である (`CODE src/rc_ir/borrow.rs: un_bump`,
`CancelAnalysis::consume_objects`, `CancelAnalysis::merge`)。差が対の判定にどう効くかは P16 から P19 が
扱う。

## P7 (消費の網羅性)

P7 の 2 つの主張を次のように書く。

- **(a)** D9 の消費の表の各行が指す leaf は、`collect_consumes` が `out` に積む。
- **(b)** `collect_consumes` が `out` に積むもののうち D9 の消費の表に無いものは、`Match` のアーム本体の
  終端の `Ret` が積むものに限る。

これに補題 L6 を添える。README の P7 の言明には無いが、報告しない箇所が参照の収支を狂わせないことを
述べるので、ここで併せて示す。

`collect_consumes` は `own` 引数を取り、`owns(p, λ)` を `own.contains(&(p.name, λ))` として使う
(`CODE src/rc_ir/ownership.rs: collect_consumes`)。D9 の `App` の行が言う所有は D14 の unit 粒度の所有
なので、P7 は次の `own` について述べる。

**DEF leaf 粒度の所有**
`own` は、`(p, λ)` を含むことが「`p` の unit `truncate_to_unit(ty(p), λ, type_env)` が D14 の意味で
所有される」ことと同値であるような集合とする。P1 より `truncate_to_unit(ty(p), λ, type_env)` は
`rc_units(ty(p))` の要素なので、この対応は定義できる。

`CancelAnalysis::consume_rhs` が作る `owns` は、まさに `owned_units` への `truncate_to_unit` 経由の所属で
ある (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs`)。`infer_ownership` が渡す `owned_leaves` は
不動点計算の途中の集合であり、それが DEF leaf 粒度の所有 に一致するかどうかは P8 が扱う。

### L5 (`collect_consumes` が積むもの)

**言明**。次が成り立つ。

- **(a)** `collect_consumes` は `owns` を作って、渡された本体について `collect_consumes_go` を呼ぶだけで
  ある。
- **(b)** `collect_consumes_go` の走査は、渡された本体の各節点をちょうど 1 度訪れる。`collect_consumes_go`
  が呼ばれる本体は、`collect_consumes` が渡した本体と、`Let` の腕が `RcRhs::Match` のときに渡す各
  `arm.body` である。また `collect_consumes` を呼ぶのは `infer_ownership` の 1 か所だけで、そこで渡すのは
  `func.body` である。
- **(c)** `RcExpr::Ret(x)` の腕は、`boxed_leaf_paths(ty(x), type_env)` の各 `p` について `(x.name, p)` を
  積む。
- **(d)** `RcExpr::Destructure(c, fs, _, k)` の腕は、`destructure_consumes(c, fs, type_env)` の各 `leaf` に
  ついて `(c.name, leaf)` を積む。
- **(e)** `destructure_consumes` は、`c.ty.is_box(type_env)` が真のとき `boxed_leaf_paths(ty(c), type_env)`
  の全部を返し、偽のときはそのうち先頭の添字が `fs` の名前付きフィールドの添字でないものだけを返す。
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
  `prog.funcs` にも無いとき、すなわち間接呼び出しのときである。
- **(k)** `rhs_consumes` の `RcRhs::Llvm(llvm_gen, args)` の腕は、`llvm_gen.borrows_operand(i, ..)` が真の
  オペランドを飛ばし、それ以外の各オペランド `args[i]` の各 boxed leaf `leaf` について、
  `passthrough_arg_leaves(llvm_gen, result_ty, args, type_env)` が `(i, leaf)` を含まないときだけ
  `(args[i].name, leaf)` を積む。
- **(l)** `passthrough_arg_leaves` は「結果のある leaf の宣言が単一の `Arg(j, p)` である」ような
  `(j, p)` の集合である。
- **(m)** 積まれるものの出どころは (c)、(d)、(h)、(i)、(k) の 5 か所で全部である。

<1>1. (a) が成り立つ。`collect_consumes` は `owns` を作って `collect_consumes_go` を呼ぶだけである。
  BY CODE src/rc_ir/ownership.rs: collect_consumes

<1>2. `RcExpr` は `Let`、`Retain`、`Release`、`Destructure`、`Eval`、`Ret` の 6 種である。
  BY D2, CODE src/rc_ir/ast.rs: RcExpr

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
  D2 より本体は木であり、`Ret` を除く 5 種の節点の継続は 1 つ、分岐は `Match` のアームだけである。`<1>4` と
  `<1>5` はその継続とアームをちょうど 1 度ずつたどり、`<1>6` が終端である。`collect_consumes_go` が
  呼ばれる本体は `<1>1` が渡す本体と `<1>5` が渡す各 `arm.body` である。`collect_consumes` を呼ぶのは
  `infer_ownership` の 1 か所だけであり、そこで渡すのは `func.body` である。
  BY D1, D2, D3, <1>1, <1>3, <1>4, <1>5, <1>6, CODE src/rc_ir/borrow.rs: infer_ownership

<1>8. (c) が成り立つ。`RcExpr::Ret(x)` の腕は `push_boxed_leaves(&x.name, &x.ty, type_env, out)` を呼び、
      `boxed_leaf_paths(x.ty, type_env)` の各 `p` について `(x.name, p)` を積む。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Ret(x)` の腕, push_boxed_leaves

<1>9. (d) が成り立つ。`RcExpr::Destructure(container, fields, _state, k)` の腕は
      `destructure_consumes(container, fields, type_env)` の各 `leaf` について `(container.name, leaf)` を
      積む。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Destructure` の腕

<1>10. (e) が成り立つ。`destructure_consumes` は、`container.ty.is_box(type_env)` が真のとき
       `boxed_leaf_paths(container.ty, type_env)` の全部を返し、偽のときはそのうち先頭の添字が `fields` の
       名前付きフィールドの添字でないものだけを返す。
  BY CODE src/rc_ir/ownership.rs: destructure_consumes

<1>11. (f) の前半が成り立つ。`RcExpr::Retain | RcExpr::Release | RcExpr::Eval` の腕は何も積まない。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の
     `RcExpr::Retain(..) | RcExpr::Release(..) | RcExpr::Eval(..)` の腕

<1>12. (f) の後半が成り立つ。`RcExpr::Let(x, rhs, k)` の腕は、`rhs` が `RcRhs::Match` のとき自身は何も
       積まず、それ以外の 4 種のとき `rhs_consumes(rhs, &x.ty, vars, prog, type_env, owns, out)` を呼ぶ。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Let` の腕

<1>13. `RcRhs` は `Var`、`App`、`Closure`、`Llvm`、`Match` の 5 種である。
  BY D2, CODE src/rc_ir/ast.rs: RcRhs

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
  BY A14, CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App(callee, args)` の腕, push_boxed_leaves

<1>18. (j) が成り立つ。`resolve_callee_params` が `None` を返すのは、`callee.name` が
       `vars.closure_targets` にも `prog.funcs` にも無いとき、すなわち間接呼び出しのときである。
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

<1>21. QED
  (a) は `<1>1`、(b) は `<1>7`、(c) は `<1>8`、(d) は `<1>9`、(e) は `<1>10`、(f) は `<1>11` と `<1>12`、
  (g) は `<1>15`、(h) は `<1>16`、(i) は `<1>17`、(j) は `<1>18`、(k) は `<1>19`、(l) は `<1>20` で
  ある。(m) は次のとおりである。`<1>3` の 4 つの腕のうち積むのは `Ret` (`<1>8`) と `Destructure`
  (`<1>9`) であり、`Retain | Release | Eval` は積まず (`<1>11`)、`Let` は自身では積まずに `rhs_consumes`
  を呼ぶ (`<1>12`)。`<1>14` の 4 つの腕のうち積むのは `Closure` (`<1>16`)、`App` (`<1>17`)、`Llvm`
  (`<1>19`) であり、`Var | Match` は積まない (`<1>15`)。
  BY <1>1, <1>3, <1>7, <1>8, <1>9, <1>10, <1>11, <1>12, <1>14, <1>15, <1>16, <1>17, <1>18, <1>19, <1>20

### P7 (a) D9 の消費はすべて報告される

<1>1. CASE D9 の `App(callee, args)` の行。
  <2>1. `App` は `RcRhs` の 1 種なので `Let(x, App(callee, args), k)` の形でだけ現れ、`L5 (f)` より
        `rhs_consumes` が呼ばれる。
    BY D2, L5 (f)
  <2>2. D9 の行の前半「callee の全 boxed leaf」は、`L5 (i)` の前半が積む。
    BY L5 (i)
  <2>3. D9 の行の後半「呼び出し先がその位置の unit を所有する引数の leaf」は、`resolve_callee_params` が
        `Some(params)` のとき `owns(&params[i], &leaf)` が積む。DEF leaf 粒度の所有 より、この述語は
        「`params[i]` の leaf `leaf` の unit が D14 の意味で所有される」ことと同値である。
    BY D14, DEF leaf 粒度の所有, L5 (i)
  <2>4. `resolve_callee_params` が `None` のときは全位置を所有として扱う (`L5 (i)`)。A7 がこれを所有を
        増やす向きの近似として置いている。
    BY A7, L5 (i), L5 (j)
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4
<1>2. CASE D9 の `Closure(f, caps)` の行。D9 の行「各 capture の全 boxed leaf」は `L5 (h)` が積む。
      `Closure` は `RcRhs` の 1 種なので `L5 (f)` より `rhs_consumes` が呼ばれる。
  BY D2, L5 (f), L5 (h)
<1>3. CASE D9 の `Llvm(gen, args)` の行。D9 の行「`borrows_operand(i)` が偽のオペランドのうち、
      `result_prov` が単一の `Arg(i, σ)` として素通しを宣言していない leaf」は `L5 (k)` の条件そのもので
      あり、その「単一の `Arg(i, σ)`」は `L5 (l)` の条件そのものである。`Llvm` は `RcRhs` の 1 種なので
      `L5 (f)` より `rhs_consumes` が呼ばれる。
  BY D2, L5 (f), L5 (k), L5 (l)
<1>4. CASE D9 の `Destructure(c, fs)` (`c` が boxed) の行。D9 の行「`c` の全 boxed leaf」は
      `L5 (e)` の `is_box` が真の場合が返し、`L5 (d)` が積む。
  BY L5 (d), L5 (e)
<1>5. CASE D9 の `Destructure(c, fs)` (`c` が unbox) の行。D9 の行「名前が付いていないフィールドの leaf」は
      `L5 (e)` の `is_box` が偽の場合が返し、`L5 (d)` が積む。
  BY L5 (d), L5 (e)
<1>6. CASE D9 の「関数本体の終端の `Ret(x)`」の行。
  <2>1. 関数本体の終端の `Ret(x)` は `RcExpr::Ret` の節点であり、`L5 (b)` より走査はそれを訪れる。
    BY D3, L5 (b)
  <2>2. D9 の行「`x` の全 boxed leaf」は `L5 (c)` が積む。
    BY L5 (c)
  <2>3. QED
    BY <2>1, <2>2
<1>7. QED
  D9 の消費の表は 6 行からなり、`<1>1` から `<1>6` がその 6 行である。
  BY D9, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### P7 (b) 余分に報告されるのはアーム本体の `Ret` に限る

<1>1. `L5 (m)` の出どころ (d) が積むものは D9 の `Destructure` の 2 行のいずれかである。`L5 (e)` の
      2 つの場合は `container.ty.is_box(type_env)` の真偽で尽きており、それぞれ D9 の `Destructure`
      (boxed) と `Destructure` (unbox) の行に等しい。
  BY D9, L5 (d), L5 (e), L5 (m)

<1>2. 出どころ (h) が積むものは D9 の `Closure` の行である。`L5 (h)` が積むのは各 capture の全 boxed
      leaf であり、D9 の `Closure` の行と同じである。
  BY D9, L5 (h), L5 (m)

<1>3. 出どころ (i) が積むものは D9 の `App` の行である。`L5 (i)` が積むのは callee の全 boxed leaf と、
      `owns(&params[i], &leaf)` が真の引数 leaf (`resolve_callee_params` が `None` のときは全部) で
      ある。前者は D9 の `App` の行の前半であり、後者は DEF leaf 粒度の所有 により後半と同値である。
      `None` の場合は A7 が定める扱いである。
  BY A7, D9, DEF leaf 粒度の所有, L5 (i), L5 (j), L5 (m)

<1>4. 出どころ (k) が積むものは D9 の `Llvm` の行である。`L5 (k)` が積むのは `borrows_operand(i)` が偽の
      オペランドの leaf のうち `passthrough_arg_leaves` に入らないものであり、`L5 (l)` よりその条件は
      「結果のどの leaf の宣言も単一の `Arg(i, leaf)` でない」ことである。これは D9 の `Llvm` の行と
      同じである。
  BY D9, L5 (k), L5 (l), L5 (m)

<1>5. 出どころ (c) が積むものは、その `Ret` 節点が `collect_consumes` に渡された本体の終端のものなら D9 の
      `Ret` の行であり、そうでないなら `Match` のアーム本体の終端の `Ret` である。
  <2>1. 走査が訪れる `RcExpr::Ret` の節点は、`collect_consumes_go` が呼ばれた本体の終端のものである。
    <3>1. `Ret` は唯一の終端子であり、`Ret` 以外の腕は継続へ進む。
      BY D2, L5 (b)
    <3>2. QED
      BY <3>1
  <2>2. `collect_consumes_go` が呼ばれる本体は、`collect_consumes` が渡した本体と、`Let` の腕が
        `RcRhs::Match` のときに渡す各 `arm.body` である。`collect_consumes` が渡す本体は関数本体である。
    BY L5 (b)
  <2>3. QED
    BY D3, <2>1, <2>2

<1>6. QED
  出どころ (d)、(h)、(i)、(k) は D9 の消費の行そのものであり (`<1>1` から `<1>4`)、(c) は関数本体の終端の
  `Ret` (D9 の行) か `Match` のアーム本体の終端の `Ret` かのどちらかである (`<1>5`)。`L5 (m)` より
  出どころはこの 5 つで全部である。
  BY L5 (m), <1>1, <1>2, <1>3, <1>4, <1>5

### L6 (報告しない箇所は D9 の消費ではない)

**言明**。`collect_consumes` が報告しない箇所は、いずれも D9 の消費ではない。それらが義務集合 `Obl` に
対して行うのは、何もしないか、D9 の移動 (`Obl` を変えない) か、D10 が `Retain` / `Release` の行で直接
定める増減かのどれかである。

<1>1. `rhs_consumes` の `RcRhs::Var(_)` の腕 (`L5 (g)`)。D9 の消費の表の 6 行 (`App`、`Closure`、
      `Llvm`、`Destructure` の 2 行、終端の `Ret`) に `Var` の行は無いので、これは消費ではない。
      `Let(x, Var(y), k)` は D9 の移動の表の第 1 行であり、`y` の参照は活性化の中で `x` へ移る。
      D10 の移動の行より `Obl` は変わらない。
  BY D9, D10, L5 (g)

<1>2. `rhs_consumes` の `RcRhs::Match(..)` の腕 (`L5 (g)`)、および `collect_consumes_go` の `RcExpr::Let` の
      腕の `RcRhs::Match` の場合 (`L5 (f)`)。D9 の消費の表に `Match` の行は無いので、これは消費ではない。
      D9 は `Match` 節点自身が参照を作らず、移さず、手放さないと述べるので `Obl` は変わらない。アームの
      中の消費は `L5 (b)` の再帰が報告する。
  BY D9, L5 (b), L5 (f), L5 (g)

<1>3. `collect_consumes_go` の `RcExpr::Retain` の腕 (`L5 (f)`)。D9 の消費の表に `Retain` の行は無いので、
      これは消費ではない。`Retain` は D10 が直接扱う構文であり、D10 の `Retain` の行は `Obl` への追加で
      ある。
  BY D8, D9, D10, L5 (f)

<1>4. `collect_consumes_go` の `RcExpr::Release` の腕 (`L5 (f)`)。D9 の消費の表に `Release` の行は
      無いので、これは消費ではない。`Release` は参照を処分するが、D10 は `Release` の行を消費の行とは
      別に持ち、その増減を直接定める。
  BY D9, D10, L5 (f)

<1>5. `collect_consumes_go` の `RcExpr::Eval` の腕 (`L5 (f)`)。D9 の消費の表に `Eval` の行は無いので、
      これは消費ではない。D9 は `Eval(v, k)` が参照を作らず、移さず、手放さないと述べるので `Obl` は
      変わらない。D7 の読む構文の表には入っているが、読みは `Obl` を変えない。
  BY D7, D9, L5 (f)

<1>6. `rhs_consumes` の `RcRhs::Llvm` の腕が `borrows_operand(i)` が真のときに飛ばすオペランド
      (`L5 (k)`)。D9 の `Llvm` の行は消費を `borrows_operand(i)` が偽のオペランドに限るので、これは
      消費ではない。A3 が「`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの参照を処分
      しない」と置くので、`Obl` は変わらない。
  BY A3, D9, L5 (k)

<1>7. `rhs_consumes` の `RcRhs::Llvm` の腕が `passthrough` に入るとして飛ばす leaf (`L5 (k)`, `L5 (l)`)。
      D9 の `Llvm` の行は消費から素通し leaf を外しているので、これは消費ではない。A3 の表の「単一の
      `Arg(j, σ)`」の行が、生成コードはそこに第 `j` オペランドの leaf `σ` と同じ参照を置き、新しい参照を
      作らないと述べる。D9 の移動の表の最後の行がこれを移動とし、D10 の移動の行より `Obl` は変わらない。
  BY A3, D9, D10, L5 (k), L5 (l)

<1>8. `destructure_consumes` が unbox 容器について落とす名前付きフィールドの leaf (`L5 (e)`)。D9 の
      `Destructure` (unbox) の行は消費を名前が付いていないフィールドの leaf に限るので、これは消費では
      ない。D9 の移動の表の第 3 行がこれを移動とし、D10 の移動の行より `Obl` は変わらない。
  BY D9, D10, L5 (e)

<1>8a. `rhs_consumes` の `RcRhs::App` の腕が、`resolve_callee_params` が `Some(params)` を返し
       `owns(&params[i], &leaf)` が偽のときに積まない引数 leaf (`L5 (i)`)。DEF leaf 粒度の所有 より、この
       述語が偽であることは、`params[i]` のその leaf の unit を呼び出し先が**借用する** (D14) ことと
       同値である。D9 の `App` の行は消費を呼び出し先が所有する位置の leaf に限っているので、これは
       消費ではない。D14 より借用する unit の参照は呼び出し元が処分するので、`Obl` は変わらない。
  BY D9, D14, DEF leaf 粒度の所有, L5 (i)

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
  BY L5 (c), L5 (h), L5 (m), <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>8a

### P7 の結論

<1>1. (a) は P7 (a) の言明である。
  BY P7 (a)
<1>2. (b) は P7 (b) の言明である。
  BY P7 (b)
<1>3. QED
  README の P7 は (a) と (b) からなる。
  BY <1>1, <1>2

## 9. `README.md` へ差し戻す点

### `Match` の網羅性 -- 仮定 A16 の提案

第 1 節の H1 を仮定として `README.md` へ足すことを提案する。案文は次のとおりである。

> **A16 (`Match` のアームは scrutinee のタグを尽くす)** -- 果たす者: lowering
> (`CODE src/rc_ir/lower.rs: Lowerer::lower_match`, `Lowerer::lower_if`) と、アームの列を保つ後段の
> パス。検査: 無し。
> すべての `Match(s, arms)` について、`arms` が catch-all アーム (`tag` が `None`) を持つか、`s` の値が
> 取りうる実行時のタグがいずれかのアームの `tag` である。

これが要るのは、D21 が「`v` の値の実行時のタグに `tag` が等しいアームが無ければコード生成の振る舞いに
従う」と書いており、コード生成は最後のアームのブロックを switch の default とするからである
(`CODE src/rc_ir/codegen.rs: Generator::eval_rc_match`)。最後のアームが変位アームで、どのアームも
名指さない変位が在ると、実行はその変位の値をもって `tag = Some(t)` のアームに入る。そのとき D9 の移動の
表の「unbox union の変位アームの payload 束縛」の行が名指す**活性**変位と、`origin_inner` の
`Binding::Payload(scrut, Some(t))` の腕が辿る静的な変位番号 `t` が食い違い、L1b、L4、P5 (a)、P6 (b) が
偽になる。`validate` が見るのは、アームが 1 つ以上あること、catch-all アームが最後にあること、2 つの
アームが同じ変位を担わないことだけである (`CODE src/rc_ir/validate.rs: Validator::check_rhs`)。

### D9 の移動の表を leaf の粒度で読む規則

D9 の移動の表は構文の粒度で書かれている -- 「`c` のそのフィールドの参照がフィールド変数へ」。ところが
`origin` も `Retain`/`Release` も leaf の粒度で動くので、この表を使う証明は「始点のどの leaf が終点の
どの leaf に対応するか」を要る。この文書は第 2 節の DEF 辺の leaf 対応 でそれを与えた。写り方は D17 が
`origin` の辿る辺についてすでに書いているものと同じなので、D9 (または D20) の脇に同じ表を置くことを提案
する。置かないと、D9 の行を読む証明はどれもこの読み替えを自前で作ることになる。

### P5 (b) の「同じオブジェクトを指す」は使っていない

P5 (b) の証明は前提のうち別名の道の条件だけを使う。E1 から E5 の別名の道で結ばれた 2 つのスロットは、
オブジェクトが同じかどうかを問わず `identity` が等しい。言明を強い側へ書き直せるが、読み手 (P17、P19) が
要るのは現在の形なので、変えなくても困らない。

### P5 (a) が載っている仮定 -- A3 の「複数の元」の行

P5 (a) の証明は L4 を通り、L4 は `result_prov` が leaf に置く集合の元数が 0 か 1 であること (A3) に
載っている。元数 2 以上の宣言を持つ op が現れると、`origin` は boxed leaf の path から `origin_from_leaves_under`
の `truncate_to_unit` を通る枝に入りうる。そのとき、1 つの unbox union の下の 2 つの leaf がどちらも
その union の unit path へ切り詰められ、`identity` が 1 つに潰れる。潰れた 2 つの leaf は別々の
オブジェクトを指しうるので、P5 (a) は破れる。`Std::Option (a, b)` の payload がこの形の値である。

A3 は「この仮定は誰も果たさない」と書いており、`LLVMGen` の型と doc は元数 2 以上の宣言を許す。P5 (a) が
この事実に載っていることは、A3 の脇に書いておく価値がある。

### D9 の `App` の行と `collect_consumes` の粒度が違う

D9 の `App` の行は「呼び出し先がその位置の **unit** を所有する引数の leaf」と unit 粒度で述べ、
`collect_consumes` の `owns` は leaf 粒度の集合への所属である
(`CODE src/rc_ir/ownership.rs: collect_consumes`, `CODE src/rc_ir/borrow.rs: OwnedLeaves`)。P7 は
DEF leaf 粒度の所有 でこの 2 つを橋渡ししたが、`infer_ownership` が渡す `owned_leaves` が不動点でその形に
なることは P8 が示す必要がある。

### コードに残っている `unit_key` / `unit_of` への言及

`a924f115` が取り除いた `unit_key` と `unit_of` を、次の 3 か所の doc がまだ名指している。証明の引用先では
ないが、読み手を存在しない記号へ送る。

- `CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh` の doc (「two bindings under one `unit_key`」)。
- `CODE src/rc_ir/ownership.rs: References` の doc (「Two operations that key to one `unit_key`」)。
- `CODE src/rc_ir/ownership.rs: tests::the_leaves_of_a_type_truncate_onto_its_units` の doc
  (「`unit_of` asserts this of each key it makes」)。
