# P8 - P14 -- `borrow_ify` と RC 規律の保存

対象コミット `b81cc2c8e859a00cbf007e4f43483a514c813c73`。定義・仮定・命題の番号は同ディレクトリの
`README.md` による。第 15 節以降は、`infer_ownership` に平準化の段 (`levelled_sites`、
`level_ownership`、`covered_leaves`) が入った後のコードを対象とする。

## 0. この文書の状態

**P8 から P13 までを証明した。P14 は依然として偽であり、その反例が第 18 節にある。**

第 3 節の反例 `R2` は平準化の段によって消えた (第 17 節)。第 18 節の反例 `R3` は、平準化の段が閉じて
いない形を突く。第 16 節が、平準化の段が H2 の前半を与え後半を与えないことを述べる。

証明の状態は次のとおりである。

| 命題 | 状態 | 局所の仮説を使ったか |
|---|---|---|
| P8 (推論の停止性と安全性) | 停止性と不動点の閉包は証明済み。D9 の消費との対応は P3/P4 に依る | **H1 を使う** |
| P9 (複製は名前替えである) | 前半・後半とも証明済み | H1 を使わない。後半が H3 を使う |
| P10 (借用版が落とす RC 節点) | 証明済み | 使わない |
| P11 (呼び出し側の補正) | 証明済み | 使わない |
| P12 (振り分けの安全性) | 証明済み | 使わない |
| P13 (注釈の一致) | 証明済み | 使わない |
| P14 (`borrow_ify` は RC 規律を保存する) | **偽** (第 18 節の反例 `R3`) | -- |
| H2 の前半 (`owns_unit` が真ならば leaf も所有) | 証明済み (第 16.2 節、P7a による) | -- |
| H2 の後半 (leaf が所有ならば `owns_unit` が真) | **偽** (第 18 節の反例 `R3`) | -- |
| P11 の「ちょうど埋める」 | H2 の下で証明済み (第 19 節) | H2 を使う |

局所の仮説 H1・H2・H3 は第 4 節が定める。H1 は依頼が置いた所有の一様性、H2 はそれを local な値の unit へ
広げたもの、H3 は入力の束縛名の形についてのものである。

**H1 は `R2` を排除しない** (第 11.1 節)。`R2` の 2 つのパラメータはどちらも leaf 1 つの unit 1 つしか
持たないので、H1 は `R2` の入力で成り立つ。よって H1 を成り立たせる直し方だけでは P14 は閉じない。P14 が
要求するのは H2 であり、H2 を成り立たせるには `origin` の候補の所有をそろえる必要がある (第 11.2 節)。

第 11 節が、H1 についての 3 つの問いへの答えである。第 13 節と第 21 節が `README.md` へ差し戻す点で
ある。第 15 節から第 22 節までが、平準化の段が入った後の状態である。

## 1. 記法

`origin(x, π)` は `origin(vars, type_env, &x, &π)` の略記とする。`vars` はその時点で問題にしている版の
`VarTable` である。`VarPath` を `(x, π)` と書く。

`leaves(τ)` は `boxed_leaf_paths(τ, type_env)`、`units(τ)` は `rc_units(τ, type_env)` とする
(`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`, `CODE src/rc_ir/ownership.rs: rc_units`)。

`Obl` は D10 の義務集合とする。`H(o)` は D7 の参照カウントとする。

`borrow_ify` の局所変数を次の名で参照する (`CODE src/rc_ir/borrow.rs: borrow_ify`)。

- `owned_leaves`: `infer_ownership` が返す `OwnedLeaves` の中身。leaf の集合である
  (`CODE src/rc_ir/borrow.rs: OwnedLeaves`)。
- `owned_units`: `borrow_ify` が組み立てる `Set<VarPath>`。unit の集合である。
- `borrow_versions`: 借用版を持つ関数からその版の名前への `Map`。

**DEF f_own** -- 入力の関数 `f` について、`borrow_ify` が `funcs` に `f.name` の名で入れる版をいう
(`CODE src/rc_ir/borrow.rs: borrow_ify` の `for func in prog.funcs.values()` の 4 番目のループ)。

**DEF f_borrow** -- `borrow_versions` が `f.name` に対応させる名の版をいう。`clone_func` が作る
(`CODE src/rc_ir/borrow.rs: borrow_ify` の `for (borrow_version, mut clone, _) in clones` のループ)。

**DEF rename_f** -- `borrow_versions` に載る入力の関数 `f` について、`clone_func(f, ..)` が返す束縛の
付け替え `Map<FullName, FullName>` をいう (`CODE src/rc_ir/borrow.rs: clone_func`)。

**DEF 入力の束縛名** -- `borrow_ify` の入力プログラムの、ある関数のパラメータ・capture の名前、または
ある関数の本体かあるグローバル初期化子の本体が束縛する変数の名前をいう。束縛する構文は `Let` の第 1 成分、
`Destructure` のフィールド変数、`MatchArm` の `payload` の 3 つである
(`CODE src/rc_ir/ast.rs: RcExpr`, `MatchArm`)。

**DEF 出力の束縛名** -- 同じものを `borrow_ify` の出力プログラムについて言う。

## 2. 補題

### L1 (A2 の path の下の unit はその path 自身だけである)

**言明**。`π` が `units(τ)` の要素であるとき、`units_under(τ, π, type_env)` は `[π]` である。

<1>1. `rc_units_go` が `out` に積む path は、`UnitStep::Fields` の腕で積んだ添字の列 `ρ` に、
      `UnitStep::Unit` の腕では何も足さず、`UnitStep::Capture` の腕では `capture_idx` を 1 つ足したもので
      ある。
  BY CODE src/rc_ir/ownership.rs: rc_units_go

<1>2. `π` が `units(τ)` の要素であるとき、`π` は `ρ` (`Unit` の腕で積まれた場合) か `ρ ++ [c]`
      (`Capture` の腕で積まれた場合) の形である。ここで `ρ` の各添字は、その位置の型の `unit_step` が
      `UnitStep::Fields` を返し、その `held_fields` が持つ添字である。
  BY <1>1, CODE src/rc_ir/ownership.rs: rc_units, rc_units_go

<1>3. `subtree_type(τ, ρ, type_env)` は、`ρ` の各添字について `UnitStep::Fields` の腕を通り、
      `held_field_type` でその添字の型へ降りて、`Some(σ)` を返す。`σ` は `ρ` が指す部分木の型である。
  BY <1>2, CODE src/rc_ir/ownership.rs: subtree_type, held_field_type

<1>4. CASE `π = ρ` (`Unit` の腕で積まれた場合)。
  <2>1. `unit_step(σ, type_env)` は `UnitStep::Unit` である。
    BY <1>2, <1>3, CODE src/rc_ir/ownership.rs: rc_units_go の `UnitStep::Unit` の腕
  <2>2. `rc_units(σ, type_env)` は `[[]]` である。
    BY <2>1, CODE src/rc_ir/ownership.rs: rc_units, rc_units_go の `UnitStep::Unit` の腕
  <2>3. `units_under(τ, π, type_env)` は `subtree_type` が `Some(σ)` を返す腕を通り、`rc_units(σ)` の各元を
        `π` の後ろに繋いだものを返す。`<2>2` よりそれは `[π ++ []] = [π]` である。
    BY <1>3, <2>2, CODE src/rc_ir/ownership.rs: units_under

<1>5. CASE `π = ρ ++ [c]` (`Capture` の腕で積まれた場合)。
  <2>1. `unit_step(σ, type_env)` は `UnitStep::Capture { capture_idx: c, .. }` である。
    BY <1>2, <1>3, CODE src/rc_ir/ownership.rs: rc_units_go の `UnitStep::Capture` の腕
  <2>2. `subtree_type(τ, π, type_env)` は、`ρ` を降りた後の添字 `c` で `UnitStep::Capture` の腕に入り、
        `None` を返す。
    BY <1>3, <2>1, CODE src/rc_ir/ownership.rs: subtree_type の
       `UnitStep::NoUnit | UnitStep::Capture { .. } | UnitStep::Unit` の腕
  <2>3. `units_under(τ, π, type_env)` は `None` の腕を通り、`vec![π]` を返す。
    BY <2>2, CODE src/rc_ir/ownership.rs: units_under

<1>6. QED
  `<1>2` の 2 つの形が場合を尽くしており、`<1>4` と `<1>5` がそれぞれを与える。
  BY <1>2, <1>4, <1>5

### L2 (`owned_units` に入るもの)

**言明**。`borrow_ify` が組み立てる `owned_units` は、次の 2 種の元だけからなる。

- (a) 入力の各関数 `f` の各パラメータ・capture `p` と各 `unit ∈ units(ty(p))` について `(p.name, unit)`。
- (b) `borrow_versions` に載る各関数 `f` の各パラメータ `p` と、`owned_leaves.owns(p.name, λ)` が真である
  各 `λ ∈ leaves(ty(p))` について `(rename[p.name], truncate_to_unit(ty(p), λ, type_env))`。ここで `rename` は
  `clone_func` が返す束縛の付け替えである。

<1>1. `owned_units` に元を入れるのは、`borrow_ify` の 2 番目のループの 2 か所だけである。1 つは
      `owned_units.extend(param_capture_units(func, type_env))`、もう 1 つは
      `owned_units.insert((rename[&p.name].clone(), unit))` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `param_capture_units(func, type_env)` は、`func.params` と `func.capture` の各元 `p` と各
      `unit ∈ units(ty(p))` について `(p.name, unit)` を返す。
  BY CODE src/rc_ir/borrow.rs: param_capture_units

<1>3. 1 つ目の `extend` は入力の各関数について走るので、それが入れるのは (a) である。
  BY <1>1, <1>2, CODE src/rc_ir/borrow.rs: borrow_ify

<1>4. 2 つ目の `insert` は `borrow_versions.get(&func.name)` が `Some` のときだけ走り、`func.params` の各元
      `p` と `boxed_leaf_paths(&p.ty, type_env)` の各元 `leaf` について、`owned_leaves.owns(&p.name, &leaf)` が
      真のときに `(rename[&p.name], truncate_to_unit(&p.ty, &leaf, type_env))` を入れる。
  BY <1>1, CODE src/rc_ir/borrow.rs: borrow_ify

<1>5. QED
  BY <1>3, <1>4

### L3 (借用版の `rewrite_rc` が A2 の path の節点に何をするか)

**言明**。`is_borrow_version` が真の `RewriteCtx` について、`Retain(v, π, s, k)` または `Release(v, π, s, k)` で
`π ∈ units(ty(v))` であるものは、`owns_unit(v, π)` が真ならば同じ種類・同じ変数・同じ path の節点 1 つに、
偽ならば節点無しに書き換えられる。

<1>1. `rewrite_inner` の `RcExpr::Retain` と `RcExpr::Release` の腕は、いずれも
      `self.rewrite_rc(v, path, *state, is_release, k, &node.source)` を呼ぶ。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

<1>2. `rewrite_rc` は `self.is_borrow_version` が真のとき、`units_under(&v.ty, path, self.type_env)` を
      `self.owns_unit(v, unit)` で絞った `kept` を作り、`kept` の各元について `rc_node` を 1 つ重ねる。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>3. `π ∈ units(ty(v))` なので `units_under(ty(v), π, type_env)` は `[π]` である。
  BY L1

<1>4. QED
  `<1>2` の `kept` は、`<1>3` より `owns_unit(v, π)` が真なら `[π]`、偽なら空である。`rc_node` は
  `is_release` の真偽でそれぞれ `RcExpr::Release` と `RcExpr::Retain` を作る。
  BY <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: rc_node

### L4 (`owns_object` の値)

**言明**。`owns_object(root, path)` は、`self.vars.param_tys` が `root` を持たないとき真であり、持つとき
(その型を `τ`) 、`units_under(τ, path, type_env)` の各 `unit` について
`(root, truncate_to_unit(τ, unit, type_env))` が `self.owned_units` に入ることと同値である。

<1>1. `owns_object` は `self.vars.param_tys.get(root)` で場合分けし、`None` の腕で `true` を返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>2. `Some(root_ty)` の腕は `units_under(root_ty, path, self.type_env)` の各 `unit` について
      `self.owned_units.contains(&(root.clone(), truncate_to_unit(root_ty, unit, self.type_env)))` を要求する。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>3. QED
  BY <1>1, <1>2

### L5 (`RewriteCtx::rewrite` は束縛を導入も除去もしない)

**言明**。任意の `RewriteCtx` と任意の本体 `B` について、`ctx.rewrite(B)` が束縛する変数名の集合は、`B` が
束縛する変数名の集合に等しい。

<1>1. `rewrite` は `grow_stack(|| self.rewrite_inner(node))` であり、`grow_stack(f)` は `f()` を返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite, CODE src/misc.rs: grow_stack

<1>2. `rc_node` が作る `RcExpr::Retain` と `RcExpr::Release` は変数を束縛しない。`prepend_rc` が作る節点は
      `rc_node` が作るものだけである。
  BY CODE src/rc_ir/borrow.rs: rc_node, prepend_rc, CODE src/rc_ir/ast.rs: RcExpr

<1>3. `B` の構造についての帰納法で示す。以下、`bind(B)` を `B` が束縛する変数名の集合とする。

  <2>1. CASE `B = Let(x, App(callee, args), k)`。
        `rewrite_inner` はこの腕で `prepend_rc(before, false, expr_node(RcExpr::Let(x.clone(),
        RcRhs::App(callee, args.clone()), prepend_rc(after, true, self.rewrite(k))), ..))` を返す。
        `x` はそのまま束縛子として残り、`<1>2` より `prepend_rc` の節点は何も束縛しない。帰納法の仮定より
        `bind(rewrite(k)) = bind(k)`。よって束縛する名前の集合は `x` と `bind(k)` の合併であり、`bind(B)` に
        等しい。
    BY <1>2, 帰納法の仮定, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
       CODE src/rc_ir/ast.rs: RcExpr

  <2>2. CASE `B = Let(x, Match(scrut, arms), k)`。
        `rewrite_inner` は各 `arm` を `arm.with_body(self.rewrite(&arm.body))` に替える。`with_body` は
        `body` 以外のフィールド (`tag`、`payload`、`payload_state`) をそのまま写すので、各アームの
        `payload` は変わらない。帰納法の仮定より各アーム本体と `k` の束縛は変わらない。
    BY 帰納法の仮定, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
       CODE src/rc_ir/ast.rs: MatchArm::with_body

  <2>3. CASE `B = Let(x, rhs, k)` で `rhs` が `App` でも `Match` でもない場合。
        `rewrite_inner` は `RcExpr::Let(x.clone(), rhs.clone(), self.rewrite(k))` を返す。`RcRhs::Var`、
        `RcRhs::Closure`、`RcRhs::Llvm` は変数を束縛しない。帰納法の仮定より `k` の束縛は変わらない。
    BY 帰納法の仮定, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/ast.rs: RcRhs

  <2>4. CASE `B = Retain(v, π, s, k)` または `B = Release(v, π, s, k)`。
        `rewrite_inner` は `rewrite_rc` を呼び、`rewrite_rc` は `self.rewrite(k)` の上に `rc_node` を
        重ねるか、`rc_node` を 1 つ置くかのどちらかである。`<1>2` よりどちらも束縛しない。帰納法の仮定より
        `k` の束縛は変わらない。
    BY <1>2, 帰納法の仮定, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::rewrite_rc

  <2>5. CASE `B = Destructure(container, fields, s, k)`。
        `rewrite_inner` は `fields.clone()` をそのまま写す。帰納法の仮定より `k` の束縛は変わらない。
    BY 帰納法の仮定, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

  <2>6. CASE `B = Eval(v, k)` または `B = Ret(v)`。
        `rewrite_inner` はどちらも束縛子を持たない節点を作る。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/ast.rs: RcExpr

  <2>7. QED
    D2 より `RcExpr` は `Let`、`Retain`、`Release`、`Destructure`、`Eval`、`Ret` の 6 種であり、`Let` を
    `App`・`Match`・それ以外の 3 つに分けた `<2>1`-`<2>6` がこれを尽くす。
    BY D2, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, CODE src/rc_ir/ast.rs: RcExpr

<1>4. QED
  BY <1>1, <1>3

### L6 (`callee_params` と出力の `funcs` が持つ鍵)

**言明**。`borrow_ify` の `callee_params` の鍵の集合と、出力の `funcs` の鍵の集合は、どちらも
「入力の各関数の名前」と「`borrow_versions` の各値」の合併である。

<1>1. `callee_params` に元を入れるのは 2 か所である。入力の各関数について
      `callee_params.insert(func.name.clone(), ..)`、`clones` の各元について
      `callee_params.insert(borrow_version.clone(), ..)` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `clones` に積まれる第 1 成分は、`borrow_versions.get(&func.name)` が `Some(borrow_version)` を返した
      ときのその `borrow_version` である。`borrow_versions` に元を入れるのは 1 番目のループであり、その鍵は
      `func.name` すなわち入力の関数の名前だけなので、2 番目のループはそのすべての鍵を引き当て、
      `borrow_versions` の値はすべて `clones` に現れる。
  BY CODE src/rc_ir/borrow.rs: borrow_ify の `for func in prog.funcs.values()` の 1 番目と 2 番目のループ

<1>3. 出力の `funcs` に元を入れるのは 2 か所である。入力の各関数について
      `funcs.insert(f_own.name.clone(), f_own)` (`f_own.name` は `func.name` である)、`clones` の各元に
      ついて `funcs.insert(borrow_version, clone)` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>4. QED
  BY <1>1, <1>2, <1>3

## 3. 発見 -- P14 の反例

### 3.1 反例が置く型と関数

**DEF A** -- `Array I64`。

**DEF P** -- `(A, A)` すなわち 2 つの `A` を持つ unbox のタプル。

**DEF U** -- `unbox union { twins : P, none : () }`。Fix で書ける宣言である。`Std::LoopState s r` が
`unbox union { continue : s, break : r }` であり、`s` を `P` にとったものが同じ形である
(`CODE src/fixstd/std.fix: LoopState`)。

**DEF f** -- 次の関数とする。funptr ABI (`capture` は `None`)、パラメータは `x : A` と `y : A`、返り値の型は
`A` である。`s` は任意の `RcState` とする。本体 `B` は次のとおりである。

```
B = Retain(x, [], s,
    Let(t, Llvm(MS, [x, y]),
    Let(a, Llvm(MU, [t]),
    Release(a, [], s,
    Ret(x)))))
```

ここで `MS` は `InlineLLVMMakeStructBody` の `P` での実体、`MU` は `InlineLLVMMakeUnionBody` の `U` での実体で
変位 `twins` (添字 0) を構築するものである。`t : P`、`a : U` である。

入力プログラム `prog` は `funcs = {f}`、`globals = []` とし、`f.borrowed_units` は空とする。

### 3.2 型の leaf と unit

<1>1. `leaves(A) = {[]}`、`units(A) = {[]}` である。
  <2>1. `A` は `is_array` が真である。
    BY DEF A, CODE src/ast/types.rs: TypeNode::is_array
  <2>2. `boxed_leaf_paths` は `is_fully_unboxed`、`is_closure`、`is_box` のどれでもなく `is_array` が真の型に
        ついて、自分自身の path 1 つを積む。
    BY D4, <2>1, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `unit_step(A, type_env)` は `is_array` の行で `UnitStep::Unit` を返し、`rc_units_go` は `path` すなわち
        `[]` を積む。
    BY D5, <2>1, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>4. QED
    BY <2>2, <2>3

<1>2. `leaves(P) = {[0], [1]}`、`units(P) = {[0], [1]}` である。
  <2>1. `P` は unbox のタプルなので、`is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、`is_array`、
        `is_punched_array` のいずれも偽であり、`unpunched_field_types` は `[(0, A), (1, A)]` を返す。
    BY DEF P, CODE src/ast/types.rs: TypeNode::unpunched_field_types
  <2>2. `boxed_leaf_paths` は `<2>1` の型について各フィールドへ降り、`<1>1` より各フィールドで自分自身の
        path を積むので、`{[0], [1]}` を返す。
    BY D4, <1>1, <2>1, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `unit_step(P, type_env)` は `UnitStep::Fields { field_count: 2, held_fields: [(0, A), (1, A)] }` で
        あり、`rc_units_go` は各フィールドへ降りて `<1>1` より `[0]` と `[1]` を積む。
    BY D5, <1>1, <2>1, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>4. QED
    BY <2>2, <2>3

<1>3. `leaves(U) = {[0, 0], [0, 1]}`、`units(U) = {[]}` である。
  <2>1. `U` は `is_union` が真で `is_box` が偽であり、`unpunched_field_types` は `[(0, P), (1, ())]` を返す。
    BY DEF U, CODE src/ast/types.rs: TypeNode::is_union, TypeNode::unpunched_field_types
  <2>2. `boxed_leaf_paths` は `is_fully_unboxed`、`is_closure`、`is_box`、`is_array` のいずれも偽なので
        `unpunched_field_types` の各元へ降りる。変位 0 の `P` へ降りて `<1>2` より `[0, 0]` と `[0, 1]` を積み、
        変位 1 の `()` は `is_fully_unboxed` が真なので何も積まない。
    BY D4, <1>2, <2>1, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `unit_step(U, type_env)` は `is_union` の行で `UnitStep::Unit` を返し、`rc_units_go` は `[]` を積む。
    BY D5, <2>1, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>4. QED
    BY <2>2, <2>3

<1>4. `truncate_to_unit(U, [0, 0], type_env)` と `truncate_to_unit(U, [0, 1], type_env)` はどちらも `[]` で
      ある。
  `truncate_to_unit` は最初の添字で `unit_step(U, type_env)` を見て、`<1>3 の <2>3` よりそれは
  `UnitStep::Unit` なので `break` し、`out` は空のままである。
  BY <1>3, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>5. `truncate_to_unit(P, [0], type_env) = [0]`、`truncate_to_unit(P, [1], type_env) = [1]` である。
  `unit_step(P, type_env)` は `UnitStep::Fields` なので添字を `out` に積んで `A` へ降り、path はそこで
  終わる。
  BY <1>2, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>6. `B` の各 `Retain`/`Release` の path は、その変数の型の `units` の要素である。すなわち `f` は A2 を
      満たす。
  `Retain(x, [], ..)` の `x` の型は `A` で `[] ∈ units(A)` (`<1>1`)、`Release(a, [], ..)` の `a` の型は `U` で
  `[] ∈ units(U)` (`<1>3`) である。
  BY A2, <1>1, <1>3, DEF f

### 3.3 `MS` と `MU` が宣言するもの

<1>1. `MS` の `result_prov` は、`P` の各 boxed leaf `[i]` に単一の `Arg(i, [])` を宣言する。
  <2>1. `MS` の `result_prov` は `Provenance::build_shape(result_ty, type_env, &|path| match
        path.split_first() { None => sole_origin(Fresh), Some((i, rest)) => sole_origin(Arg(*i,
        rest.to_vec())) })` である。
    BY CODE src/fixstd/builtin.rs: `impl LLVMGen for InlineLLVMMakeStructBody` の result_prov
  <2>2. `build_shape` は `boxed_leaf_paths(result_ty, type_env)` の各元についてのみ関数を呼び、その path を
        鍵とする。
    BY CODE src/rc_ir/provenance.rs: Provenance::build_shape, CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape
  <2>3. QED
    `<1>2 の <2>2` より `P` の leaf は `[0]` と `[1]` であり、`split_first` はそれぞれ `(0, [])` と `(1, [])` を
    返す。
    BY <2>1, <2>2, 3.2 の <1>2

<1>2. `MU` の `result_prov` は、`U` の boxed leaf `[0, 0]` に単一の `Arg(0, [0])` を、`[0, 1]` に単一の
      `Arg(0, [1])` を宣言する。`[]` は鍵ではない。
  <2>1. `MU` の `result_prov` は `Provenance::build_shape(result_ty, type_env, &|path| match
        path.split_first() { None => sole_origin(Fresh), Some((k, rest)) if *k == variant_idx =>
        sole_origin(Arg(0, rest.to_vec())), Some(_) => Set::default() })` であり、`variant_idx` は 0 である。
    BY DEF f, CODE src/fixstd/builtin.rs: `impl LLVMGen for InlineLLVMMakeUnionBody` の result_prov,
       InlineLLVMMakeUnionBody::variant_index
  <2>2. QED
    `<1>1 の <2>2` と `3.2 の <1>3` より `U` の leaf は `[0, 0]` と `[0, 1]` だけで、`split_first` はそれぞれ
    `(0, [0])` と `(0, [1])` を返し、どちらも `*k == 0` である。鍵は leaf だけなので `[]` は鍵ではない。
    BY <2>1, <1>1, 3.2 の <1>3

<1>3. `MS` と `MU` の `borrows_operand` はどのオペランドについても偽である。
  どちらも `borrows_operand` を override しないので、`LLVMGen` の既定である `false` である。
  BY CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand,
     CODE src/fixstd/builtin.rs: `impl LLVMGen for InlineLLVMMakeStructBody`,
     `impl LLVMGen for InlineLLVMMakeUnionBody`

<1>4. `passthrough_arg_leaves(MS, P, [x, y], type_env)` は `{(0, []), (1, [])}` である。
  `passthrough_arg_leaves` は `result_prov` の各 leaf に `as_arg_projection` をかけ、単一の `Arg(j, p)` の
  ものを集める。`<1>1` の 2 つの leaf はどちらも単一の `Arg` である。
  BY <1>1, CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection

<1>5. `passthrough_arg_leaves(MU, U, [t], type_env)` は `{(0, [0]), (0, [1])}` である。
  BY <1>2, CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection

### 3.4 入力プログラムは D12 の意味で健全である

<1>1. `f` の実行路はただ 1 本であり、節点の列は `Retain(x, [])`、`Let(t, ..)`、`Let(a, ..)`、
      `Release(a, [])`、`Ret(x)` である。
  D2 より本体は木であり、`B` は `Match` を含まないので分岐が無い。
  BY D2, D3, DEF f

<1>2. `Obl` の初期値は `obj(x, [])` への参照 1 つと `obj(y, [])` への参照 1 つである。
  A1 より `f.borrowed_units` は空なのでパラメータの全 unit が所有され (D14)、`3.2 の <1>1` より `x` と `y` の
  boxed leaf はそれぞれ `[]` の 1 つで、unbox union を通らないので inhabited である (D16)。
  BY A1, D10, D14, D16, 3.2 の <1>1

<1>3. `Retain(x, [], s, ..)` の後、`Obl` は `obj(x, [])` への参照 2 つと `obj(y, [])` への参照 1 つを持つ。
      `H(obj(x, []))` は 1 上がる。
  BY D10, <1>2, 3.2 の <1>1

<1>4. `Let(t, Llvm(MS, [x, y]), ..)` は `Obl` を変えない。`t` の leaf `[0]` のスロットは `obj(x, [])` を、
      `[1]` のスロットは `obj(y, [])` を指す。
  <2>1. `MS` の 2 つの結果 leaf はどちらも単一の `Arg` を宣言するので、D10 の生成の表の `Llvm` の行には
        当たらず、D9 の移動の表の最後の行 (素通し leaf) に当たる。
    BY A3, D9, D10, 3.3 の <1>1
  <2>2. `MS` のどのオペランドも `borrows_operand` は偽だが、`<2>1` より `x` の leaf `[]` は `(0, [])` として、
        `y` の leaf `[]` は `(1, [])` として素通しに入るので、D9 の `Llvm` の消費の行には当たらない。
    BY D9, 3.3 の <1>3, 3.3 の <1>4, 3.2 の <1>1
  <2>3. QED
    移動は `Obl` を変えない。A3 の「単一の `Arg(j, σ)`」の行より、生成コードは結果の leaf `[0]` に第 0
    オペランド (`x`) の leaf `[]` と同じ参照を、leaf `[1]` に第 1 オペランド (`y`) の leaf `[]` と同じ参照を
    置く。
    BY A3, D9, D10, <2>1, <2>2

<1>5. `Let(a, Llvm(MU, [t]), ..)` は `Obl` を変えない。`a` の変位は `twins` (添字 0) であり、`a` の leaf
      `[0, 0]` のスロットは `obj(x, [])` を、`[0, 1]` のスロットは `obj(y, [])` を指す。この 2 つの leaf は
      inhabited であり、`[1]` の下に leaf は無い。
  <2>1. `MU` の 2 つの結果 leaf はどちらも単一の `Arg` を宣言するので、D9 の移動の表の最後の行に当たり、
        D10 の生成の表の `Llvm` の行には当たらない。
    BY A3, D9, D10, 3.3 の <1>2
  <2>2. `t` の leaf `[0]` は `(0, [0])` として、`[1]` は `(0, [1])` として素通しに入るので、D9 の `Llvm` の
        消費の行には当たらない。
    BY D9, 3.3 の <1>3, 3.3 の <1>5, 3.2 の <1>2
  <2>3. `a` は `MU` が変位 0 を構築した値なので、`a` の unbox union の節のタグは 0 である。`3.2 の <1>3` より
        `U` の boxed leaf は `[0, 0]` と `[0, 1]` の 2 つで、どちらもこの節で変位 0 を選ぶので inhabited で
        ある。`[1]` の下に leaf は無い。
    BY A3, D16, 3.2 の <1>3, 3.3 の <1>2
  <2>4. QED
    移動は `Obl` を変えない。A3 の「単一の `Arg(j, σ)`」の行より、生成コードは結果の leaf `[0, 0]` に
    `t` の leaf `[0]` と同じ参照を、leaf `[0, 1]` に `t` の leaf `[1]` と同じ参照を置く。`<1>4` より
    その 2 つはそれぞれ `obj(x, [])` と `obj(y, [])` を指す。
    BY A3, D9, D10, <1>4, <2>1, <2>2, <2>3

<1>6. `Release(a, [], s, ..)` の後、`Obl` は `obj(x, [])` への参照 1 つを持ち、`obj(y, [])` への参照を
      持たない。
  D10 の `Release` の行は `[]` の下の inhabited な各 leaf につき参照を 1 つ取り除く。`3.2 の <1>3` と
  `<1>5` よりその leaf は `[0, 0]` と `[0, 1]` の 2 つで、指すオブジェクトは `obj(x, [])` と `obj(y, [])` で
  ある。`<1>3` の `Obl` はそれぞれ 2 つと 1 つを持つ。
  BY D10, 3.2 の <1>3, <1>3, <1>5

<1>7. 終端の `Ret(x)` の消費の後、`Obl` は空である。
  D9 の終端の `Ret` の行は `x` の全 boxed leaf の参照を取り除く。`3.2 の <1>1` よりそれは `[]` の 1 つで、
  `<1>6` の `Obl` はその参照を 1 つ持つ。
  BY D9, D10, 3.2 の <1>1, <1>6

<1>8. `f` の本体は D11 の意味で健全である。
  <2>1. (S-a) が成り立つ。`Obl` から参照を取り除く操作は `Release(a, [])` と終端の `Ret(x)` の消費の 2 つで
        あり、`<1>6` と `<1>7` がそれぞれ取り除かれる参照がその時点の `Obl` にあることを与える。
    BY D11, <1>1, <1>6, <1>7
  <2>2. (S-b) が成り立つ。`<1>7` である。
    BY D11, <1>7
  <2>3. (S-c) が成り立つ。この実行路にある読む構文は `Let(t, Llvm(MS, [x, y]), ..)` と
        `Let(a, Llvm(MU, [t]), ..)` の 2 つである。前者が読みうるのは `obj(x, [])` と `obj(y, [])` で、その
        時点の `Obl` は両方への参照を持つので `H` は正であり、解放されていない。後者が読みうるのは `t` の
        inhabited な leaf が指すオブジェクト、すなわち同じ 2 つであり、`Obl` は同じく両方への参照を持つ。
    BY D7, D8, D11, <1>3, <1>4, <1>5
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>9. `prog` は D12 の意味で健全であり、A1 と A2 を満たす。
  `funcs` は `f` だけ、`globals` は空である。健全性は `<1>8`、A1 の `borrowed_units` が空であることは
  `DEF f`、A2 は `3.2 の <1>6` である。
  BY A1, A2, D12, DEF f, <1>8, 3.2 の <1>6

### 3.5 `infer_ownership` が返すもの

<1>1. `collect_consumes(B, vars, prog, own, type_env, out)` は、`own` が何であっても `out` に `(x, [])` を
      1 つだけ積む。
  <2>1. `collect_consumes_go` が `out` に積むのは、`RcExpr::Ret` の腕、`RcExpr::Destructure` の腕、および
        `RcExpr::Let` の腕が `rhs` の種類に応じて呼ぶ `rhs_consumes` の `Closure`・`App`・`Llvm` の腕の
        5 か所である。ほかの腕 (`RcExpr::Retain | RcExpr::Release | RcExpr::Eval`、`rhs_consumes` の
        `Var | Match`) は何も積まない。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes
  <2>2. `B` は `Destructure`、`Closure`、`App` を含まない。
    BY DEF f
  <2>3. `Let(t, Llvm(MS, [x, y]), ..)` について `rhs_consumes` の `Llvm` の腕は何も積まない。
        `borrows_operand` は両オペランドについて偽だが、`x` の唯一の leaf `[]` は `(0, [])` として、`y` の
        唯一の leaf `[]` は `(1, [])` として `passthrough` に入る。
    BY 3.2 の <1>1, 3.3 の <1>3, 3.3 の <1>4,
       CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>4. `Let(a, Llvm(MU, [t]), ..)` について `rhs_consumes` の `Llvm` の腕は何も積まない。`t` の leaf は
        `[0]` と `[1]` で、それぞれ `(0, [0])` と `(0, [1])` として `passthrough` に入る。
    BY 3.2 の <1>2, 3.3 の <1>3, 3.3 の <1>5,
       CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>5. `RcExpr::Ret(x)` の腕は `push_boxed_leaves(&x.name, &x.ty, type_env, out)` を呼び、
        `3.2 の <1>1` より `(x, [])` を 1 つ積む。
    BY 3.2 の <1>1, CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Ret` の腕,
       push_boxed_leaves
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>2. `origin(x, [])` は `Exactly((x, []))` である。
  `x` は `f` のパラメータなので `VarTable::of` はその `Binding` を `Param` にし、`origin_inner` の
  `Some(Binding::Param)` の腕は `here()` を返す。
  BY CODE src/rc_ir/ownership.rs: VarTable::of, origin_inner

<1>3. `infer_ownership(prog, type_env)` が返す `owned_leaves` は `{(x, [])}` である。
  <2>1. 不動点の各周回で `consumed` は `[(x, [])]` である。
    BY <1>1, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>2. その 1 元について `origin(vars, type_env, &x, &[]).candidates()` は `[(x, [])]` であり、`x` は
        `vars.param_tys` にあるので `(x, [])` が挿入される。
    BY <1>2, CODE src/rc_ir/borrow.rs: infer_ownership, CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>3. 2 周目は `owned_leaves` を変えないので `changed` が偽になり、ループは終わる。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>4. `borrow_versions` は `f.name` に `f_borrow` の名を対応させる。
  <2>1. `f.capture` は `None` である。
    BY DEF f
  <2>2. `func_has_borrowable_param(f, owned_leaves, type_env)` は真である。`y` の唯一の leaf `[]` について
        `owned_leaves.owns(&y, &[])` は `<1>3` より偽である。
    BY <1>3, 3.2 の <1>1, CODE src/rc_ir/borrow.rs: func_has_borrowable_param, OwnedLeaves::owns
  <2>3. QED
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: borrow_ify

<1>5. `owned_units` は `{(x, []), (y, []), (x', [])}` である。ここで `x'` は `rename[x]`、`y'` は
      `rename[y]` である。
  <2>1. L2 の (a) が入れるのは `f` のパラメータ `x`、`y` の unit である。`3.2 の <1>1` より
        `units(A) = {[]}` なので `(x, [])` と `(y, [])` である。`f.capture` は `None` なので capture の分は
        無い。
    BY L2, DEF f, 3.2 の <1>1
  <2>2. L2 の (b) が入れるのは、`<1>4` より `f` について走り、`<1>3` より `owned_leaves.owns` が真なのは
        `(x, [])` だけなので、`(rename[x], truncate_to_unit(A, [], type_env))` すなわち `(x', [])` である。
    BY L2, <1>3, <1>4, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>3. QED
    BY <2>1, <2>2

<1>6. `f_borrow` の `borrowed_units` は `{(y', [])}` である。
  `borrow_ify` は各版について `param_capture_units` から `owned_units` に入らないものを集める。`f_borrow` の
  パラメータは `x'` と `y'` で、`3.2 の <1>1` より unit はそれぞれ `[]` の 1 つである。`<1>5` より `(x', [])` は
  `owned_units` にあり、`(y', [])` は無い。
  BY <1>5, CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units

### 3.6 `f_borrow` の本体

`f_borrow` の `VarTable` を `vars'` とし、この節の `origin` は `vars'` について読む。`fresh_rename_function` は
束縛変数を一斉に付け替えるだけなので、`f_borrow` の本体は `B` の `x`、`y`、`t`、`a` をそれぞれ `x'`、`y'`、
`t'`、`a'` に置き換えたものである (`CODE src/rc_ir/rename.rs: fresh_rename_function`)。

<1>1. `origin(t', [0])` は `Exactly((x', []))`、`origin(t', [1])` は `Exactly((y', []))` である。
  <2>1. `vars'` は `t'` の `Binding` を `Llvm(MS, [x', y'], P)` にする。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Llvm` の場合
  <2>2. `decl.leaf_origins_at(&[0])` は単一の `Arg(0, [])` なので `as_arg_projection` は `Some((0, []))` を
        返し、`origin_inner` は `origin(x', [])` へ進む。
    BY 3.3 の <1>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕,
       as_arg_projection
  <2>3. `x'` と `y'` は `f_borrow` のパラメータなので `origin(x', [])` は `Exactly((x', []))`、
        `origin(y', [])` は `Exactly((y', []))` である。
    BY 3.5 の <1>2, CODE src/rc_ir/ownership.rs: VarTable::of, origin_inner
  <2>4. `decl.leaf_origins_at(&[1])` は単一の `Arg(1, [])` なので、同じ腕が `origin(y', [])` へ進む。
    BY 3.3 の <1>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕,
       as_arg_projection
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4

<1>2. `origin(a', [])` は `Join { identity: (a', []), candidates: {(x', []), (y', [])} }` である。
  <2>1. `vars'` は `a'` の `Binding` を `Llvm(MU, [t'], U)` にする。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Llvm` の場合
  <2>2. `decl.leaf_origins_at(&[])` は `None` である。`Provenance` の鍵は leaf だけであり、`[]` は `U` の
        leaf ではない。
    BY 3.2 の <1>3, 3.3 の <1>2, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at
  <2>3. よって `origin_inner` は `origin_from_leaves_under(vars', type_env, &decl, [t'], [], (a', []))` を
        呼ぶ。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕
  <2>4. `decl.leaf_origins_under(&[])` は `[0, 0]` の宣言 `{Arg(0, [0])}` と `[0, 1]` の宣言
        `{Arg(0, [1])}` を返す。
    BY 3.2 の <1>3, 3.3 の <1>2, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
  <2>5. `operand_units` は `{(0, [0]), (0, [1])}` である。`truncate_to_unit(P, [0], type_env)` は `[0]`、
        `truncate_to_unit(P, [1], type_env)` は `[1]` である。
    BY 3.2 の <1>5, <2>4, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>6. `produced_here` は偽である。`<2>4` の宣言に `Fresh` も `Unknown` も無い。
    BY <2>4, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>7. `reached` は `[origin(t', [0]), origin(t', [1])]` すなわち `[Exactly((x', [])), Exactly((y', []))]`
        である (順序は `operand_units` の反復順による)。
    BY <1>1, <2>5, <2>6, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>8. `reached` の 2 元は等しくないので、`origin_from_leaves_under` は
        `Origin::of_candidates({(x', []), (y', [])}, (a', []))` を返す。
    A6 より `x'` と `y'` は相異なる名である。
    BY A6, <2>7, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>9. QED
    `of_candidates` は 2 元の集合に対し `Join { identity, candidates }` を返す。
    BY <2>8, CODE src/rc_ir/ownership.rs: Origin::of_candidates

<1>3. `owns_object(x', [])` は真、`owns_object(y', [])` は偽である。
  <2>1. `vars'.param_tys` は `x'` と `y'` をどちらも `A` で持つ。
    BY CODE src/rc_ir/ownership.rs: VarTable::of
  <2>2. `units_under(A, [], type_env)` は `[[]]` である。
    BY L1, 3.2 の <1>1
  <2>3. `truncate_to_unit(A, [], type_env)` は `[]` である。空の path について繰り返しを回さない。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>4. QED
    L4 と `<2>1`-`<2>3` より、`owns_object(x', [])` は `(x', []) ∈ owned_units`、`owns_object(y', [])` は
    `(y', []) ∈ owned_units` と同値である。`3.5 の <1>5` より前者は真、後者は偽である。
    BY L4, 3.5 の <1>5, <2>1, <2>2, <2>3

<1>4. `owns_unit(a', [])` は偽である。
  `owns_unit` は `origin(a', []).candidates()` の全元について `owns_object` を要求する。`<1>2` より
  candidates は `{(x', []), (y', [])}` であり、`<1>3` より `owns_object(y', [])` は偽である。
  BY <1>2, <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit, CODE src/rc_ir/ownership.rs:
     Origin::candidates

<1>5. `owns_unit(x', [])` は真である。
  `origin(x', [])` は `Exactly((x', []))` (`<1>1 の <2>3`) なので candidates は `{(x', [])}` であり、
  `<1>3` より `owns_object(x', [])` は真である。
  BY <1>1, <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>6. `f_borrow` の本体は次のとおりである。

      ```
      Retain(x', [], s,
      Let(t', Llvm(MS, [x', y']),
      Let(a', Llvm(MU, [t']),
      Ret(x'))))
      ```

  <2>1. `Retain(x', [], s, ..)` は残る。L3 と `<1>5` による。`[] ∈ units(A)` は `3.2 の <1>1` である。
    BY L3, 3.2 の <1>1, <1>5
  <2>2. `Release(a', [], s, ..)` は消える。L3 と `<1>4` による。`[] ∈ units(U)` は `3.2 の <1>3` である。
    BY L3, 3.2 の <1>3, <1>4
  <2>3. 2 つの `Let` は `rewrite_inner` の `RcExpr::Let(x, rhs, k)` の腕 (`App` でも `Match` でもない場合) を
        通り、`rhs` をそのまま写す。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
  <2>4. `Ret(x')` は `rewrite_inner` の `RcExpr::Ret` の腕を通り、そのまま写る。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4

### 3.7 `f_borrow` は D11 の (S-b) を破る

<1>1. `f_borrow` の実行路はただ 1 本であり、節点の列は `Retain(x', [])`、`Let(t', ..)`、`Let(a', ..)`、
      `Ret(x')` である。
  BY D2, D3, 3.6 の <1>6

<1>2. `Obl` の初期値は `obj(x', [])` への参照 1 つだけである。
  `3.5 の <1>6` より `f_borrow` の `borrowed_units` は `{(y', [])}` なので、D14 より `x'` の unit `[]` は
  所有され `y'` の unit `[]` は借用される。`3.2 の <1>1` より両者の inhabited な leaf はそれぞれ `[]` の
  1 つである。
  BY D10, D14, D16, 3.2 の <1>1, 3.5 の <1>6

<1>3. `Retain(x', [], s, ..)` の後、`Obl` は `obj(x', [])` への参照 2 つを持つ。
  BY D10, <1>2, 3.2 の <1>1

<1>4. 2 つの `Let` は `Obl` を変えない。`a'` の変位は `twins` (添字 0) であり、`a'` の leaf `[0, 0]` の
      スロットは `obj(x', [])` を、`[0, 1]` のスロットは `obj(y', [])` を指す。この 2 つの leaf は inhabited
      である。
  <2>1. `Let(t', Llvm(MS, [x', y']), ..)` は `Obl` を変えない。`MS` の 2 つの結果 leaf はどちらも単一の
        `Arg` を宣言するので D9 の移動の表の最後の行に当たり、D10 の生成の表の `Llvm` の行には当たらない。
        `x'` の唯一の leaf `[]` は `(0, [])` として、`y'` の唯一の leaf `[]` は `(1, [])` として
        `passthrough` に入るので、D9 の `Llvm` の消費の行にも当たらない。
    BY A3, D9, D10, 3.2 の <1>1, 3.3 の <1>1, 3.3 の <1>3, 3.3 の <1>4
  <2>2. `t'` の leaf `[0]` のスロットは `obj(x', [])` を、`[1]` のスロットは `obj(y', [])` を指す。
        A3 の「単一の `Arg(j, σ)`」の行より、生成コードは結果の leaf `[0]` に第 0 オペランド (`x'`) の
        leaf `[]` と同じ参照を、leaf `[1]` に第 1 オペランド (`y'`) の leaf `[]` と同じ参照を置く。
    BY A3, 3.2 の <1>1, 3.3 の <1>1
  <2>3. `Let(a', Llvm(MU, [t']), ..)` は `Obl` を変えない。`MU` の 2 つの結果 leaf はどちらも単一の `Arg` を
        宣言するので D9 の移動の表の最後の行に当たり、D10 の生成の表の `Llvm` の行には当たらない。`t'` の
        leaf `[0]` は `(0, [0])` として、`[1]` は `(0, [1])` として `passthrough` に入るので、D9 の `Llvm` の
        消費の行にも当たらない。
    BY A3, D9, D10, 3.2 の <1>2, 3.3 の <1>2, 3.3 の <1>3, 3.3 の <1>5
  <2>4. `a'` の unbox union の節のタグは 0 であり、`a'` の leaf `[0, 0]` のスロットは `obj(x', [])` を、
        `[0, 1]` のスロットは `obj(y', [])` を指す。この 2 つの leaf は inhabited である。
        A3 より `MU` は変位 0 を構築し、その変位の leaf に `t'` の leaf `[0]`、`[1]` と同じ参照を置く。
    BY A3, D16, 3.3 の <1>2, <2>2
  <2>5. QED
    BY <2>1, <2>3, <2>4

<1>5. 終端の `Ret(x')` の消費の後、`Obl` は `obj(x', [])` への参照 1 つを持つ。
  D9 の終端の `Ret` の行は `x'` の全 boxed leaf の参照を 1 つずつ取り除く。`3.2 の <1>1` よりそれは `[]` の
  1 つである。
  BY D9, D10, 3.2 の <1>1, <1>3, <1>4

<1>6. `f_borrow` の本体は D11 の (S-b) を破る。
  BY D11, <1>1, <1>5

<1>7. `borrow_ify(prog, type_env)` の出力は D12 の意味で健全でない。
  出力の `funcs` は `f_borrow` を含み (`3.5 の <1>4`)、その本体は健全でない (`<1>6`)。
  BY D12, 3.5 の <1>4, <1>6

<1>8. P14 は偽である。
  `3.4 の <1>9` より `prog` は D12 の意味で健全で A1 と A2 を満たし、`<1>7` より出力は D12 の意味で健全で
  ない。
  BY P14, 3.4 の <1>9, <1>7

### 3.8 実行時に何が起きるか

<1>1. `f_borrow` の 1 回の活性化は、`H(obj(x', []))` を 1 上げ、下げない。
  <2>1. `Retain(x', [])` は `H(obj(x', []))` を 1 上げる。
    BY D10, 3.6 の <1>6
  <2>2. `f_borrow` の本体に `Release` は無い。`Ret(x')` の消費は参照を呼び出し元へ渡すので `H` を変えない。
    BY D9, D10, 3.6 の <1>6
  <2>3. QED
    BY <2>1, <2>2

<1>2. 呼び出し元が `x'` の位置へ渡した参照は、どこでも処分されない。
  `3.5 の <1>6` より `f_borrow` はその unit を所有するので、D14 より処分するのは `f_borrow` の側である。
  `<1>1 の <2>2` より `f_borrow` はそれを処分しない。
  BY D14, 3.5 の <1>6, <1>1

<1>3. QED
  `obj(x', [])` の参照カウントは活性化ごとに 1 ずつ増え、0 にならない。すなわち `A` の記憶域が解放されない。
  BY D7, <1>1, <1>2

### 3.9 これが `borrow_ify` の何に当たるか

反例が働かせたのは `rewrite_rc` のフィルタである。借用版は `Release(a', [])` を落とすが、この 1 節点は
`obj(x', [])` への参照と `obj(y', [])` への参照の 2 つを処分していた (`3.4 の <1>6`)。前者は `f_borrow` が
所有し、後者は借用する。`owns_unit` は unit 1 つにつき真偽値 1 つを答えるので、この 2 つを分けられない。
`owns_unit` が偽を答えて節点全体が落ち、所有する側の参照が残る。

`owns_unit` がこの位置で真を答えていれば、`Release(a', [])` は残る。`3.7 の <1>3` と `3.7 の <1>4` より
その位置の `Obl_borrow` は `obj(x', [])` への参照 2 つだけを持ち、`obj(y', [])` への参照を持たない。D10 の `Release` の
行はその参照を 1 つ取り除こうとするので、D11 の (S-a) が破れる。同じ 1 つの本体が、`owns_unit` の答えが偽の
ときは (S-b) を、真のときは (S-a) を破る。`rewrite_rc` の判定を真偽どちらへ倒しても直らない。

同じ形は `call_rc` の 2 分岐にも現れる。`callee_owns` が真で `arg_owned` が偽のとき、`call_rc` は
`Retain(arg, unit)` を 1 つ置く。この `Retain` は D10 より `unit` の下の inhabited な**すべての** leaf の
参照を 1 つずつ作るので、所有する側の leaf についても 1 つ作る。呼び出しはその leaf の参照を 1 つ消費するだけ
なので、活性化の初期値として持っていた参照が余る。`callee_owns` が偽で `arg_owned` も偽のときは、
`call_rc` は何も置かないが、呼び出し先は借用するので消費もしないため、所有する側の参照が処分されない。

**1 つの unit の下の leaf の所有が分かれる**ことが、この 3 か所すべての前提である。それが起きうるのは、
1 つの unit が 2 つ以上の leaf を持つ型があるからである。`unit_step` が `UnitStep::Unit` を返す 4 つの型の
うち、`is_box` の型・`is_array` の型・`is_punched_array` の型は leaf を 1 つしか持たず (`boxed_leaf_paths` が
それぞれ自分自身の path で止まるか、`_arr` フィールドの `Array` で止まる)、`UnitStep::Capture` も
capture 1 つである。leaf を 2 つ以上持つ unit は **unbox union だけ**である。反例の `U` がそれであり、
`Std::LoopState s r` の `s` に参照を 2 つ持つ値をとったものが同じ形になる。

### 3.10 反例が入力に要求したもの

反例は次の 3 つだけを使った。どれも `insert_rc` の出力が満たしうるものである。

- 1 つの unbox union の 1 つの変位の payload が、別々のパラメータに由来する参照を 2 つ持つこと。
- その union を 1 つの `Release` が処分すること。A2 より union は 1 unit なので、`split_rc_units` が出す
  `Release` は必ずこの形である。
- 2 つのパラメータのうち片方だけが `owned_leaves` に入ること。反例では終端の `Ret(x)` が `x` を消費し、`y` は
  どこでも消費されない。

Fix のソースでは、2 つの配列を持つ値を unbox union に入れて捨て、引数の片方だけを返す関数がこの形になる。

## 4. 局所の仮説

この節が置く 3 つは、`README.md` の仮定ではなく、この文書の中だけで使う仮説である。どれも対象コミットの
コードでは成り立たない (H1、H2) か、コードのどこも検査していない (H3)。

### H1 (所有の一様性)

**言明**。`infer_ownership` が返す `owned_leaves` について、入力の各関数の各パラメータ・capture `p` と
`ty(p)` の各 unit `u` をとる。`truncate_to_unit(ty(p), λ, type_env) = u` を満たす `λ ∈ leaves(ty(p))` の
すべてについて、`owned_leaves.owns(p.name, λ)` の値は等しい。

P1 より `leaves(ty(p))` の各 leaf の `truncate_to_unit` は `units(ty(p))` の要素であり、`units(ty(p))` の
各 unit はある leaf の `truncate_to_unit` であるから、この言明は `leaves(ty(p))` を `units(ty(p))` の上の
空でない類に分けたうえで、各類の中で `owns` が一定であることを言っている。

**成り立たない例**。`unit_step` が `UnitStep::Unit` を返す型のうち、1 つの unit の下に 2 つ以上の leaf を
持つのは unbox union だけである (第 3.9 節)。union 型のパラメータの一方の変位の payload だけが消費される
関数では、`owned_leaves` はその変位の leaf だけを持ち、H1 は破れる。

### H2 (unit の下の leaf の所有の一様性)

**言明**。`borrow_ify` の出力の各版について、`RewriteCtx::rewrite` が走査する本体 (`f_own` については
入力の関数の本体、`f_borrow` については `clone_func` が返した本体) と、その版の `RewriteCtx` `ctx` をとる。
その本体の各 `Retain(w, π, ..)` / `Release(w, π, ..)` 節点の各 `u ∈ units_under(ty(w), π, type_env)`、および
その本体の各 `Let(x, App(callee, args), k)` の各引数 `w ∈ args` の各 `u ∈ units(ty(w))` について、次が
成り立つ。`u` を接頭辞に持つすべての `λ ∈ leaves(ty(w))` について、`origin(w, λ).candidates()` の全元が
`ctx.owns_object` を満たすことと、`ctx.owns_unit(w, u)` が真であることとが同値である。

**成り立たない例**。第 3 節の `f_borrow` の `Release(a', [], ..)` の `u = []` である。`3.6 の <1>4` より
`owns_unit(a', [])` は偽である。一方 `3.2 の <1>3` より `[0, 0]` は `U` の boxed leaf であり、
`3.3 の <1>2` より `MU` の `result_prov` はその leaf に単一の `Arg(0, [0])` を宣言するので、`origin_inner` の
`Some(Binding::Llvm(..))` の腕は `origin(t', [0])` へ進み、`3.6 の <1>1` よりそれは `Exactly((x', []))` で
ある。`3.6 の <1>3` より `owns_object(x', [])` は真である。よって同値が破れる。

### H3 (入力の束縛名の形)

**言明**。入力の束縛名 (DEF 入力の束縛名) の `name` フィールドを `#` で区切ったとき、最後の断片が、文字
`b` の後に 10 進数字だけが 1 個以上続く形になっているものは無い。

**この仮説を果たす者はいない。** 検査するコードも表明も無い。成り立つと考える根拠は、入力の束縛名の作られ方が
次の 2 通りしか無いことである。

- `Lowerer::fresh_var` が作る `FullName::local(format!("{}#{}{}", hint, self.symbol_tag, self.fresh_counter))`
  (`CODE src/rc_ir/lower.rs: Lowerer::fresh_var`)。`symbol_tag` は md5 の 16 進表記の先頭 16 文字であり
  (`CODE src/rc_ir/lower.rs: Lowerer::lower_symbol`, `SYMBOL_TAG_LENGTH`)、`fresh_counter` は 1 以上の
  10 進数である。よって最後の断片は「16 個の 16 進文字 + 10 進数」である。
- `simplify` が `clone_fresh(&outer.body, PASS_TAG, counter)` で作る `format!("{}#{}{}", name, "cc", counter)`
  (`CODE src/rc_ir/simplify.rs: PASS_TAG`, `CODE src/rc_ir/rename.rs: clone_fresh, assign_fresh_name`)。
  最後の断片は `cc` で始まる。

`cc` で始まる断片は `b` で始まらない。16 進文字 + 10 進数の断片が `b` の後に 10 進数だけが続く形になるには、
`symbol_tag` の先頭が `b` で、残りの 15 文字がすべて 10 進数字で、その先頭が `0` でないことが要る。そのとき
`b` の後の数字列は 15 + 1 桁以上であり、10 進数として `10^15` 以上である。`borrow_ify` が付ける番号は
`rename_counter` の値であり、これは 1 回の `borrow_ify` の実行の中で複製された束縛子の個数を超えない
(`CODE src/rc_ir/borrow.rs: borrow_ify`, `CODE src/rc_ir/rename.rs: assign_fresh_name`)。よって衝突は、
プログラムが `10^15` 個以上の束縛子を持つときにしか起こりえない。

Fix のソースの識別子は ASCII の英字・数字・`_`・先頭の `@` だけからなり、`#` を含まない
(`CODE src/parse/grammer.pest: name`, `name_char`, `name_head`)。よって束縛名の中の `#` はすべてコンパイラが
置いたものである。

## 5. P8 -- 推論の停止性と安全性

### 5.1 P8 の言明が読む所有権の割り当て

P8 の後半は「D9 の意味で消費される」と言う。D9 の `App` の行は「呼び出し先がその位置の unit を所有する
(D14) 引数の leaf」であり、D14 の所有は `RcFunc::borrowed_units` が定める。よって「どの割り当ての下での
消費か」を決めないと言明が定まらない。候補は 2 つある -- 入力の割り当て (A1 より全所有) と、
`infer_ownership` が計算している割り当てである。L7 が前者を退ける。

#### L7 (入力の割り当てで読むと P8 は偽である)

**言明**。D12 の意味で RC 規律を満たし A1 と A2 を満たす入力プログラム `Q` で、次を満たすものがある。
`Q` のある関数のあるパラメータ leaf の参照が、A1 の割り当て (全所有) の下で D9 の意味で消費される実行路が
あるのに、その leaf は `infer_ownership(Q, type_env)` が返す `owned_leaves` に入らない。

**`Q` の定義**。`A` は第 3.1 節の `DEF A` (`Array I64`) とする。`s` は任意の `RcState` とする。

- `g`: funptr ABI (`capture` は `None`)、パラメータは `y : A` と `n : I64`、`ret_ty` は `I64`。
  本体は `Release(y, [], s, Ret(n))`。
- `f`: funptr ABI、パラメータは `x : A` と `m : I64`、`ret_ty` は `I64`。本体は
  `Let(w, App(gv, [x, m]), Ret(w))`。ここで `gv` は `g` の名前を持つ `RcVar` で、その型は `g` の funptr 型、
  `w : I64` である。
- `funcs = {f, g}`、`globals = []`、両方の `borrowed_units` は空。

<1>1. `leaves(I64) = {}` であり、`leaves(gv.ty) = {}` である。
  `I64` は `is_box` でも `is_closure` でも `is_array` でもなく、`unpunched_field_types` が空なので
  `is_fully_unboxed` が真である。funptr 型は `is_funptr` の行で `is_fully_unboxed` が真である。
  `boxed_leaf_paths` は `is_fully_unboxed` の型について何も積まない。
  BY D4, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `Q` は A2 を満たす。
  `Q` の `Retain`/`Release` 節点は `Release(y, [], s, ..)` の 1 つだけであり、`3.2 の <1>1` より
  `[] ∈ units(A)` である。
  BY A2, 3.2 の <1>1

<1>3. `g` の本体は D11 の意味で RC 規律を満たす。
  <2>1. `g` の実行路は 1 本であり、節点の列は `Release(y, [])`、`Ret(n)` である。
    BY D2, D3
  <2>2. `Obl` の初期値は `obj(y, [])` への参照 1 つである。`n` は `<1>1` より leaf を持たない。
    BY A1, D10, D14, D16, <1>1, 3.2 の <1>1
  <2>3. `Release(y, [], s, ..)` の後、`Obl` は空である。
    BY D10, <2>2, 3.2 の <1>1
  <2>4. 終端の `Ret(n)` の消費は何も取り除かない。`<1>1` より `n` は boxed leaf を持たない。
    BY D9, <1>1
  <2>5. QED
    (S-a) は `<2>3` が、(S-b) は `<2>3` と `<2>4` が与える。この実行路に D7 の読む構文は無い。
    BY D7, D11, <2>1, <2>3, <2>4

<1>4. `f` の本体は D11 の意味で RC 規律を満たす。
  <2>1. `f` の実行路は 1 本であり、節点の列は `Let(w, App(gv, [x, m]))`、`Ret(w)` である。
    BY D2, D3
  <2>2. `Obl` の初期値は `obj(x, [])` への参照 1 つである。
    BY A1, D10, D14, D16, <1>1, 3.2 の <1>1
  <2>3. `App(gv, [x, m])` は `obj(x, [])` への参照 1 つを消費し、参照を作らない。`gv` と `m` は `<1>1` より
        leaf を持たず、`w` も leaf を持たない。`g` の `borrowed_units` は空なので、D14 より `g` は
        `y` の unit `[]` を所有し、D9 の `App` の行より `x` の leaf `[]` は消費される。
    BY A1, D9, D10, D14, <1>1, 3.2 の <1>1
  <2>4. 終端の `Ret(w)` の消費は何も取り除かない。
    BY D9, <1>1
  <2>5. QED
    `<2>2` と `<2>3` より `App` の直後の `Obl` は空である。(S-a) は `<2>2` と `<2>3` が、(S-b) は
    `<2>3` と `<2>4` が与える。(S-c) は、この実行路の読む構文が `App` の 1 つだけで、それが読みうる
    オブジェクトは `obj(x, [])` であり、`<2>2` よりその時点の `Obl` がその参照を持つので `H` が正である
    ことによる。
    BY D7, D8, D11, <2>1, <2>2, <2>3, <2>4

<1>5. `Q` は D12 の意味で RC 規律を満たし、A1 と A2 を満たす。
  BY A1, A2, D12, <1>2, <1>3, <1>4

<1>6. `infer_ownership(Q, type_env)` は `OwnedLeaves` の中身が空の値を返す。
  <2>1. `owned_leaves` が空のとき、`g` について `collect_consumes` は何も積まない。
        `RcExpr::Release` の腕は継続へ進むだけであり、`RcExpr::Ret(n)` の腕は
        `push_boxed_leaves(&n.name, &n.ty, ..)` を呼び、`<1>1` より `I64` の leaf は無い。
    BY <1>1, CODE src/rc_ir/ownership.rs: collect_consumes_go, push_boxed_leaves
  <2>2. `owned_leaves` が空のとき、`f` について `collect_consumes` は何も積まない。
        `RcExpr::Let` の腕は `rhs_consumes` の `RcRhs::App` の腕を呼ぶ。そこで
        `push_boxed_leaves(&gv.name, &gv.ty, ..)` は `<1>1` より何も積まない。
        `resolve_callee_params(gv, vars, prog)` は `prog.funcs` に `g` があるので `Some([y, n])` を返す。
        引数 `x` の唯一の leaf `[]` について `owns(&y, &[])` は `owned_leaves.contains(&(y, []))` すなわち偽で
        あり、積まれない。引数 `m` は `<1>1` より leaf を持たない。継続の `RcExpr::Ret(w)` の腕は `<1>1` より
        何も積まない。
    BY <1>1, 3.2 の <1>1, CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes,
       resolve_callee_params, push_boxed_leaves
  <2>3. QED
    初期値は空であり、`<2>1` と `<2>2` より 1 周目で `consumed` はどちらの関数についても空なので
    `changed` は偽のままである。`insert` が 1 度も呼ばれないので `owned_leaves` は空のまま返る。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: infer_ownership

<1>7. QED
  `<1>4 の <2>3` より、`f` の唯一の実行路で、A1 の割り当ての下でパラメータ leaf `(x, [])` の参照が D9 の
  意味で消費される。`<1>6` よりその leaf は `owned_leaves` に入らない。`<1>5` より `Q` は P8 の前提を
  満たす。
  BY <1>4, <1>5, <1>6

#### 割り当ての決定

**この文書は P8 を、`infer_ownership` が計算している割り当ての下で読む。** 理由は 3 つである。

- L7 より、A1 の割り当てで読むと P8 は偽である。
- `infer_ownership` が `collect_consumes` に渡す `own` は `owned_leaves` そのものである
  (`CODE src/rc_ir/borrow.rs: infer_ownership`)。この読み方のもとで P8 は「`owned_leaves` が自分自身に
  ついての不動点条件を満たす」という言明になり、コードが計算しているものと言明が一致する。
- P14 が P8 を使う先は借用版である。借用版のパラメータの所有は `owned_units` が定め、`owned_units` の
  借用版の分は `owned_leaves` から `truncate_to_unit` で作られる (L2 の (b))。

**DEF 推論の割り当て**
入力の各関数 `f` の各パラメータ・capture `p` と各 `u ∈ units(ty(p))` について、`f` が `(p, u)` を
**推論の意味で所有する**とは、`truncate_to_unit(ty(p), λ, type_env) = u` を満たすある `λ ∈ leaves(ty(p))`
について `owned_leaves.owns(p.name, λ)` が真であることをいう。そうでないとき **推論の意味で借用する**と
いう。

#### L8 (H1 の下で `owned_leaves` は leaf 粒度の所有である)

**言明**。H1 の下で、各パラメータ・capture `p` と各 `λ ∈ leaves(ty(p))` について、
`owned_leaves.owns(p.name, λ)` は「`p` を持つ関数が `(p, truncate_to_unit(ty(p), λ, type_env))` を推論の
意味で所有する」ことと同値である。すなわち `owned_leaves` は、DEF 推論の割り当て についての
`p12-keys-and-consumes.md` の DEF leaf 粒度の所有 である。

<1>1. `u = truncate_to_unit(ty(p), λ, type_env)` は `units(ty(p))` の元である。
  BY P1

<1>2. `owned_leaves.owns(p.name, λ)` が真ならば、`f` は `(p, u)` を推論の意味で所有する。
  `λ` 自身が DEF 推論の割り当て の要求する leaf である。
  BY DEF 推論の割り当て, <1>1

<1>3. `f` が `(p, u)` を推論の意味で所有するならば、`owned_leaves.owns(p.name, λ)` は真である。
  DEF 推論の割り当て より、`truncate_to_unit(ty(p), λ', type_env) = u` かつ
  `owned_leaves.owns(p.name, λ')` が真であるような `λ' ∈ leaves(ty(p))` がある。`λ` も同じ `u` へ
  切り詰まるので、H1 よりこの 2 つの `owns` の値は等しい。
  BY DEF 推論の割り当て, H1, <1>1

<1>4. QED
  BY <1>2, <1>3

### 5.2 P8 の証明

**P8 (a)**。`infer_ownership(prog, type_env)` は停止する。

<1>1. `owned_leaves` が変わるのは `owned_leaves.insert((root_var.clone(), root_path.clone()))` の 1 か所
      だけであり、`changed` が真になるのはその `insert` が真を返したときだけである。ループは `changed` が
      偽のとき `break` する。
  BY CODE src/rc_ir/borrow.rs: infer_ownership

<1>2. `collect_consumes(&func.body, vars, prog, own, type_env, &mut consumed)` が `consumed` に積む対の
      全体は、`own` の値によらない有限集合 `S_func` に含まれる。
  <2>1. `collect_consumes_go` が `out` に積むのは、`RcExpr::Ret` の腕、`RcExpr::Destructure` の腕、および
        `rhs_consumes` の `Closure`・`App`・`Llvm` の腕の 5 か所だけである。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes
  <2>2. `<2>1` の 5 か所が積む対の第 1 成分は、その節点に現れる `RcVar` の名前であり、第 2 成分は
        その変数の型の `boxed_leaf_paths` の元である。`RcExpr::Ret` と `Closure`・`App` の腕は
        `push_boxed_leaves` を、`RcExpr::Destructure` の腕は `destructure_consumes` を、`Llvm` の腕は
        `boxed_leaf_paths` を直に使う。`destructure_consumes` は `boxed_leaf_paths` の絞り込みを返す。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes, push_boxed_leaves,
       destructure_consumes, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `own` を読むのは `rhs_consumes` の `App` の腕の `is_owning_position` だけであり、それは積むか
        積まないかを決めるだけで、積む対を変えない。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App` の腕
  <2>4. QED
    D2 より本体は有限の木であり、A10 より各型の `boxed_leaf_paths` は有限である。`<2>2` の対の全体は
    有限であり、`<2>3` よりその全体は `own` に依らない。
    BY A10, D2, <2>1, <2>2, <2>3

<1>3. 挿入されうる元の全体は有限である。
  <2>1. 挿入される元は、`consumed` の各元 `(var, path)` について
        `origin(vars, type_env, &var, &path).candidates()` の元のうち、`vars.param_tys` に鍵 `root_var` を
        持つものである。
    BY CODE src/rc_ir/borrow.rs: infer_ownership
  <2>2. `var_tables` はループの外で 1 度だけ作られ、`origin` は `vars` と `type_env` だけに依る。よって
        1 つの `(var, path)` に対する `candidates()` は周回によらず同じ有限集合である。`Origin::Exactly` は
        1 元、`Origin::Join` は有限集合の `candidates` を持つ。
    BY P2, CODE src/rc_ir/borrow.rs: infer_ownership,
       CODE src/rc_ir/ownership.rs: origin, Origin, Origin::candidates
  <2>3. QED
    `<1>2` より `consumed` に現れうる対 `(var, path)` の全体は有限であり、`<2>2` より各対の
    `candidates()` は有限である。関数は有限個である (D1 の `funcs` は写像である)。
    BY D1, <1>2, <2>1, <2>2

<1>4. QED
  `<1>1` より、`changed` が真になる周回では `owned_leaves` は真に大きくなる。`<1>3` よりその大きさには
  上界があるので、`changed` が真である周回は有限回しかなく、その後の 1 周で `changed` は偽になり `break`
  する。各周回は有限の仕事しかしない (`<1>2`、`<1>3`、P2)。
  BY P2, <1>1, <1>2, <1>3

**P8 (b)**。`infer_ownership` が返す `owned_leaves` は次の閉包条件を満たす。入力の各関数 `f` に
ついて、その最終の `owned_leaves` で `collect_consumes` を呼んだ結果の各元 `(var, path)` と、
`origin(vars, type_env, &var, &path).candidates()` の各元 `(root_var, root_path)` について、
`vars.param_tys` が `root_var` を鍵に持つならば `(root_var, root_path) ∈ owned_leaves` である。

<1>1. ループが `break` する周回では `changed` が偽である。
  BY CODE src/rc_ir/borrow.rs: infer_ownership

<1>2. その周回の間、`owned_leaves` は変わらない。
  `<1>1` より `insert` は 1 度も真を返さない。`insert` が偽を返すとき集合は変わらない。
  BY <1>1, CODE src/rc_ir/borrow.rs: infer_ownership

<1>3. その周回で各関数について `collect_consumes` に渡される `own` は、返される最終の `owned_leaves` と
      同じである。
  BY <1>2

<1>4. QED
  その周回の内側のループは、`vars.param_tys` が `root_var` を持つ各 `(root_var, root_path)` について
  `owned_leaves.insert` を呼ぶ。`<1>1` よりそのすべてが偽を返すので、すべて既に入っている。`<1>3` より
  その `collect_consumes` の入力は最終の `owned_leaves` である。
  BY <1>1, <1>3, CODE src/rc_ir/borrow.rs: infer_ownership

**P8 (c)**。H1 の下で、DEF 推論の割り当て を D14 の割り当てとして読んだとき、入力の各関数 `f` の
どの実行路のどの位置についても、そこで D9 の意味で消費される各 leaf `(v, λ)` について、
`origin(vars, type_env, &v, &λ).candidates()` の元のうち `vars.param_tys` に載るものはすべて
`owned_leaves` に入っている。

<1>1. H1 の下で、`owned_leaves` は DEF 推論の割り当て についての DEF leaf 粒度の所有 である。
  BY H1, L8

<1>2. DEF 推論の割り当て を D14 の割り当てとして読んだとき、D9 の消費の表の各行が指す leaf は、
      `own = owned_leaves` で呼んだ `collect_consumes` が `out` に積む。
  BY P7, <1>1

<1>3. QED
  `<1>2` の積まれた元に P8 (b) の閉包条件を適用する。
  BY P8 (b), <1>2

**系 (`README.md` の P8 の文面へ渡る段)**。H1 の下で、D9 の意味で消費される leaf `(v, λ)` について、
`origin(vars, type_env, &v, &λ).candidates()` の元であってパラメータ leaf であるものは、その参照を
`(v, λ)` のスロットと共有し、かつ `owned_leaves` に入っている。

<1>1. 消費は D9 の消費の表のある行が指す leaf `(v, λ)` について起きる。
  BY D9

<1>2. P3 と P4 より、`(v, λ)` のスロットが持つ参照は、`origin(v, λ)` の `candidates` のいずれかの下の
      対応するスロット (D17) が持つ参照と同一である。`λ` は leaf なので、`λ` の下の leaf は `λ` 自身だけで
      あり、対応するスロットはその候補 `(root, ρ)` そのものである。
  BY D17, P3, P4

<1>3. QED
  `<1>2` の候補のうち `vars.param_tys` に載るものは、P8 (c) より `owned_leaves` に入っている。
  BY P8 (c), <1>2

**この系は P3 と P4 に依る。** `README.md` 第 7 節が述べるとおり P3 と P4 はこのコミットでは閉じないので、
系もそこまでしか進まない。P8 (a)、P8 (b)、P8 (c) は P3 と P4 に依らない。

**`README.md` の P8 の文面との差**。P8 は「あるパラメータ leaf の参照が消費されるならば、その leaf は
`owned_leaves` に入っている」と書く。系が与えるのは「消費される leaf の `origin` の候補であるパラメータ
leaf は `owned_leaves` に入っている」である。この 2 つが同じでないのは、D8 より同じオブジェクトへの参照が
互いに区別されないからである。2 つのパラメータ leaf が同じ参照を持つ位置では、消費されたのがどちらの
leaf の参照かは決まらず、候補でない側について P8 の文面は主張できない。P14 の帰納法が要るのは系の形
(候補について述べる形) であり、そこでは `Obl` がオブジェクトごとの多重集合なので、候補でない側は要らない。

## 6. P9 -- 複製は名前替えである

### 6.1 前半 -- 本体は束縛変数を一斉に付け替えたものである

**言明**。`clone_func(func, new_ref, rename_counter)` が返す `RcFunc` の `body` は、`func.body` の各節点を
同じ種類・同じ並びの節点に写し、`FieldPath`・`RcState`・`source`・`MatchArm` の `tag` と `payload_state`・
`RcRhs::Closure` の `FuncRef` を変えず、変数の出現だけを 1 つの写像 `rename_f` で置き換えたものである。

<1>1. `clone_func` は `fresh_rename_function(&func.params, &func.capture, &func.body, "b", rename_counter)`
      を呼び、返った `params`、`capture`、`body` を `RcFunc` に入れ、`rename` を返す。`fn_ty`、`ret_ty`、
      `source`、`inline_into_callers` は `func` から写し、`name` は `new_ref`、`borrowed_units` は空である。
  BY CODE src/rc_ir/borrow.rs: clone_func

<1>2. `fresh_rename_function` は `renaming` を組み立てたうえで、`params` を `rename_var` で、`cap` を
      `rename_var` で、`body` を `rename_expr(body, &renaming)` で写す。
  BY CODE src/rc_ir/rename.rs: fresh_rename_function

<1>3. `renaming` の定義域は、`func` のパラメータ・capture の名前と、`func.body` が束縛する名前の全体である。
  <2>1. `fresh_rename_function` は `params.iter().chain(cap.iter())` の各元について `assign_fresh_name` を
        呼び、次に `assign_fresh_names_to_binders(body, ..)` を呼ぶ。`renaming` に元を入れるのは
        `assign_fresh_name` の `renaming.insert` だけである。
    BY CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name
  <2>2. `assign_fresh_names_to_binders_inner` は、`RcExpr::Let(x, rhs, k)` の `x`、`rhs` が
        `RcRhs::Match(_, arms)` のときの各 `arm.payload`、`RcExpr::Destructure(_, fields, ..)` の各
        フィールド変数について `assign_fresh_name` を呼び、`k` と各 `arm.body` へ降りる。
        `RcExpr::Retain | Release | Eval` は継続へ降り、`RcExpr::Ret` は降りない。
    BY CODE src/rc_ir/rename.rs: assign_fresh_names_to_binders_inner
  <2>3. QED
    `<2>2` の 3 種は DEF 入力の束縛名 が数える束縛子の全体である。
    BY DEF 入力の束縛名, <2>1, <2>2, CODE src/rc_ir/ast.rs: RcExpr, MatchArm

<1>4. `renaming` は写像であり、定義域の各名前に像を 1 つだけ持つ。
  `assign_fresh_name` は `renaming.insert` の返り値が `None` であることを表明するので、1 つの名前について
  2 度目の登録は起きない。
  BY CODE src/rc_ir/rename.rs: assign_fresh_name

<1>5. `rename_expr_inner` は `RcExpr` の 6 種のそれぞれを同じ種の節点に写し、`FieldPath`・`RcState`・
      `source` をそのまま写し、`RcVar` の出現を `rename_var` で写す。`rename_rhs` は `RcRhs` の 5 種の
      それぞれを同じ種に写し、`RcRhs::Closure` の `FuncRef` をそのまま写し、`MatchArm` の `tag` と
      `payload_state` をそのまま写し、`RcRhs::Llvm` については `llvm_gen` を clone して
      `free_vars_mut` の各スロットを `renaming` で写す。
  BY CODE src/rc_ir/rename.rs: rename_expr_inner, rename_rhs, rename_var

<1>6. `rename_var` は `renaming` に鍵を持たない名前をそのまま残す。
  BY CODE src/rc_ir/rename.rs: rename_var

<1>7. QED
  `<1>5` と `<1>6` より、写された本体は元の本体と同じ形の木であり、変わるのは `renaming` の定義域にある
  名前の出現だけである。`<1>3` よりその定義域は束縛名の全体であり、`<1>4` より写像である。A6 と A11 より、
  定義域にある名前の出現はすべて、その名前を束縛する `func` の束縛子に解決する出現である。よって置き換えは
  一斉の名前替えである。
  BY A6, A11, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### 6.2 後半 -- 導入する名前は入力の束縛名と異なる

**言明**。H3 の下で、`borrow_ify` の実行中に `clone_func` が導入する名前は、どの入力の束縛名とも異なる。

<1>1. `clone_func` が導入する名前は、`assign_fresh_name(&name, "b", &mut renaming, counter)` が作る
      `FullName` であり、その `namespace` は `name.namespace` のまま、`name` フィールドは
      `format!("{}#{}{}", name.name, "b", counter)` である。`counter` は使用の直前に 1 増やされるので
      1 以上である。
  BY CODE src/rc_ir/borrow.rs: clone_func, CODE src/rc_ir/rename.rs: assign_fresh_name

<1>2. `<1>1` の `name` フィールドを `#` で区切った最後の断片は、`b` の後に 10 進数字が 1 個以上続く形で
      ある。
  `counter` の 10 進表記は 10 進数字だけからなり `#` を含まないので、追加された `#` が最後の `#` である。
  BY <1>1

<1>3. 入力の束縛名の `name` フィールドを `#` で区切った最後の断片は、`<1>2` の形ではない。
  BY H3

<1>4. QED
  `FullName` の相等は `namespace` と `name` で決まるので、`name` フィールドが異なれば名前は異なる。
  BY <1>2, <1>3, CODE src/ast/name.rs: FullName

### 6.3 系 -- 出力の束縛名は互いに相異なる

**言明**。H3 の下で、`borrow_ify` の出力の束縛名 (DEF 出力の束縛名) は互いに相異なる。

<1>1. `clone_func` が導入する 2 つの名前は、`counter` の値が異なれば異なる。
  <2>1. `M1 ++ "#b" ++ dec(c1) = M2 ++ "#b" ++ dec(c2)` を仮定する。`"b" ++ dec(c)` は `#` を含まないので、
        両辺の最後の `#` は追加された `#` である。よって `M1 = M2` かつ `dec(c1) = dec(c2)`、すなわち
        `c1 = c2` である。
    BY 6.2 の <1>1
  <2>2. `rename_counter` は `borrow_ify` の中で 1 つだけ作られ、`clones` を作るループを通じて
        `clone_func` に渡され、`assign_fresh_name` の呼び出しごとに 1 増える。よって 1 回の `borrow_ify` の
        実行の中で同じ値が 2 度使われることはない。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, clone_func, CODE src/rc_ir/rename.rs: assign_fresh_name
  <2>3. QED
    BY <2>1, <2>2

<1>2. 出力の各 `f_own` の束縛名は、対応する入力の関数の束縛名と同じである。
  `f_own` は `func.clone()` の `body` を `ctx.rewrite(&f_own.body)` に差し替えたものであり、`params` と
  `capture` は `func` のままである。L5 より `rewrite` は本体の束縛名を変えない。
  BY L5, CODE src/rc_ir/borrow.rs: borrow_ify

<1>3. 出力の各 `f_borrow` の束縛名は、`clone_func` が導入した名前の全体である。
  6.1 の `<1>3` と `<1>7` より `clone_func` の出力の束縛名は `rename_f` の像であり、その後の
  `ctx.rewrite(&clone.body)` は L5 より束縛名を変えない。
  BY L5, 6.1 の <1>3, 6.1 の <1>7, CODE src/rc_ir/borrow.rs: borrow_ify

<1>5. 出力のグローバル初期化子の束縛名は、入力のグローバル初期化子の束縛名と同じである。
  `borrow_ify` はグローバル初期化子の `init` を `ctx.rewrite(&g.init)` に差し替えるだけであり、L5 より
  `rewrite` は束縛名を変えない。
  BY L5, CODE src/rc_ir/borrow.rs: borrow_ify

<1>6. QED
  出力の束縛名は、`<1>2` のもの、`<1>3` のもの、`<1>5` のものの 3 つに分かれる。`<1>2` と `<1>5` の
  名前はどれも入力の束縛名であり、A6 より互いに相異なる。`<1>3` の名前どうしは `<1>1` より相異なる。
  `<1>2`・`<1>5` の側と `<1>3` の側は 6.2 の言明より相異なる。
  BY A6, 6.2 の言明, <1>1, <1>2, <1>3, <1>5

## 7. P10 -- 借用版が落とす RC 節点

**言明**。`is_borrow_version` が真の `RewriteCtx` を `ctx` とする。`ctx.rewrite` は、`Retain(v, π, s, k)` を
次の節点に写す。`units_under(ty(v), π, type_env)` の元のうち `ctx.owns_unit(v, ・)` が真であるものを、
`units_under` が返す並びの順に `u_1, ..., u_r` とする。写る先は

```
Retain(v, u_1, s, Retain(v, u_2, s, ... Retain(v, u_r, s, ctx.rewrite(k)) ... ))
```

であり、`r = 0` のときは `ctx.rewrite(k)` そのものである。`Release(v, π, s, k)` については同じ並びで
`Release` の列になる。`ctx.owns_unit(v, ・)` が偽である unit についての節点は、この写像の像に現れない。

<1>1. `ctx.rewrite(node)` は `ctx.rewrite_inner(node)` の値である。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite, CODE src/misc.rs: grow_stack

<1>2. `rewrite_inner` の `RcExpr::Retain(v, path, state, k)` の腕は
      `self.rewrite_rc(v, path, *state, false, k, &node.source)` を、`RcExpr::Release` の腕は
      同じ引数で `is_release` を `true` にしたものを返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

<1>3. `rewrite_rc` はまず `let k = self.rewrite(k);` を行う。`self.is_borrow_version` が真なので、
      `rc_node(is_release, v.clone(), path.clone(), state, k, source)` を返す腕は通らない。
  BY <1>2, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>4. `kept` は `units_under(&v.ty, path, self.type_env)` を `self.owns_unit(v, unit)` で絞ったものであり、
      `Iterator::filter` は元の並びを保つので、`kept` は `u_1, ..., u_r` である。
  BY <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>5. `kept.into_iter().rev().fold(k, |cont, unit| rc_node(is_release, v.clone(), unit, state, cont, source))`
      は、`u_r` を最も内側に、`u_1` を最も外側に置いた節点の鎖を返す。`r = 0` のときは `k` を返す。
  `rev()` により fold は `u_r` から始まり、各段が直前の結果を継続として包む。
  BY <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>6. `rc_node(is_release, var, path, state, k, source)` は、`is_release` が真なら
      `RcExpr::Release(var, path, state, k)` を、偽なら `RcExpr::Retain(var, path, state, k)` を作る。
  BY CODE src/rc_ir/borrow.rs: rc_node

<1>7. QED
  `<1>1`-`<1>6` が言明の形を与える。`<1>4` の `filter` が落とした unit について `rewrite_rc` は
  `rc_node` を呼ばないので、その unit の節点は像に現れない。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

**注**。L3 はこの命題の `π ∈ units(ty(v))` の場合である。A2 より入力のすべての `Retain`/`Release` の path は
その場合に当たるので、`r` は 0 か 1 であり、`r = 1` のときの unit は `π` 自身である (L1)。

## 8. P11 -- 呼び出し側の補正

**言明**。`RewriteCtx` を `ctx` とし、`ctx.rewrite` が `Let(x, App(callee, args), k)` に何を返すかを述べる。
`callee' = ctx.route(x, callee, args, k)`、`params = ctx.callee_params.get(&FuncRef { name: callee'.name })`
とする。各引数の添字 `i` と各 `u ∈ units(ty(args[i]))` について、

- `callee_owns(i, u)` を、`params` が `None` のとき真、`Some(ps)` のとき
  `ctx.owned_units.contains(&(ps[i].0, u))` と定める。
- `arg_owned(i, u)` を `ctx.owns_unit(&args[i], &u)` と定める。

`before` は `callee_owns(i, u)` が真かつ `arg_owned(i, u)` が偽である対 `(args[i], u)` を、`i` の昇順・
`units(ty(args[i]))` の並び順に並べた列、`after` は `callee_owns(i, u)` が偽かつ `arg_owned(i, u)` が真で
ある対を同じ順に並べた列である。このとき `ctx.rewrite` が返すのは

```
Retain(before_1) ... Retain(before_q)
Let(x, App(callee', args),
  Release(after_1) ... Release(after_t)
  ctx.rewrite(k))
```

である。ここで `before_j` と `after_j` は対 `(a, u)` であり、置かれる節点はそれぞれ
`Retain(a, u, RcState::Unknown, ..)` と `Release(a, u, RcState::Unknown, ..)` で、source span は `None` で
ある。`callee_owns` と `arg_owned` が一致する対については、節点が 1 つも置かれない。

<1>1. `rewrite_inner` の `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕は、`callee` を
      `self.route(x, callee, args, k)` に替え、`(before, after) = self.call_rc(&callee, args)` をとり、
      `prepend_rc(before, false, expr_node(RcExpr::Let(x.clone(), RcRhs::App(callee, args.clone()),
      prepend_rc(after, true, self.rewrite(k))), &node.source))` を返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

<1>2. `prepend_rc(units, is_release, k)` は `units` の第 1 元を最も外側に、最後の元を最も内側に置いた
      `rc_node` の鎖を返し、`units` が空のときは `k` を返す。置く節点の `RcState` は `RcState::Unknown`、
      source span は `None` である。
  `units.into_iter().rev().fold(k, ..)` は最後の元から包み始め、`rc_node` に `RcState::Unknown` と `&None` を
  渡す。
  BY CODE src/rc_ir/borrow.rs: prepend_rc

<1>3. `<1>1` の第 2 引数より、`before` の節点は `is_release` が偽なので `Retain`、`after` の節点は真なので
      `Release` である。
  BY <1>1, <1>2, CODE src/rc_ir/borrow.rs: rc_node

<1>4. `call_rc(callee, args)` は `params = self.callee_params.get(&FuncRef { name: callee.name.clone() })` を
      とり、`args` を添字つきで、各 `arg` について `rc_units(&arg.ty, self.type_env)` を順に回る。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>5. その内側で `callee_owns` は `params` が `None` のとき `true`、`Some(params)` のとき
      `self.owned_units.contains(&(params[arg_idx].0.clone(), unit.clone()))` であり、`arg_owned` は
      `self.owns_unit(arg, &unit)` である。
  BY <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>6. `!callee_owns && arg_owned` のとき `after.push((arg.clone(), unit))`、
      `callee_owns && !arg_owned` のとき `before.push((arg.clone(), unit))`、それ以外のとき何も積まない。
      `if`/`else if` の 2 分岐であり、どちらの条件も満たさない対は素通りする。
  BY <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>7. QED
  `<1>4` の 2 重ループの順序が `before` と `after` の並びを決め、`<1>6` がその中身を決める。`<1>1`-`<1>3` が
  節点の位置と種類を決める。`call_rc` に渡されるのは振り分け後の `callee` なので、`params` は
  `callee'` について引かれる。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

**`params[arg_idx]` が範囲内であること**。`call_rc` は `args.len() <= params.len()` を前提にしている
(`CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc` のコメント)。`callee_params` は
`param_names_and_types(func)` すなわち `func.params` に `func.capture` を鎖にしたものである
(`CODE src/rc_ir/borrow.rs: param_names_and_types`)。この前提を果たすのは lowering であり、`README.md` の
仮定にはこれを述べるものが無い (第 13 節)。

**「ちょうど埋める」について**。P11 の第 1 文の「呼び出し元と呼び出し先の所有権の食い違いをちょうど埋める」を
第 2 文の言い換えとして読むなら、上の言明がそれである。実行時の義務集合 (D10) についての主張として読むなら、
すなわち「補正の後、呼び出しの前後で `Obl` の収支が合う」として読むなら、H2 が要る。`Retain(a, u)` は
D10 より `u` の下の inhabited な**すべての** leaf の参照を 1 つずつ作り、`Release(a, u)` はすべてを 1 つずつ
処分するので、`arg_owned` の 1 つの真偽値が `u` の下の leaf ごとの所有と食い違うと、作る参照の個数と
不足している参照の個数がずれる。第 11.2 節がこの読み方を扱う。

## 9. P12 -- 振り分けの安全性

**P12 (a)**。`route(x, callee, args, k)` が `callee` と異なる名前の `RcVar` を返すのは、
`self.borrow_versions` が `FuncRef { name: callee.name }` を鍵に持ち、かつ `self.routing_is_safe(x, args)` と
`self.routing_saves_retain(borrow_version, args, k)` がともに真であるときだけである。そのとき返る名前は
`self.borrow_versions[&FuncRef { name: callee.name }].name` である。

<1>1. `route` は `orig = FuncRef { name: callee.name.clone() }` をとり、
      `self.borrow_versions.get(&orig)` が `Some(borrow_version)` で、かつ
      `self.routing_is_safe(x, args) && self.routing_saves_retain(borrow_version, args, k)` が真のときに
      `callee` の複製の `name` を `borrow_version.name` に替えて返し、それ以外のときは `callee.clone()` を
      返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::route

<1>2. QED
  BY <1>1

**P12 (b)**。`routing_is_safe(x, args)` が真であるのは、`x.name` が `self.tail` に入らないとき (末尾位置の
呼び出しでないとき) か、`args` のどの元 `a` についても `self.any_owned_unit(a)` が偽であるとき
(所有する unit を持つ引数を 1 つも持たないとき) である。

<1>1. `routing_is_safe` は `!self.tail.contains(&x.name) || !args.iter().any(|a| self.any_owned_unit(a))`
      である。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::routing_is_safe

<1>2. `self.tail` は `tail_result_vars(&func.body)` であり、`mark_tail` が末尾位置の `App` と `Match` の
      束縛変数を集めたものである。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, tail_result_vars, mark_tail

<1>3. `any_owned_unit(arg)` は `rc_units(&arg.ty, self.type_env)` のいずれかの `unit` について
      `self.owns_unit(arg, unit)` が真であることである。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::any_owned_unit

<1>4. QED
  BY <1>1, <1>2, <1>3

**P12 (c)**。`route` が返す `RcVar` の名前は、`callee.name` そのものか、`borrow_versions` のある値の
`name` である。後者のとき、その名前は出力の `funcs` の鍵である。前者のとき、`callee.name` が入力の
`funcs` の鍵であるならば、それは出力の `funcs` の鍵でもある。

<1>1. `route` が返すのは `callee.clone()` か、`callee` の複製の `name` を
      `borrow_versions[&orig].name` に替えたものである。
  BY P12 (a)

<1>2. `borrow_versions` の値はすべて出力の `funcs` の鍵である。
  BY L6

<1>3. 入力の `funcs` の各鍵は出力の `funcs` の鍵である。
  BY L6

<1>4. QED
  BY <1>1, <1>2, <1>3

**「同じ関数の版である」について**。`borrow_versions` の鍵 `orig` に対する値は
`borrow_funcref(&func.name)` であり、`func.name` の `name` フィールドに `#borrow` を継ぎ足したものである
(`CODE src/rc_ir/borrow.rs: borrow_funcref`)。よって振り分け先は、`orig` の借用版として作られた版である。
呼び出し先が入力の関数を名指していないとき (局所変数を経由する間接呼び出しのとき) は、`borrow_versions` が
その名前を鍵に持たないので `route` は `callee` をそのまま返す。`README.md` の P12 の「`prog.funcs` に
存在する」は、この場合を除いて読む必要がある (第 13 節)。

## 10. P13 -- 注釈の一致

**言明**。出力の各版 `V` について、`V.borrowed_units` は
`param_capture_units(V, type_env)` の元のうち `owned_units` に入らないものの集合である。

<1>1. `borrow_ify` は `funcs` を組み立てた後、`for func in funcs.values_mut()` のループで
      `func.borrowed_units = param_capture_units(func, type_env).into_iter()
      .filter(|unit_path| !owned_units.contains(unit_path)).collect();` を実行する。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. このループは出力の `funcs` の全元を回り、`borrowed_units` に書き込むのはここだけである。
      `f_own` は `func.clone()` の `body` を差し替えたもの、`f_borrow` は `clone_func` が
      `borrowed_units` を空にして作ったものであり、どちらもこのループで上書きされる。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>3. QED
  BY <1>1, <1>2

**系 1 (`f_own` は何も借用しない)**。出力の各 `f_own` の `borrowed_units` は空である。

<1>1. `borrow_ify` の 2 番目のループは入力の各関数について
      `owned_units.extend(param_capture_units(func, type_env))` を実行する。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `f_own` の `params` と `capture` は入力の `func` のものと同じなので、
      `param_capture_units(f_own, type_env) = param_capture_units(func, type_env)` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units

<1>3. QED
  `<1>1` よりその全元が `owned_units` にあるので、P13 の `filter` は何も残さない。
  BY P13, <1>1, <1>2

**系 2 (`f_borrow` が借用する unit)**。H3 の下で、`borrow_versions` に載る入力の関数 `f` の各パラメータ `p` と
各 `u ∈ units(ty(p))` について、`f_borrow.borrowed_units` が `(rename_f[p.name], u)` を含むことは、
`f` が `(p, u)` を推論の意味で借用すること (DEF 推論の割り当て) と同値である。

<1>1. `f_borrow` の `params` は `rename_var(p, rename_f)` の列であり、`ty` は変わらない。`f_borrow` の
      `capture` は `None` である。
  `rename_var` は `RcVar` を複製して `name` だけを写す。`borrow_versions` に載るのは
  `func.capture.is_none()` の関数だけであり、`rename_var` は `None` を `None` に写す。
  BY CODE src/rc_ir/rename.rs: rename_var, fresh_rename_function, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. よって `param_capture_units(f_borrow, type_env)` は
      `{(rename_f[p.name], u) : p ∈ f.params, u ∈ units(ty(p))}` である。
  BY <1>1, CODE src/rc_ir/borrow.rs: param_capture_units

<1>3. `f` が `(p, u)` を推論の意味で所有するならば、`(rename_f[p.name], u) ∈ owned_units` である。
  DEF 推論の割り当て より `truncate_to_unit(ty(p), λ, type_env) = u` かつ `owned_leaves.owns(p.name, λ)` が
  真である `λ` があり、L2 の (b) がその `λ` について `(rename_f[p.name], u)` を入れる。
  BY DEF 推論の割り当て, L2

<1>4. `(rename_f[p.name], u) ∈ owned_units` ならば、`f` は `(p, u)` を推論の意味で所有する。
  <2>1. L2 より `owned_units` の元は (a) 入力の関数のパラメータ・capture の名前を第 1 成分に持つものか、
        (b) ある `borrow_versions` に載る関数 `g` のあるパラメータ `q` について
        `(rename_g[q.name], truncate_to_unit(ty(q), λ, type_env))` (`λ` は `owned_leaves.owns(q.name, λ)` が
        真である leaf) のどちらかである。
    BY L2
  <2>2. `rename_f[p.name]` は入力の束縛名ではないので、(a) の形ではありえない。
    BY 6.2 の言明
  <2>3. (b) の形であるとき、`rename_g[q.name] = rename_f[p.name]` である。`assign_fresh_name` は束縛子
        ごとに 1 度だけ呼ばれ (6.1 の `<1>4`)、`rename_counter` の値は 1 回の `borrow_ify` の実行の中で
        使い回されない (6.3 の `<1>1`) ので、相異なる束縛子には相異なる名前が付く。よって `g = f` かつ
        `q = p` である。
    BY 6.1 の <1>4, 6.3 の <1>1
  <2>4. QED
    `<2>3` より `u = truncate_to_unit(ty(p), λ, type_env)` かつ `owned_leaves.owns(p.name, λ)` が真である
    `λ` がある。これは DEF 推論の割り当て が要求するものである。
    BY DEF 推論の割り当て, <2>1, <2>2, <2>3

<1>5. QED
  P13 と `<1>2` より `f_borrow.borrowed_units` は `<1>2` の集合から `owned_units` の元を除いたものであり、
  `<1>3` と `<1>4` よりその除かれる元は推論の意味で所有される `(p, u)` にちょうど対応する。
  BY P13, <1>2, <1>3, <1>4

**系 2 は P9 の後半 (6.2) を使う。** `owned_units` は変数名で引く集合なので、複製が導入した名前が入力の
束縛名と衝突すると、(a) の元が (b) の元として読まれ、借用版が所有していない unit を所有していることに
なる。

## 11. H1 は何に足りて何に足りないか

### 11.1 H1 は `R2` を排除しない

**言明**。第 3 節の入力プログラム `prog` について H1 は成り立つ。

<1>1. `prog` の関数は `f` だけであり、そのパラメータは `x : A` と `y : A`、capture は `None` である。
  BY DEF f

<1>2. `units(A) = {[]}` であり、`leaves(A) = {[]}` である。よって `A` の唯一の unit の下の leaf は
      1 つだけである。
  BY 3.2 の <1>1

<1>3. QED
  `<1>1` より `prog` のパラメータ・capture は `x : A` と `y : A` の 2 つだけであり、`<1>2` よりそれぞれの
  唯一の unit の下の leaf は 1 つだけである。1 元の集合の上では `owns` の値は 1 つしかない。
  BY H1 の言明, <1>1, <1>2

**帰結**。`3.7 の <1>8` より `prog` は P14 の反例である。よって **H1 を成り立たせても P14 は閉じない**。
H1 が縛るのはパラメータの unit の下の leaf の所有であり、`R2` が壊すのは local な値 (`a' : U`) の unit の
下の leaf の所有だからである。`R2` の `a'` の 2 つの leaf は、別々のパラメータ (`x'` と `y'`) に由来する。
どちらのパラメータも unit を 1 つ・leaf を 1 つしか持たないので、H1 は何も言わない。

### 11.2 P14 が要求するもの -- H2

第 14 節が、P14 の帰納法が止まる場所を述べる。要るのは次の補題である。

> `owns_unit(v, u)` が偽ならば、`(v, u)` の下の inhabited な leaf の参照は `Obl_borrow` に 1 つも無く、
> `owns_unit(v, u)` が真ならば、`(v, u)` の下の inhabited な各 leaf の参照が `Obl_borrow` に 1 つずつある。

これが H2 である。H2 が P10 と P11 のどこで要るかは次のとおりである。

- **P10 (落とす `Release`)**。`owns_unit(v, u)` が偽なので節点が落ち、`Obl_borrow` から何も取り除かれない。
  入力の `Release` は D10 より `u` の下の inhabited な**すべての** leaf の参照を 1 つずつ取り除いていたので、
  借用版がその leaf の参照を 1 つでも持つなら、その参照は残る。`R2` ではそれが (S-b) の違反になる
  (`3.7 の <1>5`)。
- **P10 (残す `Release`)**。`owns_unit(v, u)` が真なので節点が残り、D10 より `u` の下の inhabited な各 leaf の
  参照が 1 つずつ取り除かれる。`Obl_borrow` がそのうちの 1 つでも持っていなければ (S-a) が破れる。
- **P11 (前に置く `Retain`)**。`callee_owns` が真で `arg_owned` が偽なので `Retain(a, u)` が置かれる。この
  `Retain` は D10 より `u` の下の inhabited な**すべての** leaf の参照を 1 つずつ作り、呼び出しはそのすべてを
  1 つずつ消費する。呼び出しの前に `Obl_borrow` がそのうちのどれかを既に持っていれば、その参照は消費されずに
  残る。
- **P11 (後に置く `Release`)**。`callee_owns` が偽で `arg_owned` が真なので `Release(a, u)` が置かれ、
  呼び出しは何も消費しない。`Obl_borrow` が `u` の下の inhabited な各 leaf の参照を 1 つずつ持っていなければ
  (S-a) が破れる。

**H1 と H2 の役割の違い**。第 11.2 節の 4 か所は、`owns_unit(v, u)` の 1 つの真偽値と `u` の下の leaf ごとの
所有との一致だけを使う。すなわち H2 だけで閉じ、H1 を使わない。H1 が要るのは P8 の言明を、D9 の `App` の行の
unit 粒度の読みで真にするためである (第 5.2 節の P8 (c))。この 2 つは独立である -- `R2` は H1 を満たして
H2 を破り、パラメータの union の一方の変位の payload だけを消費する関数は H1 を破って H2 を満たす。

**H2 を成り立たせる直し方**。`owns_unit(v, u)` は `origin(v, u).candidates()` の全元について
`owns_object` を要求する。`u` の下の各 leaf の所有もまた、その leaf の `origin` が指す root の
`owns_object` で決まる。この 2 つがそろうには、1 つの unit に関わる root の所有がそろっていればよい。
すなわち `infer_ownership` の不動点に次の閉包を足す。

> ある関数の本体のある `Retain`/`Release` 節点の各 `u ∈ units_under(ty(v), π)`、およびある `App` の各引数
> `a` の各 `u ∈ units(ty(a))` について、`origin(v, u).candidates()` (`origin(a, u).candidates()`) のいずれかの
> 元が `owns_object` を満たすならば、その候補の全元を所有にする。

`owns_object` は `param_tys` に載らない root について無条件に真を返す (L4) ので、この閉包は「候補に
producer が 1 つでも混ざればパラメータの候補はすべて所有になる」を含む。

**この閉包は借用の機会を減らしすぎない**。`origin(v, u)` が 2 つ以上の候補を持つのは、`Binding::Join`
(match の束縛) か `origin_from_leaves_under` (unit の下の leaf が別々の operand へ届く場合) のどちらかで
ある (`CODE src/rc_ir/ownership.rs: origin_inner`, `origin_from_leaves_under`)。どちらの場合も、その unit の
1 つの参照カウント操作は複数のオブジェクトの参照に同時に作用する。そのうち一部だけを借用する版は、
`Retain`/`Release` の節点で表せない。よってこの閉包が取り除くのは、表せなかった借用だけである。

**足りない命題**。上の議論は「`u` の下の各 leaf の `origin` が指す root が、`origin(v, u).candidates()` に
含まれる」ことを使う。`README.md` の P1 から P7 にこれを述べるものは無い。
`p12-keys-and-consumes.md` の末尾も同じ穴を「unit path の `origin` と、その下の leaf の `origin` の関係を
述べる命題が要る」として記録している。この命題が入るまで、H2 を成り立たせる直し方が本当に H2 を
成り立たせるかは決まらない。

### 11.3 H1 の閉包は借用の機会を減らさない

H1 を成り立たせる直し方は、`infer_ownership` の不動点に「あるパラメータ unit の leaf が 1 つでも所有なら
その unit の下の全 leaf を所有にする」という閉包を足すことである。この閉包の代償を数える。

<1>1. 与えられた `owned_leaves` に対して、この閉包は `owned_units` を変えない。
  L2 の (b) は `owned_leaves` の各元 `λ` を `truncate_to_unit(ty(p), λ, type_env)` へ写して入れるので、
  同じ unit の下の別の leaf を足しても入る元は同じである。
  BY L2

<1>2. この閉包が変えるのは不動点そのものである。`collect_consumes` の `owns` は leaf 粒度の集合への所属
      なので、閉包が leaf を足すと `App` の引数位置でより多くの leaf が消費として報告される。
  BY CODE src/rc_ir/ownership.rs: collect_consumes, rhs_consumes の `RcRhs::App` の腕

<1>3. `<1>2` で新たに報告される leaf は、DEF 推論の割り当て を D14 の割り当てとして読んだ D9 の `App` の行が
      指す leaf である。
  新たに報告されるのは、呼び出し先のパラメータ leaf `(q, λ)` が閉包によって所有になった位置の引数 leaf で
  ある。閉包が `(q, λ)` を足すのは、同じ unit の下の別の leaf `(q, λ')` が既に所有のときであり、`<1>1` より
  そのとき `q` の unit `truncate_to_unit(ty(q), λ)` は推論の割り当ての意味で所有される。D9 の `App` の行は
  unit 粒度なので、その位置は `λ` に対応する引数 leaf を消費する。
  BY D9, D14, DEF 推論の割り当て, <1>1

<1>4. `func_has_borrowable_param` の答えが真から偽へ倒れるのは、その関数の借用可能な leaf がすべて
      「同じ unit の別の leaf が所有されている」ものだった場合に限る。そのとき、その関数の借用版は
      どの呼び出しからも振り分けられない。
  <2>1. その場合、閉包の後は `func.params` のすべての leaf が所有なので、`<1>1` よりその関数のすべての
        パラメータ unit が `owned_units` に入る。
    BY <1>1, CODE src/rc_ir/borrow.rs: func_has_borrowable_param
  <2>2. `routing_saves_retain` は `callee_borrows = !self.owned_units.contains(&(borrow_params[arg_idx].0,
        unit))` がどの引数のどの unit についても偽なら、`any` が偽になり偽を返す。
    BY <2>1, CODE src/rc_ir/borrow.rs: RewriteCtx::routing_saves_retain
  <2>3. QED
    P12 (a) より、`routing_saves_retain` が偽なら `route` は借用版を返さない。
    BY P12 (a), <2>1, <2>2

<1>5. QED
  `<1>3` より、閉包が取り除く借用は、unit 粒度の D9 で実際に消費される leaf についての借用である。`<1>4` より、
  閉包が取り除く借用版は、どの呼び出しの振り分け先にもならない版である。よって閉包は、成り立っていなかった
  借用と、使われなかった版だけを取り除く。
  BY <1>3, <1>4

### 11.4 3 つの問いへの答え

- **P8-P13 のうち H1 が要るのはどれか**。P8 だけである。P8 の言明が読む D9 の `App` の行は unit 粒度で
  述べられ、`infer_ownership` が `collect_consumes` に渡す `own` は leaf 粒度なので、この 2 つを橋渡し
  するのに H1 が要る (L8)。P9・P10・P11・P12・P13 は H1 を使わない。P9 の後半は代わりに H3 を使う。
  さらに、H1 が要るのは P8 の言明を真にするためであって、P10 と P11 が支えるべき実行時の収支は H2 だけで
  決まる (第 11.2 節)。
- **H1 は十分か**。十分でない。第 11.1 節より `R2` は H1 を満たす。P14 が要るのは H2 (第 11.2 節) であり、
  H2 を成り立たせるには `origin` の候補の所有をそろえる閉包が要る。H1 の閉包 (パラメータ unit の leaf を
  そろえる) はこれを含まない。
- **H1 は強すぎないか**。強すぎない。第 11.3 節より、H1 の閉包は `owned_units` を直接には変えず、不動点で
  新たに所有になる leaf は unit 粒度の D9 で実際に消費される leaf であり、消える借用版はどの呼び出しの
  振り分け先にもならない版である。H2 の閉包についても同じことが言える (第 11.2 節)。

## 12. 観察

### 12.1 `infer_ownership` の `owns` は leaf 粒度、後段の読み手は unit 粒度

`collect_consumes` は `own` から `owns(p, λ) = own.contains(&(p.name, λ))` を作る
(`CODE src/rc_ir/ownership.rs: collect_consumes`)。`infer_ownership` が渡す `own` は `owned_leaves` そのもので
ある (`CODE src/rc_ir/borrow.rs: infer_ownership`)。

一方、`owned_leaves` を読んで `owned_units` を作るとき、`borrow_ify` は leaf を `truncate_to_unit` で unit へ
写す (L2 の (b))。そして `borrowed_units` を経て `all_owned_units` が答えるのはその unit である
(`CODE src/rc_ir/ownership.rs: all_owned_units`)。`cancel` が `collect_consumes` と同じモデルを使うときの
`owns` は、`truncate_to_unit` を経た所属である
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs`)。D9 の `App` の行も unit 粒度で述べられている。

この 2 つは、1 つの unit の下に leaf が 2 つ以上あるとき (第 3.9 節より unbox union のとき) 食い違う。
`owned_leaves` がその unit の leaf の一部だけを持つと、`infer_ownership` はその位置を「消費しない」と読み、
`owned_units` を読む後段はすべて「消費する」と読む。P7 が `DEF leaf 粒度の所有` で 2 つを橋渡ししているのは
まさにこの点であり、`p12-keys-and-consumes.md` の「D9 の `App` の行と `collect_consumes` の粒度が違う」は
この橋渡しが不動点で成り立つことを P8 に委ねている。**その委ねられた条件が H1 である** (L8)。

反例 `R2` はこの食い違いを使っていない。`R2` の `f` には呼び出しが無く、`owned_leaves` の粒度は
`owned_units` の粒度と一致している。よってこの食い違いは `R2` とは独立であり、`R2` を直しても残る。
逆に、H1 を成り立たせてもこの食い違いが消えるだけで `R2` は残る (第 11.1 節)。

### 12.2 グローバル初期化子の `RewriteCtx`

`borrow_ify` はグローバル初期化子について `RewriteCtx` を直接組み立てる
(`CODE src/rc_ir/borrow.rs: borrow_ify` の `globals` を作る `map`)。`RewriteCtx::new` が置く値との対応は
次のとおりである。

| フィールド | `new` が置くもの | グローバルで置かれるもの |
|---|---|---|
| `type_env` | 引数 | 引数 |
| `is_borrow_version` | 引数 | `false` |
| `owned_units` | 引数 | 同じ `owned_units` |
| `borrow_versions` | 引数 | 同じ `borrow_versions` |
| `callee_params` | 引数 | 同じ `callee_params` |
| `tail` | `tail_result_vars(&func.body)` | `tail_result_vars(&g.init)` |
| `vars` | `VarTable::of(func)` | `VarTable::body_only(&g.init)` |

`VarTable::of` は `func.params` と `func.capture` を `Binding::Param` として登録し、そのうえで
`collect_bindings(&func.body, ..)` を呼ぶ。`VarTable::body_only` は `collect_bindings(body, ..)` だけを
呼ぶ (`CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only`)。D1 よりグローバル初期化子は
パラメータも capture も持たないので、`VarTable::of` の最初のループは空を回ることになり、2 つは同じ表を作る。
`tail` は同じ関数を初期化子の本体に適用したものである。`is_borrow_version` が `false` であることは、
`f_own` に対する `new` の呼び出しと一致する。食い違いは無い。

### 12.3 capture を持つ関数

`borrow_versions` に入るのは `func.capture.is_none()` の関数だけである
(`CODE src/rc_ir/borrow.rs: borrow_ify`)。`owned_units` へ (b) の元を入れるループは `func.params` だけを
回り、`func.capture` を回らない (L2 の (b))。この 2 つは対応している。`clone_func` は
`fresh_rename_function` に `&func.capture` を渡し、返った `capture` を `RcFunc` に入れるが
(`CODE src/rc_ir/borrow.rs: clone_func`)、`borrow_versions` に載る関数の `capture` は `None` なので、
借用版の `capture` も `None` である。`param_capture_units` は `params` と `capture` を鎖にして回るので、
借用版についてはパラメータだけを返す。`borrowed_units` に capture の unit が現れることは無い。

### 12.4 `infer_ownership` はグローバル初期化子を走査しない

`infer_ownership` の不動点は `prog.funcs.values()` だけを回り、`prog.globals` を回らない
(`CODE src/rc_ir/borrow.rs: infer_ownership`)。D1 よりグローバル初期化子はパラメータも capture も持たない
ので、その本体の消費が `origin` で遡り着く root は `vars.param_tys` に載らず、`insert` の条件が偽になる。
よって走査しても `owned_leaves` は変わらない。食い違いは無い。

### 12.5 `owned_leaves` には capture の leaf が入りうるが、読み手はいない

`infer_ownership` の `insert` の条件は `vars.param_tys.contains_key(root_var)` であり、`VarTable::of` は
`func.capture` も `param_tys` に登録する (`CODE src/rc_ir/ownership.rs: VarTable::of`)。よって
`owned_leaves` には capture の leaf が入りうる。一方、`owned_leaves.owns` を呼ぶのは
`func_has_borrowable_param` と `owned_units` を組み立てるループの 2 か所だけで、どちらも `func.params` しか
回らない (`CODE src/rc_ir/borrow.rs: borrow_ify, func_has_borrowable_param`)。`collect_consumes` の `owns` が
引くのは `resolve_callee_params` が返す `func.params` の元である
(`CODE src/rc_ir/ownership.rs: rhs_consumes, resolve_callee_params`)。よって capture の分は誰も読まない。
所有を増やす向きの情報なので安全側であり、代償は不動点の周回だけである。

## 13. `README.md` へ差し戻す点

### P14 が偽であること

第 3 節が反例である。P14 の言明は変えずに、コードを直すのが筋である。直し方の形は 2 つ考えられる。
どちらも H2 (第 11.2 節) を成り立たせることを狙う。

- **形 1**: 1 つの unit に関わる root の所有が分かれないようにする。`infer_ownership` の不動点に、
  「ある `Retain`/`Release` 節点または引数位置の unit `(v, u)` について `origin(v, u).candidates()` の
  いずれかが `owns_object` を満たすなら、その候補はすべて所有される」という閉包を足す。
  第 11.2 節が、この閉包が借用の機会を減らしすぎないことを述べる。
- **形 2**: 借用版を作る条件を狭める。`func_has_borrowable_param` に、その関数のどの `Retain`/`Release`
  節点・どの引数位置についても `origin(v, u).candidates()` の所有が分かれないこと、を足す。

**H1 (依頼が置いた「パラメータ unit の下の leaf の所有が一致する」) はどちらでもない。** 第 11.1 節より
`R2` は H1 を満たすので、H1 を成り立たせる直し方だけでは P14 は閉じない。

### unit path の `origin` と、その下の leaf の `origin` の関係を述べる命題が無い

第 11.2 節の直し方が H2 を成り立たせるかどうかは、「`u` の下の各 leaf の `origin` が指す root が
`origin(v, u).candidates()` に含まれる」に依る。`README.md` の P1 から P7 にこれを述べるものは無い。
`p12-keys-and-consumes.md` の末尾も、`check_one_key_per_object` の表明が発火しないことを示すのに同じ命題が
要ると記録している。P7 と P8 の間に置くのがよい。

### P8 の言明が読む所有権の割り当て

L7 (第 5.1 節) が、A1 の割り当て (全所有) で読むと P8 が偽になる入力プログラムを与える。よって P8 は
`infer_ownership` が計算している割り当て (DEF 推論の割り当て、第 5.1 節) の下で読むほかない。`README.md` が
どちらであるかを書くとよい。

### P8 の文面を `origin` の候補の形へ

P8 は「あるパラメータ leaf の参照が消費されるならば、その leaf は `owned_leaves` に入っている」と書く。
証明が与えるのは「消費される leaf の `origin` の候補であるパラメータ leaf は `owned_leaves` に入っている」で
ある (第 5.2 節の P8 (c) とその系)。D8 より同じオブジェクトへの参照は互いに区別されないので、2 つのパラメータ
leaf が同じ参照を持つ位置では、消費されたのがどちらの leaf の参照かは決まらない。P14 の帰納法が使うのは
候補の形なので、P8 の文面をその形にするのがよい。

### P9 の後半には入力の束縛名の形についての仮定が要る

`fresh_rename_function` は入力の名前を読まない。導入する名前 `<元の名前>#b<番号>` が入力の束縛名と
異なることは、入力の束縛名の形についての仮定 (H3、第 4 節) からしか出ない。この仮定を果たすコードも
表明も無い。`README.md` の A6 の隣に置くのがよい。文面の案は次のとおりである。

> **A13 (束縛名の形)** -- 果たす者: `Lowerer::fresh_var` と `clone_fresh`。検査: 無し。
> `borrow_ify` の入力のすべての束縛名について、その `name` フィールドを `#` で区切った最後の断片は、
> 文字 `b` の後に 10 進数字だけが続く形ではない。

### P11 の「ちょうど埋める」

P11 の第 2 文 (「すなわち、呼び出し元が借用し呼び出し先が所有する unit には前に `Retain` を ...」) は
`call_rc` の記述として正しく、第 8 節がそれを証明する。第 1 文の「ちょうど埋める」を実行時の義務集合に
ついての主張として読むなら、H2 が要る。`Retain(a, u)` は `u` の下の inhabited なすべての leaf の参照を
作るので、`arg_owned` の 1 つの真偽値が leaf ごとの所有と食い違うと収支が合わない (第 11.2 節)。

### P12 の「`prog.funcs` に存在する」

`route` は、呼び出し先が入力の関数を名指していないとき (局所変数を経由する間接呼び出しのとき) は
`callee` をそのまま返す。その名前は `funcs` の鍵ではない。P12 の「`prog.funcs` に存在する」は、
呼び出し先が入力の関数を名指す場合についての主張として読む必要がある。また、ここでの `prog.funcs` は
出力のものである (借用版は入力の `funcs` に無い)。

### `args.len() <= params.len()` を述べる仮定が無い

`call_rc` と `rhs_consumes` はどちらも `params[arg_idx]` を範囲内として添字づける
(`CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc`, `CODE src/rc_ir/ownership.rs: rhs_consumes`)。果たす者は
lowering である。A12 は型の整合を述べるが引数の個数を述べない。A12 に足すか、独立の仮定にするのがよい。

## 14. 証明のどこで止まったか

P14 の証明として自然な形は、入力の本体と借用版の本体の対応する位置について、次を不変条件とする本体の木の
上の帰納法である。

> `Obl_borrow` は、`Obl_orig` から「呼び出し元が貸したままの参照」を取り除いたものに等しい。

この不変条件を回すには、貸したままかどうかを判定する述語が要る。コードでその役を負うのは `owns_unit` で
あり、要るのは次の補題である。

> `owns_unit(v, u)` が偽ならば、`(v, u)` の下の inhabited な leaf の参照は `Obl_borrow` に 1 つも無い。

`R2` の `Release(a', [])` の位置がこの補題の反例である。`owns_unit(a', [])` は偽 (`3.6 の <1>4`) だが、
`(a', [])` の下の inhabited な leaf `[0, 0]` が指す `obj(x', [])` への参照は `Obl_borrow` にある
(`3.7 の <1>3`)。帰納法はここで止まる。

止まった原因は証明の運びでも命題の前提でもなく、コードである。`owns_unit` が答える真偽値 1 つでは、
1 つの unit の下の leaf が別々の root に由来してその所有が分かれる場合を表せない。

この補題を両向きにしたものが H2 である。第 11.2 節が、H2 が P10 と P11 のどの位置で要るかと、H2 を
成り立たせる直し方を述べる。

## 15. 平準化の段 (`level_ownership`)

`infer_ownership` の不動点は、消費から所有を引き上げる段のほかに、**平準化の段**を持つ
(`CODE src/rc_ir/borrow.rs: infer_ownership`, `levelled_sites`, `level_ownership`, `covered_leaves`)。
この節はその段が何を与えるかを述べる。第 16 節が H2 を定理として述べ、第 18 節がその後半が偽であることを
反例で示す。

### 15.1 記法

**DEF 平準化サイト** -- 入力の関数 `f` について、`levelled_sites(f, type_env)` が返す対 `(v, π)` をいう。
その全体は次の 2 種である (`CODE src/rc_ir/borrow.rs: levelled_sites`,
`CODE src/rc_ir/ast.rs: for_each_node`)。

- `f.body` の各 `Retain(v, π, s, k)` / `Release(v, π, s, k)` 節点について `(v, π)`。
- `f.body` の各 `Let(x, App(callee, args), k)` 節点の各 `arg ∈ args` と各 `u ∈ units(ty(arg))` に
  ついて `(arg, u)`。

`for_each_node` は `Match` の各アーム本体へも降りるので、アームの中の節点もサイトを与える。

**DEF 覆う leaf** -- `covered_leaves(τ, π)` は、`leaves(τ)` の元のうち `π` を接頭辞に持つものと、`π` の
接頭辞であるものの全体である (`CODE src/rc_ir/borrow.rs: covered_leaves`)。

**DEF LeafOwned** -- `VarTable` `vars` と leaf の集合 `L` と対 `(root, ρ)` について、`LeafOwned_L(root, ρ)`
とは、`vars.param_tys` が `root` を鍵に持たないか、持つとき (その型を `τ`) `covered_leaves(τ, ρ)` の
ある `λ` について `(root, λ) ∈ L` であることをいう。`level_ownership` の `owns_a_candidate` は、`L` を
その時点の `owned_leaves` としたときの、候補のいずれかについての `LeafOwned_L` である
(`CODE src/rc_ir/borrow.rs: level_ownership`)。

**DEF AllLeafOwned** -- 同じ設定で、`vars.param_tys` が `root` を鍵に持たないか、持つとき
`covered_leaves(τ, ρ)` の**すべての** `λ` について `(root, λ) ∈ L` であることをいう。

### 15.2 補題

#### L9 (leaf は反鎖をなす)

**言明**。型 `τ` の相異なる 2 つの leaf について、一方が他方の接頭辞になることはない。

<1>1. `boxed_leaf_paths` の内側の `go` が `out` に積むのは 3 か所であり、いずれもその直後に `return` する。
      積む path は、`is_closure` の腕では `path ++ [CLOSURE_CAPTURE_IDX]`、`is_box` の腕と `is_array` の
      腕では `path` である。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `go` が自分を呼ぶのは `unpunched_field_types` のループの中だけであり、そこでの呼び出し先の `path` は
      自分の `path` に添字を 1 つ足したものである。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. `go` の呼び出しの木において、path が `ρ` を真に伸ばす呼び出しは、path が `ρ` である呼び出しの
      子孫としてしか現れない。
  `<1>2` より path は 1 段につき添字 1 つずつ伸びるので、`ρ` を真に伸ばす path へ至る道は path が `ρ` で
  ある呼び出しを通る。
  BY <1>2

<1>4. QED
  leaf `λ1` が leaf `λ2` の真の接頭辞であると仮定する。`<1>1` より、`λ1` を積んだ呼び出しの path `ρ1` は
  `λ1` か `λ1` の最後の添字を除いたものであり、`λ2` を積んだ呼び出しの path `ρ2` は `λ2` か `λ2` の最後の
  添字を除いたものである。`λ1` が `λ2` の真の接頭辞なので `λ1` は `ρ2` の接頭辞であり、`ρ1` は `λ1` の
  接頭辞なので `ρ1` は `ρ2` の接頭辞である。`ρ1 = ρ2` のとき、その 1 つの呼び出しが `λ1` と `λ2` の
  両方を積んだことになるが、`<1>1` より積んだ直後に `return` するので積むのは 1 つだけであり、矛盾する。
  `ρ1` が `ρ2` の真の接頭辞のときは、`<1>3` より path `ρ2` の呼び出しは path `ρ1` の呼び出しの子孫で
  なければならないが、`<1>1` より path `ρ1` の呼び出しは積んだ直後に `return` するので子を持たず、
  矛盾する。
  BY <1>1, <1>3

#### L10 (`covered_leaves` の 2 つの向きは同時に発火しない)

**言明**。`covered_leaves(τ, π)` の元のうち、`π` を接頭辞に持つものの集合を `Under`、`π` の真の接頭辞で
あるものの集合を `Above` と書く。`Under` と `Above` が同時に空でないことはなく、`Above` は高々 1 元で
ある。

<1>1. `Above` の 2 元は互いに比較可能である。どちらも `π` の接頭辞なので、短い方が長い方の接頭辞である。
  BY DEF 覆う leaf

<1>2. `Above` は高々 1 元である。
  2 元あれば `<1>1` より一方が他方の真の接頭辞となり、L9 に反する。
  BY L9, <1>1

<1>3. `Under` と `Above` が同時に空でないことはない。
  `λ ∈ Above` かつ `μ ∈ Under` とすると、`λ` は `π` の真の接頭辞、`π` は `μ` の接頭辞なので `λ` は `μ` の
  真の接頭辞であり、L9 に反する。
  BY L9

<1>4. QED
  BY <1>2, <1>3

#### L11 (leaf は unit の真の接頭辞にならない)

**言明**。型 `τ` の leaf `λ` と unit `u` について、`λ` が `u` の真の接頭辞になることはない。

<1>1. `rc_units_go` が `out` に積むのは 2 か所である。`UnitStep::Unit` の腕では `path`、
      `UnitStep::Capture` の腕では `path ++ [capture_idx]` であり、どちらの腕も自分を呼ばない。自分を
      呼ぶのは `UnitStep::Fields` の腕だけで、そこでは `held_fields` の添字を 1 つ足す。
  BY CODE src/rc_ir/ownership.rs: rc_units_go

<1>2. `unit_step(σ)` が `UnitStep::Capture` を返すときの `capture_idx` は `CLOSURE_CAPTURE_IDX` であり、
      `boxed_leaf_paths` の `go` が `is_closure` の腕で足す添字と同じである。
  BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. `unit_step(σ)` が `UnitStep::Fields` を返すならば `go` は `σ` で降り、しかも同じ
      `unpunched_field_types` の列を回る。逆に `go` が `σ` で降りるとき、`unit_step(σ)` は
      `UnitStep::Fields` か `UnitStep::Unit` のどちらかであり、`Unit` になるのは `is_union` か
      `is_punched_array` が真のときだけである。
  `go` が降りるのは `is_fully_unboxed`・`is_closure`・`is_box`・`is_array` がいずれも偽のときであり、
  `unit_step` が `Fields` を返すのはこの 4 つに `is_union` と `is_punched_array` を加えた 6 つが
  いずれも偽のときである。前者の条件は後者の条件より弱い。`unit_step` はこの 6 つのうち
  `is_fully_unboxed`・`is_closure`・`is_box`・`is_array` でそれぞれ `NoUnit` / `Capture` / `Unit` /
  `Unit` を返すので、`go` が降りて `unit_step` が `Fields` でない残りの場合は `is_union` か
  `is_punched_array` であり、そこで `unit_step` は `Unit` を返す。降りる先の列はどちらも
  `unpunched_field_types` である。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step

<1>4. `λ` を積んだ `go` の呼び出しの path を `ρ` とする。`ρ` の真の接頭辞 `ρ'` で `unit_step` が
      `UnitStep::Fields` でないものがあるとき、`λ` を真に伸ばす unit は無い。
  <2>1. そのような `ρ'` のうち最短のものを `ρ0` とする。`go` は `ρ` まで降りているので `ρ0` でも降りて
        おり、`<1>3` より `unit_step` は `ρ0` で `UnitStep::Unit` である。
    BY <1>3, L9 の <1>2
  <2>2. `rc_units_go` は `ρ0` の真の接頭辞ではすべて `UnitStep::Fields` を通るので `ρ0` まで降り、`ρ0` で
        `path` すなわち `ρ0` を積んで止まる。よって `ρ0` を接頭辞に持つ unit は `ρ0` だけである。
    BY <1>1, <2>1
  <2>3. QED
    `ρ0` は `ρ` の真の接頭辞であり、`ρ` は `λ` の接頭辞なので、`ρ0` は `λ` の接頭辞である。`<2>2` より
    `ρ0` を接頭辞に持つ unit は `ρ0` だけであり、それは `λ` を真に伸ばさない。`ρ0` を接頭辞に持たない
    unit は `λ` も接頭辞に持たない。
    BY <2>1, <2>2

<1>5. `ρ` の真の接頭辞のすべてで `unit_step` が `UnitStep::Fields` であるとき、`λ` を真に伸ばす unit は
      無い。
  <2>1. `rc_units_go` は `ρ` まで降りる。
    BY <1>1
  <2>2. `unit_step` は `ρ` で `UnitStep::Unit` か `UnitStep::Capture` である。`go` は `ρ` で積んで
        `return` したので `is_box`・`is_array`・`is_closure` のいずれかが真である。
    BY L9 の <1>1, CODE src/rc_ir/ownership.rs: unit_step
  <2>3. `unit_step` が `ρ` で `UnitStep::Unit` のとき、`λ = ρ` であり、`ρ` を接頭辞に持つ unit は `ρ` だけ
        である。
    BY L9 の <1>1, <1>1, <2>1, <2>2
  <2>4. `unit_step` が `ρ` で `UnitStep::Capture` のとき、`λ = ρ ++ [CLOSURE_CAPTURE_IDX]` であり、`ρ` を
        接頭辞に持つ unit は `ρ ++ [capture_idx]` すなわち `λ` だけである。
    BY L9 の <1>1, <1>1, <1>2, <2>1, <2>2
  <2>5. QED
    `<2>3` と `<2>4` のどちらでも、`ρ` を接頭辞に持つ unit は `λ` に等しい。`ρ` を接頭辞に持たない unit は
    `λ` も接頭辞に持たない。
    BY <2>3, <2>4

<1>6. QED
  `<1>4` と `<1>5` が場合を尽くす。
  BY <1>4, <1>5

**注**。逆向きは成り立つ。unit が leaf の真の接頭辞になるのは `<1>3` の 2 つの型、すなわち unbox union と
punched array のときである。これが D5 が述べる「leaf と unit がずれる 2 か所」である。

#### L12 (`origin` が辿る path は leaf の真の子孫にならない)

**言明**。`level_ownership` が `covered_leaves` に渡す対 `(root, ρ)` について、`ty(root)` の leaf で `ρ` の
真の接頭辞であるものは無い。

**DEF 深すぎない** -- 変数 `w` と path `π` について、`ty(w)` の leaf で `π` の真の接頭辞であるものが無い
ことをいう。

<1>1. `level_ownership` が `origin` に渡す最初の `(v, u)` は平準化サイトであり、そこでは「深すぎない」が
      成り立つ。
  DEF 平準化サイト の 2 種はどちらも `u ∈ units(ty(v))` である。1 種目は A2 による。2 種目は
  `rc_units(ty(arg))` の元である。L11 よりどちらも「深すぎない」を満たす。
  BY A2, L11, DEF 平準化サイト, CODE src/rc_ir/borrow.rs: level_ownership, levelled_sites

<1>2. `origin_inner(vars, type_env, w, π)` が「深すぎない」`(w, π)` について呼ばれるとき、そこから
      再帰する先の `(w', π')` も「深すぎない」を満たし、そこで作られる `Origin` の候補として現れる対も
      「深すぎない」を満たす。
  <2>1. CASE `None | Some(Binding::Param) | Some(Binding::Producer)`。`here()` すなわち `(w, π)` を返す。
        再帰しない。
    BY 仮定 (`(w, π)` は「深すぎない」), CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. CASE `Some(Binding::Move(y))`。`origin(y, π)` へ再帰する。A12 より `ty(y) = ty(w)` なので
        `(y, π)` は「深すぎない」。
    BY A12, 仮定, CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. CASE `Some(Binding::Join(arm_results))`。各 `arm_result` について `origin(arm_result, π)` へ
        再帰し、`of_candidates(.., (w, π))` を作る。A12 より各アームの結果の型は `ty(w)` に等しいので
        再帰先は「深すぎない」。`identity` として加わる `(w, π)` も仮定より「深すぎない」。
    BY A12, 仮定, CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates
  <2>4. CASE `Some(Binding::Llvm(..))` で `decl.leaf_origins_at(π)` が単一の `Arg(j, σ)` である場合。
        `origin(args[j], σ)` へ再帰する。A3 より `σ` は `ty(args[j])` の leaf であり、L9 より leaf は
        他の leaf の真の接頭辞にならないので `(args[j], σ)` は「深すぎない」。
    BY A3, L9, CODE src/rc_ir/ownership.rs: origin_inner, as_arg_projection
  <2>5. CASE `Some(Binding::Llvm(..))` で `<2>4` に当たらない場合。`origin_from_leaves_under` は、
        `LeafOrigin::Arg(j, leaf)` の各元について `origin(args[j], truncate_to_unit(ty(args[j]), leaf))`
        へ再帰し、`Fresh` / `Unknown` があるときは `Exactly(here)` を、どの leaf も対象を名指さないときは
        `here()` を混ぜる。A3 より `leaf` は `ty(args[j])` の leaf であり、P1 よりその
        `truncate_to_unit` は `units(ty(args[j]))` の元なので、L11 より再帰先は「深すぎない」。
        `here` は `(w, π)` であり仮定より「深すぎない」。
    BY A3, P1, L11, 仮定, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
  <2>6. CASE `Some(Binding::Field(container, idx))` で `container` が boxed の場合。`here()` を返す。
    BY 仮定, CODE src/rc_ir/ownership.rs: origin_inner
  <2>7. CASE `Some(Binding::Field(container, idx))` で `container` が unbox の場合。
        `origin(container, [idx] ++ π)` へ再帰する。
    <3>1. A12 より `container` は構造体であり、`ty(w)` は `ty(container)` の第 `idx` フィールドの型で
          ある。`boxed_leaf_paths` は unbox の構造体で `unpunched_field_types` へ降りるので、
          `ty(container)` の leaf のうち `[idx]` を接頭辞に持つものは、`[idx] ++ λ'` (`λ'` は `ty(w)` の
          leaf) の全体である。
      BY A12, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <3>2. `ty(container)` の leaf で `[idx] ++ π` の真の接頭辞であるものは、`<3>1` より `[idx] ++ λ'` の
          形で `λ'` が `π` の真の接頭辞であるものか、`[idx]` の真の接頭辞であるもの、すなわち空列が leaf
          である場合に限る。前者は仮定に反する。後者は `<3>1` より `boxed_leaf_paths` が `ty(container)`
          で降りることに反する。
      BY 仮定, <3>1
    <3>3. QED
      BY <3>1, <3>2
  <2>8. CASE `Some(Binding::Payload(scrut, None))`。`origin(scrut, π)` へ再帰する。A12 より catch-all の
        payload の型は `ty(scrut)` に等しい。
    BY A12, 仮定, CODE src/rc_ir/ownership.rs: origin_inner
  <2>9. CASE `Some(Binding::Payload(scrut, Some(tag)))` で `scrut` が unbox の場合。
        `origin(scrut, [tag] ++ π)` へ再帰する。A12 より `scrut` は union であり `ty(w)` は第 `tag` 変位の
        型である。`ty(scrut)` が leaf を持つならそれは `is_fully_unboxed` ではなく、unbox なので `is_box`
        でもなく、union なので `is_closure` でも `is_array` でもないから、`boxed_leaf_paths` は
        `unpunched_field_types` へ降りる。あとは `<2>7` と同じである。
    BY A12, 仮定, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>10. CASE `Some(Binding::Payload(scrut, Some(_)))` で `scrut` が boxed の場合。`here()` を返す。
    BY 仮定, CODE src/rc_ir/ownership.rs: origin_inner
  <2>11. QED
    `Binding` は 7 種であり (`CODE src/rc_ir/ownership.rs: Binding`)、鍵が無い場合を加えた 8 通りを、
    `Llvm` を 2 つ、`Field` を 2 つ、`Payload` を 3 つに分けた `<2>1`-`<2>10` が尽くす。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, <2>8, <2>9, <2>10,
       CODE src/rc_ir/ownership.rs: Binding, origin_inner

<1>3. QED
  `level_ownership` が `covered_leaves` に渡すのは `origin(vars, type_env, v.name, unit).candidates()` の
  元である。`<1>1` が出発点について、`<1>2` が 1 段ごとに「深すぎない」を保つことを与える。P2 より
  `origin` は停止するので、この帰納は有限段で終わる。
  BY P2, <1>1, <1>2, CODE src/rc_ir/borrow.rs: level_ownership

**系 (`covered_leaves` の 2 つの向きについて)**。`level_ownership` が渡す `(root, ρ)` について、
`covered_leaves(ty(root), ρ)` の `Above` (`ρ` の真の接頭辞である leaf) は空である。よって
`covered_leaves` の `path.starts_with(leaf)` の側が真になるのは `ρ` 自身が leaf であるときに限り、そのとき
`leaf.starts_with(path)` の側も真である。すなわちこの呼び出しでは `covered_leaves` の値は
`leaf.starts_with(path)` の側だけで決まる。

BY L12, DEF 覆う leaf, CODE src/rc_ir/borrow.rs: covered_leaves

#### L13 (`covered_leaves` と `units_under` の粒度)

**言明**。`τ` と `π` について、`ty` の leaf で `π` の真の接頭辞であるものは無いとする。
`U(π) = { truncate_to_unit(τ, u, type_env) : u ∈ units_under(τ, π, type_env) }`、
`L(π) = covered_leaves(τ, π, type_env)` とおく。

- (a) `L(π)` の各 `λ` について `truncate_to_unit(τ, λ, type_env) ∈ U(π)` である。
- (b) `L(π)` が空でないとき、`U(π)` の各元はある `λ ∈ L(π)` の `truncate_to_unit(τ, λ, type_env)` で
  ある。
- (c) `L(π)` は `{ λ ∈ leaves(τ) : truncate_to_unit(τ, λ, type_env) ∈ U(π) }` に含まれる。等しいとは
  限らない。

<1>1. 仮定と L10 より、`L(π)` は `π` を接頭辞に持つ leaf の全体である。
  BY L10, 仮定

<1>2. CASE `subtree_type(τ, π, type_env) = Some(σ)`。
  <2>1. `π` の各真の接頭辞で `unit_step` は `UnitStep::Fields` であり、`σ` は `π` が指す部分木の型で
        ある。
    BY CODE src/rc_ir/ownership.rs: subtree_type
  <2>2. `boxed_leaf_paths` は `<2>1` の各段で `unpunched_field_types` へ降りるので、`π` を接頭辞に持つ
        `τ` の leaf は `π ++ leaves(σ)` である。すなわち `L(π) = π ++ leaves(σ)` である。
    L11 の `<1>3` より `unit_step` が `Fields` の型では `go` も降り、しかも同じ `unpunched_field_types` を
    回る。
    BY L11 の <1>3, <1>1, <2>1, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `units_under(τ, π, type_env) = π ++ rc_units(σ)` である。
    BY <2>1, CODE src/rc_ir/ownership.rs: units_under
  <2>4. `ρ` を `σ` の path とするとき `truncate_to_unit(τ, π ++ ρ, type_env) = π ++ truncate_to_unit(σ, ρ,
        type_env)` である。
    `truncate_to_unit` は `<2>1` の各段で `UnitStep::Fields` の腕を通り、添字を `out` に積んで降りる。
    BY <2>1, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>5. QED
    `<2>2`-`<2>4` と P1 (`σ` について) が (a) と (b) を与える。
    BY P1, <2>2, <2>3, <2>4

<1>3. CASE `subtree_type(τ, π, type_env) = None`。
  <2>1. `units_under(τ, π, type_env) = [π]` であり、`U(π) = { truncate_to_unit(τ, π, type_env) }` で
        ある。
    BY CODE src/rc_ir/ownership.rs: units_under
  <2>2. `truncate_to_unit` の走査は、`π` の真の接頭辞のうち `unit_step` が `UnitStep::Fields` でない
        最初のもの `π0` で止まる (`UnitStep::Unit` なら `break`、`UnitStep::Capture` なら添字を 1 つ
        積んで `break`)。よって `π0` を接頭辞に持つどの path についても `truncate_to_unit` の値は同じで
        ある。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit, subtree_type
  <2>3. `L(π)` の各 `λ` は `π` を接頭辞に持ち (`<1>1`)、よって `π0` を接頭辞に持つ。
    BY <1>1, <2>2
  <2>4. QED
    `<2>2` と `<2>3` より `L(π)` の各 `λ` の `truncate_to_unit` は `truncate_to_unit(τ, π, type_env)` に
    等しく、`<2>1` よりそれが `U(π)` の唯一の元である。これが (a) と (b) を与える。
    BY <2>1, <2>2, <2>3

<1>4. QED
  (a) と (b) は `<1>2` と `<1>3` による。(c) の包含は (a) による。等しくない例は第 18 節の型 `W` と
  `π = [1]` である (`18.2 の <1>4`)。
  BY <1>2, <1>3, 18.2 の <1>4

### 15.3 平準化は停止性を壊さない

**言明**。平準化の段が入っても `infer_ownership` は停止する (P8 (a))。

<1>1. `owned_leaves` を変える箇所は 2 つになった。消費の段の `owned_leaves.insert((root_var, root_path))`
      と、`level_ownership` の中の `owned_leaves.insert((root, leaf))` である。どちらも挿入だけで、
      取り除かない。
  BY CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

<1>2. `changed` が真になるのは、`<1>1` のどちらかの `insert` が真を返したときだけである。
      `level_ownership` は `owns_a_candidate` が偽なら `false` を返し、真のときは `insert` の返り値の
      論理和を返す。
  BY CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

<1>3. `level_ownership` が挿入する元は、`vars.param_tys` に鍵を持つ `root` と `boxed_leaf_paths(ty(root))`
      の元からなる。`param_tys` に無い `root` は `continue` で飛ばされる。
  BY CODE src/rc_ir/borrow.rs: level_ownership, covered_leaves

<1>4. 挿入されうる元の全体は有限である。
  D1 より関数は有限個、各関数の `param_tys` は有限、A10 より各型の `boxed_leaf_paths` は有限である。
  消費の段の分は 5.2 の P8 (a) の `<1>3` が与える。
  BY A10, D1, P8 (a) の <1>3, <1>3

<1>5. 1 周回の仕事は有限である。
  `sites` はループの外で 1 度だけ作られる。各サイトについて `origin` は停止して有限集合を返し (P2)、
  `covered_leaves` は有限の列を返す。
  BY P2, A10, CODE src/rc_ir/borrow.rs: infer_ownership, levelled_sites, covered_leaves

<1>6. QED
  `<1>1` と `<1>2` より `changed` が真の周回では `owned_leaves` は真に大きくなり、`<1>4` よりその大きさ
  には上界がある。よって `changed` が真の周回は有限回で、その次の周で `changed` が偽になり `break` する。
  各周回は `<1>5` より有限である。
  BY <1>1, <1>2, <1>4, <1>5

### 15.4 平準化は P8 の言明を壊さない

**言明**。5.2 節の P8 (b) と P8 (c) は、平準化の段が入った後もそのまま成り立つ。ただし平準化は H1 を
成り立たせない。

<1>1. P8 (b) の証明はそのまま通る。
  <2>1. ループが `break` する周回では `changed` が偽であり、15.3 の `<1>2` よりその周回のどの `insert` も
        偽を返す。よってその周回の間 `owned_leaves` は変わらない。これは 5.2 の P8 (b) の `<1>2` が
        必要としたことである。
    BY 15.3 の <1>2, P8 (b) の <1>2, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>2. QED
    P8 (b) の残りの段は `collect_consumes` と消費の段だけを読み、平準化の段を読まない。
    BY P8 (b), <2>1

<1>2. P8 (c) の証明はそのまま通る。
  P8 (c) は P7、L8 (H1 の下で)、P8 (b) だけを使う。平準化はこの 3 つのどれも変えない。
  BY P7, L8, P8 (b), <1>1

<1>3. 平準化は H1 を成り立たせない。
  <2>1. 平準化がそろえるのは、平準化サイトの `origin` の候補が覆う leaf だけである。
    BY DEF 平準化サイト, CODE src/rc_ir/borrow.rs: level_ownership
  <2>2. パラメータ `p` とその unit `u` について、`(p, u)` が平準化サイトでなく、かつどの平準化サイトの
        候補も `p` の `u` の下の leaf を覆わないことがありうる。後の第 18 節が置く関数 `g` の
        パラメータ `p` と `u = []` がその例である (この段は第 18 節の事実を引く)。`p` は `Retain`/`Release` の対象でも `App` の引数でもなく (`18.1 の
        DEF g`)、`p` を覆う候補を持つ唯一のサイトは `(v, [])` であり、その候補は `(p, [1])` であって
        `covered_leaves(W, [1]) = {[1]}` は `[0]` を含まない (`18.2 の <1>4`)。
    BY 18.1 の DEF g, 18.2 の <1>4, 18.4 の <1>5
  <2>3. QED
    `18.4 の <1>6` より `owned_leaves` は `(p, [0])` を含み `(p, [1])` を含まない。`18.2 の <1>3` より
    `truncate_to_unit(W, [0]) = truncate_to_unit(W, [1]) = []` なので、H1 の言明の「同じ unit へ切り詰まる
    leaf の `owns` の値は等しい」が破れる。
    BY H1, 18.2 の <1>3, 18.4 の <1>6, <2>1, <2>2

### 15.5 平準化の不動点が与えるもの

#### L14 (借用版の `owned_units` は `owned_leaves` の unit への像である)

**言明**。H3 の下で、`borrow_versions` に載る入力の関数 `f` のパラメータ `p` と `u ∈ units(ty(p))` に
ついて、`(rename_f[p.name], u) ∈ owned_units` であることと、`truncate_to_unit(ty(p), λ, type_env) = u`
かつ `(p.name, λ) ∈ owned_leaves` である `λ ∈ leaves(ty(p))` があることとは同値である。

<1>1. 後者ならば前者である。
  L2 の (b) がその `λ` について `(rename_f[p.name], truncate_to_unit(ty(p), λ, type_env))` を入れる。
  BY L2

<1>2. 前者ならば後者である。
  L2 より `owned_units` の元は (a) の形か (b) の形である。`rename_f[p.name]` は 6.2 の言明より入力の
  束縛名ではないので (a) の形ではありえない。(b) の形であるとき、6.1 の `<1>4` と 6.3 の `<1>1` より
  相異なる束縛子には相異なる名前が付くので、その元を入れた関数とパラメータは `f` と `p` である。
  BY L2, 6.1 の <1>4, 6.2 の言明, 6.3 の <1>1

<1>3. QED
  BY <1>1, <1>2

#### T1 (平準化の不動点)

**言明**。`infer_ownership` が返す `owned_leaves` を `L` とする。入力の各関数 `f` の各平準化サイト
`(v, u)` について、`origin(vars_f, type_env, v.name, u).candidates()` の元 `c` を候補と呼ぶとき、次の
いずれかが成り立つ。

- (a) すべての候補 `c` について `AllLeafOwned_L(c)` が成り立つ。
- (b) どの候補 `c` についても `LeafOwned_L(c)` が成り立たない。

<1>1. ループが `break` する周回では、その周回のどの `level_ownership` の呼び出しも偽を返す。
  BY 15.3 の <1>2, CODE src/rc_ir/borrow.rs: infer_ownership

<1>2. その周回の間 `owned_leaves` は変わらず、その値が返される `L` である。
  BY 15.3 の <1>2, <1>1, CODE src/rc_ir/borrow.rs: infer_ownership

<1>3. サイト `(v, u)` について `owns_a_candidate` が偽であるとき、(b) が成り立つ。
  `owns_a_candidate` は「ある候補について `LeafOwned_L`」そのものである。`param_tys` に鍵を持たない
  `root` の候補では `None` の腕が `true` を返すので、そのような候補があれば `owns_a_candidate` は真で
  ある。
  BY DEF LeafOwned, <1>2, CODE src/rc_ir/borrow.rs: level_ownership

<1>4. サイト `(v, u)` について `owns_a_candidate` が真であるとき、(a) が成り立つ。
  `owns_a_candidate` が真のとき `level_ownership` は、`param_tys` に鍵を持つ各候補の
  `covered_leaves(ty(root), path)` の各 `leaf` について `owned_leaves.insert((root, leaf))` を呼ぶ。
  `<1>1` よりそのすべてが偽を返すので、そのすべてがすでに `L` に入っている。`param_tys` に鍵を持たない
  候補は `AllLeafOwned_L` を定義から満たす。
  BY DEF AllLeafOwned, <1>1, <1>2, CODE src/rc_ir/borrow.rs: level_ownership

<1>5. QED
  BY <1>3, <1>4

#### T2 (T1 の (a) から `owns_unit` へ)

**言明**。H3 の下で、`borrow_versions` に載る入力の関数 `f` の平準化サイト `(v, u)` が T1 の (a) を
満たし、かつ `param_tys` に鍵を持つ各候補 `(root, ρ)` について `covered_leaves(ty(root), ρ)` が空でない
ならば、`f_borrow` の `RewriteCtx` `ctx` について `ctx.owns_unit(v', u)` は真である。ここで `v'` は
`rename_f` による `v` の像である。

<1>1. `f_borrow` の `VarTable` で読んだ `origin(v', u)` の候補は、`f` の `VarTable` で読んだ
      `origin(v, u)` の候補を `rename_f` で写したものである。
  P9 の前半より `f_borrow` の本体は `f` の本体の束縛変数を一斉に付け替えたものであり、`collect_bindings`
  と `origin_inner` は変数の名前を鍵にして同じ形の束縛を辿るので、辿る道は同じで名前だけが写る。
  `param_tys` の鍵も同じ写像で写る。
  BY P9, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings, origin_inner

<1>2. `param_tys` に鍵を持たない候補 `(root, ρ)` について `ctx.owns_object(root, ρ)` は真である。
  BY L4

<1>3. `param_tys` に鍵を持つ候補 `(root, ρ)` について `ctx.owns_object(rename_f[root], ρ)` は真である。
  <2>1. 仮定より `covered_leaves(ty(root), ρ)` は空でなく、(a) よりその各 `λ` について
        `(root, λ) ∈ owned_leaves` である。
    BY DEF AllLeafOwned, 仮定
  <2>2. L12 より `(root, ρ)` は「深すぎない」ので L13 が使える。L13 の (b) より、
        `units_under(ty(root), ρ, type_env)` の各 `unit` の `truncate_to_unit(ty(root), unit, type_env)`
        は、ある `λ ∈ covered_leaves(ty(root), ρ)` の `truncate_to_unit(ty(root), λ, type_env)` である。
    BY L12, L13
  <2>3. `<2>1` と `<2>2` と L14 より、`units_under(ty(root), ρ, type_env)` の各 `unit` について
        `(rename_f[root], truncate_to_unit(ty(root), unit, type_env)) ∈ owned_units` である。
    BY L14, <2>1, <2>2
  <2>4. QED
    L4 より `owns_object(rename_f[root], ρ)` はまさに `<2>3` が言うことである。
    BY L4, <2>3

<1>4. QED
  `owns_unit(v', u)` は `origin(v', u).candidates()` の全元について `owns_object` を要求する。`<1>1` で
  候補を写し、`<1>2` と `<1>3` が各候補について `owns_object` を与える。
  BY <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

## 16. H2 -- 定理としての言明

### 16.1 言明

第 4 節の H2 を、平準化の段の下で証明すべき定理として書き直す。文面は変えない。

**H2**。`borrow_ify` の出力の各版について、`RewriteCtx::rewrite` が走査する本体と、その版の
`RewriteCtx` `ctx` をとる。その本体の各 `Retain(w, π, ..)` / `Release(w, π, ..)` 節点の各
`u ∈ units_under(ty(w), π, type_env)`、およびその本体の各 `Let(x, App(callee, args), k)` の各引数
`w ∈ args` の各 `u ∈ units(ty(w))` について、`u` を接頭辞に持つ**各** `λ ∈ leaves(ty(w))` について次が
成り立つ。

> `origin(w, λ).candidates()` の全元が `ctx.owns_object` を満たすことと、`ctx.owns_unit(w, u)` が真で
> あることとが同値である。

「各 `λ` について同値」であって「すべての `λ` についての連言と同値」ではない。この違いが load-bearing で
ある。第 14 節が要求するのは、`owns_unit` の 1 つの真偽値が `u` の下の leaf ごとの所有と leaf ごとに
一致することだからである。

以下、`u` を接頭辞に持つ `λ` について「`X(λ)`」を「`origin(w, λ).candidates()` の全元が
`ctx.owns_object` を満たす」、「`Y`」を「`ctx.owns_unit(w, u)` が真である」と書く。

### 16.2 前半 -- `Y` ならば `X(λ)`

<1>1. P7a より、`u` を接頭辞に持つ各 `λ` について `origin(w, λ).candidates()` の各元は
      `origin(w, u).candidates()` の元である。
  BY P7a

<1>2. QED
  `Y` は `origin(w, u).candidates()` の全元が `ctx.owns_object` を満たすことである。`<1>1` よりその
  部分集合である `origin(w, λ).candidates()` の全元も満たす。
  BY <1>1, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

**注**。この向きは平準化を使わない。P7a だけで出る。

### 16.3 後半 -- `X(λ)` ならば `Y`

この向きは**成り立たない**。第 18 節の反例 `R3` がその反例である。

平準化がこの向きに与えるものと、足りないものは次のとおりである。

<1>1. `X(λ)` が成り立つとき、`origin(w, u).candidates()` のある元 `c` について `ctx.owns_object(c)` が
      成り立つ。
  P7a より `origin(w, λ).candidates()` は `origin(w, u).candidates()` に含まれ、`of_candidates` の表明より
  空でない。
  BY P7a, CODE src/rc_ir/ownership.rs: Origin::of_candidates

<1>2. `origin(w, u).candidates()` のある元 `c` について `LeafOwned_L(c)` が成り立ち、かつ
      `param_tys` に鍵を持つ各候補の `covered_leaves` が空でないならば、T1 の (a) が成り立ち、T2 より
      `Y` が成り立つ。
  BY T1, T2

<1>3. `ctx.owns_object(c)` から `LeafOwned_L(c)` は出ない。
  <2>1. `ctx.owns_object(root_b, ρ)` は、L4 と L14 より、`units_under(ty(root), ρ, type_env)` の各 `unit`
        について「`truncate_to_unit(ty(root), unit, type_env)` へ切り詰まる `ty(root)` の leaf のうち少なく
        とも 1 つが `owned_leaves` に入っている」ことである。
    BY L4, L14
  <2>2. `LeafOwned_L(root, ρ)` は、`covered_leaves(ty(root), ρ)` のうち少なくとも 1 つが `owned_leaves` に
        入っていることである。
    BY DEF LeafOwned
  <2>3. L13 の (c) より `covered_leaves(ty(root), ρ)` は
        `{ λ ∈ leaves(ty(root)) : truncate_to_unit(ty(root), λ, type_env) ∈ U(ρ) }` に真に含まれることが
        ある。その差の leaf だけが `owned_leaves` に入っているとき、`<2>1` は真で `<2>2` は偽である。
    BY L13
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>4. QED
  `<1>1` が与えるのは `owns_object` を満たす候補の存在であり、`<1>2` が要求するのは `LeafOwned_L` を
  満たす候補の存在である。`<1>3` よりこの 2 つは異なる。第 18 節が、この差が実際に開く入力を与える。
  BY <1>1, <1>2, <1>3

**差が開く形**。`<1>3 の <2>3` の差の leaf は、L13 の証明より、`ρ` が入っていく先とは別の変位の leaf で
ある。1 つの unit の下に leaf を 2 つ以上持つ型は unbox union だけなので (第 3.9 節)、差が開くのは
**`ρ` が unbox union の 1 つの変位の中を指し、その union の別の変位の leaf だけが所有されているとき**で
ある。

## 17. R2 は平準化の下で消える

**言明**。第 3 節の入力プログラム `prog` について、平準化の段を含む `infer_ownership` は
`owned_leaves = {(x, []), (y, [])}` を返す。よって `f` は借用版を持たず、`borrow_ify(prog, type_env)` の
出力は D12 の意味で健全である。

<1>1. `f` の平準化サイトは `(x, [])` と `(a, [])` の 2 つである。
  `DEF f` の本体の `Retain`/`Release` 節点は `Retain(x, [], s, ..)` と `Release(a, [], s, ..)` の 2 つで
  あり、`App` 節点は無い。
  BY DEF f, DEF 平準化サイト, CODE src/rc_ir/borrow.rs: levelled_sites

<1>2. 消費の段は `(x, [])` を `owned_leaves` に入れる。
  BY 3.5 の <1>1, 3.5 の <1>2, 3.5 の <1>3

<1>3. サイト `(a, [])` の候補は `{(x, []), (y, [])}` である。
  `3.6 の <1>2` は `f_borrow` の `VarTable` で `origin(a', [])` を計算したものだが、T2 の `<1>1` より
  `f` の `VarTable` で読んだ候補はそれを `rename_f` の逆で写したものであり、`{(x, []), (y, [])}` で
  ある。
  BY T2 の <1>1, 3.6 の <1>2

<1>4. サイト `(a, [])` で `owns_a_candidate` は真である。
  候補 `(x, [])` について `x ∈ param_tys` であり、`covered_leaves(A, [])` は `3.2 の <1>1` より `{[]}` で
  あり、`<1>2` より `(x, []) ∈ owned_leaves` である。
  BY 3.2 の <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: level_ownership, covered_leaves

<1>5. よって `level_ownership` は `(x, [])` と `(y, [])` を `owned_leaves` に入れる。
  挿入するのは各候補の `covered_leaves` の全元であり、`3.2 の <1>1` よりそれぞれ `{[]}` である。
  BY 3.2 の <1>1, <1>3, <1>4, CODE src/rc_ir/borrow.rs: level_ownership

<1>6. 不動点は `owned_leaves = {(x, []), (y, [])}` である。
  消費は `3.5 の <1>1` より `(x, [])` だけであり、サイト `(x, [])` の候補は `{(x, [])}` (`3.5 の <1>2`)、
  サイト `(a, [])` の候補は `<1>3` である。次の周回はどちらも既にある元しか挿入しない。
  BY 3.5 の <1>1, 3.5 の <1>2, <1>3, <1>5

<1>7. `func_has_borrowable_param(f, owned_leaves, type_env)` は偽である。
  `f` のパラメータは `x : A` と `y : A` で、`3.2 の <1>1` より leaf はそれぞれ `[]` の 1 つであり、
  `<1>6` よりどちらも `owned_leaves` に入る。
  BY 3.2 の <1>1, <1>6, CODE src/rc_ir/borrow.rs: func_has_borrowable_param, OwnedLeaves::owns

<1>8. QED
  `<1>7` より `borrow_versions` は空であり、出力の `funcs` は `f_own` だけである (L6)。`f_own` の
  `RewriteCtx` は `is_borrow_version` が偽なので、`rewrite_rc` は与えられた節点をそのまま置く (L3 の
  `<1>2` が読む分岐の手前で返る)。`route` は `borrow_versions` が空なので `callee` をそのまま返し
  (P12 (a))、`B` に `App` は無いので `call_rc` も呼ばれない。よって `f_own` の本体は `f` の本体と同じで
  あり、`3.4 の <1>8` よりそれは D11 を満たす。
  BY L3, L6, P12 (a), DEF f, 3.4 の <1>8, <1>7,
     CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, borrow_ify

**注**。`R2` を排除したのは「借用版を作らない」という結末であって、`owns_unit` の答えが変わったこと
ではない。仮に `f` が別のパラメータを持って借用版が作られたとしても、`<1>5` の後は T2 より
`owns_unit(a', [])` が真になるので、`Release(a', [])` は残る。

## 18. 発見 -- P14 の反例 `R3`

第 3 節の `R2` は平準化で消えたが、**P14 は依然として偽である**。この節がその反例 `R3` を与える。
`R3` が使うのは、H2 の後半が破れる形 (第 16.3 節の「差が開く形」)、すなわち**パラメータの unbox union の
一方の変位の leaf だけが所有されている**ことである。

### 18.1 反例が置く型と関数

第 3.1 節の `DEF A` (`Array I64`)、`DEF P` (`(A, A)`)、`DEF U`
(`unbox union { twins : P, none : () }`) をそのまま使う。加えて次を置く。

**DEF W** -- `unbox union { a : A, b : A }`。Fix で書ける宣言である。

**DEF AE** -- `InlineLLVMArrayUnsafeEmpty` の `A` での実体。オペランドを 1 つ (容量、`I64`) とり、`A` を
返す (`CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMArrayUnsafeEmpty`)。

**DEF g** -- 次の関数とする。funptr ABI (`capture` は `None`)、パラメータは `p : W`、`q : A`、`c : I64`、
返り値の型は `A` である。`s` は任意の `RcState` とする。本体 `B_g` は次のとおりである。

```
B_g = Let(m, Match(p, [
          arm0 (tag 0, payload y0): Release(q, [], s, Ret(y0)),
          arm1 (tag 1, payload y1):
              Let(t, Llvm(MS, [y1, q]),
              Let(v, Llvm(MU, [t]),
              Release(v, [], s,
              Let(z, Llvm(AE, [c]),
              Ret(z)))))
        ]),
      Ret(m))
```

`MS` と `MU` は第 3.1 節の `DEF f` と同じもの、すなわち `P` を作る `InlineLLVMMakeStructBody` の実体と、
`U` の変位 `twins` (添字 0) を作る `InlineLLVMMakeUnionBody` の実体である。型は `y0 : A`、`y1 : A`、
`t : P`、`v : U`、`z : A`、`m : A` である。

入力プログラム `prog3` は `funcs = {g}`、`globals = []` とし、`g.borrowed_units` は空とする。

### 18.2 型の leaf と unit

<1>1. `leaves(A) = {[]}`、`units(A) = {[]}`、`leaves(P) = {[0], [1]}`、`units(P) = {[0], [1]}`、
      `leaves(U) = {[0, 0], [0, 1]}`、`units(U) = {[]}` である。
  BY 3.2 の <1>1, 3.2 の <1>2, 3.2 の <1>3

<1>2. `leaves(W) = {[0], [1]}` である。
  `W` は unbox の union であり、`is_fully_unboxed` は偽である (変位の型 `A` が `is_array` なので偽で
  ある)。`is_closure`、`is_box`、`is_array` はいずれも偽なので、`boxed_leaf_paths` は
  `unpunched_field_types` が返す `[(0, A), (1, A)]` の各元へ降り、`<1>1` より各変位で自分自身の path を
  積む。
  BY D4, DEF W, <1>1, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::unpunched_field_types,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. `units(W) = {[]}` であり、`truncate_to_unit(W, [0], type_env) = truncate_to_unit(W, [1], type_env)
      = []` である。
  `unit_step(W, type_env)` は `is_union` の行で `UnitStep::Unit` を返すので、`rc_units_go` は `[]` を
  積む。`truncate_to_unit` は最初の添字で `UnitStep::Unit` を見て `break` するので `out` は空のままで
  ある。
  BY D5, DEF W, <1>2, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go, truncate_to_unit

<1>4. `covered_leaves(W, [0], type_env) = {[0]}`、`covered_leaves(W, [1], type_env) = {[1]}` である。
      一方 `units_under(W, [1], type_env) = [[1]]` であり、その `truncate_to_unit` は `[]` である。
  `<1>2` の 2 つの leaf のうち `[1]` を接頭辞に持つのは `[1]` だけであり、`[1]` の接頭辞である leaf は
  無い。`subtree_type(W, [1], type_env)` は最初の添字で `UnitStep::Unit` を見て `None` を返すので
  `units_under` は `vec![[1]]` を返す。切り詰めは `<1>3` である。
  BY <1>2, <1>3, CODE src/rc_ir/borrow.rs: covered_leaves,
     CODE src/rc_ir/ownership.rs: units_under, subtree_type

<1>5. `B_g` の各 `Retain`/`Release` の path は、その変数の型の `units` の要素である。すなわち `g` は
      A2 を満たす。
  `Release(q, [], ..)` の `q` の型は `A` で `[] ∈ units(A)`、`Release(v, [], ..)` の `v` の型は `U` で
  `[] ∈ units(U)` である。
  BY A2, <1>1, DEF g

<1>6. `AE` の `result_prov` は、`A` の唯一の leaf `[]` に単一の `Fresh` を宣言する。`borrows_operand` は
      既定の偽である。`passthrough_arg_leaves(AE, A, [c], type_env)` は空である。
  `result_prov` は `Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)` であり、`uniform` は
  `boxed_leaf_paths` の各元にその値を置く。`as_arg_projection` は `Fresh` について `None` を返す。
  BY <1>1, CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMArrayUnsafeEmpty,
     CODE src/rc_ir/provenance.rs: Provenance::uniform,
     CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand,
     CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection

<1>7. `leaves(I64) = {}` である。
  BY 5.1 の L7 の <1>1

### 18.3 入力プログラムは D12 の意味で健全である

<1>1. `B_g` の実行路は 2 本であり、`Match` のどちらのアームを選ぶかで決まる。
  BY D2, D3, DEF g

<1>2. `Obl` の初期値は、`p` の inhabited な leaf 1 つ (tag が 0 なら `obj(p, [0])`、1 なら
      `obj(p, [1])`) への参照 1 つと、`obj(q, [])` への参照 1 つである。`c` は leaf を持たない。
  A1 より `g.borrowed_units` は空なのでパラメータの全 unit が所有される (D14)。`18.2 の <1>2` より `W` の
  leaf は `[0]` と `[1]` であり、D16 よりそのうち活性な変位のものだけが inhabited である。`18.2 の <1>1`
  より `A` の leaf は `[]` の 1 つで、unbox union を通らないので inhabited である。
  BY A1, D10, D14, D16, 18.2 の <1>1, 18.2 の <1>2, 18.2 の <1>7

<1>3. tag が 0 の実行路 (arm0) で D11 が成り立つ。
  <2>1. payload 束縛 `y0` は、D9 の移動の表の「unbox union の変位アームの payload 束縛」の行に当たり、
        `Obl` を変えない。`obj(y0, [])` は `obj(p, [0])` である。
    BY A4, D9, 18.2 の <1>2
  <2>2. `Release(q, [], s, ..)` は `obj(q, [])` への参照を 1 つ取り除く。`<1>2` よりその参照は `Obl` に
        ある。
    BY D10, 18.2 の <1>1, <1>2
  <2>3. アーム本体の `Ret(y0)` は D9 の移動の表の「`Match` のアーム本体の `Ret`」の行に当たり、`Obl` を
        変えない。`obj(m, [])` は `obj(p, [0])` である。
    BY D9, <2>1
  <2>4. 終端の `Ret(m)` の消費は `obj(m, [])` への参照を 1 つ取り除く。`<1>2` と `<2>2` よりその時点の
        `Obl` はその参照 1 つだけを持ち、取り除いた後は空である。
    BY D9, D10, 18.2 の <1>1, <1>2, <2>2, <2>3
  <2>5. QED
    (S-a) は `<2>2` と `<2>4`、(S-b) は `<2>4` による。この路の読む構文は `Let(m, Match(p, arms), ..)`
    だけであり、読みうるのは `obj(p, [0])` で、`<1>2` よりその時点の `Obl` がその参照を持つので `H` は
    正である。
    BY D7, D8, D11, <1>1, <1>2, <2>2, <2>4

<1>4. tag が 1 の実行路 (arm1) で D11 が成り立つ。
  <2>1. payload 束縛 `y1` は移動であり、`obj(y1, [])` は `obj(p, [1])` である。
    BY A4, D9, 18.2 の <1>2
  <2>2. `Let(t, Llvm(MS, [y1, q]), ..)` は `Obl` を変えない。`t` の leaf `[0]` は `obj(p, [1])` を、
        `[1]` は `obj(q, [])` を指す。
    BY A3, D9, D10, 3.3 の <1>1, 3.3 の <1>3, 3.3 の <1>4, 18.2 の <1>1, <2>1
  <2>3. `Let(v, Llvm(MU, [t]), ..)` は `Obl` を変えない。`v` の変位は 0 であり、leaf `[0, 0]` は
        `obj(p, [1])` を、`[0, 1]` は `obj(q, [])` を指し、どちらも inhabited である。
    BY A3, D9, D10, D16, 3.3 の <1>2, 3.3 の <1>3, 3.3 の <1>5, 18.2 の <1>1, <2>2
  <2>4. `Release(v, [], s, ..)` は `obj(p, [1])` への参照 1 つと `obj(q, [])` への参照 1 つを取り除く。
        `<1>2` よりどちらも `Obl` にあり、取り除いた後の `Obl` は空である。
    BY D10, 18.2 の <1>1, <1>2, <2>3
  <2>5. `Let(z, Llvm(AE, [c]), ..)` は `obj(z, [])` への参照を 1 つ作る。`c` は leaf を持たないので消費は
        無い。
    BY A3, D10, 18.2 の <1>6, 18.2 の <1>7
  <2>6. アーム本体の `Ret(z)` は移動であり、終端の `Ret(m)` の消費が `obj(z, [])` への参照を取り除く。
        その後の `Obl` は空である。
    BY D9, D10, 18.2 の <1>1, <2>4, <2>5
  <2>7. QED
    (S-a) は `<2>4` と `<2>6`、(S-b) は `<2>6` による。この路の読む構文は `Let(m, Match(p, arms), ..)`、
    `Let(t, Llvm(MS, [y1, q]), ..)`、`Let(v, Llvm(MU, [t]), ..)`、`Let(z, Llvm(AE, [c]), ..)` の 4 つで
    ある。前 3 つが読みうるのは `obj(p, [1])` と `obj(q, [])` であり、`<1>2` よりその時点の `Obl` は
    どちらの参照も持つ。4 つ目が読むのは `c` であり leaf を持たない。
    BY D7, D8, D11, <1>1, <1>2, <2>2, <2>3, <2>5, 18.2 の <1>7

<1>5. `prog3` は D12 の意味で健全であり、A1 と A2 を満たす。
  BY A1, A2, D12, DEF g, <1>1, <1>3, <1>4, 18.2 の <1>5

### 18.4 `infer_ownership` が返すもの

<1>1. `collect_consumes(B_g, ..)` が `out` に積むのは、`own` が何であっても `(y0, [])`、`(z, [])`、
      `(m, [])` の 3 つである。
  <2>1. `collect_consumes_go` は `Match` の各アーム本体へ降り、そのあと継続へ降りる。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go
  <2>2. arm0 の `Release(q, ..)` は何も積まず、`Ret(y0)` は `push_boxed_leaves(y0, A, ..)` で `(y0, [])` を
        積む。
    BY 18.2 の <1>1, CODE src/rc_ir/ownership.rs: collect_consumes_go, push_boxed_leaves
  <2>3. arm1 の `Let(t, Llvm(MS, [y1, q]), ..)` は何も積まない。`borrows_operand` は両オペランドについて
        偽だが、`y1` の唯一の leaf `[]` は `(0, [])` として、`q` の唯一の leaf `[]` は `(1, [])` として
        `passthrough` に入る。
    BY 18.2 の <1>1, 3.3 の <1>3, 3.3 の <1>4,
       CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>4. arm1 の `Let(v, Llvm(MU, [t]), ..)` は何も積まない。`t` の leaf `[0]`、`[1]` はそれぞれ
        `(0, [0])`、`(0, [1])` として `passthrough` に入る。
    BY 18.2 の <1>1, 3.3 の <1>3, 3.3 の <1>5,
       CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>5. arm1 の `Release(v, ..)` は何も積まない。`Let(z, Llvm(AE, [c]), ..)` は、`c` が leaf を持たない
        ので何も積まない。`Ret(z)` は `(z, [])` を積む。
    BY 18.2 の <1>1, 18.2 の <1>6, 18.2 の <1>7,
       CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes, push_boxed_leaves
  <2>6. 継続の `Ret(m)` は `(m, [])` を積む。
    BY 18.2 の <1>1, CODE src/rc_ir/ownership.rs: collect_consumes_go, push_boxed_leaves
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>2. `origin(y0, []) = Exactly((p, [0]))`、`origin(y1, []) = Exactly((p, [1]))` である。
  `collect_bindings` は `y0` の `Binding` を `Payload(p, Some(0))`、`y1` の `Binding` を
  `Payload(p, Some(1))` にする。`W` は unbox なので `origin_inner` の `Some(tag) if !scrut.ty.is_box` の
  腕を通り、それぞれ `origin(p, [0])`、`origin(p, [1])` へ進む。`p` はパラメータなので `here()` が返る。
  BY DEF W, CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner, VarTable::of

<1>3. `origin(z, []) = Exactly((z, []))` である。
  `z` の `Binding` は `Llvm(AE, [c], A)` である。`decl.leaf_origins_at([])` は `18.2 の <1>6` より
  `{Fresh}` であり、`as_arg_projection` は `None` を返すので `origin_from_leaves_under` へ進む。
  `leaf_origins_under([])` は `{Fresh}` の 1 つだけを返すので `operand_units` は空、`produced_here` は真、
  `reached` は `[Exactly((z, []))]` の 1 元であり、全元が等しいのでそれがそのまま返る。
  BY 18.2 の <1>6, CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner,
     origin_from_leaves_under

<1>4. `origin(m, [])` の候補は `{(p, [0]), (z, [])}` である。
  `m` の `Binding` は `Join([y0, z])` である。`collect_bindings` の `RcRhs::Match` の腕は、各アーム本体に
  ついて `returned_var(&arm.body)` を `arm_results` に積む。arm0 の本体 `Release(q, [], s, Ret(y0))` の
  `returned_var` は `y0`、arm1 の本体の `returned_var` は `z` である。
  `<1>2` と `<1>3` より各アームの `origin` は `Exactly` であり、`acted_on` はその 1 元を返す。2 元は
  等しくないので `of_candidates` は `Join` を返す。
  BY <1>2, <1>3, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var,
     origin_inner の `Binding::Join` の腕, Origin::acted_on, Origin::of_candidates

<1>5. `g` の平準化サイトは `(q, [])` と `(v, [])` の 2 つであり、`origin(v, [])` の候補は
      `{(p, [1]), (q, [])}` である。
  <2>1. `B_g` の `Retain`/`Release` 節点は `Release(q, [], s, ..)` と `Release(v, [], s, ..)` の 2 つで
        あり、`App` 節点は無い。
    BY DEF g, DEF 平準化サイト, CODE src/rc_ir/borrow.rs: levelled_sites
  <2>2. `v` の `Binding` は `Llvm(MU, [t], U)` である。`decl.leaf_origins_at([])` は `None` である
        (`U` の leaf は `[0, 0]` と `[0, 1]` であり `[]` は鍵ではない)。
    BY 18.2 の <1>1, 3.3 の <1>2, CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at
  <2>3. `origin_from_leaves_under` は `operand_units = {(0, [0]), (0, [1])}` を作り、`produced_here` は
        偽であり、`reached = [origin(t, [0]), origin(t, [1])]` である。
    BY 3.3 の <1>2, 3.2 の <1>5, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
  <2>4. `origin(t, [0]) = Exactly((p, [1]))`、`origin(t, [1]) = Exactly((q, []))` である。
    `t` の `Binding` は `Llvm(MS, [y1, q], P)` であり、`decl.leaf_origins_at([0])` は単一の `Arg(0, [])`、
    `[1]` は単一の `Arg(1, [])` なので (`3.3 の <1>1`)、`origin_inner` はそれぞれ `origin(y1, [])` と
    `origin(q, [])` へ進む。前者は `<1>2`、後者は `q` がパラメータなので `here()` である。
    BY 3.3 の <1>1, <1>2, CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner,
       as_arg_projection, VarTable::of
  <2>5. QED
    `<2>4` の 2 元は等しくないので、`origin_from_leaves_under` は `acted_on` を集めた
    `{(p, [1]), (q, [])}` を候補とする `Join` を返す。
    BY <2>1, <2>3, <2>4, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       Origin::acted_on, Origin::of_candidates

<1>6. `infer_ownership(prog3, type_env)` が返す `owned_leaves` は `{(p, [0])}` である。
  <2>1. 消費の段は、`<1>1` の 3 元について `origin` の候補のうち `param_tys` に載るものを入れる。
        `<1>2` より `(y0, [])` の候補は `{(p, [0])}` であり `p` は載る。`<1>3` より `(z, [])` の候補は
        `{(z, [])}` であり `z` は載らない。`<1>4` より `(m, [])` の候補は `{(p, [0]), (z, [])}` であり、
        載るのは `(p, [0])` だけである。よって入るのは `(p, [0])` だけである。
    BY <1>1, <1>2, <1>3, <1>4, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>2. サイト `(q, [])` で `owns_a_candidate` は偽である。
        `origin(q, [])` は `Exactly((q, []))` であり、候補は `{(q, [])}`、`covered_leaves(A, [])` は
        `18.2 の <1>1` より `{[]}` であり、`(q, []) ∈ owned_leaves` は `<2>1` より偽である。
    BY 18.2 の <1>1, <2>1, CODE src/rc_ir/borrow.rs: level_ownership, covered_leaves,
       CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. サイト `(v, [])` で `owns_a_candidate` は偽である。
        `<1>5` より候補は `{(p, [1]), (q, [])}` である。`(p, [1])` について
        `covered_leaves(W, [1]) = {[1]}` (`18.2 の <1>4`) であり `(p, [1]) ∈ owned_leaves` は `<2>1` より
        偽である。`(q, [])` については `<2>2` と同じである。
    BY 18.2 の <1>4, <1>5, <2>1, <2>2, CODE src/rc_ir/borrow.rs: level_ownership, covered_leaves
  <2>4. QED
    `<2>1` の周回で `(p, [0])` が入り、`<2>2` と `<2>3` より平準化は何も入れない。次の周回は
    `<2>1`-`<2>3` を繰り返して何も入れないので `changed` が偽になり、ループは終わる。
    BY <2>1, <2>2, <2>3, CODE src/rc_ir/borrow.rs: infer_ownership

<1>7. `borrow_versions` は `g.name` に `g_borrow` の名を対応させる。
  `g.capture` は `None` である (`DEF g`)。`func_has_borrowable_param` は、`p` の leaf `[1]` について
  `owned_leaves.owns(p, [1])` が `<1>6` より偽なので真を返す。
  BY DEF g, 18.2 の <1>2, <1>6, CODE src/rc_ir/borrow.rs: borrow_ify, func_has_borrowable_param

<1>8. `owned_units` は `{(p, []), (q, []), (p', [])}` である。ここで `p'`、`q'`、`c'` は `rename_g` に
      よる `p`、`q`、`c` の像である。
  <2>1. L2 の (a) が入れるのは `g` のパラメータの unit である。`18.2 の <1>3` より `units(W) = {[]}`、
        `18.2 の <1>1` より `units(A) = {[]}` である。`I64` は `is_fully_unboxed` なので `unit_step` は
        `UnitStep::NoUnit` を返し、`rc_units_go` は何も積まないから `units(I64)` は空である。よって
        `(p, [])` と `(q, [])` である。`g.capture` は `None` なので capture の分は無い。
    BY L2, DEF g, 18.2 の <1>1, 18.2 の <1>3, 5.1 の L7 の <1>1,
       CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>2. L2 の (b) が入れるのは、`<1>7` より `g` について走り、`<1>6` より `owned_leaves.owns` が真なのは
        `(p, [0])` だけなので、`(rename_g[p], truncate_to_unit(W, [0], type_env))` すなわち `(p', [])` で
        ある。
    BY L2, 18.2 の <1>3, <1>6, <1>7
  <2>3. QED
    BY <2>1, <2>2

<1>9. `g_borrow` の `borrowed_units` は `{(q', [])}` である。
  P13 より `borrowed_units` は `param_capture_units(g_borrow, type_env)` のうち `owned_units` に入らない
  ものである。P9 の前半より `g_borrow` のパラメータは `p' : W`、`q' : A`、`c' : I64` であり (型は変わら
  ない)、`18.2 の <1>3` より `units(W) = {[]}`、`18.2 の <1>1` より `units(A) = {[]}`、`I64` は
  `is_fully_unboxed` なので unit を持たない。よって `param_capture_units(g_borrow, type_env)` は
  `{(p', []), (q', [])}` である。`<1>8` より前者は `owned_units` にあり、後者は無い。
  BY P9, P13, DEF g, 18.2 の <1>1, 18.2 の <1>3, <1>8,
     CODE src/rc_ir/borrow.rs: param_capture_units, CODE src/rc_ir/ownership.rs: unit_step

### 18.5 `g_borrow` の本体

`g_borrow` の `VarTable` を `vars'` とし、この節の `origin` は `vars'` について読む。P9 の前半より
`g_borrow` の本体は `B_g` の束縛変数を `rename_g` で一斉に付け替えたものである。付け替えた名前を
`p'`、`q'`、`c'`、`m'`、`y0'`、`y1'`、`t'`、`v'`、`z'` と書く。

<1>1. `owns_object(q', [])` は偽である。
  `vars'.param_tys` は `q'` を `A` で持つ。`units_under(A, [], type_env)` は L1 と `18.2 の <1>1` より
  `[[]]` であり、`truncate_to_unit(A, [], type_env)` は `[]` である。`18.2 の <1>8` より
  `(q', []) ∈ owned_units` は偽である。
  BY L1, L4, 18.2 の <1>1, 18.4 の <1>8, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>2. `owns_object(p', [1])` は**真**である。
  `vars'.param_tys` は `p'` を `W` で持つ。`18.2 の <1>4` より `units_under(W, [1], type_env) = [[1]]` で
  あり、`18.2 の <1>3` より `truncate_to_unit(W, [1], type_env) = []` である。`18.4 の <1>8` より
  `(p', []) ∈ owned_units` である。
  BY L4, 18.2 の <1>3, 18.2 の <1>4, 18.4 の <1>8

<1>3. `owns_unit(v', [])` は偽である。
  T2 の `<1>1` より `origin(v', [])` の候補は `{(p', [1]), (q', [])}` である (`18.4 の <1>5` を
  `rename_g` で写したもの)。`owns_unit` は全元について `owns_object` を要求するが、`<1>1` より
  `owns_object(q', [])` は偽である。
  BY T2 の <1>1, 18.4 の <1>5, <1>1, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>4. `owns_unit(q', [])` は偽である。
  `origin(q', [])` は `Exactly((q', []))` であり、候補は `{(q', [])}` である。`<1>1` による。
  BY <1>1, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
     CODE src/rc_ir/ownership.rs: origin_inner

<1>5. `g_borrow` の本体は次のとおりである。

      ```
      Let(m', Match(p', [
          arm0 (tag 0, payload y0'): Ret(y0'),
          arm1 (tag 1, payload y1'):
              Let(t', Llvm(MS, [y1', q']),
              Let(v', Llvm(MU, [t']),
              Let(z', Llvm(AE, [c']),
              Ret(z'))))
        ]),
      Ret(m'))
      ```

  <2>1. `Release(q', [], s, ..)` は消える。L3 と `<1>4` による。`[] ∈ units(A)` は `18.2 の <1>1` で
        ある。
    BY L3, 18.2 の <1>1, <1>4
  <2>2. `Release(v', [], s, ..)` は消える。L3 と `<1>3` による。`[] ∈ units(U)` は `18.2 の <1>1` で
        ある。
    BY L3, 18.2 の <1>1, <1>3
  <2>3. 残りの節点はそのまま写る。`Let` の 3 つは `rewrite_inner` の `RcExpr::Let(x, rhs, k)` の腕
        (`App` でも `Match` でもない場合) を、`Let(m', Match(..), ..)` は `RcRhs::Match` の腕を、`Ret` は
        `RcExpr::Ret` の腕を通る。`B_g` に `App` は無いので `route` も `call_rc` も呼ばれない。
    BY DEF g, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
  <2>4. QED
    BY <2>1, <2>2, <2>3

### 18.6 `g_borrow` は D11 の (S-b) を破る

<1>1. `Obl` の初期値は、tag が 0 なら `obj(p', [0])` への参照 1 つ、tag が 1 なら `obj(p', [1])` への
      参照 1 つである。`q'` の分は入らない。
  `18.4 の <1>9` より `g_borrow` の `borrowed_units` は `{(q', [])}` なので、D14 より `p'` の unit `[]` は
  所有され `q'` の unit `[]` は借用される。D10 の初期値は所有する unit の下の inhabited な leaf につき
  1 つであり、`18.2 の <1>2` と D16 より `W` の inhabited な leaf は活性な変位のもの 1 つである。
  BY D10, D14, D16, 18.2 の <1>2, 18.4 の <1>9

<1>2. tag が 1 の実行路をとる。`Match` の payload 束縛は移動であり、`obj(y1', [])` は `obj(p', [1])` で
      ある。
  BY A4, D9, 18.2 の <1>2, 18.5 の <1>5

<1>3. 2 つの `Llvm` (`MS` と `MU`) は `Obl` を変えず、`v'` の leaf `[0, 0]` は `obj(p', [1])` を、
      `[0, 1]` は `obj(q', [])` を指す。どちらも inhabited である。
  BY A3, D9, D10, D16, 3.3 の <1>1, 3.3 の <1>2, 3.3 の <1>3, 3.3 の <1>4, 3.3 の <1>5,
     18.2 の <1>1, <1>2

<1>4. `Let(z', Llvm(AE, [c']), ..)` は `obj(z', [])` への参照を 1 つ作る。
  BY A3, D10, 18.2 の <1>6, 18.2 の <1>7

<1>5. アーム本体の `Ret(z')` は移動であり、終端の `Ret(m')` の消費は `obj(m', [])` すなわち
      `obj(z', [])` への参照を 1 つ取り除く。
  BY D9, D10, 18.2 の <1>1, <1>4

<1>6. 終端の `Ret(m')` の消費の後、`Obl` は `obj(p', [1])` への参照 1 つを持つ。
  `<1>1` の初期値からこの路で取り除かれるのは `<1>5` の 1 つだけであり、それは `obj(z', [])` への参照で
  ある。`18.5 の <1>5` よりこの本体に `Release` は無い。
  BY D10, <1>1, <1>2, <1>3, <1>4, <1>5, 18.5 の <1>5

<1>7. `g_borrow` の本体は D11 の (S-b) を破る。
  BY D11, <1>6

<1>8. `borrow_ify(prog3, type_env)` の出力は D12 の意味で健全でない。
  出力の `funcs` は `g_borrow` を含み (`18.4 の <1>7`、L6)、その本体は健全でない (`<1>7`)。
  BY D12, L6, 18.4 の <1>7, <1>7

<1>9. P14 は偽である。
  `18.3 の <1>5` より `prog3` は D12 の意味で健全で A1 と A2 を満たし、`<1>8` より出力は健全でない。
  BY P14, 18.3 の <1>5, <1>8

**実行時に何が起きるか**。`g_borrow` は tag が 1 の呼び出しで `obj(p', [1])` の参照カウントを一度も
下げない。`18.4 の <1>9` よりその unit は `g_borrow` が所有するので、呼び出し元も下げない (D14)。よって
その配列の記憶域は解放されない。

### 18.7 これが平準化の何に当たるか

`R3` が働かせたのは、`level_ownership` の `owns_a_candidate` と `RewriteCtx::owns_object` の**粒度の
食い違い**である。

- `owns_a_candidate` は `covered_leaves(ty(root), ρ)` の leaf が `owned_leaves` にあるかを見る。
  `18.2 の <1>4` より `covered_leaves(W, [1]) = {[1]}` であり、`18.4 の <1>6` よりそれは
  `owned_leaves` に無い。よって平準化は発火しない。
- `owns_object` は `units_under(ty(root), ρ)` を `truncate_to_unit` で写した unit が `owned_units` に
  あるかを見る。`18.2 の <1>4` よりその unit は `[]` であり、`18.4 の <1>8` よりそれは `owned_units` に
  ある。よって `owns_object(p', [1])` は真である。

`owns_object` の答えが正しい。`g_borrow` は `p'` の union unit を所有しているので、活性な変位が 1 でも
その参照は `g_borrow` のものである。誤っているのは `owns_a_candidate` の側であり、それは同じ unit の
別の変位の leaf が所有されていることを見ない。

第 16.3 節の「差が開く形」がこれであり、L13 の (c) がその形を型の側から述べたものである。

**直し方の形**。`level_ownership` の判定を `owns_object` と同じ粒度にする。すなわち、候補 `(root, ρ)` に
ついて見る leaf の集合を `covered_leaves(ty(root), ρ)` から

```
{ λ ∈ leaves(ty(root)) : truncate_to_unit(ty(root), λ) が
                          { truncate_to_unit(ty(root), u) : u ∈ units_under(ty(root), ρ) } の元 }
```

に広げ、判定の側と挿入の側の両方でその集合を使う。L13 の (c) よりこの集合は `covered_leaves` を含むので、
挿入の側の性質 (T2 の `<1>3`) は保たれる。どの直し方を採るかは実装の細部ではなく設計の判断であり、この
文書は形を挙げるにとどめる。

### 18.8 実測

`R3` の形を Fix で書いて測った。`g` の役をする関数は、AST の inliner に畳まれないように `n` についての
算術で嵩を足してある (`INLINE_COST_THRESHOLD` は 30 である --
`CODE src/optimization/inline.rs: INLINE_COST_THRESHOLD`)。この嵩は `p` にも `q` にも触れないので、
平準化サイトを増やさない。

```fix
module Main;

type U = union { a : Array I64, b : Array I64 };
type V = union { twins : (Array I64, Array I64), nothing : () };

f : I64 -> U -> Array I64 -> Array I64;
f = |n, p, q| (
    let m = n + 1;
    // ここに算術の鎖が 19 段続く (inliner の閾値を越えさせるためだけのもの)
    let m = m * 8;
    match p {
        a(y0) => y0,
        b(y1) => (
            let v = V::twins((y1, q));
            eval v;
            Array::fill(1, m)
        )
    }
);

main : IO () = (
    let arr1 = Array::fill(3, 1);
    let arr2 = Array::fill(3, 2);
    let u = U::b(arr1);
    let r = f(0, u, arr2);
    println(r.@(0).to_string + " " + arr2.@(0).to_string)
);
```

`fix build -O max --emit-rc-ir all` が出す `rc_ir.post.txt` で、`Main::f#...#borrow` のパラメータ注釈は
`p` が `{own}`、`q` が `{borrow}` であり、その `b` 変位のアームには `union_make_0` と `eval` はあるが
`release` が無い。`main` はこの借用版を呼ぶ。

valgrind の結果は次のとおりである。

| 最適化 | definitely lost |
|---|---|
| `-O basic` | 0 bytes in 0 blocks |
| `-O max` | **32 bytes in 1 blocks** |

漏れる 32 バイトは `main` が `Array::fill(3, 1)` で割り当てた配列、すなわち `U::b` の payload である。

対照実験として `a` 変位のアームを `y0` を返さない形 (`(eval y0; Array::fill(1, m))`) に替えると、
`owned_leaves` は `p` のどの leaf も持たなくなり、借用版の `p` の注釈は `{borrow}` になり、
`definitely lost` は 0 に戻る。漏れは「`p` の一方の変位の leaf だけが所有されている」ことに依っている。

## 19. P11 の「ちょうど埋める」

P11 の第 1 文の「呼び出し元と呼び出し先の所有権の食い違いをちょうど埋める」を実行時の義務集合 (D10) に
ついての主張として読むと、次の命題になる。第 8 節が `call_rc` の記述としての P11 を証明しているので、
ここでは収支だけを見る。

**命題 (呼び出しの収支)**。ある版の本体の `Let(x, App(callee, args), k)` について、次を仮定する。

- (i) H2 がこの節点の各引数の各 unit について成り立つ。
- (ii) 呼び出しの直前の位置で、`Obl` は、その位置の各スロット `(w, λ)` のうち `origin(w, λ)` の候補が
  すべて `ctx.owns_object` を満たすものの参照をちょうど 1 つずつ持ち、それ以外の参照を持たない。

このとき、`call_rc` が置いた `before` の `Retain` 群を実行し、呼び出しの消費を行い、`after` の
`Release` 群を実行した後の `Obl` は、引数の各 unit について収支が合っている。すなわち、`before` と
`after` が置く節点は過不足なく食い違いを埋め、その各操作は (S-a) を破らない。

<1>1. 引数 `args[i]` と `u ∈ units(ty(args[i]))` を 1 つとる。`u` を接頭辞に持つ inhabited な leaf の
      全体を `Λ` と書く。
  BY D16, P1

<1>2. `arg_owned(i, u)` が真ならば `Obl` は `Λ` の各 leaf の参照を 1 つずつ持ち、偽ならば `Λ` の
      どの leaf の参照も持たない。
  H2 の同値を各 `λ ∈ Λ` に適用すると、`X(λ)` と `arg_owned(i, u)` は一致する。(ii) より `X(λ)` が真の
  スロットの参照だけが `Obl` にある。
  BY (i), (ii), H2, <1>1

<1>3. `callee_owns(i, u)` が真ならば呼び出しは `Λ` の各 leaf の参照を 1 つずつ消費し、偽ならば
      `Λ` のどの leaf の参照も消費しない。
  P11 より `callee_owns(i, u)` は `(params[i].0, u) ∈ owned_units` である。P13 より呼び出し先の版の
  `borrowed_units` はその版のパラメータ unit のうち `owned_units` に入らないものなので、
  `callee_owns(i, u)` は「呼び出し先がその unit を所有する」(D14) と同値である。D9 の `App` の行は、
  呼び出し先が所有する unit の下の引数 leaf を消費する。添字が範囲内であることは A14 による。
  BY A14, D9, D14, P11, P13, <1>1

<1>4. `before` に `(args[i], u)` が入るのは `callee_owns` が真で `arg_owned` が偽のときであり、そのとき
      置かれる `Retain(args[i], u)` は `Λ` の各 leaf の参照を 1 つずつ作る。
  BY D10, P11, <1>1

<1>5. `after` に `(args[i], u)` が入るのは `callee_owns` が偽で `arg_owned` が真のときであり、そのとき
      置かれる `Release(args[i], u)` は `Λ` の各 leaf の参照を 1 つずつ取り除く。
  BY D10, P11, <1>1

<1>6. CASE `callee_owns` が真、`arg_owned` が偽。
  `<1>2` より `Obl` は `Λ` の参照を持たず、`<1>3` より呼び出しは `Λ` の各参照を 1 つずつ消費する。
  `<1>4` の `Retain` がちょうどその不足を作る。`Retain` は `Obl` から取り除かないので (S-a) を破らない。
  BY <1>2, <1>3, <1>4

<1>7. CASE `callee_owns` が偽、`arg_owned` が真。
  `<1>2` より `Obl` は `Λ` の各参照を 1 つずつ持ち、`<1>3` より呼び出しは何も消費しない。`<1>5` の
  `Release` がちょうどその余りを取り除く。取り除く参照はその時点の `Obl` にあるので (S-a) を破らない。
  `Release` は呼び出しの後に置かれるので、呼び出し先が読んでいる間はカウントが下がらない。
  BY <1>2, <1>3, <1>5, P11

<1>8. CASE `callee_owns` と `arg_owned` が一致する。
  どちらも真なら `<1>2` の参照を `<1>3` の消費がちょうど取り除き、どちらも偽なら `Obl` は `Λ` の参照を
  持たず消費も無い。P11 よりこの場合 `before` にも `after` にも何も入らない。
  BY P11, <1>2, <1>3

<1>9. QED
  `<1>6`-`<1>8` が 4 通りを尽くす。unit ごとに独立であり、相異なる unit の下の leaf は互いに素である
  (P1)。
  BY P1, <1>6, <1>7, <1>8

**この命題は H2 に依る。** 第 18 節より H2 はこのコミットのコードでは偽なので、これで P11 の「ちょうど
埋める」が閉じたわけではない。閉じるのは H2 が成り立つようにコードが直ってからである。

## 20. A13 と `check_clone_names_are_fresh`

`borrow_ify` に、複製が導入する名前が入力のどの束縛名とも異なることを検査する表明が入った
(`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`)。この節はそれが A13 の何を変えるかを述べる。

<1>1. 表明が検査するのは A13 の言明ではなく、P9 の後半 (6.2 の言明) の結論そのものである。
  `check_clone_names_are_fresh` は、入力プログラムの各関数のパラメータ・capture の名前と、各関数本体と
  各グローバル初期化子の `for_each_var` が訪れる変数の名前を `bound` に集め、`clones` の各 `rename` の
  **値**が `bound` に入らないことを表明する。`rename` の値は `clone_func` が導入する名前である
  (6.1 の `<1>3`)。
  BY 6.1 の <1>3, 6.2 の言明, CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh, clone_func

<1>2. `bound` は入力の束縛名の上位集合である。
  `for_each_var` は `Let` の束縛子、`Destructure` の容器とフィールド変数、`MatchArm` の payload、および
  値として読まれるすべての変数を訪れる。DEF 入力の束縛名 が数える 3 種はこの中に含まれる。
  BY DEF 入力の束縛名, CODE src/rc_ir/ast.rs: for_each_var, for_each_var_of_node, for_each_var_of_rhs

<1>3. 表明は `develop_mode` のときだけ走る。
  `borrow_ify` は `if develop_mode { check_clone_names_are_fresh(..) }` を実行する。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>4. QED
  A13 の「果たす者」は変わらない。名前を作るのは依然として `Lowerer::fresh_var` と `clone_fresh` で
  あり、表明はそれらの振る舞いを変えない。変わるのは「検査」の欄であり、「無し」から
  「`check_clone_names_are_fresh` (`develop_mode` のときだけ走る)」になる。
  develop build では、P9 の後半 (6.2) は A13 を経由せずにこの表明から出る。`<1>2` より表明が見る集合は
  束縛名より広いので、表明が通れば 6.2 の言明が成り立つ。release build では表明が走らないので、6.2 の
  言明は A13 に依ったままである。
  BY 6.2 の言明, A13, <1>1, <1>2, <1>3

**系 (6.3 についても同じ)**。6.3 (出力の束縛名は互いに相異なる) が H3 を使うのは、`<1>2` と `<1>5` の
名前の側と `<1>3` の名前の側が相異なることを言う一点である。develop build ではその一点が表明から出るので、
6.3 は H3 に依らない。

## 21. `README.md` へ差し戻す点 (この回の追加)

第 13 節の各項目のうち、次の 2 つは解消した。

- **「unit path の `origin` と、その下の leaf の `origin` の関係を述べる命題が無い」** -- P7a が入った。
  第 16.2 節がそれを使う。
- **「P9 の後半には入力の束縛名の形についての仮定が要る」** -- A13 が入った。第 20 節が、表明が入った
  ことで何が変わるかを述べる。**A13 の「検査」の欄を「無し」から
  `check_clone_names_are_fresh` (`develop_mode` のときだけ走る) に改めるのがよい。**

新しく差し戻す点は次のとおりである。

### P14 は依然として偽である

第 18 節が反例 `R3` である。README 第 8 節の #530 の記述は、「1 つの unit の下の leaf が別々の root に
由来して所有が分かれる」ことを原因として挙げている。平準化の段はその分かれ方のうち
「`origin` の候補が leaf 粒度で見て所有と借用に分かれる」場合を閉じたが、
「候補の 1 つが unbox union の別の変位を通じて unit 粒度で所有されている」場合を閉じていない。
P14 の言明は変えずにコードを直すのが筋である。直し方の形は第 18.7 節にある。

### `level_ownership` と `owns_object` の粒度が違うことを述べる場所

第 12.1 節が「`infer_ownership` の `owns` は leaf 粒度、後段の読み手は unit 粒度」を観察として置いて
いる。平準化の段はその後段の読み手の 1 つでありながら、判定を leaf 粒度で行う。README がこの食い違いを
仮定や命題として持つ必要は無いが、第 8 節の #530 の記述を「平準化が閉じた形」と「閉じていない形」に
分けて書くとよい。

### H1 は平準化では成り立たない

第 15.4 節の `<1>3` が、平準化が H1 を成り立たせない入力 (第 18 節の `g`) を与える。P8 (c) が H1 を
必要とすることは変わらない。README の P8 の項が「どの割り当ての下で読むか」を書くとき、この点も併せて
書くとよい。

### `covered_leaves` の doc の第 2 文

`covered_leaves` の doc は「a path into a variant runs below the leaf a punched array holds」と述べて
`path.starts_with(leaf)` の側を正当化しているが、L12 の系より、`level_ownership` が渡す path について
その側が真になるのは path が leaf そのものであるときに限る。すなわちこの側は、この呼び出し方では
`leaf.starts_with(path)` の側が拾わない leaf を 1 つも足さない。防御としては無害だが、doc が挙げる根拠は
成り立っていない。これはコードの欠陥ではなく doc の欠陥である。

## 22. 証明のどこで止まったか (この回)

P14 の証明として自然な形は、第 14 節が述べた不変条件による帰納法であり、それが要求するのは H2 (第 16.1
節) である。H2 の前半は P7a から出た (第 16.2 節)。**後半は偽であり** (第 16.3 節)、その反例が `R3`
(第 18 節) である。

止まった原因は、第 14 節のときと同じく、証明の運びでも命題の前提でもなくコードである。前回は
`owns_unit` の真偽値 1 つが leaf ごとの所有を表せないことであった。今回は、その食い違いを閉じるために
入った平準化の段が、閉じるべき述語 (`owns_object`) とは別の述語 (`covered_leaves` による leaf 粒度の
所有) で発火を決めていることである。
