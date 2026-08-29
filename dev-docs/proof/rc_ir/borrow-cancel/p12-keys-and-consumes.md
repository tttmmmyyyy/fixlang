# P5, P6, P7 -- キーと `acted_references` と消費の網羅性

`README.md` の定義 D1-D16、仮定 A1-A8、および命題 P1-P4 の言明の上に立つ。P1-P4 の証明は
`p10-leaves-and-units.md` と `p11-origin-soundness.md` にあり、この文書はその言明だけを使う。

**結論を先に書く。P6 と P7 は証明できた。P5 は偽である。** 反例を `R1` として与える。P5 のうち証明できる部分を
`P5a` として切り出し、`cancel` が P5 の代わりに使っている性質を `L3` として証明した。差し戻す言明は最後の節に
まとめる。

## 記法

1 つの関数 (またはグローバル初期化子) の本体を固定し、その本体から作られる `VarTable` を `vars`、プログラムの
`TypeEnv` を `type_env` と書く (`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`)。この
2 つは本体ごとに 1 つなので、以下では `origin`、`unit_key`、`acted_references`、`acted_unit_keys` の
第 1・第 2 引数を落として書く。

- `origin(x, π)` は `origin(vars, type_env, x, π)`。
- `id(x, π)` は `origin(x, π).identity()` (`CODE src/rc_ir/ownership.rs: Origin::identity`)。
- `cand(x, π)` は `origin(x, π).candidates()` を集合とみなしたもの
  (`CODE src/rc_ir/ownership.rs: Origin::candidates`)。
- `key(x, π)` は `unit_key(vars, type_env, x, π)` (D15)。
- `L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合。D4 より、これが
  「`v` の `π` の下の boxed leaf」の全体である。inhabited (D16) でないものを含む。
- 実行路の位置 `p` において、`L(v, π)` の元のうち inhabited なものの集合を `Linh(v, π, p)` と書く。

`VarPath` は対 `(FullName, FieldPath)` である (`CODE src/rc_ir/ast.rs: VarPath`)。等号はこの対の等号である。

この文書は補題を `L1` から `L5`、証明した命題を `P5a`、反例を `R1` と呼ぶ。番号を持たない補題は節の見出しの
文で呼ぶ (`別名の道は同じオブジェクトを指す`)。`BY` の行ではそれらを名前で引用し、その中のステップを指すときは
`L1 の <1>3` と書く。

## 移動の辺

D9 の移動の表の 6 行を、スロットの間の有向辺として次の名前で呼ぶ。以下 `λ` は leaf を渡る。

- **E1** (`Let(x, Var(y), k)`): `(y, λ)` から `(x, λ)` へ。
- **E2** (`Destructure(c, fs, s, k)` で `c` が unbox、`(i, f)` が `fs` の元): `(c, [i] ++ λ)` から
  `(f, λ)` へ。
- **E3** (`Match(s, arms)` の unbox union の変位アームの payload 束縛、変位番号 `t`): `(s, [t] ++ λ)` から
  `(p, λ)` へ。
- **E4** (`Match(s, arms)` の catch-all アームの payload 束縛): `(s, λ)` から `(p, λ)` へ。
- **E5** (`Let(x, Llvm(gen, args), k)` の素通し leaf): 結果の leaf `λ` の `result_prov` の宣言が単一の
  `Arg(j, σ)` のとき、`(args[j], σ)` から `(x, λ)` へ。
- **E6** (`Match` のアーム本体の `Ret(x)`): `Match` の束縛変数を `m` として、`(x, λ)` から `(m, λ)` へ。

**DEF 別名の道**
1 つの実行路の 1 つの位置 `p` において、スロット `s` とスロット `t` が**別名の道で結ばれている**とは、`p` までに
実行された E1-E6 の辺だけを、向きを問わずに辿って `s` から `t` へ行けることをいう。

## L1 (`unit_key` は identity の関数である)

**言明**。`id(x, π) = id(y, ρ)` ならば `key(x, π) = key(y, ρ)` である。また `key(x, π)` の根は
`id(x, π)` の根に等しい。

<1>1. `key(x, π) = unit_of(vars, type_env, id(x, π))` である。
  BY D15, CODE src/rc_ir/ownership.rs: unit_key

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
    <3>1. この腕は `truncate_to_unit(ty, path, type_env)` を計算し、`rc_units(ty, type_env)` がそれを含む
          ことを表明したのち `(root.clone(), truncated)` を返す。
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

**補足 (切り詰めは何をするか)**。`unit_of` の 2 つの腕のどちらを通るかは `root` だけで決まり、切り詰めの結果は
`(ty, path, type_env)` だけで決まる (`<1>2`)。よって切り詰めは、**等しい** identity を異なるキーへ分けることが
できない。切り詰めが**異なる** identity を 1 つのキーへまとめることはある。unbox union の変位の下の leaf は
union 自身のキーへ落ち、punched array の内側の配列は punched array 自身のキーへ落ちる (D5)。P5 は「同じ参照
ならば同じキー」の向きしか主張しないので、まとめる向きは P5 に反しない。

## L2 (E1-E5 の辺は `origin` を保つ)

**言明**。E1、E2、E3、E4、E5 のいずれかの辺で結ばれた 2 つのスロットは、同じ `origin` を持つ。

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
    BY D9
  <2>3. QED
    BY <1>2, <2>1, <2>2

<1>7. E3 の辺 `(s, [t] ++ λ) -> (p, λ)` について `origin(p, λ) = origin(s, [t] ++ λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の
        `Some(tag) if !scrut.ty.is_box(type_env)` の場合は、`scrut_path` を `[*tag] ++ path` として作り
        `origin(vars, type_env, &scrut.name, &scrut_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. E3 の辺は `s` が unbox union のときにだけ引かれるので、`scrut.ty.is_box(type_env)` は偽であり、
        `arm.tag` は `Some(t)` である。
    BY D9
  <2>3. QED
    BY <1>3, <2>1, <2>2

<1>8. E4 の辺 `(s, λ) -> (p, λ)` について `origin(p, λ) = origin(s, λ)` である。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の `None` の場合は
        `origin(vars, type_env, &scrut.name, path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. E4 の辺は catch-all アームについてだけ引かれるので、`arm.tag` は `None` である。
    BY D9
  <2>3. QED
    BY <1>3, <2>1, <2>2

<1>9. E5 の辺 `(args[j], σ) -> (x, λ)` について `origin(x, λ) = origin(args[j], σ)` である。
  <2>1. `origin_inner` の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕は、`decl` を
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` として作り、
        `decl.leaf_origins_at(path).and_then(as_arg_projection)` が `Some((j, p))` のとき
        `origin(vars, type_env, &args[j].name, &p)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕
  <2>2. `as_arg_projection(sources)` が `Some((j, p))` を返すのは、`sources` がちょうど 1 元からなり、その元が
        `LeafOrigin::Arg(j, p)` のときに限る。
    BY CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>3. E5 の辺は結果の leaf `λ` の宣言が単一の `Arg(j, σ)` のときにだけ引かれるので、
        `decl.leaf_origins_at(λ)` は 1 元集合 `{Arg(j, σ)}` であり、`as_arg_projection` はそれを
        `Some((j, σ))` に写す。
    BY D9, <2>2
  <2>4. QED
    BY <1>4, <2>1, <2>3

<1>10. QED
  BY <1>5, <1>6, <1>7, <1>8, <1>9

**補足 (E1-E5 に入らない `origin_inner` の腕)**。`origin_inner` が `here()` を返して値自身を origin とするのは、
`Binding` が `None` / `Param` / `Producer` のとき、`Field` で容器が boxed のとき、`Payload` で scrutinee が
boxed のとき、`Llvm` で `as_arg_projection` も `origin_from_leaves_under` も答えを出さないときである
(`CODE src/rc_ir/ownership.rs: origin_inner`)。このうち `Param` は D10 の初期値、`Producer` は D10 の生成の
表の `App` と `Closure` の行 (`CODE src/rc_ir/ownership.rs: collect_bindings` がこの 2 つに `Producer` を
付ける)、boxed の `Field` と boxed の `Payload` は D10 の生成の表の対応する行、`Llvm` は D10 の生成の表の
`Llvm` の行に当たる。`None` の腕はグローバルか直接呼び出しの関数名であり、前者は A8 が線形規律の外に置き、
後者は funptr 型なので boxed leaf を持たない。

## L3 (E6 の辺は candidates を保つが identity を保たない)

**言明**。E6 の辺 `(x, λ) -> (m, λ)` について、`cand(x, λ)` は `cand(m, λ)` に含まれる。一方
`id(x, λ) = id(m, λ)` は成り立たないことがある。

<1>1. `collect_bindings` は、`Let(x, Match(scrut, arms), k)` に対し `x` の `Binding` を `Join(arm_results)`
      とする。ここで `arm_results` は各アームについての `returned_var(&arm.body)` の列である。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Match` の場合

<1>2. `returned_var(&arm.body)` は、アーム本体の終端の `Ret` が名指す変数である。
  BY CODE src/rc_ir/ownership.rs: returned_var

<1>3. `origin_inner` の `Some(Binding::Join(arm_results))` の腕は、各 `arm_result` について
      `origin(vars, type_env, &arm_result.name, path).candidates()` の元を集めて `candidates` とし、
      `Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))` を返す。
  BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕

<1>4. `Origin::of_candidates(candidates, identity)` は、`candidates` が 1 元のときその元の `Exactly`、
      2 元以上のとき `Join { identity, candidates }` を返す。空の `candidates` は表明で弾く。
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
    この 2 つを返し分ける。
    BY D13, CODE src/rc_ir/ownership.rs: Origin, <1>4, <2>1, <2>2

<1>6. `cand(x, λ)` は `cand(m, λ)` に含まれる。
  <2>1. `x` は `m` を束縛する `Match` のいずれかのアーム本体の `returned_var` である。
    BY D9, <1>1, <1>2
  <2>2. `<1>3` が集める `candidates` は、そのアームの寄与として `cand(x, λ)` の全元を含む。
    BY <1>3, <2>1
  <2>3. QED
    BY <1>3, <1>5, <2>2

<1>7. `id(x, λ)` と `id(m, λ)` が異なる `VarTable` がある。
  <2>1. `p` と `q` の `Binding` が `Producer`、`m` の `Binding` が `Join([p, q])` である `VarTable` に
        ついて、`origin(m, [])` の `identity()` は `(m, [])` である。
    BY CODE src/rc_ir/ownership.rs: tests::a_match_binding_may_be_any_arm_result
  <2>2. 同じ `VarTable` について、`Binding` が `Producer` である変数 `p` の `origin(p, [])` は
        `Exactly((p, []))` であり、その `identity()` は `(p, [])` である。
    BY CODE src/rc_ir/ownership.rs: tests::a_producer_is_exactly_itself,
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Producer)` の腕
  <2>3. QED
    `(p, [])` と `(m, [])` は根が異なるので `VarPath` として異なる。
    BY A6, <2>1, <2>2

<1>8. QED
  BY <1>6, <1>7

**補足 (入れ子の `Join` では identity が候補に残らない)**。`<1>3` が集めるのは各アームの `origin` の
`candidates()` であって `identity()` ではない。よってアーム本体が返す変数の `origin` が `Join` のとき、その
`identity` は外側の `candidates` に入らない。`CODE src/rc_ir/ownership.rs:
tests::a_join_of_joins_flattens_and_keeps_the_inner_name` がこれを表明している (内側の `Join` の束縛変数 `inner` は
外側の候補 `{p, q, r}` に入らない)。`acted_unit_keys` は `Origin::acted_on` の像なので、この identity のキーを
報告しない。

## L4 (boxed leaf は互いに前置にならない)

**言明**。型 `τ` について、`boxed_leaf_paths(τ, type_env)` の相異なる 2 元の一方が他方の前置になることはない。
とくに `π` が leaf であるとき `L(v, π)` は `{π}` である。

<1>1. `boxed_leaf_paths` の内部の走査 `go` は、`ty.is_closure()`、`ty.is_box(type_env)`、`ty.is_array()` の
      各腕で `out` に path を積んだ直後に `return` する。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `go` が path を積むのはその 3 つの腕だけであり、`ty.is_fully_unboxed(type_env)` の腕は積まずに `return`
      し、最後の腕は `unpunched_field_types` の各フィールドへ降りる。
  BY D4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

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

## 別名の道は同じオブジェクトを指す

**言明 (DEF 別名の道 の意味)**。1 つの実行路の 1 つの位置において、E1-E6 のいずれかの辺で結ばれた 2 つの
スロット `s`、`t` が両方ともその位置に存在するなら、その 2 つは同じオブジェクト `o` を指す。さらに、その
位置の `Obl` が持つ `o` への参照がちょうど 1 つであり、`s` と `t` が持つ参照がどちらもその `Obl` の元で
あるなら、`s` と `t` は同じ参照を持つ。

<1>1. 辺 `s -> t` が実行された位置以降で、`s` と `t` はどちらも同じオブジェクト `o` を指す。
  D9 の移動の 6 行はいずれも、参照の持ち手が変わるだけであって、指すオブジェクトを変えない。
  BY D9

<1>2. その位置において、`s` が持つ参照も `t` が持つ参照も `o` への参照である。
  A5 より、スロットはちょうど 1 つの参照を持つ。D8 より、その参照は指すオブジェクトへの処分義務の 1 単位で
  ある。
  BY A5, D8, <1>1

<1>3. QED
  `Obl` が持つ `o` への参照がちょうど 1 つであり、`s` と `t` の持つ参照がどちらもその `Obl` の元であるなら、
  `<1>2` の 2 つの参照はどちらもその 1 つである。
  BY <1>1, <1>2

**この補題が `Obl` の条件を必要とする理由**。`Retain(v, π)` は `o` への参照を `Obl` に 1 つ足す (D10) ので、
`Obl` が `o` への参照を 2 つ以上持つことがある。そのとき、移動の後に `s` と `t` がどちらの参照を持つのかは
D9 も D10 も述べていない。`README.md` へ差し戻す点にこれを書く。

## P5a (E6 を通らない別名の道についての P5)

**言明**。1 つの活性化の 1 つの実行路の 1 つの位置において、E1-E5 の辺だけからなる別名の道で結ばれた 2 つの
スロットは、同じ `unit_key` を持つ。

<1>1. 道の各辺の両端は同じ `origin` を持つ。
  BY L2

<1>2. 道の両端のスロットを `(x, λ)`、`(y, μ)` とすると `origin(x, λ) = origin(y, μ)` であり、したがって
      `id(x, λ) = id(y, μ)` である。
  BY <1>1, CODE src/rc_ir/ownership.rs: Origin::identity

<1>3. QED
  BY L1, <1>2

## R1 (P5 の反例)

**言明**。P5 は偽である。1 つの実行路の 1 つの位置において同じ参照を持ちながら、異なる `unit_key` を持つ
2 つのスロットがある。

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
  D14 と A1 よりパラメータの unit はすべて所有され、`<1>2` より `x` と `y` の inhabited な leaf は `[]` だけ
  で、`c` は leaf を持たない。
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

<1>6. 変位 0 のアームを選ぶ実行路の `Ret(m)` の位置において、`(x, [])` と `(m, [])` はどちらもスロットで
      あり、同じ参照 `r_x` を持つ。
  <2>1. `x` はパラメータなのでこの位置で束縛されている。`m` は `Let(m, Match(c, ...), Ret(m))` が束縛して
        おり、この位置はその継続の中である。`[]` は `T` の inhabited な boxed leaf なので、どちらもスロット
        である。
    BY D2, D6, <1>2
  <2>2. アーム本体の `Ret(x)` は E6 の辺 `(x, []) -> (m, [])` を実行するので、この 2 つのスロットは同じ
        オブジェクト `obj(x, [])` を指す。
    BY D9, 別名の道は同じオブジェクトを指す, <2>1
  <2>3. この位置の `Obl` は `{r_x}` であり、`r_x` は `obj(x, [])` への参照である。よって `Obl` が持つ
        `obj(x, [])` への参照はちょうど 1 つである。
    BY <1>3, <1>4
  <2>4. `(x, [])` と `(m, [])` が持つ参照はどちらも `Obl` の元である。`<1>3` より `r_x` は所有される
        パラメータの leaf の参照として `Obl` に入り、`<2>2` の辺は移動なので `Obl` を変えない (D9, D10)。
        この活性化は `obj(x, [])` への参照をほかに持たない。
    BY A1, D9, D10, D14, <1>3, <2>2
  <2>5. QED
    BY 別名の道は同じオブジェクトを指す, <2>1, <2>2, <2>3, <2>4

<1>7. `f` の本体は D11 の意味で健全である。
  <2>1. CASE 変位 0 のアームを選ぶ実行路。
    <3>1. (S-a) が成り立つ。参照を取り除く操作は `Release(y, [], s, ...)` と終端の `Ret(m)` の消費の 2 つで
          ある。前者が取り除く `r_y` は `<1>3` の `Obl` にあり、後者が取り除くのはスロット `(m, [])` が持つ
          参照、すなわち `r_x` (`<1>6`) で、これは `<1>4` の `Obl` にある。
      BY D9, D10, D11, <1>3, <1>4, <1>6
    <3>2. (S-b) が成り立つ。終端の `Ret(m)` の消費が `<1>4` の `Obl` から `r_x` を取り除くと、`Obl` は空に
          なる。
      BY D9, D10, D11, <1>4, <1>6
    <3>3. (S-c) が成り立つ。この実行路にある読む構文は `Let(m, Match(c, ...), Ret(m))` の `Match` だけで、
          読まれるのは `c` である。`c` は leaf を持たないので読まれるオブジェクトは無い。
      BY D7, D11, <1>2
    <3>4. QED
      BY <3>1, <3>2, <3>3
  <2>2. CASE 変位 1 のアームを選ぶ実行路。
    <3>1. アーム本体の `Ret(y)` は E6 の辺 `(y, []) -> (m, [])` を実行するので、スロット `(m, [])` が持つ
          参照は `r_y` である。理由は `<1>6` と同じ構成であり、`<1>5` が `Obl` を `{r_y}` と与える。
      BY A1, D2, D6, D9, D10, D14, 別名の道は同じオブジェクトを指す, <1>2, <1>3, <1>5
    <3>2. (S-a) が成り立つ。参照を取り除く操作は `Release(x, [], s, ...)` と終端の `Ret(m)` の消費の 2 つで
          ある。前者が取り除く `r_x` は `<1>3` の `Obl` にあり、後者が取り除く `r_y` は `<1>5` の `Obl` に
          ある。
      BY D9, D10, D11, <1>3, <1>5, <3>1
    <3>3. (S-b) が成り立つ。終端の `Ret(m)` の消費が `<1>5` の `Obl` から `r_y` を取り除くと、`Obl` は空に
          なる。
      BY D9, D10, D11, <1>5, <3>1
    <3>4. (S-c) が成り立つ。読む構文は `Match` だけで、`c` は leaf を持たない。
      BY D7, D11, <1>2
    <3>5. QED
      BY <3>2, <3>3, <3>4
  <2>3. QED
    D3 より実行路はアームの選び方で尽くされ、アームは 2 つである。
    BY D3, <2>1, <2>2

<1>8. `key(x, []) = (x, [])` である。
  <2>1. `x` はパラメータなので `x` の `Binding` は `Param` であり、`origin_inner` の
        `Some(Binding::Param)` の腕は `here()` すなわち `Exactly((x, []))` を返す。
    BY CODE src/rc_ir/ownership.rs: VarTable::of, origin_inner
  <2>2. `id(x, []) = (x, [])` である。
    BY <2>1, CODE src/rc_ir/ownership.rs: Origin::identity
  <2>3. `vars.var_tys` は `x` に `T` を記録しているので、`unit_of` は `truncate_to_unit(T, [], type_env)` を
        返す。これは空の `path` について繰り返しを回さないので `[]` である。
    BY CODE src/rc_ir/ownership.rs: VarTable::of, unit_of, truncate_to_unit
  <2>4. QED
    BY D15, <2>2, <2>3

<1>9. `key(m, []) = (m, [])` である。
  <2>1. `m` の `Binding` は `Join([x, y])` である。`x` と `y` はそれぞれのアーム本体の `returned_var` で
        ある。
    BY L3 の <1>1, L3 の <1>2, <1>1
  <2>2. `cand(x, [])` は `{(x, [])}`、`cand(y, [])` は `{(y, [])}` である。どちらもパラメータなので
        `origin` は `Exactly` を返し、その `candidates()` は 1 元である。
    BY <1>8, CODE src/rc_ir/ownership.rs: origin_inner, Origin::candidates
  <2>3. `origin_inner` の `Join` の腕が集める `candidates` は `{(x, []), (y, [])}` の 2 元なので、返り値は
        `Join { identity: (m, []), .. }` であり `id(m, []) = (m, [])` である。
    BY A6, L3 の <1>3, L3 の <1>4, <2>1, <2>2
  <2>4. `vars.var_tys` は `m` に `T` を記録しているので、`unit_of` は `truncate_to_unit(T, [], type_env)` を
        返し、これは `[]` である。
    BY CODE src/rc_ir/ownership.rs: collect_bindings, unit_of, truncate_to_unit
  <2>5. QED
    BY D15, <2>3, <2>4

<1>10. QED
  `<1>7` より `f` の本体は健全であり、`<1>6` より変位 0 の実行路の `Ret(m)` の位置において `(x, [])` と
  `(m, [])` は同じ参照 `r_x` を持つ。`<1>8` と `<1>9` より `key(x, []) = (x, [])`、`key(m, []) = (m, [])` で
  あって、A6 よりこの 2 つは異なる。よって P5 は偽である。
  BY A6, <1>6, <1>7, <1>8, <1>9

**この反例が `cancel` の健全性をただちに壊すわけではない**。`cancel` が E6 の辺を渡って安全であるのは、
`Release` と消費のところで `acted_unit_keys` が `Origin::acted_on` の像 (identity と candidates の両方) を
返し、`walk_inner` の `Release` の腕と `consume` がそれらのキーの pending retain を `consume_unit` で
`needed_retains` に入れるからである (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の
`RcExpr::Release` の腕, `CancelAnalysis::consume`)。その議論に要るのは `L3 の <1>6` であって P5 ではない。

## unbox union の二重命名 (P5 が主張していないこと)

P5 が言うのは「同じ**参照**を持つスロットは同じキー」であって「同じ**オブジェクト**に触れる操作は同じキー」では
ない。この違いを、`acted_references` と `unit_key` が読む path の違いとして書き下す。

<1>1. `unit_key(v, π)` は `origin(v, π)` を、`acted_references(v, π)` は `L(v, π)` の各 leaf `λ` について
      `origin(v, λ)` を読む。`π` が leaf でないとき、この 2 つは別の path の `origin` である。
  BY D15, CODE src/rc_ir/ownership.rs: unit_key, acted_references

<1>2. `π` が leaf でない RC unit であるのは、`π` が unbox union を名指すときと punched array を名指すときで
      ある。
  BY D5

<1>3. unbox union `s` の変位 `t` のアームの payload `p` について、その変位が活性である位置において、スロット
      `(p, λ)` とスロット `(s, [t] ++ λ)` は同じキーを持つ。
  <2>1. この 2 つは E3 の辺で結ばれる。
    BY D9
  <2>2. QED
    BY L1, L2, <2>1

<1>4. 一方で、`Retain(un, π)` の節点のキーと、`un` の payload の元になった値を名指す `Release` の節点のキーは、
      その 2 つが 1 つのオブジェクトに触れていても異なりうる。
  <2>1. `InlineLLVMMakeUnionBody` の `result_prov` は、unbox union の結果について、構築される変位の leaf に
        `Arg(0, rest)` を、他の変位の leaf に空集合を宣言する。
    BY CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov
  <2>2. `un` を `Let(un, Llvm(MakeUnion_0, [tw]), k)` で束縛し、`un` の型が boxed leaf を 2 つ持つ変位を
        第 0 変位に持つ unbox union であるとする。このとき `decl.leaf_origins_at([])` は `None` である。
    <3>1. `Provenance::leaf_origins_at(path)` は、`path` がその値の boxed leaf でないとき `None` を返す。
      BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at
    <3>2. `[]` は unbox union の boxed leaf ではない。
      BY D5, <1>2
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
    BY L3 の <1>4, <2>4, CODE src/rc_ir/ownership.rs: Origin::identity
  <2>6. `key(un, [])` の根は `un` であり、`tw` の leaf のスロットのキーの根は `tw` である。A6 より `un` と
        `tw` は相異なる変数なので、この 2 つのキーは異なる。
    BY A6, L1, <2>5
  <2>7. QED
    `Retain(un, [])` と、`tw` の leaf を名指す `Release` は、`un` の payload が持つ参照という同じオブジェクトの
    参照に触れながら、異なるキーを持つ。
    BY <2>5, <2>6

<1>5. QED
  `<1>3` と `<1>4` が主張である。**スロット**のキーは E3 の辺を越えて一致するが、**節点**のキーは一致しなくて
  よい。節点のキーは unit path の `origin` から作られ、その unit path は leaf でないことがある (`<1>2`)。
  BY <1>3, <1>4

## `check_one_key_per_object` の境界との照合

`check_one_key_per_object` の doc は次の 3 つを述べる
(`CODE src/rc_ir/borrow.rs: check_one_key_per_object`)。

- (i) 同じ参照に作用する 2 つの操作は同じオブジェクトに作用し、1 つのオブジェクトは 1 つの unit key で
  数えられる。
- (ii) unbox union の retain はその payload が持つ参照を全部上げ、その payload の値の release はそのうち 1 つを
  下げる。これは 2 つのキー・1 つのオブジェクトであって、どちらの命名も正しい。
- (iii) そのような release は真部分集合を un-bump するので `un_bump` は `OutsideBracket` を返す。よって
  `References` が**等しい**場合だけが不一致である。

<1>1. (ii) と (iii) が述べる「2 つのキー」は、`unbox union の二重命名` の `<1>4` が示した**節点**のキーの
      違いである。P5 が述べるスロットのキーは、同じ節の `<1>3` のとおり E3 の辺を越えて一致する。よって
      (ii)(iii) は P5 と矛盾しない。
  BY L1, L2

<1>2. (i) の後半「1 つのオブジェクトは 1 つの unit key で数えられる」は、`unbox union の二重命名` の `<1>4` に
      より偽である。P5 が主張するのはスロットについてであって、オブジェクトについてではない。
  BY A6, L1

<1>3. 表明が発火しないことは、両方の節点の path が boxed leaf のときには L1 と L4 から従う。
  <2>1. `Retain(v, π)` と `Release(w, ρ)` の path がどちらも boxed leaf のとき、`acted_references` は
        それぞれ 1 元の `Map` `{id(v, π): 1}`、`{id(w, ρ): 1}` を返す。
    BY D15, L4, CODE src/rc_ir/ownership.rs: acted_references
  <2>2. その 2 つが等しいなら `id(v, π) = id(w, ρ)` であり、`key(v, π) = key(w, ρ)` である。
    BY L1, <2>1
  <2>3. QED
    `check_one_key_per_object` はキーが異なる pending retain についてだけ表明するので、`<2>2` の下では
    発火しない。
    BY CODE src/rc_ir/borrow.rs: check_one_key_per_object, <2>2

<1>4. `π` または `ρ` が leaf でない unit のときは、`<1>3` の議論は使えない。
  <2>1. 節点のキーは path `π` の `origin` から、`References` は `π` の下の leaf の `origin` から作られる。
    BY D15
  <2>2. この 2 つを結ぶ主張は P1 から P6 のどれでもない。
    BY P1, P2, P3, P4, P5, P6
  <2>3. QED
    BY <2>1, <2>2

<1>5. QED
  doc の (ii)(iii) は P5 と矛盾しない (`<1>1`)。doc の (i) の後半の言い方は、そのままでは
  `unbox union の二重命名` の `<1>4` に反する (`<1>2`)。表明が発火しないことは、両方の path が boxed leaf の
  ときには従う (`<1>3`) が、leaf でない unit のときは P1 から P6 では埋まらない (`<1>4`)。
  BY <1>1, <1>2, <1>3, <1>4

## P6 (`acted_references` は静的な上位近似である)

P6 の 2 つの主張を次のように書く。

- **(a)** `acted_references(v, π)` が返す `Map` は、`L(v, π)` のすべての元 `λ` を `id(v, λ)` で名付けて
  数えた多重集合である。
- **(b)** 位置 `p` において `Retain(v, π)` が実行時に作る参照の多重集合は、(a) の数え上げを `Linh(v, π, p)` に
  制限したものに等しい。`Release(v, π)` が実行時に処分する参照の多重集合も同じものに等しい。

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

<1>2. `Release(v, π)` が `Obl` から取り除く参照は、`Linh(v, π, p)` の各 `λ` についてちょうど 1 つずつであり、
      それが全部である。
  <2>1. D10 の `Release` の行は「`π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ
        取り除く」である。
    BY D10
  <2>2. 「`π` の下の inhabited な leaf」の全体は `Linh(v, π, p)` である。
    BY D4, D16, 記法 の `Linh` の定義
  <2>3. QED
    BY <2>1, <2>2

<1>3. `λ` に `r_λ` を対応させる写像は、`Linh(v, π, p)` から `Retain(v, π)` が作る参照の多重集合への全単射で
      ある。
  <2>1. A5 より、`Linh(v, π, p)` の各 `λ` についてスロット `(v, λ)` はちょうど 1 つの参照を持つ。
    BY A5, D6, D16
  <2>2. QED
    BY DEF 名前づけ, <1>1, <2>1

<1>4. `λ` に `r'_λ` を対応させる写像は、`Linh(v, π, p)` から `Release(v, π)` が処分する参照の多重集合への
      全単射である。
  <2>1. A5 より、`Linh(v, π, p)` の各 `λ` についてスロット `(v, λ)` はちょうど 1 つの参照を持つ。
    BY A5, D6, D16
  <2>2. QED
    BY DEF 名前づけ, <1>2, <2>1

<1>5. `id(v, λ)` は参照 `r_λ` および `r'_λ` の名前である。すなわち `origin(v, λ)` が `Exactly(u, σ)` なら
      それらはスロット `(u, σ)` の下の対応するスロットが持つ参照と同一であり、
      `Join { identity, candidates }` なら `candidates` のいずれかの下の対応するスロットが持つ参照と同一で
      あって、`identity` はその参照に対して実行路をまたいで同じ `VarPath` である。
  BY P3, P4

<1>6. `Retain(v, π)` が作る参照の多重集合の、DEF 名前づけ の下での像は、多重集合として
      `{ id(v, λ) : λ が Linh(v, π, p) の元 }` である。
  BY DEF 名前づけ, <1>3, <1>5

<1>7. `Release(v, π)` が処分する参照の多重集合の、DEF 名前づけ の下での像は、多重集合として
      `{ id(v, λ) : λ が Linh(v, π, p) の元 }` である。
  BY DEF 名前づけ, <1>4, <1>5

<1>8. P6 (a) の数え上げを `Linh(v, π, p)` に制限したものは、多重集合として
      `{ id(v, λ) : λ が Linh(v, π, p) の元 }` である。
  BY P6 (a) の <1>3

<1>9. QED
  BY <1>6, <1>7, <1>8

### P6 の結論

<1>1. (a) は `P6 (a) の <1>3` である。
  BY P6 (a)
<1>2. (b) は `P6 (b) の <1>9` である。
  BY P6 (b)
<1>3. QED
  BY <1>1, <1>2

**補足 1 (2 つの leaf が 1 つの名前を持つとき)**。`L(v, π)` の相異なる 2 つの leaf が同じ `id` を持つことが
ある。`Map` はそのとき計数を 2 にする (`P6 (a) の <1>1`)。これは 1 つのオブジェクトへの参照を 2 つ持つ値
(`MakeStruct(a, a)` の結果など) に対応し、A5 の下で参照は inhabited な leaf ごとに 1 つなので、参照としても
2 つある。

**補足 2 (identity は切り詰められていない)**。`acted_references` のキーは `origin(...).identity()` であって
`unit_of` を通していない (`P6 (a) の <1>1`)。節点のキー `unit_key` は通している (D15)。この 2 つを混ぜては
ならない。

**補足 3 (上位近似のずれは片側だけである)**。`acted_references` は `L(v, π)` を数え、実行時に触れるのは
`Linh(v, π, p)` である。`Linh(v, π, p)` は `L(v, π)` の部分集合なので、`References` はつねに実行時以上に
数える。`References` を読むのは `un_bump` の `covers` と `subtract`、および `check_one_key_per_object` の
等号である (`CODE src/rc_ir/borrow.rs: un_bump`, `check_one_key_per_object`)。retain の
`outstanding` も release の `un_bumped` も同じ向きに多く数えるので、`covers` は真になりにくくなる。

## P7 (消費の網羅性)

P7 の 2 つの主張を次のように書く。

- **(a)** D9 の消費の表の各行が指す leaf は、`collect_consumes` が `out` に積む。
- **(b)** `collect_consumes` が `out` に積むもののうち D9 の消費の表に無いものは、`Match` のアーム本体の
  終端の `Ret` が積むものに限る。

`collect_consumes` は `own` 引数を取り、`owns(p, λ)` を `own.contains(&(p.name, λ))` として使う
(`CODE src/rc_ir/ownership.rs: collect_consumes`)。D9 の `App` の行が言う所有は D14 の unit 粒度の所有なので、
P7 は次の `own` について述べる。

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
  <2>3. `collect_consumes` が渡す本体は関数本体である。
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

## `README.md` へ差し戻す点

### P5 の言明は E6 の辺を除く必要がある

`R1` が P5 の反例である。P5 を次のどちらかに直すことを提案する。

- **案 1**: 「1 つの実行路の 1 つの位置において同じ参照を持ち、その参照が `Match` のアーム本体の `Ret` を
  経ずに両方のスロットへ届いている 2 つのスロットは、同じ `unit_key` を持つ」。これが `P5a` であり、証明
  済みである。
- **案 2**: 案 1 に、E6 の辺についての次の主張を足す。「`Match` のアーム本体の `Ret` の辺
  `(x, λ) -> (m, λ)` について、`origin(x, λ).candidates()` は `origin(m, λ).candidates()` に含まれる」。
  これが `L3 の <1>6` であり、証明済みである。`cancel` が E6 の辺を渡って安全であるのは、`Release` と消費の
  ところで `acted_unit_keys` が identity と candidates の両方のキーを返し、`walk_inner` の `Release` の腕と
  `consume` がそれらのキーの pending retain を `needed_retains` に入れるからであり、その議論には案 2 の
  第 2 の主張が要る。P19 から P21 が使うのは案 2 の形である。

案 2 の主張は candidates についてのものであって identity についてのものではない。`Join` の入れ子では内側の
`Join` の identity は外側の candidates に入らない (`L3` の補足)。この差が `cancel` の健全性に効くかどうかは
P19-P21 の担当である。

### P5 と P6 は 1 つの活性化についての主張である

`unit_key` も `acted_references` も `VarTable` を引数に取り、`VarTable` は 1 つの関数の本体から作られる
(`CODE src/rc_ir/ownership.rs: VarTable::of`)。よって 2 つのスロットのキーを比べる主張は、1 つの活性化の中の
スロットについてのものでなければならない。D10 と D11 が「関数の 1 回の活性化について」と書いているので文脈から
は読めるが、P5 の言明にはその限定が無い。`P5a` の言明には入れてある。

### `Retain` が作る参照の持ち手が D10 に書かれていない

D10 の `Retain` の行は「`obj(v, λ)` への参照を 1 つ加える」とだけ述べ、その参照をどのスロットが持つかを述べて
いない。A5 は「値が保持する参照は inhabited な leaf にちょうど 1 つずつある」と述べるので、`Retain` の後の
スロット `(v, λ)` が 2 つの参照を持つのか 1 つなのかが決まらない。

この不足は `別名の道は同じオブジェクトを指す` にも出る。移動の 2 つの端が同じ**オブジェクト**を指すことは
D9 から従うが、同じ**参照**を持つことは、`Obl` がそのオブジェクトへの参照を 1 つしか持たないときにしか
言えない。`R1` はその条件を満たす本体を選んである。`P6 (b)` はこの点を使っていない (`Retain` が加える参照の
個数は D10 の行が直接与える) が、P16 と P19 は `outstanding` を「その `Retain` が作った参照のうちまだ
処分されていないもの」として扱うので、そこでは要る。

### D9 の `App` の行と `collect_consumes` の粒度が違う

D9 の `App` の行は「呼び出し先がその位置の **unit** を所有する引数の leaf」と unit 粒度で述べ、
`collect_consumes` の `owns` は leaf 粒度の集合への所属である
(`CODE src/rc_ir/ownership.rs: collect_consumes`, `CODE src/rc_ir/borrow.rs: OwnedLeaves`)。P7 は
DEF leaf 粒度の所有 でこの 2 つを橋渡ししたが、`infer_ownership` が渡す `owned_leaves` が不動点でその形に
なることは P8 が示す必要がある。

### `check_one_key_per_object` の doc の 1 文

doc の「one object is counted under one unit key」は、`unbox union の二重命名` の `<1>4` により偽である
(1 つのオブジェクトが 2 つの節点キーの下で数えられ、どちらの命名も正しい)。doc の残りの 2 文はこの反例を
正しく述べているので、直すのはこの 1 文だけである。表明が発火しないこと自体は、節点の path が boxed leaf の
ときには L1 と L4 から従うが、leaf でない unit のときは P1 から P6 では埋まらない。埋めるには「unit path の
`origin` と、その下の leaf の `origin` の関係」を述べる命題が要る。
