# P1 (leaf と unit の対応) と P2 (`origin` の全域性と停止性) の証明

この文書が立つのは README の定義 D2、D4、D5 と仮定 A3、A6、A9、A10、A11、A12 の上である。証明は
1 本の構造化証明で、その QED が次の 4 つである。

- **P1** (leaf と unit の対応)。
- **P2** (`origin` の全域性と停止性)。しかも README が置く定義域は外せる。
- **P2 の定義域は `origin` の再帰について閉じていない** (`<1>32`)。これは他の証明者からの報告
  「`origin` は leaf でも unit でもない中間の path でも呼ばれる」の確認である。
- **P1 の系** (`<1>33`, `<1>34`)。`origin` の再帰が辿る path も、`origin` が返す `VarPath` の path も、
  どれも「その型の unit に届く」。すなわちそれらに `truncate_to_unit` を当てると abort せず、値は
  `rc_units` の要素である。これは「unit path の `origin` と、その下の leaf の `origin` の関係」に
  ついての要望への答えである。

P1 と P2 は共通の補題 (型の上の walk が停止すること、`unit_step` と `boxed_leaf_paths` の内部関数
`go` の分類) を使うので、その補題を先頭の `<1>` ステップに置き、P1 と P2 をその後ろに置く。

P1 は 2 つの静的な列挙 (`boxed_leaf_paths` と `rc_units`) の対応についての主張なので、D16 の
inhabited は現れない。実行時にどの leaf が参照を持つかは P1 の主張に入らない。

`<1>1`、`<1>2`、`<1>3a` は、README の A10、A11、A12 をこの文書の記法で述べたものである。README の
文面との差は「入力についての 3 つの前提と README の仮定」の節に書く。

## 1. 記法

型環境 `E` を 1 つ固定する。以下、型に関する関数の `type_env` 引数は `E` に固定し、書かない。

- `L(t)` := `boxed_leaf_paths(t, E)` (`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)
- `U(t)` := `rc_units(t, E)` (`CODE src/rc_ir/ownership.rs: rc_units`)
- `T(t, p)` := `truncate_to_unit(t, p, E)` (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)
- `F(t)` := `t.unpunched_field_types(E)` (`CODE src/ast/types.rs: TypeNode::unpunched_field_types`)。
  これは対 `(添字, 型)` の列である。
- `c` := `CLOSURE_CAPTURE_IDX as usize` (`CODE src/constants.rs: CLOSURE_CAPTURE_IDX`)

型は `t`、`s`、`f` などのラテン文字で表す。path は `usize` の有限列 (`CODE src/rc_ir/ast.rs:
FieldPath`) であり、`p`、`q`、`u`、`lam` などで表す。`p[i]` は第 `i` 要素 (0 始まり)、`p[0..k]` は
先頭 `k` 要素からなる前置、`|p|` は長さ、`p ++ q` は連結、`[]` は空列とする。

**DEF cls** -- 型 `t` の**クラス** `cls(t)` を、次の順に最初に当たるもので定める。

| クラス | 条件 |
|---|---|
| `NB` | `t.is_fully_unboxed(E)` |
| `CL` | 上に当たらず `t.is_closure()` |
| `BX` | 上に当たらず `t.is_box(E)` |
| `AR` | 上に当たらず `t.is_array()` |
| `UN` | 上に当たらず (`t.is_union(E)` または `t.is_punched_array()`) |
| `ST` | 上のどれにも当たらない |

`UN` は D5 が leaf と unit のずれる 2 か所として挙げるもの、すなわち unbox union と punched array を
1 つのクラスにまとめたものである。

**DEF fld** -- `(i, f)` が `F(t)` の要素であるとき `fld(t, i) := f` と書く。この写像が一価であることは
`<1>12` で示す。

**DEF REC** -- 型の間の関係 `t REC f` を、「`t.is_box(E)`、`t.is_closure()`、`t.is_array()`、
`t.is_funptr()` がすべて偽であり、かつある `i` について `(i, f)` が `F(t)` の要素である」で定める。
これは `is_fully_unboxed` が自分を再帰呼び出しする辺そのものである。

**DEF DESC** -- 型の間の関係 `t DESC f` を、「`cls(t)` が `UN` か `ST` であり、かつある `i` について
`(i, f)` が `F(t)` の要素である」で定める。

**DEF ST-道** -- path `p` が型 `t` の **ST-道**であるとは、`s_0 := t`、`s_{j+1} := fld(s_j, p[j])` と
定めたとき、`s_0` から `s_{|p|}` までがすべて定義されており、かつ `j` が `|p|` 未満のすべてで
`cls(s_j) = ST` であることをいう。このとき `s_{|p|}` を `end(t, p)` と書く。

**DEF UNST-道** -- path `p` が型 `t` の **UNST-道**であるとは、上と同じ `s_j` について、`s_0` から
`s_{|p|}` までがすべて定義されており、かつ `j` が `|p|` 未満のすべてで `cls(s_j)` が `UN` か `ST`
であることをいう。ST-道はどれも UNST-道である。UNST-道 `p` についても `end(t, p) := s_{|p|}` と書く
(ST-道はどれも UNST-道なので、この 2 つの `end` は矛盾しない)。

**DEF unit に届く** -- path `p` が型 `t` の **unit に届く**とは、`T(t, p)` が abort せずに値を返し、
その値が `U(t)` の要素であることをいう。

## 2. 証明

<1>1. **(H1: 型の well-formedness)** RC IR に現れるすべての型 `t` について、次の 3 つが成り立つ。
   - (i) `t` は ground であり、`t.toplevel_tycon()` は `Some` を返し、その型構成子は `E` に登録されて
     いる。
   - (ii) `no_size_in_place` が辿る in-place 降下、すなわち型 `t'` から `held_types(t')` の要素のうち
     `is_unbox(E)` が真のものへの辺は、`t` から始めて有限の道しか作らない。
   - (iii) その降下で `t` から到達できる型も (i) を満たす。
  BY CODE src/rc_ir/ast.rs: RcVar (`ty` の doc「always concrete (monomorphic)」),
     CODE src/ast/program.rs: Program::validate_layouts,
     CODE src/type_size.rs: no_size_in_place,
     CODE src/type_size.rs: held_types,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed (その doc が同じことを述べる)

<1>2. **(H2: 変数のスコープ規律)** 関数本体の各節点 `n` について、`n` が使う変数はどれも、`Scope(n)`
   (下の `DEF Scope`) の要素であるか、この関数のどの束縛でもない名前 (グローバル) である。ここで
   `Let(x, rhs, k)` の `rhs` が使う変数は、`Scope` に `x` が入る**前**の集合、すなわち
   `Scope(Let(x, rhs, k) の節点)` で解決される。

   **DEF Scope** -- 本体の各節点 `n` に対する名前の集合 `Scope(n)` を次で定める。
   - `VarTable::of(func)` が作る表では、`Scope(根)` は `func.params` と `func.capture` の名前の集合。
     `VarTable::body_only(body)` が作る表では、`Scope(根)` は空集合。
   - `n = Let(x, rhs, k)` のとき、`Scope(k)` は `Scope(n)` に `x` を加えたもの。`rhs` が
     `Match(scrut, arms)` のとき、各アームについて `Scope(arm.body)` は `Scope(n)` に `arm.payload`
     を加えたもの。
   - `n = Destructure(cont, fields, s, k)` のとき、`Scope(k)` は `Scope(n)` に `fields` の各変数を
     加えたもの。
   - `n` が `Retain(v, p, s, k)`、`Release(v, p, s, k)`、`Eval(v, k)` のとき、`Scope(k)` は
     `Scope(n)` に等しい。
  BY CODE src/rc_ir/validate.rs: Validator::check_expr_inner,
     CODE src/rc_ir/validate.rs: Validator::check_rhs,
     CODE src/rc_ir/validate.rs: Validator::use_var,
     CODE src/rc_ir/validate.rs: Validator::bind

<1>3. `RcRhs::Match(scrut, arms)` の `arms` は空でない。
  BY A9, CODE src/rc_ir/validate.rs: Validator::check_rhs (`RcRhs::Match` の腕の `arms.is_empty()`
     検査)

<1>3a. **(H4: 束縛の形と型が合っている)** 関数本体について次の 6 つが成り立つ。
   - (i) `Let(x, RcRhs::Var(y), k)` について `ty(y)` は `ty(x)` に等しい。
   - (ii) `Let(x, RcRhs::Match(scrut, arms), k)` の各アームについて、`returned_var(&arm.body)` の型は
     `ty(x)` に等しい。
   - (iii) 同じ `Match` の各アームについて、`arm.tag` が `Some(k)` のとき `(k, ty(arm.payload))` は
     `F(ty(scrut))` の要素であり、`arm.tag` が `None` のとき `ty(arm.payload)` は `ty(scrut)` に
     等しい。
   - (iv) 同じ `Match` の `ty(scrut)` は union の型である。すなわち
     `ty(scrut).toplevel_tycon_info(E).variant` は `TyConVariant::Union` である。
   - (v) `Destructure(cont, fields, s, k)` について `ty(cont)` は構造体 (タプルを含む) の型であり、
     各 `(i, fv)` について `(i, ty(fv))` は `F(ty(cont))` の要素である。
   - (vi) 同じ名前を持つ `RcVar` の出現はどれも同じ型を持つ。したがって `vars.var_tys` が記録する型は、
     その名前を使う側の `RcVar` の `ty` に等しい。以下ではこの型を `ty(名前)` と書く。
  BY CODE src/rc_ir/ast.rs: RcRhs (`Var` の doc「Move / rename `y := x`, consuming `x`」),
     CODE src/rc_ir/ast.rs: MatchArm (`tag` と `payload` の doc),
     CODE src/rc_ir/ast.rs: RcExpr (`Destructure` の doc「Destructure a struct/tuple container into
       its fields at once ... Each `(index, var)` binds field `index` to `var`」),
     CODE src/rc_ir/ownership.rs: returned_var

<1>4. 任意の型 `t` について `cls(t)` はちょうど 1 つの値に定まる。
  <2>1. `DEF cls` の 6 つの条件は、上から順に最初に成り立つものを採る形で書かれている。よって高々
     1 つに定まる。
    BY DEF cls
  <2>2. 最後の `ST` は「上のどれにも当たらない」なので、6 つの場合は尽きている。よって 1 つ以上に
     定まる。
    BY DEF cls
  <2>3. QED
    BY <2>1, <2>2

<1>5. `t REC f` を満たす `f` が存在するとき、次の 3 つが成り立つ。
   - (a) `t.is_unbox(E)` は真である。
   - (b) `t.toplevel_tycon_info(E).variant` は `TyConVariant::Struct` か `TyConVariant::Union` で
     ある。
   - (c) `F(t)` の各要素の第 2 成分は `held_types(t)` の要素である。
  <2>1. (a) が成り立つ。`DEF REC` より `t.is_box(E)` は偽であり、`is_box` は `is_unbox` の否定で
     ある。
    BY DEF REC, CODE src/ast/types.rs: TypeNode::is_box
  <2>2. `DEF REC` より `t.is_closure()` は偽なので `t.toplevel_tycon_info(E)` の
     `assert!(!self.is_closure())` は通り、`<1>1` (i) より `t.toplevel_tycon().unwrap()` と
     `type_env.tycons().get(&tycon).unwrap()` も成功する。よって `variant` は `TyConVariant` の
     8 つの値のどれかである。
    BY <1>1, DEF REC, CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
       CODE src/ast/types.rs: TyConVariant
  <2>3. CASE `variant` が `TyConVariant::Primitive` である。
    <3>1. `TyConVariant::Primitive` を持つ型構成子は `bulitin_tycons` が登録する 13 個であり、その
       すべてが `fields: vec![]` を持つ。ユーザの型宣言が作る `TyConInfo` の `variant` は `Struct`
       か `Union` だけである。
      BY CODE src/fixstd/builtin.rs: bulitin_tycons, CODE src/ast/typedecl.rs: TypeDefn::tycon_info
    <3>2. `<3>1` より `F(t)` は空である。
      BY <3>1, CODE src/ast/types.rs: TypeNode::unpunched_field_types
    <3>3. `<3>2` は「`t REC f` を満たす `f` が存在する」という仮定に反する。`DEF REC` はそのような
       `f` が `F(t)` の要素であることを要求するからである。
      BY <3>2, DEF REC
    <3>4. QED
      BY <3>3
  <2>4. CASE `variant` が `TyConVariant::Arrow` である。
    <3>1. `TyConVariant::Arrow` は関数型構成子 `->` の variant であり、`is_closure()` はその型構成子
       の名前を問う述語なので、`t.is_closure()` は真である。
      BY CODE src/ast/types.rs: TyConVariant, CODE src/ast/types.rs: TypeNode::is_closure,
         CODE src/fixstd/builtin.rs: bulitin_tycons
    <3>2. `<3>1` は `DEF REC` (`is_closure` が偽であること) に反する。
      BY <3>1, DEF REC
    <3>3. QED
      BY <3>2
  <2>5. CASE `variant` が `TyConVariant::Array` である。
    <3>1. `TyConVariant::Array` を持つ型構成子は `Std::Array` だけであり、`is_array()` はその型構成子
       であることを問う述語なので、`t.is_array()` は真である。
      BY CODE src/ast/types.rs: TyConVariant, CODE src/fixstd/builtin.rs: bulitin_tycons,
         CODE src/fixstd/builtin.rs: is_array_tycon, CODE src/ast/types.rs: TypeNode::is_array
    <3>2. `<3>1` は `DEF REC` (`is_array` が偽であること) に反する。
      BY <3>1, DEF REC
    <3>3. QED
      BY <3>2
  <2>6. CASE `variant` が `TyConVariant::DynamicObject` である。`bulitin_tycons` が登録する
     `#DynamicObject` の `TyConInfo` は `is_unbox: false` を持つ。`DEF REC` より `t.is_closure()` は
     偽なので `t.is_unbox(E)` は `is_unbox` フィールドに等しく偽であり、`<2>1` (a) に反する。
    BY <2>1, DEF REC, CODE src/fixstd/builtin.rs: bulitin_tycons,
       CODE src/ast/types.rs: TypeNode::is_unbox
  <2>7. CASE `variant` が `TyConVariant::ArrayStorage` である。`bulitin_tycons` が登録する
     `#ArrayStorage` の `TyConInfo` は `is_unbox: false` を持つ。`DEF REC` より `t.is_closure()` は
     偽なので `t.is_unbox(E)` は偽であり、`<2>1` (a) に反する。
    BY <2>1, DEF REC, CODE src/fixstd/builtin.rs: bulitin_tycons,
       CODE src/ast/types.rs: TypeNode::is_unbox
  <2>8. CASE `variant` が `TyConVariant::Opaque` である。不透明型の型構成子を登録する
     `register_opaque_tycon` が作る `TyConInfo` は `is_unbox: false` を持つ。`DEF REC` より
     `t.is_closure()` は偽なので `t.is_unbox(E)` は偽であり、`<2>1` (a) に反する。
    BY <2>1, DEF REC, CODE src/elaboration/desugar_opaque.rs: register_opaque_tycon,
       CODE src/ast/types.rs: TypeNode::is_unbox
  <2>9. (b) が成り立つ。`TyConVariant` の値は `Primitive`、`Arrow`、`Array`、`Struct`、`Union`、
     `DynamicObject`、`ArrayStorage`、`Opaque` の 8 つである。`<2>3` から `<2>8` がそのうち 6 つを
     退けたので、残るのは `Struct` と `Union` である。
    BY <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, <2>8, CODE src/ast/types.rs: TyConVariant
  <2>10. `F(t)` の各要素の第 2 成分は `t.field_types(E)` の要素である。`unpunched_field_types` は
     `instance_field_types(tycon_info, E)` の結果を `enumerate` してから `filter` したものを返し、
     `field_types` は同じ `instance_field_types(tycon_info, E)` の結果をそのまま返す。
    BY CODE src/ast/types.rs: TypeNode::unpunched_field_types, CODE src/ast/types.rs: TypeNode::field_types
  <2>11. (c) が成り立つ。`ty_to_object_ty(t, &vec![], E)` は、`<2>9` の `Struct` のとき
     `ty.field_types(type_env)` の各要素 `field_ty` について
     `ObjectFieldType::SubObject(field_ty, punched)` を積み、`Union` のとき
     `ObjectFieldType::UnionBuf(ty.field_types(type_env))` を積む。`held_types` は `SubObject` の
     型と `UnionBuf` の型をすべて集める。よって `t.field_types(E)` の各要素は `held_types(t)` の
     要素であり、`<2>10` より `F(t)` の各要素の第 2 成分もそうである。
    BY <2>9, <2>10, CODE src/object.rs: ty_to_object_ty, CODE src/type_size.rs: held_types
  <2>12. QED
    BY <2>1, <2>9, <2>11

<1>6. `REC` は整礎である。すなわち `t_0 REC t_1 REC t_2 ...` という無限列は存在しない。
  <2>1. `t_j REC t_{j+1}` であるとき、`<1>5` (a) より `t_j` は `is_unbox(E)` が真であり、`<1>5` (c)
     より `t_{j+1}` は `held_types(t_j)` の要素である。
    BY <1>5
  <2>2. 無限列 `t_0 REC t_1 REC ...` があれば、`<2>1` をすべての `j` に適用して、各 `t_j` は
     `is_unbox(E)` が真であり `t_{j+1}` は `held_types(t_j)` の要素のうち `is_unbox` が真のもので
     ある。よってこの列は `<1>1` (ii) の in-place 降下の無限の道である。
    BY <2>1
  <2>3. QED
    `<2>2` は `<1>1` (ii) に反する。
    BY <1>1, <2>2

<1>7. `cls(t)` が `UN` か `ST` であるとき、次の 3 つが成り立つ。
   - (a) `t.is_box(E)`、`t.is_closure()`、`t.is_array()`、`t.is_funptr()` はすべて偽である。
   - (b) `t.is_fully_unboxed(E)` の値は `F(t)` の各要素の第 2 成分についての `is_fully_unboxed` の
     連言に等しく、その連言は偽である。
   - (c) `t DESC f` は `t REC f` を含意する。
  <2>1. `DEF cls` より `t.is_fully_unboxed(E)`、`t.is_closure()`、`t.is_box(E)`、`t.is_array()` は
     いずれも偽である。
    BY DEF cls
  <2>2. `is_fully_unboxed` の本体は、`is_box` が真なら偽を返し、`is_closure` が真なら偽を返し、
     `is_array` が真なら偽を返し、`is_funptr` が真なら真を返し、そのどれでもなければ `F(t)` の各要素
     の第 2 成分についての `is_fully_unboxed` の連言を返す。
    BY CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>3. (a) が成り立つ。`<2>1` が 3 つを与える。`t.is_funptr()` が真だとすると、`<2>1` と `<2>2` に
     より `is_fully_unboxed(t)` は真を返し、`<2>1` に反する。
    BY <2>1, <2>2
  <2>4. (b) が成り立つ。`<2>2` の 5 段のうち最初の 4 段の条件は `<2>1` と `<2>3` によりすべて偽なので、
     値は連言である。`<2>1` よりその値は偽である。
    BY <2>1, <2>2, <2>3
  <2>5. (c) が成り立つ。`t DESC f` は `cls(t)` が `UN` か `ST` であることと `(i, f)` が `F(t)` の
     要素であることを言う。`<2>3` (a) が `DEF REC` の 4 つの条件を与える。
    BY <2>3, DEF REC, DEF DESC
  <2>6. QED
    BY <2>3, <2>4, <2>5

<1>8. `DESC` は整礎である。
  BY <1>6, <1>7

<1>9. `is_fully_unboxed`、`unit_step`、`rc_units`、`boxed_leaf_paths` はどの型に対しても停止し、
   `L(t)` と `U(t)` は有限集合である。また `truncate_to_unit` は、abort しない限り停止する。
  <2>1. `is_fully_unboxed(t)` が自分を再帰呼び出しするのは、`is_box`、`is_closure`、`is_array`、
     `is_funptr` がすべて偽のときに、`F(t)` の各要素の第 2 成分に対してだけである。すなわち再帰の辺は
     `REC` の辺そのものである。
    BY DEF REC, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>2. `<1>6` より `REC` は整礎であり、`F(t)` は有限列なので分岐も有限である。よって
     `is_fully_unboxed` は停止する。
    BY <1>6, <2>1
  <2>3. `unit_step(t)` は `is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、`is_array`、
     `is_punched_array`、`toplevel_tycon_info`、`unpunched_field_types` を各 1 回呼ぶだけで、自分を
     再帰呼び出ししない。`<2>2` と `<1>1` (i) より、これらはすべて停止する。
    BY <1>1, <2>2, CODE src/rc_ir/ownership.rs: unit_step
  <2>4. `rc_units_go(s, q, out)` の再帰呼び出しは `unit_step(s)` が `UnitStep::Fields` を返したときの
     `held_fields` の各要素に対してだけ起き、そのとき `held_fields` は `F(s)` である。
    BY CODE src/rc_ir/ownership.rs: rc_units_go, CODE src/rc_ir/ownership.rs: unit_step
  <2>5. `unit_step(s)` が `Fields` を返すのは `is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、
     `is_array`、`is_punched_array` がすべて偽のとき、すなわち `cls(s) = ST` のときだけである。
    BY DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>6. `<2>4` と `<2>5` より `rc_units_go` の再帰の辺は `DESC` の辺である。`<1>8` と `F` の有限性より
     `rc_units_go` は停止し、`out` に積む path の個数は有限である。よって `rc_units` は停止し `U(t)`
     は有限である。
    BY <1>8, <2>3, <2>4, <2>5, DEF DESC, CODE src/rc_ir/ownership.rs: rc_units
  <2>7. `boxed_leaf_paths` の内部関数 `go(s, q, out)` の再帰呼び出しは、`is_fully_unboxed`、
     `is_closure`、`is_box`、`is_array` がすべて偽のときに `F(s)` の各要素に対してだけ起きる。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>8. `<2>7` の条件のとき `cls(s)` は `UN` か `ST` である。
    BY <2>7, DEF cls
  <2>9. `<2>7` と `<2>8` より `go` の再帰の辺は `DESC` の辺である。`<1>8` と `F` の有限性より `go` は
     停止し、`out` に積む path の個数は有限である。よって `boxed_leaf_paths` は停止し `L(t)` は
     有限である。
    BY <1>8, <2>2, <2>7, <2>8, DEF DESC, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>10. `truncate_to_unit(t, p)` の本体は `p` の要素についての `for` ループ 1 つであり、各周で
     `unit_step` と `held_field_type` を呼ぶだけで、自分を再帰呼び出ししない。`p` は有限列で、`<2>3`
     より `unit_step` は停止し、`held_field_type` は有限列の線形探索である。
    BY <2>3, CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/rc_ir/ownership.rs: held_field_type
  <2>11. QED
    BY <2>2, <2>3, <2>6, <2>9, <2>10

<1>10. `unit_step(t, E)` の返す `UnitStep` は `cls(t)` で決まり、次の表の通りである。

   | `cls(t)` | `unit_step(t, E)` |
   |---|---|
   | `NB` | `UnitStep::NoUnit` |
   | `CL` | `UnitStep::Capture { capture_idx: c, .. }` |
   | `BX` | `UnitStep::Unit` |
   | `AR` | `UnitStep::Unit` |
   | `UN` | `UnitStep::Unit` |
   | `ST` | `UnitStep::Fields { held_fields: F(t), .. }` |

  <2>1. CASE `cls(t) = NB`。定義より `t.is_fully_unboxed(E)` が真であり、`unit_step` の最初の検査が
     これなので `UnitStep::NoUnit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>2. CASE `cls(t) = CL`。定義より `is_fully_unboxed` は偽、`is_closure()` は真である。`unit_step`
     は 2 番目の検査で
     `UnitStep::Capture { capture_idx: CLOSURE_CAPTURE_IDX as usize, field_count: CLOSURE_FIELD_COUNT }`
     を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step, CODE src/constants.rs: CLOSURE_CAPTURE_IDX
  <2>3. CASE `cls(t) = BX`。定義より `is_fully_unboxed` と `is_closure` は偽、`is_box(E)` は真で
     ある。`unit_step` の 3 番目の検査は `is_box || is_union || is_array || is_punched_array` の
     選言なので真になり、`UnitStep::Unit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>4. CASE `cls(t) = AR`。定義より `is_fully_unboxed`、`is_closure`、`is_box` は偽、`is_array()` は
     真である。`<2>3` と同じ選言の第 3 項が真になり `UnitStep::Unit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>5. CASE `cls(t) = UN`。定義より `is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽、
     `is_union(E)` か `is_punched_array()` のどちらかが真である。同じ選言の第 2 項 (unbox union) か
     第 4 項 (punched array) が真になり `UnitStep::Unit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>6. CASE `cls(t) = ST`。定義より `is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、
     `is_array`、`is_punched_array` がすべて偽である。よって `unit_step` は 3 つの検査をすべて通り
     抜け、
     `UnitStep::Fields { field_count: t.toplevel_tycon_info(E).fields.len(), held_fields: t.unpunched_field_types(E) }`
     を返す。`held_fields` は `F(t)` そのものである。
    BY D5, DEF cls, DEF F, CODE src/rc_ir/ownership.rs: unit_step
  <2>7. QED
    `<1>4` より場合は尽きており排他である。
    BY <1>4, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>11. `boxed_leaf_paths` の内部関数 `go(t, path, out)` の振る舞いは `cls(t)` で決まり、次の通りで
   ある (`path` は呼び出し時の値、`out` への追加だけを書く)。

   | `cls(t)` | `go(t, path, out)` |
   |---|---|
   | `NB` | 何もしない |
   | `CL` | `path ++ [c]` を `out` に積む |
   | `BX` | `path` を `out` に積む |
   | `AR` | `path` を `out` に積む |
   | `UN` | `F(t)` の各 `(i, f)` について `go(f, path ++ [i], out)` |
   | `ST` | `F(t)` の各 `(i, f)` について `go(f, path ++ [i], out)` |

  <2>1. CASE `cls(t) = NB`。`go` の最初の検査 `is_fully_unboxed` が真なので即 `return` する。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>2. CASE `cls(t) = CL`。`is_fully_unboxed` は偽、`is_closure()` は真である。`go` は
     `path.push(CLOSURE_CAPTURE_IDX as usize)`、`out.push(path.clone())`、`path.pop()` を行って
     `return` する。積まれる path は `path ++ [c]` である。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`),
       CODE src/constants.rs: CLOSURE_CAPTURE_IDX
  <2>3. CASE `cls(t) = BX`。`is_fully_unboxed` と `is_closure` は偽、`is_box(E)` は真である。`go` は
     `out.push(path.clone())` を行って `return` する。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>4. CASE `cls(t) = AR`。`is_fully_unboxed`、`is_closure`、`is_box` は偽、`is_array()` は真で
     ある。`go` は `is_array` の検査で `out.push(path.clone())` を行って `return` する。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>5. CASE `cls(t) = UN`。`is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽である。`go`
     にはこの 4 つの検査しか無いので、そのすべてを通り抜けて最後の
     `for (i, fty) in ty.unpunched_field_types(type_env)` に進み、各要素について `path.push(i)`、
     `go(&fty, ..., path, out)`、`path.pop()` を行う。D4 の第 5 項が言うとおり、`F(t)` は穴 (punched
     field) を含まないので、穴の下へは降りない。
    BY D4, DEF cls, DEF F, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>6. CASE `cls(t) = ST`。`is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽である (`ST`
     はこの 4 つに加えて `is_union` と `is_punched_array` も偽である場合だが、`go` はその 2 つを
     問わない)。`<2>5` と同じ最後のループに進む。
    BY D4, DEF cls, DEF F, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>7. QED
    `<1>4` より場合は尽きており排他である。`UN` と `ST` で振る舞いが同じことが、この 2 つの walk の
    違いの全部である。`unit_step` は `UN` で止まり (`<1>10`)、`go` は `UN` で降りる。
    BY <1>4, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>12. `F(t)` に現れる添字は相異なる。したがって `fld(t, i)` は一価であり、`(i, f)` が `F(t)` の要素
   であるとき `held_field_type(F(t), i, w)` は abort せず `f` を返す。
  <2>1. `unpunched_field_types` は `instance_field_types(...)` の結果に `into_iter().enumerate()` を
     適用してから `filter` する。`enumerate` の添字は 0 から始まる相異なる整数の列であり、`filter` は
     その一部を残すだけである。
    BY CODE src/ast/types.rs: TypeNode::unpunched_field_types
  <2>2. `<2>1` より `F(t)` の第 1 成分は相異なる。よって「第 1 成分が `i` である要素」は高々 1 つで
     あり、`fld(t, i)` は一価である。
    BY <2>1, DEF fld
  <2>3. `held_field_type(held_fields, idx, walk_name)` は
     `held_fields.iter().find(|(i, _)| *i == idx)` を行い、見つからないときだけ `panic!` し、
     見つかったときはその第 2 成分の複製を返す。
    BY CODE src/rc_ir/ownership.rs: held_field_type
  <2>4. QED
    `(i, f)` が `F(t)` の要素なら `find` は成功し、`<2>2` よりその要素は `(i, f)` に限るので、返る値
    は `f` である。
    BY <2>2, <2>3

<1>13. 任意の型 `t` について、`U(t)` は次の 2 つの集合の合併に等しい。
   - `{ p : p は t の ST-道で、cls(end(t, p)) が BX、AR、UN のどれか }`
   - `{ p ++ [c] : p は t の ST-道で、cls(end(t, p)) = CL }`
  <2>1. `rc_units(t)` は `out` を空にして `rc_units_go(t, E, &mut vec![], &mut out)` を呼び、`out` を
     返す。
    BY CODE src/rc_ir/ownership.rs: rc_units
  <2>2. `rc_units_go` の `UnitStep::Capture` の腕は `path.push(capture_idx)`、
     `out.push(path.clone())`、`path.pop()` を行い、`UnitStep::Unit` の腕は `out.push(path.clone())`
     を行い、`UnitStep::Fields` の腕は各フィールドについて `path.push(i)`、再帰、`path.pop()` を
     行う。`UnitStep::NoUnit` の腕は何もしない。よって、呼び出し時の `path` の値を `q` と書くと、
     再帰から戻ったとき `path` は `q` に戻っている。
    BY CODE src/rc_ir/ownership.rs: rc_units_go
  <2>3. `rc_units_go(s, E, path, out)` が `out` に積む path の集合を `R(s, q)` (`q` は呼び出し時の
     `path` の値) と書くと、`<1>10` と `<2>2` より次が成り立つ。
     - `cls(s) = NB` のとき `R(s, q)` は空集合
     - `cls(s) = CL` のとき `R(s, q)` は `{q ++ [c]}`
     - `cls(s)` が `BX`、`AR`、`UN` のとき `R(s, q)` は `{q}`
     - `cls(s) = ST` のとき `R(s, q)` は `F(s)` の各 `(i, f)` についての `R(f, q ++ [i])` の合併
    BY <1>10, <2>2, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>4. `<2>3` の再帰式を `DESC` に関する整礎帰納法 (`<1>8`) で展開する。`R(t, [])` の要素は、
     `cls(s_j) = ST` (`j` は `|p|` 未満) を満たす道 `p` をたどって到達した型 `end(t, p)` のクラスが
     `NB` でも `ST` でもないところで積まれた path である。すなわち `R(t, [])` は
     `{ p : p は t の ST-道で cls(end(t, p)) が BX、AR、UN のどれか }` と
     `{ p ++ [c] : p は t の ST-道で cls(end(t, p)) = CL }` の合併である。
    BY <1>8, <2>3, DEF ST-道, DEF fld
  <2>5. QED
    BY <2>1, <2>4

<1>14. 任意の型 `t` について、`L(t)` は次の 2 つの集合の合併に等しい。
   - `{ p : p は t の UNST-道で、cls(end(t, p)) が BX か AR }`
   - `{ p ++ [c] : p は t の UNST-道で、cls(end(t, p)) = CL }`

   さらに、`L(t)` の要素 `lam` に対するこの分解 `lam = p ++ e` は一意である。すなわち `p` は、`lam`
   の前置のうち `t` の UNST-道であって `cls(end(t, p))` が `UN` でも `ST` でもないもの、ただ 1 つで
   ある。
  <2>1. `boxed_leaf_paths(t)` は `out` を空にして `go(t, E, &mut Vec::new(), &mut out)` を呼び、`out`
     を返す。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>2. `go` はクロージャの腕で `path.push`、`out.push(path.clone())`、`path.pop()` を行い、フィールド
     のループで `path.push(i)`、再帰、`path.pop()` を行う。よって、呼び出し時の `path` の値を `q` と
     書くと、再帰から戻ったとき `path` は `q` に戻っている。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>3. `go(s, E, path, out)` が `out` に積む path の集合を `G(s, q)` と書くと、`<1>11` と `<2>2` より
     次が成り立つ。
     - `cls(s) = NB` のとき `G(s, q)` は空集合
     - `cls(s) = CL` のとき `G(s, q)` は `{q ++ [c]}`
     - `cls(s)` が `BX` か `AR` のとき `G(s, q)` は `{q}`
     - `cls(s)` が `UN` か `ST` のとき `G(s, q)` は `F(s)` の各 `(i, f)` についての `G(f, q ++ [i])`
       の合併
    BY <1>11, <2>2, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>4. `<2>3` の再帰式を `DESC` に関する整礎帰納法 (`<1>8`) で展開する。`G(t, [])` の要素は、
     `cls(s_j)` が `UN` か `ST` (`j` は `|p|` 未満) を満たす道 `p` をたどって到達した型 `end(t, p)`
     のクラスが `NB` でも `UN` でも `ST` でもないところで積まれた path である。すなわち `G(t, [])` は
     `{ p : p は t の UNST-道で cls(end(t, p)) が BX か AR }` と
     `{ p ++ [c] : p は t の UNST-道で cls(end(t, p)) = CL }` の合併である。
    BY <1>8, <2>3, DEF UNST-道, DEF fld
  <2>5. 分解の一意性。`lam` が `L(t)` の要素で `lam = p ++ e` が `<2>4` の分解であるとする。`q` が
     `lam` の別の前置で `t` の UNST-道であり `cls(end(t, q))` が `UN` でも `ST` でもないとする。
     `|q|` が `|p|` 未満なら、`p` が UNST-道であることから `cls(end(t, q)) = cls(s_{|q|})` は `UN` か
     `ST` であり、仮定に反する。`|q|` が `|p|` より大きいなら、`q` が UNST-道であることから
     `cls(s_{|p|}) = cls(end(t, p))` は `UN` か `ST` であり、`<2>4` の `cls(end(t, p))` が `BX`、
     `AR`、`CL` のどれかであることに反する。よって `|q| = |p|`、すなわち `q = p` である。
    BY <2>4, DEF UNST-道
  <2>6. QED
    BY <2>1, <2>4, <2>5

<1>15. `p` が `t` の UNST-道であり、`r` が `L(end(t, p))` の要素であるとき、`p ++ r` は `L(t)` の
   要素である。
  <2>1. `s := end(t, p)` と置く。`<1>14` を `s` に適用すると `r = q ++ e` と書ける。ここで `q` は `s`
     の UNST-道で、`cls(end(s, q))` は `BX`、`AR`、`CL` のどれかであり、`CL` のとき `e = [c]`、他の
     とき `e = []` である。
    BY <1>14
  <2>2. CASE `|q|` が 0 より大きい。`q` が `s` の UNST-道であることを `j = 0` に適用すると `cls(s)` は
     `UN` か `ST` である。したがって `p ++ q` は `t` の UNST-道である。位置 `j` (`j` は `|p|` 未満) の
     型は `p` が `t` の UNST-道であることから `UN` か `ST`、位置 `|p|` の型は `s` で `UN` か `ST`、
     位置 `|p| + j` (`j` は 1 以上 `|q|` 未満) の型は `q` の位置 `j` の型で `UN` か `ST` である。
    BY <2>1, DEF UNST-道
  <2>3. CASE `|q|` が 0。このとき `p ++ q` は `p` に等しく、`p` は `t` の UNST-道である。
    BY DEF UNST-道
  <2>4. `<2>2` と `<2>3` のどちらの場合も `end(t, p ++ q) = end(s, q)` である。`fld` の連鎖を位置
     `|p|` で切ると、前半が `p` の連鎖、後半が `q` の連鎖になる。
    BY <2>2, <2>3, DEF UNST-道, DEF fld
  <2>5. QED
    `<2>2` と `<2>3` は `|q|` が 0 かどうかで尽きている。`<2>4` より `p ++ q` は `t` の UNST-道で、
    `cls(end(t, p ++ q))` は `<2>1` の `cls(end(s, q))` に等しく `BX`、`AR`、`CL` のどれかである。
    `<1>14` の特徴づけを `t` に適用すると `p ++ q ++ e`、すなわち `p ++ r` は `L(t)` の要素である。
    BY <1>14, <2>1, <2>2, <2>3, <2>4

<1>16. `cls(s)` が `NB` でないとき `L(s)` は空でない。
  <2>1. CASE `cls(s) = CL`。`<1>14` で `p = []` を取ると `end(s, [])` は `s` でそのクラスは `CL` なので、
     `[c]` は `L(s)` の要素である。
    BY <1>14, DEF UNST-道
  <2>2. CASE `cls(s)` が `BX` か `AR`。`<1>14` で `p = []` を取ると `[]` は `L(s)` の要素である。
    BY <1>14, DEF UNST-道
  <2>3. CASE `cls(s)` が `UN` か `ST`。
    <3>1. `<1>7` (b) より `is_fully_unboxed(s)` の値は `F(s)` の各要素の第 2 成分についての
       `is_fully_unboxed` の連言に等しく、その連言は偽である。よってある `(i, f)` が `F(s)` の要素で
       `f.is_fully_unboxed(E)` が偽である。
      BY <1>7
    <3>2. `<3>1` の `f` は `cls(f)` が `NB` でないことを満たす。
      BY <3>1, DEF cls
    <3>3. `s DESC f` である。
      BY <3>1, DEF DESC
    <3>4. `DESC` に関する整礎帰納法 (`<1>8`) の仮定を `f` に適用すると `L(f)` は空でない。その要素を
       `r` とする。
      BY <1>8, <3>2, <3>3
    <3>5. `[i]` は `s` の UNST-道であり `end(s, [i]) = f` である。`j = 0` の唯一の条件は `cls(s)` が
       `UN` か `ST` であることで、これは場合の仮定である。
      BY <3>1, DEF UNST-道, DEF fld
    <3>6. QED
      `<1>15` を `t := s`、`p := [i]`、`r := r` に適用すると、`[i] ++ r` は `L(s)` の要素である。
      よって `L(s)` は空でない。
      BY <1>15, <3>4, <3>5
  <2>4. QED
    `<1>4` より `cls(s)` は 6 つのどれかであり、仮定から `NB` は除かれる。残る 5 つを `<2>1`、`<2>2`、
    `<2>3` が尽くしている。
    BY <1>4, <2>1, <2>2, <2>3

<1>17. `lam` が `L(t)` の要素であるとする。`<1>14` により `lam` は `lam = p ++ e` と一意に分解される
   (`p` は `t` の UNST-道、`cls(end(t, p))` は `CL`、`BX`、`AR` のどれか、`e` は `CL` のとき `[c]`、
   他のとき `[]`)。`s_j` を `lam[0..j]` の位置の型 (`j` は `|p|` 以下)、`k` を「`cls(s_j)` が `ST`
   でない `j` (`|p|` 以下) のうち最小のもの」とすると、次の 3 つが成り立つ。
   - (i) `k` は定義され、`k` は `|p|` 以下であり、`|p|` は `|lam|` 以下である。また `j` が `k` 未満の
     すべてで `cls(s_j) = ST` である。
   - (ii) `k` が `|p|` 未満のとき `cls(s_k) = UN` であり、`k = |p|` のとき `cls(s_k)` は `CL`、`BX`、
     `AR` のどれかである。
   - (iii) `T(t, lam)` は abort せずに値を返す。とくに `UnitStep::NoUnit` の腕の `panic!`
     (「holds no reference」) にも `UnitStep::Capture` の腕の `assert_eq!` にも達しない。返り値は、
     `cls(s_k)` が `UN`、`BX`、`AR` のとき `lam[0..k]`、`cls(s_k)` が `CL` のとき `lam[0..k+1]` で
     ある。
  <2>1. (i) が成り立つ。`cls(s_{|p|})` は `cls(end(t, p))` に等しく、`<1>14` より `CL`、`BX`、`AR` の
     どれかであって `ST` ではない。よって `k` を取る対象の集合は `|p|` を含み空でなく、`k` は定義
     されて `|p|` 以下である。`e` は `[]` か `[c]` なので `|p|` は `|lam|` 以下である。`k` の最小性
     から、`j` が `k` 未満のすべてで `cls(s_j) = ST` である。
    BY <1>14
  <2>2. (ii) が成り立つ。`j` が `|p|` 未満のとき、`p` が `t` の UNST-道であることから `cls(s_j)` は
     `UN` か `ST` である。`k` が `|p|` 未満なら `cls(s_k)` は `ST` でないので `UN` である。`k = |p|`
     なら `cls(s_k)` は `cls(end(t, p))` に等しく、`<1>14` より `CL`、`BX`、`AR` のどれかである。
    BY <1>14, DEF UNST-道
  <2>3. `truncate_to_unit(t, lam, E)` は `out := []`、`cur := t` から始め、`lam` の各要素 `idx` に
     ついて `unit_step(cur, E)` で場合分けする `for` ループを回し、ループを抜けたら `out` を返す。
     場合分けは `UnitStep::NoUnit` のとき `panic!`、`UnitStep::Capture { capture_idx, .. }` のとき
     `assert_eq!(idx, capture_idx, ...)` と `out.push(idx)` と `break`、`UnitStep::Unit` のとき
     `break` (`out` に積まない)、`UnitStep::Fields { held_fields, .. }` のとき `out.push(idx)` と
     `cur = held_field_type(&held_fields, idx, "truncate_to_unit")` である。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>4. `j` が 0 以上 `k` 以下のすべてについて、ループの最初の `j` 周はすべて `UnitStep::Fields` の腕
     を通って完了し、その時点で `cur = s_j` かつ `out = lam[0..j]` である。
    <3>1. `j = 0` のとき。0 周が完了した時点で `cur = t = s_0` かつ `out = [] = lam[0..0]` である。
      BY <2>3
    <3>2. ASSUME: `j` は `k` 未満であり、最初の `j` 周がすべて `Fields` の腕を通って完了し、その時点で
       `cur = s_j` かつ `out = lam[0..j]` である
       PROVE: 最初の `j+1` 周がすべて `Fields` の腕を通って完了し、その時点で `cur = s_{j+1}` かつ
       `out = lam[0..j+1]` である
      <4>1. `j` は `k` 未満で `k` は `|p|` 以下、`|p|` は `|lam|` 以下なので、`j` は `|lam|` 未満で
         ある。よってループは第 `j` 周に入る。
        BY <2>1
      <4>2. `<2>1` より `cls(s_j) = ST` である。
        BY <2>1
      <4>3. `<1>10` と `<4>2` より `unit_step(s_j, E)` は
         `UnitStep::Fields { held_fields: F(s_j), .. }` である。
        BY <1>10, <4>2
      <4>4. 第 `j` 周の `idx` は `lam[j]` である。`j` は `k` 以下で `k` は `|p|` 以下なので
         `lam[j] = p[j]` であり、`p` が `t` の UNST-道であることから `(lam[j], s_{j+1})` は `F(s_j)`
         の要素である。
        BY <2>1, DEF UNST-道, DEF fld
      <4>5. `<1>12`、`<4>3`、`<4>4` より `held_field_type(&F(s_j), lam[j], "truncate_to_unit")` は
         abort せず `s_{j+1}` を返す。
        BY <1>12, <4>3, <4>4
      <4>6. QED
        `<2>3` の `Fields` の腕により、第 `j` 周の後 `out` は `lam[0..j] ++ [lam[j]] = lam[0..j+1]`
        になり、`cur` は `s_{j+1}` になる。
        BY <2>3, <4>1, <4>3, <4>5
    <3>3. QED
      `<3>1` を基底、`<3>2` を帰納段とする `j` についての帰納法。
      BY <3>1, <3>2
  <2>5. CASE `k = |lam|`。
    <3>1. ループは第 `k` 周に入らずに終わる。`<2>4` を `j = k` に適用すると、その時点で
       `out = lam[0..k] = lam` である。
      BY <2>4
    <3>2. `k` は `|p|` 以下で `|p|` は `|lam| = k` 以下なので `|p| = |lam|`、すなわち `e = []` かつ
       `k = |p|` である。`<2>2` より `cls(s_k)` は `CL`、`BX`、`AR` のどれかである。
      BY <2>1, <2>2
    <3>3. `cls(s_k) = CL` ではない。`cls(end(t, p)) = CL` なら `<1>14` の分解は `e = [c]` を与え、
       `<3>2` の `e = []` に反する。
      BY <1>14, <3>2
    <3>4. QED
      `<3>2` と `<3>3` より `cls(s_k)` は `BX` か `AR` であり、(iii) の第 1 の式は
      `T(t, lam) = lam[0..k] = lam` を与える。`<3>1` よりこれは実際の返り値である。ループの中で
      `panic!` にも `assert_eq!` にも達していない。
      BY <3>1, <3>2, <3>3
  <2>6. CASE `k` が `|lam|` 未満。
    <3>1. `<2>4` を `j = k` に適用すると、ループは第 `k` 周に入り、その時点で `cur = s_k`、
       `out = lam[0..k]`、`idx = lam[k]` である。
      BY <2>4
    <3>2. `cls(s_k)` は `ST` でも `NB` でもない。`ST` でないのは `k` の定義から、`NB` でないのは
       `<2>2` (値は `UN`、`CL`、`BX`、`AR` のどれか) からである。
      BY <2>2
    <3>3. CASE `cls(s_k)` が `UN`、`BX`、`AR` のどれか。`<1>10` より
       `unit_step(s_k, E) = UnitStep::Unit` であり、`<2>3` のその腕は `out` に積まずに `break` する。
       よって返り値は `lam[0..k]` であり、(iii) の第 1 の式と一致する。この腕は `panic!` も
       `assert_eq!` も持たない。
      BY <1>10, <2>3, <3>1
    <3>4. CASE `cls(s_k) = CL`。
      <4>1. `<2>2` より `k = |p|` である。`k` が `|p|` 未満なら `cls(s_k) = UN` となり、この場合の
         仮定に反するからである。
        BY <2>2
      <4>2. `<4>1` より `cls(end(t, p)) = cls(s_{|p|}) = CL` である。`<1>14` の分解はこのとき
         `e = [c]` を与えるので、`lam = p ++ [c]`、`|lam| = |p| + 1 = k + 1`、`lam[k] = c` である。
        BY <1>14, <4>1
      <4>3. `<1>10` より `unit_step(s_k, E)` は `UnitStep::Capture { capture_idx: c, .. }` である。
        BY <1>10
      <4>4. `<2>3` の `Capture` の腕は `assert_eq!(idx, capture_idx, ...)` を置き、`out.push(idx)`
         して `break` する。`<3>1` と `<4>2` より `idx = lam[k] = c` であり、`<4>3` より
         `capture_idx = c` なので `assert_eq!` は通る。
        BY <2>3, <3>1, <4>2, <4>3
      <4>5. QED
        返り値は `lam[0..k] ++ [c] = lam[0..k+1]` であり、(iii) の第 2 の式と一致する。`<4>4` より
        `assert_eq!` は通り、`panic!` の腕には達しない。
        BY <3>1, <4>4
    <3>5. QED
      `<3>2` より `cls(s_k)` は `UN`、`BX`、`AR`、`CL` のどれかで、`<3>3` と `<3>4` がそれを尽くして
      いる。`truncate_to_unit` が `panic!` する腕は `UnitStep::NoUnit`、すなわち `cls(s_k) = NB` の
      ときだけで (`<1>10`)、`<3>2` がそれを排除している。
      BY <1>10, <3>2, <3>3, <3>4
  <2>7. QED
    `k` は `|p|` 以下で `|p|` は `|lam|` 以下なので、`<2>5` と `<2>6` で場合は尽きている。`<1>9` より
    ループは停止する。(i) は `<2>1`、(ii) は `<2>2`、(iii) は `<2>5` と `<2>6` である。
    BY <1>9, <2>1, <2>2, <2>5, <2>6

<1>18. **(P1 の前半)** `<1>1` を満たす任意の型 `t` と `L(t)` の任意の要素 `lam` について、`T(t, lam)`
   は `U(t)` の要素である。
  <2>1. `<1>17` の記号 (`p`、`e`、`s_j`、`k`) を使う。
    BY <1>17
  <2>2. CASE `k` が `|p|` 未満。
    <3>1. `<1>17` (ii) より `cls(s_k) = UN` である。
      BY <1>17
    <3>2. `<1>17` (iii) と `<3>1` より `T(t, lam) = lam[0..k]` である。
      BY <1>17, <3>1
    <3>3. `lam[0..k]` は `t` の ST-道である。`j` が `k` 未満のすべてで `cls(s_j) = ST` であることは
       `<1>17` (i) から従う。
      BY <1>17, DEF ST-道
    <3>4. `end(t, lam[0..k]) = s_k` であり、`<3>1` よりそのクラスは `UN` である。
      BY <3>1, <3>3, DEF ST-道
    <3>5. QED
      `<1>13` の第 1 の集合に `<3>3` と `<3>4` を当てはめると `lam[0..k]` は `U(t)` の要素である。
      `<3>2` よりこれは `T(t, lam)` に等しい。
      BY <1>13, <3>2, <3>3, <3>4
  <2>3. CASE `k = |p|`。
    <3>1. `p` は `t` の ST-道である。`j` が `|p| = k` 未満のすべてで `cls(s_j) = ST` であることは
       `<1>17` (i) から従う。
      BY <1>17, DEF ST-道
    <3>2. `end(t, p) = s_{|p|} = s_k` であり、`<1>17` (ii) よりそのクラスは `CL`、`BX`、`AR` の
       どれかである。
      BY <1>17, <3>1
    <3>3. CASE `cls(s_k)` が `BX` か `AR`。`<1>14` の分解より `e = []`、すなわち `lam = p` である。
       `<1>17` (iii) より `T(t, lam) = lam[0..k] = lam[0..|p|] = p = lam` である。`<1>13` の第 1 の
       集合に `<3>1` とこの場合の仮定を当てはめると `p` は `U(t)` の要素である。
      BY <1>13, <1>14, <1>17, <3>1
    <3>4. CASE `cls(s_k) = CL`。`<1>14` の分解より `e = [c]`、すなわち `lam = p ++ [c]` である。
       `<1>17` (iii) より `T(t, lam) = lam[0..k+1] = lam[0..|p|+1] = p ++ [c] = lam` である。`<1>13`
       の第 2 の集合に `<3>1` とこの場合の仮定を当てはめると `p ++ [c]` は `U(t)` の要素である。
      BY <1>13, <1>14, <1>17, <3>1
    <3>5. QED
      `<3>2` より `cls(s_k)` は `CL`、`BX`、`AR` のどれかで、`<3>3` と `<3>4` がそれを尽くしている。
      BY <3>2, <3>3, <3>4
  <2>4. QED
    `<1>17` (i) より `k` は `|p|` 以下なので `<2>2` と `<2>3` で場合は尽きている。
    BY <1>17, <2>1, <2>2, <2>3

<1>19. **(P1 の後半)** `<1>1` を満たす任意の型 `t` と `U(t)` の任意の要素 `u` について、
   `T(t, lam) = u` となる `L(t)` の要素 `lam` が存在する。
  <2>1. `<1>13` より `u` は次の 2 つのどちらかの形をしている。
     - (a) `u = p` で `p` は `t` の ST-道、`cls(end(t, p))` は `BX`、`AR`、`UN` のどれか。
     - (b) `u = p ++ [c]` で `p` は `t` の ST-道、`cls(end(t, p)) = CL`。
    BY <1>13
  <2>2. どちらの形でも `p` は `t` の UNST-道である。ST-道はどれも UNST-道だからである。
    BY <2>1, DEF ST-道, DEF UNST-道
  <2>3. どちらの形でも、`j` が `|p|` 未満のすべてで `cls(s'_j) = ST` である。ここで `s'_j` は
     `p[0..j]` の位置の型である。
    BY <2>1, DEF ST-道
  <2>4. CASE `u` が (a) の形で `cls(end(t, p))` が `BX` か `AR`。
    <3>1. `lam := p` と置く。`<1>14` の第 1 の集合に `<2>2` とこの場合の仮定を当てはめると `lam` は
       `L(t)` の要素である。
      BY <1>14, <2>2
    <3>2. `<1>17` を `lam` に適用する。`<1>14` の分解の一意性より、`lam` の分解の `p` はこの `p` に
       等しく `e = []` である。`lam` に沿う `s_j` は `s'_j` に等しい。
      BY <1>14, <3>1
    <3>3. `<2>3` と `<3>2` より、`j` が `|p|` 未満のすべてで `cls(s_j) = ST` であり、
       `cls(s_{|p|}) = cls(end(t, p))` はこの場合の仮定より `ST` でない。よって `k = |p|` である。
      BY <1>17, <2>3, <3>2
    <3>4. `<1>17` (iii) の第 1 の式より `T(t, lam) = lam[0..k] = lam[0..|p|] = p = u` である。
      BY <1>17, <3>3
    <3>5. QED
      BY <3>1, <3>4
  <2>5. CASE `u` が (b) の形。
    <3>1. `lam := p ++ [c]` と置く。これは `u` に等しい。`<1>14` の第 2 の集合に `<2>2` とこの場合の
       仮定を当てはめると `lam` は `L(t)` の要素である。
      BY <1>14, <2>2
    <3>2. `<1>17` を `lam` に適用する。`<1>14` の分解の一意性より、`lam` の分解の `p` はこの `p` に
       等しく `e = [c]` である。`lam` に沿う `s_j` は `s'_j` に等しい。
      BY <1>14, <3>1
    <3>3. `<2>3` と `<3>2` より、`j` が `|p|` 未満のすべてで `cls(s_j) = ST` であり、
       `cls(s_{|p|}) = cls(end(t, p)) = CL` は `ST` でない。よって `k = |p|` である。
      BY <1>17, <2>3, <3>2
    <3>4. `<1>17` (iii) の第 2 の式より `T(t, lam) = lam[0..k+1] = lam[0..|p|+1] = p ++ [c] = u` で
       ある。
      BY <1>17, <3>3
    <3>5. QED
      BY <3>1, <3>4
  <2>6. CASE `u` が (a) の形で `cls(end(t, p)) = UN`。この場合は D5 が挙げる 2 か所、すなわち unbox
     union と punched array の両方を覆う。
    <3>1. `s := end(t, p)` と置く。`<1>16` より `L(s)` は空でない。その要素を `r` とする。
      BY <1>16
    <3>2. `r` は空でない。`<1>14` を `s` に適用すると `r = q ++ e'` で `q` は `s` の UNST-道、
       `cls(end(s, q))` は `BX`、`AR`、`CL` のどれかである。`r = []` なら `q = []` かつ `e' = []` で
       あり、そのとき `end(s, q) = s` でそのクラスは `UN` となって `BX`、`AR`、`CL` のどれでもないので
       矛盾する。
      BY <1>14, <3>1
    <3>3. `lam := p ++ r` と置く。`<2>2` と `<3>1` から `<1>15` が使え、`lam` は `L(t)` の要素で
       ある。
      BY <1>15, <2>2, <3>1
    <3>4. `lam` に沿う位置 `j` (`j` は `|p|` 以下) の型は `s'_j` である。`lam` の先頭 `|p|` 要素は
       `p` に等しいからである。とくに `lam` に沿う位置 `|p|` の型は `s'_{|p|} = s` である。
      BY <3>3, DEF UNST-道, DEF fld
    <3>5. `lam` の `<1>14` の分解 `lam = p_lam ++ e_lam` について、`|p_lam|` は `|p|` より大きい。
       `<2>3` と場合の仮定より、`j` が `|p|` 以下のすべてで `cls(s'_j)` は `UN` か `ST` である。
       `<1>14` の一意性の言い換えより `p_lam` は「`lam` の前置のうち `t` の UNST-道であって
       `cls(end(t, p_lam))` が `UN` でも `ST` でもないもの」であり、`|p_lam|` が `|p|` 以下なら
       `cls(end(t, p_lam)) = cls(s'_{|p_lam|})` が `UN` か `ST` になって矛盾する。
      BY <1>14, <2>3, <3>3, <3>4
    <3>6. `<1>17` を `lam` に適用したときの `k` は `|p|` である。`<2>3` と `<3>4` より `j` が `|p|`
       未満のすべてで `cls(s_j) = ST` であり、`<3>4` と場合の仮定より `cls(s_{|p|}) = UN` は `ST` で
       ない。`<3>5` より `|p|` は `|p_lam|` 以下なので、`|p|` は `k` を取る対象の範囲に入る。
      BY <1>17, <2>3, <3>4, <3>5
    <3>7. `<1>17` (iii) の第 1 の式 (`cls(s_k) = UN` なので) より
       `T(t, lam) = lam[0..k] = lam[0..|p|] = p = u` である。
      BY <1>17, <3>4, <3>6
    <3>8. QED
      BY <3>3, <3>7
  <2>7. QED
    `<2>1` の 2 つの形のうち (a) を `<2>4` と `<2>6` が `cls(end(t, p))` の値で尽くし ((a) のクラスは
    `BX`、`AR`、`UN` のどれか)、(b) を `<2>5` が尽くしている。
    BY <2>1, <2>4, <2>5, <2>6

<1>20. **P1 が成り立つ。**
  BY <1>18, <1>19

<1>21. `VarTable::of(func)` または `VarTable::body_only(body)` が作る表 `vars` について、
   `vars.bindings` への挿入は 1 つの有限列をなし、その列に現れる名前は相異なる。`ord(y)` を、`y` が
   その列に現れるときはその位置、現れないときは `-1` と定める。
  <2>1. `VarTable::of(func)` は、`func.params` と `func.capture` の各要素について
     `vars.bindings.insert(p.name.clone(), Binding::Param)` を行い、そののち
     `collect_bindings(&func.body, &mut vars)` を呼ぶ。
    BY CODE src/rc_ir/ownership.rs: VarTable::of
  <2>2. `VarTable::body_only(body)` は `collect_bindings(body, &mut vars)` だけを呼ぶ。
    BY CODE src/rc_ir/ownership.rs: VarTable::body_only
  <2>3. `collect_bindings` は本体の木を辿り、有限個の `vars.bindings.insert` を行う。D2 より本体は
     有限の木である。
    BY D2, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>4. `<2>1` から `<2>3` より挿入は有限列をなす。挿入される名前は、パラメータ、capture、`Let` の
     束縛変数、`Destructure` のフィールド変数、`Match` のアームの payload であり、どれもプログラムの
     束縛変数である。A6 よりこれらの名前は相異なる。
    BY A6, <2>1, <2>2, <2>3, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>5. QED
    BY <2>4

<1>22. `collect_bindings` の各節点での挿入の順序は次の通りである。節点 `n` の呼び出しが行う挿入の全体を
   `Ins(n)` と書くと、`Ins(n)` は `<1>21` の列の連続する区間を占め、`n` の子の `Ins` は `Ins(n)` に
   含まれる。
   - `n = Ret(v)`: `Ins(n)` は空である。
   - `n = Let(x, Match(scrut, arms), k)`: 各アームについて順に `arm.payload` を挿入してから
     `Ins(arm.body)` を行い、それがすべて済んでから `x` を挿入し、そののち `Ins(k)` を行う。
   - `n = Let(x, rhs, k)` (`rhs` が `Match` でない): `x` を挿入し、そののち `Ins(k)` を行う。
   - `n = Destructure(cont, fields, s, k)`: `fields` の各変数を順に挿入し、そののち `Ins(k)` を行う。
   - `n` が `Retain(v, p, s, k)`、`Release(v, p, s, k)`、`Eval(v, k)`: `Ins(n)` は `Ins(k)` に
     等しい。
  <2>1. `RcExpr::Ret(_)` の腕は何もしない。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>2. `RcExpr::Let(x, rhs, k)` の腕は、まず `rhs` に応じて `binding` を作り、そののち
     `vars.bindings.insert(x.name.clone(), binding)`、`vars.var_tys.insert(...)`、
     `collect_bindings(k, vars)` を行う。`rhs` が `RcRhs::Match(scrut, arms)` のときの `binding` の
     構成は、各 `arm` について
     `vars.bindings.insert(arm.payload.name.clone(), Binding::Payload(scrut.clone(), arm.tag))`、
     `vars.var_tys.insert(...)`、`collect_bindings(&arm.body, vars)`、
     `arm_results.push(returned_var(&arm.body).clone())` を順に行い、最後に
     `Binding::Join(arm_results)` を返す。`rhs` が他の 4 つの形のときは `vars.bindings` への挿入を
     行わない。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>3. `RcExpr::Destructure(container, fields, _state, k)` の腕は、`fields` の各 `(idx, fv)` に
     ついて `vars.bindings.insert(fv.name.clone(), Binding::Field(container.clone(), *idx))` と
     `vars.var_tys.insert(...)` を行い、そののち `collect_bindings(k, vars)` を行う。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>4. `RcExpr::Retain(..)`、`RcExpr::Release(..)`、`RcExpr::Eval(..)` の腕は
     `collect_bindings(k, vars)` だけを行う。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>5. `<2>1` から `<2>4` より、各節点の呼び出しは自分の挿入と子の呼び出しだけを行い、他の節点の
     呼び出しの途中に割り込まない。よって `Ins(n)` は連続区間であり、子の `Ins` はその中に含まれる。
    BY <2>1, <2>2, <2>3, <2>4
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>23. 本体の各節点 `n` について、`Scope(n)` の要素であり `<1>21` の列に現れる名前はどれも、`Ins(n)`
   が始まるより前に挿入されている。
  <2>1. `n` が根のとき。`VarTable::of` の場合 `Scope(根)` はパラメータと capture の名前で、これらは
     `collect_bindings` を呼ぶ前に挿入される。`VarTable::body_only` の場合 `Scope(根)` は空集合で
     ある。
    BY <1>21, DEF Scope, CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only
  <2>2. ASSUME: `n` について主張が成り立ち、`n'` は `DEF Scope` が `Scope(n')` を `Scope(n)` から
     定める子である
     PROVE: `n'` について主張が成り立つ
    <3>1. CASE `n = Let(x, rhs, k)` かつ `n' = k`。`Scope(k)` は `Scope(n)` に `x` を加えたもので
       ある。`Scope(n)` の名前は帰納法の仮定より `Ins(n)` の開始より前に挿入されており、`Ins(n)` の
       開始は `Ins(k)` の開始より前である。`x` は `<1>22` より `Ins(k)` が始まる直前に挿入される。
      BY <1>22, DEF Scope
    <3>2. CASE `n = Let(x, Match(scrut, arms), k)` かつ `n'` が `arms` のあるアームの `body`。
       `Scope(arm.body)` は `Scope(n)` に `arm.payload` を加えたものである。`Scope(n)` の名前は
       帰納法の仮定より `Ins(n)` の開始より前に挿入されており、`Ins(n)` の開始は `Ins(arm.body)` の
       開始より前である。`arm.payload` は `<1>22` より `Ins(arm.body)` が始まる直前に挿入される。
      BY <1>22, DEF Scope
    <3>3. CASE `n = Destructure(cont, fields, s, k)` かつ `n' = k`。`Scope(k)` は `Scope(n)` に
       `fields` の各変数を加えたものである。`Scope(n)` の名前は帰納法の仮定より `Ins(n)` の開始より
       前に挿入されており、`Ins(n)` の開始は `Ins(k)` の開始より前である。`fields` の各変数は
       `<1>22` より `Ins(k)` が始まる前に挿入される。
      BY <1>22, DEF Scope
    <3>4. CASE `n` が `Retain(v, p, s, k)`、`Release(v, p, s, k)`、`Eval(v, k)` のどれかで `n' = k`。
       `Scope(k)` は `Scope(n)` に等しく `Ins(n)` は `Ins(k)` に等しいので、帰納法の仮定がそのまま
       主張である。
      BY <1>22, DEF Scope
    <3>5. QED
      `DEF Scope` が `Scope` を定める親子の組はこの 4 つで尽きている。
      BY DEF Scope, <3>1, <3>2, <3>3, <3>4
  <2>3. QED
    `<2>1` を基底、`<2>2` を帰納段とする、根からの深さについての帰納法。
    BY <2>1, <2>2

<1>24. 本体の節点 `n'` が節点 `n` の部分木に属するとき、`Scope(n')` の各名前は、`Scope(n)` の要素で
   あるか、`Ins(n)` の中で挿入される。
  <2>1. `n' = n` のとき `Scope(n')` は `Scope(n)` に等しい。
    BY DEF Scope
  <2>2. ASSUME: `n''` は `n` の部分木に属し、`n''` について主張が成り立ち、`n'` は `DEF Scope` が
     `Scope(n')` を `Scope(n'')` から定める子である
     PROVE: `n'` について主張が成り立つ
    <3>1. `DEF Scope` の 4 つの形のどれでも、`Scope(n')` は `Scope(n'')` に、`n''` の節点が挿入する
       名前 (`Let` の `x`、`Match` のアームの `payload`、`Destructure` のフィールド変数) を 0 個
       以上加えたものである。
      BY DEF Scope
    <3>2. `<3>1` で加わる名前はどれも `Ins(n'')` の中で挿入される。
      BY <1>22, <3>1
    <3>3. `n''` は `n` の部分木に属するので `Ins(n'')` は `Ins(n)` に含まれる。
      BY <1>22
    <3>4. QED
      `Scope(n'')` の各名前は帰納法の仮定より `Scope(n)` の要素であるか `Ins(n)` の中で挿入され、
      `<3>2` で加わる名前は `<3>3` より `Ins(n)` の中で挿入される。
      BY <3>1, <3>2, <3>3
  <2>3. QED
    `<2>1` を基底、`<2>2` を帰納段とする、`n` からの深さについての帰納法。
    BY <2>1, <2>2

<1>25. `x` が `vars.bindings` の定義域の要素であり、`origin_inner(vars, E, x, pi)` がある `pi` に
   ついて `origin(vars, E, y, pi')` を呼ぶとき、`ord(y) < ord(x)` である。
  <2>1. `m(x)` を、`<1>21` の列で `x` を挿入する節点とする。`<1>22` より、`x` が `Binding::Param` で
     挿入されるときを除き、`m(x)` は次のどれかである。`x` を束縛する `Let` の節点、`x` をフィールド
     変数とする `Destructure` の節点、`x` を payload とするアームを持つ `Match` を右辺とする `Let` の
     節点。
    BY <1>21, <1>22
  <2>2. `origin_inner` は `vars.bindings.get(var)` で場合分けし、`None`、`Some(Binding::Param)`、
     `Some(Binding::Producer)` の腕では再帰呼び出しを行わない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. CASE `vars.bindings[x]` が `Binding::Move(y)` である。
    <3>1. `origin_inner` のこの腕は `origin(vars, type_env, &y.name, path)` を呼ぶ。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Move(y)` は `collect_bindings` が `Let(x, RcRhs::Var(y), k)` の腕で作る。よって
       `m(x)` はその `Let` の節点であり、`y` はその `rhs` が使う変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `y.name` は `Scope(m(x))` の要素であるか、`vars.bindings` の定義域に
       無い。
      BY <1>2, <3>2
    <3>4. CASE `y.name` が `vars.bindings` の定義域に無い。このとき `ord(y.name) = -1` であり、
       `ord(x)` は 0 以上なので `ord(y.name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `y.name` が `Scope(m(x))` の要素である。`<1>23` より `y.name` は `Ins(m(x))` が
       始まるより前に挿入されている。`<1>22` より `x` は `Ins(m(x))` の中で挿入される。よって
       `ord(y.name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>4. CASE `vars.bindings[x]` が `Binding::Llvm(llvm_gen, args, result_ty)` である。
    <3>1. `origin_inner` のこの腕が行う再帰呼び出しは 2 種類である。`decl.leaf_origins_at(path)` が
       単一の `Arg(j, p)` のときの `origin(vars, type_env, &args[j].name, &p)` と、そうでないときに
       呼ぶ `origin_from_leaves_under(vars, type_env, &decl, args, path, &here_identity)` の中の
       `origin(vars, type_env, &args[j].name, &unit)` である。どちらの呼び先も `args[j].name` で
       ある。
      BY CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>2. `Binding::Llvm(llvm_gen, args, x.ty)` は `collect_bindings` が
       `Let(x, RcRhs::Llvm(llvm_gen, args), k)` の腕で作る。よって `m(x)` はその `Let` の節点で
       あり、`args` の各要素はその `rhs` が使う変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `args[j].name` は `Scope(m(x))` の要素であるか、`vars.bindings` の定義域に
       無い。
      BY <1>2, <3>2
    <3>4. CASE `args[j].name` が `vars.bindings` の定義域に無い。`ord(args[j].name) = -1` であり
       `ord(x)` は 0 以上なので `ord(args[j].name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `args[j].name` が `Scope(m(x))` の要素である。`<1>23` より `Ins(m(x))` の開始より前に
       挿入されており、`<1>22` より `x` は `Ins(m(x))` の中で挿入される。よって
       `ord(args[j].name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>5. CASE `vars.bindings[x]` が `Binding::Field(container, idx)` である。
    <3>1. `origin_inner` のこの腕は、`container.ty.is_box(type_env)` が偽のときだけ
       `origin(vars, type_env, &container.name, &container_path)` を呼ぶ。真のときは `here()` を
       返して再帰しない。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Field(container, idx)` は `collect_bindings` が
       `Destructure(container, fields, _state, k)` の腕で作る。よって `m(x)` はその `Destructure` の
       節点であり、`container` はその節点が使う変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `container.name` は `Scope(m(x))` の要素であるか、`vars.bindings` の
       定義域に無い。
      BY <1>2, <3>2
    <3>4. CASE `container.name` が `vars.bindings` の定義域に無い。`ord(container.name) = -1` であり
       `ord(x)` は 0 以上なので `ord(container.name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `container.name` が `Scope(m(x))` の要素である。`<1>23` より `Ins(m(x))` の開始より前
       に挿入されており、`<1>22` より `x` は `Ins(m(x))` の中で挿入される。よって
       `ord(container.name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>6. CASE `vars.bindings[x]` が `Binding::Payload(scrut, variant)` である。
    <3>1. `origin_inner` のこの腕は、`variant` が `None` のとき
       `origin(vars, type_env, &scrut.name, path)` を、`Some(tag)` かつ `!scrut.ty.is_box(type_env)`
       のとき `origin(vars, type_env, &scrut.name, &scrut_path)` を呼ぶ。残りの `Some(_)` では
       `here()` を返して再帰しない。呼び先はどちらも `scrut.name` である。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Payload(scrut, arm.tag)` は `collect_bindings` が
       `Let(z, RcRhs::Match(scrut, arms), k)` の腕で、`arms` の各 `arm` について作る (`x` は
       `arm.payload.name`)。よって `m(x)` はその `Let` の節点であり、`scrut` はその `rhs` が使う
       変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `scrut.name` は `Scope(m(x))` の要素であるか、`vars.bindings` の定義域に
       無い。
      BY <1>2, <3>2
    <3>4. CASE `scrut.name` が `vars.bindings` の定義域に無い。`ord(scrut.name) = -1` であり
       `ord(x)` は 0 以上なので `ord(scrut.name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `scrut.name` が `Scope(m(x))` の要素である。`<1>23` より `Ins(m(x))` の開始より前に
       挿入されており、`<1>22` より `x`、すなわち `arm.payload.name` は `Ins(m(x))` の中で挿入
       される。よって `ord(scrut.name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>7. CASE `vars.bindings[x]` が `Binding::Join(arm_results)` である。
    <3>1. `origin_inner` のこの腕は `arm_results` の各要素 `arm_result` について
       `origin(vars, type_env, &arm_result.name, path)` を呼ぶ。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Join(arm_results)` は `collect_bindings` が
       `Let(x, RcRhs::Match(scrut, arms), k)` の腕で作り、`arm_results` の各要素は
       `returned_var(&arm.body)` である。よって `m(x)` はその `Let` の節点である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `returned_var(&arm.body)` は `arm.body` の中の `RcExpr::Ret(v)` の `v` である。すなわち
       `arm.body` の部分木にある `Ret` の節点が使う変数である。その節点を `rt` と書く。
      BY CODE src/rc_ir/ownership.rs: returned_var
    <3>4. `<1>2` (H2) より `arm_result.name` は `Scope(rt)` の要素であるか、`vars.bindings` の
       定義域に無い。
      BY <1>2, <3>3
    <3>5. CASE `arm_result.name` が `vars.bindings` の定義域に無い。`ord(arm_result.name) = -1` で
       あり `ord(x)` は 0 以上なので `ord(arm_result.name) < ord(x)` である。
      BY <1>21
    <3>6. CASE `arm_result.name` が `Scope(rt)` の要素である。
      <4>1. `rt` は `arm.body` の部分木に属するので、`<1>24` より `arm_result.name` は
         `Scope(arm.body)` の要素であるか、`Ins(arm.body)` の中で挿入される。
        BY <1>24, <3>3
      <4>2. `<1>22` より `x` の挿入は、すべてのアームの `arm.payload` の挿入と `Ins(arm.body)` が
         済んだ後に行われる。
        BY <1>22
      <4>3. CASE `arm_result.name` が `Scope(arm.body)` の要素である。`<1>23` より `Ins(arm.body)` の
         開始より前に挿入されている。`<4>2` より `x` の挿入は `Ins(arm.body)` が済んだ後なので、
         `ord(arm_result.name) < ord(x)` である。
        BY <1>23, <4>2
      <4>4. CASE `arm_result.name` が `Ins(arm.body)` の中で挿入される。`<4>2` より `x` の挿入は
         `Ins(arm.body)` が済んだ後なので、`ord(arm_result.name) < ord(x)` である。
        BY <4>2
      <4>5. QED
        BY <4>1, <4>3, <4>4
    <3>7. QED
      BY <3>1, <3>4, <3>5, <3>6
  <2>8. QED
    `Binding` の値は `Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join` の 7 つであり、
    `vars.bindings.get(var)` の結果は `None` を加えた 8 通りである。`<2>2` が `None`、`Param`、
    `Producer` を、`<2>3` から `<2>7` が残る 5 つを尽くしている。
    BY <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, CODE src/rc_ir/ownership.rs: Binding

<1>26. `origin_inner(vars, E, x, pi)` の 1 回の実行が行う `origin` の呼び出しは有限個である。
  <2>1. `None`、`Param`、`Producer` の腕は 0 個である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `Move(y)` の腕は 1 個、`Field` の腕は高々 1 個、`Payload` の腕は高々 1 個である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `Join(arm_results)` の腕は `arm_results` の長さだけ呼ぶ。`arm_results` は `collect_bindings`
     が `arms` の各要素について 1 つ積んだ有限の `Vec` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>4. `Llvm` の腕で `decl.leaf_origins_at(path)` が単一の `Arg(j, p)` のときは 1 個である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>5. `Llvm` の腕で `origin_from_leaves_under` を呼ぶときは、`operand_units` の要素数だけ呼ぶ。
     `operand_units` は `decl.leaf_origins_under(path)` が渡す各 `LeafOrigins` の各要素から作る
     `Set` である。`Provenance` は `LeafMap`、すなわち有限の `Map<FieldPath, LeafOrigins>` を包む値で
     あり、`LeafOrigins` は有限の `Set<LeafOrigin>` である。よって `operand_units` は有限である。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/provenance.rs: Provenance,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>27. `Origin` 型の任意の値 `o` について、`o.candidates()` は空でない列を返す。
  <2>1. `Origin::Exactly(p)` について `candidates()` は `vec![p]` を返す。これは長さ 1 である。
    BY CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>2. `Origin::Join { candidates, .. }` について `candidates()` は `candidates` の要素を集めた列を
     返す。
    BY CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>3. `Origin::Join` を作る場所は `Origin::of_candidates` の `_ => Origin::Join { .. }` の腕だけで
     ある。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, CODE src/rc_ir/ownership.rs: Origin
  <2>4. `of_candidates` はまず `assert!(!candidates.is_empty(), ...)` を置き、そののち
     `candidates.len()` で場合分けし、1 のとき `Origin::Exactly`、それ以外のとき `Origin::Join` を
     作る。`assert!` を通った時点で長さは 1 以上なので、`Origin::Join` を作るとき `candidates` の
     長さは 2 以上である。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>5. QED
    `Origin` の値は `Exactly` と `Join` の 2 つで、`<2>1` が前者を、`<2>2` から `<2>4` が後者を
    尽くしている。
    BY <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/ownership.rs: Origin

<1>27a. `Origin` 型の任意の値 `o` について、`o.acted_on()` は空でない列を返す。
  `acted_on` は `let mut out = vec![self.identity()];` で始め、そののち `out` を伸ばすだけなので、
  返り値は `o.identity()` を必ず含む。
  BY CODE src/rc_ir/ownership.rs: Origin::acted_on

<1>28. `origin_inner` の `Llvm` の腕が呼ぶ `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` は
   abort せずに `Provenance` を返す。
  <2>1. A3 は「`result_prov` は結果の leaf ごとに `LeafOrigin` の集合を宣言する」と述べる。宣言が
     存在するとは、この呼び出しが `Provenance` を返すことである。
    BY A3
  <2>2. 既定の実装は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` であり、これは
     `LeafMap::uniform` を経て `boxed_leaf_paths(result_ty, type_env)` を呼ぶ。`<1>1` (i) と `<1>9`
     よりこれは abort せず停止する。
    BY <1>1, <1>9, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
       CODE src/rc_ir/provenance.rs: Provenance::uniform, CODE src/rc_ir/leaf_map.rs: LeafMap::uniform
  <2>3. QED
    BY <2>1, <2>2

<1>29. `origin(vars, E, x, pi)` の**呼び出しの木** -- 根をその呼び出しとし、各節点の子をその実行が
   行う `origin` の呼び出しとする木 -- は有限である。したがって `origin` は停止する。
  <2>1. `origin` の本体は、`vars.origins` の中に `(x, pi)` の答えがあればそれを複製して返し、無ければ
     `origin_inner` を `grow_stack` の中で呼び、その答えを `vars.origins` に記録して返す。
     `grow_stack` は `stacker::maybe_grow` に与えた閉包をそのまま実行する。
    BY CODE src/rc_ir/ownership.rs: origin, CODE src/misc.rs: grow_stack
  <2>2. `origin` の呼び出しの木を考える。根は最初の呼び出しであり、節点 `origin(_, _, y, _)` の子は
     その実行が行う `origin` の呼び出しである。memo が当たった呼び出しは葉である。
    BY <2>1
  <2>3. `<1>25` より、この木の各辺 `origin(_, _, x, _)` から `origin(_, _, y, _)` について、`y` が
     `vars.bindings` の定義域に無いならその子は葉であり (`origin_inner` の `None` の腕は再帰しない)、
     定義域にあるなら `ord(y) < ord(x)` である。
    BY <1>25, CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. `<2>3` より、根から葉へ向かう任意の道の上で `ord` は狭義に減少する。`ord` の値は `-1` と
     `<1>21` の有限列の位置に限られるので、道の長さは `vars.bindings` の要素数に 2 を足した値で
     上から抑えられる。
    BY <1>21, <2>3
  <2>5. `<1>26` より木の各節点の子は有限個であり、`<2>4` より木の深さは有限である。有限分岐で深さが
     有限の木は有限なので、`origin` の呼び出しは有限回で終わる。
    BY <1>26, <2>4
  <2>6. `origin` の 1 回の実行が行う、`origin` 以外の計算は有限時間で終わる。`vars.origins` の探索と
     挿入、`Origin` の複製、`result_prov` の呼び出し (`<1>28`)、`origin_inner` の中の `unit_step` と
     `truncate_to_unit` の呼び出し (`<1>9` より停止する)、`Provenance` の有限の `Map` と `Set` の
     走査 (`<1>26`)、`args` の添字付けがその全部である。
    BY <1>9, <1>26, <1>28, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>7. QED
    BY <2>5, <2>6

<1>30. `origin(vars, E, x, pi)` は abort しない。
  <2>1. `origin` 自身の abort の可能性は `vars.origins` の `RefCell` の借用の衝突だけである。
    <3>1. `origin` は `vars.origins.borrow()` を
       `if let Some(known) = ... { return known.clone(); }` の走査対象で 1 回、
       `vars.origins.borrow_mut()` を最後の文で 1 回使う。
      BY CODE src/rc_ir/ownership.rs: origin
    <3>2. `if let` の走査対象が作る一時値は、その `if let` 文の終わりで落ちる (`Cargo.toml` の
       `edition = "2021"`)。`origin_inner` を呼ぶのはその次の文なので、その `Ref` は既に落ちて
       いる。
      BY <3>1
    <3>3. `borrow_mut()` が作る一時値はその文の終わりで落ちる。その文の中で `origin` は呼ばれない
       (`answer.clone()` は既に得た `Origin` の複製である)。
      BY <3>1
    <3>4. QED
      `<3>2` と `<3>3` より、`Ref` が生きている間に `borrow_mut()` は起きず、`RefMut` が生きている
      間に `borrow()` も `borrow_mut()` も起きない。
      BY <3>2, <3>3
  <2>2. `origin_inner` の `None`、`Param`、`Producer` の腕は `Origin::Exactly` を作るだけで abort
     しない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `origin_inner` の `Move` の腕は `origin` を呼ぶだけで abort しない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. `origin_inner` の `Field` の腕は `container.ty.is_box(type_env)` を呼ぶ。`is_box` は
     `is_unbox`、すなわち `is_closure() || toplevel_tycon_info(type_env).is_unbox` を呼ぶ。
     `is_closure()` が真のときは短絡して `toplevel_tycon_info` を呼ばないので、その
     `assert!(!self.is_closure())` は発火しない。偽のときは `<1>1` (i) より
     `toplevel_tycon().unwrap()` と `tycons().get(&tycon).unwrap()` が成功する。その他にこの腕が
     呼ぶのは `origin` と `Vec` の連結だけである。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner, CODE src/ast/types.rs: TypeNode::is_box,
       CODE src/ast/types.rs: TypeNode::is_unbox,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info
  <2>5. `origin_inner` の `Payload` の腕は `scrut.ty.is_box(type_env)` を呼ぶ。`is_closure()` が真の
     ときは短絡して `toplevel_tycon_info` を呼ばないので、その `assert!(!self.is_closure())` は発火
     しない。偽のときは `<1>1` (i) より `toplevel_tycon().unwrap()` と
     `tycons().get(&tycon).unwrap()` が成功する。その他にこの腕が呼ぶのは `origin` と `Vec` の連結
     だけである。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner, CODE src/ast/types.rs: TypeNode::is_box,
       CODE src/ast/types.rs: TypeNode::is_unbox,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info
  <2>6. `origin_inner` の `Join(arm_results)` の腕は `Origin::of_candidates(candidates, ...)` を
     呼ぶ。`of_candidates` は `candidates` が空のとき `assert!` で abort するが、この `candidates` は
     空でない。
    <3>1. `arm_results` は `collect_bindings` が `Let(x, RcRhs::Match(scrut, arms), k)` の腕で
       `arms` の各要素について 1 つずつ積んだものなので、その長さは `arms` の長さに等しい。
      BY CODE src/rc_ir/ownership.rs: collect_bindings
    <3>2. `<1>3` より `arms` は空でない。`<3>1` より `arm_results` も空でない。
      BY <1>3, <3>1
    <3>3. QED
      この腕は `arm_results` の各要素について `origin(...).acted_on()` の全要素を `candidates` に
      入れる。`<3>2` より要素は 1 つ以上あり、`<1>27a` よりその `acted_on()` は空でないので、
      `candidates` は空でない。
      BY <1>27a, <3>2, CODE src/rc_ir/ownership.rs: origin_inner
  <2>7. `origin_inner` の `Llvm` の腕は abort しない。
    <3>1. この腕はまず `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を呼ぶ。`<1>28` より
       これは abort しない。そののち `decl.leaf_origins_at(path)` (`LeafMap::get` を経由し `Option`
       を返す) に `as_arg_projection` を `and_then` する。`as_arg_projection` は `Set` の大きさを見て
       場合分けするだけで abort しない。ここで `path` は `origin` に渡された `pi` そのものであり、
       `leaf_origins_at` はそれを `Map` の鍵として引くだけなので、`pi` が `decl` の leaf でなくても
       `None` が返るだけである。
      BY <1>28, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: as_arg_projection,
         CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
         CODE src/rc_ir/leaf_map.rs: LeafMap::get
    <3>2. `Some((j, p))` の腕は `args[j]` で添字付けする。A3 の表は、`result_prov` が単一の
       `Arg(j, sig)` を宣言するとき、生成コードが第 `j` オペランドの leaf `sig` を結果に置く、と
       述べる。第 `j` オペランドが存在しなければこの宣言は意味を持たないので、`j` は `args.len()`
       未満であり、添字付けは範囲内である。
      BY A3, CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. `None` の腕は
       `origin_from_leaves_under(vars, type_env, &decl, args, path, &here_identity)` を呼び、その
       結果が `None` なら `here()` を返す。`unwrap_or_else` は abort しない。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>4. `origin_from_leaves_under` は `decl.leaf_origins_under(path)` が渡す `LeafOrigin::Arg(j,
       leaf)` の各出現について `args[*j]` で添字付けし、`truncate_to_unit(&args[*j].ty, leaf,
       type_env)` を呼ぶ。`<3>2` と同じ理由で `j` は `args.len()` 未満である。また A3 の同じ行より
       `leaf` は第 `j` オペランドの leaf、すなわち `L(args[j].ty)` の要素である。`origin` に渡された
       `pi` は `leaf_origins_under` の絞り込みに使われるだけで、`truncate_to_unit` には渡らない。
      BY A3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
    <3>5. `<1>17` (iii) と `<3>4` より `truncate_to_unit(&args[j].ty, leaf, E)` は abort しない。
       とくに `UnitStep::NoUnit` の腕の `panic!` (「holds no reference」) には達しない。
      BY <1>17, <3>4
    <3>6. `origin_from_leaves_under` の残りは、`Set` への挿入、`origin` の呼び出し、
       `reached.first()?` (空なら `None` を返す)、`Origin` の等価比較、そして `reached` が空でない
       ときの `Origin::of_candidates(candidates, here)` である。`candidates` は各 `reached` の
       `acted_on()` を集めたものであり、`reached` が空でなく、`<1>27a` より各 `Origin` の
       `acted_on()` が空でないので、`candidates` は空でなく `assert!` は発火しない。
      BY <1>27a, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates
    <3>7. QED
      BY <3>1, <3>2, <3>3, <3>4, <3>5, <3>6
  <2>8. `origin_inner` が呼ぶ型の上の walk (`truncate_to_unit` と、その中の `unit_step`、さらにその
     中の `is_fully_unboxed` と `unpunched_field_types`) は、`<1>1` (i) と `<1>9` より停止する。
    BY <1>1, <1>9
  <2>9. QED
    `origin` の実行が触れるのは `origin` 自身 (`<2>1`)、`origin_inner` の 8 通りの腕 (`<2>2` から
    `<2>7`)、および型の上の walk (`<2>8`) だけである。`Binding` の 7 つの値と `None` の 8 通りは
    `<2>2` から `<2>7` が尽くしている。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, <2>8, CODE src/rc_ir/ownership.rs: Binding

<1>31. **P2 が成り立ち、しかも README が置く定義域は外せる。** すなわち `<1>1`、`<1>2`、`<1>3a` を
   満たすプログラムについて、`vars` をその関数の `VarTable`、`x` を任意の `FullName`、`pi` を任意の
   `FieldPath` とすると、`origin(vars, E, x, pi)` は panic せずに `Origin` の値を返し、停止する。
   README の P2 は、この主張を「`x` がプログラムの束縛変数であり、`pi` が `L(ty(x))` の要素または
   `U(ty(x))` の要素である」場合に制限したものである。
  <2>1. `<1>29` (停止性) と `<1>30` (abort しないこと) の主張も証明も、`pi` に条件を置いていない。
    BY <1>29, <1>30
  <2>2. `pi` が型の上の walk に渡らないことは `<1>30` が場合ごとに述べている。`origin` と
     `origin_inner` は `pi` を、`Origin::Exactly` と `Origin::of_candidates` の成分として複製するか、
     前に添字を継ぎ足して再帰へ渡すか、`decl.leaf_origins_at` の鍵および `decl.leaf_origins_under` の
     絞り込みに使うかのいずれかにしかしない。`truncate_to_unit` に渡るのは宣言が名指す leaf だけで
     ある。
    BY <1>30, CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>3. QED
    BY <1>29, <1>30, <2>1, <2>2

<1>32. README の P2 が置く定義域は `origin` の再帰について閉じていない。すなわち、その定義域の中の
   `(x, pi)` から始めた `origin` の計算が、`sig` が `L(ty(u))` の要素でも `U(ty(u))` の要素でも
   ない呼び出し `origin(vars, E, u, sig)` を行うことがある。
  <2>1. 次の 2 つの型を取る。
     `Choice = unbox union { l : Array I64, r : I64 }`、
     `Outer  = unbox union { c : Choice, n : I64 }`。
     どちらも `TyConVariant::Union` で `is_unbox` が真である。
    BY CODE src/ast/typedecl.rs: TypeDefn::tycon_info,
       CODE src/parse/parser.rs: parse_union_defn (`is_unbox` の既定値は真)
  <2>2. `cls(Choice)` は `UN` である。`is_box` は偽 (unbox)、`is_closure` は偽 (union)、`is_array` は
     偽 (union)、`is_funptr` は偽 (`#FunPtr{n}` の variant は `Primitive`)、`F(Choice)` は
     `[(0, Array I64), (1, I64)]` で `Array I64` は `is_array` により `is_fully_unboxed` が偽なので、
     `is_fully_unboxed(Choice)` は偽である。`is_union` が真なので `DEF cls` より `UN` である。
    BY DEF cls, DEF F, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/fixstd/builtin.rs: bulitin_tycons
  <2>3. `U(Choice)` は `{[]}` である。`<1>13` で `p = []` を取ると ST-道であり
     `cls(end(Choice, [])) = UN` なので `[]` は要素である。`cls(Choice)` は `ST` でないので長さ 1 以上
     の ST-道は無い。
    BY <1>13, <2>2, DEF ST-道
  <2>4. `cls(Outer)` は `UN` である。`is_box` は偽 (unbox)、`is_closure` は偽 (union)、`is_array` は
     偽 (union)、`is_funptr` は偽 (`#FunPtr{n}` の variant は `Primitive`) であり、`F(Outer)` は
     `[(0, Choice), (1, I64)]` で `<2>2` より `is_fully_unboxed(Choice)` が偽なので
     `is_fully_unboxed(Outer)` は偽である。`is_union` が真なので `DEF cls` より `UN` である。
    BY DEF cls, DEF F, <2>2, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/fixstd/builtin.rs: bulitin_tycons
  <2>5. `L(Outer)` は `{[0, 0]}` であり、`U(Outer)` は `{[]}` である。
    <3>1. `<1>11` の `UN` の行より `go(Outer, [], out)` は `F(Outer)` の各要素へ降りる。`go(I64, [1])`
       は `cls(I64) = NB` なので何も積まない。`go(Choice, [0])` は `cls(Choice) = UN` なので
       `F(Choice)` の各要素へ降り、`go(Array I64, [0, 0])` は `cls(Array I64) = AR` なので `[0, 0]`
       を積み、`go(I64, [0, 1])` は何も積まない。
      BY <1>11, <2>2, <2>4, DEF cls
    <3>2. `U(Outer)` は `{[]}` である。`[]` は `Outer` の長さ 0 の ST-道であり
       `cls(end(Outer, [])) = cls(Outer) = UN` なので、`<1>13` の第 1 の集合の要素である。
       `cls(Outer)` は `ST` でないので長さ 1 以上の ST-道は無く、他に要素は無い。
      BY <1>13, <2>4, DEF ST-道
    <3>3. QED
      BY <3>1, <3>2
  <2>6. 次の本体を取る。`s : Outer` をパラメータとし、
     `let m = match s { c(p) -> Ret p_res ; n(q) -> Ret q_res }` とする。第 1 のアームは
     `tag = Some(0)`、`payload = p` で、`ty(p) = Choice` である。`collect_bindings` はこのアームに
     ついて `p` を `Binding::Payload(s, Some(0))` で束縛する。
    BY <1>3a, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>7. `origin(vars, E, p, [])` は README の P2 の定義域の中にある。`p` は束縛変数であり、`[]` は
     `<2>3` より `U(ty(p)) = U(Choice)` の要素である。
    BY <2>3, <2>6
  <2>8. `origin_inner(vars, E, p, [])` は `origin(vars, E, s, [0])` を呼ぶ。`Binding::Payload(s,
     Some(0))` の腕で、`s.ty = Outer` は `is_box` が偽なので `scrut_path` は `[0]` に `[]` を継いだ
     `[0]` になる。
    BY <2>4, <2>6, DEF cls, CODE src/rc_ir/ownership.rs: origin_inner
  <2>9. QED
    `<2>5` より `L(Outer)` は `{[0, 0]}`、`U(Outer)` は `{[]}` であり、`[0]` はそのどちらの要素でも
    ない。よって `<2>7` の呼び出しから `<2>8` の呼び出しが出て、後者は定義域の外にある。
    BY <2>5, <2>7, <2>8

<1>32a. `u` が `U(t)` の要素であるとき、`u` は `t` の unit に届き `T(t, u) = u` である。
  <2>1. `<1>13` より `u` は次のどちらかの形である。`u = p` で `p` は `t` の ST-道、`cls(end(t, p))`
     は `BX`、`AR`、`UN` のどれか。または `u = p ++ [c]` で `p` は `t` の ST-道、
     `cls(end(t, p)) = CL`。`s_j` を `u[0..j]` の位置の型 (`j` は `|p|` 以下) と書く。
    BY <1>13
  <2>2. `j` が 0 以上 `|p|` 以下のすべてについて、`T(t, u)` のループの最初の `j` 周はすべて
     `UnitStep::Fields` の腕を通って完了し、その時点で `cur = s_j` かつ `out = u[0..j]` である。
    <3>1. `j = 0` のとき。0 周が完了した時点で `cur = t = s_0` かつ `out = [] = u[0..0]` である。
      BY CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>2. ASSUME: `j` は `|p|` 未満であり、最初の `j` 周がすべて `Fields` の腕を通って完了し、その
       時点で `cur = s_j` かつ `out = u[0..j]` である
       PROVE: 最初の `j+1` 周がすべて `Fields` の腕を通って完了し、その時点で `cur = s_{j+1}` かつ
       `out = u[0..j+1]` である
      <4>1. `j` は `|p|` 未満で `|p|` は `|u|` 以下なので、ループは第 `j` 周に入る。
        BY <2>1
      <4>2. `p` が `t` の ST-道なので `cls(s_j) = ST` であり、`<1>10` より `unit_step(s_j, E)` は
         `UnitStep::Fields { held_fields: F(s_j), .. }` である。
        BY <1>10, <2>1, DEF ST-道
      <4>3. 第 `j` 周の `idx` は `u[j] = p[j]` であり、`p` が `t` の ST-道であることから
         `(p[j], s_{j+1})` は `F(s_j)` の要素である。`<1>12` より
         `held_field_type(&F(s_j), p[j], "truncate_to_unit")` は abort せず `s_{j+1}` を返す。
        BY <1>12, <2>1, DEF ST-道, DEF fld
      <4>4. QED
         `Fields` の腕は `out.push(idx)` と `cur = held_field_type(...)` を行うので、`out` は
         `u[0..j+1]`、`cur` は `s_{j+1}` になる。
        BY <4>1, <4>2, <4>3, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>3. QED
      `<3>1` を基底、`<3>2` を帰納段とする `j` についての帰納法。
      BY <3>1, <3>2
  <2>3. CASE `u = p` で `cls(end(t, p))` が `BX`、`AR`、`UN` のどれか。`|u| = |p|` なのでループは第
     `|p|` 周に入らずに終わり、`<2>2` を `j = |p|` に適用すると `out = u[0..|p|] = u` である。abort
     する腕には達していない。`<2>1` より `u` は `U(t)` の要素である。
    BY <2>1, <2>2
  <2>4. CASE `u = p ++ [c]` で `cls(end(t, p)) = CL`。`|u| = |p| + 1` なのでループは第 `|p|` 周に
     入る。`<2>2` を `j = |p|` に適用すると `cur = s_{|p|} = end(t, p)`、`out = p`、
     `idx = u[|p|] = c` である。`<1>10` より `unit_step(cur, E)` は
     `UnitStep::Capture { capture_idx: c, .. }` なので `assert_eq!(idx, capture_idx, ...)` は通り、
     `out.push(c)` して `break` する。よって `T(t, u) = p ++ [c] = u` である。
    BY <1>10, <2>1, <2>2, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>5. QED
    `<2>1` の 2 つの形を `<2>3` と `<2>4` が尽くしている。どちらも `T(t, u) = u` を与え、`u` は
    `U(t)` の要素である。
    BY <2>1, <2>3, <2>4, DEF unit に届く

<1>33. **(P1 の系 1: `origin` が辿る path はどれも unit に届く)** `<1>1`、`<1>2`、`<1>3a` を満たす
   プログラムについて、`pi` が `L(ty(x))` の要素または `U(ty(x))` の要素であるとき、
   `origin(vars, E, x, pi)` の計算の中で起きるすべての呼び出し `origin(vars, E, u, sig)` について、
   `u` が `vars.var_tys` に型を持つならば `sig` は `ty(u)` の unit に届く (`DEF unit に届く`)。
  <2>1. `lam` が `L(t)` の要素であるとき、`lam` は `t` の unit に届く。`<1>17` (iii) が `T(t, lam)`
     の abort しないことを、`<1>18` がその値が `U(t)` の要素であることを与える。
    BY <1>17, <1>18, DEF unit に届く
  <2>2. `q` が `t` の unit に届き、`cls(t') = ST` かつ `(i, t)` が `F(t')` の要素であるとき、
     `[i] ++ q` は `t'` の unit に届く。
    <3>1. `T(t', [i] ++ q)` のループの第 0 周は、`<1>10` より
       `unit_step(t') = UnitStep::Fields { held_fields: F(t'), .. }` を見て `out.push(i)` を行い、
       `<1>12` より `held_field_type(&F(t'), i, "truncate_to_unit")` が abort せず `t` を返すので
       `cur = t` になる。
      BY <1>10, <1>12, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>2. ループの各周の振る舞いは `cur` と `idx` だけで決まり、`out` には後ろに継ぎ足すことしか
       しない。`<3>1` の後の状態は `cur = t`、残りの入力が `q`、`out = [i]` であり、`T(t, q)` の
       初期状態は `cur = t`、残りの入力が `q`、`out = []` である。よって `T(t', [i] ++ q)` は abort
       せず、その値は `[i] ++ T(t, q)` である。
      BY <3>1, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>3. `T(t, q)` は `U(t)` の要素なので、`<1>13` より `T(t, q) = r` (`r` は `t` の ST-道で
       `cls(end(t, r))` が `BX`、`AR`、`UN` のどれか) か `T(t, q) = r ++ [c]` (`r` は `t` の ST-道で
       `cls(end(t, r)) = CL`) である。
      BY <1>13, DEF unit に届く
    <3>4. `[i] ++ r` は `t'` の ST-道であり `end(t', [i] ++ r) = end(t, r)` である。`cls(t') = ST`
       であり `fld(t', i) = t` だからである。
      BY <3>3, DEF ST-道, DEF fld
    <3>5. QED
      `<3>3` と `<3>4` を `<1>13` に当てはめると `[i] ++ T(t, q)` は `U(t')` の要素である。`<3>2` と
      合わせて `[i] ++ q` は `t'` の unit に届く。
      BY <1>13, <3>2, <3>3, <3>4, DEF unit に届く
  <2>3. `cls(t')` が `BX`、`AR`、`UN` のどれかで `q'` が空でない path であるとき、`q'` は `t'` の
     unit に届き `T(t', q') = []` である。
    <3>1. `<1>10` より `unit_step(t', E) = UnitStep::Unit` であり、その腕は `out` に積まずに `break`
       する。`q'` は空でないのでループは第 0 周に入る。よって `T(t', q') = []` であり abort しない。
      BY <1>10, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>2. `[]` は `t'` の長さ 0 の ST-道であり `cls(end(t', [])) = cls(t')` は `BX`、`AR`、`UN` の
       どれかなので、`<1>13` の第 1 の集合より `[]` は `U(t')` の要素である。
      BY <1>13, DEF ST-道
    <3>3. QED
      BY <3>1, <3>2, DEF unit に届く
  <2>4. `sig` が `ty(u)` の unit に届くならば `cls(ty(u))` は `NB` でない。
    <3>1. `cls(ty(u)) = NB` とする。`<1>10` より `unit_step(ty(u), E) = UnitStep::NoUnit` である。
      BY <1>10
    <3>2. `sig` が空でなければ、ループは第 0 周に入り `NoUnit` の腕の `panic!` に達する。これは
       `sig` が unit に届くことに反する。
      BY <3>1, DEF unit に届く, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>3. `sig` が空なら `T(ty(u), sig) = []` である。`<1>13` より `U(ty(u))` の要素はどれも
       `ty(u)` の ST-道 `p` (末尾のクラスが `BX`、`AR`、`UN`) か `p ++ [c]` (末尾のクラスが `CL`) の
       形であるが、`cls(ty(u)) = NB` なので長さ 0 の ST-道の末尾のクラスは `NB` であり、`cls(ty(u))`
       は `ST` でないので長さ 1 以上の ST-道は無い。よって `U(ty(u))` は空集合であり、`[]` はその
       要素でない。これは `sig` が unit に届くことに反する。
      BY <1>13, <3>1, DEF ST-道, DEF unit に届く
    <3>4. QED
      BY <3>2, <3>3
  <2>5. 呼び出しの木の根 `(x, pi)` について主張が成り立つ。
    BY <1>32a, <2>1
  <2>6. ASSUME: 呼び出しの木の節点 `(u, sig)` について、`u` が `vars.var_tys` に型を持つならば `sig`
     は `ty(u)` の unit に届く
     PROVE: その節点の子 `(u', sig')` についても、`u'` が `vars.var_tys` に型を持つならば `sig'` は
     `ty(u')` の unit に届く
    <3>1. CASE `vars.bindings[u]` が `Binding::Move(y)` である。子は `(y, sig)` であり、`<1>3a` (i)
       と (vi) より `ty(y) = ty(u)` なので、帰納法の仮定がそのまま主張である。
      BY <1>3a, CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. CASE `vars.bindings[u]` が `Binding::Join(arm_results)` である。子は `(arm_result, sig)` で
       あり、`<1>3a` (ii) と (vi) より `ty(arm_result) = ty(u)` なので、帰納法の仮定がそのまま主張で
       ある。
      BY <1>3a, CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. CASE `vars.bindings[u]` が `Binding::Payload(scrut, None)` である。子は `(scrut, sig)` で
       あり、`<1>3a` (iii) と (vi) より `ty(u) = ty(scrut)` なので、帰納法の仮定がそのまま主張で
       ある。
      BY <1>3a, CODE src/rc_ir/ownership.rs: origin_inner
    <3>4. CASE `vars.bindings[u]` が `Binding::Payload(scrut, Some(tag))` で `scrut.ty.is_box(E)` が
       偽である。
      <4>1. 子は `(scrut, [tag] ++ sig)` である。
        BY CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. `<1>3a` (iii) と (vi) より `(tag, ty(u))` は `F(ty(scrut))` の要素であり、`<1>3a` (iv)
         より `ty(scrut)` は union の型である。
        BY <1>3a
      <4>3. `cls(ty(scrut))` は `NB` でない。`ty(scrut)` は union の型なので `is_closure()` は偽
         (union の型構成子の variant は `Union`、closure の型構成子の variant は `Arrow`)、
         `is_array()` は偽 (`Std::Array` の variant は `Array`)、`is_funptr()` は偽 (`#FunPtr{n}` の
         variant は `Primitive`) であり、この場合の仮定より `is_box(E)` も偽である。よって
         `is_fully_unboxed(ty(scrut))` の値は `F(ty(scrut))` の各要素の第 2 成分についての
         `is_fully_unboxed` の連言である。`<4>2` より `ty(u)` はその 1 つであり、帰納法の仮定と
         `<2>4` より `cls(ty(u))` は `NB` でない、すなわち `is_fully_unboxed(ty(u))` は偽なので、
         連言は偽である。
        BY <2>4, <4>2, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
           CODE src/ast/types.rs: TyConVariant, CODE src/fixstd/builtin.rs: bulitin_tycons
      <4>4. `cls(ty(scrut)) = UN` である。`<4>3` より `NB` でなく、`<4>3` の中で `is_closure`、
         `is_box`、`is_array` が偽であることを示したので `CL`、`BX`、`AR` でもない。`ty(scrut)` は
         union の型なので `is_union(E)` が真であり、`DEF cls` より `UN` である。
        BY <4>2, <4>3, DEF cls
      <4>5. QED
        `[tag] ++ sig` は空でないので、`<2>3` を `t' := ty(scrut)` に適用すると `[tag] ++ sig` は
        `ty(scrut)` の unit に届く。
        BY <2>3, <4>1, <4>4
    <3>5. CASE `vars.bindings[u]` が `Binding::Field(cont, idx)` で `cont.ty.is_box(E)` が偽で
       ある。
      <4>1. 子は `(cont, [idx] ++ sig)` である。
        BY CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. `<1>3a` (v) と (vi) より `(idx, ty(u))` は `F(ty(cont))` の要素であり、`ty(cont)` は
         構造体の型である。
        BY <1>3a
      <4>3. `cls(ty(cont))` は `NB` でない。`ty(cont)` は構造体の型なので `is_closure()` は偽
         (構造体の型構成子の variant は `Struct`)、`is_array()` は偽、`is_funptr()` は偽であり、この
         場合の仮定より `is_box(E)` も偽である。よって `is_fully_unboxed(ty(cont))` の値は
         `F(ty(cont))` の各要素の第 2 成分についての `is_fully_unboxed` の連言である。`<4>2` より
         `ty(u)` はその 1 つであり、帰納法の仮定と `<2>4` より `is_fully_unboxed(ty(u))` は偽なので、
         連言は偽である。
        BY <2>4, <4>2, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
           CODE src/ast/types.rs: TyConVariant, CODE src/fixstd/builtin.rs: bulitin_tycons
      <4>4. `cls(ty(cont))` は `UN` か `ST` である。`<4>3` より `NB`、`CL`、`BX`、`AR` のどれでも
         ない。
        BY <4>3, DEF cls
      <4>5. CASE `cls(ty(cont)) = ST`。`<2>2` を `t' := ty(cont)`、`t := ty(u)`、`i := idx`、
         `q := sig` に適用する。`<4>2` が `(idx, ty(u))` が `F(ty(cont))` の要素であることを、
         帰納法の仮定が `sig` が `ty(u)` の unit に届くことを与える。
        BY <2>2, <4>1, <4>2
      <4>6. CASE `cls(ty(cont)) = UN`。`[idx] ++ sig` は空でないので `<2>3` を `t' := ty(cont)` に
         適用する。
        BY <2>3, <4>1
      <4>7. QED
        BY <4>4, <4>5, <4>6
    <3>6. CASE `vars.bindings[u]` が `Binding::Llvm(llvm_gen, args, result_ty)` で、
       `decl.leaf_origins_at(sig)` が単一の `Arg(j, p)` である。子は `(args[j], p)` であり、A3 より
       `p` は第 `j` オペランドの leaf、すなわち `L(ty(args[j]))` の要素である。`<2>1` を適用する。
      BY A3, <1>3a, <2>1, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: as_arg_projection
    <3>7. CASE `vars.bindings[u]` が `Binding::Llvm(llvm_gen, args, result_ty)` で、
       `origin_from_leaves_under` が呼ばれる。子は `(args[j], unit)` であり、
       `unit = truncate_to_unit(&args[j].ty, leaf, E)` で `leaf` は宣言が名指す leaf である。A3 より
       `leaf` は `L(ty(args[j]))` の要素なので、`<1>18` より `unit` は `U(ty(args[j]))` の要素で
       ある。`<1>32a` を適用する。
      BY A3, <1>3a, <1>18, <1>32a, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>8. QED
      `origin_inner` が `origin` を呼ぶ場所は `<3>1` から `<3>7` の 7 つで尽きている。`None`、
      `Binding::Param`、`Binding::Producer` の腕、`Binding::Field` で容器が boxed の腕、
      `Binding::Payload` で `Some(_)` かつ scrutinee が boxed の腕は呼ばない。
      BY <3>1, <3>2, <3>3, <3>4, <3>5, <3>6, <3>7,
         CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding
  <2>7. QED
    `<2>5` を基底、`<2>6` を帰納段とする、呼び出しの木の根からの深さについての帰納法。`<1>29` より
    この木は有限なので、帰納法は木全体に届く。
    BY <1>29, <2>5, <2>6

<1>34. **(P1 の系 2: `origin` が返す path も unit に届く)** `<1>33` の条件の下で、
   `origin(vars, E, x, pi)` の返り値に現れる各 `VarPath` `(u, sig)` (identity と candidates の両方)
   について、`u` が `vars.var_tys` に型を持つならば `sig` は `ty(u)` の unit に届く。すなわち
   `T(ty(u), sig)` は abort せず、その値は `U(ty(u))` の要素である。
  <2>1. `origin(vars, E, u, sig)` の返り値に現れる各 `VarPath` は、`origin` の呼び出しの木の
     `(u, sig)` を根とする部分木のある節点そのものである。
    <3>1. `origin_inner` の `None`、`Binding::Param`、`Binding::Producer` の腕、`Binding::Field` で
       容器が boxed の腕、`Binding::Payload` で `Some(_)` かつ scrutinee が boxed の腕は
       `here() = Origin::Exactly((var.clone(), path.to_vec()))` を返す。現れる `VarPath` はこの節点
       そのものである。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Move` の腕、`Binding::Payload` で `None` の腕、`Binding::Payload` で `Some(tag)`
       かつ scrutinee が unbox の腕、`Binding::Field` で容器が unbox の腕、`Binding::Llvm` で
       `leaf_origins_at` が単一の `Arg` の腕は、子の返り値をそのまま返す。帰納法の仮定より、現れる
       `VarPath` は子の部分木の節点である。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. `Binding::Join` の腕は `Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))`
       を返す。`candidates` は各子の返り値の `acted_on()` の合併である。`acted_on()` が返すのは
       `identity()` と `candidates()` の元、すなわちその子の返り値に現れる `VarPath` だけなので、
       帰納法の仮定よりその各要素は子の部分木の節点である。`of_candidates` は要素数 1 のとき
       `Origin::Exactly(その要素)` を、それ以外のとき
       `Origin::Join { identity: (var, path), candidates }` を返す。前者に現れるのは子の部分木の
       節点、後者に現れるのは子の部分木の節点とこの節点自身である。
      BY CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates,
         CODE src/rc_ir/ownership.rs: Origin::acted_on,
         CODE src/rc_ir/ownership.rs: Origin::candidates
    <3>4. `Binding::Llvm` で `origin_from_leaves_under` を呼ぶ腕。`origin_from_leaves_under` は
       `reached` を、各子の返り値と、`produced_here` が真のときの `Origin::Exactly(here.clone())`
       (`here` はこの節点自身) から作る。`reached` が空なら `None` を返し、`origin_inner` は
       `here()` を返す (この節点自身)。`reached` の要素がすべて等しければその 1 つを返す。そうで
       なければ `Origin::of_candidates(candidates, here)` を返し、`candidates` は各 `reached` の
       `acted_on()` の合併である。`acted_on()` が返すのは `identity()` と `candidates()` の元、
       すなわちその `Origin` に現れる `VarPath` だけである。いずれの場合も現れる `VarPath` は、
       子の部分木の節点かこの節点自身である。
      BY CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates,
         CODE src/rc_ir/ownership.rs: Origin::acted_on,
         CODE src/rc_ir/ownership.rs: Origin::candidates
    <3>5. memo が当たった呼び出しは、その `(u, sig)` について前に計算した答えを複製して返す。その
       答えは同じ `(u, sig)` を根とする部分木から作られたものである。
      BY CODE src/rc_ir/ownership.rs: origin
    <3>6. QED
      `<3>1` から `<3>5` を帰納段とする、呼び出しの木の葉からの高さについての帰納法。`<1>29` より
      この木は有限である。
      BY <1>29, <3>1, <3>2, <3>3, <3>4, <3>5
  <2>2. `<1>33` と `<2>1` より、返り値に現れる各 `VarPath` `(u, sig)` について、`u` が
     `vars.var_tys` に型を持つならば `sig` は `ty(u)` の unit に届く。
    BY <1>33, <2>1
  <2>4. QED
    `<2>2` が主張そのものであり、`DEF unit に届く` が「unit に届く」を「`T(ty(u), sig)` は abort
    せず、その値は `U(ty(u))` の要素である」と展開する。
    BY <2>2, DEF unit に届く

<1>37. QED
  結論の 4 つは順に `<1>20` (P1)、`<1>31` (P2 と定義域)、`<1>32` (定義域が閉じていないこと)、
  `<1>33` と `<1>34` (P1 の系) である。
  BY <1>20, <1>31, <1>32, <1>33, <1>34

## 3. 入力についての 3 つの前提と README の仮定

`<1>1` (H1)、`<1>2` (H2)、`<1>3a` (H4) は、README の A10 (型の well-formedness)、A11 (スコープの
規律)、A12 (束縛の形と型が合っている) である。README の文面との差は 3 つある。

- `<1>1` は A10 に (iii) を足す。in-place の降下で到達する型も ground であり、その型構成子が
  `type_env` に登録されていること。`is_fully_unboxed` はその降下の上の再帰なので、途中の型に
  ついてもこれが要る。
- `<1>2` は A11 の「スコープに入っている束縛」を `DEF Scope` で節点の種類ごとに書き下し、さらに
  この関数のどの束縛でもない名前 (グローバル) を許す。`origin_inner` の `None` の腕がその名前を
  受ける (`CODE src/rc_ir/ownership.rs: origin_inner`)。
- `<1>3a` は A12 の項目のうちこの文書が使う 6 つを述べる。A12 の残り -- `App` の引数と呼び出し先の
  パラメータの型が合っていること、`Destructure` と `Match` が名指すフィールドと変位が punched で
  ないこと -- は、この文書のどのステップも読まない。

## 4. leaf と unit がずれる 2 か所が P1 に効いた場所

`unit_step` は `is_union` と `is_punched_array` で `UnitStep::Unit` を返して止まり、
`boxed_leaf_paths` の内部関数 `go` はその 2 つを問わずにフィールドへ降りる (`<1>10` と `<1>11` の表の
`UN` の行)。D5 が挙げる 2 か所は、この 1 つのクラス `UN` にまとまる。

- **unbox union**: `Choice = unbox union { l : Array I64, r : I64 }` の `rc_units` は `[[]]`、
  `boxed_leaf_paths` は `[[0]]` である。
- **punched array**: `PunchedArray a = unbox struct { _arr : Array a, _idx : I64 }` は
  `is_punched_array` が真なので `cls` は `UN` であり、`rc_units` は `[[]]`、`boxed_leaf_paths` は
  `[[0]]` である。

この差は P1 の 2 つの向きにそれぞれ別の形で効く。どちらの向きも `cls` が `UN` であることだけを使うので、
上の 2 か所を同時に扱う。

- **前半 (`<1>18`)**: leaf `lam` の道の途中に `UN` の型があると、`truncate_to_unit` はそこで
  `UnitStep::Unit` を見て `break` し、`lam` を最初の `UN` の位置で切る (`<1>17` の `k` の定義と
  (iii))。切った結果が `U(t)` の要素になるのは、`rc_units_go` がその同じ位置まで `ST` の型だけを降りて
  `UnitStep::Unit` を見て path を積むからである (`<1>13` の第 1 の集合)。すなわち、切り詰めが止まる
  位置と unit が置かれる位置が、どちらも「最初に `ST` でなくなる型」で一致している。
- **後半 (`<1>19`)**: `cls(s) = UN` の位置に置かれた unit `p` を証拠立てる leaf が要る。`go` は `UN`
  で降りるので、`p` の下の leaf は `p` より真に長い (`<1>19`)。真に長いことが効くのは、
  `truncate_to_unit` の `for` ループが位置 `|p|` の周に入ってはじめて `UnitStep::Unit` を見て
  `break` するからである (`<1>17` の (iii) が `cls(s_k)` に応じて場を分ける形である)。そして `p` の下に
  leaf が 1 つ以上あることは、`cls(s) = UN` なら `is_fully_unboxed(s)` が偽で `is_funptr(s)` も偽な
  ので `F(s)` のどれかのフィールドが `is_fully_unboxed` でない、という `<1>7` (b) と `<1>16` から
  出る。

つまり P1 は、`UN` の型が (a) `truncate_to_unit` で切り詰めが止まる位置であり、かつ (b) その下に必ず
leaf を持つ、という 2 つの事実の上に立っている。どちらか一方でも崩れると P1 は偽になる。

## 5. unit path の `origin` と、その下の leaf の `origin` の関係

`Retain`/`Release` の path が leaf でない unit (unbox union、punched array) のとき、その unit の
`origin` と、その下の各 leaf の `origin` について言えるのは次の 1 つである。

**`<1>33` と `<1>34`。** どちらの `origin` についても、その計算の中で起きる呼び出しの `(u, sig)` も、
返り値に現れる `VarPath` `(u, sig)` も、`sig` が `ty(u)` の unit に届く。すなわち
`truncate_to_unit(ty(u), sig)` は abort せず `rc_units(ty(u))` の要素になる。この性質は `origin` の
再帰の各辺 (move-bind、`Match` のアームの結果、変位アームの payload、catch-all の payload、
unbox 容器のフィールド、`Llvm` の 2 つの道) が保つ。

`origin` の答えに `truncate_to_unit` を当てるコードは `borrow.rs` の `owns_object` である。
`owns_unit` と `check_ownership_is_levelled` が `origin(v, unit).candidates()` の各 `(root, path)` を
`owns_object` に渡し、`owns_object` は `root` が `vars.param_tys` にあるとき `path` を `units_under`
と `truncate_to_unit` に掛ける。`param_tys` に入る名前は `var_tys` にも同じ型で入るので、`<1>34` が
その `path` について「`ty(root)` の unit に届く」を与える。`owns_object` を主語とする命題は P7e で
ある。
