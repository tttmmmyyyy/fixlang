# P8 - P14 -- `borrow_ify` の健全性

対象コミット `b81cc2c8e859a00cbf007e4f43483a514c813c73`。定義・仮定・命題の番号は同ディレクトリの
`README.md` による。

## 0. この文書の状態

**証明は完成していない。P14 の周りでコードの誤りを見つけたので、そこで止めた。**

持ち帰ったものは 3 つである。

1. **発見 (コードの誤り、第 3 節)。** P14 は偽である。D12 の意味で健全で A1 と A2 を満たすプログラムを
   `borrow_ify` に与えて、出力の借用版が D11 の (S-b) を破る例がある (`R2`)。原因は、所有 (D14) が leaf ごとの
   性質であるのに `owns_unit` が unit ごとの真偽値を答えることである。1 つの unit の下の leaf が別々の root に
   由来し、その root の所有が分かれるとき、`rewrite_rc` はその unit の `Release` を丸ごと落とし、所有する側の
   参照が処分されないまま残る。反例は分岐を要さず、`Match` も呼び出しも含まない 5 節点の本体である。
2. **観察 3 つ (第 4 節)。** `infer_ownership` が `collect_consumes` に渡す `owns` の粒度、グローバル初期化子の
   `RewriteCtx` の不変条件、capture を持つ関数の扱い。後の 2 つは食い違いが無いことを確かめた記録である。
3. **README への要望 (第 5 節)。**

第 2 節は反例が使う補題である。これらは P10 と P13 の一部でもあるので、証明を再開する者はそのまま使える。

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

## 2. 反例が使う補題

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

## 4. 観察

### 4.1 `infer_ownership` の `owns` は leaf 粒度、後段の読み手は unit 粒度

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
この橋渡しが不動点で成り立つことを P8 に委ねている。

反例 `R2` はこの食い違いを使っていない。`R2` の `f` には呼び出しが無く、`owned_leaves` の粒度は
`owned_units` の粒度と一致している。よってこの食い違いは `R2` とは独立であり、`R2` を直しても残る。

### 4.2 グローバル初期化子の `RewriteCtx`

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

### 4.3 capture を持つ関数

`borrow_versions` に入るのは `func.capture.is_none()` の関数だけである
(`CODE src/rc_ir/borrow.rs: borrow_ify`)。`owned_units` へ (b) の元を入れるループは `func.params` だけを
回り、`func.capture` を回らない (L2 の (b))。この 2 つは対応している。`clone_func` は
`fresh_rename_function` に `&func.capture` を渡し、返った `capture` を `RcFunc` に入れるが
(`CODE src/rc_ir/borrow.rs: clone_func`)、`borrow_versions` に載る関数の `capture` は `None` なので、
借用版の `capture` も `None` である。`param_capture_units` は `params` と `capture` を鎖にして回るので、
借用版についてはパラメータだけを返す。`borrowed_units` に capture の unit が現れることは無い。

## 5. `README.md` へ差し戻す点

### P14 が偽であること

第 3 節が反例である。P14 の言明は変えずに、コードを直すのが筋である。直し方の形は 2 つ考えられるが、どちらも
この文書は検証していない。

- **形 1**: 1 つの unit の下の leaf の所有が分かれないようにする。`infer_ownership` の不動点に、
  「ある `Retain`/`Release` 節点または引数位置の unit `(v, u)` について `origin(v, u).candidates()` の
  いずれかが所有されるなら、その candidates はすべて所有される」という閉包を足す。
- **形 2**: 借用版を作る条件を狭める。`func_has_borrowable_param` に、その関数のどの `Retain`/`Release`
  節点・どの引数位置についても `origin(v, u).candidates()` の所有が分かれないこと、を足す。

### P11 の「ちょうど埋める」

P11 の言明は 2 文からなり、第 2 文 (「すなわち、呼び出し元が借用し呼び出し先が所有する unit には前に
`Retain` を ...」) は `call_rc` の記述として正しい。第 1 文の「ちょうど埋める」は、第 3.9 節が述べる意味で
偽である。unit の下の leaf の所有が分かれるとき、`Retain(arg, unit)` は D10 より不足していない leaf の
参照まで作る。P11 を使う証明が「ちょうど」に依るなら、そこも閉じない。

### P8 の言明が読む D14 の割り当て

P8 の「D9 の意味で消費される」は、D9 の `App` の行が D14 の所有を読むので、どの割り当ての下での消費かを
言わないと定まらない。入力の割り当て (A1、全所有) で読むと、`collect_consumes` が `owned_leaves` を渡す
のと合わない。出力の割り当て (`owned_units` が定めるもの) で読むのが `borrow_ify` の使い方であり、そのとき
第 4.1 節の粒度の差が P8 の証明の中心になる。README がどちらであるかを書くとよい。

## 6. 証明のどこで止まったか

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
