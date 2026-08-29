# P8 - P14 -- `borrow_ify` と RC 規律の保存

対象コミット `2d49d350d7f79351c57029e8ab9e8fdfa45afacc`。定義・仮定・命題の番号は同ディレクトリの
`README.md` による。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P8 (a) 停止性 | 証明した (第 3.3 節)。平準化の周回を含む |
| P8 (b) 不動点の閉包 | 証明した (第 3.4 節) |
| P8 (c) D9 の消費との対応 | 証明した (第 3.5 節)。`README.md` の P8 が文面に持つとおり `App` の引数の位置を除く形であり、その位置を P14 は `call_rc` が置く節点で扱う (第 3.6 節) |
| P9 複製は名前替えである | 証明した (第 4 節)。後半は A13 に立つ |
| P10 借用版が落とす RC 節点 | 証明した (第 5 節) |
| P11 呼び出し側の補正 | 証明した (第 6 節)。「ちょうど埋める」は P14 の中で示す (第 10 節) |
| P12 振り分けの安全性 | 証明した (第 7 節)。`funcs_observing_uniqueness` の門を含む |
| P13 注釈の一致 | 証明した (第 8 節) |
| P14 `borrow_ify` は RC 規律を保存する | **証明した** (第 10 節)。(S-a) と (S-b) は A19 (ii-a) の下で、(S-c) は A19 (ii-a) と A20 の下で |

**この文書は局所の仮説を置かない。** P14 が読む 2 つ -- A19 (ii-a) (別名類の持つ参照は非負であり、読む
構文と `Retain`/`Release` がその類を名指す時点では 1 以上) と A20 (借りた参照は活性化の間 生きている) --
はどちらも `README.md` の仮定である。第 9.1 節がこの 2 つを、この文書の記法 (由来) へ渡す。

**P14 の実質は第 9 節にある。** 出力の各版の義務集合は、入力の義務集合を**由来 (DEF 由来) ごとに**
分けたときの、その版が所有する由来の分にちょうど等しい。借用する由来の分は呼び出し元が持ち、`call_rc` が
置く `Retain` は、借用する由来の参照を消費が要求する位置でだけ 1 つ作って同じ位置で消費させる。この
「由来ごとの収支」が、P11 の言明が保留した「ちょうど埋める」である。

## 1. 記法

`origin(x, π)` は `origin(vars, type_env, &x, &π)` の略記とする。`vars` はその時点で問題にしている版の
`VarTable` である。`VarPath` を `(x, π)` と書く。

`leaves(τ)` は `boxed_leaf_paths(τ, type_env)`、`units(τ)` は `rc_units(τ, type_env)`、
`trunc(τ, π)` は `truncate_to_unit(τ, π, type_env)`、`under(τ, π)` は `units_under(τ, π, type_env)`、
`sub(τ, π)` は `subtree_type(τ, π, type_env)` とする
(`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`, `CODE src/rc_ir/ownership.rs: rc_units`,
`truncate_to_unit`, `units_under`, `subtree_type`)。

`cand(x, π)` は `origin(x, π).candidates()` を集合とみなしたもの、`act(x, π)` は
`origin(x, π).acted_on()` を集合とみなしたものである
(`CODE src/rc_ir/ownership.rs: Origin::candidates`, `Origin::acted_on`)。

`Obl` は D10 の義務集合、`H(o)` は D7 の参照カウントとする。`σ ⊑ π` は「`σ` は `π` の接頭辞である」。

`borrow_ify` の局所変数を次の名で参照する (`CODE src/rc_ir/borrow.rs: borrow_ify`)。

- `owned_leaves`: `infer_ownership` が返す `OwnedLeaves` の中身。leaf の集合である
  (`CODE src/rc_ir/borrow.rs: OwnedLeaves`)。以下では `OL` とも書く。
- `owned_units`: `borrow_ify` が組み立てる `Set<VarPath>`。unit の集合である。
- `borrow_versions`: 借用版を持つ関数からその版の名前への `Map`。
- `observing`: `funcs_observing_uniqueness(prog, type_env)` の値
  (`CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness`)。

**DEF f_own** -- 入力の関数 `f` について、`borrow_ify` が `funcs` に `f.name` の名で入れる版をいう
(`CODE src/rc_ir/borrow.rs: borrow_ify` の `for func in prog.funcs.values()` の 4 番目のループ)。

**DEF f_borrow** -- `borrow_versions` が `f.name` に対応させる名の版をいう。`clone_func` が作る
(`CODE src/rc_ir/borrow.rs: borrow_ify` の `for (borrow_version, mut clone, _) in clones` のループ)。

**DEF rename_f** -- `borrow_versions` に載る入力の関数 `f` について、`clone_func(f, ..)` が返す束縛の
付け替え `Map<FullName, FullName>` をいう (`CODE src/rc_ir/borrow.rs: clone_func`)。`ρ_f` は `rename_f` を
その鍵でない名前の上の恒等写像で延ばしたものとする。

**DEF 入力の束縛名** -- `borrow_ify` の入力プログラムの、ある関数のパラメータ・capture の名前、または
ある関数の本体かあるグローバル初期化子の本体が束縛する変数の名前をいう。束縛する構文は `Let` の第 1 成分、
`Destructure` のフィールド変数、`MatchArm` の `payload` の 3 つである
(`CODE src/rc_ir/ast.rs: RcExpr`, `MatchArm`)。**DEF 出力の束縛名** -- 同じものを出力プログラムについて言う。

**DEF 出力の版** -- 出力プログラムの `funcs` の各元と、出力の各グローバル初期化子をいう。第 8 節の
`L6` より、`funcs` の元は各 `f_own` と各 `f_borrow` である。

**`develop_mode` について。** `borrow_ify` は `develop_mode` が真のとき `check_clone_names_are_fresh` と
`RewriteCtx::check_ownership_is_levelled` を呼ぶ。どちらも `assert!` を行うだけで出力を作らない
(`CODE src/rc_ir/borrow.rs: borrow_ify`, `check_clone_names_are_fresh`,
`RewriteCtx::check_ownership_is_levelled`)。よって以下のすべての命題は `develop_mode` の値によらない。
表明が発火する入力では `borrow_ify` は出力を返さないので、そのときも命題は真である。

## 2. 補題

### L1 (A2 の path の下の unit はその path 自身だけである)

**言明**。`π` が `units(τ)` の要素であるとき、`under(τ, π)` は `[π]` である。

<1>1. `rc_units_go` が `out` に積む path は、`UnitStep::Fields` の腕で積んだ添字の列 `ρ` に、
      `UnitStep::Unit` の腕では何も足さず、`UnitStep::Capture` の腕では `capture_idx` を 1 つ足したもので
      ある。
  BY CODE src/rc_ir/ownership.rs: rc_units_go

<1>2. `π` が `units(τ)` の要素であるとき、`π` は `ρ` (`Unit` の腕で積まれた場合) か `ρ ++ [c]`
      (`Capture` の腕で積まれた場合) の形である。ここで `ρ` の各添字は、その位置の型の `unit_step` が
      `UnitStep::Fields` を返し、その `held_fields` が持つ添字である。
  BY <1>1, CODE src/rc_ir/ownership.rs: rc_units, rc_units_go

<1>3. `sub(τ, ρ)` は、`ρ` の各添字について `UnitStep::Fields` の腕を通り、`held_field_type` でその添字の
      型へ降りて、`Some(σ)` を返す。`σ` は `ρ` が指す部分木の型である。
  BY <1>2, CODE src/rc_ir/ownership.rs: subtree_type, held_field_type

<1>4. CASE `π = ρ` (`Unit` の腕で積まれた場合)。
  <2>1. `unit_step(σ, type_env)` は `UnitStep::Unit` である。
    BY <1>2, <1>3, CODE src/rc_ir/ownership.rs: rc_units_go の `UnitStep::Unit` の腕
  <2>2. `rc_units(σ, type_env)` は `[[]]` である。
    BY <2>1, CODE src/rc_ir/ownership.rs: rc_units, rc_units_go の `UnitStep::Unit` の腕
  <2>3. `under(τ, π)` は `sub` が `Some(σ)` を返す腕を通り、`rc_units(σ)` の各元を `π` の後ろに繋いだ
        ものを返す。`<2>2` よりそれは `[π ++ []] = [π]` である。
    BY <1>3, <2>2, CODE src/rc_ir/ownership.rs: units_under

<1>5. CASE `π = ρ ++ [c]` (`Capture` の腕で積まれた場合)。
  <2>1. `unit_step(σ, type_env)` は `UnitStep::Capture { capture_idx: c, .. }` である。
    BY <1>2, <1>3, CODE src/rc_ir/ownership.rs: rc_units_go の `UnitStep::Capture` の腕
  <2>2. `sub(τ, π)` は、`ρ` を降りた後の添字 `c` で `UnitStep::Capture` の腕に入り、`None` を返す。
    BY <1>3, <2>1, CODE src/rc_ir/ownership.rs: subtree_type の
       `UnitStep::NoUnit | UnitStep::Capture { .. } | UnitStep::Unit` の腕
  <2>3. `under(τ, π)` は `None` の腕を通り、`vec![π]` を返す。
    BY <2>2, CODE src/rc_ir/ownership.rs: units_under

<1>6. QED
  `<1>2` の 2 つの形が場合を尽くしており、`<1>4` と `<1>5` がそれぞれを与える。
  BY <1>2, <1>4, <1>5

### L2 (`owned_units` に入るもの)

**言明**。`borrow_ify` が組み立てる `owned_units` は、次の 2 種の元だけからなる。

- (a) 入力の各関数 `f` の各パラメータ・capture `p` と各 `unit ∈ units(ty(p))` について `(p.name, unit)`。
- (b) `borrow_versions` に載る各関数 `f` の各パラメータ `p` と、`owned_leaves.owns(p.name, λ)` が真である
  各 `λ ∈ leaves(ty(p))` について `(rename_f[p.name], trunc(ty(p), λ))`。

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
      `p` と `leaves(ty(p))` の各元 `leaf` について、`owned_leaves.owns(&p.name, &leaf)` が真のときに
      `(rename_f[p.name], trunc(ty(p), leaf))` を入れる。
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

<1>2. `rewrite_rc` は `self.is_borrow_version` が真のとき、`under(ty(v), path)` を
      `self.owns_unit(v, unit)` で絞った `kept` を作り、`kept` の各元について `rc_node` を 1 つ重ねる。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>3. `π ∈ units(ty(v))` なので `under(ty(v), π)` は `[π]` である。
  BY L1

<1>4. QED
  `<1>2` の `kept` は、`<1>3` より `owns_unit(v, π)` が真なら `[π]`、偽なら空である。`rc_node` は
  `is_release` の真偽でそれぞれ `RcExpr::Release` と `RcExpr::Retain` を作る。
  BY <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: rc_node

### L4 (`owns_object` の値)

**言明**。`owns_object(root, path)` は、`self.vars.param_tys` が `root` を持たないとき真であり、持つとき
(その型を `τ`)、`under(τ, path)` の各 `unit` について `(root, trunc(τ, unit))` が `self.owned_units` に
入ることと同値である。

<1>1. `owns_object` は `self.vars.param_tys.get(root)` で場合分けし、`None` の腕で `true` を返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>2. `Some(root_ty)` の腕は `under(root_ty, path)` の各 `unit` について
      `self.owned_units.contains(&(root.clone(), trunc(root_ty, unit)))` を要求する。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>3. QED
  BY <1>1, <1>2

### L5 (`RewriteCtx::rewrite` は束縛を導入も除去もしない)

**言明**。任意の `RewriteCtx` と任意の本体 `B` について、`ctx.rewrite(B)` が束縛する変数名の集合は、`B` が
束縛する変数名の集合に等しい。

<1>1. `rewrite` は `grow_stack(|| self.rewrite_inner(node))` であり、A15 より `grow_stack(f)` は `f()` を
      1 回だけ呼んでその値を返す。
  BY A15, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite, CODE src/misc.rs: grow_stack

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
        0 個以上重ねる。`<1>2` よりどれも束縛しない。帰納法の仮定より `k` の束縛は変わらない。
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

<1>2. `borrow_versions` の鍵は入力の関数の名前だけである。1 番目のループは、`observing` が
      `func.name` を含むとき `continue` し、含まず `func.capture.is_none()` かつ
      `func_has_borrowable_param(func, &owned_leaves, type_env)` のときに
      `borrow_versions.insert(func.name.clone(), borrow_funcref(&func.name))` を行う。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, funcs_observing_uniqueness, func_has_borrowable_param

<1>3. `borrow_versions` の値はすべて `clones` の第 1 成分として現れる。2 番目のループは入力の各関数に
      ついて `borrow_versions.get(&func.name)` を引き、`Some` のとき `clones.push((borrow_version, ..))` を
      行う。`<1>2` より `borrow_versions` の鍵はすべて入力の関数の名前なので、このループはそのすべてを
      引き当てる。
  BY <1>2, CODE src/rc_ir/borrow.rs: borrow_ify

<1>4. 出力の `funcs` に元を入れるのは 2 か所である。入力の各関数について
      `funcs.insert(f_own.name.clone(), f_own)` (`f_own.name` は `func.name` である)、`clones` の各元に
      ついて `funcs.insert(borrow_version, clone)` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>5. QED
  BY <1>1, <1>3, <1>4

## 3. P8 -- 推論の停止性と安全性

### 3.1 P8 の言明が読む所有権の割り当て

P8 の後半は「D9 の意味で消費される」と言う。D9 の `App` の行は「呼び出し先がその位置の unit を所有する
(D14) 引数の leaf」であり、D14 の所有は `RcFunc::borrowed_units` が定める。よって「どの割り当ての下での
消費か」を決めないと言明が定まらない。候補は 2 つある -- 入力の割り当て (A1 より全所有) と、
`infer_ownership` が計算している割り当てである。L7 が前者を退ける。

#### L7 (入力の割り当てで読むと P8 は偽である)

**言明**。D12 の意味で RC 規律を満たし A1 と A2 を満たす入力プログラム `Q` で、次を満たすものがある。
`Q` のある関数のあるパラメータ leaf の参照が、A1 の割り当て (全所有) の下で D9 の意味で消費される実行路が
あるのに、その leaf は `infer_ownership(Q, type_env)` が返す `owned_leaves` に入らない。

**`Q` の定義**。`A` を `Array I64` とする。`s` は任意の `RcState` とする。

- `g`: funptr ABI (`capture` は `None`)、パラメータは `y : A` と `n : I64`、`ret_ty` は `I64`。
  本体は `Release(y, [], s, Ret(n))`。
- `f`: funptr ABI、パラメータは `x : A` と `m : I64`、`ret_ty` は `I64`。本体は
  `Let(w, App(gv, [x, m]), Ret(w))`。ここで `gv` は `g` の名前を持つ `RcVar` で、その型は `g` の funptr 型、
  `w : I64` である。
- `funcs = {f, g}`、`globals = []`、`roots = {f}`、両方の `borrowed_units` は空。

<1>0. `leaves(A) = {[]}` かつ `units(A) = {[]}` である。
  `is_array` が真なので `boxed_leaf_paths` の `go` は自分自身の位置 `[]` を積んで戻り、`unit_step` は
  `UnitStep::Unit` を返すので `rc_units_go` は `[]` を積む。
  BY D4, D5, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step,
     rc_units_go

<1>1. `leaves(I64) = {}` であり、`leaves(gv.ty) = {}` である。
  `I64` は `is_box` でも `is_closure` でも `is_array` でもなく、`unpunched_field_types` が空なので
  `is_fully_unboxed` が真である。funptr 型は `is_funptr` の行で `is_fully_unboxed` が真である。
  `boxed_leaf_paths` は `is_fully_unboxed` の型について何も積まない。
  BY D4, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `Q` は A2 を満たす。
  `Q` の `Retain`/`Release` 節点は `Release(y, [], s, ..)` の 1 つだけであり、`<1>0` より `[] ∈ units(A)` で
  ある。
  BY A2, <1>0

<1>3. `g` の本体は D11 の意味で RC 規律を満たす。
  <2>1. `g` の実行路は 1 本であり、節点の列は `Release(y, [])`、`Ret(n)` である。
    BY D2, D3
  <2>2. `Obl` の初期値は `obj(y, [])` への参照 1 つである。`n` は `<1>1` より leaf を持たない。
    BY A1, D10, D14, D16, <1>0, <1>1
  <2>3. `Release(y, [], s, ..)` の後、`Obl` は空である。
    BY D10, <1>0, <2>2
  <2>4. 終端の `Ret(n)` の消費は何も取り除かない。`<1>1` より `n` は boxed leaf を持たない。
    BY D9, <1>1
  <2>5. QED
    (S-a) は `<2>3` が、(S-b) は `<2>3` と `<2>4` が与える。この実行路に D7 の読む構文は無く、
    `Release(y, [])` が触れる `obj(y, [])` は `<2>2` よりその時点で参照を持つので解放されていない。
    BY D7, D8, D11, <2>1, <2>2, <2>3, <2>4

<1>4. `f` の本体は D11 の意味で RC 規律を満たす。またその唯一の実行路で、`App(gv, [x, m])` は
      パラメータ leaf `(x, [])` の参照を A1 の割り当ての下で D9 の意味で消費する。
  <2>1. `f` の実行路は 1 本であり、節点の列は `Let(w, App(gv, [x, m]))`、`Ret(w)` である。
    BY D2, D3
  <2>2. `Obl` の初期値は `obj(x, [])` への参照 1 つである。
    BY A1, D10, D14, D16, <1>0, <1>1
  <2>3. `App(gv, [x, m])` は `obj(x, [])` への参照 1 つを消費し、参照を作らない。`gv` と `m` は `<1>1` より
        leaf を持たず、`w` も leaf を持たない。`g` の `borrowed_units` は空なので、D14 より `g` は
        `y` の unit `[]` を所有し、D9 の `App` の行より `x` の leaf `[]` は消費される。
    BY A1, D9, D10, D14, <1>0, <1>1
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
    BY <1>0, <1>1, CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes,
       resolve_callee_params, push_boxed_leaves
  <2>3. `owned_leaves` が空のとき、平準化の段も何も挿入しない。
        `levelled_sites(g)` は `Release(y, [])` から `(y, [])` を挙げる。`origin(y, [])` は
        `Binding::Param` の腕で `Exactly((y, []))` を返すので候補は `(y, [])` 1 つであり、
        `owns_object_yet(vars, type_env, y, [], ∅)` は `under(A, [])` の唯一の元 `[]` について
        「`trunc(A, ・) = []` を満たし `∅` に入る leaf」を求めて偽になる。よって `owns_a_candidate` は
        偽で `level_ownership` は `false` を返す。`levelled_sites(f)` は `App` の引数から `(x, [])` と
        `m` の unit を挙げる。`m` は `<1>1` と L8 (`p15-ownership-uniformity.md`) より unit を持たない。
        `(x, [])` は `g` の `(y, [])` と同じ理由で偽になる。
    BY <1>0, <1>1, CODE src/rc_ir/borrow.rs: levelled_sites, level_ownership, owns_object_yet,
       CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. QED
    初期値は空であり、`<2>1`-`<2>3` より 1 周目で `changed` は偽のままである。`insert` が 1 度も真を
    返さないので `owned_leaves` は空のまま返る。
    BY <2>1, <2>2, <2>3, CODE src/rc_ir/borrow.rs: infer_ownership

<1>7. QED
  `<1>4` より、`f` の唯一の実行路で、A1 の割り当ての下でパラメータ leaf `(x, [])` の参照が D9 の意味で
  消費される。`<1>6` よりその leaf は `owned_leaves` に入らない。`<1>5` より `Q` は P8 の前提を満たす。
  BY <1>4, <1>5, <1>6

### 3.2 割り当ての決定

**この文書は P8 を、`infer_ownership` が計算している割り当ての下で読む。** 理由は 3 つである。

- L7 より、A1 の割り当てで読むと P8 は偽である。
- `infer_ownership` が `collect_consumes` に渡す `own` は `owned_leaves` そのものである
  (`CODE src/rc_ir/borrow.rs: infer_ownership`)。この読み方のもとで P8 は「`owned_leaves` が自分自身に
  ついての不動点条件を満たす」という言明になり、コードが計算しているものと言明が一致する。
- P14 が P8 を使う先は借用版である。借用版のパラメータの所有は `owned_units` が定め、`owned_units` の
  借用版の分は `owned_leaves` から `trunc` で作られる (L2 の (b))。

**DEF 推論の割り当て**
入力の各関数 `f` の各パラメータ・capture `p` について、`f` が leaf `(p, λ)` (`λ ∈ leaves(ty(p))`) を
**推論の意味で所有する**とは `(p.name, λ) ∈ owned_leaves` であることをいい、そうでないとき**推論の意味で
借用する**という。

**この割り当ては leaf 粒度であり、D14 の割り当ては unit 粒度である。** `rhs_consumes` の
`is_owning_position` は `owns(&params[i], &leaf)` すなわち leaf ごとの問い合わせであり
(`CODE src/rc_ir/ownership.rs: rhs_consumes`)、D14 の所有は unit ごとである。第 3.5 節が、この差が
D9 の消費の 6 行のうち `App` の引数の行にだけ現れることを述べる。

### 3.3 P8 (a) -- 停止性

**言明**。`infer_ownership(prog, type_env)` は停止する。

<1>1. `owned_leaves` を変える箇所は 2 つである。消費の段の
      `owned_leaves.insert((root_var.clone(), root_path.clone()))` と、`level_ownership` の中の
      `owned_leaves.insert((root.clone(), leaf))` である。どちらも挿入だけで、取り除かない。
  BY CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

<1>2. `changed` が真になるのは、`<1>1` のどちらかの `insert` が真を返したときだけである。消費の段は
      `insert` の返り値で `changed` を立て、平準化の段は `changed |= level_ownership(..)` であり、
      `level_ownership` は `owns_a_candidate` が偽なら `false` を返し、真のときは `insert` の返り値の
      論理和を返す。ループは `changed` が偽のとき `break` する。
  BY CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

<1>3. 消費の段が挿入しうる元の全体は有限である。
  <2>1. `collect_consumes(&func.body, vars, prog, own, type_env, &mut consumed)` が `consumed` に積む対の
        全体は、`own` の値によらない有限集合 `S_func` に含まれる。
    <3>1. `collect_consumes_go` が `out` に積むのは、`RcExpr::Ret` の腕、`RcExpr::Destructure` の腕、および
          `rhs_consumes` の `Closure`・`App`・`Llvm` の腕の 5 か所だけである。
      BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes
    <3>2. `<3>1` の 5 か所が積む対の第 1 成分は、その節点に現れる `RcVar` の名前であり、第 2 成分は
          その変数の型の `boxed_leaf_paths` の元である。`RcExpr::Ret` と `Closure`・`App` の腕は
          `push_boxed_leaves` を、`RcExpr::Destructure` の腕は `destructure_consumes` を、`Llvm` の腕は
          `boxed_leaf_paths` を直に使う。`destructure_consumes` は `boxed_leaf_paths` の絞り込みを返す。
      BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes, push_boxed_leaves,
         destructure_consumes, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <3>3. `own` を読むのは `rhs_consumes` の `App` の腕の `is_owning_position` だけであり、それは積むか
          積まないかを決めるだけで、積む対を変えない。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App` の腕
    <3>4. QED
      D2 より本体は有限の木であり、A10 より各型の `boxed_leaf_paths` は有限である。`<3>2` の対の全体は
      有限であり、`<3>3` よりその全体は `own` に依らない。
      BY A10, D2, <3>1, <3>2, <3>3
  <2>2. 消費の段が挿入するのは、`consumed` の各元 `(var, path)` について `origin(var, path).candidates()` の
        元のうち `vars.param_tys` に鍵 `root_var` を持つものである。
    BY CODE src/rc_ir/borrow.rs: infer_ownership
  <2>3. `var_tables` はループの外で 1 度だけ作られ、`origin` は `vars` と `type_env` だけに依る。よって
        1 つの `(var, path)` に対する `candidates()` は周回によらず同じ有限集合である。`Origin::Exactly` は
        1 元、`Origin::Join` は有限集合の `candidates` を持つ。
    BY P2, CODE src/rc_ir/borrow.rs: infer_ownership,
       CODE src/rc_ir/ownership.rs: origin, Origin, Origin::candidates
  <2>4. QED
    D1 より関数は有限個であり、`<2>1` より各関数の `consumed` に現れうる対は有限、`<2>3` より各対の
    候補は有限である。
    BY D1, <2>1, <2>2, <2>3

<1>4. 平準化の段が挿入しうる元の全体は有限である。
  `level_ownership` が挿入するのは `(root, leaf)` であり、`root` は `vars.param_tys` に鍵を持つ名前
  (`param_tys` に無い `root` は `continue` で飛ばされる)、`leaf` は `covered_leaves(ty(root), path)` の元
  すなわち `leaves(ty(root))` の元である。D1 より関数は有限個、各 `param_tys` は有限、A10 より各型の
  `boxed_leaf_paths` は有限である。
  BY A10, D1, CODE src/rc_ir/borrow.rs: level_ownership, covered_leaves

<1>5. 1 周回の仕事は有限である。
  `var_tables` と `sites` はループの外で 1 度だけ作られる。`levelled_sites` は本体の節点を
  `for_each_node` で 1 度ずつ歩き、有限の列を返す (D2 より本体は有限の木)。各周回は、各関数について
  `collect_consumes` を 1 回 (`<2>1` より有限)、その各元について `origin` を 1 回 (P2 より停止)、
  各 site について `level_ownership` を 1 回呼ぶ。`level_ownership` は `origin` を 1 回呼び (P2)、
  各候補について `owns_object_yet` と `covered_leaves` を呼ぶ。`owns_object_yet` は `under` と
  `boxed_leaf_paths` を呼び、A10 よりどちらも有限で停止する。`collect_consumes` が積む対の全体が有限で
  あることは `<1>3` が与える。
  BY A10, D2, P2, <1>3, CODE src/rc_ir/borrow.rs: infer_ownership, levelled_sites, level_ownership,
     owns_object_yet, covered_leaves, CODE src/rc_ir/ast.rs: for_each_node

<1>6. QED
  `<1>1` と `<1>2` より、`changed` が真になる周回では `owned_leaves` は真に大きくなる。`<1>3` と `<1>4`
  よりその大きさには上界があるので、`changed` が真である周回は有限回しかなく、その次の 1 周で `changed` は
  偽になり `break` する。各周回は `<1>5` より有限の仕事しかしない。
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### 3.4 P8 (b) -- 不動点の閉包

**言明**。`infer_ownership` が返す `owned_leaves` を `OL` とする。入力の各関数 `f` について、`OL` を
`own` として `collect_consumes` を呼んだ結果の各元 `(var, path)` と、`origin(var, path).candidates()` の
各元 `(root_var, root_path)` について、`vars.param_tys` が `root_var` を鍵に持つならば
`(root_var, root_path) ∈ OL` である。

<1>1. ループが `break` する周回では `changed` が偽である。
  BY CODE src/rc_ir/borrow.rs: infer_ownership

<1>2. その周回の間、`owned_leaves` は変わらない。
  `changed` が真になるのは、消費の段の `insert` か `level_ownership` の返り値のどちらかが真のときだけで
  ある (消費の段は `insert` の返り値で `changed` を立て、平準化の段は `changed |= level_ownership(..)` で
  あり、`level_ownership` は `owns_a_candidate` が偽なら `false` を、真なら `insert` の返り値の論理和を
  返す)。`<1>1` よりその周回ではどの `insert` も真を返さない。`insert` が偽を返すとき集合は変わらない。
  BY <1>1, CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

<1>3. その周回で各関数について `collect_consumes` に渡される `own` は、返される `OL` と同じである。
  BY <1>2

<1>4. QED
  その周回の内側のループは、`vars.param_tys` が `root_var` を持つ各 `(root_var, root_path)` について
  `owned_leaves.insert` を呼ぶ。`<1>1` よりそのすべてが偽を返すので、すべてすでに入っている。`<1>3` より
  その `collect_consumes` の入力は `OL` である。
  BY <1>1, <1>3, CODE src/rc_ir/borrow.rs: infer_ownership

### 3.5 P8 (c) -- D9 の消費との対応

**DEF 所有を読まない消費** -- D9 の消費の表のうち、`App` の行の「呼び出し先がその位置の unit を所有する
引数の leaf」を除いた部分をいう。すなわち `App` の callee の全 boxed leaf、`Closure` の各 capture の全
boxed leaf、`Llvm` の消費する leaf、boxed 容器の `Destructure`、unbox 容器の `Destructure`、関数本体の
終端の `Ret` の 6 つである。D9 の消費の表でこの割り当てを読むのは `App` の引数の位置だけである。

**言明**。入力の各関数 `f` のどの実行路のどの位置についても、そこで所有を読まない消費によって消費される
leaf `(v, λ)` について、`origin(v, λ).candidates()` の元のうち `vars.param_tys` に載るものはすべて `OL` に
入っている。

<1>1. 所有を読まない消費が消費する leaf は、`own` の値によらず `collect_consumes` が `out` に積む。
  <2>1. P7 より D9 の意味で消費する構文はすべて `collect_consumes` が報告する。
    BY P7
  <2>2. これを報告する 5 か所 -- `collect_consumes_go` の `RcExpr::Ret` の腕と `RcExpr::Destructure` の腕、
        `rhs_consumes` の `Closure` の腕、`Llvm` の腕、`App` の腕の
        `push_boxed_leaves(&callee.name, ..)` -- は、いずれも `owns` を読まない。`Llvm` の腕が読むのは
        `borrows_operand` と `passthrough_arg_leaves` だけである。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes
  <2>3. QED
    BY <2>1, <2>2

<1>2. QED
  `<1>1` の積まれた元に P8 (b) の閉包条件を適用する。
  BY P8 (b), <1>1

**アーム本体の `Ret` について**。`collect_consumes_go` の `RcExpr::Ret` の腕は、関数本体の終端の `Ret` だけ
でなく、`Match` のアーム本体の `Ret` についても `push_boxed_leaves` を呼ぶ
(`CODE src/rc_ir/ownership.rs: collect_consumes_go` -- `RcRhs::Match` の腕は各アーム本体へ降りる)。
D9 はアーム本体の `Ret` を消費とせず移動とするので、これは過剰報告である (D9 の `collect_consumes` に
ついての注)。`<1>2` の議論は「積まれた元に P8 (b) を当てる」だけなので、過剰報告された元についても
同じ結論が出る。第 9.4 節の `L11` がこれを使う。

### 3.6 `App` の引数の行について

D9 の `App` の引数の行は「呼び出し先がその位置の **unit** を所有する引数の leaf」であり、
`rhs_consumes` の `is_owning_position` は **leaf** ごとに `owned_leaves` を引く。1 つの unit の下に
2 つ以上の leaf を持つ型 -- unbox union と punched array (D5) -- では、この 2 つは食い違いうる。
`owned_leaves` が unit `u` の下の leaf を 1 つだけ持つとき、DEF 推論の割り当て を unit 粒度へ持ち上げた
割り当て (「`u` の下のある leaf が所有される」) の下で D9 はその unit の下の**すべての** leaf を消費と
数えるのに対し、`collect_consumes` は 1 つだけを報告する。よって P8 (c) をこの行について、unit 粒度の
割り当てで述べることはできない。

**この差は下流に届かない。** P14 は `App` の引数の位置の消費を P8 で扱わず、`call_rc` が置く節点で扱う
(第 10.3 節)。`p13-disposals-and-pending.md` の `L16` の言明も同じ分け方をする -- その (A) と (B)
は `owns_unit` の真偽で分かれ、(B) すなわち `owns_unit` が偽の場合は `App` の引数の位置に限られ、そこでは
`call_rc` が `Retain` を置く。`README.md` の P8 はこの狭めをすでに文面に持つ。

### 3.7 系 -- 消費される leaf の候補は所有されている

**言明**。`(v, λ)` を、所有を読まない消費 (DEF 所有を読まない消費) によって消費される leaf、または
`Match` のアーム本体の `Ret` が名指す leaf とする。`f` の借用版 `f_borrow` の `RewriteCtx` を `ctx` とすると、
`cand(v, λ)` の各元 `(r, p)` について `ctx.owns_object(ρ_f(r), p)` は真である。

<1>1. `cand(v, λ)` の元 `(r, p)` で `vars_f.param_tys` が `r` を鍵に持たないものについて、
      `ctx.owns_object(ρ_f(r), p)` は真である。
  `f_borrow` の `RewriteCtx` は `RewriteCtx::new(&clone, true, ..)` が作り、その `vars` は
  `VarTable::of(clone)` である。`L15` (`p15-ownership-uniformity.md`) より `VarTable::of(clone)` の
  `param_tys` の鍵は `vars_f.param_tys` の鍵の `ρ_f` による像ちょうどであり、`ρ_f` は単射なので `ρ_f(r)` は
  その鍵でない。L4 より `owns_object` はこのとき真を返す。
  BY L4, p15-ownership-uniformity.md の L15, CODE src/rc_ir/borrow.rs: RewriteCtx::new, borrow_ify

<1>2. `cand(v, λ)` の元 `(r, p)` で `vars_f.param_tys` が `r` を鍵に持つもの (その型を `τ`) について、
      `(r, p) ∈ OL` である。
  所有を読まない消費については P8 (c) が、アーム本体の `Ret` については第 3.5 節の注と P8 (b) が与える。
  BY P8 (b), P8 (c), 3.5 の注

<1>3. `<1>2` の `p` は `leaves(τ)` の元である。
  `origin(v, λ)` の候補が持つ path は、`origin_inner` の再帰が作るものである。`λ` は `ty(v)` の leaf で
  あり、再帰の各段は leaf を leaf へ写す -- `Binding::Move`、catch-all の `Binding::Payload`、
  `Binding::Join` は path を変えず (A12 より相手の型は同じ)、unbox 容器の `Binding::Field` と unbox の
  変位アームの `Binding::Payload` は添字を 1 つ前に足し、D4 の第 5 の規則よりその結果は容器・scrutinee の
  型の leaf であり、`Binding::Llvm` の単一 `Arg(j, σ)` の腕では A3 より `σ` は `ty(args[j])` の leaf で
  ある。残る腕は `here()` を返して再帰しない。
  BY A3, A12, D4, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>4. `covered_leaves(τ, p) = {p}` である。
  `<1>3` より `p ∈ leaves(τ)` である。`p13-disposals-and-pending.md` の `L7` より `leaves(τ)` の相異なる
  2 元は一方が他方の接頭辞にならないので、`λ' ⊑ p` または `p ⊑ λ'` を満たす `λ' ∈ leaves(τ)` は `p` だけ
  である。
  BY <1>3, p13-disposals-and-pending.md の L7, CODE src/rc_ir/borrow.rs: covered_leaves

<1>5. `owns_object_yet(vars_f, type_env, r, p, OL)` は真である。
  `<1>2` より `covered_leaves(τ, p) = {p} ⊆ { λ' : (r, λ') ∈ OL }` であり、`<1>4` よりそれは空でない。
  BY <1>2, <1>4, p15-ownership-uniformity.md の L11

<1>6. QED
  `L16` (`p15-ownership-uniformity.md`) より
  `ctx.owns_object(ρ_f(r), p) = owns_object_yet(vars_f, type_env, r, p, OL)` である。`<1>1` が
  パラメータでない候補を、`<1>5` がパラメータである候補を与える。
  BY <1>1, <1>5, p15-ownership-uniformity.md の L16

## 4. P9 -- 複製は名前替えである

### 4.1 前半 -- 本体は束縛変数を一斉に付け替えたものである

**言明**。`clone_func(func, new_ref, rename_counter)` が返す `RcFunc` の `body` は、`func.body` の各節点を
同じ種類・同じ並びの節点に写し、`FieldPath`・`RcState`・`source`・`MatchArm` の `tag` と `payload_state`・
`RcRhs::Closure` の `FuncRef` を変えず、変数の出現だけを 1 つの写像 `rename_f` で置き換えたものである。
さらに `rename_f` の定義域は `func` のパラメータ・capture の名前と `func.body` が束縛する名前の全体で
あり、`rename_f` は定義域の各名前に像を 1 つだけ持ち、相異なる束縛子には相異なる像を与える。

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

<1>7. 相異なる束縛子には相異なる像が与えられる。
  `assign_fresh_name` は `counter` を 1 増やしてから `name#b<counter>` を作り、`<1>4` より 1 つの束縛子に
  ついて 1 度だけ呼ばれる。`"b" ++ dec(c)` は `#` を含まないので、追加された `#` が像の最後の `#` であり、
  像から `c` が読み取れる。相異なる呼び出しは相異なる `c` を使う。
  BY <1>4, CODE src/rc_ir/rename.rs: assign_fresh_name

<1>8. QED
  `<1>5` と `<1>6` より、写された本体は元の本体と同じ形の木であり、変わるのは `renaming` の定義域にある
  名前の出現だけである。`<1>3` よりその定義域は束縛名の全体であり、`<1>4` より写像、`<1>7` より単射で
  ある。A6 と A11 より、定義域にある名前の出現はすべて、その名前を束縛する `func` の束縛子に解決する
  出現である。よって置き換えは一斉の名前替えである。
  BY A6, A11, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7

### 4.2 後半 -- 導入する名前は入力の束縛名と異なる

**言明**。A13 の下で、`borrow_ify` の実行中に `clone_func` が導入する名前は、どの入力の束縛名とも異なる。
導入される名前は `M ++ "#b" ++ dec(c)` の形であり、`M` は元の束縛名の `name` フィールド、`c` は 1 以上の
整数で、1 回の `borrow_ify` の実行の中で 2 度使われることはない。

<1>1. `clone_func` が導入する名前は、`assign_fresh_name(&name, "b", &mut renaming, counter)` が作る
      `FullName` であり、その `namespace` は `name.namespace` のまま、`name` フィールドは
      `format!("{}#{}{}", name.name, "b", counter)` である。`counter` は使用の直前に 1 増やされるので
      1 以上である。`clone_func` は `fresh_rename_function(.., "b", rename_counter)` を呼び、
      `fresh_rename_function` が名前を作るのは `assign_fresh_name` の呼び出しだけである。
  BY CODE src/rc_ir/borrow.rs: clone_func, CODE src/rc_ir/rename.rs: fresh_rename_function,
     assign_fresh_name

<1>2. `<1>1` の `name` フィールドを `#` で区切った最後の断片は、`b` の後に 10 進数字が 1 個以上続く形で
      ある。
  `counter` の 10 進表記は 10 進数字だけからなり `#` を含まないので、追加された `#` が最後の `#` である。
  BY <1>1

<1>3. 入力の束縛名の `name` フィールドを `#` で区切った最後の断片は、`<1>2` の形ではない。
  BY A13

<1>4. `c` は 1 回の `borrow_ify` の実行の中で 2 度使われない。
  `rename_counter` は `borrow_ify` の中で 1 つだけ作られ、`clones` を作るループを通じて `clone_func` に
  渡され、`assign_fresh_name` の呼び出しごとに 1 増える。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, clone_func, CODE src/rc_ir/rename.rs: assign_fresh_name

<1>5. QED
  `FullName` の相等は `namespace` と `name` で決まるので、`name` フィールドが異なれば名前は異なる。
  形についての後半は `<1>1` と `<1>4` による。
  BY <1>1, <1>2, <1>3, <1>4, CODE src/ast/name.rs: FullName

**この言明を検査するコード**。`develop_mode` のとき、`borrow_ify` は `check_clone_names_are_fresh(prog,
clones.iter().map(|(_, _, rename)| rename))` を呼ぶ。これは入力プログラムのパラメータ・capture と
`for_each_var` が訪れる全変数の名前を集め、各 `rename` の像がそのどれでもないことを `assert!` する
(`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`, `borrow_ify`)。A13 の「検査: 無し」は、この
検査を書いた形へ改めるべきである (第 11 節)。

### 4.3 系 -- 出力の束縛名は互いに相異なる

**言明**。A13 の下で、`borrow_ify` の出力の束縛名 (DEF 出力の束縛名) は互いに相異なる。

<1>1. `clone_func` が導入する 2 つの名前は、相異なる束縛子のものならば異なる。
  4.2 の言明より、導入される名前は `M ++ "#b" ++ dec(c)` の形で `c` は 2 度使われない。
  `M1 ++ "#b" ++ dec(c1) = M2 ++ "#b" ++ dec(c2)` ならば、`"b" ++ dec(c)` が `#` を含まないので両辺の
  最後の `#` は追加された `#` であり、`c1 = c2` である。4.1 の言明より 1 つの束縛子には 1 つの `c` しか
  使われないので、`c1 = c2` は同じ束縛子であることを意味する。
  BY 4.1 の言明, 4.2 の言明

<1>2. 出力の各 `f_own` の束縛名は、対応する入力の関数の束縛名と同じである。
  `f_own` は `func.clone()` の `body` を `ctx.rewrite(&f_own.body)` に差し替えたものであり、`params` と
  `capture` は `func` のままである。L5 より `rewrite` は本体の束縛名を変えない。
  BY L5, CODE src/rc_ir/borrow.rs: borrow_ify

<1>3. 出力の各 `f_borrow` の束縛名は、`clone_func` が導入した名前の全体である。
  4.1 の言明より `clone_func` の出力は `func` の束縛名を `rename_f` で一斉に付け替えたものであり、その
  定義域は `func` の束縛名の全体なので、出力の束縛名は `rename_f` の像である。その後の
  `ctx.rewrite(&clone.body)` は L5 より束縛名を変えない。
  BY L5, 4.1 の言明, CODE src/rc_ir/borrow.rs: borrow_ify

<1>4. 出力のグローバル初期化子の束縛名は、入力のグローバル初期化子の束縛名と同じである。
  `borrow_ify` はグローバル初期化子の `init` を `ctx.rewrite(&g.init)` に差し替えるだけであり、L5 より
  `rewrite` は束縛名を変えない。
  BY L5, CODE src/rc_ir/borrow.rs: borrow_ify

<1>5. QED
  出力の束縛名は、`<1>2` のもの、`<1>3` のもの、`<1>4` のものの 3 つに分かれる。`<1>2` と `<1>4` の
  名前はどれも入力の束縛名であり、A6 より互いに相異なる。`<1>3` の名前どうしは `<1>1` より相異なる。
  `<1>2`・`<1>4` の側と `<1>3` の側は 4.2 の言明より相異なる。
  BY A6, 4.2 の言明, <1>1, <1>2, <1>3, <1>4

## 5. P10 -- 借用版が落とす RC 節点

**言明**。`is_borrow_version` が真の `RewriteCtx` を `ctx` とする。`ctx.rewrite` は、`Retain(v, π, s, k)` を
次の節点に写す。`under(ty(v), π)` の元のうち `ctx.owns_unit(v, ・)` が真であるものを、`units_under` が
返す並びの順に `u_1, ..., u_r` とする。写る先は

```
Retain(v, u_1, s, Retain(v, u_2, s, ... Retain(v, u_r, s, ctx.rewrite(k)) ... ))
```

であり、`r = 0` のときは `ctx.rewrite(k)` そのものである。`Release(v, π, s, k)` については同じ並びで
`Release` の列になる。`ctx.owns_unit(v, ・)` が偽である unit についての節点は、この写像の像に現れない。

<1>1. `ctx.rewrite(node)` は `ctx.rewrite_inner(node)` の値である。
  BY A15, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite, CODE src/misc.rs: grow_stack

<1>2. `rewrite_inner` の `RcExpr::Retain(v, path, state, k)` の腕は
      `self.rewrite_rc(v, path, *state, false, k, &node.source)` を、`RcExpr::Release` の腕は
      同じ引数で `is_release` を `true` にしたものを返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

<1>3. `rewrite_rc` はまず `let k = self.rewrite(k);` を行う。`self.is_borrow_version` が真なので、
      `rc_node(is_release, v.clone(), path.clone(), state, k, source)` を返す腕は通らない。
  BY <1>2, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>4. `kept` は `under(ty(v), path)` を `self.owns_unit(v, unit)` で絞ったものであり、
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

## 6. P11 -- 呼び出し側の補正

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
      `self.owns_unit(arg, &unit)` である。A14 より `arg_idx` は `params` の範囲内である。
  BY A14, <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>6. `!callee_owns && arg_owned` のとき `after.push((arg.clone(), unit))`、
      `callee_owns && !arg_owned` のとき `before.push((arg.clone(), unit))`、それ以外のとき何も積まない。
      `if`/`else if` の 2 分岐であり、どちらの条件も満たさない対は素通りする。
  BY <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>7. QED
  `<1>4` の 2 重ループの順序が `before` と `after` の並びを決め、`<1>6` がその中身を決める。`<1>1`-`<1>3` が
  節点の位置と種類を決める。`call_rc` に渡されるのは振り分け後の `callee` なので、`params` は
  `callee'` について引かれる。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

**`params[arg_idx]` が範囲内であること**。A14 (過適用が無い) がこれを与える。`callee_params` は
`param_names_and_types(func)` すなわち `func.params` に `func.capture` を鎖にしたものなので、
`args.len() <= params.len()` は `args.len() <= func.params.len()` から従う
(`CODE src/rc_ir/borrow.rs: param_names_and_types`)。

**「ちょうど埋める」について**。P11 の第 1 文の「呼び出し元が借用し呼び出し先が所有する unit には前に
`Retain` を」を第 2 文の言い換えとして読むなら、上の言明がそれである。実行時の義務集合 (D10) についての
主張として読むなら、すなわち「補正の後、呼び出しの前後で `Obl` の収支が合う」として読むなら、それは
第 10.3 節と第 10.4 節が示す。

## 7. P12 -- 振り分けの安全性

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

<1>3. `any_owned_unit(arg)` は `units(ty(arg))` のいずれかの `unit` について `self.owns_unit(arg, unit)` が
      真であることである。
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

**P12 (d) (門)**。`observing` に入る関数は借用版を持たず、その名前への直接呼び出しは `route` を素通りする。

<1>1. 1 番目のループは `observing.contains(&func.name)` のとき `continue` するので、`borrow_versions` は
      `observing` の元を鍵に持たない。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `route` は `borrow_versions.get(&orig)` が `None` のとき `callee.clone()` を返す。
  BY P12 (a), <1>1

<1>3. `observing` は、次のグラフの上の最小不動点である。頂点は `prog.funcs` の各関数。種は、本体に
      `llvm_gen.observes_uniqueness()` が真の `RcRhs::Llvm` を持つ関数。辺は 2 種で、`f` の本体に
      `RcRhs::App(callee, _)` があり `FuncRef { name: callee.name }` が `prog.funcs` の鍵であるときの
      `f -> その関数`、および `f` の本体に `App` があってその名前が `prog.funcs` の鍵でないとき
      (`f ∈ calls_indirectly`) の、`f` から**すべての closure の対象**への辺である。closure の対象は
      `prog.funcs` の全本体と `prog.globals` の全 `init` を歩いて `RcRhs::Closure(target, _)` から集める。
      グローバル初期化子は `owner` が `None` なので、種にも `callees` の鍵にもならず、closure の対象だけを
      寄与する。
  `scan` は `for_each_node` で本体の全節点を歩き、`RcRhs::Llvm` の腕で `owner` を `observing` に、
  `RcRhs::App` の腕で `prog.funcs.contains_key(&target)` の真偽により `cs` か `calls_indirectly` に、
  `RcRhs::Closure` の腕で `closure_targets` に積む。走査の後、`calls_indirectly` の各元の `callees` に
  `closure_targets` の全元が足される。最後のループが `cs` の元が `observing` に入る頂点を `observing` に
  入れることを、変化が無くなるまで繰り返す。
  BY CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness, CODE src/rc_ir/ast.rs: for_each_node

<1>4. QED
  BY <1>1, <1>2, <1>3

**「同じ関数の版である」について**。`borrow_versions` の鍵 `orig` に対する値は
`borrow_funcref(&func.name)` であり、`func.name` の `name` フィールドに `#borrow` を継ぎ足したものである
(`CODE src/rc_ir/borrow.rs: borrow_funcref`)。よって振り分け先は、`orig` の借用版として作られた版である。
呼び出し先が入力の関数を名指していないとき (局所変数を経由する間接呼び出しのとき) は、`borrow_versions` が
その名前を鍵に持たないので `route` は `callee` をそのまま返す。`README.md` の P12 の「呼び出し先が入力の
関数を名指すとき、返る名前は出力の `funcs` の鍵である」はそのとおりであり、名指さないときの行も
`<1>2` が与える。

## 8. P13 -- 注釈の一致

**言明**。出力の各版 `V` について、`V.borrowed_units` は `param_capture_units(V, type_env)` の元のうち
`owned_units` に入らないものの集合である。

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

**系 2 (`f_borrow` が借用する unit)**。A13 の下で、`borrow_versions` に載る入力の関数 `f` の各パラメータ `p` と
各 `u ∈ units(ty(p))` について、`f_borrow.borrowed_units` が `(rename_f[p.name], u)` を含むことと、
`trunc(ty(p), λ) = u` かつ `(p.name, λ) ∈ OL` である `λ ∈ leaves(ty(p))` が無いこととは同値である。

<1>1. `f_borrow` の `params` は `rename_var(p, rename_f)` の列であり、`ty` は変わらない。`f_borrow` の
      `capture` は `None` である。
  `rename_var` は `RcVar` を複製して `name` だけを写す。`borrow_versions` に載るのは
  `func.capture.is_none()` の関数だけであり、`rename_var` は `None` を `None` に写す。
  BY CODE src/rc_ir/rename.rs: rename_var, fresh_rename_function, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. よって `param_capture_units(f_borrow, type_env)` は
      `{(rename_f[p.name], u) : p ∈ f.params, u ∈ units(ty(p))}` である。
  BY <1>1, CODE src/rc_ir/borrow.rs: param_capture_units

<1>3. `trunc(ty(p), λ) = u` かつ `(p.name, λ) ∈ OL` である `λ` があるならば、
      `(rename_f[p.name], u) ∈ owned_units` である。
  L2 の (b) がその `λ` について `(rename_f[p.name], trunc(ty(p), λ))` を入れる。
  BY L2

<1>4. `(rename_f[p.name], u) ∈ owned_units` ならば、そのような `λ` がある。
  <2>1. L2 より `owned_units` の元は (a) 入力の関数のパラメータ・capture の名前を第 1 成分に持つものか、
        (b) ある `borrow_versions` に載る関数 `g` のあるパラメータ `q` について
        `(rename_g[q.name], trunc(ty(q), λ))` (`λ` は `(q.name, λ) ∈ OL` である leaf) のどちらかである。
    BY L2
  <2>2. `rename_f[p.name]` は入力の束縛名ではないので、(a) の形ではありえない。
    BY 4.2 の言明
  <2>3. (b) の形であるとき、`rename_g[q.name] = rename_f[p.name]` である。4.1 の言明より相異なる束縛子には
        相異なる像が与えられ、4.2 の言明より `borrow_ify` の 1 回の実行の中で `c` は 2 度使われないので、
        相異なる関数の相異なるパラメータには相異なる名前が付く。よって `g = f` かつ `q = p` である。
    BY 4.1 の言明, 4.2 の言明
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>5. QED
  P13 と `<1>2` より `f_borrow.borrowed_units` は `<1>2` の集合から `owned_units` の元を除いたものであり、
  `<1>3` と `<1>4` がその除かれる元を特徴づける。
  BY P13, <1>2, <1>3, <1>4

**系 2 は P9 の後半 (4.2) を使う。** `owned_units` は変数名で引く集合なので、複製が導入した名前が入力の
束縛名と衝突すると、(a) の元が (b) の元として読まれ、借用版が所有していない unit を所有していることに
なる。

**系 3 (パラメータ unit の所有と `owns_object` が合う)**。出力の版 `V` の `RewriteCtx` を `ctx` とし、
`p` を `V` のパラメータか capture、`u ∈ units(ty(p))` とする。このとき `ctx.owns_object(p.name, u)` は、
D14 の意味で `V` が `(p, u)` を所有することと同値である。

<1>1. L1 より `under(ty(p), u) = [u]` であり、L6 (`p15-ownership-uniformity.md`) より `trunc(ty(p), u) = u`
      である。
  BY L1, p15-ownership-uniformity.md の L6

<1>2. `ctx.vars.param_tys` は `p.name` を鍵に持つ。
  `V` が関数の版のとき、その `RewriteCtx` は `RewriteCtx::new` が作り、`vars` は `VarTable::of(func)` で
  ある。`VarTable::of` は各パラメータ・capture を `param_tys` に入れる。`V` がグローバル初期化子のときは
  `borrow_ify` が `RewriteCtx` を構造体リテラルで作り、`vars` は `VarTable::body_only(&g.init)` である
  が、D1 よりグローバル初期化子はパラメータも capture も持たないので、この言明は空虚に成り立つ。
  BY D1, CODE src/rc_ir/borrow.rs: RewriteCtx::new, borrow_ify,
     CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only

<1>3. L4 と `<1>1` と `<1>2` より、`ctx.owns_object(p.name, u)` は `(p.name, u) ∈ owned_units` と同値で
      ある。
  BY L4, <1>1, <1>2

<1>4. QED
  D14 より `V` が `(p, u)` を所有するとは `(p.name, u) ∉ V.borrowed_units` であり、P13 よりそれは
  `(p.name, u) ∈ owned_units` と同値である。
  BY D14, P13, <1>3

## 9. P14 の準備

P14 は、出力の 3 種の版 -- 全所有版 `f_own`、借用版 `f_borrow`、グローバル初期化子 -- のそれぞれの本体に
ついて D11 の 3 つの節を示すことである。この節がその 3 種に共通の道具を作る。

### 9.1 固定するものと、P14 が読む 2 つの仮定

以下、出力の版 `V` を 1 つ固定する。`B_V` を `RewriteCtx::rewrite` が受け取る本体、`B'_V` をその値、
`ctx` を `V` の `RewriteCtx` とする。すなわち `f_own` とグローバル初期化子については `B_V` は入力の本体で
あり、`f_borrow` については `B_V` は `clone_func` が返した本体である
(`CODE src/rc_ir/borrow.rs: borrow_ify`)。

`B_V` の実行路 `ρ` (D3) と、`ρ` を辿る活性化 (D21) を 1 つ固定する。

**活性化が渡すもの**。D21 より、活性化が渡すのは辿る実行路だけではなく、各位置での値の割り当ても含む組で
ある。よって `B'_V` の活性化はそのまま `B_V` の対応する実行路の活性化である -- `rewrite` が足し引きする
節点は `Retain` と `Release` だけであり、これらは変数を束縛せず (L5)、D9 の消費・移動の表にも D10 の生成の
表にも現れないので、割り当てはそのまま運ばれる。これは D21 が「出力の活性化から入力の活性化を作る段
(P14、P21)」について書いていることである。
  BY D21, L5

**P14 が読む 2 つの仮定**。この文書は局所の仮説を置かない。第 10 節が読むのは `README.md` の A19 (ii-a) と
A20 である。第 9.7 節の記法へ渡す形で書き直すと次のようになる。

- **A19 (ii-a)**。第 9.7 節の `n_in` について、`ρ` の各節点の入口 `τ` と終端の `Ret` の消費の後の位置で、
  (a) どの由来 `T` についても `n_in(τ, T) ≥ 0` であり、(b) `τ` の節点が D7 の読む構文であるとき、その
  構文が読みうる各スロット -- D7 の表が名指す値の inhabited な各 boxed leaf `(x, λ)` -- で `obj(x, λ)` が
  計数下 (D26) であるものについて `n_in(τ, T_ρ(x, λ)) ≥ 1` であり、(b') `τ` の節点が `Retain(v, π)` か
  `Release(v, π)` であるとき、`π` の下の inhabited な各 leaf `λ` で `obj(v, λ)` が計数下であるものに
  ついて `n_in(τ, T_ρ(v, λ)) ≥ 1` である。

  A19 (ii-a) の文面は別名類を主語にする。第 9.3 節の DEF 由来 は
  `p13-disposals-and-pending.md` の第 7.5.2 節の `ρ`-終端と同じ規則であり、A19 が引く別名類は
  「`T_ρ` が等しいスロットの同値類」なので、別名類と `ρ`-由来は 1 対 1 に対応する。A19 が数える
  「その類が持つ参照の個数」は `p13` の `held_ρ` であり、その 6 行 -- 生成で 1 から始まる、所有する
  パラメータ・capture の leaf で 1 から始まる、借用するそれで 1 から始まる、`Retain` で +1、`Release` で
  -1、消費で -1 -- は、DEF 由来ごとの義務 の各行と一致する。ただし第 3 行 (借用する leaf) は、A1 より
  `borrow_ify` の入力のすべての関数の `borrowed_units` が空なので当たらない。よって `borrow_ify` の入力の
  本体について `held_ρ(τ, C(T)) = n_in(τ, T)` であり、A19 (ii-a) の「非負」と「読む構文と
  `Retain`/`Release` がその類を名指す時点では 1 以上」は上の (a)・(b)・(b') である。
  BY A1, A19, DEF 由来, DEF 由来ごとの義務

- **A20**。`V` が D14 の意味で借用するパラメータ・capture の unit `u` について、`u` の下の inhabited な
  leaf が指す計数下のオブジェクトは、この活性化が生きている間 解放されていない。呼び出し元がその参照を
  呼び出しが返るまで処分しないからである。
  BY A20, D14

### 9.2 全所有版とグローバル初期化子では `owns_object` は常に真である

#### L8

**言明**。`V` が `f_own` かグローバル初期化子であるとき、`ctx.owns_object(r, p)` は、値を返すどの `(r, p)`
についても真である。

<1>1. グローバル初期化子の `ctx` では `vars` が `VarTable::body_only` で作られ、その `param_tys` は空で
      ある。よって L4 の第 1 の場合に入り、真を返す。
  BY L4, CODE src/rc_ir/borrow.rs: borrow_ify のグローバルを写す繰り返し,
     CODE src/rc_ir/ownership.rs: VarTable::body_only

<1>2. `f_own` の `ctx` の `vars` は `VarTable::of(f_own)` であり、その `param_tys` の鍵は入力の関数
      `func` のパラメータ・capture の名前ちょうどで、その型は `func` のものである。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, borrow_ify, CODE src/rc_ir/ownership.rs: VarTable::of

<1>3. `param_tys` が `r` を鍵に持つとき (その型を `τ`)、`under(τ, p)` の各 `unit` について
      `trunc(τ, unit) ∈ units(τ)` である。
  BY p15-ownership-uniformity.md の L9

<1>4. QED
  `borrow_ify` は入力の各関数について `owned_units.extend(param_capture_units(func, type_env))` を行い、
  `param_capture_units` は各パラメータ・capture `p` と各 `unit ∈ units(ty(p))` について `(p.name, unit)` を
  並べる。`<1>2` より `r` は `func` のパラメータか capture で `τ = ty(r)` なので、`<1>3` の各
  `(r, trunc(τ, unit))` はこの集合に入る。L4 よりこれが `owns_object(r, p)` の真であることである。
  BY L4, <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units

### 9.3 由来

**DEF 由来の 1 歩** -- `ρ` の上のスロット `(x, λ)` (D6) について、`ctx.vars.bindings.get(x)` に応じて次の
対を **`(x, λ)` の 1 歩**と呼ぶ (`CODE src/rc_ir/ownership.rs: origin_inner`, `Binding`)。

| `x` の `Binding` | 1 歩の先 |
|---|---|
| `Move(y)` | `(y, λ)` |
| `Payload(s, None)` | `(s, λ)` |
| `Payload(s, Some(t))` で `s` が unbox | `(s, [t] ++ λ)` |
| `Field(c, idx)` で `c` が unbox | `(c, [idx] ++ λ)` |
| `Llvm(gen, args, rty)` で `decl.leaf_origins_at(λ)` が単一の `Arg(j, σ)` | `(args[j], σ)` |
| `Join(arm_results)` | `(a_ρ, λ)`。`a_ρ` は `ρ` が選んだアームの `returned_var` |
| 上のどれでもない | 無し。このとき `(x, λ)` を **`ρ`-由来**と呼ぶ |

**DEF 由来** -- `(x, λ)` から 1 歩を繰り返して着く `ρ`-由来を `T_ρ(x, λ)` と書き、`(x, λ)` の**由来**と
呼ぶ。

#### L9 (1 歩は同じ参照を運ぶ)

**言明**。`(x, λ)` を `ρ` の上のスロットとし、その 1 歩の先を `(x', λ')` とする。このとき `(x', λ')` も
`ρ` の上のスロットであり、`obj(x', λ') = obj(x, λ)` であって、両者が持つ参照は同一である。また
`T_ρ(x, λ)` は有限歩で定まる。

<1>1. 表の 6 行は D20 の別名の辺の 6 つと 1 対 1 に対応する。
  D20 は移動の表 (D9) の 6 行を別名の辺と呼ぶ。`Move(y)` は `Let(x, Var(y), k)` の行、`Join` は
  アーム本体の `Ret` の行、`Field` の unbox の行は unbox 容器の `Destructure` の名前付きフィールドの行、
  `Payload(s, Some(t))` の unbox の行は unbox union の変位アームの payload 束縛の行、
  `Payload(s, None)` の行は catch-all アームの payload 束縛の行、`Llvm` の単一 `Arg` の行は素通し leaf の
  行である。`collect_bindings` は `Let(x, Var(y), k)` に `Move(y)`、`Let(x, Match(s, arms), k)` に
  `Join(arm_results)` と各 `arm.payload` への `Payload(s, arm.tag)`、`Destructure` の各フィールド変数に
  `Field(container, idx)`、`Let(x, Llvm(gen, args), k)` に `Llvm(gen, args, ty(x))` を入れる。
  BY D9, D20, CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner

<1>2. `(x', λ')` は `ρ` の上のスロットである。
  <2>1. `x'` は `ρ` の上でこの位置までに値を得ている。
    A11 より変数の使用はその位置でスコープに入っている束縛に解決するので、`<1>1` の各行で名指される
    `y`・`s`・`c`・`args[j]`・`a_ρ` は、`x` を束縛する節点より前に値を得ている。
    BY A11, D6, <1>1
  <2>2. `λ'` は `ty(x')` の boxed leaf である。
    `Move` と `Payload(s, None)` では A12 より `ty(x') = ty(x)`。`Field` の unbox の行と
    `Payload(s, Some(t))` の unbox の行では、A12 より `ty(x)` は `ty(x')` の第 `idx` (resp. 第 `t` 変位の)
    フィールドの型であり、D4 の第 5 の規則より `[idx] ++ λ` (resp. `[t] ++ λ`) は `ty(x')` の leaf で
    ある。`Llvm` の行では A3 より `σ` は `ty(args[j])` の leaf である。`Join` では A12 より
    `ty(a_ρ) = ty(x)` である。
    BY A3, A12, D4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `λ'` はその時点で inhabited (D16) である。
    `Move`・`Payload(s, None)`・`Join` では値が同じなので D16 の条件も同じである。`Field` の unbox の行で
    足す添字は unbox 構造体のフィールド添字であり (A12 より `Destructure` の容器は構造体)、D16 が数える
    unbox union の節を増やさない。`Payload(s, Some(t))` の unbox の行が足す添字は unbox union の節を
    1 つ増やすが、D21 よりこの活性化がこのアームを選んだのは scrutinee の実行時のタグが `t` に等しい
    ときであり、A16 よりそのようなアームが選ばれるので、その節について D16 の条件は成り立つ。`Llvm` の
    行では A3 の第 2 行が「結果のその leaf が inhabited であることと第 `j` オペランドの leaf `σ` が
    inhabited であることは同値」と述べる。
    BY A3, A12, A16, D16, D21
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>3. `obj(x', λ') = obj(x, λ)` であり、両者が持つ参照は同一である。
  `<1>1` より 1 歩は D9 の移動の 1 行であり、移動とは「参照の持ち手が活性化の中で変わるだけの構文」で
  ある。A5 より値が保持する参照は inhabited な各 leaf にちょうど 1 つずつあるので、移動の前後の 2 つの
  スロットが持つ参照は同一であり、したがって指すオブジェクトも同じである。
  BY A5, D9, <1>1

<1>4. `T_ρ(x, λ)` は有限歩で定まる。
  1 歩の先は、`origin_inner(vars, type_env, x, λ)` が `origin` を呼ぶ相手の 1 つである
  (`Binding::Join` の腕は各アーム結果について呼び、表はそのうち `ρ` が選んだアームのものを取る)。
  P2 より `origin(x, λ)` は停止するので、その再帰の木は有限であり、1 歩の列はその木の根から下る 1 本の
  枝である。枝は有限で、その末端では `origin_inner` が `origin` を呼ばない。
  BY P2, CODE src/rc_ir/ownership.rs: origin_inner

<1>5. QED
  BY <1>2, <1>3, <1>4

#### L10 (`origin` の候補は由来を含む)

**言明**。`ρ` の上のスロット `(x, λ)` について次の 3 つが成り立つ。

- **(a)** `T_ρ(x, λ) ∈ cand(x, λ)` である。
- **(b)** `origin(x, λ)` が `Origin::Exactly` であるとき `cand(x, λ) = {T_ρ(x, λ)}` である。
- **(c)** `origin(x, λ)` が `Origin::Join` であるとき、`(x, λ)` から 1 歩を繰り返して着くスロットのうち
  `Binding::Join` の 1 歩を持つ最初のものを `(z, μ)` として、`origin(x, λ) = origin(z, μ)` であり、
  `cand(x, λ) = ⋃_{a ∈ arm_results(z)} act(a, μ)` である。ここで `arm_results(z)` は `z` の
  `Binding::Join` が持つアーム結果の列である。

<1>1. `ρ`-由来 `(u, σ)` について `origin(u, σ) = Origin::Exactly((u, σ))` である。
  DEF 由来の 1 歩 の「上のどれでもない」に当たるのは、`bindings.get(u)` が `None`、`Param`、`Producer`、
  boxed 容器の `Field`、`Some(tag)` かつ boxed の `Payload`、および `Llvm` で
  `decl.leaf_origins_at(σ).and_then(as_arg_projection)` が `None` の場合である。前の 5 つの腕は `here()`
  すなわち `Origin::Exactly((u, σ))` を返す。最後の場合、`σ` は `ty(u)` の leaf なので
  `decl.leaf_origins_under(σ)` が挙げるのは `σ` の記録 1 つだけであり (leaf は反鎖をなす)、その記録は
  `as_arg_projection` が `None` を返したことから、空集合か `{Fresh}` か `{Unknown}` か元数 2 以上の
  いずれかである。A3 より元数 2 以上の宣言はこのプログラムに無い。空集合のとき `origin_from_leaves_under` は
  `reached` が空なので `None` を返し、`unwrap_or_else(here)` が `Exactly((u, σ))` を与える。`{Fresh}` か
  `{Unknown}` のとき `produced_here` が真で `operand_units` は空なので `reached = [Exactly(here)]` であり、
  全元が等しいので `first.clone()` すなわち `Exactly((u, σ))` を返す。
  `(u, σ)` が `ρ` の上のスロットであることと `σ ∈ leaves(ty(u))` であることは、L9 が 1 歩ごとに保つ。
  BY A3, L9, p13-disposals-and-pending.md の L7,
     CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under, as_arg_projection,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, Provenance::leaf_origins_under

<1>2. 1 歩が `Move`・`Payload`・`Field` の unbox・`Llvm` の単一 `Arg` のいずれかであるとき、
      `origin(x, λ) = origin(x', λ')` である。
  この 4 つの腕はいずれも `origin(...)` の値をそのまま返す。
  BY CODE src/rc_ir/ownership.rs: origin_inner

<1>3. 1 歩が `Join(arm_results)` であるとき、`S := ⋃_{a ∈ arm_results} act(a, λ)` として、
      `origin(x, λ) = Origin::of_candidates(S, (x, λ))` であり、`cand(x, λ)` は `|S| = 1` のとき `S`、
      `|S| ≥ 2` のとき `S` である。
  `Binding::Join` の腕は各アーム結果の `acted_on()` を `candidates` に集めて `of_candidates` に渡す。
  `of_candidates` は `|S| = 1` のとき `Exactly` を、それ以外のとき `Join { candidates: S }` を返し、
  `candidates()` はどちらでも `S` を与える。`S` が空のときは `assert!` で中断するが、`<1>1` と
  `Origin::acted_on` より各 `act(a, λ)` は空でない。
  BY CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates, Origin::candidates,
     Origin::acted_on

<1>4. (a) と (b) が成り立つ。
  1 歩の列の長さについての帰納で示す。長さ 0 のとき `<1>1` より `cand(x, λ) = {(x, λ)} = {T_ρ(x, λ)}` で
  あり `Exactly` である。長さが 1 以上のとき、`<1>2` の 4 つの場合は `origin` そのものが等しいので
  帰納法の仮定がそのまま渡る。`<1>3` の場合、帰納法の仮定より `T_ρ(x, λ) = T_ρ(a_ρ, λ) ∈ cand(a_ρ, λ)` で
  あり、`cand(a_ρ, λ) ⊆ act(a_ρ, λ) ⊆ S = cand(x, λ)` である。`Exactly` になるのは `|S| = 1` のときで、
  そのとき `cand(x, λ) = S` の唯一の元は `T_ρ(x, λ)` である。
  BY <1>1, <1>2, <1>3, CODE src/rc_ir/ownership.rs: Origin::acted_on

<1>5. QED
  (c) を示す。`origin(x, λ)` が `Origin::Join` であるとする。`<1>1` より `ρ`-由来では `Exactly` であり、
  `<1>2` の 4 つの 1 歩は `origin` の値をそのまま返すので、`Join` が作られるのは `<1>3` の
  `Binding::Join` の 1 歩においてだけである。`(x, λ)` から `Binding::Join` の 1 歩を持つ最初のスロットを
  `(z, μ)` とすると、そこまでの 1 歩は `<1>2` の 4 つなので `origin(x, λ) = origin(z, μ)` であり、
  `<1>3` より `cand(z, μ) = S = ⋃_{a} act(a, μ)` である。
  BY <1>1, <1>2, <1>3, <1>4

### 9.4 アーム結果と消費される leaf の候補は所有される

#### L11

**言明**。`x` の `Binding` が `Join(arm_results)` であるとし、`a ∈ arm_results`、`λ ∈ leaves(ty(a))` と
する。このとき `act(a, λ)` の各元 `(r, p)` について `ctx.owns_object(r, p)` は真である。

<1>1. `V` が `f_own` かグローバル初期化子であるとき、言明は L8 による。
  BY L8

<1>2. 以下 `V = f_borrow` とする。`B_V` は入力の関数 `func` の本体を `ρ_f` で付け替えたものであり、
      `a = ρ_f(a_0)` である。ここで `a_0` は `func.body` の対応する `Match` の対応するアームの
      `returned_var` である。
  P9 の前半より `B_V` は `func.body` の束縛変数を一斉に付け替えたものであり、`collect_bindings` は本体の
  形だけから `Join(arm_results)` を作り、その `arm_results` は各アーム本体の `returned_var` である。
  BY P9, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var

<1>3. `cand(a, λ)` の各元 `(r, p)` について `ctx.owns_object(r, p)` は真である。
  `returned_var(arm.body)` は そのアーム本体の終端の `Ret` が名指す変数であり、`collect_consumes_go` は
  `RcRhs::Match` の腕で各アーム本体へ降り、その `RcExpr::Ret` の腕で `push_boxed_leaves` を呼ぶので、
  `(a_0, λ)` は `collect_consumes` が積む元である。第 3.7 節の系より、`cand_f(a_0, λ)` の各元 `(r_0, p)`
  について `ctx.owns_object(ρ_f(r_0), p)` は真である。`L15` (`p15-ownership-uniformity.md`) より
  `cand(a, λ) = ρ_f(cand_f(a_0, λ))` である。
  BY 3.7 の系, <1>2, p15-ownership-uniformity.md の L15,
     CODE src/rc_ir/ownership.rs: collect_consumes_go, returned_var, push_boxed_leaves

<1>4. `identity(a, λ)` について `ctx.owns_object` は真である。
  `origin(a, λ)` が `Origin::Exactly(p)` のとき `identity` は `p` であり `cand(a, λ)` の元なので `<1>3` に
  よる。`Origin::Join` のとき、`L13` (`p15-ownership-uniformity.md`) より `Join` の `identity` の変数は
  `collect_bindings` が入れる `Binding` を持つので `param_tys` の鍵ではなく、L4 より `owns_object` は
  真である。
  BY L4, <1>3, p15-ownership-uniformity.md の L13, CODE src/rc_ir/ownership.rs: Origin::identity

<1>5. QED
  `Origin::acted_on` より `act(a, λ) = cand(a, λ) ∪ {identity(a, λ)}` である。
  BY <1>1, <1>3, <1>4, CODE src/rc_ir/ownership.rs: Origin::acted_on

#### L12 (スロットの所有は由来の所有である)

**言明**。`ρ` の上のスロット `(x, λ)` について、次の 2 つは同値である。

1. `cand(x, λ)` のすべての元 `(r, p)` について `ctx.owns_object(r, p)` が真である。
2. `ctx.owns_object(T_ρ(x, λ))` が真である。

<1>1. CASE `origin(x, λ)` が `Origin::Exactly` である。
  L10 より `cand(x, λ) = {T_ρ(x, λ)}` なので、1 と 2 は同じ条件である。
  BY L10

<1>2. CASE `origin(x, λ)` が `Origin::Join` である。
  <2>1. L10 の (c) より `cand(x, λ) = ⋃_{a ∈ arm_results(z)} act(a, μ)` であり、L11 よりその各元に
        ついて `ctx.owns_object` は真である。よって 1 は真である。
    BY L10, L11
  <2>2. QED
    L10 の (a) より `T_ρ(x, λ) ∈ cand(x, λ)` なので、`<2>1` より 2 も真である。1 と 2 がどちらも真なので
    同値である。
    BY L10, <2>1

<1>3. QED
  `Origin` は `Exactly` と `Join` の 2 つの構成子を持つ。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: Origin

#### L13 (site の下の inhabited な leaf の由来は `owns_unit` と一致する)

**言明**。`(v, u)` を `B_V` について `levelled_sites` が挙げる site とし、`λ` を `u` の下の boxed leaf で
`ρ` のこの位置で inhabited なものとする。このとき `ctx.owns_unit(v, u)` が真であることと
`ctx.owns_object(T_ρ(v, λ))` が真であることとは同値である。

<1>1. `ctx.owns_unit(v, u)` が真ならば、`cand(v, λ)` のすべての元について `owns_object` は真である。
  P7a の節 1 から節 3 への含意である。`λ` は `Λ(u)` の inhabited な leaf である。
  BY P7a

<1>2. `ctx.owns_unit(v, u)` が偽ならば、`cand(v, λ)` に `owns_object` が偽である元がある。
  P7a の節 2 から節 1 への含意の対偶より、節 2 が偽である。節 2 は「`Λ(u)` のある inhabited な leaf の
  すべての候補について `owns_object` が真」なので、その否定は「`Λ(u)` のどの inhabited な leaf にも
  `owns_object` が偽である候補がある」である。
  BY P7a

<1>3. QED
  L12 より「`cand(v, λ)` のすべての元について `owns_object` が真」と `owns_object(T_ρ(v, λ))` は同値で
  ある。`<1>1` と `<1>2` がその両向きを与える。
  BY L12, <1>1, <1>2

**この命題を検査するコード**。`develop_mode` のとき `borrow_ify` は借用版ごとに
`RewriteCtx::check_ownership_is_levelled` を呼び、`levelled_sites` の各 site について候補ごとの
`owns_object` が一致することを `assert!` する
(`CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled`, `borrow_ify`)。

#### L14 (消費される leaf の由来は所有される)

**言明**。`ρ` の上のスロット `(w, μ)` が、`B_V` のある節点で所有を読まない消費 (DEF 所有を読まない消費) に
よって消費されるとする。このとき `ctx.owns_object(T_ρ(w, μ))` は真である。

<1>1. `V` が `f_own` かグローバル初期化子であるとき、L8 による。
  BY L8

<1>2. `V = f_borrow` のとき、`cand(w, μ)` の各元について `ctx.owns_object` は真である。
  P9 の前半より `B_V` は `func.body` の付け替えであり、所有を読まない消費はどれも節点の形だけで決まる。
  よって `func.body` の対応する節点が対応する leaf `(w_0, μ)` を同じ行で消費する。第 3.7 節の系と `L15` (`p15-ownership-uniformity.md`) より、
  `cand(w, μ) = ρ_f(cand_f(w_0, μ))` の各元について `ctx.owns_object` は真である。
  BY D9, P9, 3.7 の系, p15-ownership-uniformity.md の L15

<1>3. QED
  L12 と L10 より、`cand(w, μ)` の全元が所有されることと `T_ρ(w, μ)` が所有されることは同値である。
  BY L10, L12, <1>1, <1>2

### 9.5 `App` の呼び出し先の所有

#### L15

**言明**。`B'_V` の節点 `Let(x, App(callee', args), k)` について、この活性化がこの位置で作る活性化の本体を
持つ関数 (D9 と D10 が「呼び出し先」と呼ぶもの) を `W` とする。各引数の添字 `i` と各
`u ∈ units(ty(args[i]))` について、`W` が `(W のパラメータ i, u)` を D14 の意味で所有することと、
P11 の `callee_owns(i, u)` が真であることとは同値である。

<1>1. CASE `callee'.name` が `ctx.callee_params` の鍵である。
  <2>1. `W` は `callee'.name` を名前とする出力の版である。
    A6 より束縛変数の名前はどの関数の名前とも異なるので、`callee'` は局所変数ではなく、出力の関数を
    名指す `RcVar` である (L6 より `callee_params` の鍵は出力の `funcs` の鍵ちょうどである)。コード生成は
    `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕で `self.get_scoped_obj(&callee.name)` を
    `apply_lambda` に渡す。その値は関数を名指す名前のもの、すなわちその名前の版の funptr であり、
    `apply_lambda` はその関数を呼ぶ。
    BY A6, L6, CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
       CODE src/generator.rs: Generator::apply_lambda
  <2>2. `callee_params[callee'.name][i].0` は `W` の第 `i` パラメータの名前である。
    `callee_params` は入力の各関数について `param_names_and_types(func)`、各借用版について
    `param_names_and_types(clone)` を入れる。`param_names_and_types` は `params` に `capture` を鎖にした
    列である。A14 より `i` は `params` の範囲内なので、第 `i` 元はパラメータである。
    BY A14, CODE src/rc_ir/borrow.rs: borrow_ify, param_names_and_types
  <2>3. QED
    P11 の `callee_owns(i, u)` は `owned_units.contains(&(callee_params[..][i].0, u))` である。第 8 節の
    系 3 より、これは `W` が `(その第 i パラメータ, u)` を D14 の意味で所有することと同値である。
    BY 第 8 節の系 3, <2>1, <2>2

<1>2. CASE `callee'.name` が `ctx.callee_params` の鍵でない。
  <2>1. `callee_owns(i, u)` は真である。
    `call_rc` は `params` が `None` のとき `true` を使う。
    BY P11, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc
  <2>2. `W` は全所有版か、`prog.funcs` に無い関数である。
    L6 より `callee_params` の鍵でない名前は、入力の関数の名前でも借用版の名前でもない。よって
    `callee'` は局所変数であり、この呼び出しは間接呼び出しである。その値はクロージャであり、その対象は
    それを作った `RcRhs::Closure(fref, caps)` の `fref` である。`rewrite_inner` は `RcRhs::Closure` を
    持つ `Let` を `RcExpr::Let(x.clone(), rhs.clone(), self.rewrite(k))` に写すので `fref` は入力の関数の
    名前のままであり、L6 よりその名前の出力の版は `f_own` である。`prog.funcs` に無い名前を持つ
    呼び出し先は A7 が扱う。
    BY A7, L6, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
  <2>3. QED
    第 8 節の系 1 より `f_own` の `borrowed_units` は空なので、D14 より `f_own` は全パラメータの全 unit を
    所有する。`prog.funcs` に無い呼び出し先は A7 より同じである。
    BY A7, D14, 第 8 節の系 1, <2>1, <2>2

<1>3. QED
  BY <1>1, <1>2

#### L18 (振り分けられる呼び出し先は boxed leaf を持たない)

**言明**。`B_V` の節点 `Let(x, App(c, args), k)` について、`ctx.route(x, c, args, k)` が `c` と異なる名前の
`RcVar` を返すならば `leaves(ty(c)) = ∅` である。`route` が `c` をそのまま返すときは、`B'_V` の対応する
節点の callee は `c` と同じ `RcVar` である。

<1>1. `route` が異なる名前を返すのは、`borrow_versions` が `FuncRef { name: c.name }` を鍵に持つときだけで
      ある。
  BY P12 (a)

<1>2. `borrow_versions` の鍵は `prog.funcs` の鍵である。
  BY L6

<1>3. `prog.funcs` の関数の名前は、`sym.ty.is_funptr()` が真の記号の名前である。`lower_symbol` は
      `is_funptr` が真の記号だけを `LoweredSymbol::Func` にし、偽の記号はグローバル初期化子にする。
  BY CODE src/rc_ir/lower.rs: Lowerer::lower_symbol

<1>4. `ty(c)` は funptr 型である。
  `Lowerer::lower_var` は、局所に解決しない名前について記号の型を持つ `RcVar` を作る。A12 より同じ名前の
  `RcVar` が持つ型は一致するので、`c.name` を持つどの `RcVar` の型もその記号の型であり、`<1>3` よりそれは
  funptr 型である。
  BY A12, <1>3, CODE src/rc_ir/lower.rs: Lowerer::lower_var

<1>5. QED
  `TypeNode::is_fully_unboxed` は `is_funptr` が真のとき真を返し、D4 の第 1 の規則より
  `is_fully_unboxed` が真の型は leaf を持たない。`route` が `c` をそのまま返す場合は
  `rewrite_inner` がその値を `RcRhs::App` に入れるので、callee は同じ `RcVar` である。
  BY D4, <1>1, <1>4, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

### 9.6 出力の本体の節点

#### L16

**言明**。`B'_V` の実行路は `B_V` の実行路と 1 対 1 に対応し、対応する路の上で `Retain`/`Release` 以外の
節点の列は、**`Let(x, App(callee, args), k)` 節点の `callee` の名前を除いて**等しい。すなわち対応する
`App` 節点は、`x`・`args`・継続を共有し、`callee` は `B'_V` では `ctx.route(x, callee, args, k)` の値で
ある。`route` は `callee` を複製して `name` だけを差し替えるので、両者の `ty` は等しい (P12 (a))。
`B'_V` の `Retain`/`Release` 節点は次の 3 種で尽きる。

- **(K)** `B_V` の `Retain(v, π)`/`Release(v, π)` 節点のうち、`V` が借用版でないか `owns_unit(v, π)` が
  真であるもの。同じ変数・同じ path・同じ位置に立つ。
- **(A-前)** `B_V` の `Let(x, App(callee, args), k)` 節点ごとに、P11 の `before` の各元 `(a, u)` に
  ついての `Retain(a, u)`。`App` の直前に立つ。
- **(A-後)** 同じ節点ごとに、P11 の `after` の各元 `(a, u)` についての `Release(a, u)`。`App` の直後に
  立つ。

<1>1. `rewrite_inner` は `RcExpr` の 6 種のそれぞれについて、同じ種類の節点を作るか (`Let` の 3 つの腕、
      `Destructure`、`Eval`、`Ret`)、`rewrite_rc` を呼ぶ (`Retain`、`Release`)。`Let(x, App(..), k)` の
      腕は `prepend_rc` で `Retain`/`Release` の鎖を前後に足す。`Let(x, Match(scrut, arms), k)` の腕は
      アームの数・`tag`・`payload` を変えずに各アーム本体を書き換える。
  BY P10, P11, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

<1>2. `rewrite_rc` が作る節点は、`V` が借用版でないとき元の節点そのもの、借用版のとき A2 と L3 より
      `owns_unit(v, π)` が真ならば元の節点そのもの、偽ならば節点無しである。
  BY A2, L3, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>3. QED
  `<1>1` と `<1>2` より、`B'_V` は `B_V` の木から `Retain`/`Release` を落とし、`App` の前後に
  `Retain`/`Release` を足し、`App` の `callee` を `route` の値に差し替えたものである。`Match` のアームの
  構造 (D3 が実行路を作るのに使う唯一の分岐) は変わらないので、実行路は 1 対 1 に対応する。`route` が
  返す `RcVar` は `callee.clone()` か、その複製の `name` を差し替えたものなので `ty` は等しい (P12 (a))。
  BY D3, P10, P11, P12, <1>1, <1>2

### 9.7 由来ごとの義務

**DEF 由来ごとの義務** -- 本体 `C` (`B_V` か `B'_V`) と、`V` のパラメータ・capture の leaf に 0 か 1 を
与える**初期値の規則** `ι` について、`ρ` に対応する `C` の実行路の上の位置 `τ` と `ρ`-由来 `T` に対する
整数 `n^ι_C(τ, T)` を次で定める。ただし `obj(T)` が計数下 (D26) でない `T` については定めない (D26 より
そのような `T` を指すスロットは D8 の意味の参照を持たない)。

- `T = (p, σ)` で `p` が `V` のパラメータか capture であるとき、初期値は `ι(p, σ)` である。
- それ以外の `T = (u, σ)` は、D10 の生成の表のいずれかの行が `u` に値を与える節点で 1 になる。その前は
  0 である。`u` が `ctx.vars.bindings` の鍵でないとき (`u` はグローバル値の名前である) は、D26 より
  `obj(u, σ)` はグローバル状態なので、この由来は勘定の外にある。
- `Retain(v, π)` の節点で、`π` の下の inhabited な各 leaf `λ` につき `n^ι_C(・, T_ρ(v, λ))` を 1 増やす。
- `Release(v, π)` の節点で、同じ `λ` につき 1 減らす。
- D9 の消費が inhabited な leaf `(w, μ)` を消費するとき、`n^ι_C(・, T_ρ(w, μ))` を 1 減らす。
- 他のどの節点も `n^ι_C` を変えない。

2 つの規則を使う。`ι_全` はすべての leaf に 1 を与える規則、`ι_V` は `ctx.owns_object(p, σ)` が真である
leaf に 1、偽である leaf に 0 を与える規則である。

**DEF n_in、n_out** -- `n_in := n^{ι_全}_{B_V}`、`n_out := n^{ι_V}_{B'_V}` と書く。

#### L19 (パラメータでない由来は所有される)

**言明**。`ρ`-由来 `T = (u, σ)` について、`u` が `V` のパラメータでも capture でもないならば、
`ctx.owns_object(u, σ)` は真である。

<1>1. `ρ`-由来では `ctx.vars.bindings.get(u)` は `None`、`Some(Binding::Param)`、`Some(Binding::Producer)`、
      boxed 容器の `Some(Binding::Field(..))`、`Some(Binding::Payload(s, Some(t)))` で `s` が boxed、
      `Some(Binding::Llvm(..))` で `decl.leaf_origins_at(σ)` が単一の `Arg` でないもの、のいずれかである。
  DEF 由来の 1 歩 の表の 6 行のどれにも当たらない場合がこれである。
  BY DEF 由来の 1 歩, CODE src/rc_ir/ownership.rs: origin_inner, Binding

<1>2. `Some(Binding::Param)` は仮定から除かれる。
  `VarTable::of` が `Binding::Param` を入れるのは、その関数のパラメータと capture についてだけである。
  BY CODE src/rc_ir/ownership.rs: VarTable::of

<1>3. `None` のとき、`u` は `ctx.vars.param_tys` の鍵ではない。
  `VarTable::of` は各パラメータ・capture を `bindings` と `param_tys` の両方に入れ、`param_tys` に
  入れるのはそこだけである。
  BY CODE src/rc_ir/ownership.rs: VarTable::of

<1>4. 残る 4 つのとき、`u` は `ctx.vars.param_tys` の鍵ではない。
  この 4 つは `collect_bindings` が入れる `Binding` である。
  BY p15-ownership-uniformity.md の L13, CODE src/rc_ir/ownership.rs: collect_bindings

<1>5. QED
  L4 より `param_tys` の鍵でない `u` について `owns_object(u, σ)` は真である。
  BY L4, <1>1, <1>2, <1>3, <1>4

#### L17 (義務集合は由来ごとの和である)

**言明**。本体 `C` の各位置 `τ` と各計数下オブジェクト `O` について、`C` の 1 回の活性化の D10 の義務集合は
`Obl(τ)(O) = Σ_{T : obj(T) = O} n^ι_C(τ, T)` を満たす。ここで D10 の初期値を決める所有と借用の割り当ては、
`ι = ι_全` のときすべてのパラメータ・capture の unit を所有する割り当て、`ι = ι_V` のとき出力における
`V` の割り当て (P13) である。

<1>1. 初期値が合う。
  D10 の初期値は、所有するパラメータ・capture の unit の下の inhabited な各 leaf につき 1 つである。
  パラメータ leaf の由来は自分自身である (`Binding::Param` の腕は `here()` を返すので DEF 由来の 1 歩 の
  「上のどれでもない」に当たる)。`ι = ι_全` のときは全 unit が所有されるので、両辺はどの inhabited な
  パラメータ leaf も 1 と数える。`ι = ι_V` のときは、第 8 節の系 3 と P7e より
  `ctx.owns_object(p, σ) = ctx.owns_object(p, trunc(ty(p), σ))` であり、これは `V` が unit
  `trunc(ty(p), σ)` を D14 の意味で所有することと同値なので、両辺は同じ leaf を数える。P1 より
  `trunc(ty(p), σ)` は `units(ty(p))` の元であり、D10 の初期値が渡る unit の 1 つである。
  BY D10, D14, P1, P7e, 第 8 節の系 3, CODE src/rc_ir/ownership.rs: origin_inner

<1>2. 生成が合う。
  D10 の生成の表の 5 行 -- `Llvm` の結果の leaf で宣言が単一の `Arg` でないもの、`App` の結果の各 boxed
  leaf、`Closure` の結果、boxed 容器の `Destructure` の名前付きフィールドの各 leaf、boxed union の変位
  アームの payload の各 leaf -- が値を与える変数の `Binding` は、順に `Llvm` (単一 `Arg` でない)、
  `Producer`、`Producer`、boxed 容器の `Field`、boxed の `Payload(s, Some(t))` である。DEF 由来の 1 歩 の
  表より、これらはいずれも 1 歩を持たないので、生じた leaf は自分自身を由来とする。D10 は生じた inhabited な
  各 leaf につき参照を 1 つ加え、DEF 由来ごとの義務 はその由来を 1 にする。
  BY D10, CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner

<1>3. `Retain`・`Release`・消費が合う。
  D10 の `Retain(v, π)` の行は `π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ加え、
  `Release` の行は 1 つ取り除く。D10 の消費の行は消費される inhabited な各 leaf につき 1 つ取り除く。
  L9 より `obj(T_ρ(v, λ)) = obj(v, λ)` なので、DEF 由来ごとの義務 の同じ 3 行が同じオブジェクトについて
  同じ増減を与える。
  BY D10, L9

<1>4. 移動は両辺を変えない。
  D9 より移動は義務集合を変えない。DEF 由来ごとの義務 も移動の構文を挙げていない。L9 より移動の前後の
  2 つのスロットの由来は同じである。
  BY D9, L9

<1>5. QED
  D9 と D10 より、義務集合を動かす構文は生成・消費・`Retain`・`Release` の 4 つで尽きる。`<1>1` から
  `<1>4` がそのすべてと初期値を与える。
  BY D9, D10, <1>1, <1>2, <1>3, <1>4

## 10. P14 -- `borrow_ify` は RC 規律を保存する

**言明** (README の P14)。D12 の意味で RC 規律を満たし、かつ A1 と A2 を満たすプログラムを入力とすると、
`borrow_ify` の出力は D12 の意味で RC 規律を満たす。

### 10.1 示す形

**言明**。出力の各版 `V`、`B_V` の各実行路、それを辿る各活性化について `B'_V` が (S-a)・(S-b)・(S-c) を
満たすことを示せば、P14 が出る。

<1>1. 出力のプログラムの本体は、入力の各関数 `func` についての `f_own` の `body`、`borrow_versions` に載る
      各関数についての `f_borrow` の `body`、入力の各グローバル初期化子についての出力の `init` で尽きる。
  BY L6, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. D12 は、これらの本体のそれぞれが、出力の `borrowed_units` が定める割り当て (D14) の下で D11 を
      満たすことである。P13 よりその割り当ては `owned_units` が定めるものであり、第 8 節の系 3 より
      `ctx.owns_object` がパラメータ unit についてそれを答える。
  BY D12, D14, P13, 第 8 節の系 3

<1>3. QED
  以下、出力の版 `V` を 1 つ取り、第 9.1 節のとおり `B_V`、`B'_V`、`ctx`、実行路 `ρ`、活性化を固定して、
  `B'_V` の対応する実行路について (S-a)、(S-b)、(S-c) を示す。`V` と実行路と活性化は任意なので、これで
  `<1>2` が出る。
  BY D11, <1>1, <1>2

### 10.2 `B_V` は入力の割り当ての下で D11 を満たす

**言明**。`B_V` は、すべてのパラメータ・capture の unit を所有する割り当ての下で D11 を満たす。

<1>1. `V` が `f_own` かグローバル初期化子であるとき、`B_V` は入力の関数の本体または入力のグローバル
      初期化子の `init` そのものである。A1 と D12 よりそれは D11 を満たし、A1 よりその割り当ては全所有で
      ある。
  BY A1, D12, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `V = f_borrow` であるとき、`B_V` は入力の関数 `func` の本体を `ρ_f` で一斉に付け替えたものである。
  BY P9

<1>3. 一斉の名前替えは D11 を保つ。
  D3 の実行路、D6 のスロット、D9 の消費と移動、D10 の義務集合、D11 の 3 つの節は、いずれも本体の節点の
  種類・並び・`FieldPath`・変数の型・呼び出し先の所有だけから定まり、束縛名がどの文字列であるかを読まない。
  P9 の前半よりこの 5 つは `ρ_f` の下で保たれる -- 節点の種類・並び・`FieldPath`・`MatchArm` の `tag` は
  変わらず、`rename_var` は型を残し、`RcRhs::Closure` の `FuncRef` と `App` の呼び出し先の名前 (束縛では
  ないので `rename_f` の鍵ではない) も変わらない。`ρ_f` は単射なので、A6 と A11 が要求する束縛と使用の
  対応も保たれる。`func` のパラメータ・capture と `f_borrow` のそれは `ρ_f` で対応し、型は等しい。
  BY A6, A11, D3, D6, D9, D10, D11, P9, CODE src/rc_ir/rename.rs: rename_var, rename_rhs

<1>4. QED
  BY <1>1, <1>2, <1>3

### 10.3 主不変条件

**DEF 塊** -- `B_V` の 1 つの節点が `B'_V` で写る節点の列を、その節点の**塊**と呼ぶ。L16 より、
`Retain`/`Release` の塊は同じ節点 1 つか空、`Let(x, App(..), k)` の塊は (A-前) の列・`App`・(A-後) の列、
残る節点の塊は同じ節点 1 つである。

**DEF 対応する位置** -- `B_V` の実行路 `ρ` の上の**位置**とは、`ρ` の上の各節点の入口と、終端の `Ret` の
消費の後の 1 点をいう。節点の入口には `B'_V` のその節点の塊の入口が、終端の `Ret` の消費の後には `B'_V` の
終端の `Ret` の消費の後が対応する。

**INV**。`ρ` の上の各位置と、`B'_V` の対応する位置について、各 `ρ`-由来 `T` (計数下) について次が
成り立つ。

- `ctx.owns_object(T)` が真ならば `n_out(τ, T) = n_in(τ, T)`。
- 偽ならば `n_out(τ, T) = 0`。

以下「`T` は所有される」を `ctx.owns_object(T)` が真であることの略とする。

<1>1. 生成される由来は所有される。
  生成される由来の変数は `V` のパラメータでも capture でもない -- D10 の生成の表の 5 行が値を与えるのは
  `Let` の束縛変数、`Destructure` のフィールド変数、`Match` のアームの payload 変数であり、`VarTable::of`
  が `Binding::Param` を入れるのはパラメータと capture についてだけだからである。L19 より所有される。
  BY D10, L19, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings

<1>2. 根では INV が成り立つ。
  `T` がパラメータ・capture の leaf `(p, σ)` のとき、`n_in` の初期値は `ι_全` より 1、`n_out` の初期値は
  `ι_V` より `owns_object(p, σ)` が真なら 1 偽なら 0 である。それ以外の由来は両方 0 である。
  BY DEF 由来ごとの義務, DEF n_in、n_out

<1>3. CASE `τ` の節点が `Retain` でも `Release` でも `Let(x, App(..), k)` でもない。
  <2>1. L16 より `B'_V` の対応する位置の節点は同じ節点である。よって D10 の生成と D9 の消費は両側で
        同じ leaf について起きる。`Let(x, Match(scrut, arms), k)` の節点については、`Match` 節点自身は
        参照を作らず、移さず、手放さず (D9)、変位アームの payload 束縛は boxed の scrutinee のとき D10 の
        生成、unbox の scrutinee と catch-all のとき D9 の移動であり、L16 よりアームの `tag` と `payload` は
        両側で同じである。
    BY D9, D10, L16
  <2>2. この節点が行う消費は、所有を読まない消費 (DEF 所有を読まない消費) である。
    D9 の消費の表で `App` の引数の位置以外の行を行うのは、`Closure`、`Llvm`、`Destructure` の 2 行、
    終端の `Ret` であり、いずれもこの場合の節点である。`App` の callee の行はこの場合の節点ではない。
    BY D9, DEF 所有を読まない消費
  <2>3. この節点が消費する leaf の由来は所有される。
    BY L14, <2>2
  <2>4. QED
    `<2>1` より両側の増減は同じであり、`<2>3` と `<1>1` よりその増減が当たる由来はすべて所有される。
    所有される `T` については両辺が同じだけ動くので等式が保たれ、所有されない `T` については両側とも
    動かないので 0 のままである。
    BY <1>1, <2>1, <2>3

<1>4. CASE `τ` の節点が `Retain(v, π)` または `Release(v, π)` である。
  <2>1. `π ∈ units(ty(v))` であり、`(v, π)` は `B_V` について `levelled_sites` が挙げる site である。
    `B_V` が入力の本体のときは A2 がそのまま与える。`B_V` が `f_borrow` の複製本体のときは、P9 の前半より
    複製は `FieldPath` を変えず、`rename_var` は `ty` を残すので、複製の `Retain`/`Release` の path も
    その変数の型の `rc_units` の元である。`levelled_sites` は `for_each_node` で本体の全節点を歩き、
    `Retain`/`Release` の節点について `(v, path)` を積む。
    BY A2, P9, CODE src/rc_ir/borrow.rs: levelled_sites, CODE src/rc_ir/ast.rs: for_each_node,
       CODE src/rc_ir/rename.rs: rename_var
  <2>2. `π` の下の inhabited な各 leaf `λ` について、`T_ρ(v, λ)` が所有されることと
        `ctx.owns_unit(v, π)` が真であることとは同値である。
    BY L13, <2>1
  <2>3. CASE `ctx.owns_unit(v, π)` が真である。L3 と L16 より `B'_V` は同じ節点を持つ。D10 より両側とも
        `π` の下の inhabited な各 leaf の由来を同じだけ動かし、`<2>2` よりその由来はすべて所有される。
    BY D10, L3, L16, <2>2
  <2>4. CASE `ctx.owns_unit(v, π)` が偽である。`owns_unit(v, π)` は `cand(v, π)` の全元についての
        `owns_object` の全称なので、偽であるとは `owns_object` が偽である候補が在ることである。L8 より
        `V` は `f_own` でもグローバル初期化子でもなく、借用版である。L3 と L16 より `B'_V` にこの節点は
        無いので `n_out` は動かない。`<2>2` よりこの節点が動かす由来はすべて所有されないので、`n_in` が
        動いても INV の 2 つの条件はどちらも保たれる。
    BY D10, L3, L8, L16, <2>2, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit
  <2>5. QED
    BY <2>3, <2>4

<1>5. CASE `τ` の節点が `Let(x, App(callee, args), k)` である。`B'_V` の対応する塊は L16 の (A-前) の列、
      `App` の節点、(A-後) の列である。この塊の全体を通した後で INV が成り立つことを示す。
  <2>1. `App` の結果 `x` の各 boxed leaf の生成は両側で同じであり、その由来は `<1>1` より所有される。
        `App` の callee の全 boxed leaf の消費も両側で同じ由来を同じだけ動かす。
    D10 の生成の `App` の行は所有の割り当てを読まず、L16 より `x` と `args` と継続は両側で同じである。
    callee については L18 が場合を分ける -- `route` が名前を差し替えたときは `leaves(ty(c)) = ∅` なので
    D9 の `App` の行の callee の部分はどちらの側でも何も消費せず、差し替えないときは両側の callee が
    同じ `RcVar` なので同じ leaf を消費し、その由来は L14 より所有される。
    BY D9, D10, L14, L16, L18, <1>1
  <2>2. 各引数の添字 `i` と各 `u ∈ units(ty(args[i]))` をとる。`(args[i], u)` は `B_V` について
        `levelled_sites` が挙げる site である。
    `levelled_sites` は `Let(_, App(_, args), _)` の各 `arg` と各 `unit ∈ rc_units(arg.ty)` について
    `(arg, unit)` を積む。
    BY CODE src/rc_ir/borrow.rs: levelled_sites
  <2>3. `u` の下の inhabited な各 leaf `λ` について、`T_ρ(args[i], λ)` が所有されることと
        P11 の `arg_owned(i, u)` が真であることとは同値である。
    BY L13, <2>2
  <2>4. `B_V` の `App` は、`u` の下の inhabited なすべての leaf を消費する。
    A1 より入力のすべての関数の `borrowed_units` は空なので、D14 よりこの `App` の呼び出し先が
    `prog.funcs` に在れば全パラメータの全 unit を所有し、`prog.funcs` に無ければ A7 より同じである
    (`B_V` が `f_borrow` の複製本体であるときも、その `App` が名指すのはまだ振り分けられていない入力の
    呼び出し先である -- 振り分けは `rewrite` が行う)。D9 の `App` の行と A12 より、`u` の下の inhabited な
    各 leaf が消費される。
    BY A1, A7, A12, D9, D14, P12, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
  <2>5. `B'_V` の `App` は、`callee_owns(i, u)` が真のとき `u` の下の inhabited なすべての leaf を消費し、
        偽のとき 1 つも消費しない。
    L15 より `callee_owns(i, u)` は呼び出し先が `u` を D14 の意味で所有することと同値であり、A12 より
    `u` は呼び出し先のパラメータの型でも同じ unit である。D9 の `App` の行がこれを与える。
    BY A12, D9, L15
  <2>6. CASE `callee_owns(i, u)` が真である。
    <3>1. `arg_owned(i, u)` が真のとき、P11 より (A-前) にも (A-後) にもこの `(args[i], u)` は入らない。
          `<2>4` と `<2>5` より両側とも `u` の下の inhabited な各 leaf の由来を 1 減らし、`<2>3` より
          その由来は所有される。よって INV は保たれる。
      BY P11, <2>3, <2>4, <2>5
    <3>2. `arg_owned(i, u)` が偽のとき、P11 より (A-前) に `(args[i], u)` が入り、(A-後) には入らない。
          `n_out` は (A-前) の `Retain(args[i], u)` で `u` の下の inhabited な各 leaf の由来を 1 増やし、
          `App` で 1 減らすので、塊の前後で変わらない。`n_in` は `<2>4` より 1 減る。`<2>3` より
          その由来は所有されないので、INV の第 2 の条件 (`n_out = 0`) が保たれる。
      BY D10, P11, <2>3, <2>4, <2>5
    <3>3. QED
      BY <3>1, <3>2
  <2>7. CASE `callee_owns(i, u)` が偽である。
    <3>1. `arg_owned(i, u)` が真のとき、P11 より (A-後) に `(args[i], u)` が入る。`n_out` は `App` では
          動かず (`<2>5`)、(A-後) の `Release(args[i], u)` で `u` の下の inhabited な各 leaf の由来を
          1 減らす。`n_in` は `<2>4` より 1 減る。`<2>3` よりその由来は所有されるので、等式が保たれる。
      BY D10, P11, <2>3, <2>4, <2>5
    <3>2. `arg_owned(i, u)` が偽のとき、P11 より (A-前) にも (A-後) にも入らない。`n_out` は動かず、
          `n_in` は 1 減る。`<2>3` よりその由来は所有されないので、`n_out = 0` が保たれる。
      BY P11, <2>3, <2>4, <2>5
    <3>3. QED
      BY <3>1, <3>2
  <2>8. QED
    D9 の `App` の行 (callee の部分と引数の部分) と D10 の生成の `App` の行が、この節点が義務集合を動かす
    すべてであり、`B'_V` の側ではそれに (A-前) と (A-後) の `Retain`/`Release` が加わる (L16)。1 つの由来
    についての塊全体の変化は、これらの事象がその由来に与える増減の**和**である (DEF 由来ごとの義務 は
    各事象について 1 ずつ足し引きする)。その和を、`<2>1` が callee と結果について、`<2>6` と `<2>7` が
    各 `(i, u)` について与える。`u` は `units(ty(args[i]))` を渡り、P1 より `ty(args[i])` の各 leaf は
    ちょうど 1 つの unit へ切り詰まるので、引数の leaf は重複なく尽くされる。
    BY D9, D10, L16, P1, DEF 由来ごとの義務, <2>1, <2>6, <2>7

<1>6. QED
  D2 より `RcExpr` は 6 種であり、`Let` を `App`・それ以外に分けると `<1>3`・`<1>4`・`<1>5` が尽くす
  (`Let(x, Match(scrut, arms), k)` は `<1>3` に入る -- `Match` の節点自身は参照を作らず、移さず、
  手放さない (D9))。`<1>2` を基底、`<1>3`-`<1>5` を段とする、`ρ` の上の位置についての帰納である。各段は
  その節点の入口で INV を仮定して、その節点の**出口** -- DEF 対応する位置 の次の位置 -- で INV を示す。
  終端の `Ret` の出口は「終端の `Ret` の消費の後」であり、その節点は `<1>3` の場合なので、最後の位置も
  この帰納が覆う。
  BY D2, D9, <1>2, <1>3, <1>4, <1>5

### 10.4 `n_out` は非負であり、塊の中でも下回らない

**言明**。A19 (ii-a) の (a) の下で、`B'_V` の実行路の各位置 (塊の中を含む) で、各由来 `T` について
`n_out ≥ 0` である。さらに、`n_out` を 1 減らす各事象の直前で、その事象が減らす由来 `T` について
`n_out(T) ≥ 1` である。

<1>1. 塊の中で `n_out` を増やす事象は、`B'_V` の `Retain` 節点だけである。
  DEF 由来ごとの義務 が増やすのは `Retain` と生成の 2 つであり、生成は新しい由来を 0 から 1 にする --
  A6 より、生成の対象となる束縛変数はこの位置より前に値を得ておらず、生成される対 `(x, λ)` はこの位置より
  前のどの `ρ`-由来とも異なる。`B'_V` の `Retain` は L16 の (K) と (A-前) の 2 種である。
  BY A6, D10, L16, DEF 由来ごとの義務

<1>2. 所有されない由来 `T` を増やす事象は (A-前) の `Retain` だけであり、1 つの塊の中で `T` に当たる
      (A-前) の増分の総和は、同じ塊の `App` の消費が `T` に当たる減分の総和に等しく、増分はすべて減分より
      前に置かれる。
  <2>1. (K) の `Retain(v, π)` は所有されない由来を増やさない。その `Retain` が `B'_V` に残ったのは
        `V` が借用版でないか `owns_unit(v, π)` が真のときであり (L16 の (K))、前者では L8 よりすべての
        由来が所有され、後者では L13 より `π` の下の inhabited な leaf の由来はすべて所有される。
    BY L8, L13, L16
  <2>2. (A-前) に入る `(a, u)` は `callee_owns(i, u)` が真かつ `arg_owned(i, u)` が偽の対である。
        `Retain(a, u)` は `u` の下の inhabited な各 leaf の由来を 1 増やし、同じ塊の `App` は
        `callee_owns(i, u)` が真なので同じ leaf を消費して 1 減らす。`B'_V` の `App` が `u` の下の
        inhabited な leaf を消費するのは `callee_owns(i, u)` が真のときちょうどである -- L15 より
        `callee_owns(i, u)` は呼び出し先が `u` を D14 の意味で所有することと同値であり、A12 より `u` は
        呼び出し先のパラメータの型でも同じ unit なので、D9 の `App` の行がそれを与える。
        `prepend_rc(before, false, ..)` は (A-前) を `App` 節点の外側に置くので、増分は減分より前に
        起きる。
    BY A12, D9, L15, P11, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, prepend_rc
  <2>3. `App` の消費のうち、所有されない由来に当たるのは `arg_owned(i, u)` が偽である `(i, u)` の分だけで
        ある。`arg_owned(i, u)` が真の `(i, u)` では L13 より `u` の下の inhabited な leaf の由来はすべて
        所有される。callee の分は L14 と L18 より所有された由来に当たるか、何も消費しない。
    BY L13, L14, L18
  <2>4. QED
    `<2>2` の同値より、`callee_owns` が偽の `(i, u)` では `App` は `u` の下の leaf を消費しない。よって
    所有されない由来に当たる `App` の消費は、`callee_owns` が真かつ `arg_owned` が偽の `(i, u)`、すなわち
    (A-前) に入る対の分ちょうどである。
    BY P11, <2>1, <2>2, <2>3

<1>3. 所有されない由来 `T` について、塊の境界では `n_out(T) = 0` であり、塊の中では非負で、1 減らす事象の
      直前は 1 以上である。
  10.3 の INV より境界では 0 である。`<1>2` より塊の中の増分の総和と減分の総和は等しく、増分がすべて先に
  起きるので、途中の値は 0 以上であり、各減分の直前の値はその減分以降に残る減分の個数以上、すなわち
  1 以上である。
  BY 10.3 の INV, <1>2

<1>4. 所有される由来 `T` について、塊の境界では `n_out(T) = n_in(T) ≥ 0` である。
  BY A19, 10.3 の INV

<1>5. 所有される由来 `T` について、1 つの塊の中で `n_out(T)` は増えない。
  `<1>1` より増やすのは `Retain` だけである。(K) の `Retain(v, π)` は L16 より `B_V` にも同じ位置に在り、
  その塊は 1 つの節点なので、その節点の中に減分は無い。(A-前) の `Retain` が増やすのは `<1>2` より
  所有されない由来だけである。
  BY L16, <1>1, <1>2

<1>6. QED
  所有される由来については、`<1>4` より塊の前後の値がどちらも `n_in` に等しく、A19 (ii-a) の (a) より
  非負であり、`<1>5` より塊の中では減る一方なので、塊の中のどの値も塊の後の値以上である。よって塊の中でも
  非負であり、1 減らす事象の直前の値は塊の後の値 + (その事象以降に減る分) ≥ 1 である。所有されない由来に
  ついては `<1>3` が同じことを与える。
  BY A19, <1>3, <1>4, <1>5

### 10.5 (S-a) 過剰処分が無い

<1>1. `B'_V` で `Obl` から参照を取り除く操作は、`Release` 節点と D9 の消費である。
  BY D9, D10

<1>2. そのどの操作についても、取り除かれる参照はその時点の `Obl` に入っている。
  取り除かれる参照は、その操作が名指す inhabited な各 leaf `λ` について `obj(・, λ)` への参照 1 つで
  ある (D10)。DEF 由来ごとの義務 より、その 1 つは由来 `T_ρ(・, λ)` の分を 1 減らす。10.4 より、
  その直前に `n_out(T_ρ(・, λ)) ≥ 1` であり、10.4 より他のどの由来の `n_out` も非負なので、L17 より
  `Obl(τ)(obj(T_ρ(・, λ))) ≥ 1` である。1 つの操作が同じオブジェクトへの参照を 2 つ以上取り除くときも、
  それぞれが別の (あるいは同じ) 由来の分を減らし、10.4 がその各段で `n_out ≥ 1` を与える。
  BY D10, L9, L17, 10.4

<1>3. QED
  BY D11, <1>1, <1>2

### 10.6 (S-b) 漏れが無い

<1>1. `B'_V` の実行路の終端の `Ret(v)` は、`B_V` の終端の `Ret(v)` に対応する位置にあり、その後に
      `Retain`/`Release` 節点は無い。
  L16 より `B'_V` の `Retain`/`Release` は (K)、(A-前)、(A-後) の 3 種であり、(A-前) と (A-後) は `App` の
  直前・直後に立つ。`App` の節点は継続を持つので、終端の `Ret` より前にある。(K) は `B_V` の節点の位置に
  立つ。
  BY D2, D3, L16

<1>2. `B_V` の終端の `Ret(v)` の消費の後、`n_in(τ, T)` はどの由来 `T` についても 0 である。
  10.2 より `B_V` は D11 を満たすので、(S-b) よりその時点の `Obl` は空である。L17 より各計数下
  オブジェクト `O` について `Σ_T n_in(τ, T) = 0` であり、A19 (ii-a) の (a) より各項は非負なので各項が
  0 である。
  BY A19, D11, L17, 10.2 の言明

<1>3. QED
  「終端の `Ret` の消費の後」は DEF 対応する位置 が挙げる位置であり、10.3 の INV はそこでも成り立つ。
  よって所有される由来については `n_out = n_in = 0`、所有されない由来については `n_out = 0` である。
  L17 より `B'_V` の `Obl` はその時点で空である。`<1>1` よりその時点は `B'_V` の実行路の終端の `Ret` の
  消費の後である。
  BY D11, L17, DEF 対応する位置, 10.3 の INV, <1>1, <1>2

### 10.7 (S-c) 解放後の読みが無い

<1>1. `B'_V` の各位置で D7 の読む構文が読みうるオブジェクトと、`Retain`/`Release` が触れるオブジェクトは、
      次の 3 種で尽きる。
  - **(i)** `B_V` の読む構文が対応する位置で読みうるオブジェクト。L16 より読む構文の列は両側で等しく、
    D7 の表が名指す値も等しい。ただ 1 つの例外が `App` の callee であり、`route` が名前を差し替えた
    `App` では L18 より `leaves(ty(c)) = ∅` なので、その `App` が callee を通じて読みうるオブジェクトは
    どちらの側にも無い (D7 は「名指した値の inhabited な各 boxed leaf が指すオブジェクト」を読みうる
    先とする)。
  - **(ii)** `B_V` の `Retain(v, π)`/`Release(v, π)` のうち `B'_V` に残ったもの (L16 の (K)) が触れる
    オブジェクト。
  - **(iii)** L16 の (A-前) と (A-後) が触れるオブジェクト。
  BY D7, L16, L18

<1>2. グローバル状態 (D26) のオブジェクトは解放されない。
  BY A8, D26

<1>3. 計数下のオブジェクト `O` について、その時点で `Obl(τ)(O) ≥ 1` ならば `O` は解放されていない。
  D8 より `Obl` の元は未処分の参照であり、`H(O)` は未処分の参照の総数なので `H(O) ≥ 1` である。D7 より
  解放されるのは `H` が 0 になったオブジェクトである。
  BY D7, D8

<1>4. `V` が借用する unit の下の inhabited な leaf が指す計数下のオブジェクトは、この活性化の間 解放
      されていない。
  BY A20

<1>5. (i) と (ii) のオブジェクトは解放されていない。
  <2>1. そのオブジェクトはあるスロット `(x, λ)` の `obj(x, λ)` である。読む構文は名指した値の inhabited な
        各 boxed leaf が指すオブジェクトを読みうる (D7)。`Retain(v, π)`/`Release(v, π)` は `π` の下の
        inhabited な各 leaf が指すオブジェクトに触れる (D7)。
    BY D7
  <2>2. `obj(x, λ)` がグローバル状態のとき、`<1>2` による。
    BY <1>2
  <2>3. `obj(x, λ)` が計数下で `T_ρ(x, λ)` が所有されるとき。A19 (ii-a) の (b) と (b') より
        `n_in(τ, T_ρ(x, λ)) ≥ 1` であり、10.3 の INV より `n_out(τ, T_ρ(x, λ)) = n_in(τ, T_ρ(x, λ)) ≥ 1`
        である。10.4 より他の由来の `n_out` は非負なので、L17 より `Obl(τ)(obj(x, λ)) ≥ 1` である。
        `<1>3` より解放されていない。
    L9 より `obj(T_ρ(x, λ)) = obj(x, λ)` である。(b) と (b') が当たるのは、(i) と (ii) の位置が `B_V` でも
    読む構文または `Retain`/`Release` の位置だからである (`<1>1`)。
    BY A19, L9, L17, <1>1, <1>3, 10.3 の INV, 10.4
  <2>4. `obj(x, λ)` が計数下で `T_ρ(x, λ)` が所有されないとき。L19 より、パラメータでも capture でもない
        変数を持つ由来は所有されるので、所有されない由来 `T_ρ(x, λ) = (p, σ)` の `p` は `V` の
        パラメータか capture である。第 8 節の系 3 と P7e より `owns_object(p, σ)` が偽であることは
        `V` が unit `trunc(ty(p), σ)` を D14 の意味で借用することである。`<1>4` より解放されていない。
    BY D14, L19, P7e, 第 8 節の系 3, <1>4
  <2>5. QED
    BY <2>2, <2>3, <2>4

<1>6. (iii) のオブジェクトは解放されていない。
  <2>0. (A-前) と (A-後) が名指す `(a, u)` について、`a` は `App` の引数であり、`(a, u)` は `B_V` に
        ついて `levelled_sites` が挙げる site である。`u` の下の inhabited な各 leaf `λ` について、L13 より
        `T_ρ(a, λ)` が所有されることと `arg_owned(i, u)` が真であることとは同値である。
    `levelled_sites` は `Let(_, App(_, args), _)` の各 `arg` と各 `unit ∈ rc_units(arg.ty)` について
    `(arg, unit)` を積む。(A-前) と (A-後) の対は P11 より `args` の元と `units(ty(args[i]))` の元の対で
    ある。
    BY L13, P11, CODE src/rc_ir/borrow.rs: levelled_sites
  <2>1. (A-前) の `Retain(a, u)` が触れるオブジェクトは解放されていない。
        P11 より `arg_owned(i, u)` は偽なので、`<2>0` より `u` の下の inhabited な各 leaf `λ` の
        `T_ρ(a, λ)` は所有されない。`obj(a, λ)` がグローバル状態なら `<1>2` による。計数下ならば、
        L19 より所有されない由来の変数は `V` のパラメータか capture であり、第 8 節の系 3 と P7e より
        `V` はその unit を D14 の意味で借用するので、`<1>4` による。
    BY D14, L19, P7e, P11, 第 8 節の系 3, <1>2, <1>4, <2>0
  <2>2. (A-後) の `Release(a, u)` が触れるオブジェクトは解放されていない。
        P11 より `arg_owned(i, u)` は真なので、`<2>0` より `u` の下の inhabited な各 leaf `λ` の
        `T_ρ(a, λ)` は所有される。この `Release` は `n_out(T_ρ(a, λ))` を 1 減らす事象なので、10.4 より
        その直前に `n_out(T_ρ(a, λ)) ≥ 1` であり、10.4 より他の由来の `n_out` は非負なので、L17 より
        `Obl(obj(a, λ)) ≥ 1` である。`<1>3` より解放されていない。`obj(a, λ)` がグローバル状態なら
        `<1>2` による。
    BY L9, L17, P11, <1>2, <1>3, <2>0, 10.4
  <2>3. QED
    BY <2>1, <2>2

<1>7. QED
  BY D11, <1>1, <1>5, <1>6

### 10.8 P14 の QED

<1>1. 10.5、10.6、10.7 より、`B'_V` は出力の割り当ての下で D11 を満たす。
  10.5 と 10.6 は A19 (ii-a) を、10.7 は A19 (ii-a) と A20 を読む。
  BY A19, A20, D11, 10.5, 10.6, 10.7

<1>2. QED
  10.1 の言明より、`V` と実行路と活性化は任意でよい。よって出力のすべての本体が D11 を満たし、
  D12 が成り立つ。**この文書が置いた仮定は無い** -- `<1>1` が読む A19 (ii-a) と A20 は `README.md` の
  仮定であり、P14 の言明はその仮定の下で読むものである。
  BY A19, A20, D12, 10.1, <1>1

## 11. `README.md` へ差し戻す点

### 差し戻し 1 (A13 の検査)

A13 は「検査: 無し」と書く。対象コミットでは、`develop_mode` のとき `borrow_ify` が
`check_clone_names_are_fresh` を呼び、複製が導入する名前が入力プログラムのどの束縛名でもないことを
`assert!` する (`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`, `borrow_ify`)。A13 の検査の欄を
この関数の名で埋めるべきである。

### 差し戻し 2 (A19 (ii-a) を読む本体の範囲)

A19 の前置きは「`cancel` の入力の各本体」と書く。P14 が (ii-a) を読むのは **`borrow_ify` の入力の各本体**に
ついてである (第 9.1 節)。A19 の果たす者の欄が `insert_rc` と `borrow_ify` の 2 人を挙げていることは、
`insert_rc` の出力で成り立ち `borrow_ify` を通って `cancel` の入力まで保たれる、という読み方を前提に
している。前置きをその形 -- 「`insert_rc` の出力から `cancel` の入力までの各本体」 -- に改めるか、(ii-a) の
行に「P14 は `borrow_ify` の入力について読む」と書き足すのが素直である。

### 差し戻し 3 (由来と別名類は同じものである)

第 9.3 節の DEF 由来 は、`p13-disposals-and-pending.md` の第 7.5.2 節の `ρ`-終端と同じ規則であり、
`T_ρ` が等しいスロットの類が同節の別名類である。`README.md` の A19 は別名類を `p13` の定義で参照して
いる。1 つの定義を `README.md` に置き、両方の文書がそれを引くようにすべきである。名前は 1 つに決める。

### 差し戻し 4 (対象コミットと `develop_mode`)

`README.md` の第 1 節は対象コミットを `39c41033` と書き、`borrow_ify` の引数に `develop_mode` が無い版を
指している。対象コミットを `2d49d350` へ改め、`borrow_ify` が `develop_mode` を取り、それが真のときだけ
`check_clone_names_are_fresh` と `RewriteCtx::check_ownership_is_levelled` を呼ぶこと (どちらも出力を
変えない) を書くべきである。
