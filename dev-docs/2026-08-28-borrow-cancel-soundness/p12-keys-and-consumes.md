# P5, P6, P7 -- キーと `acted_references` と消費の網羅性

`README.md` の定義 D1-D14、仮定 A1-A8、および命題 P1-P4 の言明の上に立つ。P1-P4 の証明は
`p10-leaves-and-units.md` と `p11-origin-soundness.md` にあり、この文書はその言明だけを使う。

## 記法

1 つの関数 (またはグローバル初期化子) の本体を固定し、その本体から作られる `VarTable` を `vars`、プログラムの
`TypeEnv` を `type_env` と書く (`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`)。この
2 つは本体ごとに 1 つなので、以下では `origin`、`unit_key`、`acted_references`、`acted_unit_keys` の
第 1・第 2 引数を落として書く。

- `origin(x, π)` は `origin(vars, type_env, x, π)`。
- `id(x, π)` は `origin(x, π).identity()` (`CODE src/rc_ir/ownership.rs: Origin::identity`)。
- `cand(x, π)` は `origin(x, π).candidates()` を集合とみなしたもの
  (`CODE src/rc_ir/ownership.rs: Origin::candidates`)。
- `key(x, π)` は `unit_key(vars, type_env, x, π)` (D14)。
- `L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合。D4 より
  これが「`v` の `π` の下の boxed leaf」の全体である。

`VarPath` は対 `(FullName, FieldPath)` である (`CODE src/rc_ir/ast.rs: VarPath`)。等号はこの対の等号である。

この文書は補題を `L1` から `L5`、証明した命題を `P5a` と呼ぶ。`BY` の行ではそれらを名前で引用し、補題の中の
ステップを指すときは `L1 の <1>3` と書く。

## P5 (キーは参照の関数である)

### 主張が定まらない点

P5 の言明は「1 つの実行路の 1 つの位置において同じ参照を持つ 2 つのスロット」を主語に取る。D6 はスロットを、
D8 は参照を定めるが、**スロットが参照を「持つ」という関係は `README.md` のどの定義も定めていない**。D9 が
「移動とは、参照の持ち手が活性化の中で変わるだけの構文である」と述べるだけである。この一文には 2 通りの読みが
あり、P5 の真偽はそのどちらを取るかで変わる。

- **読み A (持ち手は 1 つ)**: 移動の後、移動元のスロットは参照を持たなくなる。このときどの位置でも 1 つの参照を
  持つスロットは高々 1 つであり、「同じ参照を持つ 2 つのスロット」は同一のスロットに限る。P5 は空虚に真になる。
- **読み B (移動は別名を作る)**: 移動の後も移動元のスロットは参照を持ち続け、移動先が持ち手に加わる。このとき
  P5 は空虚でない。読み B が意図であることは、unbox union の変位アームから分かる。D9 はそのアームの payload
  束縛を移動とするが、アームを抜けた後に scrutinee を `Release` するのは正当な本体である。読み A ではその
  `Release` の時点で scrutinee はどの参照も持たないことになり、`Release` が何を処分するのか言えなくなる。

以下は**読み B** を取る。読み B の下で D8 と D9 と D10 から従うことを、証明の中で使う形に書き下しておく。

**DEF 持ち手集合**
実行路の位置 `p` と参照 `r` について、`p` において `r` を持つスロットの集合 `Hold(p, r)` を次で定める。`r` は
D10 の初期値・生成の表・`Retain` のいずれかによって 1 つのスロットに生じる (`Retain(v, π)` が leaf `λ` に
ついて作る参照はスロット `(v, λ)` に生じるものとする)。そのスロットを `Hold(p, r)` の元とし、`p` までに
実行された D9 の移動の各辺について、辺の始点が `Hold(p, r)` に入っているなら終点も入れる。`p` において
束縛されていない変数のスロットは除く (D6)。

D9 の移動の表の 5 行を、辺として次の名前で呼ぶ。

- **E1** (`Let(x, Var(y), k)`): 各 leaf `λ` について `(y, λ)` から `(x, λ)` へ。
- **E2** (`Destructure(c, fs, s, k)` で `c` が unbox、`(i, f)` が `fs` の元): 各 leaf `λ` について
  `(c, [i] ++ λ)` から `(f, λ)` へ。
- **E3** (`Match(s, arms)` のアームの payload 束縛): `s` が unbox union で変位番号が `t` のとき、各 leaf `λ`
  について `(s, [t] ++ λ)` から `(p, λ)` へ。catch-all のとき、各 leaf `λ` について `(s, λ)` から `(p, λ)` へ。
- **E4** (`Let(x, Llvm(gen, args), k)` の素通し leaf): 結果の leaf `λ` の `result_prov` が単一の
  `Arg(j, σ)` のとき、`(args[j], σ)` から `(x, λ)` へ。
- **E5** (`Match` のアームの `Ret(x)`): 束縛変数を `m` として、各 leaf `λ` について `(x, λ)` から `(m, λ)` へ。

### L1 (`unit_key` は identity の関数である)

**言明**。`id(x, π) = id(y, ρ)` ならば `key(x, π) = key(y, ρ)` である。また `key(x, π)` の根は
`id(x, π)` の根に等しい。

<1>1. `key(x, π) = unit_of(vars, type_env, id(x, π))` である。
  BY D14, CODE src/rc_ir/ownership.rs: unit_key

<1>2. `unit_of(vars, type_env, w)` の値は `(vars, type_env, w)` だけで決まり、その根は `w` の根に等しい。
  <2>1. `unit_of` は引数の `VarPath` を `(root, path)` に分解し、`vars.var_tys.get(root)` で 2 つの腕に
        分かれる。
    BY CODE src/rc_ir/ownership.rs: unit_of
  <2>2. CASE `vars.var_tys.get(root)` が `None`。
    <3>1. この腕は `root.is_local()` が偽であることを表明したのち `(root.clone(), path.clone())` を返す。
      BY CODE src/rc_ir/ownership.rs: unit_of
    <3>2. QED
      返り値は `(vars, type_env, w)` だけで決まり、その根は `root` である。
      BY <3>1
  <2>3. CASE `vars.var_tys.get(root)` が `Some(ty)`。
    <3>1. この腕は `truncate_to_unit(ty, path, type_env)` を計算し、`rc_units(ty, type_env)` がそれを
          含むことを表明したのち `(root.clone(), truncated)` を返す。
      BY CODE src/rc_ir/ownership.rs: unit_of
    <3>2. `truncate_to_unit(ty, path, type_env)` の値は `(ty, path, type_env)` だけで決まる。
      <4>1. `truncate_to_unit` は `path` の各添字について `unit_step(cur, type_env)` で分岐し、`cur` を
            `held_field_type` で更新するほかに状態を持たない。
        BY CODE src/rc_ir/ownership.rs: truncate_to_unit
      <4>2. `unit_step(ty, type_env)` の値は `(ty, type_env)` だけで決まる。
        BY CODE src/rc_ir/ownership.rs: unit_step
      <4>3. `held_field_type(held_fields, idx, walk_name)` の値は `(held_fields, idx)` だけで決まる。
        BY CODE src/rc_ir/ownership.rs: held_field_type
      <4>4. QED
        BY <4>1, <4>2, <4>3
    <3>3. `ty` は `vars.var_tys.get(root)` の値なので `(vars, root)` で決まる。
      BY <2>1
    <3>4. QED
      返り値は `(vars, type_env, w)` だけで決まり、その根は `root` である。
      BY <3>1, <3>2, <3>3
  <2>4. QED
    2 つの腕は `vars.var_tys.get(root)` が `None` か `Some` かで尽きている。
    BY <2>1, <2>2, <2>3
<1>3. QED
  BY <1>1, <1>2

**補足 (注意点 1 について)**。`unit_of` の 2 つの腕のどちらを通るかは `root` だけで決まり、切り詰めの結果は
`(ty, path, type_env)` だけで決まる (`L1 の <1>2`)。よって切り詰めは、等しい identity を異なるキーへ分ける
ことができない。切り詰めが**異なる** identity を 1 つのキーへ**まとめる**ことはある (unbox union の変位の下の
leaf は union 自身のキーへ落ちる。D5) が、P5 は「同じ参照ならば同じキー」の向きしか主張しないので、まとめる
向きは P5 に反しない。

### L2 (E1-E4 の辺は `origin` を保つ)

**言明**。E1、E2、E3、E4 のいずれかの辺で結ばれた 2 つのスロットは、同じ `origin` を持つ。

<1>1. `collect_bindings` は、`Let(x, Var(y), k)` に対し `x` の `Binding` を `Move(y)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Var(y)` の場合

<1>2. `collect_bindings` は、`Destructure(container, fields, _state, k)` の各 `(idx, fv)` に対し `fv` の
      `Binding` を `Field(container, idx)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Destructure` の腕

<1>3. `collect_bindings` は、`Let(x, Match(scrut, arms), k)` の各アームの `payload` に対し `Binding` を
      `Payload(scrut, arm.tag)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Match` の場合

<1>4. `collect_bindings` は、`Let(x, Llvm(llvm_gen, args), k)` に対し `x` の `Binding` を
      `Llvm(llvm_gen, args, x.ty)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Llvm` の場合

<1>5. E1 の辺 `(y, λ) -> (x, λ)` について `origin(x, λ) = origin(y, λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Move(y))` の腕は `origin(vars, type_env, &y.name, path)` を
        そのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕
  <2>2. QED
    BY <1>1, <2>1

<1>6. E2 の辺 `(c, [i] ++ λ) -> (f, λ)` について `origin(f, λ) = origin(c, [i] ++ λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Field(container, idx))` の腕は、`container.ty.is_box(type_env)`
        が偽のとき `container_path` を `[*idx] ++ path` として作り
        `origin(vars, type_env, &container.name, &container_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕
  <2>2. E2 の辺は `c` が unbox のときにだけ引かれるので、`container.ty.is_box(type_env)` は偽である。
    BY D9, DEF 持ち手集合
  <2>3. QED
    BY <1>2, <2>1, <2>2

<1>7. E3 の辺について `origin(p, λ) = origin(s, [t] ++ λ)` (変位アーム) または
      `origin(p, λ) = origin(s, λ)` (catch-all) である。
  <2>1. CASE `arm.tag` が `None` (catch-all)。
    <3>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の `None` の場合は
          `origin(vars, type_env, &scrut.name, path)` をそのまま返す。
      BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
    <3>2. QED
      BY <1>3, <3>1
  <2>2. CASE `arm.tag` が `Some(t)` かつ `scrut.ty.is_box(type_env)` が偽。
    <3>1. `origin_inner` の同じ腕の `Some(tag) if !scrut.ty.is_box(type_env)` の場合は `scrut_path` を
          `[*tag] ++ path` として作り `origin(vars, type_env, &scrut.name, &scrut_path)` をそのまま返す。
      BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
    <3>2. QED
      BY <1>3, <3>1
  <2>3. QED
    E3 の辺は D9 の移動の表により、catch-all のアームと unbox union の変位アームにだけ引かれる。boxed union
    の変位アームは D9 の移動の表に無く、D10 の生成の表の「boxed union の変位アームの payload の各 leaf」の
    行にある。`origin_inner` の同じ腕の `Some(_)` の場合 (scrutinee が boxed) が `here()` を返して
    producer になることがそれに対応する。よって <2>1 と <2>2 で場合は尽きている。
    BY D9, D10, CODE src/rc_ir/ownership.rs: origin_inner の
       `Some(Binding::Payload(scrut, variant))` の腕, <2>1, <2>2

<1>8. E4 の辺 `(args[j], σ) -> (x, λ)` について `origin(x, λ) = origin(args[j], σ)` である。
  <2>1. `origin_inner` の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕は、`decl` を
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` として作り、
        `decl.leaf_origins_at(path).and_then(as_arg_projection)` が `Some((j, p))` のとき
        `origin(vars, type_env, &args[j].name, &p)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕
  <2>2. `as_arg_projection(sources)` が `Some((j, p))` を返すのは、`sources` がちょうど 1 元からなり、その元が
        `LeafOrigin::Arg(j, p)` のときに限る。
    BY CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>3. E4 の辺は結果の leaf `λ` の `result_prov` が単一の `Arg(j, σ)` のときにだけ引かれるので、
        `decl.leaf_origins_at(λ)` は 1 元集合 `{Arg(j, σ)}` であり、`as_arg_projection` はそれを
        `Some((j, σ))` に写す。
    BY D9, DEF 持ち手集合, <2>2
  <2>4. QED
    BY <1>4, <2>1, <2>3

<1>9. QED
  E1-E4 の各辺の両端は同じ `origin` を持つ。
  BY <1>5, <1>6, <1>7, <1>8

### L3 (E5 の辺は candidates を保つが identity を保たない)

**言明**。E5 の辺 `(x, λ) -> (m, λ)` について、`cand(x, λ)` は `cand(m, λ)` に含まれる。一方
`id(x, λ) = id(m, λ)` は成り立たないことがある。

<1>1. `collect_bindings` は、`Let(x, Match(scrut, arms), k)` に対し `x` の `Binding` を
      `Join(arm_results)` とする。ここで `arm_results` は各アームについての `returned_var(&arm.body)` の列で
      ある。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Match` の場合

<1>2. `returned_var(&arm.body)` は、アーム本体の終端の `Ret` が名指す変数である。
  BY CODE src/rc_ir/ownership.rs: returned_var

<1>3. `origin_inner` の `Some(Binding::Join(arm_results))` の腕は、各 `arm_result` について
      `origin(vars, type_env, &arm_result.name, path).candidates()` の元を集めて `candidates` とし、
      `Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))` を返す。
  BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕

<1>4. `Origin::of_candidates(candidates, identity)` は、`candidates` が 1 元のときその元の `Exactly`、
      2 元以上のとき `Join { identity, candidates }` を返す。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates

<1>5. `Origin::of_candidates(candidates, identity)` が返す `Origin` の `candidates()` は `candidates` に
      等しい。
  <2>1. CASE `candidates` が 1 元のとき。返り値は `Exactly(c)` であり、`Origin::candidates` はその
        `Exactly(p)` の腕で `vec![p]` を返す。
    BY <1>4, CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>2. CASE `candidates` が 2 元以上のとき。返り値は `Join { identity, candidates }` であり、
        `Origin::candidates` はその `Join` の腕で `candidates` を返す。
    BY <1>4, CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>3. QED
    `Origin` は `Exactly` と `Join` の 2 種であり、`of_candidates` は `candidates` の元数が 1 か 2 以上かで
    この 2 つを返し分ける。`of_candidates` は空の `candidates` を表明で弾く。
    BY CODE src/rc_ir/ownership.rs: Origin, Origin::of_candidates, <1>4, <2>1, <2>2

<1>6. `cand(x, λ)` は `cand(m, λ)` に含まれる。
  <2>1. `x` は `m` を束縛する `Match` のいずれかのアームの `returned_var` である。
    BY D9, DEF 持ち手集合, <1>1, <1>2
  <2>2. `<1>3` が集める `candidates` は、そのアームの寄与として `cand(x, λ)` の全元を含む。
    BY <1>3, <2>1
  <2>3. QED
    BY <1>3, <1>5, <2>2

<1>7. `id(x, λ) = id(m, λ)` が成り立たない `VarTable` がある。
  <2>1. `p` と `q` の `Binding` が `Producer`、`m` の `Binding` が `Join([p, q])` である `VarTable` に
        ついて、`origin(m, [])` の `identity()` は `(m, [])` である。
    BY CODE src/rc_ir/ownership.rs: tests::a_match_binding_may_be_any_arm_result
  <2>2. 同じ形の `VarTable` について、`Binding` が `Producer` である変数の `origin` は
        `Exactly` のその変数自身であり、その `identity()` は `(p, [])` である。
    BY CODE src/rc_ir/ownership.rs: tests::a_producer_is_exactly_itself,
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Producer)` の腕
  <2>3. QED
    `(p, [])` と `(m, [])` は根が異なるので `VarPath` として異なる。
    BY A6, <2>1, <2>2

<1>8. QED
  BY <1>6, <1>7

### L4 (boxed leaf は互いに前置にならない)

**言明**。型 `τ` について、`boxed_leaf_paths(τ, type_env)` の相異なる 2 元の一方が他方の前置になることは
ない。とくに `π` が leaf であるとき `L(v, π)` は `{π}` である。

<1>1. `boxed_leaf_paths` の内部の走査 `go` は、`ty.is_closure()`、`ty.is_box(type_env)`、`ty.is_array()` の
      各腕で `out` に path を積んだ直後に `return` する。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `go` が path を積むのはその 3 つの腕だけであり、`ty.is_fully_unboxed(type_env)` の腕は積まずに
      `return` し、最後の腕は `unpunched_field_types` の各フィールドへ降りる。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. 積まれた path `P` について、`P` を真の前置に持つ path が積まれることはない。
  <2>1. `P` が積まれたのは `go` が `P` に対応する型に着いたときであり、その腕は積んだ直後に `return` する。
    BY <1>1, <1>2
  <2>2. `P` を真の前置に持つ path が積まれるには、`go` が `P` に対応する型からさらにフィールドへ降りる必要が
        ある。
    BY <1>2
  <2>3. QED
    BY <2>1, <2>2
<1>4. QED
  `π` が leaf であるとき、`π` を前置に持つ leaf は `π` 自身だけである。
  BY <1>3

### P5a (E5 を通らない道についての P5)

**言明**。1 つの活性化の 1 つの実行路の 1 つの位置において同じ参照を持つ 2 つのスロットが、E1-E4 の辺だけから
なる道で結ばれているなら、その 2 つは同じ `unit_key` を持つ。

<1>1. 2 つのスロットを `(x, λ)`、`(y, μ)` とし、両者が位置 `p` で参照 `r` を持つとする。すなわち
      `(x, λ)` と `(y, μ)` は `Hold(p, r)` の元である。
  BY DEF 持ち手集合

<1>2. `Hold(p, r)` の元は、`r` の生じたスロットを根とし D9 の移動の辺を辺とする有向木の節点である。
  <2>1. `Hold(p, r)` は `r` の生じた 1 つのスロットから、D9 の移動の辺で閉じて作られる。
    BY DEF 持ち手集合
  <2>2. 1 つの実行路の上では、各スロットは高々 1 つの実行された移動の辺の終点である。
    <3>1. スロット `(z, ν)` の変数 `z` は、A6 よりプログラム中でちょうど 1 か所で束縛される。
      BY A6
    <3>2. D9 の移動の表の 5 行はいずれも、終点のスロットの変数を束縛する構文に付随する。
      BY D9
    <3>3. `Match` の束縛変数へ入る辺は各アームから 1 本ずつあるが、D3 より 1 つの実行路はアームを 1 つだけ
          選ぶので、実行される辺は 1 本である。
      BY D3
    <3>4. QED
      BY <3>1, <3>2, <3>3
  <2>3. QED
    BY <2>1, <2>2

<1>3. `(x, λ)` と `(y, μ)` は、その木の辺だけからなる道で結ばれる。
  BY <1>2

<1>4. P5a の仮定より、その道の辺はすべて E1-E4 である。
  BY <1>3

<1>5. 道の各辺の両端は同じ `origin` を持つ。
  BY L2, <1>4

<1>6. `origin(x, λ) = origin(y, μ)` であり、したがって `id(x, λ) = id(y, μ)` である。
  BY <1>5, CODE src/rc_ir/ownership.rs: Origin::identity

<1>7. QED
  BY L1, <1>6

### `README.md` の P5 は閉じない

P5 の言明には道の辺を E1-E4 に限る仮定が無く、E5 の辺を含む道が残る。`L3 の <1>7` は、E5 の辺の両端が異なる
identity を持ちうることを、`ownership.rs` のテストが表明している `VarTable` の上で示している。読み B の下で、
E5 の辺の両端が 1 つの位置で同時にスロットである本体は書ける。`Match` のアームの `Ret` が名指す変数が、その
`Match` より外で束縛されていればよい。読み A を取れば P5 は空虚に真になるが、読み A では unbox union の変位
アームの後の scrutinee の `Release` が説明できない。よって P5 は、言明のままでは、空虚に真か偽かのどちらかで
ある。`README.md` へ差し戻す点は最後の節にまとめる。

### unbox union の二重命名 (P5 が主張していないこと)

P5 が言うのは「同じ**参照**を持つスロットは同じキー」であって「同じ**オブジェクト**に触れる操作は同じキー」では
ない。この違いを、`acted_references` と `unit_key` が読む path の違いとして書き下す。

<1>1. `unit_key(v, π)` は `origin(v, π)` を、`acted_references(v, π)` は `L(v, π)` の各 leaf `λ` について
      `origin(v, λ)` を読む。`π` が leaf でないとき、この 2 つは別の path の `origin` である。
  BY D14, CODE src/rc_ir/ownership.rs: unit_key, acted_references

<1>2. `π` が leaf でない RC unit であるのは、`π` が unbox union を名指すときと punched array を名指すときで
      ある。
  <2>1. `rc_units_go` が unit を積むのは `unit_step` が `Capture` か `Unit` を返した path である。
    BY CODE src/rc_ir/ownership.rs: rc_units_go
  <2>2. `unit_step` が `Capture` を返すのは closure のときで、そのとき積まれる path は
        `path ++ [capture_idx]` である。`boxed_leaf_paths` も closure に対して同じ
        `path ++ [CLOSURE_CAPTURE_IDX]` を積む。よってこの unit は leaf でもある。
    BY CODE src/rc_ir/ownership.rs: rc_units_go, unit_step,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `unit_step` が `Unit` を返すのは `ty.is_box(type_env)`、`ty.is_union(type_env)`、`ty.is_array()`、
        `ty.is_punched_array()` のいずれかのときである。
    BY CODE src/rc_ir/ownership.rs: unit_step
  <2>4. `is_box` と `is_array` のとき、`boxed_leaf_paths` も同じ path を積んで降りない。よってこの unit は
        leaf でもある。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>5. `is_union` (unbox union) のとき、`boxed_leaf_paths` は `is_box` でも `is_array` でもないので
        `unpunched_field_types` の各変位へ降りる。よってこの unit の path は leaf ではない。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>6. `is_punched_array` のとき、`boxed_leaf_paths` は同じ理由で `unpunched_field_types` の各フィールドへ
        降りる。よってこの unit の path は leaf ではない。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>7. QED
    `UnitStep` は `NoUnit`、`Capture`、`Unit`、`Fields` の 4 種であり、unit が積まれるのは `Capture` と
    `Unit` のときだけである。
    BY CODE src/rc_ir/ownership.rs: UnitStep, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>3. unbox union `s` の変位 `t` のアームの payload `p` について、スロット `(p, λ)` とスロット
      `(s, [t] ++ λ)` は同じキーを持つ。
  <2>1. この 2 つは E3 の辺で結ばれる。
    BY D9, DEF 持ち手集合
  <2>2. QED
    BY L1, L2, <2>1

<1>4. 一方で、`Retain(s, π)` の節点のキー `unit_key(s, π)` と、`Release(w, ρ)` の節点のキー
      `unit_key(w, ρ)` は、その 2 つが 1 つのオブジェクトに触れていても異なりうる。
  <2>1. `InlineLLVMMakeUnionBody` の `result_prov` は、unbox union の結果について、構築される変位の leaf に
        `Arg(0, rest)` を、他の変位の leaf に空集合を宣言する。
    BY CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov
  <2>2. `un` を `Let(un, Llvm(MakeUnion_0, [tw]), k)` で束縛し、`un` の型が boxed leaf を 2 つ持つ変位を
        第 0 変位に持つ unbox union であるとする。このとき `decl.leaf_origins_at([])` は `None` である。
    <3>1. `Provenance::leaf_origins_at(path)` は、`path` がその値の boxed leaf でないとき `None` を返す。
      BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at
    <3>2. `[]` は unbox union の boxed leaf ではない。
      BY <1>2 の <2>5
    <3>3. QED
      BY <3>1, <3>2
  <2>3. よって `origin_inner` の `Llvm` の腕は `origin_from_leaves_under` へ進む。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕,
       <2>2
  <2>4. `origin_from_leaves_under` は、`path` の下の leaf の `Arg(j, leaf)` を
        `(j, truncate_to_unit(args[j].ty, leaf, type_env))` に写して `operand_units` に集め、各 unit の
        `origin` を `reached` に並べる。`reached` の元がすべて等しいときはその `Origin` をそのまま返し、
        そうでないときは `reached` の各元の `candidates()` を集めて
        `Origin::of_candidates(candidates, here)` を返す。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>5. `tw` の 2 つの leaf の `origin` が異なるとき、`reached` は 2 つの異なる `Origin` を持つので、
        `origin(un, [])` は `Join { identity: (un, []), .. }` であり `id(un, []) = (un, [])` である。
    BY <2>4, L3 の <1>4, CODE src/rc_ir/ownership.rs: Origin::identity
  <2>6. `unit_key(un, [])` の根は `un` であり、`tw` の leaf のスロットのキーの根は `tw` である。A6 より
        `un` と `tw` は相異なる変数なので、この 2 つのキーは異なる。
    BY A6, L1, <2>5
  <2>7. QED
    `Retain(un, [])` と、`tw` の leaf を名指す `Release` は、`un` の payload が持つ参照という同じ
    オブジェクトの参照に触れながら、異なるキーを持つ。
    BY <2>5, <2>6

<1>5. QED
  `<1>3` と `<1>4` が主張である。**スロット**のキーは E3 の辺を越えて一致するが、**節点**のキーは一致しなくて
  よい。節点のキーは unit path の `origin` から作られ、その unit path は leaf でないことがある (`<1>2`)。
  BY <1>3, <1>4

### `check_one_key_per_object` の境界との照合

`check_one_key_per_object` の doc は次の 3 つを述べる
(`CODE src/rc_ir/borrow.rs: check_one_key_per_object`)。

- (i) 同じ参照に作用する 2 つの操作は同じオブジェクトに作用し、1 つのオブジェクトは 1 つの unit key で数えられる。
- (ii) unbox union の retain はその payload が持つ参照を全部上げ、その payload の値の release はそのうち 1 つを
  下げる。これは 2 つのキー・1 つのオブジェクトであって、どちらの命名も正しい。
- (iii) そのような release は真部分集合を un-bump するので `un_bump` は `OutsideBracket` を返す。よって
  `References` が**等しい**場合だけが不一致である。

<1>1. (ii) と (iii) が述べる「2 つのキー」は、`unbox union の二重命名` の `<1>4` が示した**節点**のキーの
      違いである。P5 が述べるスロットのキーは、同じ節に `<1>3` として示したとおり E3 の辺を越えて一致する。
      よって (ii)(iii) は P5 と矛盾しない。
  BY L1, L2

<1>2. (i) の後半「1 つのオブジェクトは 1 つの unit key で数えられる」は、`unbox union の二重命名` の
      `<1>4` により偽である。P5 が主張するのは「1 つの参照を持つ 1 つのスロットは 1 つの unit key を持つ」で
      あって、オブジェクトについての主張ではない。
  BY A6, L1

<1>3. 表明が発火しないことは、両方の節点の path が boxed leaf のときには P5 の道具立てから従う。
  <2>1. `Retain(v, π)` と `Release(w, ρ)` の path がどちらも boxed leaf のとき、`acted_references` は
        それぞれ 1 元の多重集合 `{id(v, π): 1}`、`{id(w, ρ): 1}` を返す。
    BY L4, CODE src/rc_ir/ownership.rs: acted_references
  <2>2. その 2 つが等しいなら `id(v, π) = id(w, ρ)` であり、`unit_key(v, π) = unit_key(w, ρ)` である。
    BY L1, <2>1
  <2>3. QED
    `check_one_key_per_object` はキーが異なる pending retain についてだけ表明するので、`<2>2` の下では
    発火しない。
    BY CODE src/rc_ir/borrow.rs: check_one_key_per_object, <2>2

<1>4. `π` または `ρ` が leaf でない unit のときは、`<1>3` の議論は使えない。
  <2>1. 節点のキーは path `π` の `origin` から、`References` は `π` の下の leaf の `origin` から作られる。
    BY CODE src/rc_ir/ownership.rs: unit_key, acted_references
  <2>2. この 2 つを結ぶ主張は P1 から P5 のどれでもない。
    BY P1, P2, P3, P4
  <2>3. QED
    BY <2>1, <2>2

<1>5. QED
  照合の結果は次のとおりである。doc の (ii)(iii) は P5 と矛盾しない (`<1>1`)。doc の (i) の後半の言い方は、
  そのままでは `unbox union の二重命名` の `<1>4` に反する (`<1>2`)。表明が発火しないことは、両方の path が
  boxed leaf のときには従う (`<1>3`) が、leaf でない unit のときは P1 から P5 では埋まらない (`<1>4`)。
  BY <1>1, <1>2, <1>3, <1>4

## P6 (`acted_references` の正しさ)

P6 の言明の 2 つの辺は住む集合が違う。`acted_references` が返す `References` は `VarPath` (identity) 上の
多重集合であり (`CODE src/rc_ir/ownership.rs: References`)、`Retain(v, π)` が作るのは D8 の意味の参照の
多重集合である。「一致する」を、次の対応の下での一致と読む。

**DEF 名前づけ**
`Retain(v, π)` が leaf `λ` (`λ` は `L(v, π)` の元) について作る参照を `r_λ` と書き、その名前を `id(v, λ)` と
する。`Release(v, π)` が leaf `λ` について処分する参照を `r'_λ` と書き、その名前も `id(v, λ)` とする。

<1>1. `acted_references(v, path)` は、`boxed_leaf_paths(&v.ty, type_env)` の要素 `leaf` のうち
      `leaf.starts_with(path)` を満たすものについて、`origin(vars, type_env, &v.name, &leaf).identity()`
      をキーとする計数を 1 ずつ増やした `References` を返す。
  BY CODE src/rc_ir/ownership.rs: acted_references

<1>2. `<1>1` の走査が回る `leaf` の全体は `L(v, π)` である。
  BY D4, DEF 名前づけ, <1>1

<1>3. `acted_references(v, π)` は多重集合として `{ id(v, λ) : λ が L(v, π) の元 }` である。
  <2>1. `<1>1` の走査は `L(v, π)` の各 `λ` について、キー `id(v, λ)` の計数をちょうど 1 増やす。
    BY <1>1, <1>2
  <2>2. `References` は `Map<VarPath, usize>` であり、`covers` は個数の比較で、`subtract` は個数の減算で
        定義される。すなわち多重集合として読む。
    BY CODE src/rc_ir/ownership.rs: References, References::covers, References::subtract
  <2>3. QED
    BY <2>1, <2>2

<1>4. `Retain(v, π)` が `Obl` に加える参照は、`L(v, π)` の各 `λ` についてちょうど 1 つずつである。
  <2>1. D10 の `Retain` の行は「`π` の下の各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ加える」である。
    BY D10
  <2>2. 「`π` の下の各 leaf」の全体は `L(v, π)` である。
    BY D4, <1>2
  <2>3. A5 より、スロット `(v, λ)` が持つ参照は leaf ごとにちょうど 1 つである。
    BY A5
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>5. `Release(v, π)` が `Obl` から取り除く参照は、`L(v, π)` の各 `λ` についてちょうど 1 つずつである。
  <2>1. D10 の `Release` の行は「`π` の下の各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ取り除く」である。
    BY D10
  <2>2. 「`π` の下の各 leaf」の全体は `L(v, π)` である。
    BY D4, <1>2
  <2>3. A5 より、スロット `(v, λ)` が持つ参照は leaf ごとにちょうど 1 つである。
    BY A5
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>6. `λ ↦ r_λ` は `L(v, π)` から `Retain(v, π)` が作る参照の多重集合への全単射である。
  BY DEF 名前づけ, <1>4

<1>7. `λ ↦ r'_λ` は `L(v, π)` から `Release(v, π)` が処分する参照の多重集合への全単射である。
  BY DEF 名前づけ, <1>5

<1>8. `id(v, λ)` は参照 `r_λ` および `r'_λ` の名前である。すなわち `origin(v, λ)` が `Exactly(u, σ)` なら
      それらはスロット `(u, σ)` が持つ参照と同一であり、`Join { identity, .. }` なら `identity` はその参照に
      対してどの実行路でも同じ名前である。
  BY P3, P4

<1>9. `Retain(v, π)` が作る参照の多重集合の、DEF 名前づけ の下での像は、多重集合として
      `{ id(v, λ) : λ が L(v, π) の元 }` である。
  BY DEF 名前づけ, <1>6, <1>8

<1>10. `Release(v, π)` が処分する参照の多重集合の、DEF 名前づけ の下での像も、多重集合として
       `{ id(v, λ) : λ が L(v, π) の元 }` である。
  BY DEF 名前づけ, <1>7, <1>8

<1>11. QED
  `<1>3` と `<1>9` より `acted_references(v, π)` は `Retain(v, π)` が作る参照の多重集合の像に等しく、
  `<1>3` と `<1>10` より `Release(v, π)` が処分する参照の多重集合の像にも等しい。
  BY <1>3, <1>9, <1>10

**補足 1 (2 つの leaf が 1 つの名前を持つとき)**。`L(v, π)` の相異なる 2 つの leaf が同じ `id` を持つことが
ある。`References` はそのとき計数を 2 にする (`<1>3` の `<2>1`)。これは 1 つのオブジェクトへの参照を 2 つ持つ値
(`MakeStruct(a, a)` の結果など) に対応し、A5 の下で参照は leaf ごとに 1 つなので、参照としても 2 つある。

**補足 2 (identity は切り詰められていない)**。`acted_references` のキーは `origin(...).identity()` であって
`unit_of` を通していない (`<1>1`)。節点のキー `unit_key` は通している (D14)。この 2 つを混ぜてはならない。

**補足 3 (A4 と unbox union)**。`<1>4` と `<1>5` は D2 と D10 の行をそのまま使っている。A4 はコード生成が
その行のとおりであると置くが、unbox union についてはそうなっていない。`RcExpr::Retain` のコード生成は
`project_rc_unit` の後に `build_retain` を呼び、`build_retain` は unbox 値の `UnionBuf` フィールドに対して
`retain_union` を呼び、`retain_union` は `retain_release_mark_union` を通じてタグを読み、**活性な変位だけ**を
retain する (`CODE src/rc_ir/codegen.rs: RcCodegen::eval_rc_expr` の `RcExpr::Retain` の腕、
`CODE src/generator.rs: Generator::build_retain`、`CODE src/object.rs: ObjectFieldType::retain_union`,
`ObjectFieldType::retain_release_mark_union`)。よって参照を持つ変位を 2 つ以上持つ unbox union
(`Result e o` で `e` と `o` がどちらも参照を持つ場合など) では、実際に上がる参照の個数は `L(v, π)` の元数より
少ない。ずれは片側だけであり、`acted_references` は retain 側でも release 側でも同じように多く数えるので、
`un_bump` の `covers` は真になりにくくなる方向にしか動かない。最後の節に差し戻す。

## P7 (消費の網羅性)

P7 の 2 つの主張を次のように書く。

- **(a)** D9 の消費の表の各行が指す leaf は、`collect_consumes` が `out` に積む。
- **(b)** `collect_consumes` が `out` に積むもののうち D9 の消費の表に無いものは、`Match` のアームの終端の
  `Ret` が積むものに限る。

`collect_consumes` の `own` 引数は、D14 の所有を leaf の粒度で述べた集合とする
(`CODE src/rc_ir/borrow.rs: OwnedLeaves` がこの粒度であることを述べている)。以下 `owns(p, λ)` はその集合への
所属である。

### L5 (走査はすべての節点を 1 度ずつ訪れ、積むものはこれで全部である)

**言明**。`collect_consumes` の走査は、渡された本体とそのすべてのアーム本体の各節点をちょうど 1 度訪れる。
`out` に積まれるものは、下に列挙する 5 か所のいずれかから来る。

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
  D2 より本体は木であり、節点の継続は 1 つ、分岐は `Match` のアームだけである。`<1>4` と `<1>5` はその継続と
  アームをちょうど 1 度ずつたどり、`<1>6` が終端である。
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
  BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App(callee, args)` の腕, push_boxed_leaves

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
       「結果のある leaf の唯一の源が引数 `j` の leaf `p` である」ような `(j, p)` の集合である。
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
  <2>3. D9 の行の後半「呼び出し先がその位置を所有する引数の leaf」は、`resolve_callee_params` が
        `Some(params)` のとき `owns(&params[i], &leaf)` が積む。`owns` は D14 の所有を leaf 粒度で述べた
        集合への所属としたので、これは D9 の行そのものである。
    BY D14, L5
  <2>4. `resolve_callee_params` が `None` のときは全位置を所有として扱う。A7 がこれを安全側の近似として
        置いている。
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
    BY L5
  <2>2. D9 の行「`x` の全 boxed leaf」は `L5 の <1>8` が積む。
    BY L5
  <2>3. QED
    BY <2>1, <2>2
<1>7. QED
  D9 の消費の表は 6 行からなり、`<1>1` から `<1>6` がその 6 行である。
  BY D9, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### P7 (b) 余分に報告されるのはアームの `Ret` に限る

<1>1. `L5` の出どころ (ii) が積むものは D9 の `Destructure` の 2 行のいずれかである。
  `L5 の <1>10` の 2 つの場合は `container.ty.is_box(type_env)` の真偽で尽きており、それぞれ D9 の
  `Destructure` (boxed) と `Destructure` (unbox) の行に等しい。
  BY D9, L5

<1>2. 出どころ (iii) が積むものは D9 の `Closure` の行である。
  BY D9, L5

<1>3. 出どころ (iv) が積むものは D9 の `App` の行である。
  BY D9, L5, P7 (a) の <1>1

<1>4. 出どころ (v) が積むものは D9 の `Llvm` の行である。
  BY D9, L5, P7 (a) の <1>3

<1>5. 出どころ (i) が積むものは、その `Ret` 節点が `collect_consumes` に渡された本体の終端のものなら
      D9 の `Ret` の行であり、そうでないなら `Match` のアームの終端の `Ret` である。
  <2>1. 走査が訪れる `RcExpr::Ret` の節点は、`collect_consumes_go` が呼ばれた本体の終端のものである。
    <3>1. `Ret` は唯一の終端であり、`Ret` 以外の腕は継続へ進む。
      BY D2, L5
    <3>2. QED
      BY <3>1
  <2>2. `collect_consumes_go` が呼ばれる本体は、`collect_consumes` が渡した本体と、`Let` の腕が
        `RcRhs::Match` のときに渡す各 `arm.body` である。
    BY L5
  <2>3. `collect_consumes` が渡す本体は関数本体である。
    BY D1, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>6. QED
  出どころ (ii)-(v) は D9 の消費の行そのものであり (`<1>1` から `<1>4`)、(i) は関数本体の終端の `Ret`
  (D9 の行) か `Match` のアームの終端の `Ret` かのどちらかである (`<1>5`)。よって D9 の消費の表に無いものは
  アームの `Ret` に限る。
  BY L5, <1>1, <1>2, <1>3, <1>4, <1>5

### 報告しない腕が参照を手放さないこと

<1>1. `rhs_consumes` の `RcRhs::Var(_)` の腕。`Let(x, Var(y), k)` は D9 の移動の表の第 1 行であり、`y` の
      参照は活性化の中で `x` へ移る。D10 の移動の行より `Obl` は変わらないので、この構文は参照を手放さない。
  BY D9, D10, L5

<1>2. `rhs_consumes` の `RcRhs::Match(..)` の腕、および `collect_consumes_go` の `RcExpr::Let` の腕の
      `RcRhs::Match` の場合。`Match` の節点自身は D9 の消費の表にも移動の表にも無い。アームの中の消費は
      `L5 の <1>5` の再帰が報告する。
  BY D9, L5

<1>3. `collect_consumes_go` の `RcExpr::Retain` の腕。`Retain` は D8 により参照を**作る**構文であり、D10 の
      `Retain` の行が `Obl` への追加として扱う。手放す構文ではない。
  BY D8, D10, L5

<1>4. `collect_consumes_go` の `RcExpr::Release` の腕。`Release` は参照を処分するが、D10 は `Release` の行を
      消費の行とは別に持ち、D9 の消費の表に `Release` の行は無い。よって `collect_consumes` が報告しないのは
      D9 に対して正しい。
  BY D9, D10, L5

<1>5. `collect_consumes_go` の `RcExpr::Eval` の腕。`Eval(v, k)` は D7 の読む構文の表の 1 行であり、D9 の
      消費の表にも移動の表にも無い。よって参照を手放さない。
  BY D7, D9, L5

<1>6. `rhs_consumes` の `RcRhs::Llvm` の腕が `borrows_operand(i)` が真のときに飛ばすオペランド。A3 が
      「`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの参照を処分しない」と置く。
  BY A3, L5

<1>7. `rhs_consumes` の `RcRhs::Llvm` の腕が `passthrough` に入るとして飛ばす leaf。A3 が「`result_prov` が
      結果の leaf を `Arg(j, σ)` と宣言するとき、生成コードはその leaf に第 `j` オペランドの leaf `σ` と同じ
      参照を置き、新しい参照を作らない」と置く。D9 の移動の表の第 5 行がこれを移動とし、D10 の移動の行より
      `Obl` は変わらない。
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

## `README.md` へ差し戻す点

### 「スロットが参照を持つ」の定義が無い (P3, P4, P5 に影響)

P3、P4、P5 はいずれも「スロットが参照を持つ」を主語に取るが、D6 (スロット) も D8 (参照) もこの関係を定めて
いない。読み A では P5 は空虚に真、読み B では P5 は偽である。定義を足す場所は D6 か D8 か D9 であり、この
文書は勝手に足していない。

### P5 の言明は E5 の辺を除く必要がある

読み B の下で証明できるのは `P5a` である。`README.md` の P5 との差は `Match` のアームの `Ret` (E5) の辺
だけである。P5 を次のどちらかに直すことを提案する。

- **案 1**: 「1 つの実行路の 1 つの位置において同じ参照を持ち、その参照の持ち手の変化が `Match` のアームの
  `Ret` を含まない 2 つのスロットは、同じ `unit_key` を持つ」。これが `P5a` であり、証明済みである。
- **案 2**: 案 1 に、E5 の辺についての次の主張を足す。「`Match` のアームの `Ret` の辺 `(x, λ) -> (m, λ)` に
  ついて、`origin(x, λ).candidates()` は `origin(m, λ).candidates()` に含まれる」。これが `L3 の <1>6` で
  あり、証明済みである。`cancel` が E5 の辺を渡って安全であるのは `Origin::acted_on` が candidates を返し、
  `walk_inner` の `Release` の腕がそれらのキーの pending retain を `consume_unit` で needed にするからで
  あり (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の `RcExpr::Release` の腕)、その議論には
  案 2 の第 2 の主張が要る。

案 2 の主張は candidates についてのものであって identity についてのものではない。`Join` の入れ子では内側の
`Join` の identity は外側の candidates に入らない (`CODE src/rc_ir/ownership.rs:
tests::a_join_of_joins_may_be_any_of_their_results` が、`inner` が外側の候補に入らないことを表明している)。
この差が `cancel` の健全性に効くかどうかは P19-P21 の担当である。

### D10 の生成の表の `Llvm` の行が `origin_inner` の producer の腕と食い違う

D10 の生成の表は `Llvm` の行を「`result_prov` が `Fresh` または `Unknown` と宣言する leaf」とする。一方
`origin_inner` が `Llvm` の腕でその値自身を origin とする (producer になる) のは、
`as_arg_projection(decl.leaf_origins_at(path))` が `None` のときである。`as_arg_projection` が `None` を
返すのは次の 3 つであり (`CODE src/rc_ir/ownership.rs: as_arg_projection`)、D10 の行が覆うのは 2 つ目だけで
ある。

- `LeafOrigins` が空集合 (不在の変位の leaf)。参照を持たないので生成が無いのが正しい。D10 がそう述べて
  いないだけである。
- `LeafOrigins` が 1 元で、その元が `Fresh` または `Unknown`。D10 の行のとおり。
- `LeafOrigins` が 2 元以上 (`{Arg(0, σ), Fresh}` や `{Arg(0, σ), Arg(1, τ)}` など)。D9 の `Llvm` の行は
  この leaf を素通しとしないので、オペランドの参照は消費されて `Obl` から抜ける。しかし D10 の生成の表には
  この場合の行が無いので、結果の leaf の参照がどこから来るのかを D10 が述べていない。

D10 の `Llvm` の行を「`result_prov` が単一の `Arg(j, σ)` を宣言しない leaf のうち、源が空集合でないもの」に
直すと、D9 の `Llvm` の消費・移動の行と、`as_arg_projection` の分岐と、3 つが 1 つの述語で揃う。

`origin_inner` のその他の producer の腕は D10 と対応している。`Binding::Param` は D10 の初期値、
`Binding::Producer` (`RcRhs::App` と `RcRhs::Closure` が付ける。`CODE src/rc_ir/ownership.rs:
collect_bindings`) は D10 の `App` と `Closure` の行、`Binding::Field` で容器が boxed の場合は D10 の
「boxed 容器の `Destructure` の各名前付きフィールド」の行、`Binding::Payload` で scrutinee が boxed の場合は
D10 の「boxed union の変位アームの payload」の行に当たる。`vars.bindings` に無い名前 (`None` の腕) は
グローバルか直接呼び出しの関数名であり、前者は A8 が線形規律の外に置き、後者は funptr 型なので boxed leaf を
持たない。

### D2 と A4 が unbox union について実装と食い違う

P6 の補足 3 に書いたとおり、`Retain(v, π)` のコード生成は unbox union のタグを読んで活性な変位だけを retain
する。D2 の `Retain`/`Release` の行と A4 は「`π` の下の各 boxed leaf の ±1」と述べており、参照を持つ変位を
2 つ以上持つ unbox union ではそうなっていない。P6 は D2 と D10 の行の上で証明したので P6 自体は閉じているが、
A4 が果たされないことが分かっている箇所として記録する。ずれは `acted_references` が多く数える向きにしか出ず、
retain 側と release 側で同じように出るので、`un_bump` の `covers` が真になりにくくなる方向である。

### `Retain` が作る参照の持ち手が D10 に書かれていない

D10 の `Retain` の行は「`obj(v, λ)` への参照を 1 つ加える」とだけ述べ、その参照をどのスロットが持つかを
述べていない。`DEF 持ち手集合` では `(v, λ)` が持つものとした。スロットが参照を持つ関係を足すときに、この点も
決める必要がある。

### P5 と P6 は 1 つの活性化についての主張である

`unit_key` も `acted_references` も `VarTable` を引数に取り、`VarTable` は 1 つの関数の本体から作られる
(`CODE src/rc_ir/ownership.rs: VarTable::of`)。よって 2 つのスロットのキーを比べる主張は、1 つの活性化の中の
スロットについてのものでなければならない。D10 と D11 が「関数の 1 回の活性化について」と書いているので文脈から
は読めるが、P5 の言明にはその限定が無い。`P5a` の言明には入れてある。
