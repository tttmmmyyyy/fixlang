# P5, P6, P7 -- identity とオブジェクト、`acted_references`、消費の網羅性

この文書は `README.md` の P5 (a)、P5 (b)、P5 (c)、P6、P7 を証明する。立つのは `README.md` の定義 D1-D19、
仮定 A1-A14、および命題 P1-P4 の**言明**である。P1-P4 の証明は `p10-leaves-and-units.md` と
`p11-origin-soundness.md` にあり、この文書はその言明だけを使う。

読んだコードは作業ツリーの版である。README の対象コミット `a924f115` との差分は、生成される `// PROOF:` の
注釈行だけであり、この文書が引用する記号の本文は対象コミットと一致する。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P5 (a) 対の健全性 | 証明した |
| P5 (b) 対の有効性 | 証明した |
| P5 (c) 被覆 | 証明した |
| P6 (`acted_references` は静的な上位近似である) | 証明した |
| P7 (消費の網羅性) | 証明した |

P5 (a) の要は L4 である。**スロットの path は boxed leaf であり、`origin` の再帰は boxed leaf の path から
出ると boxed leaf の path しか訪れない。** `origin_from_leaves_under` が `truncate_to_unit` で path を
unit へ切り詰める枝は、leaf でない path でだけ通る。1 つの unit の下に複数の leaf を持つ値
(`Std::Option (a, b)` の payload) が 2 つのオブジェクトに届いても、その 2 つの leaf の `identity` が
1 つに潰れることは無い。第 9 節に、この結論がどの仮定に載っているかを書く。

## 1. 記法

1 つの関数 (またはグローバル初期化子) の本体を固定し、その本体から作られる `VarTable` を `vars`、プログラムの
`TypeEnv` を `type_env` と書く (`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`)。この
2 つは本体ごとに 1 つなので、以下では `origin` と `acted_references` の第 1・第 2 引数を落として書く。

- `origin(x, π)` は `origin(vars, type_env, x, π)`。
- `id(x, π)` は `origin(x, π).identity()`、`cand(x, π)` は `origin(x, π).candidates()` を集合とみなした
  もの、`act(x, π)` は `origin(x, π).acted_on()` を集合とみなしたもの
  (`CODE src/rc_ir/ownership.rs: Origin::identity`, `Origin::candidates`, `Origin::acted_on`)。
- `ty(x)` は `x` が得る値の型である。A12 より同じ名前の `RcVar` が持つ型は一致するので、これは `x` の
  出現によらない。
- `L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合。D4 より、これが
  「`v` の `π` の下の boxed leaf」の全体である。inhabited (D16) でないものを含む。
- 実行路の位置 `p` において、`L(v, π)` の元のうち inhabited なものの集合を `Linh(v, π, p)` と書く。
- `ActRefs(v, π)` は D15 の `acted_references(v, π)` である。

`VarPath` は対 `(FullName, FieldPath)` である (`CODE src/rc_ir/ast.rs: VarPath`)。等号はこの対の等号である。

この文書は補題を `L1` から `L5`、反例を `R1` と呼ぶ。`BY` の行ではそれらを名前で引用し、その中のステップを
指すときは `L1 の <1>3` と書く。

**DEF 路のスロット**
1 つの活性化と、その中の 1 つの実行路 (D3) `ρ` を固定する。名前 `x` が `ρ` の上で**値を得る**とは、次の
どちらかをいう。

- `x` がこの本体のパラメータか capture であるか、`ρ` の上のある節点が `x` を束縛する。
- `x` が `vars.bindings` に無い名前である。`VarTable::of` はパラメータ・capture と、`collect_bindings` が
  歩く本体のすべての束縛を記録するので、この場合の `x` は本体が束縛しない名前、すなわち D1 の `globals` の
  記号か `funcs` の記号であり (A11、`CODE src/rc_ir/ownership.rs: VarTable::of`, `collect_bindings`)、
  その値はプログラムが定める 1 つの値である。

`x` が `ρ` の上で値を得るとき、その値の inhabited (D16) な boxed leaf `λ` との対 `(x, λ)` を **`ρ` の
スロット**と呼び、その leaf が指すオブジェクトを `obj(x, λ)`、その leaf が持つ参照 (A5 よりちょうど 1 つ) を
`ref(x, λ)` と書く。

D2 より本体は木であり、D3 の実行路は各節点を高々 1 度通り、A6 より束縛名は相異なるので、1 つの活性化・
1 つの路について `x` が得る値は高々 1 つである。よって `obj(x, λ)` と `ref(x, λ)` は路の上の位置に依らない。
位置 `p` のスロット (D6) は、`p` までに値を得た名前についての `ρ` のスロットであり、D6 の `obj(x, λ)` は
この `obj(x, λ)` である。

`ρ` のスロットは D6 のスロットより広い。`Match` のアーム本体が束縛した変数は、その `Match` の後の位置では
スコープに無いが、`ρ` の上で値を得ているので `ρ` のスロットを持つ。L4 が要るのはこの広い側である
(`identity` がアーム本体の中の変数を名指すことがある)。

## 2. 移動の辺

D9 の移動の表の 6 行を、スロットの間の辺として次の名前で呼ぶ。以下 `λ` は leaf を渡る。

- **E1** (`Let(x, Var(y), k)`): `(y, λ)` と `(x, λ)`。
- **E2** (`Destructure(c, fs, s, k)` で `c` が unbox、`(i, f)` が `fs` の元): `(c, [i] ++ λ)` と `(f, λ)`。
- **E3** (`Let(m, Match(s, arms), k)` の unbox union の変位アームの payload 束縛、変位番号 `t`、payload
  変数 `p`): `(s, [t] ++ λ)` と `(p, λ)`。
- **E4** (catch-all アームの payload 束縛、payload 変数 `p`): `(s, λ)` と `(p, λ)`。
- **E5** (`Let(x, Llvm(gen, args), k)` の素通し leaf): 結果の leaf `λ` の `result_prov` の宣言が単一の
  `Arg(j, σ)` のとき、`(args[j], σ)` と `(x, λ)`。
- **E6** (`Match` のアーム本体の `Ret(x)`): `Match` の束縛変数を `m` として、`(x, λ)` と `(m, λ)`。

**DEF 別名の道**
`ρ` のスロットを頂点とし、`ρ` の上で実行された E1-E6 の辺を (向きを問わない) 辺とするグラフの上の道を、
**別名の道**と呼ぶ。

## L1 (移動の辺の両端は同じオブジェクトを指す)

**言明**。`ρ` の上で実行された E1-E6 の辺の両端のスロットは、同じ参照を持ち、したがって同じオブジェクトを
指す。

<1>1. E1 から E6 は D9 の移動の表の 6 行に 1 対 1 で対応する。
  D9 の移動の表の行は `Let(x, Var(y), k)`、`Match` のアーム本体の `Ret(x)`、unbox 容器の `Destructure` の
  名前付きフィールド、unbox union の変位アームの payload 束縛、catch-all アームの payload 束縛、
  `Llvm` の素通し leaf の 6 行であり、それぞれ E1、E6、E2、E3、E4、E5 である。
  BY D9

<1>2. 移動は「参照の持ち手が活性化の中で変わるだけ」であり、移った参照は 1 つの同じ参照である。
  BY D9

<1>3. QED
  A5 よりスロットは参照をちょうど 1 つ持つので、辺の両端が持つ参照は `<1>2` の移った参照である。D8 より
  1 つの参照は 1 つのオブジェクトに対する処分義務の 1 単位なので、同じ参照は同じオブジェクトを指す。
  BY A5, D8, <1>1, <1>2

## L2 (E1-E5 の辺は `origin` を保つ)

**言明**。E1、E2、E3、E4、E5 のいずれかの辺で結ばれた 2 つのスロットは、同じ `origin` を持つ。

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

<1>5. E1 の辺 `(y, λ)`-`(x, λ)` について `origin(x, λ) = origin(y, λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Move(y))` の腕は `origin(vars, type_env, &y.name, path)` を
        そのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕
  <2>2. QED
    BY <1>1, <2>1

<1>6. E2 の辺 `(c, [i] ++ λ)`-`(f, λ)` について `origin(f, λ) = origin(c, [i] ++ λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Field(container, idx))` の腕は、`container.ty.is_box(type_env)`
        が偽のとき `container_path` を `[*idx] ++ path` として作り
        `origin(vars, type_env, &container.name, &container_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕
  <2>2. E2 の辺は `c` が unbox のときにだけ引かれるので、`container.ty.is_box(type_env)` は偽である。
    BY D9 (移動の表の unbox 容器の `Destructure` の行)
  <2>3. QED
    BY <1>2, <2>1, <2>2

<1>7. E3 の辺 `(s, [t] ++ λ)`-`(p, λ)` について `origin(p, λ) = origin(s, [t] ++ λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の
        `Some(tag) if !scrut.ty.is_box(type_env)` の場合は、`scrut_path` を `[*tag] ++ path` として作り
        `origin(vars, type_env, &scrut.name, &scrut_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. E3 の辺は `s` が unbox union のときにだけ引かれるので、`scrut.ty.is_box(type_env)` は偽であり、
        `arm.tag` は `Some(t)` である。
    BY D9 (移動の表の unbox union の変位アームの行), <1>3
  <2>3. QED
    BY <1>3, <2>1, <2>2

<1>8. E4 の辺 `(s, λ)`-`(p, λ)` について `origin(p, λ) = origin(s, λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の `None` の場合は
        `origin(vars, type_env, &scrut.name, path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. E4 の辺は catch-all アームについてだけ引かれるので、`arm.tag` は `None` である。
    BY D9 (移動の表の catch-all の行), <1>3
  <2>3. QED
    BY <1>3, <2>1, <2>2

<1>9. E5 の辺 `(args[j], σ)`-`(x, λ)` について `origin(x, λ) = origin(args[j], σ)` である。
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
    BY <1>4, <2>1, <2>3

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

<1>1. `boxed_leaf_paths` の内部の走査 `go` は、`ty.is_closure()`、`ty.is_box(type_env)`、`ty.is_array()` の
      各腕で `out` に path を積んだ直後に `return` する。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `go` が path を積むのはその 3 つの腕だけであり、`ty.is_fully_unboxed(type_env)` の腕は積まずに
      `return` し、最後の腕は `unpunched_field_types` の各フィールドへ降りる。
  BY D4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. 積まれた path `P` について、`P` を真の前置に持つ path が積まれることはない。
  <2>1. `P` が積まれたのは `go` が `P` に対応する型に着いたときであり、その腕は積んだ直後に `return` する。
    BY <1>1, <1>2
  <2>2. `P` を真の前置に持つ path が積まれるには、`go` が `P` に対応する型からさらにフィールドへ降りる
        必要がある。
    BY <1>2
  <2>3. QED
    BY <2>1, <2>2

<1>4. QED
  `π` が leaf であるとき、`π` を前置に持つ leaf は `π` 自身だけである。
  BY <1>3

## L4 (identity のスロット)

**言明**。`ρ` を実行路、`(x, λ)` を `ρ` のスロット (DEF 路のスロット) とする。`id(x, λ) = (w, σ)` と
おくと、次が成り立つ。

- (i) `σ` は `ty(w)` の boxed leaf であり、`(w, σ)` は `ρ` のスロットである。
- (ii) `obj(x, λ) = obj(w, σ)`。

証明は、`origin` が `(x, λ)` から行う再帰呼び出しの関係の上の帰納法による。P2 より `origin` は停止するので、
無限に降りる呼び出しの列は無く、この関係は整礎である。以下、帰納法の仮定を「IH」と書く。

<1>1. `origin(x, λ)` の値は `origin_inner(vars, type_env, x, λ)` の値である。
  BY CODE src/rc_ir/ownership.rs: origin -- memo を引くか `origin_inner` を呼ぶかのどちらかであり、memo に
     入る値は `origin_inner` の値である。

<1>2. `origin_inner` の腕は、`vars.bindings.get(x)` の値について次の 6 本で尽きている。
      `None | Some(Param) | Some(Producer)`、`Some(Move(y))`、`Some(Join(arm_results))`、
      `Some(Llvm(gen, args, result_ty))`、`Some(Field(c, i))`、`Some(Payload(s, variant))`。
  BY CODE src/rc_ir/ownership.rs: Binding (構成子は `Param`、`Move`、`Llvm`、`Producer`、`Field`、
     `Payload`、`Join` の 7 つ), origin_inner

<1>3. `x` の `Binding` が名指す変数 -- `Move(y)` の `y`、`Field(c, i)` の `c`、`Payload(s, ·)` の `s`、
      `Llvm(gen, args, ·)` の `args` の元、`Join(rs)` の `rs` のうち `ρ` が通ったアームの元 -- は、
      `ρ` の上で値を得ている。
  <2>1. `collect_bindings` はこれらの変数を `x` を束縛する節点から取る。`Move` と `Llvm` は
        `Let(x, rhs, k)` の `rhs` から、`Field` は `Destructure` の容器から、`Payload` は `Match` の
        scrutinee から、`Join` の元は各アーム本体の `returned_var` から取る。
    BY CODE src/rc_ir/ownership.rs: collect_bindings, returned_var
  <2>2. 節点に現れる変数の使用は、その節点でスコープに入っている束縛に解決する。
    BY A11
  <2>3. D2 より、`Let` と `Destructure` が束縛する変数のスコープはその継続、`Match` のアームの payload の
        スコープはそのアーム本体であり、どちらも束縛の節点の子孫である。よって節点でスコープに入っている
        束縛は、その節点の祖先が束縛したものか、パラメータ・capture である。
    BY D2
  <2>4. D3 より実行路は根から辿るので、`ρ` が着いた節点の祖先はすべて `ρ` の上にある。パラメータと
        capture は本体の実行の前に値を得ている。
    BY D3, D10 (`Obl` の初期値はパラメータ・capture の leaf である)
  <2>5. `Join(rs)` の元のうち `ρ` が通ったアームのものは、そのアーム本体の終端の `Ret` が名指す変数で
        ある。その `Ret` は `ρ` の上の節点なので、`<2>2` から `<2>4` をその位置に適用できる。
    BY D3, <2>1, <2>2, <2>3, <2>4
  <2>6. `vars.bindings` に無い名前は DEF 路のスロット の第 2 の場合であり、路によらず値を得ている。
    BY DEF 路のスロット
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>4. `x` の `Binding` が `Llvm(gen, args, result_ty)` であるとき、`result_ty` は `ty(x)` であり、
      `decl := gen.result_prov(result_ty, &arg_tys, type_env)` について `decl.leaf_origins_at(λ)` は
      `Some(S)` である。
  <2>1. `collect_bindings` は `Let(x, Llvm(llvm_gen, args), k)` に `Binding::Llvm(llvm_gen, args, x.ty)` を
        作る。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Llvm` の場合
  <2>2. `result_prov` は結果の leaf ごとに `LeafOrigins` を 1 つ宣言する。
    BY A3
  <2>3. `leaf_origins_at(π)` は、`π` に記録がある場合に `Some`、無い場合に `None` を返す。
    BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>4. QED
    `λ` は `ty(x)` の boxed leaf である (DEF 路のスロット)。
    BY <2>1, <2>2, <2>3

<1>5. `<1>4` の `S` の元数は 0 か 1 である。
  A3 は「複数の元を宣言する op は、このコミットのプログラムには存在しない」と述べ、`result_prov` を
  override する 29 個が leaf に置く集合はすべて要素数 0 か 1 であるとする。`origin_inner` が読む `decl` は
  `llvm_gen.result_prov(..)` の返り値そのものであって、`Provenance::join` や `compose` を通していない。
  BY A3, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕

<1>6. CASE `None | Some(Param) | Some(Producer)`。
  <2>1. 答えは `here()` すなわち `Exactly((x, λ))` であり、`id(x, λ) = (x, λ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner の
       `None | Some(Binding::Param) | Some(Binding::Producer)` の腕, Origin::identity
  <2>2. QED
    (i) は前提 (`(x, λ)` は `ρ` のスロットであり `λ` は `ty(x)` の boxed leaf である)、(ii) は同じ
    スロットどうしの等号である。
    BY <2>1, DEF 路のスロット

<1>7. CASE `Some(Move(y))`。
  <2>1. `origin(x, λ) = origin(y, λ)` であり、よって `id(x, λ) = id(y, λ)` である。
    BY L2 の <1>5
  <2>2. `(y, λ)` は `ρ` のスロットである。
    A12 より `ty(y) = ty(x)` なので `λ` は `ty(y)` の boxed leaf である。D9 の移動の表の第 1 行より
    `x` が得る値は `y` の値であり、D16 の inhabited は値で決まるので `λ` は `y` の値でも inhabited で
    ある。`y` が `ρ` の上で値を得ていることは `<1>3` である。
    BY A12, D9, D16, <1>3
  <2>3. `obj(x, λ) = obj(y, λ)`。
    BY L1 (E1 の辺)
  <2>4. QED
    IH を `(y, λ)` に適用すると、`id(y, λ) = (w, σ)` について (i) と `obj(y, λ) = obj(w, σ)` が出る。
    BY <2>1, <2>2, <2>3, IH

<1>8. CASE `Some(Field(c, i))` で `c` が boxed。
  <2>1. 答えは `here()` であり `id(x, λ) = (x, λ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕の
       `container.ty.is_box(type_env)` が真の枝, Origin::identity
  <2>2. QED
    BY <2>1, DEF 路のスロット -- `<1>6 の <2>2` と同じ理由で (i) と (ii) が成り立つ。

<1>9. CASE `Some(Field(c, i))` で `c` が unbox。
  <2>1. `origin(x, λ) = origin(c, [i] ++ λ)` であり、よって `id(x, λ) = id(c, [i] ++ λ)` である。
    BY L2 の <1>6
  <2>2. フィールド `i` は `c` の値が保持するフィールドである。
    D9 の移動の表の unbox 容器の `Destructure` の行より、スロット `(x, λ)` が持つ参照は `c` のフィールド
    `i` の参照である。A5 より `c` の値が持つ参照は `ty(c)` の inhabited な boxed leaf にあり、D4 の
    規則 5 は穴のフィールドの下へ降りないので、`i` が穴なら `c` はフィールド `i` に参照を持たない。
    BY A5, D4, D9
  <2>3. `[i] ++ λ` は `ty(c)` の boxed leaf である。
    A12 より `ty(x)` は `c` のフィールド `i` の型であり、D4 の規則 5 より unbox 集約の leaf は
    `unpunched_field_types` が返す各フィールドの添字を前置した leaf である。`<2>2` よりフィールド `i` は
    その中にある。
    BY A12, D4, <2>2
  <2>4. `[i] ++ λ` は `c` の値で inhabited である。
    A12 より `c` は構造体なので `[i]` は unbox union の節を通らず、`[i] ++ λ` が通る unbox union の節は
    `λ` が通る節と同じである。
    BY A12, D16, <2>3
  <2>5. `(c, [i] ++ λ)` は `ρ` のスロットである。
    BY <1>3, <2>3, <2>4
  <2>6. `obj(x, λ) = obj(c, [i] ++ λ)`。
    BY L1 (E2 の辺)
  <2>7. QED
    IH を `(c, [i] ++ λ)` に適用する。
    BY <2>1, <2>5, <2>6, IH

<1>10. CASE `Some(Payload(s, None))` (catch-all)。
  <2>1. `origin(x, λ) = origin(s, λ)` であり、よって `id(x, λ) = id(s, λ)` である。
    BY L2 の <1>8
  <2>2. `(s, λ)` は `ρ` のスロットである。
    D9 の移動の表の catch-all の行より、payload 変数 `x` が得る値は scrutinee `s` の値である。よって
    `λ` は `s` の値の inhabited な boxed leaf である (D16)。`s` が値を得ていることは `<1>3` である。
    BY D9, D16, <1>3
  <2>3. `obj(x, λ) = obj(s, λ)`。
    BY L1 (E4 の辺)
  <2>4. QED
    BY <2>1, <2>2, <2>3, IH

<1>11. CASE `Some(Payload(s, Some(t)))` で `s` が boxed。
  <2>1. 答えは `here()` であり `id(x, λ) = (x, λ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕の
       `Some(_)` の枝, Origin::identity
  <2>2. QED
    BY <2>1, DEF 路のスロット

<1>12. CASE `Some(Payload(s, Some(t)))` で `s` が unbox。
  <2>1. `origin(x, λ) = origin(s, [t] ++ λ)` であり、よって `id(x, λ) = id(s, [t] ++ λ)` である。
    BY L2 の <1>7
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf である。
    A12 より `ty(s)` は union であり `ty(x)` はその変位 `t` の型である。D4 の規則 5 より union の leaf は
    各変位の payload の leaf に変位番号を前置したものである。
    BY A12, D4
  <2>3. この位置で `s` の値のタグは `t` である。
    D9 の移動の表の unbox union の変位アームの行は、その束縛で動く参照を「scrutinee の活性変位の参照」と
    述べる。`ρ` はこのアームを通っている。
    BY D3, D9
  <2>4. `[t] ++ λ` は `s` の値で inhabited である。
    `[t] ++ λ` が通る unbox union の節は、`ty(s)` の根の節 (`<2>3` よりタグは `t`) と、`λ` が通る節
    (前提より `λ` は inhabited) である。
    BY D16, <2>2, <2>3
  <2>5. `(s, [t] ++ λ)` は `ρ` のスロットである。
    BY <1>3, <2>2, <2>4
  <2>6. `obj(x, λ) = obj(s, [t] ++ λ)`。
    BY L1 (E3 の辺)
  <2>7. QED
    BY <2>1, <2>5, <2>6, IH

<1>13. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` が単一の `Arg(j, σ')`。
  <2>1. `origin(x, λ) = origin(args[j], σ')` であり、よって `id(x, λ) = id(args[j], σ')` である。
    BY L2 の <1>9
  <2>2. `σ'` は `ty(args[j])` の boxed leaf であり、`args[j]` の値で inhabited である。
    A3 の「単一の `Arg(j, σ)`」の行は、宣言が第 `j` オペランドの leaf `σ` を名指すこと、および結果の
    その leaf が inhabited であることと第 `j` オペランドの leaf `σ` が inhabited であることが同値である
    ことを述べる。前提より `λ` は inhabited である。
    BY A3
  <2>3. `(args[j], σ')` は `ρ` のスロットである。
    BY <1>3, <2>2
  <2>4. `obj(x, λ) = obj(args[j], σ')`。
    BY L1 (E5 の辺)
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
    BY CODE src/rc_ir/ownership.rs: as_arg_projection, origin_inner の
       `Some(Binding::Llvm(..))` の腕の `None =>` の枝
  <2>2. `decl.leaf_origins_under(λ)` が返すのは `S` 1 つだけである。
    `leaf_origins_under(path)` は `path` を前置に持つ leaf の宣言を返す。`λ` は `ty(x)` の boxed leaf な
    ので、L3 よりそのような leaf は `λ` 自身だけである。
    BY L3, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
  <2>3. `origin_from_leaves_under` は `Some(Origin::Exactly((x, λ)))` を返す。
    `S` の元は `Fresh` か `Unknown` なので、ループは `operand_units` に何も入れず `produced_here` を
    立てる。よって `reached` は `Exactly((x, λ))` 1 つだけからなり、`reached.iter().all(..)` の枝が
    その値を返す。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>4. QED
    `id(x, λ) = (x, λ)` なので、(i) と (ii) は `<1>6 の <2>2` と同じ理由で成り立つ。
    BY <2>3, DEF 路のスロット, CODE src/rc_ir/ownership.rs: Origin::identity

<1>16. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` の元数が 2 以上。
  `<1>5` よりこの場合は起きない。
  BY <1>5

<1>17. CASE `Some(Join(arm_results))`。
  <2>1. 答えは `Origin::of_candidates(C, (x, λ))` である。ここで `C` は各 `r ∈ arm_results` についての
        `act(r, λ)` の合併である。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕
  <2>2. `r_0` を `ρ` が通ったアームの結果とすると、`(r_0, λ)` は `ρ` のスロットであり
        `obj(x, λ) = obj(r_0, λ)` である。
    A12 よりアームの結果と `Match` の束縛変数の型は一致するので `λ` は `ty(r_0)` の boxed leaf である。
    D9 の移動の表の `Match` のアーム本体の `Ret(x)` の行より `x` が得る値は `r_0` の値なので、`λ` は
    `r_0` の値でも inhabited である (D16)。`r_0` が値を得ていることは `<1>3`、オブジェクトの一致は L1
    (E6 の辺) である。
    BY A12, D9, D16, L1, <1>3
  <2>3. `C` は空でない。
    A9 よりアームは 1 つ以上あり、`act(r, λ)` は `identity` を含むので空でない。
    BY A9, D15, CODE src/rc_ir/ownership.rs: Origin::acted_on
  <2>4. CASE `|C| ≥ 2`。
    <3>1. `of_candidates(C, (x, λ))` は `Join { identity: (x, λ), candidates: C }` を返すので
          `id(x, λ) = (x, λ)` である。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::identity
    <3>2. QED
      BY <3>1, DEF 路のスロット -- `<1>6 の <2>2` と同じ理由。
  <2>5. CASE `|C| = 1`。
    <3>1. `C = {c}` とおくと `of_candidates(C, (x, λ))` は `Exactly(c)` を返すので `id(x, λ) = c` で
          ある。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::identity
    <3>2. `id(r_0, λ) = c` である。
      `act(r_0, λ) ⊆ C = {c}` であり (`<2>1`)、`act(r_0, λ)` は `identity` を含むので空でない
      (`<2>3` と同じ理由)。よって `act(r_0, λ) = {c}` であり、その元である `id(r_0, λ)` は `c` である。
      BY <2>1, <2>3, D15, CODE src/rc_ir/ownership.rs: Origin::acted_on
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
  `<1>12` に、`Llvm` は `<1>4` の `S` の形で `<1>13` から `<1>16` に分かれる (`<1>5` が「元数 2 以上」を
  消すので、`S` は空・単一の `Arg`・単一の `Fresh`・単一の `Unknown` で尽きる)。残りは `<1>6`、
  `<1>7`、`<1>17` である。
  BY <1>2, <1>5, <1>6, <1>7, <1>8, <1>9, <1>10, <1>11, <1>12, <1>13, <1>14, <1>15, <1>16, <1>17

**補足 (切り詰めを通る枝に入らないこと)**。`origin_from_leaves_under` が `truncate_to_unit` を呼ぶのは
`LeafOrigin::Arg(j, leaf)` の元を `operand_units` に入れるときである
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。`<1>13` から `<1>16` より、`λ` が boxed leaf で
あるときにこの関数へ入るのは `S` が空集合か単一の `Fresh` / `Unknown` のときだけで、`<1>15 の <2>2` より
そこで読む宣言は `S` だけであり、どの場合も `Arg` の元を持たない。よって leaf の path から出た再帰は
切り詰めを 1 度も通らない。

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

**言明** (README の P5 (b))。同じオブジェクトを指す 2 つの leaf のスロットで、一方から他方への別名の道が
`Match` のアーム本体の `Ret` の辺を含まないならば、両者の `identity` は等しい。

<1>1. 道の各辺は E1 から E5 のいずれかである。
  `Match` のアーム本体の `Ret` の辺は E6 であり、DEF 別名の道 の辺は E1 から E6 である。
  BY DEF 別名の道, 前提

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

**言明**。同じオブジェクトを指す 2 つのスロットで、`identity` が異なるものがある。よって P5 (b) の
「別名の道が `Match` のアーム本体の `Ret` の辺を含まない」という限定は外せない。

<1>1. 次の関数 `f` を考える。`T` は boxed な型、`Bool` は `is_fully_unboxed` が真の unbox union とする。
      パラメータは `c : Bool`、`x : T`、`y : T` の 3 つで、`borrowed_units` は空 (A1) である。本体は
      次のとおりである。

      ```
      Let(m, Match(c, [ arm(tag=0, payload=p0, body=B0), arm(tag=1, payload=p1, body=B1) ]), Ret(m))
      B0 = Release(y, [], s, Ret(x))
      B1 = Release(x, [], s, Ret(y))
      ```

      これは D2 の形の本体である。
  BY D2

<1>2. `boxed_leaf_paths(T, type_env)` は `{[]}` であり、`[]` は inhabited である。`rc_units(T, type_env)` は
      `{[]}` である。`boxed_leaf_paths(Bool, type_env)` は空である。
  <2>1. `T` は boxed なので、`boxed_leaf_paths` の 3 番目の規則により自分自身の位置 1 つが leaf である。
    BY D4
  <2>2. `T` は unbox union を通らないので `[]` は inhabited である。
    BY D16
  <2>3. `T` は boxed なので `unit_step` は `Unit` を返し、`rc_units` は `[]` を積む。
    BY D5
  <2>4. `Bool` は `is_fully_unboxed` が真なので leaf を持たない。
    BY D4
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
  <2>1. `x` はパラメータなので値を得ており、`m` は `Let(m, Match(c, ...), Ret(m))` が束縛する。`[]` は
        `T` の inhabited な boxed leaf なので、どちらも路のスロットである。
    BY D2, DEF 路のスロット, <1>2
  <2>2. アーム本体の `Ret(x)` は E6 の辺 `(x, [])`-`(m, [])` を実行する。
    BY D9, <1>1
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
    <3>1. アーム本体の `Ret(y)` は E6 の辺 `(y, [])`-`(m, [])` を実行するので、`(m, [])` は `(y, [])` と
          同じオブジェクトを指す。終端の `Ret(m)` が消費するのは `r_y` である。
      BY A5, D9, L1, DEF 路のスロット, <1>1, <1>2, <1>5
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

<1>8. `id(x, []) = (x, [])` である。
  `x` はパラメータなので `x` の `Binding` は `Param` であり、`origin_inner` の
  `None | Some(Binding::Param) | Some(Binding::Producer)` の腕は `here()` すなわち `Exactly((x, []))` を
  返す。
  BY CODE src/rc_ir/ownership.rs: VarTable::of, origin_inner, Origin::identity

<1>9. `id(m, []) = (m, [])` である。
  <2>1. `m` の `Binding` は `Join([x, y])` である。`x` と `y` はそれぞれのアーム本体の `returned_var` で
        ある。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Match` の場合,
       returned_var
  <2>2. `act(x, []) = {(x, [])}`、`act(y, []) = {(y, [])}` である。
    どちらもパラメータなので `<1>8` と同じ理由で `origin` は `Exactly` を返し、`Exactly(p)` の
    `acted_on()` は `[p]` である。
    BY <1>8, CODE src/rc_ir/ownership.rs: origin_inner, Origin::acted_on, Origin::candidates
  <2>3. `origin_inner` の `Join` の腕が集める候補集合は `{(x, []), (y, [])}` であり、A6 より 2 元である。
        よって `of_candidates` は `Join { identity: (m, []), .. }` を返す。
    BY A6, <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕,
       Origin::of_candidates
  <2>4. QED
    BY <2>3, CODE src/rc_ir/ownership.rs: Origin::identity

<1>10. QED
  `<1>7` より `f` の本体は RC 規律を満たし、`<1>6` より変位 0 の実行路の `Ret(m)` の位置において
  `(x, [])` と `(m, [])` は同じオブジェクトを指す。`<1>8` と `<1>9` より `id(x, []) = (x, [])`、
  `id(m, []) = (m, [])` であって、A6 よりこの 2 つは異なる。この 2 つのスロットを結ぶ別名の道は
  E6 の辺 1 本である。
  BY A6, <1>6, <1>7, <1>8, <1>9

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

<1>2. `ActRefs(v, π).objects()` は `{ id(v, λ) : λ ∈ L(v, π) }` である。
  <2>1. `acted_references(v, π)` は、`boxed_leaf_paths(ty(v), type_env)` の要素 `leaf` のうち
        `leaf.starts_with(π)` を満たすものについて、`origin(v, leaf).identity()` をキーとする計数を
        1 ずつ増やした `Map<VarPath, usize>` を `References` に包んで返す。
    BY D15, CODE src/rc_ir/ownership.rs: acted_references
  <2>2. 走査が回る `leaf` の全体は `L(v, π)` である。
    BY D4, 記法 の `L` の定義, <2>1
  <2>3. `References::objects()` はその `Map` のキーの列を返す。
    BY D15, CODE src/rc_ir/ownership.rs: References::objects
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>3. `other_objects(v, π)` は `∪_{λ ∈ L(v, π)} (cand(v, λ) \ {id(v, λ)})` を含む。
  `other_objects` は `boxed_leaf_paths(ty(v), type_env)` のうち `leaf.starts_with(path)` を満たす各 `leaf`
  について `where_from = origin(v, leaf)` を取り、その `candidates()` のうち `identity()` と異なるものを
  すべて `out` に積む。回る `leaf` の全体は `<1>2 の <2>2` と同じ `L(v, π)` である。
  BY D4, 記法 の `L` の定義, CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects

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

(b) の 2 つの辺は住む集合が違う。左辺は D8 の意味の参照の多重集合、右辺は `VarPath` 上の多重集合である。
「等しい」を、次の対応の下での一致と読む。

**DEF 名前づけ**
`Retain(v, π)` が leaf `λ` (`λ` は `Linh(v, π, p)` の元) について作る参照を `r_λ` と書き、その名前を
`id(v, λ)` とする。`Release(v, π)` が leaf `λ` について処分する参照を `r'_λ` と書き、その名前も
`id(v, λ)` とする。

### P6 (a)

<1>1. `acted_references(v, path)` は、`boxed_leaf_paths(&v.ty, type_env)` の要素 `leaf` のうち
      `leaf.starts_with(path)` を満たすものについて、`origin(vars, type_env, &v.name, &leaf).identity()` を
      キーとする計数を 1 ずつ増やした `Map<VarPath, usize>` を返す。
  BY CODE src/rc_ir/ownership.rs: acted_references

<1>2. `<1>1` の走査が回る `leaf` の全体は `L(v, π)` である。inhabited かどうかは判定していない。
  BY D4, D15, <1>1

<1>3. QED
  `<1>1` と `<1>2` より、返り値は `L(v, π)` の各元 `λ` を `id(v, λ)` で名付けて数えた多重集合である。
  BY <1>1, <1>2

### P6 (b)

<1>1. `Retain(v, π)` が `Obl` に加える参照は、`Linh(v, π, p)` の各 `λ` についてちょうど 1 つずつであり、
      それが全部である。
  <2>1. D10 の `Retain` の行は「`π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ
        加える」である。
    BY D10
  <2>2. 「`π` の下の inhabited な leaf」の全体は `Linh(v, π, p)` である。
    BY D4, D16, 記法 の `Linh` の定義
  <2>3. QED
    BY <2>1, <2>2

<1>2. `Release(v, π)` が `Obl` から取り除く参照は、`Linh(v, π, p)` の各 `λ` についてちょうど 1 つずつで
      あり、それが全部である。
  <2>1. D10 の `Release` の行は「`π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ
        取り除く」である。
    BY D10
  <2>2. 「`π` の下の inhabited な leaf」の全体は `Linh(v, π, p)` である。
    BY D4, D16, 記法 の `Linh` の定義
  <2>3. QED
    BY <2>1, <2>2

<1>3. `λ ↦ r_λ` は `Linh(v, π, p)` から `Retain(v, π)` が作る参照の多重集合への全単射であり、
      `λ ↦ r'_λ` は `Linh(v, π, p)` から `Release(v, π)` が処分する参照の多重集合への全単射である。
  BY DEF 名前づけ, <1>1, <1>2

<1>4. 名前づけの下での像は、どちらも多重集合 `{ id(v, λ) : λ ∈ Linh(v, π, p) }` である。
  BY DEF 名前づけ, <1>3

<1>5. P6 (a) の数え上げを `Linh(v, π, p)` に制限したものは、多重集合として
      `{ id(v, λ) : λ ∈ Linh(v, π, p) }` である。
  BY P6 (a) の <1>3

<1>6. QED
  BY <1>4, <1>5

### P6 の結論

<1>1. (a) は `P6 (a) の <1>3` である。
  BY P6 (a)
<1>2. (b) は `P6 (b) の <1>6` である。
  BY P6 (b)
<1>3. QED
  BY <1>1, <1>2

**補足 1 (名前がオブジェクトごとの数である理由)**。`References` の各キーはオブジェクトの数を持つ (D15)。
DEF 名前づけ の名前がその読みに耐えるのは P5 (a) による。すなわち、同じ位置の 2 つの inhabited な leaf の
スロットが同じ `id` を持つなら、その 2 つは同じオブジェクトを指す。

**補足 2 (2 つの leaf が 1 つの名前を持つとき)**。`L(v, π)` の相異なる 2 つの leaf が同じ `id` を持つことが
ある。`Map` はそのとき計数を 2 にする (`P6 (a) の <1>1`)。これは 1 つのオブジェクトへの参照を 2 つ持つ値に
対応し、A5 の下で参照は inhabited な leaf ごとに 1 つなので、参照としても 2 つある。

**補足 3 (上位近似のずれは片側だけである)**。`acted_references` は `L(v, π)` を数え、実行時に触れるのは
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

### L5 (走査はすべての節点を 1 度ずつ訪れ、積むものはこれで全部である)

<1>1. `collect_consumes` は `owns` を作って `collect_consumes_go` を呼ぶだけである。
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

<1>7. 走査は本体の各節点をちょうど 1 度訪れる。
  D2 より本体は木であり、`Ret` を除く 5 種の節点の継続は 1 つ、分岐は `Match` のアームだけである。`<1>4` と
  `<1>5` はその継続とアームをちょうど 1 度ずつたどり、`<1>6` が終端である。
  BY D2, D3, <1>3, <1>4, <1>5, <1>6

<1>8. `RcExpr::Ret(x)` の腕は `push_boxed_leaves(&x.name, &x.ty, type_env, out)` を呼び、
      `boxed_leaf_paths(x.ty, type_env)` の各 `p` について `(x.name, p)` を積む。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Ret(x)` の腕, push_boxed_leaves

<1>9. `RcExpr::Destructure(container, fields, _state, k)` の腕は
      `destructure_consumes(container, fields, type_env)` の各 `leaf` について `(container.name, leaf)` を
      積む。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Destructure` の腕

<1>10. `destructure_consumes` は、`container.ty.is_box(type_env)` が真のとき
       `boxed_leaf_paths(container.ty, type_env)` の全部を返し、偽のときはそのうち先頭の添字が `fields` の
       名前付きフィールドの添字でないものだけを返す。
  BY CODE src/rc_ir/ownership.rs: destructure_consumes

<1>11. `RcExpr::Retain | RcExpr::Release | RcExpr::Eval` の腕は何も積まない。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の
     `RcExpr::Retain(..) | RcExpr::Release(..) | RcExpr::Eval(..)` の腕

<1>12. `RcExpr::Let(x, rhs, k)` の腕は、`rhs` が `RcRhs::Match` のとき自身は何も積まず、それ以外の 4 種の
       とき `rhs_consumes(rhs, &x.ty, vars, prog, type_env, owns, out)` を呼ぶ。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Let` の腕

<1>13. `RcRhs` は `Var`、`App`、`Closure`、`Llvm`、`Match` の 5 種である。
  BY D2, CODE src/rc_ir/ast.rs: RcRhs

<1>14. `rhs_consumes` の `match` の腕は `RcRhs::Var(_) | RcRhs::Match(..)`、`RcRhs::Closure(_, caps)`、
       `RcRhs::App(callee, args)`、`RcRhs::Llvm(llvm_gen, args)` の 4 つであり、`<1>13` の 5 種を尽くす。
  BY <1>13, CODE src/rc_ir/ownership.rs: rhs_consumes

<1>15. `RcRhs::Var(_) | RcRhs::Match(..)` の腕は何も積まない。
  BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Var(_) | RcRhs::Match(..)` の腕

<1>16. `RcRhs::Closure(_, caps)` の腕は、`caps` の各元 `c` について `boxed_leaf_paths(c.ty, type_env)` の
       各 `p` を `(c.name, p)` として積む。
  BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Closure(_, caps)` の腕, push_boxed_leaves

<1>17. `RcRhs::App(callee, args)` の腕は、`callee` の全 boxed leaf を積み、さらに各引数 `args[i]` の各
       boxed leaf `leaf` について、`resolve_callee_params` が `Some(params)` を返したときは
       `owns(&params[i], &leaf)` が真のときだけ、`None` を返したときは常に `(args[i].name, leaf)` を積む。
       A14 より `params[i]` は範囲内である。
  BY A14, CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App(callee, args)` の腕, push_boxed_leaves

<1>18. `resolve_callee_params` が `None` を返すのは、`callee.name` が `vars.closure_targets` にも
       `prog.funcs` にも無いとき、すなわち間接呼び出しのときである。
  BY CODE src/rc_ir/ownership.rs: resolve_callee_params

<1>19. `RcRhs::Llvm(llvm_gen, args)` の腕は、`llvm_gen.borrows_operand(i, &arg_tys, type_env)` が真の
       オペランドを飛ばし、それ以外の各オペランド `args[i]` の各 boxed leaf `leaf` について、
       `passthrough_arg_leaves(llvm_gen, result_ty, args, type_env)` が `(i, leaf)` を含まないときだけ
       `(args[i].name, leaf)` を積む。
  BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm(llvm_gen, args)` の腕

<1>20. `passthrough_arg_leaves` は `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の各 leaf の
       `LeafOrigins` に `as_arg_projection` をかけて `Some((j, p))` になったものを集める。すなわち
       「結果のある leaf の宣言が単一の `Arg(j, p)` である」ような `(j, p)` の集合である。
  BY CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection,
     CODE src/rc_ir/provenance.rs: Provenance::leaves

<1>21. QED
  積まれるものの出どころは、`<1>3` と `<1>14` の腕の列挙により、次の 5 か所で全部である。
  (i) `collect_consumes_go` の `RcExpr::Ret` の腕 (`<1>8`)、
  (ii) `collect_consumes_go` の `RcExpr::Destructure` の腕 (`<1>9`, `<1>10`)、
  (iii) `rhs_consumes` の `RcRhs::Closure` の腕 (`<1>16`)、
  (iv) `rhs_consumes` の `RcRhs::App` の腕 (`<1>17`, `<1>18`)、
  (v) `rhs_consumes` の `RcRhs::Llvm` の腕 (`<1>19`, `<1>20`)。
  BY <1>3, <1>7, <1>8, <1>9, <1>10, <1>11, <1>12, <1>14, <1>15, <1>16, <1>17, <1>19

### P7 (a) D9 の消費はすべて報告される

<1>1. CASE D9 の `App(callee, args)` の行。
  <2>1. `App` は `RcRhs` の 1 種なので `Let(x, App(callee, args), k)` の形でだけ現れ、`L5 の <1>12` より
        `rhs_consumes` が呼ばれる。
    BY D2, L5
  <2>2. D9 の行の前半「callee の全 boxed leaf」は、`L5 の <1>17` の前半が積む。
    BY L5
  <2>3. D9 の行の後半「呼び出し先がその位置の unit を所有する引数の leaf」は、`resolve_callee_params` が
        `Some(params)` のとき `owns(&params[i], &leaf)` が積む。DEF leaf 粒度の所有 より、この述語は
        「`params[i]` の leaf `leaf` の unit が D14 の意味で所有される」ことと同値である。
    BY D14, DEF leaf 粒度の所有, L5
  <2>4. `resolve_callee_params` が `None` のときは全位置を所有として扱う。A7 がこれを所有を増やす向きの
        近似として置いている。
    BY A7, L5
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4
<1>2. CASE D9 の `Closure(f, caps)` の行。D9 の行「各 capture の全 boxed leaf」は `L5 の <1>16` が積む。
  BY D2, L5
<1>3. CASE D9 の `Llvm(gen, args)` の行。D9 の行「`borrows_operand(i)` が偽のオペランドのうち、
      `result_prov` が単一の `Arg(i, σ)` として素通しを宣言していない leaf」は `L5 の <1>19` の条件そのもので
      あり、その「単一の `Arg(i, σ)`」は `L5 の <1>20` の `as_arg_projection` の条件そのものである。
  BY D2, L5
<1>4. CASE D9 の `Destructure(c, fs)` (`c` が boxed) の行。D9 の行「`c` の全 boxed leaf」は
      `L5 の <1>10` の `is_box` が真の場合が返し、`L5 の <1>9` が積む。
  BY L5
<1>5. CASE D9 の `Destructure(c, fs)` (`c` が unbox) の行。D9 の行「名前が付いていないフィールドの leaf」は
      `L5 の <1>10` の `is_box` が偽の場合が返し、`L5 の <1>9` が積む。
  BY L5
<1>6. CASE D9 の「関数本体の終端の `Ret(x)`」の行。
  <2>1. 関数本体の終端の `Ret(x)` は `RcExpr::Ret` の節点であり、`L5 の <1>7` より走査はそれを訪れる。
    BY D3, L5
  <2>2. D9 の行「`x` の全 boxed leaf」は `L5 の <1>8` が積む。
    BY L5
  <2>3. QED
    BY <2>1, <2>2
<1>7. QED
  D9 の消費の表は 6 行からなり、`<1>1` から `<1>6` がその 6 行である。
  BY D9, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### P7 (b) 余分に報告されるのはアーム本体の `Ret` に限る

<1>1. `L5` の出どころ (ii) が積むものは D9 の `Destructure` の 2 行のいずれかである。`L5 の <1>10` の
      2 つの場合は `container.ty.is_box(type_env)` の真偽で尽きており、それぞれ D9 の `Destructure` (boxed) と
      `Destructure` (unbox) の行に等しい。
  BY D9, L5

<1>2. 出どころ (iii) が積むものは D9 の `Closure` の行である。
  BY D9, L5

<1>3. 出どころ (iv) が積むものは D9 の `App` の行である。
  BY D9, L5, P7 (a) の <1>1

<1>4. 出どころ (v) が積むものは D9 の `Llvm` の行である。
  BY D9, L5, P7 (a) の <1>3

<1>5. 出どころ (i) が積むものは、その `Ret` 節点が `collect_consumes` に渡された本体の終端のものなら D9 の
      `Ret` の行であり、そうでないなら `Match` のアーム本体の終端の `Ret` である。
  <2>1. 走査が訪れる `RcExpr::Ret` の節点は、`collect_consumes_go` が呼ばれた本体の終端のものである。
    <3>1. `Ret` は唯一の終端子であり、`Ret` 以外の腕は継続へ進む。
      BY D2, L5
    <3>2. QED
      BY <3>1
  <2>2. `collect_consumes_go` が呼ばれる本体は、`collect_consumes` が渡した本体と、`Let` の腕が
        `RcRhs::Match` のときに渡す各 `arm.body` である。
    BY L5
  <2>3. `collect_consumes` が渡す本体は関数本体である。`collect_consumes` を呼ぶのは `infer_ownership` の
        1 か所だけであり、そこで渡すのは `func.body` である。
    BY D1, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>4. QED
    BY D3, <2>1, <2>2, <2>3

<1>6. QED
  出どころ (ii)-(v) は D9 の消費の行そのものであり (`<1>1` から `<1>4`)、(i) は関数本体の終端の `Ret`
  (D9 の行) か `Match` のアーム本体の終端の `Ret` かのどちらかである (`<1>5`)。
  BY L5, <1>1, <1>2, <1>3, <1>4, <1>5

### 報告しない腕が参照を手放さないこと

<1>1. `rhs_consumes` の `RcRhs::Var(_)` の腕。`Let(x, Var(y), k)` は D9 の移動の表の第 1 行であり、`y` の
      参照は活性化の中で `x` へ移る。D10 の移動の行より `Obl` は変わらないので、この構文は参照を手放さない。
  BY D9, D10, L5

<1>2. `rhs_consumes` の `RcRhs::Match(..)` の腕、および `collect_consumes_go` の `RcExpr::Let` の腕の
      `RcRhs::Match` の場合。D9 は `Match` 節点自身が参照を作らず、移さず、手放さないと述べる。アームの中の
      消費は `L5 の <1>5` の再帰が報告する。
  BY D9, L5

<1>3. `collect_consumes_go` の `RcExpr::Retain` の腕。`Retain` は D10 が直接扱う構文であり、D10 の `Retain` の
      行は `Obl` への追加である。手放す構文ではない。
  BY D8, D9, D10, L5

<1>4. `collect_consumes_go` の `RcExpr::Release` の腕。`Release` は参照を処分するが、D10 は `Release` の行を
      消費の行とは別に持ち、D9 の消費の表に `Release` の行は無い。よって `collect_consumes` が報告しないのは
      D9 に対して正しい。
  BY D9, D10, L5

<1>5. `collect_consumes_go` の `RcExpr::Eval` の腕。D9 は `Eval(v, k)` が参照を作らず、移さず、手放さないと
      述べる。D7 の読む構文の表には入っているが、読みは参照を手放さない。
  BY D7, D9, L5

<1>6. `rhs_consumes` の `RcRhs::Llvm` の腕が `borrows_operand(i)` が真のときに飛ばすオペランド。A3 が
      「`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの参照を処分しない」と置く。
  BY A3, L5

<1>7. `rhs_consumes` の `RcRhs::Llvm` の腕が `passthrough` に入るとして飛ばす leaf。A3 の表の
      「単一の `Arg(j, σ)`」の行が、生成コードはそこに第 `j` オペランドの leaf `σ` と同じ参照を置き、新しい
      参照を作らないと述べる。D9 の移動の表の最後の行がこれを移動とし、D10 の移動の行より `Obl` は変わらない。
  BY A3, D9, D10, L5

<1>8. `destructure_consumes` が unbox 容器について落とす名前付きフィールドの leaf。D9 の移動の表の第 3 行が
      これを移動とし、D10 の移動の行より `Obl` は変わらない。
  BY D9, D10, L5

<1>9. QED
  `<1>1` から `<1>8` が、報告しない箇所の全部である。
  BY L5, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8

### P7 の結論

<1>1. (a) は `P7 (a) の <1>7` である。
  BY P7 (a)
<1>2. (b) は `P7 (b) の <1>6` である。
  BY P7 (b)
<1>3. QED
  BY <1>1, <1>2

## 9. `README.md` へ差し戻す点

### 「別名の道」は定義されていない

P5 (b) の言明は「一方から他方への**別名の道**が `Match` のアーム本体の `Ret` の辺を含まない」と書くが、
`README.md` はこの語を定義しない。D17 が「別名の辺」を `origin` の辿る辺として使っているだけである。この
文書は第 2 節の DEF 別名の道 で、D9 の移動の表の 6 行を辺とする無向グラフの道として定義した。この定義を
D17 の近くへ置くことを提案する。

### D6 のスロットは位置ごとであり、`identity` はそれより長く生きる

D6 は「その時点で束縛されている変数」のスロットだけを認める。ところが `identity` は `Match` のアーム本体が
束縛した変数を名指すことがあり、その変数は `Match` の後の位置ではスコープに無い。1 つのアームしか持たない
`Match` の本体 `Let(m, Match(s, [Let(a, App(f, []), Ret(a))]), k)` では `id(m, []) = (a, [])` であり、
`k` の中の位置に `(a, [])` というスロットは D6 の意味では存在しない。この文書は DEF 路のスロット で、
1 つの実行路の上で値を得た名前についてのスロットに広げた。P3 の言明の「対応するスロット」も同じ広さを要求
する。

### A12 は catch-all アームの payload の型に触れていない

A12 は「payload と変位の型」が合っていることを言うが、catch-all アームの payload と scrutinee の型が
一致することは言わない。`L4 の <1>10` はその代わりに、D9 の移動の表の catch-all の行 (payload 変数が
scrutinee の参照を受け取る) と A5 から leaf の対応を出している。A12 に catch-all の行を足す方が短い。

### P5 (b) の「同じオブジェクトを指す」は使っていない

P5 (b) の証明は前提のうち別名の道の条件だけを使う。E1 から E5 の別名の道で結ばれた 2 つのスロットは、
オブジェクトが同じかどうかを問わず `identity` が等しい。言明を強い側へ書き直せるが、読み手 (P17、P19) が
要るのは現在の形なので、変えなくても困らない。

### P5 (a) が載っている仮定 -- A3 の「複数の元」の行

P5 (a) の証明は、`result_prov` が leaf に置く集合の元数が 0 か 1 であることに載っている (`L4 の <1>5`、
A3)。元数 2 以上の宣言を持つ op が現れると、`origin` は boxed leaf の path から `origin_from_leaves_under`
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
