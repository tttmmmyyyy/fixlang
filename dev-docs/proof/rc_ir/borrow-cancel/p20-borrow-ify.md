# P8 - P14b -- `borrow_ify` と RC 規律の保存

対象コミットは `b6c51fb892746e493e155d9d59ea05d02d7357db` である。定義・仮定・命題の番号は同ディレクトリの
`README.md` による。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P8 (a) 停止性 | 証明した (第 3.3 節)。平準化の周回を含む |
| P8 (b) 不動点の閉包 | 証明した (第 3.4 節) |
| P8 (c) D9 の消費との対応 | 証明した (第 3.5 節)。`README.md` の P8 が文面に持つとおり `App` の引数の位置を除く形であり、その位置を P14 は `call_rc` が置く節点で扱う (第 3.6 節) |
| P9 複製は名前替えである | 証明した (第 4 節)。後半は A13 に立つ |
| P10 借用版が落とす RC 節点 | 証明した (第 5 節) |
| P11 呼び出し側の補正 | 証明した (第 6 節)。「**ちょうど**埋める」は P14 の中で示す (第 10 節) |
| P12 振り分けの安全性 | 証明した (第 7 節)。`funcs_observing_uniqueness` の門を含む |
| P13 注釈の一致 | 証明した (第 8 節) |
| P14 `borrow_ify` は RC 規律を保存する | **証明した** (第 10 節)。(S-a) と (S-b) は A19 (ii-a)・A13・A21・A23・A24 の下で、(S-c) は A19 (ii-a)・A20・A26・A13・A21・A23・A24 の下で |
| P14a 借用する終端の類は活性化の間 参照を持つ | **証明した** (第 11 節)。A20・A13・A21・A23・A24 の下で。A19 の (ii) も、D21 が活性化に課す制限 (A19 (i) の不等式) も読まない。示すのは README の形、すなわち**計数下** (D26) の類についての言明である |
| P14b 借用する本体の活性化は呼び出しが作る | **証明した** (第 12 節)。A13・A21・A24 と P9 の下で。`borrow_ify` の出力については L18a に立ち、`cancel` がそれを写したプログラムへは P22 と P24 の `cancel` の節で渡る |
| A19 (ii-c) の保存 | **示した** (第 13 節)。A19 (ii-a) の (a)・(a') の下で。`README.md` の A19 (ii-c) が果たす者に挙げる 2 人のうち、`borrow_ify` の側である |

**P14 が読む仮定。** `README.md` の第 7 節は、命題ごとに、その証明がどこまで進んでいるかと、証明が
どの仮定の下で立つかを 1 行に記録する。P14 の行が記録する仮定の、この文書での在りかは次のとおりである。

- **A19 (ii-a)** (各計数下の別名類の持つ参照の個数は非負であり、読む構文と `Retain`/`Release` がその類を
  名指す時点では 1 以上であり、非負であることは終端の `Ret` の消費を行った直後の時点についても言う)。
  第 9.1 節がこれをこの文書の記法 (由来) へ渡す。(ii-a) の非負性が終端の `Ret` の消費の後まで届くことを
  読むのは、(S-a) の終端の `Ret` の段 (第 10.4 節) と (S-b) (第 10.6 節) である。
- **A20** (借りた参照は活性化の間 生きている)。第 9.1 節がこれもこの文書の記法へ渡し、第 10.7 節の
  `<1>4` の `<2>2` が読む。
- **A26** (節点は、読んでから手放す)。第 10.7 節の `<1>5` の `<3>1` の `<4>3` が読む。
- **A23** (持ち上げた lambda は closure 型である)。`L18` の `<1>4` の `<2>2` が読む。`L18` を `BY` に
  挙げるのは、第 10.3 節の `<1>5` の `<2>1`、第 10.4 節の `<1>2` の `<2>3`、第 10.7 節の `<1>1`、
  第 10.8 節の `<1>1`、`L25` の `<1>1` の 5 か所である。前の 4 つが主不変条件を経て
  (S-a)・(S-b)・(S-c) のすべてに載せ、`L25` が P14a に載せる。
- **A24** (`fix` の op は capture を持つ本体にだけ在る) と **A21** (関数の値を作る演算)。`L18a` の
  `<2>3` の `<3>2` が A24 を、`<2>4` が A21 を読む。`L18a` を `BY` に挙げるのは、`L15` の `<1>2` の
  `<2>2` の `<3>2`、第 10.7 節の `<1>4` の `<2>1`、第 10.8 節の `<1>1`、第 11.7 節の `<1>4` の `<2>4`、
  第 12 節の `<1>5` と `<1>7` の `<2>3` の `<3>1`・`<3>2` の 7 か所である (第 9.1 節の A20 の項も
  その内容を述べる)。`L15` は第 10.3 節と第 10.4 節と `L26` が引くので、この 2 つは
  (S-a)・(S-b)・(S-c) と P14a に載り、第 12 節を経て P14b にも載る。
- **A13** (入力に現れる名前の形)。`L18` の `<1>4` の `<2>1` と `L18a` の `<1>2` が読む。`L18` と `L18a` を
  引く経路は上の 2 項と同じなので、A13 も同じ命題に載る。

**このほかに第 10.7 節の `<1>3` は、D21 が活性化に課す制限 -- A19 (i) の不等式 -- を読む。** これは
仮定ではなく、D21 が活性化の集合を絞る条件である (`README.md` の D21 と A19 (i))。義務集合が `O` への
参照を持つ点で `H(O) ≥ 1` であることを、そこから取る。そこから「`O` は解放されていない」へ渡すのは
D11 の (S-c) の接頭条件 -- その活性化がその点まで解放について閉じている (D11a) -- であり、第 10.7 節の
`<1>1a` がそれを仮定に置く。

**(S-c) は読みの点で示す。** D11 の (S-c) が課すのは、読み・触れる動作が実際に起きる瞬間の直前の点で
ある。第 9.7 節の DEF 時点 がその点を時点に数える。節点の入口の第 10.3 節の INV をその点へ渡すのは、
読みについては A26 -- 節点が行う記憶域からの読みはその節点が行うどの参照の手放しよりも前に起き、A26 の
「手放し」は D10 の消費と `Release` の両方なので、節点の入口から読みの点までに `n_out` を減らす事象は
無い -- であり、`Retain` が触れる動作については `Retain` の事象がどの由来も減らさないことである。`App` の
節点については、その節点が行う読みの点が節点の入口であること (第 9.7 節) が同じことを与える。

**第 9.5 節の L15 は P30 を読む。** `B'_V` の `App` の実行時の呼び出し先が `resolve_callee_params` の
引く関数と同じであることは、`borrow_ify` の**出力**についての言明であり、それを述べるのは P30 である
(P29 は入力についての言明である)。P30 の証明が読むのは P1・P9・P12・P24 なので、この向きに循環は生じない。

**P14 の実質は第 9 節にある。** 出力の各版の義務集合は、入力の義務集合を**由来 (DEF 由来) ごとに**
分けたときの、その版が所有する由来の分にちょうど等しい。借用する由来の分は呼び出し元が持ち、`call_rc` が
置く `Retain` は、借用する由来の参照を消費が要求する位置でだけ 1 つ作って同じ位置で消費させる。この
「由来ごとの収支」が、P11 の言明が保留した「**ちょうど**埋める」である。

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

**入力の関数の表について読む形。** `cand_f(x, π)` は `origin(vars_f, type_env, x, π).candidates()` を、
`act_f(x, π)` は `origin(vars_f, type_env, x, π).acted_on()` を集合とみなしたものである。ここで
`vars_f` は入力の関数 `func` の `VarTable::of(func)` である (下の「引用する他ファイルの補題」)。
`cand` と `act` は問題にしている版の `VarTable` について読むので、入力の関数の表について読むときは
この 2 つを使う。

`Obl` は D10 の義務集合、`H(o)` は D7 の参照カウントとする。`σ ⊑ π` は「`σ` は `π` の接頭辞である」。

`borrow_ify` の局所変数を次の名で参照する (`CODE src/rc_ir/borrow.rs: borrow_ify`)。

- `owned_leaves`: `infer_ownership` が返す `OwnedLeaves` の中身。leaf の集合である
  (`CODE src/rc_ir/borrow.rs: OwnedLeaves`)。以下では `OL` とも書く。
- `owned_units`: `borrow_ify` が組み立てる `Set<VarPath>`。unit の集合である。
- `borrow_versions`: 借用版を持つ関数からその版の名前への `Map`。
- `observing`: `funcs_observing_uniqueness(prog)` の値
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

**DEF site** -- 出力の版が書き換える本体 -- 関数の `body` かグローバル初期化子の `init` -- を
`for_each_node` で歩いて挙げた、`Retain`/`Release` 節点の `(v, path)` と、`App` の各引数 `arg` と各
`unit ∈ rc_units(ty(arg))` の対を、その本体の **site** と呼ぶ
(`CODE src/rc_ir/ast.rs: for_each_node`, `CODE src/rc_ir/ownership.rs: rc_units`)。

**この語を置く理由**。`levelled_sites` は `&RcFunc` を取るので、グローバル初期化子の版については
site を 1 つも挙げない。P7a と P7d はその点を避けて site を本体の側で定めており、`L13` と `L23` の
言明も同じ形を取る。2 つが同じ集合であることは `L13` の `<1>0` が述べ、関数の版でそれが
`levelled_sites` の挙げる集合と一致することは P7a が述べる。

**引用する他ファイルの補題**。この文書が引く外部の補題の言明を、引く形で写す。以下、`vars_f` は
`VarTable::of(func)`、`vars_c` は `VarTable::of(clone)` である (`func` は入力の関数、`clone` はその借用版)。

- **`p15-ownership-uniformity.md` の `L6`** -- `u ∈ units(τ)` のとき `trunc(τ, u) = u` であり、
  `under(τ, u) = [u]` である。
- **`p15` の `L9`** -- `trunc(τ, ・)` が `under(τ, p)` の各要素について値を返すとき、その値は `units(τ)` の
  要素である。
- **`p15` の `L11`** -- `r` を `vars.param_tys` が型 `τ` で持つ名前、`p` を path、`OL` を `VarPath` の
  集合とする。`covered_leaves(τ, p) ⊆ { λ : (r, λ) ∈ OL }` であり、かつ `covered_leaves(τ, p) ≠ ∅` または
  `under(τ, p) = []` であるとき、`owns_object_yet(vars, type_env, r, p, OL)` は真である。
- **`p15` の `L12`** -- `act(x, π) ⊆ Reach(x, π)` であり、とくに `cand(x, π) ⊆ Reach(x, π)` である。
  `Reach(x, π)` は `origin` の再帰が `(x, π)` から訪れる対の集合である (`p15` の第 4 節の
  `DEF 再帰で訪れる対`)。
- **`p15` の `L13`** -- `vars.bindings.get(w)` が `collect_bindings` の入れる `Binding` (`Move`、
  `Producer`、`Llvm`、`Field`、`Payload`、`Join`) のいずれかであるとき、`vars.param_tys` は `w` を鍵に
  持たない。したがって `owns_object(w, σ)` と `owns_object_yet(.., w, σ, ..)` はどちらも任意の `σ` に
  ついて真である。
- **`p15` の `L14a`** -- `x` を `func` に現れる名前とする。`origin` の再帰が `(x, π)` から訪れる各対
  `(y, ρ)` について、`y` も `func` に現れる名前である。ここで「`func` に現れる名前」とは、`func` の
  パラメータ・capture の名前と、`func.body` に現れる `RcVar` の名前の全体である
  (`p15` の第 1 節の (N))。
- **`p15` の `L15`** -- `rename` を `clone_func` が返す写像、`ρ_f` をそれを鍵でない名前の上の恒等写像で
  延ばしたものとすると、(i) `ρ_f` は `func` に現れる名前の上で**単射**であり、`func` に現れる名前のうち
  `rename` の鍵でないものは `rename` の像に入らない。`vars_c.param_tys` の鍵は `vars_f.param_tys` の鍵の
  `ρ_f` による像ちょうどであり、対応する型は等しい。(ii) `func` に現れる各名前 `x` と任意の path `π` に
  ついて `origin(vars_c, type_env, ρ_f(x), π) = ρ_f(origin(vars_f, type_env, x, π))` である
  (`ρ_f` は `VarPath` に対しその変数だけを写す)。
- **`p15` の `L16`** -- `OL` を `infer_ownership` の不動点の `owned_leaves`、`ctx` を `clone` の
  `RewriteCtx` とすると、`func` に現れる任意の名前 `r` と任意の path `p` について
  `ctx.owns_object(ρ_f(r), p) = owns_object_yet(vars_f, type_env, r, p, OL)` である。
- **`p15` の `L17`** -- ある出力版の `RewriteCtx` が `owns_unit(v, u)` を呼ぶとき、その版が関数の版で
  あれば `(v, u)` は**その版の本体**について `levelled_sites` が挙げる site である。グローバル初期化子の
  版では `owns_unit(v, u)` は真を返す。
- **`p13-disposals-and-pending.md` の `L7`** -- `leaves(τ)` の相異なる 2 元は、一方が他方の接頭辞に
  ならない。

**外部の結果**。この文書が引く、文書の外の名前つき結果を `EXT <名前>` の名札で据える。`BY` はこの名前で
引く。

- **EXT 集合と写像** -- `crate::misc::Set` は `FxHashSet`、`Map` は `FxHashMap` であり、どちらも
  ハッシャを替えた `std::collections::HashSet` と `HashMap` である
  (`CODE src/misc.rs: Set`, `Map`)。`HashSet::insert(v)` は、集合が `v` を持っていなければ `v` を加えて
  `true` を返し、持っていれば集合を変えずに `false` を返す。`HashSet::contains` は集合を変えない。
  `Extend::extend(iter)` は `iter` が返す各元を加え、それ以外の変更をしない。`HashMap` は 1 つの鍵に
  高々 1 つの値を持ち、`HashMap::insert(k, v)` はその鍵の値を `v` にする。`HashMap::values` と
  `HashMap::values_mut` は、その時点の各エントリの値をちょうど 1 度ずつ返す。
- **EXT 反復子の並び** -- `Iterator::filter(f)` は、元の反復子が返す元のうち `f` が真であるものを、元の
  順序のまま返す。`Iterator::chain(other)` は、自分の全元に続いて `other` の全元を、それぞれ 1 度ずつ
  順に返す。`Iterator::rev()` は両端反復子の元を逆順に返す。`Iterator::fold(init, f)` は `acc = init` から
  始めて、反復子が返す各元 `x` について `acc = f(acc, x)` を順に行い、最後の `acc` を返す。
- **EXT 導出した相等** -- `#[derive(PartialEq)]` を付けた構造体の 2 つの値が等しいのは、対応する各
  フィールドが等しいときであり、そのときに限る。
- **EXT 10 進表記** -- `format!("{}", n)` が `usize` の値 `n` について書き出す文字列は、10 進数字だけから
  なる。

**`p15` が固定するもの。** `p15` は 1 つの出力版とその `RewriteCtx` を固定して書かれており、そこでの
`owns(r, p)` はその版の `ctx.owns_object(r, p)`、`cand(x, π)` はその版の `vars` についての
`origin(x, π).candidates()` である (`p15` の第 1 節)。この文書が `p15` の補題と P7a を引くのは、`V` の
`RewriteCtx` と `B_V` についてである (第 9.4 節の L13 の `<1>0`)。

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
  <2>4. QED
    BY <2>3

<1>5. CASE `π = ρ ++ [c]` (`Capture` の腕で積まれた場合)。
  <2>1. `unit_step(σ, type_env)` は `UnitStep::Capture { capture_idx: c, .. }` である。
    BY <1>2, <1>3, CODE src/rc_ir/ownership.rs: rc_units_go の `UnitStep::Capture` の腕
  <2>2. `sub(τ, π)` は、`ρ` を降りた後の添字 `c` で `UnitStep::Capture` の腕に入り、`None` を返す。
    BY <1>3, <2>1, CODE src/rc_ir/ownership.rs: subtree_type の
       `UnitStep::NoUnit | UnitStep::Capture { .. } | UnitStep::Unit` の腕
  <2>3. `under(τ, π)` は `None` の腕を通り、`vec![π]` を返す。
    BY <2>2, CODE src/rc_ir/ownership.rs: units_under
  <2>4. QED
    BY <2>3

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
  BY EXT 集合と写像, <1>1, <1>2, CODE src/rc_ir/borrow.rs: borrow_ify

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
「入力の各関数の名前」と「`borrow_versions` の各値」の合併である。**また `borrow_versions` の鍵は
どれも入力の関数の名前であり、A22 よりそれは入力の `prog.funcs` の鍵である。**

<1>1. `callee_params` に元を入れるのは 2 か所である。入力の各関数について
      `callee_params.insert(func.name.clone(), ..)`、`clones` の各元について
      `callee_params.insert(borrow_version.clone(), ..)` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `borrow_versions` の鍵は入力の関数の名前だけであり、それは入力の `prog.funcs` の鍵である。
      1 番目のループは `prog.funcs.values()` を回り、`observing` が
      `func.name` を含むとき `continue` し、含まず `func.capture.is_none()` かつ
      `func_has_borrowable_param(func, &owned_leaves, type_env)` のときに
      `borrow_versions.insert(func.name.clone(), borrow_funcref(&func.name))` を行う。A22 より
      `prog.funcs` の各エントリの鍵はその `RcFunc` の `name` に等しいので、`func.name` はそのエントリの
      鍵である。
  BY A22, CODE src/rc_ir/borrow.rs: borrow_ify, funcs_observing_uniqueness, func_has_borrowable_param

<1>3. `borrow_versions` の値はすべて `clones` の第 1 成分として現れる。2 番目のループは入力の各関数に
      ついて `borrow_versions.get(&func.name)` を引き、`Some` のとき `clones.push((borrow_version, ..))` を
      行う。`<1>2` より `borrow_versions` の鍵はすべて入力の関数の名前なので、このループはそのすべてを
      引き当てる。
  BY EXT 集合と写像, <1>2, CODE src/rc_ir/borrow.rs: borrow_ify

<1>4. 出力の `funcs` に元を入れるのは 2 か所である。入力の各関数について
      `funcs.insert(f_own.name.clone(), f_own)` (`f_own.name` は `func.name` である)、`clones` の各元に
      ついて `funcs.insert(borrow_version, clone)` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>5. QED
  第 1 文は `<1>1`・`<1>3`・`<1>4` による。第 2 文は `<1>2` である。
  BY A22, <1>1, <1>2, <1>3, <1>4

### L6a (leaf の path では、`Arg` を宣言しない `Llvm` の腕は `here()` を返して再帰しない)

**言明**。`vars.bindings.get(u)` が `Some(Binding::Llvm(gen, args, rty))` であり、`σ` が `ty(u)` の boxed
leaf であり、`decl.leaf_origins_at(σ).and_then(as_arg_projection)` が `None` であるとする。ここで
`decl = gen.result_prov(rty, ..)` である。このとき `origin_inner(vars, type_env, u, σ)` は
`Origin::Exactly((u, σ))` を返し、その計算の中で `origin` を呼ばない。

<1>1. `decl.leaf_origins_under(σ)` が挙げるのは、`σ` の記録 1 つだけである。
  `leaf_origins_under(π)` は `π` の下の boxed leaf の記録を挙げ、`π` 自身が leaf のときはその leaf を
  挙げる。`p13-disposals-and-pending.md` の `L7` より `leaves(ty(u))` の相異なる 2 元は一方が他方の
  接頭辞にならないので、`σ` の真下に leaf は無い。
  BY p13-disposals-and-pending.md の L7,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under, Provenance::leaf_origins_at

<1>2. `σ` の記録は、空集合か `{Fresh}` か `{Unknown}` のいずれかである。
  `as_arg_projection` が `None` を返すのは、記録の要素数が 1 でないときと、唯一の要素が `Fresh` か
  `Unknown` であるときである。A3 より要素数 2 以上の宣言はこのプログラムに無い。
  BY A3, CODE src/rc_ir/ownership.rs: as_arg_projection

<1>3. CASE 記録が空集合である。
  `origin_from_leaves_under` の走査は `operand_units` に何も入れず `produced_here` を偽のままにするので
  `reached` は空であり、`first()?` が `None` を返す。`origin_inner` の `None` の腕は
  `unwrap_or_else(here)` で `Origin::Exactly((u, σ))` を返す。`origin` は呼ばれない。
  BY <1>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under, origin_inner

<1>4. CASE 記録が `{Fresh}` か `{Unknown}` である。
  走査は `produced_here` を真にし、`operand_units` は空のままなので `reached = [Exactly((u, σ))]` で
  ある。全元が等しいので `first.clone()` すなわち `Origin::Exactly((u, σ))` が返る。`origin` は
  `operand_units` の各元についてしか呼ばれないので、ここでも呼ばれない。
  BY <1>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under

<1>5. QED
  `<1>2` の 3 つが場合を尽くす。
  BY <1>2, <1>3, <1>4

### L6b (`route` の返り値)

**言明**。`route(x, callee, args, k)` が `callee` と異なる名前の `RcVar` を返すのは、
`self.borrow_versions` が `FuncRef { name: callee.name }` を鍵に持ち、かつ `self.routing_is_safe(x, args)` と
`self.routing_saves_retain(borrow_version, args, k)` がともに真であるときだけである。そのとき返るのは
`callee` の複製の `name` を `self.borrow_versions[&FuncRef { name: callee.name }].name` に替えたもので
あり、それ以外のときは `callee.clone()` である。

<1>1. `route` は `orig = FuncRef { name: callee.name.clone() }` をとり、
      `self.borrow_versions.get(&orig)` が `Some(borrow_version)` で、かつ
      `self.routing_is_safe(x, args) && self.routing_saves_retain(borrow_version, args, k)` が真のときに
      `callee` の複製の `name` を `borrow_version.name` に替えて返し、それ以外のときは `callee.clone()` を
      返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::route

<1>2. QED
  BY <1>1

**この補題を置く理由**。P11 の証明 (第 6 節) がこの事実を読む。命題は依存の順に並び、各命題は自分より前の
命題だけを引用してよい。P11 は P12 より前なので、P11 の証明は P12 を引けない。この補題は
`RewriteCtx::route` のコードだけに立つので、どちらの命題からも引ける。

### L6c (束縛を持たない名前の `origin`)

**言明**。`vars.bindings` が `x` を鍵に持たないとき、`origin(vars, type_env, x, π)` は任意の path `π` に
ついて panic せずに停止し、`Origin::Exactly((x, π))` を返す。

<1>1. `origin` は memo を引き、外れたときに `grow_stack(|| origin_inner(vars, type_env, var, path))` を
      呼んでその値を記録して返す。A15 より `grow_stack(f)` は `f` をちょうど 1 回呼び、その返り値を返す。
  BY A15, CODE src/rc_ir/ownership.rs: origin, CODE src/misc.rs: grow_stack

<1>2. `origin_inner` は `vars.bindings.get(var)` で場合分けし、`None` の腕で `here()` すなわち
      `Origin::Exactly((var, path))` を返す。この腕は `origin` を呼ばず、ほかの関数も呼ばない。
  BY CODE src/rc_ir/ownership.rs: origin_inner

<1>3. QED
  BY <1>1, <1>2

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

<1>1a. `units(I64) = []` であり、`units(gv.ty) = []` である。
  `<1>1` よりどちらの型も `is_fully_unboxed` が真であり、`unit_step` はその型に `UnitStep::NoUnit` を
  返す。`rc_units_go` の `NoUnit` の腕は `out` に何も積まないので、`rc_units` は空列を返す。
  BY D5, <1>1, CODE src/rc_ir/ownership.rs: unit_step, rc_units, rc_units_go

<1>2. `Q` は A2 を満たす。
  `Q` の `Retain`/`Release` 節点は `Release(y, [], s, ..)` の 1 つだけであり、`<1>0` より `[] ∈ units(A)` で
  ある。
  BY A2, <1>0

<1>3. `g` の本体は D11 の意味で RC 規律を満たす。
  <2>1. `g` の実行路は 1 本であり、節点の列は `Release(y, [])`、`Ret(n)` である。
    BY D2, D3
  <2>1a. `obj(y, [])` は、計数下 (D26) かグローバル状態 (D26) のどちらかであり、この活性化の間その
        区別は変わらない。
    BY D26
  <2>2. `obj(y, [])` が計数下であるとき、`Obl` の初期値は `obj(y, [])` への参照 1 つであり、グローバル
        状態であるときは空である。`n` は `<1>1` より leaf を持たない。
    D10 の初期値は、所有する (D14) unit の下の inhabited な各 leaf につき参照を 1 つ入れる。A1 より `g` の
    `borrowed_units` は空なので `g` は `y` の unit `[]` を所有し、`<1>0` よりその下の leaf は `[]` 1 つで
    ある。D26 より、グローバル状態のオブジェクトを指す leaf は D8 の意味の参照を持たないので、その場合
    `Obl` の初期値は空である。
    BY A1, D8, D10, D14, D16, D26, <1>0, <1>1, <2>1a
  <2>3. `Release(y, [], s, ..)` の後、`Obl` は空である。計数下のときは `<2>2` の参照 1 つが取り除かれ、
        グローバル状態のときは取り除かれる参照が無い (D26)。
    BY D10, D26, <1>0, <2>2
  <2>4. 終端の `Ret(n)` の消費は何も取り除かない。`<1>1` より `n` は boxed leaf を持たない。
    BY D9, <1>1
  <2>5. QED
    (S-a) は `<2>3` が、(S-b) は `<2>3` と `<2>4` が与える。この実行路に D7 の読む構文は無い。
    `Release(y, [])` が触れる `obj(y, [])` を見る。`<2>1` よりこの節点は実行路の最初の節点であり、
    `<1>0` より `[]` の下の inhabited な leaf は `[]` の 1 つだけなので、その触れる動作の直前の点は
    この活性化の開始の時点である。計数下のときは `<2>2` よりその時点で `Obl` がその参照を持つ。
    D21 は「活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る」と述べ、
    A19 (i) の本文は
    その不等式 `H(O) ≥ Σ_{C ∈ S} d(C) + [S に借用終端の類が在るならば 1]` の `Σ d(C)` を「その活性化の
    義務集合が持つ `O` への参照の個数である」と述べる。角括弧は 0 以上なので
    `H(obj(y, [])) ≥ Obl(obj(y, [])) ≥ 1` である。(S-c) の接頭条件よりこの活性化はその点まで解放に
    ついて閉じているので、D11a よりその点で解放されていない。グローバル状態のときは解放されることが
    無い (A8、D26)。
    BY A8, A19, D7, D11, D11a, D21, D26, <1>0, <2>1, <2>1a, <2>2, <2>3, <2>4

<1>3a. `f` の本体の `Let(w, App(gv, [x, m]), Ret(w))` の段の実行時の呼び出し先 (D23) は、それが `f` と
       `g` のどちらであっても、型 `A` の第 0 パラメータと型 `I64` の第 1 パラメータを持ち、その全
       パラメータ unit を D14 の意味で所有する。
  **呼び出し先がどちらであるかは要らない** -- この段の下流が読むのは、その `params` の型の列と、その
  位置の unit を所有するかどうかだけであり、`Q` の 2 つの関数はその 2 つについて一致する。
  <2>1. 実行時の呼び出し先は `Q` の `funcs` の関数、すなわち `f` か `g` である。
    D23 は「D9 の `App` の行と D10 の生成の `App` の行が「呼び出し先」と言うのは、この実行時の関数で
    ある」と述べ、続けて「D9 の `App` の行が読む所有は D14 が `RcFunc::borrowed_units` から定めるもの
    なので、**その呼び出し先はプログラムの `funcs` の関数である**」と述べる。`Q` の定義より `funcs` は
    `f` と `g` の 2 つである。
    BY D14, D23, Q の定義
  <2>2. `f` と `g` はどちらも、型 `A` の第 0 パラメータと型 `I64` の第 1 パラメータを持つ。
    `Q` の定義より `f.params = [x : A, m : I64]`、`g.params = [y : A, n : I64]` である。
    BY Q の定義
  <2>3. QED
    `Q` の定義より `f` と `g` の `borrowed_units` はどちらも空なので、D14 よりどちらも自分の全
    パラメータ unit を所有する。`<2>1` と `<2>2` と合わせて言明を得る。
    BY D14, Q の定義, <2>1, <2>2

<1>4. `f` の本体は D11 の意味で RC 規律を満たす。またその唯一の実行路で、`App(gv, [x, m])` は
      パラメータ leaf `(x, [])` の参照を A1 の割り当ての下で D9 の意味で消費する。
  <2>1. `f` の実行路は 1 本であり、節点の列は `Let(w, App(gv, [x, m]))`、`Ret(w)` である。
    BY D2, D3
  <2>1a. `obj(x, [])` は、計数下 (D26) かグローバル状態 (D26) のどちらかであり、この活性化の間その
        区別は変わらない。
    BY D26
  <2>2. `obj(x, [])` が計数下であるとき、`Obl` の初期値は `obj(x, [])` への参照 1 つであり、グローバル
        状態であるときは空である。
    D10 の初期値は所有する unit の下の inhabited な各 leaf につき参照を 1 つ入れ、A1 より `f` は `x` の
    unit `[]` を所有し、`<1>0` よりその unit の下の leaf は `[]` 1 つである。D26 よりグローバル状態の
    オブジェクトを指す leaf は D8 の意味の参照を持たないので、その場合の初期値は空である。
    BY A1, D8, D10, D14, D16, D26, <1>0, <1>1, <2>1a
  <2>3. `App(gv, [x, m])` は `x` の leaf `[]` を消費し、参照を作らない。`obj(x, [])` が計数下のときは
        その参照 1 つが `Obl` から取り除かれ、グローバル状態のときは取り除かれる参照が無い (D26)。
        `gv` と `m` は `<1>1` より leaf を持たず、`w` も leaf を持たない。D9 の `App` の行が読む
        呼び出し先は実行時の呼び出し先であり (D23)、`<1>3a` よりその第 0 パラメータの型は `A` で、
        その unit を所有する。`<1>0` より `A` の leaf は `[]` の 1 つで `units(A) = {[]}` なので、
        `x` の leaf `[]` が属する呼び出し先の unit は `[]` であり、D9 の `App` の行より
        `x` の leaf `[]` は消費される。
    BY D9, D10, D14, D23, D26, <1>0, <1>1, <1>3a, <2>1a
  <2>4. 終端の `Ret(w)` の消費は何も取り除かない。
    BY D9, <1>1
  <2>5. QED
    `<2>2` と `<2>3` より `App` の直後の `Obl` は空である。(S-a) は `<2>2` と `<2>3` が、(S-b) は
    `<2>3` と `<2>4` が与える。(S-c) は、この実行路の読む構文が `App` の 1 つだけで、それが読みうる
    オブジェクトが `obj(x, [])` であることによる -- 計数下のときは、その読みの直前の点である `App` の
    節点の入口 (`<2>1` よりこの節点は実行路の最初の節点なので、この活性化の開始の時点でもある) で
    `<2>2` より `Obl` がその参照を持つ。D21 は「活性化は、その各時点と各段内の点 (D24) で A19 (i) の
    不等式を満たすものに限る」と述べ、A19 (i) の本文はその不等式の `Σ d(C)` を「その活性化の義務集合が
    持つ `O` への参照の個数である」と述べる。角括弧は 0 以上なので
    `H(obj(x, [])) ≥ Obl(obj(x, [])) ≥ 1` である。(S-c) の
    接頭条件よりこの活性化はその点まで解放について閉じているので、D11a よりその点で解放されていない。
    グローバル状態のときは解放されることが無い (A8、D26)。
    BY A8, A19, D7, D11, D11a, D21, D26, <2>1, <2>1a, <2>2, <2>3, <2>4

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
        `m` の unit を挙げる。`m` は `<1>1a` より unit を持たない。
        `(x, [])` については、`origin(x, [])` が `Binding::Param` の腕で `Exactly((x, []))` を返すので
        候補は `(x, [])` 1 つであり、`owns_object_yet(vars, type_env, x, [], ∅)` は `under(A, [])` の
        唯一の元 `[]` について「`trunc(A, ・) = []` を満たし `∅` に入る leaf」を求めて偽になる。よって
        `owns_a_candidate` は偽で `level_ownership` は `false` を返す。
    BY <1>0, <1>1, <1>1a, CODE src/rc_ir/borrow.rs: levelled_sites, level_ownership, owns_object_yet,
       CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. QED
    初期値は空であり、`<2>1`-`<2>3` より 1 周目で `changed` は偽のままである。`insert` が 1 度も真を
    返さないので `owned_leaves` は空のまま返る。
    BY EXT 集合と写像, <2>1, <2>2, <2>3, CODE src/rc_ir/borrow.rs: infer_ownership

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
  BY EXT 集合と写像, CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

<1>2. `changed` が真になるのは、`<1>1` のどちらかの `insert` が真を返したときだけである。消費の段は
      `insert` の返り値で `changed` を立て、平準化の段は `changed |= level_ownership(..)` であり、
      `level_ownership` は `owns_a_candidate` が偽なら `false` を返し、真のときは `insert` の返り値の
      論理和を返す。ループは `changed` が偽のとき `break` する。
  BY EXT 集合と写像, CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

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
  <2>3. 1 つの `(var, path)` に対する `candidates()` は周回によらず同じ有限集合である。
        `Origin::Exactly` は 1 元、`Origin::Join` は有限集合の `candidates` を持つ。
    `var_tables` はループの外で 1 度だけ作られるので、1 つの関数について `origin` に渡る第 1 引数
    (`VarTable` の値) と第 2 引数 (`type_env`) は全周回で同じである。**それだけでは答えが同じことは
    出ない** -- `origin` は答えを `vars.origins` の `RefCell<Map<VarPath, Origin>>` に記録し、次の
    呼び出しはそれを先に読んで返すので、答えは memo の状態に依りうる。P2a が「**1 つの `VarTable` の値
    `vars` と 1 つの `TypeEnv` の値を固定する。** その 2 つを第 1・第 2 引数とし、鍵 `(x, π)` が等しい
    2 つの `origin` の呼び出しがどちらも値を返すならば、その 2 つの返り値は等しい」と述べ、これを
    閉じる。**当てる表はその制限を満たす。** P2a は「**`vars` は、A6 と A11 を満たすプログラムの本体に
    ついて `VarTable::of` か `VarTable::body_only` が作った表である。**」を言明の一部として持つ。
    `var_tables` は `prog.funcs` の各 `f` について `VarTable::of(f)` を作った表であり、`prog` は
    `borrow_ify` の入力プログラムなので、A6 と A11 がそれについて成り立つ。
    `origin(var, path)` が答えを返すのは、
    `var` がプログラムの束縛変数であるとき P2 が、`vars.bindings` が `var` を鍵に持たないとき L6c が
    与える。`consumed` の対の第 1 成分にはその 2 つ目が現れる -- D6 の第 3 の形、すなわち
    `App` の callee や `Llvm` のオペランドとして現れるグローバル値の名前である。
    BY A6, A11, D6, L6c, P2, P2a, CODE src/rc_ir/borrow.rs: infer_ownership,
       CODE src/rc_ir/ownership.rs: origin, Origin, Origin::candidates, VarTable
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

<1>4a. `collect_consumes` の 1 回の呼び出しの再帰は有限回で終わり、その中で走る `boxed_leaf_paths` と
      `result_prov` はどちらも abort せず値を返し、`rhs_consumes` の `App` の腕が引く `params[i]` は
      範囲内である。
  <2>1. `collect_consumes_go` の再帰は本体の木の上の走査であり、各節点をちょうど 1 度訪れる。
        `RcExpr::Let(x, rhs, k)` の腕は `rhs` が `Match(_, arms)` のとき各 `arm.body` へ降りてから `k` へ
        降り、`Destructure`・`Retain`・`Release`・`Eval` の腕は `k` へ降り、`Ret` の腕は降りない。
        D2 より本体は有限の木なので、走査は有限回で終わる。
    BY D2, CODE src/rc_ir/ownership.rs: collect_consumes, collect_consumes_go,
       CODE src/rc_ir/ast.rs: RcExpr
  <2>2. 1 つの節点で走る `push_boxed_leaves`・`destructure_consumes` と、`rhs_consumes` の
        `Closure`・`App`・`Llvm` の腕は `boxed_leaf_paths` を呼ぶ。A10 より `boxed_leaf_paths` は
        有限の列を返して停止する。
    BY A10, CODE src/rc_ir/ownership.rs: rhs_consumes, push_boxed_leaves, destructure_consumes,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `rhs_consumes` の `Llvm` の腕は `passthrough_arg_leaves` を呼び、`passthrough_arg_leaves` は
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を呼んでその宣言の leaf を走る。A3 より
        `result_prov` の呼び出しは abort せず `Provenance` を返す。同じ腕が読む `borrows_operand(i, ..)`
        の真偽値も、A3 がその演算の宣言として読むものである。
    BY A3, CODE src/rc_ir/ownership.rs: rhs_consumes, passthrough_arg_leaves,
       CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, LLVMGen::borrows_operand
  <2>4. `rhs_consumes` の `App` の腕が `resolve_callee_params` から `Some(params)` を受け取るとき、
        `params[i]` は範囲内である。A14 は `App(callee, args)` の `args` の個数を、
        `resolve_callee_params` が静的に引く関数のパラメータの個数に等しいとし、`i` は `args` の
        添字である。
    BY A14, CODE src/rc_ir/ownership.rs: rhs_consumes, resolve_callee_params
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4

<1>5. 1 周回の仕事は有限である。
  `var_tables` と `sites` はループの外で 1 度だけ作られる。`levelled_sites` は本体の節点を
  `for_each_node` で 1 度ずつ歩き、有限の列を返す (D2 より本体は有限の木)。各周回は、各関数について
  `collect_consumes` を 1 回 (`<1>4a` よりその呼び出しは有限回の再帰で終わり、`<1>3` より、積む対の
  全体は `own` によらない有限集合に含まれる)、その
  各元について `origin` を 1 回、各 site について `level_ownership` を 1 回呼ぶ。`level_ownership` は
  `origin` を 1 回呼び、各候補について `owns_object_yet` と `covered_leaves` を呼ぶ。`origin` の停止は、
  第 1 成分がプログラムの束縛変数であるとき P2 が、`vars.bindings` が鍵に持たない名前 (D6 の第 3 の形)
  であるとき L6c が与える。`owns_object_yet` は `boxed_leaf_paths` を 1 回、`under` を 1 回、
  `truncate_to_unit` を 2 か所 -- `under` が返す各 unit の鍵と、`boxed_leaf_paths` が返す各 leaf の鍵 --
  で呼ぶ。A10 より、`boxed_leaf_paths` と `rc_units` の歩みも、`subtree_type` と `truncate_to_unit` が
  取るフィールドの型の歩みも有限であり、`truncate_to_unit` の繰り返しは path の長さで抑えられる。
  **この 3 つは中断しうる。** `truncate_to_unit` は `unit_step` が `NoUnit` を返す位置で `panic!` し、
  `Capture` を返す位置で path の添字が capture の添字でなければ `assert_eq!` で止まる
  (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。`under` は `subtree_type` を通り、`unit_step` が
  `Fields` を返す位置で `held_field_type` を呼ぶので、その添字がその値の持つフィールドを名指さなければ
  `panic!` する (`CODE src/rc_ir/ownership.rs: units_under`, `subtree_type`, `held_field_type`)。
  **中断も停止である** -- どの場合も歩みはそこで終わり、周回が終わらないことは無い。中断する
  `(root, path)` を持つ入力では `infer_ownership` は値を返さず、`borrow_ify` も出力を返さないので、
  P8 (b) と P8 (c) はその入力について空虚に真である。L8 の `<1>3` が `owns_object` について切り出す
  のと同じ面である。`collect_consumes` が積む対の全体が有限であることは `<1>3` が与える。
  BY A10, A15, D2, D6, L6c, P2, <1>3, <1>4a, CODE src/rc_ir/borrow.rs: infer_ownership, levelled_sites,
     level_ownership, owns_object_yet, covered_leaves, CODE src/rc_ir/ast.rs: for_each_node,
     CODE src/rc_ir/ownership.rs: truncate_to_unit, units_under, subtree_type, held_field_type,
     unit_step, rc_units, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/misc.rs: grow_stack

<1>6. QED
  `<1>1` と `<1>2` より、`changed` が真になる周回では `owned_leaves` は真に大きくなる。`<1>3` と `<1>4`
  よりその大きさには上界があるので、`changed` が真である周回は有限回しかなく、その次の 1 周で `changed` は
  偽になり `break` する。各周回は `<1>5` より有限の仕事しかしない。
  BY EXT 集合と写像, <1>1, <1>2, <1>3, <1>4, <1>5

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
  BY EXT 集合と写像, <1>1, CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

<1>3. その周回で各関数について `collect_consumes` に渡される `own` は、返される `OL` と同じである。
  BY <1>2

<1>4. QED
  その周回の内側のループは、`vars.param_tys` が `root_var` を持つ各 `(root_var, root_path)` について
  `owned_leaves.insert` を呼ぶ。`<1>1` よりそのすべてが偽を返すので、すべてすでに入っている。`<1>3` より
  その `collect_consumes` の入力は `OL` である。
  **言明が名指す `origin(var, path).candidates()` は、その周回の呼び出しが返したものである。**
  `var_tables` は `infer_ownership` のループの外で 1 度だけ作られるので、1 つの関数について `origin` に
  渡る第 1 引数 (`VarTable` の値) と第 2 引数 (`type_env`) はどの周回でも同じである。P2a が
  「**1 つの `VarTable` の値 `vars` と 1 つの `TypeEnv` の値を固定する。** その 2 つを第 1・第 2 引数とし、
  鍵 `(x, π)` が等しい 2 つの `origin` の呼び出しがどちらも値を返すならば、その 2 つの返り値は等しい」と
  述べるので、2 つは同じ集合である。**当てる表はその制限を満たす。** P2a は「**`vars` は、A6 と A11 を
  満たすプログラムの本体について `VarTable::of` か `VarTable::body_only` が作った表である。**」を言明の
  一部として持つ。`var_tables` は `prog.funcs` の各 `f` について `VarTable::of(f)` を作った表であり、
  `prog` は `borrow_ify` の入力プログラムなので、A6 と A11 がそれについて成り立つ。
  `origin(var, path)` が値を返すのは、`var` がプログラムの束縛変数で
  あるとき P2 が、`vars.bindings` が `var` を鍵に持たないとき (D6 の第 3 の形) L6c が与える。
  BY A6, A11, D6, EXT 集合と写像, L6c, P2, P2a, <1>1, <1>3, CODE src/rc_ir/borrow.rs: infer_ownership,
     CODE src/rc_ir/ownership.rs: origin, VarTable

### 3.5 P8 (c) -- D9 の消費との対応

**DEF 所有を読まない消費** -- D9 の消費の表のうち、`App` の行の「呼び出し先がその位置の unit を
所有する (D14) 引数の leaf」を除いた部分をいう。すなわち `App` の callee の全 boxed leaf、`Closure` の各 capture の全
boxed leaf、`Llvm` の消費する leaf、boxed 容器の `Destructure`、unbox 容器の `Destructure`、関数本体の
終端の `Ret` の 6 つである。D9 の消費の表でこの割り当てを読むのは `App` の引数の位置だけである。

**言明**。入力の各関数 `f` のどの実行路のどの位置についても、そこで所有を読まない消費によって消費される
leaf `(v, λ)` について、`origin(v, λ).candidates()` の元のうち `vars.param_tys` に載るものはすべて `OL` に
入っている。

<1>1. 所有を読まない消費が消費する leaf は、`own` の値によらず `collect_consumes` が `out` に積む。
  <2>0. `own_全` を、入力の各関数の各パラメータ・capture `p` と各 `λ ∈ leaves(ty(p))` について
        `(p.name, λ)` を持つ集合とする。`own_全` は D14 の所有をちょうど報告する。
    A1 より入力のすべての関数の `borrowed_units` は空なので、D14 よりどの関数もすべてのパラメータ・
    capture の unit を所有する。`collect_consumes` が `own` を読むのは
    `owns(p, leaf) = own.contains(&(p.name, leaf))` を通じてだけであり、`rhs_consumes` の `App` の腕は
    それを呼び出し先のパラメータ `params[i]` と引数の leaf について呼ぶ。A12 より引数と呼び出し先の
    対応するパラメータの型は等しいので、その leaf は `leaves(ty(params[i]))` の元であり、`own_全` は
    その問いに真を返す。
    BY A1, A12, D14, CODE src/rc_ir/ownership.rs: collect_consumes, rhs_consumes
  <2>1. `own = own_全` を渡した `collect_consumes` の呼び出しは、D9 の意味で消費する構文をすべて
        報告する。
    P7 の前半である。P7 はその前半を「**D14 の所有をちょうど報告する `own` を渡した** `collect_consumes` の
    呼び出し」に限っており、`<2>0` がそれを満たす。
    BY P7, <2>0
  <2>1a. DEF 所有を読まない消費 が挙げる 6 行が消費する leaf は、所有と借用の割り当てによらない。
    その 6 行 -- `App` の callee の全 boxed leaf、`Closure` の各 capture の全 boxed leaf、`Llvm` の
    消費する leaf、boxed 容器の `Destructure`、unbox 容器の `Destructure`、関数本体の終端の `Ret` --
    はどれも節点の形と変数の型だけで決まり、D14 の割り当てを読まない。
    BY D9, DEF 所有を読まない消費
  <2>2. これを報告する 5 か所 -- `collect_consumes_go` の `RcExpr::Ret` の腕と `RcExpr::Destructure` の腕、
        `rhs_consumes` の `Closure` の腕、`Llvm` の腕、`App` の腕の
        `push_boxed_leaves(&callee.name, ..)` -- は、いずれも `owns` を読まない。`Llvm` の腕が読むのは
        `borrows_operand` と `passthrough_arg_leaves` だけである。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes
  <2>3. QED
    `<2>1` と `<2>1a` より、所有を読まない消費が消費する leaf は `own = own_全` を渡した呼び出しが
    `out` に積む。`<2>2` よりそれを積む 5 か所は `owns` を読まず、`collect_consumes_go` の走査も
    `own` で分岐しないので、どの `own` についても同じ元が積まれる。
    BY <2>1, <2>1a, <2>2, CODE src/rc_ir/ownership.rs: collect_consumes_go

<1>2. QED
  `<1>1` の積まれた元に P8 (b) の閉包条件を適用する。
  BY P8 (b), <1>1

**アーム本体の `Ret` は過剰報告である**。D9 はアーム本体の `Ret` を消費とせず移動とするので、次の
補題が挙げる元は D9 の意味の消費ではない (D9 の `collect_consumes` についての注)。それでも P8 (b) の
閉包条件は積まれた元の全体に掛かるので、その元についても同じ結論が出る。第 9.4 節の `L11` がこれを使う。

#### L7a (アーム本体の `Ret` が名指す leaf も `collect_consumes` が積む)

**言明**。入力の関数 `f` の `Match` のアーム本体の `Ret(x)` について、`x` の各 boxed leaf `λ` の対
`(x.name, λ)` は、`own` の値によらず `collect_consumes(&f.body, vars, prog, own, type_env, &mut consumed)`
が `consumed` に積む元である。

<1>1. 走査はアーム本体の節点を訪れる。
  `collect_consumes_go` の `RcExpr::Let(x, rhs, k)` の腕は、`rhs` が `RcRhs::Match(_, arms)` のとき
  各 `arm` について `collect_consumes_go(&arm.body, ..)` を呼ぶ。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go

<1>2. `RcExpr::Ret(x)` の腕は `push_boxed_leaves(&x.name, &x.ty, type_env, out)` を呼び、
      `push_boxed_leaves` は `boxed_leaf_paths(ty(x), type_env)` の各元 `λ` について `(x.name, λ)` を
      `out` に積む。この腕は `owns` を読まない。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go, push_boxed_leaves,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. QED
  `<1>1` よりアーム本体の `Ret` の節点は走査が訪れる節点であり、`<1>2` がその節点で積まれる元と、それが
  `own` に依らないことを与える。`collect_consumes` は `collect_consumes_go` を本体の根から 1 回呼ぶ。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: collect_consumes

### 3.6 `App` の引数の行について

D9 の `App` の引数の行は「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」であり、
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
`cand_f(v, λ)` の各元 `(r, p)` について `ctx.owns_object(ρ_f(r), p)` は真である。

<1>0. `cand_f(v, λ)` の各元 `(r, p)` について、`r` は `func` に現れる名前である。
  `v` は `func.body` に現れる `RcVar` の名前か `func` のパラメータ・capture の名前であり、どちらも
  `func` に現れる名前である (第 1 節に写した `p15` の第 1 節の (N) の集合)。第 1 節の `p15` の `L12` より
  `cand_f(v, λ) ⊆ Reach(v, λ)` であり、`p15` の `L14a` より `Reach(v, λ)` の各元の変数も `func` に
  現れる名前である。
  BY p15 の L12, p15 の L14a

<1>1. `cand_f(v, λ)` の元 `(r, p)` で `vars_f.param_tys` が `r` を鍵に持たないものについて、
      `ctx.owns_object(ρ_f(r), p)` は真である。
  <2>1. `r` は `func` に現れる名前であり、`vars_f.param_tys` の鍵ではない。
    前者は `<1>0` が、後者はこの場合の仮定が与える。
    BY <1>0
  <2>2. `ctx.vars` は `VarTable::of(clone)` である。
    `f_borrow` の `RewriteCtx` は `RewriteCtx::new(&clone, true, ..)` が作る。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, borrow_ify
  <2>3. `ρ_f(r)` は `ctx.vars.param_tys` の鍵でない。
    第 1 節の `p15` の `L15` の (i) より、`VarTable::of(clone)` の `param_tys` の鍵は
    `vars_f.param_tys` の鍵の `ρ_f` による像ちょうどであり、`ρ_f` は `func` に現れる名前の上で単射で
    ある。`vars_f.param_tys` の鍵は `func` のパラメータ・capture の名前なのでどれも `func` に現れる
    名前であり、`<2>1` より `r` もそうである。`<2>1` より `r` はその鍵のどれでもないので、単射性より
    `ρ_f(r)` はそれらの像のどれとも異なる。
    BY p15 の L15, <2>1, <2>2
  <2>4. QED
    L4 より、`param_tys` が鍵に持たない名前について `owns_object` は真を返す。
    BY L4, <2>2, <2>3

<1>2. `cand_f(v, λ)` の元 `(r, p)` で `vars_f.param_tys` が `r` を鍵に持つもの (その型を `τ`) について、
      `(r, p) ∈ OL` である。
  所有を読まない消費については P8 (c) が、アーム本体の `Ret` については L7a と P8 (b) が与える。
  BY L7a, P8 (b), P8 (c)

<1>3. `<1>2` の `p` は `leaves(τ)` の元である。
  次の言明を、`origin` の再帰についての帰納で示す。**`ρ' ∈ leaves(ty(y))` であるとき、`act_f(y, ρ')` の
  各元 `(z, σ)` について `σ ∈ leaves(ty(z))` である。** `origin(y, ρ')` が停止するので再帰の木は有限で
  あり、帰納が回る -- `y` がプログラムの束縛変数であるときは P2 が、`vars_f.bindings` が `y` を鍵に
  持たないとき (D6 の第 3 の形) は L6c が、それを与える。`λ ∈ leaves(ty(v))` なのでこれを `(v, λ)` に
  当てると、`cand_f(v, λ) ⊆ act_f(v, λ)` より言明が出る。
  <2>1. CASE `vars_f.bindings.get(y)` が `None`、`Some(Binding::Param)`、`Some(Binding::Producer)`、
        boxed 容器の `Some(Binding::Field(..))`、boxed scrutinee の `Some(Binding::Payload(_, Some(_)))`
        のいずれかである。
    これらの腕は `here()` すなわち `Origin::Exactly((y, ρ'))` を返すので `act_f(y, ρ') = {(y, ρ')}` で
    あり、`ρ' ∈ leaves(ty(y))` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, Origin::acted_on, Origin::candidates
  <2>2. CASE `Some(Binding::Llvm(..))` で `decl.leaf_origins_at(ρ').and_then(as_arg_projection)` が
        `None` である。
    L6a よりこの腕は `Origin::Exactly((y, ρ'))` を返す。
    BY L6a, CODE src/rc_ir/ownership.rs: Origin::acted_on
  <2>3. CASE `Some(Binding::Move(z))` または catch-all の `Some(Binding::Payload(z, None))`。
    どちらの腕も `origin(z, ρ')` をそのまま返す。A12 より move-bind の両辺の型は等しく、catch-all
    アームの payload と scrutinee の型も等しいので `ty(z) = ty(y)` であり、`ρ' ∈ leaves(ty(z))` で
    ある。帰納法の仮定による。
    BY A12, 帰納法の仮定, CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. CASE unbox 容器の `Some(Binding::Field(c, idx))` または unbox scrutinee の
        `Some(Binding::Payload(sc, Some(t)))`。
    どちらの腕も添字を 1 つ前に足した path で `origin(c, [idx] ++ ρ')` (resp. `origin(sc, [t] ++ ρ')`) を
    返す。A12 より `ty(y)` は `ty(c)` の第 `idx` フィールドの型 (resp. `ty(sc)` の第 `t` 変位の payload の
    型) であり、D4 の第 5 の規則より unbox の集約と union はフィールド・変位の下へ降りて leaf を挙げるので、
    足した path はその型の leaf である。帰納法の仮定による。
    BY A12, D4, 帰納法の仮定, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>5. CASE `Some(Binding::Llvm(gen, args, _))` で `decl.leaf_origins_at(ρ')` が単一の `Arg(j, σ)` で
        ある。
    この腕は `origin(args[j], σ)` を返す。A3 の「**単一の `Arg(j, σ)` の宣言は well-formed である。**
    `j` は `args` の添字であり、`σ` はその型の boxed leaf である」より、`args[j]` は存在し
    `σ ∈ leaves(ty(args[j]))` である。帰納法の仮定による。
    BY A3, 帰納法の仮定, CODE src/rc_ir/ownership.rs: origin_inner, as_arg_projection
  <2>6. CASE `Some(Binding::Join(arm_results))`。
    この腕は各 `a ∈ arm_results` について `act_f(a, ρ')` を集めた集合 `S` を作り、
    `Origin::of_candidates(S, (y, ρ'))` を返す。A12 よりアームの結果と `Match` の束縛変数の型は等しいので
    `ρ' ∈ leaves(ty(a))` であり、帰納法の仮定より `S` の各元の path はその変数の型の leaf である。
    `of_candidates` は `|S| = 1` のとき `Exactly(p)` を返し (`p` は `S` の唯一の元)、`|S| ≥ 2` のとき
    `Join { identity: (y, ρ'), candidates: S }` を返す。`Origin::acted_on` は `identity()` を先頭に
    それと異なる `candidates()` の元を続けるので、前者では `act_f(y, ρ') = S`、後者では
    `act_f(y, ρ') = {(y, ρ')} ∪ S` である。どちらの場合も `act_f(y, ρ') ⊆ S ∪ {(y, ρ')}` であり、
    `ρ' ∈ leaves(ty(y))` なので `(y, ρ')` の path も leaf である。
    BY A12, 帰納法の仮定, CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates,
       Origin::acted_on
  <2>7. QED
    `Binding` は `Param`、`Producer`、`Move`、`Field`、`Payload`、`Llvm`、`Join` の 7 種であり、
    `bindings.get` はそれに `None` を加える。`Field` と `Payload` を容器・scrutinee が boxed か
    unbox かで、`Payload` をさらに catch-all かどうかで、`Llvm` を `as_arg_projection` の答えで
    分けた `<2>1`-`<2>6` がこれを尽くす。
    BY L6c, P2, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, CODE src/rc_ir/ownership.rs: Binding, origin_inner

<1>4. `covered_leaves(τ, p) = {p}` である。
  `<1>3` より `p ∈ leaves(τ)` である。`p13-disposals-and-pending.md` の `L7` より `leaves(τ)` の相異なる
  2 元は一方が他方の接頭辞にならないので、`λ' ⊑ p` または `p ⊑ λ'` を満たす `λ' ∈ leaves(τ)` は `p` だけ
  である。
  BY <1>3, p13-disposals-and-pending.md の L7, CODE src/rc_ir/borrow.rs: covered_leaves

<1>5. `owns_object_yet(vars_f, type_env, r, p, OL)` は真である。
  第 1 節に写した `p15` の `L11` は、「`r` を `vars.param_tys` が型 `τ` で持つ名前、`p` を path、`OL` を
  `VarPath` の集合とする。`covered_leaves(τ, p) ⊆ { λ : (r, λ) ∈ OL }` であり、かつ
  `covered_leaves(τ, p) ≠ ∅` または `under(τ, p) = []` であるとき、
  `owns_object_yet(vars, type_env, r, p, OL)` は真である」と述べる。
  `<1>2` の `r` は `vars_f.param_tys` が型 `τ` で持つ名前である。`<1>4` より
  `covered_leaves(τ, p) = {p}` であり、これは空でない。`<1>2` より `(r, p) ∈ OL` なので包含も
  成り立つ。
  BY <1>2, <1>4, p15 の L11

<1>6. QED
  第 1 節に写した `p15` の `L16` は、「`func` に現れる任意の名前 `r` と任意の path `p` について
  `ctx.owns_object(ρ_f(r), p) = owns_object_yet(vars_f, type_env, r, p, OL)` である」と述べる。`<1>0` より
  `cand_f(v, λ)` の各元の変数は `func` に現れる名前なので、これが当たる。`vars_f.param_tys` が `r` を鍵に
  持たない候補は `<1>1` が、持つ候補は `<1>5` と この等式が与える。
  BY p15 の L16, <1>0, <1>1, <1>5

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
    BY EXT 反復子の並び, CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name
  <2>2. `assign_fresh_names_to_binders_inner` は、`RcExpr::Let(x, rhs, k)` の `x`、`rhs` が
        `RcRhs::Match(_, arms)` のときの各 `arm.payload`、`RcExpr::Destructure(_, fields, ..)` の各
        フィールド変数について `assign_fresh_name` を呼び、`k` と各 `arm.body` へ降りる。
        `RcExpr::Retain | Release | Eval` は継続へ降り、`RcExpr::Ret` は降りない。
    BY CODE src/rc_ir/rename.rs: assign_fresh_names_to_binders_inner
  <2>3. QED
    `<2>2` の 3 種は DEF 入力の束縛名 が数える束縛子の全体である。
    BY DEF 入力の束縛名, <2>1, <2>2, CODE src/rc_ir/ast.rs: RcExpr, MatchArm

<1>3a. `assign_fresh_name` が呼ばれるのは、`func` のパラメータ・capture の各元につき 1 度と、
       `func.body` の各束縛子につき 1 度である。
  `fresh_rename_function` は `params.iter().chain(cap.iter())` の各元について 1 度呼ぶ。
  `assign_fresh_names_to_binders_inner` は `RcExpr` の 6 種を走査し、`Let(x, rhs, k)` で `x` に 1 度、
  `rhs` が `Match(_, arms)` のとき各 `arm.payload` に 1 度、`Destructure` で各フィールド変数に 1 度
  呼び、`k` と各 `arm.body` へ 1 度ずつ降りる。`Ret` では降りない。よって走査は本体の各節点をちょうど
  1 度訪れ、各束縛子についてちょうど 1 度呼ぶ。
  BY A15, EXT 反復子の並び, CODE src/rc_ir/rename.rs: fresh_rename_function,
     assign_fresh_names_to_binders, assign_fresh_names_to_binders_inner, CODE src/misc.rs: grow_stack

<1>4. `renaming` は写像であり、定義域の各名前に像を 1 つだけ持つ。
  `<1>3a` より `assign_fresh_name` の呼び出しの列は、`func` のパラメータ・capture の名前と `func.body` の
  各束縛子の名前をちょうど 1 度ずつ並べたものである。A6 より入力のすべての束縛名は互いに相異なるので、
  この列に同じ名前は 2 度現れない。`renaming` に元を入れるのは `assign_fresh_name` の `renaming.insert`
  だけである (`<1>3`)。よって 1 つの名前についての `insert` は 1 度きりであり、上書きは起きない。
  BY A6, EXT 集合と写像, <1>3, <1>3a, CODE src/rc_ir/rename.rs: assign_fresh_name

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
  BY EXT 10 進表記, <1>4, CODE src/rc_ir/rename.rs: assign_fresh_name

<1>8. QED
  `<1>5` と `<1>6` より、写された本体は元の本体と同じ形の木であり、変わるのは `renaming` の定義域にある
  名前の出現だけである。`<1>3` よりその定義域は束縛名の全体であり、`<1>4` より写像、`<1>7` より単射で
  ある。A6 と A11 より、定義域にある名前の出現はすべて、その名前を束縛する `func` の束縛子に解決する
  出現である。よって置き換えは一斉の名前替えである。
  BY A6, A11, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7

### 4.2 後半 -- 導入する名前は入力の束縛名と異なる

**言明**。A13 の下で、`borrow_ify` の実行中に `clone_func` が導入する名前は、どの入力の束縛名とも異なる。
導入される名前は `M ++ "#b" ++ dec(c)` の形であり、`M` は元の束縛名の `name` フィールド、`c` は 1 以上の
整数で、1 回の `borrow_ify` の実行の中で 2 度使われることはない。**その `name` フィールドを `#` で
区切った最後の断片は、`b` の後に 10 進数字が 1 個以上続く形である。**

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
  BY EXT 10 進表記, <1>1

<1>3. 入力の束縛名の `name` フィールドを `#` で区切った最後の断片は、`<1>2` の形ではない。
  BY A13

<1>4. `c` は 1 回の `borrow_ify` の実行の中で 2 度使われない。
  `rename_counter` は `borrow_ify` の中で 1 つだけ作られ、`clones` を作るループを通じて `clone_func` に
  渡され、`assign_fresh_name` の呼び出しごとに 1 増える。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, clone_func, CODE src/rc_ir/rename.rs: assign_fresh_name

<1>5. QED
  `FullName` は `namespace` と `name` の 2 つのフィールドを持ち `PartialEq` を derive するので、
  EXT 導出した相等 より、`name` フィールドが異なれば名前は異なる。形についての後半は `<1>1` と `<1>4` に
  よる。
  BY EXT 導出した相等, <1>1, <1>2, <1>3, <1>4, CODE src/ast/name.rs: FullName

**この言明を検査するコード**。`develop_mode` のとき、`borrow_ify` は `check_clone_names_are_fresh(prog,
clones.iter().map(|(_, _, rename)| rename))` を呼ぶ。これは入力プログラムのパラメータ・capture と
`for_each_var` が訪れる全変数の名前を集め、各 `rename` の像がそのどれでもないことを `assert!` する
(`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`, `borrow_ify`)。A13 はこの関数を「検査」の
欄に名指している。

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

### 4.4 系 -- 出力の束縛名は `funcs` の鍵ではない

**言明**。A13 の下で、`borrow_ify` の出力の束縛名 (DEF 出力の束縛名) は、入力の束縛名か `clone_func` が
導入した名前のどちらかであり、入力の `funcs` の鍵でも出力の `funcs` の鍵でもない。

<1>1. 出力の束縛名は、入力の束縛名か、`clone_func` が導入した名前のどちらかである。
  `f_own` は `func.clone()` の `body` を `ctx.rewrite` の値に差し替えたものであり、`params` と `capture` は
  `func` のままである。グローバル初期化子も `init` を差し替えるだけである。L5 より `rewrite` は本体の
  束縛名を変えないので、この 2 種の束縛名は入力の束縛名である。`f_borrow` の束縛名は、4.1 の言明より
  `rename_f` の像 -- すなわち `clone_func` が導入した名前 -- であり、その後の `ctx.rewrite` も L5 より
  束縛名を変えない。
  BY L5, 4.1 の言明, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. 入力の束縛名は入力の `funcs` の鍵ではない。
  DEF 入力の束縛名 の名前は、入力の関数のパラメータ・capture の名前か、入力のいずれかの本体が束縛する
  変数の名前である。D6 は「`VarTable::of` と `VarTable::body_only` がその表に入れる鍵は、パラメータ・
  capture の名前と節点が束縛する変数の名前だけで、どれも `Lowerer::fresh_var` が `FullName::local` で
  作ったものである」と述べるので、どちらも局所名である。A13 は「**最上位の記号の名前は局所名ではない。**
  `FullName::is_local` が偽であり、`prog.funcs` の鍵と `global_types` の鍵はどちらもそのような名前で
  ある」と述べる。A6 も同じ結論を別に与える -- 入力のすべての束縛変数の名前はどの関数の名前とも異なる。
  BY A6, A13, D6

<1>3. 入力の束縛名は `borrow_versions` の値ではない。
  `borrow_funcref` は借用版の名前を `<元の名前>#borrow` として作るので、その `name` フィールドを `#` で
  区切った最後の断片は `borrow` である。A13 は入力に現れるすべての名前について、その断片が `borrow` で
  ないと述べる。入力の束縛名は入力に現れる名前である。
  BY A13, CODE src/rc_ir/borrow.rs: borrow_funcref

<1>4. `clone_func` が導入した名前は入力の `funcs` の鍵ではない。
  4.2 の言明よりその名前の最後の断片は `b` の後に 10 進数字が 1 個以上続く形であり、A13 は入力に現れる
  すべての名前 -- `prog.funcs` の鍵を含む -- についてその断片がその形でないと述べる。
  BY A13, 4.2 の言明

<1>5. `clone_func` が導入した名前は `borrow_versions` の値ではない。
  `<1>3` より借用版の名前の最後の断片は `borrow` であり、4.2 の言明より複製の名前の最後の断片は `b` の
  後に 10 進数字が 1 個以上続く形である。`borrow` の 2 文字目は 10 進数字ではないので、2 つは異なる。
  `FullName` は `namespace` と `name` の 2 つのフィールドを持ち `PartialEq` を derive するので、
  EXT 導出した相等 より、`name` フィールドが異なれば名前は異なる。
  BY EXT 導出した相等, 4.2 の言明, <1>3, CODE src/ast/name.rs: FullName

<1>6. QED
  L6 より出力の `funcs` の鍵の集合は「入力の各関数の名前」と「`borrow_versions` の各値」の合併であり、
  A22 より前者は入力の `funcs` の鍵の集合である。`<1>1` の 2 種のそれぞれについて、入力の `funcs` の鍵で
  ないことを `<1>2` と `<1>4` が、`borrow_versions` の値でないことを `<1>3` と `<1>5` が与える。
  BY A22, L6, <1>1, <1>2, <1>3, <1>4, <1>5

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
  BY EXT 反復子の並び, <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>5. `kept.into_iter().rev().fold(k, |cont, unit| rc_node(is_release, v.clone(), unit, state, cont, source))`
      は、`u_r` を最も内側に、`u_1` を最も外側に置いた節点の鎖を返す。`r = 0` のときは `k` を返す。
  `rev()` により fold は `u_r` から始まり、各段が直前の結果を継続として包む。
  BY EXT 反復子の並び, <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

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
  BY EXT 反復子の並び, CODE src/rc_ir/borrow.rs: prepend_rc

<1>3. `<1>1` の第 2 引数より、`before` の節点は `is_release` が偽なので `Retain`、`after` の節点は真なので
      `Release` である。
  BY <1>1, <1>2, CODE src/rc_ir/borrow.rs: rc_node

<1>4. `call_rc(callee, args)` は `params = self.callee_params.get(&FuncRef { name: callee.name.clone() })` を
      とり、`args` を添字つきで、各 `arg` について `rc_units(&arg.ty, self.type_env)` を順に回る。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>4a. `params` が `Some(ps)` であるとき `args.len() ≤ |ps|` であり、各引数の添字は `ps` の範囲内で
       ある。
  <2>0. `ctx.rewrite` が受け取る本体のこの `App` について、次の 3 つが成り立つ。(a) `args` の個数は
        `borrow_ify` の入力の対応する `App` のものに等しい。(b) `callee.name` の `name` フィールドを
        `#` で区切った最後の断片は `borrow` ではないので、`callee.name` は借用版の名前ではない。
        (c) `callee.name` が入力の関数の名前であるならば、入力の対応する `App` の callee の名前も
        `callee.name` である。
    `ctx.rewrite` が受け取る本体は、入力の関数の本体か入力のグローバル初期化子の `init` (`f_own` と
    グローバル初期化子) か、入力の関数の本体を `clone_func` が一斉に付け替えたもの (`f_borrow`) である。
    前の 2 つでは本体が入力のものそのものであり、この `App` が入力の対応する `App` そのものなので
    (a) と (c) が出る。A13 より入力に現れるすべての
    名前の最後の断片は `borrow` ではないので (b) が出る。後者について、P9 の前半より付け替えは節点の
    種類・並び・変数の型を変えず、変数の出現だけを `ρ_f` で写すので、`args` の列の長さは変わらず (a) が
    出る。`ρ_f` は名前を 2 通りに写す -- 鍵でない名前はそのまま残し (`rename_var` は鍵を持たない名前を
    そのまま残す)、鍵である名前は 4.2 の言明より最後の断片が `b` の後に 10 進数字が続く形の名前へ写す。
    前者は入力に現れる名前なので A13 より最後の断片が `borrow` ではなく、後者の `b<10 進数字>` も
    `borrow` と異なるので、(b) が出る。**局所変数を経由する間接呼び出しの callee は後者である** --
    `rename_rhs` の `App` の腕は callee を `rename_var` で写すので、その名前は複製が導入したものになる。
    (c) はその形から出る -- `callee.name` が入力の関数の名前ならば、A13 よりその最後の断片は
    `b<10 進数字>` の形ではないので `ρ_f` の像ではなく、よって `ρ_f` が写す前の名前がそのまま残った
    ものである。借用版の名前の最後の断片が `borrow` であるのは、`borrow_funcref` が借用版の名前を
    元の名前に `#borrow` を足して作り、`borrow` が `#` を含まないからである。
    BY A13, P9, 4.2 の言明, CODE src/rc_ir/borrow.rs: borrow_funcref,
       CODE src/rc_ir/rename.rs: rename_var, rename_rhs
  <2>1. `params` が `Some(ps)` であるとき、`callee.name` は `borrow_ify` の入力の `prog.funcs` の鍵で
        ある。その関数を `f` とすると `args.len() = |f.params|` である。
    `params` は `callee_params` を `callee'.name` で引いた値である (`<1>4`)。L6b より `callee'` は
    `callee` そのものか、`borrow_versions` が `FuncRef { name: callee.name }` に対応させる借用版の名前を
    持つ複製かのどちらかである。前者のとき、L6 より `callee_params` の鍵は入力の関数の名前か借用版の
    名前であり、`<2>0` の (b) より `callee.name` は借用版の名前ではないので、入力の関数の名前である。
    後者のとき、L6 より `borrow_versions` の鍵は入力の関数の名前なので `callee.name` はやはり入力の
    関数の名前である。A22 よりそれは入力の `prog.funcs` の鍵であり、その関数が `f` である。
    `<2>0` の (c) より入力の対応する `App` の callee の名前も `callee.name` である。A6 より入力の
    どの束縛名も関数の名前と異なるので、その `App` について `resolve_callee_params` は
    `closure_targets` の枝で外れ (その鍵は `RcRhs::Closure` を右辺に持つ `Let` の束縛変数の名前で
    ある)、`prog.funcs` の枝で `f` を引く。A14 はその `App` の `args` の個数を、
    `resolve_callee_params` が静的に引く関数のパラメータの個数に等しいとするので
    `args.len() = |f.params|` であり、`<2>0` の (a) よりその個数はこの `App` の `args` の個数である。
    BY A6, A14, A22, L6, L6b, <1>4, <2>0, CODE src/rc_ir/ownership.rs: resolve_callee_params,
       collect_bindings
  <2>2. CASE `route` が名前を差し替えなかった。
    `ps = param_names_and_types(f)` は `f.params` に `f.capture` を鎖にした列なので
    `|ps| ≥ |f.params|` であり、`<2>1` より `args.len() ≤ |ps|` である。
    BY <2>1, CODE src/rc_ir/borrow.rs: borrow_ify, param_names_and_types
  <2>3. CASE `route` が名前を差し替えた。
    L6b よりその名前は `borrow_versions[FuncRef { name: callee.name }]` すなわち `f` の借用版
    `f_borrow` の名前であり、`ps = param_names_and_types(f_borrow)` である。`clone_func` は
    `f_borrow.params` を `fresh_rename_function` から取り、それは `f.params` の各元を `rename_var` で
    1 つずつ写した列なので長さが等しい。`f_borrow.capture` は `None` である -- `borrow_versions` に
    載るのは `f.capture.is_none()` である関数だけであり、`fresh_rename_function` は `None` を `None` に
    写す。よって `|ps| = |f.params|` であり、`<2>1` より `args.len() ≤ |ps|` である。
    BY L6b, <2>1, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func, param_names_and_types,
       CODE src/rc_ir/rename.rs: fresh_rename_function, rename_var
  <2>4. QED
    L6b より `route` の返り値は `<2>2` と `<2>3` の 2 つの場合で尽きる。
    BY L6b, <2>2, <2>3

<1>5. その内側で `callee_owns` は `params` が `None` のとき `true`、`Some(params)` のとき
      `self.owned_units.contains(&(params[arg_idx].0.clone(), unit.clone()))` であり、`arg_owned` は
      `self.owns_unit(arg, &unit)` である。`<1>4a` より `arg_idx` は `params` の範囲内である。
  BY <1>4, <1>4a, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>6. `!callee_owns && arg_owned` のとき `after.push((arg.clone(), unit))`、
      `callee_owns && !arg_owned` のとき `before.push((arg.clone(), unit))`、それ以外のとき何も積まない。
      `if`/`else if` の 2 分岐であり、どちらの条件も満たさない対は素通りする。
  BY <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>7. QED
  `<1>4` の 2 重ループの順序が `before` と `after` の並びを決め、`<1>6` がその中身を決める。`<1>1`-`<1>3` が
  節点の位置と種類を決める。`call_rc` に渡されるのは振り分け後の `callee` なので、`params` は
  `callee'` について引かれる。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

**`params[arg_idx]` が範囲内であること**。`<1>4a` がこれを示す。A14 が個数を抑えるのは**呼び出し先の**
パラメータであり、`call_rc` が引くのは振り分け後の版のパラメータなので、この 2 つの個数が一致することが
そこに要る。

**P11 が保留した節について**。README の P11 は「この節点が義務集合 (D10) の食い違いを**ちょうど**埋める
ことは、別の主張であり、P14 が示す。」と述べる。P11 の第 1 文の「呼び出し元が借用し呼び出し先が所有する
unit には前に `Retain` を」を第 2 文の言い換えとして読むなら、上の言明がそれである。実行時の義務集合
(D10) についての主張として読むなら、すなわち「補正の後、呼び出しの前後で `Obl` の収支が合う」として読むなら、それは
第 10.3 節と第 10.4 節が示す。

## 7. P12 -- 振り分けの安全性

**P12 (a)**。`route(x, callee, args, k)` が `callee` と異なる名前の `RcVar` を返すのは、
`self.borrow_versions` が `FuncRef { name: callee.name }` を鍵に持ち、かつ `self.routing_is_safe(x, args)` と
`self.routing_saves_retain(borrow_version, args, k)` がともに真であるときだけである。そのとき返る名前は
`self.borrow_versions[&FuncRef { name: callee.name }].name` である。

<1>1. L6b が言明そのものを述べる。
  BY L6b

<1>2. QED
  BY <1>1

**P12 (b)**。`routing_is_safe(x, args)` が真であるのは、`x.name` が `self.tail` に入らないとき (末尾位置の
呼び出しでないとき) か、`args` のどの元 `a` についても `self.any_owned_unit(a)` が偽であるとき
(所有する unit を持つ引数を 1 つも持たないとき) である。

<1>1. `routing_is_safe` は `!self.tail.contains(&x.name) || !args.iter().any(|a| self.any_owned_unit(a))`
      である。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::routing_is_safe

<1>2. `self.tail` はその版の本体についての `tail_result_vars` の値であり、`mark_tail` が末尾位置の
      `App` と `Match` の束縛変数を集めたものである。
  関数の版では `RewriteCtx::new(func, ..)` が `tail: tail_result_vars(&func.body)` を置く。グローバル
  初期化子では `borrow_ify` が `RewriteCtx` を構造体リテラルで作り、`tail: tail_result_vars(&g.init)` を
  置く。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, borrow_ify のグローバルを写す繰り返し, tail_result_vars,
     mark_tail

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
  L6 より出力の `funcs` の鍵の集合は「入力の各関数の名前」と「`borrow_versions` の各値」の合併であり、
  A22 より入力の `funcs` の各エントリの鍵はその `RcFunc` の `name` に等しいので、入力の `funcs` の鍵は
  「入力の各関数の名前」である。
  BY A22, L6

<1>4. QED
  BY <1>1, <1>2, <1>3

**P12 (d) (門)**。`observing` に入る関数は借用版を持たず、その名前への直接呼び出しは `route` を素通りする。

<1>1. 1 番目のループは `observing.contains(&func.name)` のとき `continue` するので、`borrow_versions` は
      `observing` の元を鍵に持たない。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `route` は `borrow_versions.get(&orig)` が `None` のとき `callee.clone()` を返す。
  BY P12 (a), <1>1

<1>3. `observing = funcs_observing_uniqueness(prog)` は、次のグラフの上の最小不動点である。頂点は
      `prog.funcs` の各関数。種は、本体に `llvm_gen.observes_uniqueness()` が真の `RcRhs::Llvm` を持つ
      関数。辺は 3 種である。

  - **直接呼び出しの辺**。`f` の本体に `RcRhs::App(callee, _)` があり `FuncRef { name: callee.name }` が
    `prog.funcs` の鍵であるときの、`f` からその関数への辺。
  - **名指さない適用の辺**。`f` が `calls_indirectly` に入るときの、`f` から**すべての closure の対象**への
    辺。`f` が `calls_indirectly` に入るのは 2 つの場合である。`f` の本体に `App` があってその名前が
    `prog.funcs` の鍵でないとき、および `f` の本体に `llvm_gen.applies_a_function_operand()` が真の
    `RcRhs::Llvm` があるときである。
  - **解放の辺**。`builds_a_destructor(prog)` が真のときの、`prog.funcs` の**すべての**関数からすべての
    closure の対象への辺。

  closure の対象は、`prog.funcs` の全本体と `prog.globals` の全 `init` を歩いて `RcRhs::Closure(target, _)`
  から集めたものである。グローバル初期化子は `owner` が `None` なので、種にも `callees` の鍵にもならず、
  closure の対象だけを寄与する。
  `scan` は `for_each_node` で本体の全節点を歩き、`RcRhs::Llvm` の腕で `observes_uniqueness()` が真なら
  `owner` を `observing` に、`applies_a_function_operand()` が真なら `owner` を `calls_indirectly` に、
  `RcRhs::App` の腕で `prog.funcs.contains_key(&target)` の真偽により `cs` か `calls_indirectly` に、
  `RcRhs::Closure` の腕で `closure_targets` に積む。走査の後、`calls_indirectly` の各元の `callees` に
  `closure_targets` の全元が足され、`builds_a_destructor(prog)` が真ならば `prog.funcs` の各鍵の
  `callees` にも足される。最後のループが、`cs` の元が `observing` に入る頂点を `observing` に入れる
  ことを、変化が無くなるまで繰り返す。
  BY CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness, builds_a_destructor,
     CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness, LLVMGen::applies_a_function_operand,
     CODE src/rc_ir/ast.rs: for_each_node

<1>4. QED
  BY <1>1, <1>2, <1>3

**P12 (e) (間接呼び出しの callee の名前)**。`V` の本体の `Let(x, App(callee, args), k)` の `callee` が
局所変数であるとき、`route` は `callee` をそのまま返し、その名前は入力の `funcs` の鍵でも出力の `funcs` の
鍵でもない。

<1>1. `callee` が局所変数であるとは、その名前が `V` の本体の束縛名 -- パラメータ・capture の名前か、
      節点が束縛する変数の名前 -- であることである。`V` は出力の版なので、それは出力の束縛名
      (DEF 出力の束縛名) である。
  BY D6, DEF 出力の束縛名

<1>2. その名前は入力の `funcs` の鍵でも出力の `funcs` の鍵でもない。
  BY 4.4 の系, <1>1

<1>3. `route` は `callee.clone()` を返す。
  P12 (a) より `route` が異なる名前を返すのは `borrow_versions` が `FuncRef { name: callee.name }` を鍵に
  持つときだけであり、L6 よりその鍵はどれも入力の関数の名前、すなわち入力の `funcs` の鍵である
  (A22)。`<1>2` よりこの名前はその鍵ではない。
  BY A22, L6, P12 (a), <1>2

<1>4. QED
  BY <1>2, <1>3

**「同じ関数の版である」について**。`borrow_versions` の鍵 `orig` に対する値は
`borrow_funcref(&func.name)` であり、`func.name` の `name` フィールドに `#borrow` を継ぎ足したものである
(`CODE src/rc_ir/borrow.rs: borrow_funcref`)。よって振り分け先は、`orig` の借用版として作られた版である。
`README.md` の P12 の「呼び出し先が入力の関数を名指すとき、返る名前は出力の `funcs` の鍵である」は
P12 (c) が、「局所変数を経由する間接呼び出しでは `route` は呼び出し先をそのまま返し、その名前はどちらの
`funcs` の鍵でもない」は P12 (e) が与える。

## 8. P13 -- 注釈の一致

**言明**。出力の各関数の版 `V` について、`V.borrowed_units` は `param_capture_units(V, type_env)` の元の
うち `owned_units` に入らないものの集合である。出力のグローバル初期化子については、`RcGlobalInit` が
`borrowed_units` の欄を持たず、パラメータも capture も持たない (D1) ので、この命題が述べる集合は無い
(`<1>2a`)。

<1>1. `borrow_ify` は `funcs` を組み立てた後、`for func in funcs.values_mut()` のループで
      `func.borrowed_units = param_capture_units(func, type_env).into_iter()
      .filter(|unit_path| !owned_units.contains(unit_path)).collect();` を実行する。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. このループは出力の `funcs` の全元を回り、その代入が各元の `borrowed_units` を決める。
      `f_own` は `func.clone()` の `body` を差し替えたものなので入力の関数の集合を写しており、
      `f_borrow` は `clone_func` が `Set::default()` を置いたものである。どちらもこのループの代入で
      上書きされる。ループは `funcs` を組み立てた後に走り、`borrow_ify` はその後 `funcs` を返すだけで
      ある。D14 も「`borrowed_units` に unit を**入れる**のは `borrow_ify` の末尾ただ 1 か所であり」
      「他の書き込みは空集合を置くか既存の鍵を改名するだけである」と述べる。
  BY D14, EXT 集合と写像, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>2a. 出力のグローバル初期化子について、この命題は集合を述べない。
  `RcGlobalInit` は `symbol`、`ty`、`init`、`owns_initializer`、`owns_storage` の 5 つのフィールドを
  持ち、`borrowed_units` の欄を持たない (D1)。D1 より初期化子はパラメータも capture も持たないので、
  `param_capture_units` が数える unit も無い。D14 の割り当ても、`borrowed_units` を読む先が無いので
  初期化子については空である。
  BY D1, D14, CODE src/rc_ir/ast.rs: RcGlobalInit, CODE src/rc_ir/borrow.rs: param_capture_units

<1>3. QED
  `<1>1` と `<1>2` が関数の版を、`<1>2a` がグローバル初期化子を与える。DEF 出力の版 より出力の版は
  この 2 種で尽きる。
  BY DEF 出力の版, <1>1, <1>2, <1>2a

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

### 9.1 固定するものと、P14 が読む 2 つ

以下、出力の版 `V` を 1 つ固定する。`B_V` を `RewriteCtx::rewrite` が受け取る本体、`B'_V` をその値、
`ctx` を `V` の `RewriteCtx` とする。すなわち `f_own` とグローバル初期化子については `B_V` は入力の本体で
あり、`f_borrow` については `B_V` は `clone_func` が返した本体である
(`CODE src/rc_ir/borrow.rs: borrow_ify`)。

**固定する活性化**。`B'_V` の活性化 (D21) を 1 つ固定し、それが辿る `B'_V` の実行路に対応する `B_V` の
実行路を `ρ` とする。実行路が 1 対 1 に対応することは L16 が、この活性化に対応する `B_V` の活性化 --
パラメータ・capture の値と、オペランドから結果が決まらない 4 種の各位置での結果を **L16 が対応させる
位置について**共有する活性化 -- が在ることは L16a が与える。その 1 つを取って固定する。以下、この 2 つを
併せて「この活性化」と呼び、`n_in` は `B_V` の
側、`n_out` は `B'_V` の側について読む。スロット (D6)、由来 (第 9.3 節)、inhabited (D16) はどちらの側で
読んでも同じものを指す -- L16a より対応する位置で対応する変数が得る値は等しい。

**この節が渡す 3 つ**。第 10 節が直に読む A19 (ii-a)・A20・A26 を、以下でこの文書の記法へ渡す。P14 が読む
仮定はこの 3 つに限らない -- 第 0 節が全体を挙げる。

**A26 の主語と「手放し」の範囲。** A26 は「1 つの節点について、D7 の読む構文がオブジェクトの
**記憶域から読む**動作は、その節点が行うどの参照の**手放し**よりも前に起きる」と述べる。**主語は節点で
あって段ではない** -- A26 はその読みを名指して退け、「D24 の (E2) は「この段は活性化を 1 つ作るごとに
区切られる」と述べるので、`applies_a_function_operand` を宣言する op の節点は段を複数持つ。段ごとに
読むと、前の段の手放しと後の段の読みの順序が付かない」と書く。この文書も節点を主語にして読む。

**「手放し」の範囲も A26 が自分で定める** -- 「「手放し」は D10 の消費と `Release` の両方である。渡す先の
ある消費を含む」。第 10.7 節の `<3>1` はこの形で読む -- そこで要るのは、読みの点までに `n_out` が減って
いないことであり、`n_out` は D9 の消費のどちらの向きでも減る (DEF 由来ごとの義務)。

残る 2 つを第 9.7 節の記法へ渡す形で書くと次のようになる。

- **A19 (ii-a)**。第 9.7 節の `n_in` について次の 4 つが成り立つ。(a) `ρ` の各節点の入口 `τ` と各計数下
  (D26) の `ρ`-由来 `T` について `n_in(τ, T) ≥ 0` である。(a') `ρ` の終端の `Ret` の消費を行った直後の
  位置 `τ_a` と各計数下の `ρ`-由来 `T` についても `n_in(τ_a, T) ≥ 0` である。(b) D7 の読む構文の節点の
  入口 `τ` と、その表が名指す値の inhabited な boxed leaf のスロット `(x, λ)` について、`obj(x, λ)` が
  計数下であるとき `n_in(τ, T_ρ(x, λ)) ≥ 1` である。(b') `Retain(v, π)` か `Release(v, π)` の節点の
  入口 `τ` と、`π` の下の inhabited な各 leaf `λ` について、`obj(v, λ)` が計数下であるとき
  `n_in(τ, T_ρ(v, λ)) ≥ 1` である。L27 がこの形を A19 から導く。

  **(b) と (b') が読む時点は節点の入口である。** A19 (ii-a) は「各時点と各計数下の別名類について、
  その類が持つ参照の個数は非負であり、読む構文と `Retain`/`Release` がその類を名指す時点では 1 以上で
  ある」と述べ、A19 は「「各時点」は、その活性化が生きている (D23) 間の、その活性化の節点の訪問の入口で
  ある時点である。」「この 2 つを読める点はそこに限る。」と定める。構文が値を名指すのはその構文の節点で
  あり、DEF 時点 はその節点の入口を時点に数える。
  D11 の (S-c) が課すのは「その読み・その触れる動作が実際に起きる瞬間の直前」の点である。節点の入口から
  その点へ渡すのは、読みについては A26 (上の主語と「手放し」の読みの下で、節点の入口から読みの点までに `n_out`
  を減らす事象が無い)、`Retain` が触れる動作については `Retain` の事象がどの由来も減らさないこと (D10)
  である (第 10.7 節の `<3>1` と `<3>2`)。

  **(a') を別に書くのは、A19 がその 1 点を名指しているからである。** A19 の (ii-a) は「各時点と各計数下の
  別名類について、その類が持つ参照の個数は非負であり、読む構文と `Retain`/`Release` がその類を名指す
  時点では 1 以上である。**非負であることは、終端の `Ret` の消費を行った直後の時点についても言う。**」と
  書く。A19 は「「各時点」は、その活性化が生きている (D23) 間の、その活性化の節点の訪問の入口である
  時点である。」と定め、D23 は「活性化 `a` が**終わる**とは、`a` の位置が `B(a)` の終端の `Ret` に着き、
  その `Ret` の消費 (D9) を行うことをいう」「**生きている活性化**とは、始まって終わっていない活性化で
  ある」と書くので、終端の `Ret` の消費を行った直後の時点は「各時点」に入らない -- その点でこの活性化は
  生きておらず、その点は節点の訪問の入口でもない。最後の一文がその 1 点を足す。

  読む者は L28 であり、L28 を読むのは (S-a) の終端の `Ret` の段 (第 10.4 節の `<1>5a`) と (S-b)
  (第 10.6 節) である。**(a) だけではこの 2 つは閉じない。** 終端の `Ret(v)` は D7 の読む構文でも
  `Retain`/`Release` でもないので (b) も (b') も当たらず、節点の入口での非負性は「終端の `Ret` が消費
  する分がその類に在る」を与えない。D11 の (S-a) と (S-b) はオブジェクトごとの多重集合しか見ないので、
  1 つのオブジェクトを指す 2 つの別名類の間で終端の `Ret` で収支を融通する本体は、節点の入口では非負で
  ありながら `n_in(τ_a, ・)` を負にできる。A19 の解説が「1 つのオブジェクトを指す 2 つの別名類の間で
  収支を融通する本体は D12 を満たしながらこれを破り、そのとき借用版が落とす節点と残す節点の対応が
  崩れる」と書くのはこの形である。

- **A20**。`V` が D14 の意味で借用するパラメータ・capture の unit `u` について、`u` の下の inhabited な
  leaf が指す計数下のオブジェクトは、この活性化が生きている間 解放されていない。第 10.7 節の `<1>4` が
  これを A20 から導く。A20 は呼び出し元についての仮定なので、この活性化に呼び出し元が在ることが要り、
  それを L18a が与える。

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
  第 1 節に写した `p15` の `L9` は仮説つきである -- 「`trunc(τ, ・)` が `under(τ, p)` の各要素について
  値を返すとき、その値は `units(τ)` の要素である」。言明は `ctx.owns_object(r, p)` が値を返す `(r, p)` に
  限っており、L4 より `owns_object` のこの場合の腕は `under(τ, p)` の各 `unit` について
  `trunc(τ, unit)` を計算して `owned_units` を引くので、値を返すことがその仮説を与える。
  BY L4, p15-ownership-uniformity.md の L9, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

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
`ρ` の上のスロットであり、`obj(x', λ') = obj(x, λ)` である。そのオブジェクトが計数下 (D26) であるときは、
両者が持つ参照は同一である。また `T_ρ(x, λ)` は有限歩で定まる。

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
    <3>1. `Move(y)`、`Payload(s, ・)`、`Field(c, idx)`、`Llvm(gen, args, ・)` の行では、名指される
          `y`・`s`・`c`・`args[j]` は `x` を束縛する節点のオペランドである。`Llvm` の行の `j` が `args` の
          添字であることは、A3 の「**単一の `Arg(j, σ)` の宣言は well-formed である。**`j` は `args` の
          添字であり、`σ` はその型の boxed leaf である」による。A11 より変数の使用はその
          位置でスコープに入っている束縛に解決するので、これらはその節点より前に値を得ている。
      BY A3, A11, D6, <1>1
    <3>2. `Join(arm_results)` の行の `a_ρ` は、`x` を束縛する節点 `Let(x, Match(s, arms), k)` の、`ρ` が
          選んだアームの本体の終端の `Ret` が名指す変数である。`a_ρ` が値を得るのはこの節点より前では
          なく、この節点の中である。D3 より `ρ` はそのアーム本体を辿ってから `k` へ進むので、`a_ρ` は
          `x` が値を得る時点までに値を得ており、D6 が要求するのはその時点までに値を得ていることだけで
          ある。
      BY D3, D6, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var
    <3>3. QED
      BY <3>1, <3>2
  <2>2. `λ'` は `ty(x')` の boxed leaf である。
    `Move` と `Payload(s, None)` では A12 より `ty(x') = ty(x)`。`Field` の unbox の行と
    `Payload(s, Some(t))` の unbox の行では、A12 より `ty(x)` は `ty(x')` の第 `idx` (resp. 第 `t` 変位の)
    フィールドの型であり、D4 の第 5 の規則より `[idx] ++ λ` (resp. `[t] ++ λ`) は `ty(x')` の leaf で
    ある。`Llvm` の行では A3 の「単一の `Arg(j, σ)` の宣言は well-formed である」より
    `σ ∈ leaves(ty(args[j]))` である。`Join` では A12 より `ty(a_ρ) = ty(x)` である。
    BY A3, A12, D4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `λ'` はその時点で inhabited (D16) である。
    `Move`・`Payload(s, None)`・`Join` では 2 つのスロットの値が同じなので、D16 の条件も同じである。
    値が同じであることは D9 の値の水準の行が与える -- `Let(x, Var(y), k)` の行は「`x` の値は `y` の値で
    ある」、catch-all アームの payload 束縛の行は「payload 変数の値は scrutinee の値そのものである」、
    `Match` のアーム本体の `Ret(x)` の行は「`Match` の束縛変数の値は `x` の値である」と述べる。
    `Field` の unbox の行で足す添字は unbox 構造体のフィールド添字であり (A12 より `Destructure` の
    容器は構造体)、D16 が数える unbox union の節を増やさない。`Payload(s, Some(t))` の unbox の行が足す添字は unbox union の節を
    1 つ増やすが、D21 よりこの活性化がこのアームを選んだのは scrutinee の実行時のタグが `t` に等しい
    ときであり、A16 よりそのようなアームが選ばれるので、その節について D16 の条件は成り立つ。`Llvm` の
    行では A3 の第 2 行が「結果のその leaf が inhabited であることと、第 `j` オペランドの leaf `σ` が
    inhabited であることは同値である」と述べる。
    BY A3, A12, A16, D9, D16, D21
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>3. `obj(x', λ') = obj(x, λ)` であり、そのオブジェクトが計数下 (D26) であるときは両者が持つ参照は
      同一である。
  <2>1. 2 つのスロットが保持する値は等しい。
    `<1>1` より 1 歩は D9 の移動の 1 行であり、D9 の**値の水準**の 6 行がその行ごとに何が渡るかを述べる
    -- `Let(x, Var(y), k)` では `x` の値は `y` の値、アーム本体の `Ret` では `Match` の束縛変数の値は
    その変数の値、unbox 容器の `Destructure` ではフィールド変数の値は容器の値のそのフィールド、
    unbox union の変位アームでは payload 変数の値は scrutinee の値の活性変位の payload、catch-all
    アームでは payload 変数の値は scrutinee の値そのもの、`Llvm` の素通し leaf では結果のその leaf の
    値はオペランド `i` のその leaf の値である。1 歩の表の各行はこの 6 行の 1 つに当たり、`(x, λ)` の
    位置の値と `(x', λ')` の位置の値を等しいものにする。
    BY D9, <1>1
  <2>2. よって `obj(x', λ') = obj(x, λ)` である。
    D6 より `obj(x, λ)` はスロット `(x, λ)` が指すオブジェクト、すなわち `x` の値の leaf `λ` の位置に
    在るポインタが指すオブジェクトである。`<2>1` よりその位置の値は両側で等しい。
    BY D6, <2>1
  <2>3. そのオブジェクトが計数下であるとき、両者が持つ参照は同一である。
    D9 より移動は「参照の持ち手が活性化の中で変わるだけの構文」であり、義務集合を変えない。A5 より、
    値が保持する参照は inhabited であって計数下のオブジェクトを指す各 leaf にちょうど 1 つずつある。
    `<2>2` より両端は同じオブジェクトを指すので、両端ともその 1 つを持ち、移動はその持ち手を変える
    だけなので 2 つは同一の参照である。
    BY A5, D9, D26, <2>2
  <2>4. QED
    `obj(x, λ)` がグローバル状態であるときは、D26 よりどちらの leaf も D8 の意味の参照を持たないので、
    後半の主張は当たらない。`<2>2` が前半を、`<2>3` が後半を与える。
    BY D8, D26, <2>2, <2>3

<1>4. `T_ρ(x, λ)` は有限歩で定まる。
  <2>1. DEF 由来の 1 歩 の表の各行の 1 歩の先は、`origin_inner(vars, type_env, x, λ)` が `origin` を
        呼ぶ相手の 1 つである。
    `origin_inner` は `vars.bindings.get(var)` で場合を分け、`Binding::Move(y)` の腕は
    `origin(.., &y.name, path)` を、catch-all の `Binding::Payload(scrut, None)` の腕は
    `origin(.., &scrut.name, path)` を、unbox scrutinee の `Binding::Payload(scrut, Some(tag))` の腕は
    `origin(.., &scrut.name, [tag] ++ path)` を、unbox 容器の `Binding::Field(container, idx)` の腕は
    `origin(.., &container.name, [idx] ++ path)` を、`decl.leaf_origins_at(path)` が単一の `Arg(j, p)` で
    ある `Binding::Llvm` の腕は `origin(.., &args[j].name, &p)` を呼ぶ。これは表の第 1・第 2・第 3・
    第 4・第 5 行の 1 歩の先そのものである。`Binding::Join(arm_results)` の腕は各 `arm_result` について
    `origin(.., &arm_result.name, path)` を呼び、表の第 6 行はそのうち `ρ` が選んだアームの
    `returned_var` についての呼び出しを取る。
    BY DEF 由来の 1 歩, CODE src/rc_ir/ownership.rs: origin_inner, collect_bindings
  <2>2. 1 歩の列は、`origin(x, λ)` の再帰の木の根から下る 1 本の枝である。
    `origin` は memo を引き、外れたときに `grow_stack(|| origin_inner(..))` を呼ぶ。A15 より
    `grow_stack(f)` は `f` をちょうど 1 回呼ぶ。よって `origin(x, λ)` の計算は、`origin` の呼び出しを
    節点とする木をなす。`<2>1` より各 1 歩はその木の 1 本の辺であり、1 歩の列は根 `(x, λ)` から下る枝で
    ある。
    BY A15, <2>1, CODE src/rc_ir/ownership.rs: origin, CODE src/misc.rs: grow_stack
  <2>3. QED
    P2 より `origin(x, λ)` は停止するので、`<2>2` の木は有限であり、枝も有限である。その末端は
    DEF 由来の 1 歩 の「上のどれでもない」に当たる位置であり、そこで `origin_inner` は `origin` を
    呼ばない (`<2>1` の場合分けの残る腕 -- `None`、`Binding::Param`、`Binding::Producer`、boxed 容器の
    `Binding::Field`、boxed scrutinee の `Binding::Payload`、単一の `Arg` でない `Binding::Llvm` -- は
    `here()` を返すか `origin_from_leaves_under` を呼ぶ。後者について L6a が `origin` を呼ばないことを
    述べ、L6a が要求する「`σ` が `ty(u)` の boxed leaf である」は、その位置が `ρ` の上のスロットである
    こと (D6) と、`<1>2` が 1 歩ごとにそれを保つことによる)。
    BY D6, DEF 由来の 1 歩, L6a, P2, <1>2, <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_inner

<1>5. QED
  BY <1>2, <1>3, <1>4

#### L9a (DEF 由来 は D33 の別名類を与える)

**言明**。`ρ` の上のスロット `(x, λ)` について、DEF 由来 の `T_ρ(x, λ)` は `(x, λ)` の D33 の
**`ρ` 終端**であり、`T_ρ` が等しいスロットの同値類は D33 の別名類である。すなわち `ρ`-由来 `T` について、
`T_ρ(x, λ) = T` である任意のスロット `(x, λ)` の D33 の類 `C_ρ(x, λ)` は
`{ (y, μ) : (y, μ) は `ρ` の上のスロットであり `T_ρ(y, μ) = T` }` であり、`T_ρ(C_ρ(x, λ)) = T` である。

<1>1. DEF 由来の 1 歩 の表の 6 行は D20 の別名の辺の 6 つと 1 対 1 に対応する。
  BY D20, DEF 由来の 1 歩, L9

<1>2. QED
  D33 は「1 つの実行路 `ρ` の上のスロット (D6) を、`ρ` 終端が等しいという関係で分けた同値類を
  **別名類**と呼ぶ」と述べ、スロット `(x, λ)` が属する別名類を `C_ρ(x, λ)`、その終端を
  `T_ρ(C)` と書き、その歩みの各段が D20 の別名の辺であるとする。`<1>1` より DEF 由来の 1 歩 がその
  歩みであり、DEF 由来 の「1 歩を繰り返して着く `ρ`-由来」がその終端である。よって `T_ρ` が等しい
  スロットの類が D33 の別名類である。
  BY D20, D33, D6, DEF 由来, DEF 由来の 1 歩, <1>1

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
  すなわち `Origin::Exactly((u, σ))` を返す。最後の場合は L6a が同じ答えを与える。L6a が要求する
  `σ ∈ leaves(ty(u))` は、`(u, σ)` が `ρ` の上のスロットであること (D6) と、L9 が 1 歩ごとにそれを保つ
  ことによる。
  BY D6, L6a, L9, CODE src/rc_ir/ownership.rs: origin_inner

<1>2. 1 歩が `Move`・`Payload`・`Field` の unbox・`Llvm` の単一 `Arg` のいずれかであるとき、
      `origin(x, λ) = origin(x', λ')` である。
  この 4 つの腕はいずれも `origin(...)` の値をそのまま返す。
  BY CODE src/rc_ir/ownership.rs: origin_inner

<1>3. 1 歩が `Join(arm_results)` であるとき、`S := ⋃_{a ∈ arm_results} act(a, λ)` として、
      `origin(x, λ) = Origin::of_candidates(S, (x, λ))` であり、`cand(x, λ)` は `|S| = 1` のとき `S`、
      `|S| ≥ 2` のとき `S` である。
  `Binding::Join` の腕は各アーム結果の `acted_on()` を `candidates` に集めて `of_candidates` に渡す。
  `of_candidates` は `|S| = 1` のとき `Exactly` を、それ以外のとき `Join { candidates: S }` を返し、
  `candidates()` はどちらでも `S` を与える。`S` が空のときは `assert!` で中断するが、`S` は空でない --
  `Binding::Join(arm_results)` の `arm_results` は `collect_bindings` が `Match` の各アームの
  `returned_var` を並べたものであり、A9 より `Match` は 1 つ以上のアームを持つので `arm_results` は
  空でなく、`Origin::acted_on` は `identity()` を先頭に持つので各 `act(a, λ)` も空でない。
  BY A9, CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates, Origin::candidates,
     Origin::acted_on, collect_bindings

<1>4. (a) と (b) が成り立つ。
  1 歩の列の長さについての帰納で示す。長さ 0 のとき `<1>1` より `cand(x, λ) = {(x, λ)} = {T_ρ(x, λ)}` で
  あり `Exactly` である。長さが 1 以上のとき、`<1>2` の 4 つの場合は `origin` そのものが等しいので
  帰納法の仮定がそのまま渡る。`<1>3` の場合、帰納法の仮定より `T_ρ(x, λ) = T_ρ(a_ρ, λ) ∈ cand(a_ρ, λ)` で
  あり、`cand(a_ρ, λ) ⊆ act(a_ρ, λ) ⊆ S = cand(x, λ)` である。`Exactly` になるのは `|S| = 1` のときで、
  そのとき `cand(x, λ) = S` の唯一の元は `T_ρ(x, λ)` である。
  BY 帰納法の仮定, <1>1, <1>2, <1>3, CODE src/rc_ir/ownership.rs: Origin::acted_on

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
  <2>1. `(a_0, λ)` は `collect_consumes` が積む元である。
    `returned_var(arm.body)` はそのアーム本体の終端の `Ret` が名指す変数であり、`collect_consumes_go` は
    `RcRhs::Match` の腕で各アーム本体へ降り、その `RcExpr::Ret` の腕で `push_boxed_leaves` を呼ぶ。
    BY <1>2, CODE src/rc_ir/ownership.rs: collect_consumes_go, returned_var, push_boxed_leaves
  <2>2. `cand(a, λ) = ρ_f(cand_f(a_0, λ))` である。ここで `ρ_f` は `VarPath` に対しその変数だけを写す。
    第 1 節に写した `p15` の `L15` の (ii) は、`func` に現れる各名前 `x` と任意の path `π` について
    `origin(vars_c, type_env, ρ_f(x), π) = ρ_f(origin(vars_f, type_env, x, π))` であると述べる。
    `Origin::candidates` は `Exactly(p)` に `{p}`、`Join { candidates, .. }` に `candidates` を返すので、
    変数の付け替えと可換である。`<1>2` より `a = ρ_f(a_0)` であり、`a_0` は `func.body` に現れる名前で
    ある。
    BY <1>2, p15 の L15, CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>3. QED
    第 3.7 節の系より、`cand_f(a_0, λ)` の各元 `(r_0, p)` について `ctx.owns_object(ρ_f(r_0), p)` は
    真である。`<2>2` より `cand(a, λ)` の元はちょうどその `(ρ_f(r_0), p)` である。
    BY 3.7 の系, <2>1, <2>2

<1>4. `identity(a, λ)` について `ctx.owns_object` は真である。
  <2>1. `origin(a, λ)` が `Origin::Exactly(p)` であるとき、`identity(a, λ) = p` であり、それは
        `cand(a, λ)` の元である。
    `Origin::identity` は `Exactly(p)` に `p` を返し、`Origin::candidates` は `Exactly(p)` に `{p}` を
    返す。
    BY CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates
  <2>2. `origin(vars, type_env, y, π)` が返す `Origin::Join` の `identity` を `(w, σ)` とすると、
        `vars.bindings.get(w)` は `Some(Binding::Join(..))` か `Some(Binding::Llvm(..))` である。
    P2 より `origin` の再帰は有限なので、その再帰の上の帰納で示す。`Origin::Join` の値を作るのは
    `Origin::of_candidates` の `candidates.len() ≥ 2` の腕だけであり、その `identity` は引数として
    渡された `VarPath` である。`of_candidates` の呼び出しは 2 か所ある。1 つは `origin_inner` の
    `Binding::Join(arm_results)` の腕で、渡すのは `(var, path)` であり、その `var` は `bindings` が
    `Binding::Join` を持つ変数である。もう 1 つは `origin_from_leaves_under` の末尾で、渡すのは引数
    `here` であり、その唯一の呼び出し元は `origin_inner` の `Binding::Llvm` の腕で
    `here_identity = (var, path)` を渡すので、その `var` は `bindings` が `Binding::Llvm` を持つ変数で
    ある。`origin_inner` の残る腕は、`Origin` を新しく作らずに再帰の返り値をそのまま返すか
    (`Binding::Move`、catch-all の `Payload`、unbox の `Field` と `Payload`、単一の `Arg` を宣言する
    `Llvm`)、`Origin::Exactly` を返すかである。`origin_from_leaves_under` の「`reached` の全元が
    等しい」腕も再帰の返り値をそのまま返し、`reached` に自分で積むのは `Origin::Exactly(here)` だけで
    ある。`origin` の memo は答えをそのまま記録して返す。よって返る `Join` は上の 2 か所のどちらかが
    作ったものか、再帰の返り値をそのまま運んだものであり、後者については帰納法の仮定がその返り値に
    ついて言明を与えるので、どちらの場合も `identity` は上の 2 種のどちらかである。
    BY P2, 帰納法の仮定, CODE src/rc_ir/ownership.rs: origin, origin_inner, origin_from_leaves_under,
       Origin::of_candidates, Origin::identity
  <2>3. QED
    `<2>1` の場合は `<1>3` による。`Origin::Join` の場合、`<2>2` の `Binding::Join` と `Binding::Llvm` は
    どちらも `collect_bindings` が入れる `Binding` なので、第 1 節に写した `p15` の `L13` より
    `ctx.vars.param_tys` は `w` を鍵に持たず、L4 より `ctx.owns_object(w, σ)` は真である。`Origin` は
    `Exactly` と `Join` の 2 つの構成子を持つ。
    BY L4, p15-ownership-uniformity.md の L13, <1>3, <2>1, <2>2,
       CODE src/rc_ir/ownership.rs: Origin, collect_bindings

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
  <2>0. L10 の (c) が名指す `(z, μ)` は `ρ` の上のスロットであり、`z` の `Binding` は
        `Join(arm_results(z))` であって、各 `a ∈ arm_results(z)` について `μ ∈ leaves(ty(a))` である。
    L10 の (c) より `(z, μ)` は `(x, λ)` から 1 歩を繰り返して着くスロットである。`(x, λ)` は `ρ` の上の
    スロットであり、L9 より 1 歩の先も `ρ` の上のスロットなので、`(z, μ)` もそうである。D6 より `μ` は
    `ty(z)` の inhabited な boxed leaf である。A12 よりアームの結果と `Match` の束縛変数の型は等しいので
    `ty(a) = ty(z)` であり、`μ ∈ leaves(ty(a))` である。
    BY A12, D6, L9, L10
  <2>1. L10 の (c) より `cand(x, λ) = ⋃_{a ∈ arm_results(z)} act(a, μ)` であり、`<2>0` より L11 の仮説
        (`a ∈ arm_results(z)` かつ `μ ∈ leaves(ty(a))`) が満たされるので、L11 よりその各元に
        ついて `ctx.owns_object` は真である。よって 1 は真である。
    BY L10, L11, <2>0
  <2>2. QED
    L10 の (a) より `T_ρ(x, λ) ∈ cand(x, λ)` なので、`<2>1` より 2 も真である。1 と 2 がどちらも真なので
    同値である。
    BY L10, <2>1

<1>3. QED
  `Origin` は `Exactly` と `Join` の 2 つの構成子を持つ。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: Origin

#### L13 (site の下の inhabited な leaf の由来は `owns_unit` と一致する)

**言明**。`(v, u)` を `B_V` の site (DEF site) とし、`λ` を `u` の下の boxed leaf で
`ρ` のこの位置で inhabited なものとする。このとき `ctx.owns_unit(v, u)` が真であることと
`ctx.owns_object(T_ρ(v, λ))` が真であることとは同値である。

<1>0. DEF site が定める `B_V` の site は P7a が site と呼ぶ集合であり、この命題が P7a を読むのは、
      `V` の `RewriteCtx` (`ctx`) と `B_V` についてである。
  P7a は「**出力の版 `V` を 1 つ固定し、`owns_unit` と `owns_object` は `V` の `RewriteCtx` のものと
  する。**」と書き、site を「**site とは、その版が書き換える本体 -- 関数の `body` かグローバル初期化子の
  `init` -- を `for_each_node` で歩いて挙げた、`Retain`/`Release` 節点の `(v, path)` と、`App` の各引数
  `arg` と各 `unit ∈ rc_units(ty(arg))` の対である。**」と定める。DEF site はこの文をその主語のまま
  写したものである。この文書が固定しているのは第 9.1 節の `V` であり、その `RewriteCtx` が `ctx`、
  その本体が `B_V` である。`V = f_borrow` の ときその本体は `clone_func` が返した `B_V` であり、`ctx` は `RewriteCtx::new(&clone, true, ..)` が
  作ったものである。`owns_unit` を呼ぶ位置がその版の site を出ないことは 第 1 節に写した `p15` の
  `L17` が述べる -- 関数の版については「`(v, u)` は**その版の本体**について `levelled_sites` が挙げる
  site である」として、グローバル初期化子の版については「`owns_unit(v, u)` は真を返す」として述べる。
  P7a はその 2 つの版を 1 つの site の定義で覆い、「関数の版ではこれは `levelled_sites` が挙げる集合と
  一致する」と述べる。
  BY DEF site, P7a, p15 の L17, CODE src/rc_ir/borrow.rs: borrow_ify, RewriteCtx::new

<1>1. `ctx.owns_unit(v, u)` が真ならば、`cand(v, λ)` のすべての元について `owns_object` は真である。
  P7a の節 1 から節 3 への含意である。`λ` は `Λ(u)` の inhabited な leaf である。`<1>0` より、読む
  P7a は `ctx` と `B_V` についてのものである。
  BY P7a, <1>0

<1>2. `ctx.owns_unit(v, u)` が偽ならば、`cand(v, λ)` に `owns_object` が偽である元がある。
  P7a の節 2 から節 1 への含意の対偶より、節 2 が偽である。節 2 は「`Λ(u)` の**ある inhabited な**
  leaf `λ` の**すべての**候補 `(r, p)` について `owns_object(r, p)` が真である」なので、その否定は
  「`Λ(u)` のどの inhabited な leaf にも `owns_object` が偽である候補がある」である。`<1>0` より、読む P7a は `ctx` と `B_V` についてのもので
  ある。
  BY P7a, <1>0

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
  <2>1. `func.body` の対応する節点が、対応する leaf `(w_0, μ)` を同じ行で消費する。ここで
        `w = ρ_f(w_0)` である。
    P9 の前半より `B_V` は `func.body` の束縛変数を `ρ_f` で一斉に付け替えたものであり、節点の種類・
    並び・`FieldPath`・変数の型を変えない。D9 の所有を読まない消費の 6 行のうち 5 行は節点の形と型だけで
    決まる。残る `Llvm` の行は `borrows_operand(i, arg_tys, type_env)` と
    `result_prov(result_ty, arg_tys, type_env)` を読み、この 2 つは `&self` を取るので、`rename_rhs` の
    `Llvm` の腕が作る `llvm_gen.clone()` は原本とは別のオブジェクトである。A3 は「`rhs.clone()` や
    `fresh_rename_function` が作る複製の op は、原本と同じ引数を渡されれば同じ `Provenance` を返す」と
    述べ、`borrows_operand` にも同じことを述べる。`rename_var` が型を残すので `arg_tys` と `result_ty` も
    両側で等しく、よって宣言は等しい。
    BY A3, D9, P9, CODE src/rc_ir/rename.rs: rename_rhs, rename_var,
       CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, LLVMGen::borrows_operand
  <2>2. `cand(w, μ) = ρ_f(cand_f(w_0, μ))` である。ここで `ρ_f` は `VarPath` に対しその変数だけを写す。
    第 1 節に写した `p15` の `L15` の (ii) は、`func` に現れる各名前 `x` と任意の path `π` について
    `origin(vars_c, type_env, ρ_f(x), π) = ρ_f(origin(vars_f, type_env, x, π))` であると述べる。
    `Origin::candidates` は `Exactly(p)` に `{p}`、`Join { candidates, .. }` に `candidates` を返すので、
    変数の付け替えと可換である。`<2>1` より `w = ρ_f(w_0)` であり、`w_0` は `func.body` に現れる名前で
    ある。
    BY <2>1, p15 の L15, CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>3. QED
    第 3.7 節の系より、`cand_f(w_0, μ)` の各元 `(r_0, p)` について `ctx.owns_object(ρ_f(r_0), p)` は
    真である。
    BY 3.7 の系, <2>1, <2>2

<1>3. QED
  L12 と L10 より、`cand(w, μ)` の全元が所有されることと `T_ρ(w, μ)` が所有されることは同値である。
  BY L10, L12, <1>1, <1>2

### 9.5 `App` の呼び出し先の所有

#### L18a (借用版の関数値はどこにも作られない)

**言明**。出力プログラムの本体において、借用版の名前 (`borrow_versions` の値) が現れるのは
`RcRhs::App` の callee の位置だけである。よって借用版の関数値はプログラムのどこにも作られず、
環境 (D22) が持つ値にも、局所変数の値にも、オブジェクトの欄が保持する値にも、グローバル値の記憶域が
保持する値にも入らない。したがって借用版の本体の活性化を作る段は (E3) の呼び出しの段だけである。

<1>1. `rewrite_inner` が callee を差し替えるのは `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕の
      `self.route(x, callee, args, k)` だけであり、他の腕は `rhs` と変数をそのまま写す。とくに
      `RcRhs::Closure(fref, caps)` を持つ `Let` は `rhs.clone()` で写されるので、`fref` は入力の関数の
      名前のままである。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

<1>2. `borrow_ify` が借用版の名前を書き込む先は、`borrow_versions` の値、`clones` の第 1 成分、
      `callee_params` の鍵、出力の `funcs` の鍵、`clone.name` である。本体の中へ書くのは `route` を
      通した `<1>1` の位置だけである。出力の本体の残りの位置が借用版の名前を持たないことは、書き換えが
      受け取る本体 -- `f_own` とグローバル初期化子については入力の本体、借用版については `clone_func` が
      返した本体 -- の名前がどれも借用版の名前でないことから出る。`borrow_funcref` は借用版の名前を
      `<元の名前>#borrow` として作るのに対し、A13 は入力に現れるすべての名前について `name` フィールドを
      `#` で区切った最後の断片が `borrow` でないと述べ、4.2 の言明より複製が導入する名前の最後の断片は
      `b` の後に 10 進数字が続く形である。
  BY A13, 4.2 の言明, <1>1, CODE src/rc_ir/borrow.rs: borrow_ify, borrow_funcref, clone_func,
     RewriteCtx::route

<1>3. 出力の `roots` は入力の `roots` そのものである。
  `borrow_ify` は `RcProgram { funcs, globals, roots: prog.roots.clone() }` を返す。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>4. コード生成が Fix の関数の値を作るのは 3 か所であり、そのうち借用版の値を作りうるものは無い。
  <2>1. `RcRhs::Closure(fref, caps)` について `build_rc_closure` が `func_vals[fref]` の関数ポインタを
        持つクロージャを作る。`<1>1` より `fref` は入力の関数の名前であり、`<1>2` よりそれは借用版の
        名前ではない。
    BY <1>1, <1>2, CODE src/rc_ir/codegen.rs: Generator::build_rc_closure,
       Generator::eval_rc_expr_inner
  <2>2. `RcExpr::Let(x, RcRhs::App(callee, args), k)` について `get_scoped_obj(&callee.name)` が返す値を
        `apply_lambda` に渡す。この値は束縛されず、オブジェクトにも格納されない。
    BY CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
       CODE src/generator.rs: Generator::get_scoped_obj, ValueAccessor::get, Generator::apply_lambda
  <2>3. `InlineLLVMFixBody` は `gc.current_function()` の関数ポインタを持つクロージャを作り、それを
        オペランドに適用する。`gc.current_function()` はいま本体を生成中の関数であり、それは借用版では
        ない。
    <3>1. 借用版 `f_borrow` の本体に `InlineLLVMFixBody` の `Llvm` 節点が在るならば、その原本
          `func` の本体にも在る。
      P9 の前半より `clone_func` の出力は `func.body` の各節点を同じ種類の節点に写し、`RcRhs::Llvm`
      については `llvm_gen` を clone して `free_vars_mut` の名前を写すだけで、op の種類を変えない。
      その後の `ctx.rewrite` は `RcRhs::Llvm` を右辺に持つ `Let` を `rhs.clone()` で写す
      (`rewrite_inner` の `Let` の 3 番目の腕)。
      BY P9, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/rename.rs: rename_rhs
    <3>2. QED
      `borrow_ify` の 1 番目のループが `borrow_versions` に元を入れるのは `func.capture.is_none()` で
      ある関数についてだけである。`<3>1` とこの文書が置く事実 A24 より、その op を
      本体に持つ借用版が在れば `func.capture` は `Some` であり、これに反する。よってこの op を本体に
      持つ関数は借用版ではなく、その本体を生成中の `gc.current_function()` も借用版ではない。
      BY A24, <3>1, CODE src/fixstd/builtin.rs: InlineLLVMFixBody,
         CODE src/rc_ir/borrow.rs: borrow_ify
  <2>4. QED
    A21 は「Fix の関数型の値に LLVM 関数の番地を書き込むのは、クロージャを作る段 (`build_rc_closure`)、
    funptr のグローバルを読む段 (`ValueAccessor::get` の `is_funptr` の枝)、そして `InlineLLVMFixBody` の
    3 か所だけである。ほかのどの構文も op も、既にある関数の値を写すだけである」と述べる。その 3 か所を
    `<2>1`・`<2>2`・`<2>3` が順に扱った -- `<2>2` の `get_scoped_obj` が通るのが `ValueAccessor::get` で
    あり、`<1>2` より本体が借用版の名前を引くのはその位置だけである。
    BY A21, <1>2, <2>1, <2>2, <2>3

<1>5. QED
  `<1>4` より借用版の関数値はプログラムのどこにも作られない。よってそれは環境 (D22) が持つ値にも、
  局所変数の値にも、オブジェクトの欄が保持する値にもならない -- オブジェクトの欄が保持するのは
  プログラムが作った値だからである (A4 -- 段がオブジェクトの記憶域へ書き込む内容はその段の節点と
  オペランドの値と D21 の 4 種の結果だけで決まる)。**グローバル値の記憶域が保持する値も同じである** --
  D24 の (E7) より、アクセサが記憶域へ格納するのはその初期化子の活性化が終端の `Ret` で返した値であり、
  それはその本体がプログラムの構文で作った値だからである。D24 の「活性化の林」より活性化を作る段は 5 種で
  尽きる。(E1) は環境が持つ値から活性化を作り、`<1>3` より環境が読む `roots` も変わらない。
  (E3) の呼び出しの段は callee の値が呼び出し先を決め (D23)、callee が局所変数であればその値は
  `<1>4` より借用版の関数値ではない。(E2) のオペランドを適用する `Llvm` の段が適用するのは、その op の
  オペランドの値か、その段が組み立てた値である (D24 の (E2))。どちらもプログラムが持つ関数値なので、
  `<1>4` より借用版の関数値ではない。(E7) はグローバル初期化子の活性化しか作らない。
  **(F) の解放が `Destructor` について作る段**が作る活性化は 2 つであり、D24 の「活性化の林」は
  「`_dtor` の欄の関数を `_value` の欄の値へ適用するものと、それが返した `IO` の動作の runner を適用
  するものであり、2 つ目の入力は 1 つ目の結果である」と述べる。1 つ目が適用するのはオブジェクトの欄が
  保持する Fix の関数型の値であり、2 つ目が適用するのは 1 つ目の活性化が返した `IO` の動作の runner、
  すなわちプログラムが作った Fix の関数型の値である。`<1>4` よりどちらも借用版の関数値ではない。
  よって借用版の本体の活性化を作るのは、その名前を callee に持つ `App` の段 (E3) だけである。
  BY A4, D21, D22, D23, D24, <1>3, <1>4

#### L15

**言明**。`B'_V` の節点 `Let(x, App(callee', args), k)` について、この活性化がこの位置で作る活性化の本体を
持つ関数 (D9 と D10 が「呼び出し先」と呼ぶもの) を `W` とする。各引数の添字 `i` と各
`u ∈ units(ty(args[i]))` について、`W` が `(W のパラメータ i, u)` を D14 の意味で所有することと、
P11 の `callee_owns(i, u)` が真であることとは同値である。

<1>0. `W` は第 `i` パラメータを持つ。
  A14 は `App(callee, args)` の `args` の個数を呼び出し先のパラメータの個数で抑える。D23 より `W` は
  この段の呼び出し先である。`i` は `args` の添字なので `i < |W.params|` である。
  BY A14, D23

<1>1. CASE `callee'.name` が `ctx.callee_params` の鍵である。
  <2>1. `W` は出力の `funcs` が `callee'.name` を鍵として持つ関数である。
    <3>1. `resolve_callee_params(callee', VarTable::of(V), 出力プログラム)` は
          `Some(出力の funcs[callee'.name].params)` を返す。
      `resolve_callee_params` は `vars.closure_targets` を `callee'.name` で引き、外れたときは
      `FuncRef { name: callee'.name }` が `prog.funcs` の鍵かを見る。`closure_targets` に元を入れるのは
      `collect_bindings` の `RcRhs::Closure` の腕だけで、鍵はその本体の `Let` の束縛変数の名前 --
      すなわち出力の束縛名 (DEF 出力の束縛名) -- である。L6 より `callee_params` の鍵は出力の `funcs` の
      鍵ちょうどであり、4.4 の系 より出力の束縛名は出力の `funcs` の鍵ではないので、`callee'.name` は
      `closure_targets` の鍵ではない。よって第 2 の枝が当たり、`callee'.name` は出力の `funcs` の鍵なので
      `Some` が返る。
      BY L6, 4.4 の系, DEF 出力の束縛名, CODE src/rc_ir/ownership.rs: resolve_callee_params,
         VarTable::of, collect_bindings
    <3>2. QED
      P30 より、`borrow_ify` の出力の `App` について `resolve_callee_params` が解決する関数が
      `Some` であるならば、それはその段の実行時の呼び出し先 (D23) と同じ `RcFunc` である。`<3>1` より
      それは出力の `funcs[callee'.name]` である。
      BY D23, P30, <3>1
  <2>2. `callee_params[callee'.name][i].0` は `W` の第 `i` パラメータの名前である。
    `callee_params` は入力の各関数について `param_names_and_types(func)`、各借用版について
    `param_names_and_types(clone)` を、その版の名前を鍵に入れる。L6 と `<2>1` よりこの 2 種が出力の
    `funcs` の元を尽くすので、`callee_params[callee'.name]` は `param_names_and_types(W)` である。
    `param_names_and_types` は `W.params` に `W.capture` を鎖にした列であり、`<1>0` より
    `i < |W.params|` なので、第 `i` 元は `W` の第 `i` パラメータである。
    BY L6, <1>0, <2>1, CODE src/rc_ir/borrow.rs: borrow_ify, param_names_and_types
  <2>3. QED
    P11 の `callee_owns(i, u)` は `owned_units.contains(&(callee_params[..][i].0, u))` である。
    `u ∈ units(ty(args[i]))` であり、A12 より `App` の各引数と呼び出し先の対応するパラメータの型は
    等しいので、`u` は `W` の第 `i` パラメータの型の unit でもある。第 8 節の系 3 より、これは `W` が
    `(その第 i パラメータ, u)` を D14 の意味で所有することと同値である。
    BY A12, 第 8 節の系 3, <2>1, <2>2

<1>2. CASE `callee'.name` が `ctx.callee_params` の鍵でない。
  <2>1. `callee_owns(i, u)` は真である。
    `call_rc` は `params` が `None` のとき `true` を使う。
    BY P11, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc
  <2>2. `W` は全所有版 `f_own` である。
    <3>1. この呼び出しの呼び出し先は、`callee'` の値が決める。
      L6 より `callee_params` の鍵でない名前は、入力の関数の名前でも借用版の名前でもない。よって
      `callee'` は出力のどの関数も名指さず、局所変数かグローバル値を読む atom であり、その値は
      プログラムが作った関数値である。D23 より呼び出し先はその値が決める -- 値がクロージャならその
      funptr が指す関数、funptr ならそれ自身である。
      BY D23, L6
    <3>2. `W` は借用版ではない。
      `<3>1` より `callee'` は局所変数かグローバル値を読む atom であり、その値は局所変数の値か
      グローバル値の記憶域が保持する値である。L18a より、借用版の関数値はプログラムのどこにも作られない
      ので、その 2 つのどちらにもならない。
      BY L18a, <3>1
    <3>3. QED
      D23 は「D9 の `App` の行が読む所有は D14 が `RcFunc::borrowed_units` から定めるものなので、
      **その呼び出し先はプログラムの `funcs` の関数である**」と述べる。よって `W` の名前は出力の `funcs` の鍵で
      あり、L6 よりそれは入力の関数の名前か借用版の名前で、`<3>2` より後者ではないので、`W` は `f_own`
      である。
      BY D23, L6, <3>1, <3>2
  <2>3. QED
    第 8 節の系 1 より `f_own` の `borrowed_units` は空なので、D14 より `f_own` は全パラメータの全 unit を
    所有する。`<1>0` より `W` は第 `i` パラメータを持つので、`(その第 i パラメータ, u)` も所有する。
    BY D14, 第 8 節の系 1, <1>0, <2>1, <2>2

<1>3. QED
  BY <1>0, <1>1, <1>2

#### L18 (振り分けられる呼び出し先は boxed leaf を持たない)

**言明**。`B_V` の節点 `Let(x, App(c, args), k)` について、`ctx.route(x, c, args, k)` が `c` と異なる名前の
`RcVar` を返すならば `leaves(ty(c)) = ∅` である。`route` が `c` をそのまま返すときは、`B'_V` の対応する
節点の callee は `c` と同じ `RcVar` である。

<1>1. `route` が異なる名前を返すのは、`borrow_versions` が `FuncRef { name: c.name }` を鍵に持つときだけで
      ある。
  BY P12 (a)

<1>2. `borrow_versions` の鍵は `prog.funcs` の鍵である。
  BY L6

<1>3. `borrow_versions` の鍵 `FuncRef { name: c.name }` に対応する `prog.funcs` の関数を `g` とすると、
      `g.capture` は `None` である。
  `borrow_ify` の 1 番目のループは `func.capture.is_none()` である関数についてだけ `borrow_versions` に
  元を入れる。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>3a. `borrow_ify` の入力の `prog.funcs` の鍵は 2 種である。`sym.ty.is_funptr()` を満たす記号 `sym` の
       名前 `sym.name` と、`Lowerer::lower_lam` が持ち上げた lambda に `fresh_closure_ref` で付ける
       名前である。
  <2>1. `lower_program` は、`lower_symbol` が `LoweredSymbol::Func(f)` を返した記号について `f.name` を
        鍵に入れる。`lower_symbol` が `Func` を返すのは `sym.ty.is_funptr()` のときだけであり、その
        `f.name` は `sym.name` である。`sym.ty.is_funptr()` が偽の記号は `LoweredSymbol::Global` に
        なって `globals` へ行く。
    BY CODE src/rc_ir/lower.rs: lower_program, Lowerer::lower_symbol,
       CODE src/ast/types.rs: TypeNode::is_funptr
  <2>2. `Lowerer::lower_lam` は `fresh_closure_ref()` が作る名前を鍵に入れる。`Lowerer` が `funcs` に
        鍵を入れるのはこの 2 か所だけである。この鍵の関数を作るのは `lower_lambda_as_function` であり、
        その `fn_ty` は `Expr::Lam` の節点が持つ型 `lam.type_` -- 以下 `lam_ty` -- である。その `capture`
        は `lam_ty.is_closure()` が真のときだけ `Some` である (偽のときは `captures` が空であることを
        表明して `None` を入れる)。
    BY CODE src/rc_ir/lower.rs: lower_program, Lowerer::lower_lam,
       Lowerer::fresh_closure_ref, Lowerer::lower_lambda_as_function,
       CODE src/ast/types.rs: TypeNode::is_closure
  <2>3. `lower_program` の出力から `borrow_ify` の入力までの間に、`funcs` に鍵を足すパスは無い。
        `simplify` は `prog.funcs.values_mut()` の本体を書き換え、`insert_rc` は同じ鍵で `Map` を
        組み直し、`split_rc_units` は `prog.funcs.values_mut()` の本体を書き換える。関数を複製する
        2 つのパス (`unique_check_elim::specialize` と `locality::specialize`) は
        `optimize_rc_program` の中で `borrow_ify` より後に走る。
    BY CODE src/build/build_object_files.rs: lower_and_insert_rc, optimize_rc_program,
       CODE src/rc_ir/simplify.rs: simplify, CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/borrow.rs: split_rc_units
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>4. `ty(c)` は funptr 型である。
  <2>1. `c.name` は `ctx.vars.bindings` の鍵ではない。よって D6 より `c` は束縛を持たない `RcVar` で
        あり、`c.name` は最上位の記号の名前である。
    `<1>1` と `<1>2` より `c.name` は入力の関数の名前である。`ctx.vars.bindings` の鍵は、`V` の
    パラメータ・capture の名前と `B_V` が束縛する変数の名前ちょうどである (`VarTable::of` が前者を、
    `collect_bindings` が後者を入れ、グローバル初期化子の `VarTable::body_only` は後者だけを入れる)。
    `V` が `f_own` かグローバル初期化子であるとき、`B_V` は入力の本体なのでそれらはどれも入力の束縛名
    であり、A6 より入力のすべての束縛変数の名前はどの関数の名前とも異なる。`V = f_borrow` であるとき、
    それらはどれも `clone_func` が導入した名前である -- 4.1 の言明より `rename_f` の定義域は `func` の
    パラメータ・capture の名前と `func.body` が束縛する名前の全体であり、`fresh_rename_function` は
    `clone` の `params` と `capture` を `rename_var` でその像に写し、`B_V` の各束縛子も同じ写像で写る。
    4.2 の言明よりその `name` フィールドを `#` で区切った最後の断片は `b` の後に 10 進数字が続く形で
    あり、A13 より入力に現れるどの名前 -- `prog.funcs` の鍵を含む -- の最後の断片もその形ではないので、
    それらは `c.name` と異なる。D6 は、`vars.bindings` に束縛を持たない名前は最上位の記号の名前であり、
    lowering がそのような `RcVar` を作るのは `Lowerer::lower_var` と `Lowerer::lower_llvm` の
    `resolve` が `None` を返す 2 つの腕だけであると述べる。
    BY A6, A13, D6, <1>1, <1>2, 4.1 の言明, 4.2 の言明,
       CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only, collect_bindings,
       CODE src/rc_ir/rename.rs: fresh_rename_function, rename_var,
       CODE src/rc_ir/lower.rs: Lowerer::lower_var
  <2>2. `c.name` は記号の名前であり、その記号の型は funptr 型である。
    `<2>1` より `c` は束縛を持たない `RcVar` であり、A12 の「束縛を持たない `RcVar` の型が、その名前の
    記号の型であること」より `c.name` の記号が在る。`<1>1` と `<1>2` より `c.name` は `borrow_ify` の
    入力の `prog.funcs` の鍵であり、`<1>3a` よりその鍵は 2 種のどちらかである。後者であるとすると、
    A23 よりその関数の `fn_ty` は closure 型であり、`<1>3a` より
    その `capture` は `Some` である。これは `<1>3` に反する。よって `c.name` は `sym.ty.is_funptr()` を
    満たす記号 `sym` の名前である。
    BY A12, A23, <1>1, <1>2, <1>3, <1>3a, <2>1
  <2>3. QED
    A12 の「束縛を持たない `RcVar` の型が、その名前の記号の型であること」より `ty(c)` は `c.name` の
    記号の型であり、`<2>2` よりそれは funptr 型である。
    BY A12, <2>1, <2>2

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
**`Llvm` を右辺に持つ `Let` については、両側の op は別のオブジェクトでありながら、同じオペランドの列と
同じ型を持ち、`borrows_operand` と `result_prov` に同じ値を返す。**
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
      アームの数・`tag`・`payload` を変えずに各アーム本体を書き換える。**`Llvm` を右辺に持つ `Let` は
      3 番目の腕が `rhs.clone()` で写すので、出力の op は入力とは別のオブジェクトである。** それでも
      節点が「等しい」と言えるのは A3 による -- A3 は「`rhs.clone()` や `fresh_rename_function` が作る
      複製の op は、原本と同じ引数を渡されれば同じ `Provenance` を返す」と述べ、`borrows_operand` にも
      同じことを述べる。`rhs.clone()` はオペランドの列も型も変えないので、D9 の消費の表の `Llvm` の行が
      読む `borrows_operand(i, arg_tys, type_env)` と、D10 の生成の表の `Llvm` の行が読む
      `result_prov(result_ty, arg_tys, type_env)` は両側で同じ値を返す。
  BY A3, D9, D10, P10, P11, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
     CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, LLVMGen::borrows_operand

<1>2. `rewrite_rc` が作る節点は、`V` が借用版でないとき元の節点そのもの、借用版のとき
      `owns_unit(v, π)` が真ならば元の節点そのもの、偽ならば節点無しである。
  `B_V` の `Retain(v, π)`/`Release(v, π)` 節点の path は、その変数の型の unit である。`V` が `f_own` か
  グローバル初期化子のとき `B_V` は `borrow_ify` の入力の本体なので A2 がそれを与える。`V` が
  `f_borrow` のとき `B_V` は入力の関数の本体を `ρ_f` で一斉に付け替えたものであり、P9 の前半より複製は
  `FieldPath` を変えず `rename_var` は `ty` を残すので、複製の節点の path もその変数の型の unit である。
  L3 がこの場合の `rewrite_rc` の値を与える。
  BY A2, L3, P9, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc,
     CODE src/rc_ir/rename.rs: rename_var

<1>3. QED
  `<1>1` と `<1>2` より、`B'_V` は `B_V` の木から `Retain`/`Release` を落とし、`App` の前後に
  `Retain`/`Release` を足し、`App` の `callee` を `route` の値に差し替えたものである。`Match` のアームの
  構造 (D3 が実行路を作るのに使う唯一の分岐) は変わらないので、実行路は 1 対 1 に対応する。`route` が
  返す `RcVar` は `callee.clone()` か、その複製の `name` を差し替えたものなので `ty` は等しい (P12 (a))。
  `Llvm` を右辺に持つ `Let` についての節は `<1>1` が与える。
  BY D3, P10, P11, P12, <1>1, <1>2

#### L16a (出力の活性化に対応する入力の活性化)

**言明**。`B'_V` の活性化 `α'` を 1 つ取り、`α'` が辿る `B'_V` の実行路に L16 で対応する `B_V` の実行路を
`ρ` とする。このとき `B_V` の活性化 `α` であって、`ρ` を辿り、`α'` とパラメータ・capture の値を共有し、
D21 が挙げる「オペランドから結果が決まらない 4 種」の各位置での結果を **L16 が対応させる位置について**
`α'` と共有するものが在る。とくに対応する位置で対応する変数が得る値は等しい。

**対応の付かない位置がある。** D21 の 4 種のうち「子の活性化を作る段」には (F) の解放が入り、D21 は
「**(F) はどの構文でも起こりうる** -- 参照を処分する段はどれもオブジェクトの解放を起こしうるので、
`Release` の節点も、消費を行う `App` や `Destructure` の節点も、この意味で外から与えられる量を持つ」と
述べる。L16 は `Retain`/`Release` 節点については両側の対応を主張しない -- (K) で残る節点は対応するが、
借用版が落とす節点は `B_V` にだけ在り、(A-前) と (A-後) は `B'_V` にだけ在る。片側にしか無い位置の
(F) の結果は、その側の活性化のデータであって、対応の与件ではない。`α` の側にしか無い位置の結果は
任意に選んでよく、どう選んでも下の `<1>6` の構成は同じ実行路と同じ値を与える。**したがって言明は
「在る」であって「ちょうど 1 つ在る」ではない。** 以下ではその 1 つを取って固定する。

<1>1. D21 より、1 つの本体の活性化は、パラメータ・capture の値と、オペランドから結果が決まらない 4 種の
      各位置での結果を与えると 1 つに決まり、辿る実行路もそれで決まる。
  BY D21

<1>2. `B_V` と `B'_V` の `Retain`/`Release` 以外の節点は 1 対 1 に対応し、対応する節点は種類・変数・
      `FieldPath`・`Match` のアームの `tag` と `payload` を共有する。`Let(x, App(callee, args), k)` に
      ついては `x`・`args`・継続を共有し、`callee` の名前だけが違う。
  BY L16

<1>3. D21 の 4 種の位置のうち、`Retain`/`Release` 節点の上に無いものは `B_V` と `B'_V` で 1 対 1 に
      対応する。`Retain`/`Release` 節点の上にありうるのは (F) の解放と、その節点が束縛を持たない名前を
      名指すときの (E7) の段の 2 つである。後者は `B_V` の側の節点については両側で 1 対 1 に対応し、
      片側にしか位置が無いのは (F) の解放と、`B'_V` にだけ在る (A-前)・(A-後) の節点の上の位置である。
  4 種は、一意性の観測点 (D18)、外部の状態を読む `Llvm` の演算、**実行時の参照カウントで分岐する
  `Llvm` の演算**、子の活性化を作る段である。第 3 の種を宣言で読まないのは D21 と D30 による -- D21 は
  「分岐する op と `LLVMGen::unique_check_operand` を宣言する op は一致しない (D30 の (X2))」と書き、
  D30 の (X2) は「数え上げるのは生成コードの分岐であって、`LLVMGen::unique_check_operand` の宣言では
  ない」と書く。この段が要るのはその位置が `RcRhs::Llvm` の節点であることだけなので、どちらの読みでも
  結論は同じである。4 種目は D24 の「活性化の林」より `App` の段 (E3)、
  オペランドを適用する `Llvm` の段 (E2)、グローバルの初期化の段 (E7)、そして `Destructor` の
  オブジェクトを解放する段 (F) の 4 つである。前の 3 種と、4 種目のうち `Llvm` の段は `RcRhs::Llvm` の
  節点であり、L16 の言明より両側の op は同じオペランドの列と同じ型を持ち、`borrows_operand` と
  `result_prov` に同じ値を返す。`App` の段は `<1>2` より
  対応する。グローバルの初期化の段は、グローバルを読む節点に対応し、その節点も `<1>2` の対応に入る。
  **(F) の解放は参照を処分するどの段でも起こりうる** (D21) ので、その位置は `Retain`/`Release` 節点の
  上にも在る。`<1>2` が対応させない節点はその 2 種だけなので、それ以外の節点の上の (F) の位置は
  1 対 1 に対応する。

  **`Retain`/`Release` 節点の上には (E7) の段も在りうる。** その節点のコード生成は
  `get_scoped_obj_noretain` を呼び、`ValueAccessor::get` の `Global` の枝は getter を `build_call` する
  (`CODE src/generator.rs: Generator::get_scoped_obj_noretain`, `get_scoped_value`,
  `ValueAccessor::get`)。よってその節点が束縛を持たない名前 (D6 の記号の位置) を名指すとき、その段は
  まだ初期化されていない記号について (E7) を起こす。**そのような名前を名指す `B_V` の `Retain`/`Release`
  節点は `B'_V` に残る。** `g` を束縛を持たない名前とすると、L6c より
  `origin(g, π) = Origin::Exactly((g, π))` なので `cand(g, π) = {(g, π)}` である。`VarTable::of` と
  `VarTable::body_only` が `param_tys` に入れる鍵はパラメータ・capture の名前だけなので、`g` はその鍵では
  なく、L4 より `ctx.owns_object(g, π)` は真である。`owns_unit(g, π)` は `cand(g, π)` の全元についての
  `owns_object` の全称なので真であり、L16 の (K) よりその節点は `B'_V` に同じ変数・同じ path・同じ位置で
  立つ。よってその節点の上の (E7) の位置は両側で 1 対 1 に対応する。残る (A-前) と (A-後) は `B'_V` に
  だけ在る節点であり、その上の (E7) の位置は片側にしかない。
  BY D6, D18, D21, D24, D30, L4, L6c, L16, <1>2,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
     CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only,
     CODE src/generator.rs: Generator::get_scoped_obj_noretain, get_scoped_value,
     ValueAccessor::get

<1>4. `callee` の名前が違うことは、この対応を妨げない。
  D21 は `App` の段の結果 -- 返る値と参照カウントに与える変化 -- を活性化の側のデータとして与えるので、
  `α` の側でその位置に与える結果は `α'` の側と同じものを取れる。呼び出し先の本体が両側で違うことは、
  この対応の主張ではない。
  BY D21

<1>5. `Retain`/`Release` 節点の増減は、この構成が与えるデータを変えない。これらは変数を束縛せず
      (L5)、`<1>3` より D21 の 4 種のうちその上に在りうるのは (F) の解放と (E7) の段の 2 つである。
      片側にしか無い節点の上のその結果は、その側の活性化のデータであり、`α` の側にしか無い位置に
      ついては任意に選ぶ。`<1>1` が活性化を決めるのに使うのは与えたデータの全体であり、選び方に
      よらず、その他の位置に与えたデータは同じである。
      **(E7) の結果は記号の値なので、選び方を 1 つ指定する。** (E7) の段の結果 -- アクセサが記号の
      記憶域へ格納する値 -- は、その記号を読む後の節点が読むものである (D24 の (E7))。`<1>3` より
      その位置が片側にしか無いのは `B'_V` にだけ在る (A-前)・(A-後) の節点の上であり、`α` の構成は
      その結果を使わない。`B_V` で束縛を持たない名前を名指す節点は、`Retain`/`Release` 節点であれば
      `<1>3` より `B'_V` に残って両側で対応し、そうでなければ `<1>2` の対応に入る。`α` の側でその節点に
      与える (E7) の結果は、`α'` におけるその記号の値を取る。
  BY D6, D21, D24, L5, <1>2, <1>3

<1>6. QED
  `<1>3`-`<1>5` の対応で与えたデータ (対応の付かない位置については任意に選んだ結果) を `<1>1` に渡すと
  `B_V` の活性化が 1 つ決まる。辿る実行路が `ρ` であることは、D21 より `Match` のアームの選択が
  scrutinee の値の実行時のタグで決まり、その値が `<1>1` のデータから両側で同じに決まることによる。
  **選び方が各位置の値を変えないのは、値がオペランドから決まらない位置が D21 の 4 種で尽きているから
  である。** その 4 種のうち対応の付く位置の結果は `α'` と共有し、`Retain`/`Release` 節点は変数を
  束縛しない (L5) ので、値を作る節点は `<1>2` の対応で 1 対 1 に並び、各節点の値はオペランドの値か
  共有した結果のどちらかで決まる。よって対応する位置で対応する変数が得る値は `α'` のものと等しい。
  BY D21, L5, L16, <1>1, <1>2, <1>3, <1>4, <1>5

**`Retain`/`Release` 節点が記号の位置を名指す形は在る**。`insert_rc` が `Release` を置く位置のうち、
`v.name.is_local()` の門の下に無いものが在る -- `insert_into_match` の
`release_container && arm.tag.is_some() && needs_rc(&scrut)` の枝が boxed union の変位アームの頭へ置く
`Release(scrut, ..)` がそれで、`scrut` に `is_local()` の検査は無い
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match`)。`README.md` の A19 の脇が
「boxed union の変位アームの頭に置かれた `Release`」と名指すのはこの節点である。よって `<1>3` はこの形を
数え上げで退けず、そのような節点が `B'_V` に残ることで扱う。

### 9.7 由来ごとの義務

**DEF 時点** -- 本体 `C` (`B_V` か `B'_V`) の 1 回の活性化の**事象**とは、参照を 1 つ作る D10 の生成と
`Retain`、参照を 1 つ処分する `Release` と D9 の消費である。**1 つの節点は leaf ごとに事象を行う** --
`Retain(v, π)` と `Release(v, π)` は `π` の下の inhabited な各 leaf につき 1 つ、消費は消費される
inhabited な各 leaf につき 1 つ、生成は生じた inhabited な各 leaf につき 1 つである (D10)。
事象は、その節点の実行のあいだに時間の順に並ぶ。その順が生成コードのどの位置に当たるかは `L29` (a) が
述べる。D9 (移動の表の下の注) より、参照を作る・移す・手放す構文はこれで尽きる。

この活性化の**時点**とは、活性化の開始 (D10 の初期値が置かれた直後) と、各事象の直後の点と、
**D7 の読む構文が行う各読みの直前の点**である。**事象の直前の時点**とは、その事象より前にある最後の
時点をいう。節点の入口は、その節点より前の最後の事象の直後の時点と読み、最後の時点は終端の `Ret` の
消費の直後である。

**読みの直前の点を時点に数えるのは、D11 の (S-c) がその点で条件を課すからである。** 第 10.7 節は
その点で L17 と第 10.4 節を読む。`Retain(v, π)` と `Release(v, π)` が leaf `λ` のオブジェクトに触れる
動作は、D10 のその 2 行が参照を 1 つ加え・取り除くのと同時に `H(obj(v, λ))` を 1 動かすので、その動作の
直前の点は事象の直前の時点である。

**読みの点で動くものは無い。** 読みは事象ではないので、読みの直前の点では `n^ι_C` も義務集合も動かず、
その値はその点より前の最後の事象の直後の値に等しい。よって「各時点について」の形の言明 -- L17、
第 10.3 節の INV、第 10.4 節 -- は、この点を足しても事象の列の上の同じ帰納で立つ。

呼び出しは D9 と D10 では `App` の節点の 1 組の事象 -- 引数と callee の消費、結果の生成 -- として現れる
ので、呼び出し先の活性化が動いている間にこの活性化の時点は無い。D21 より、`App` が返す値と参照カウントに
与える変化は、1 つの本体を見ている間は活性化の側のデータであって、この本体の事象ではない。

`App` の節点が行う読みの直前の点で、勘定がその節点の入口のものであることは `L29` (b) が述べる。
以下、この文書はその読みの点をその節点の入口として扱う。

**その読み方は D11 の (S-c) が要求する。** 呼び出し先が引数を読む瞬間をこの活性化の読みの点に数えると、
所有する引数を渡すどの本体でも (S-c) の結論が立たない -- その参照は呼び出しでその類を離れる (D9 の
`App` の行) ので、呼び出し先が読む時点でその類の `held` は 0 でありうる。`App` の節点がその類を名指す
のは、それを渡す点までであり、A19 (ii-a) の (b) が 1 以上を言うのはその節点の入口である。

**DEF 由来ごとの義務** -- 本体 `C` (`B_V` か `B'_V`) と、`V` のパラメータ・capture の leaf に 0 か 1 を
与える**初期値の規則** `ι` について、`ρ` に対応する `C` の実行路の上の時点 `τ` (DEF 時点) と `ρ`-由来
`T` に対する
整数 `n^ι_C(τ, T)` を次で定める。ただし `obj(T)` が計数下 (D26) でない `T` については定めない (D26 より
そのような `T` を指すスロットは D8 の意味の参照を持たない)。

- `T = (p, σ)` で `p` が `V` のパラメータか capture であるとき、初期値は `ι(p, σ)` である。
- それ以外の `T = (u, σ)` は、D10 の生成の表のいずれかの行が `u` に値を与える事象で 1 になる。その前は
  0 である。`u` が `ctx.vars.bindings` の鍵でないとき (`u` はグローバル値の名前である) は、D26 より
  `obj(u, σ)` はグローバル状態なので、この由来は勘定の外にある。
- `Retain(v, π)` の節点の leaf `λ` の事象が、`n^ι_C(・, T_ρ(v, λ))` を 1 増やす。
- `Release(v, π)` の節点の leaf `λ` の事象が、同じものを 1 減らす。
- D9 の消費の leaf `(w, μ)` の事象が、`n^ι_C(・, T_ρ(w, μ))` を 1 減らす。
- 他のどの事象も `n^ι_C` を変えない。

2 つの規則を使う。`ι_全` はすべての leaf に 1 を与える規則、`ι_V` は `ctx.owns_object(p, σ)` が真である
leaf に 1、偽である leaf に 0 を与える規則である。

**DEF n_in、n_out** -- `n_in := n^{ι_全}_{B_V}`、`n_out := n^{ι_V}_{B'_V}` と書く。

#### L29 (節点の事象の位置と、`App` の節点の読みの点)

**言明**。次の 2 つが成り立つ。

- **(a)** 1 つの節点の事象 (DEF 時点) の時間の順は、その節点についてコード生成が出すコードがそれらを
  行う順である。参照を処分する消費と `Release` の事象はその release を出す位置に、渡す先のある消費
  (D10) の事象はその値を渡す・書き込む命令の位置に、生成と `Retain` の事象はその参照を作る命令の位置に
  ある。
- **(b)** `Let(x, App(callee, args), k)` の節点が行う D7 の意味の読みの直前の点と、その節点の入口
  (DEF 時点) とのあいだに、この活性化の事象は 1 つも無い。したがって DEF 時点 の「読みの点で動くものは
  無い」より、その 2 点で `n^ι_C` と義務集合は等しい。

<1>1. (a) が成り立つ。
  DEF 時点 の事象は、D10 の `Retain` と `Release` の 2 行、D10 の生成の表、D9 の消費の表が挙げる
  leaf ごとの参照の増減である。A4 より、コード生成は `Retain(v, π)` / `Release(v, π)` を `π` の下の
  inhabited な各 boxed leaf の参照カウントの ±1 として実装し、`Destructure` と `Match` の変位アームに
  ついては D9 の消費・移動の表と D10 の生成の表のとおりに実装する。A3 より、`Llvm` の op の
  `result_prov` と `borrows_operand` はその op が生成するコードを正しく述べる。
  `Generator::eval_rc_expr_inner` が節点の種類ごとにその列を出す。事象はその生成コードが行う動作そのもの
  なので、事象の時間の順はコードがそれらを行う順であり、各事象の位置は言明が挙げるとおりである。
  BY A3, A4, D9, D10, DEF 時点, CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner

<1>2. この活性化が `App` の節点で行う読みは、callee のオブジェクトの読みだけである。
  D7 の読む構文の表は `Let(x, App(callee, args), k)` について callee と各引数を挙げる。引数の値はこの
  活性化が既に持っている番地としてそのまま渡され、この活性化はその指す先を読まない
  (`CODE src/generator.rs: Generator::apply_lambda`)。引数のオブジェクトを読むのは呼び出し先の本体の
  読む構文であり、D32 の (読み-1) がその読みをそちらに数える。
  BY D7, D32, CODE src/generator.rs: Generator::apply_lambda

<1>3. その読みは、この節点のどの事象よりも前に起きる。
  `apply_lambda` が callee のオブジェクトを読むのは `get_lambda_func_ptr` が関数ポインタを取り出すときと
  capture の欄を取り出すときであり、どちらも `build_indirect_call` より前にある。この節点の事象は
  D9 の `App` の行の消費と D10 の生成の `App` の行であり、`<1>1` より前者は値を渡す命令の位置、すなわち
  `build_indirect_call` が引数と callee を渡す位置にあり、後者は呼び出しが返った後にある。
  BY D9, D10, <1>1, <1>2, CODE src/generator.rs: Generator::apply_lambda

<1>4. QED
  DEF 時点 は「節点の入口は、その節点より前の最後の事象の直後の時点と読み」と定めるので、その入口と
  この節点の最初の事象のあいだにこの活性化の事象は無い。`<1>3` より読みはこの節点のどの事象よりも前に
  あるので、入口と読みの直前の点のあいだにも事象は無い。DEF 時点 は「読みは事象ではないので、読みの
  直前の点では `n^ι_C` も義務集合も動かず、その値はその点より前の最後の事象の直後の値に等しい」と
  述べるので、その 2 点で 2 つの量は等しい。(a) は `<1>1` である。
  BY DEF 時点, <1>1, <1>2, <1>3

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

**言明**。本体 `C` の各時点 `τ` (DEF 時点) と各計数下オブジェクト `O` について、`C` の 1 回の活性化の
D10 の義務集合は
`Obl(τ)(O) = Σ_{T : obj(T) = O} n^ι_C(τ, T)` を満たす。**時点は節点の入口に限らない** -- 1 つの節点が
leaf ごとに複数の事象を行うとき、その事象と事象のあいだの点も時点である。ここで D10 の初期値を決める
所有と借用の割り当ては、
`ι = ι_全` のときすべてのパラメータ・capture の unit を所有する割り当て、`ι = ι_V` のとき出力における
`V` の割り当て (P13) である。

<1>1. 初期値が合う。
  D10 の初期値は、所有するパラメータ・capture の unit の下の inhabited な各 leaf につき 1 つである。
  パラメータ leaf の由来は自分自身である (`Binding::Param` の腕は `here()` を返すので DEF 由来の 1 歩 の
  「上のどれでもない」に当たる)。`ι = ι_全` のときは全 unit が所有されるので、両辺はどの inhabited な
  パラメータ leaf も 1 と数える。`ι = ι_V` のときは、第 8 節の系 3 と P7e より
  `ctx.owns_object(p, σ) = ctx.owns_object(p, trunc(ty(p), σ))` であり、これは `V` が unit
  `trunc(ty(p), σ)` を D14 の意味で所有することと同値なので、両辺は同じ leaf を数える。P1 は
  **A10 を満たす**型についての言明であり、A10 はプログラムに現れる型の全体についてそれを与えるので
  `ty(p)` に当たる。P1 より
  `trunc(ty(p), σ)` は `units(ty(p))` の元であり、D10 の初期値が渡る unit の 1 つである。
  BY A10, D10, D14, P1, P7e, 第 8 節の系 3, CODE src/rc_ir/ownership.rs: origin_inner

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
  D9 と D10 より、義務集合を動かす構文は生成・消費・`Retain`・`Release` の 4 つで尽きる。DEF 時点 より
  時点は活性化の開始とこの 4 種の事象の直後の点であり、1 つの節点が leaf ごとに行う事象は D10 と
  DEF 由来ごとの義務 で同じ leaf の列を同じ順に取るので、事象の列の上の帰納で全時点について等式が
  立つ。`<1>1` が開始の時点を、`<1>2` から `<1>4` が各事象を与える。
  BY D9, D10, DEF 時点, DEF 由来ごとの義務, <1>1, <1>2, <1>3, <1>4

#### L27 (A19 (ii-a) を `B_V` の記法へ渡す)

**言明**。第 9.1 節が書いた形の A19 (ii-a) -- (a) `ρ` の各節点の入口 `τ` と各計数下の `ρ`-由来 `T` に
ついて `n_in(τ, T) ≥ 0`、(a') 終端の `Ret` の消費を行った直後の位置 `τ_a` でも `n_in(τ_a, T) ≥ 0`、
(b) `obj(x, λ)` を読む構文の節点の入口 `τ` (DEF 時点) で `n_in(τ, T_ρ(x, λ)) ≥ 1`、
(b') leaf `λ` のオブジェクトに触れる `Retain`/`Release` の節点の入口 `τ` で
`n_in(τ, T_ρ(v, λ)) ≥ 1` -- が成り立つ。

<1>1. `B_V` は、`borrow_ify` の入力の本体 (`V` が `f_own` かグローバル初期化子のとき) か、入力の関数
      `func` の本体を `ρ_f` で一斉に付け替えた本体 (`V = f_borrow` のとき) である。
  BY P9, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `borrow_ify` の入力の本体について、A19 が主語にする別名類 (D33) と第 9.3 節の `ρ`-由来 は
      1 対 1 に対応し、A19 が数える「その類が持つ参照の個数」(D34 の `held_ρ`) は、その類の開始の時点
      以後の各時点 `τ` について `n_in(τ, T)` に等しい。
  <2>1. D33 の別名類と第 9.3 節の `ρ`-由来 は 1 対 1 に対応する。
    L9a がこれを述べる。`ρ`-由来 `T` に対応する類が `C_T` である。
    BY D33, L9a
  <2>2. D34 の 6 行は DEF 由来ごとの義務 の行と、`ι = ι_全` の下で一致する。
    D34 の 6 行は、`C` の終端が D10 の生成で作られるとき 1 から始まる、終端が**所有する** (D14)
    パラメータ・capture の leaf であるとき 1 から始まる、終端が**借用する**それであるとき 1 から
    始まる、`Retain(v, π)` が `(v, λ) ∈ C` である `λ` を `π` の下に持つとき `λ` 1 つにつき +1、
    `Release(v, π)` について同じく -1、`(w, μ) ∈ C` の D9 の消費で -1、である。DEF 由来ごとの義務 は
    パラメータ・capture の leaf に初期値 `ι(p, σ)` を、それ以外の由来に「D10 の生成の表のいずれかの行が
    `u` に値を与える事象で 1 になる。その前は 0 である」を置き、`Retain`・`Release`・消費について同じ
    3 行を持つ。
    `ι_全` はすべての leaf に 1 を与えるので、第 2 行と第 3 行の開始値 1 が両方とも出る。A1 より
    `borrow_ify` の入力のすべての関数の `borrowed_units` は空なので、第 3 行に当たる類は入力の本体には
    無い。D6 より `(v, λ)` がスロットであることは `λ` がその時点で inhabited であることであり、
    `<2>1` より `(v, λ) ∈ C_T` は `T_ρ(v, λ) = T` である。
    BY A1, D6, D9, D10, D14, D34, DEF 由来ごとの義務, DEF n_in、n_out, <2>1
  <2>3. QED
    D34 は「最初の 3 行が置く開始値は、`T_ρ(C) = (u, σ)` の変数 `u` が値を得る時点で置かれる」とし、
    `held_ρ(τ, C)` をその時点以後の `τ` についてだけ定める。DEF 由来ごとの義務 も、生成の由来を
    「`u` に値を与える事象で 1 になる。その前は 0 である」とし、パラメータ・capture の由来には活性化の
    開始で初期値を置く。
    **2 つの定義は、生成の由来について開始の点を節点の内側で別に置く。** D34 は「`u` を束縛する節点を
    実行する段の直後」に置き、DEF 由来ごとの義務 はその節点の中の生成の事象に置く。DEF 時点 は節点の
    中の事象と事象のあいだの点も時点に数えるので、この 2 つはずれうる -- boxed 容器の `Destructure` は
    1 つの節点の中で容器の消費の事象とフィールドの生成の事象を持つ。**ずれる区間で `n_in(・, T)` を
    動かす事象は無い。** `T = (u, σ)` の `u` はこの節点が束縛する変数であり、この節点が生成の事象より
    後に行う事象が当たる由来は 2 種である -- この節点が消費する変数のスロットの由来 (その変数はこの
    節点より前に値を得ており、L9 より 1 歩の先も同じ時点のスロットなので、その由来の変数も `u` では
    ない) と、この節点が束縛する別の変数の対 (L5 より `ctx.rewrite` は束縛を導入も除去もしないので
    `B_V` の束縛名は `B'_V` のそれ、すなわち `V` の出力の束縛名と同じであり、4.3 の系 よりそれらは
    互いに相異なる)。
    よって D34 の開始の時点 -- 2 つのうち後の方 -- 以後の各 `τ` で両者は同じ値から同じ増減を積むので、
    その範囲で `held_ρ(τ, C_T) = n_in(τ, T)` である。
    BY D34, DEF 時点, DEF 由来ごとの義務, L5, L9, 4.3 の系, <2>1, <2>2

<1>3. `borrow_ify` の入力の各本体、その各実行路、それを辿る各活性化について、(a)・(a')・(b)・(b') が
      成り立つ。
  A19 の範囲は「**`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち
  `cancel` の入力) の両方**について、(ii-a) と (ii-b) を仮定する。」であり、入力の本体はその第 1 の側で
  ある。A19 (ii-a) は「各時点と各計数下の別名類について、その類が持つ参照の個数は非負で
  あり、読む構文と `Retain`/`Release` がその類を名指す時点では 1 以上である。**非負であることは、終端の
  `Ret` の消費を行った直後の時点についても言う。**」と述べる。
  **A19 が読める点は節点の訪問の入口に限る。** A19 は「「各時点」は、その活性化が生きている (D23) 間の、
  その活性化の節点の訪問の入口である時点である。」と定め、「`bumps` を定める D27 が `B(p, ρ)` を走査中の
  位置 -- 節点の訪問の入口 -- でしか定めないので、この 2 つを読める点はそこに限る。」と続ける。DEF 時点 は
  「節点の入口は、その節点より前の最後の事象の直後の時点と読み」と定めるので、節点の入口はこの文書の
  時点でもある。最後の一文が足すのは終端の `Ret` の消費の直後の 1 点であり、DEF 時点 の「最後の時点は
  終端の `Ret` の消費の直後である」がその点である。
  **(a)・(b)・(b') が量化するのはどれも節点の入口であり、(a') が量化するのはその 1 点である。**
  構文がその類を名指すのはその構文の節点であり、(b) と (b') はその節点の入口だけを量化する。よって
  4 つはどれも A19 が読める点の上の言明である。
  D34 より `held_ρ(・, C)` が定まるのは `C` の開始の時点以後なので、A19 (ii-a) が「その類が持つ参照の
  個数」と言うのもその範囲の `τ` についてである。
  (a) と (a') が言うのは非負性であり、開始の時点より前の `τ` では DEF 由来ごとの義務 より
  `n_in(τ, T) = 0` なので、そこでも非負である。(b) と (b') が当たる `τ` は開始の時点以後である --
  そこにスロット `(x, λ)` (resp. `(v, λ)`) が在り、L9 より 1 歩の先も同じ時点のスロットなので、
  `T_ρ(x, λ) = (u, σ)` の `u` はその時点までに値を得ているからである (D6)。
  `<1>2` より (a)・(a')・(b)・(b') はその言い換えである。
  BY A19, D6, D23, D34, DEF 時点, DEF 由来ごとの義務, L9, <1>2

<1>4. CASE `B_V` が入力の本体である。
  `<1>3` をこの本体に当てる。
  BY <1>3

<1>5. CASE `B_V` が `ρ_f` による付け替えである。
  <2>1. `B_V` は A19 の範囲のどちらの側でもない。`clone_func` の出力は `borrow_ify` の入力の本体では
        なく、`borrow_ify` が写した本体 (`B'_V`) でもない。
    BY A19, P9, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func
  <2>2. 一斉の名前替えは、実行路・活性化・スロット・由来・時点・`n_in` を写す。すなわち `func.body` の
        実行路と `B_V` の実行路が 1 対 1 に対応し、対応する路を辿る活性化が (パラメータ・capture の値と
        D21 の 4 種の結果を共有する形で) 1 対 1 に対応し、スロット `(x, λ)` と `(ρ_f(x), λ)` が対応し、
        `T_ρ(ρ_f(x), λ) = ρ_f(T_{ρ_0}(x, λ))` であり、DEF 時点 の各時点 -- 各事象の直後と各読みの直前 --
        が 1 対 1 に対応し、対応する時点 `τ` で
        `n_in^{B_V}(τ, ρ_f(T)) = n_in^{func.body}(τ, T)` である。
    P9 の前半より、複製は節点の種類・並び・`FieldPath`・`RcState`・`MatchArm` の `tag` と
    `payload_state`・`RcRhs::Closure` の `FuncRef` を変えず、`rename_var` は型を残し、変数の出現だけを
    `ρ_f` で写す。`collect_bindings` は本体の形だけから `Binding` を作るので、`vars_c` の束縛は `vars_f` の
    束縛の `ρ_f` による像であり、`p15` の `L15` より `origin` も `ρ_f` で写る。D3 の実行路、D6 のスロット、
    D16 の inhabited、D9 の消費と移動、D10 の生成と義務集合、DEF 由来の 1 歩、DEF 時点、
    DEF 由来ごとの義務 は
    いずれも、節点の種類・並び・`FieldPath`・変数の型・呼び出し先の所有だけから定まり、束縛名がどの
    文字列であるかを読まない。コード生成が節点について出すコードも `ρ_f` の下で変わらない --
    `rename_var` は型を残し、`rename_rhs` の `Llvm` の腕は `llvm_gen` の名前の欄だけを写すので宣言は
    同じである (A3)。呼び出し先 (D23) は callee の値が決める -- 直接呼び出しの名前は関数の名前で
    あり、A6 より束縛名ではないので `ρ_f` の鍵ではなく、間接呼び出しの callee は局所変数で、`ρ_f` は
    その変数と束縛を一斉に写すので、対応する活性化が対応する位置で持つ値は等しい。
    BY A3, A6, D3, D6, D9, D10, D16, D23, P9, p15 の L15, DEF 由来の 1 歩, DEF 時点,
       DEF 由来ごとの義務,
       CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/rename.rs: rename_var,
       CODE src/rc_ir/rename.rs: rename_rhs
  <2>3. QED
    `<1>3` を `func.body` に当てると (a)・(a')・(b)・(b') が `func.body` について成り立ち、`<2>2` が
    それを `B_V` へ写す。
    BY <1>3, <2>2

<1>6. QED
  `<1>1` の 2 つが場合を尽くす。
  BY <1>1, <1>4, <1>5

#### L28 (終端の `Ret` の消費の後、`n_in` はどの由来についても 0 である)

**言明**。`B_V` がすべてのパラメータ・capture の unit を所有する割り当ての下で D11 を満たすとき、
`ρ` の終端の `Ret` の消費を行った直後の位置 `τ_a` において、各計数下の `ρ`-由来 `T` について
`n_in(τ_a, T) = 0` である。

<1>1. `τ_a` における `Obl` は空である。
  D11 の (S-b) は「実行路の終端の `Ret(v)` において、その `Ret` の消費を行った後の `Obl` は空である」で
  ある。
  BY D11

<1>2. 各計数下オブジェクト `O` について `Σ_{T : obj(T) = O} n_in(τ_a, T) = 0` である。和は有限である。
  L17 を `ι = ι_全` について当てる -- L17 が読む割り当ては、`ι = ι_全` のときすべてのパラメータ・
  capture の unit を所有する割り当てであり、それが言明の割り当てである。和が有限なのは、D2 と D3 より
  実行路が有限であり、各節点が束縛する変数が有限個なので、`ρ` の上のスロット (D6) が有限個で、由来も
  有限個だからである。
  BY D2, D3, D6, L17, <1>1

<1>3. 各項は非負である。
  L27 の (a') である。
  BY L27

<1>4. QED
  非負の有限個の項の和が 0 なので、各項が 0 である。各計数下の `ρ`-由来 `T` は `obj(T)` が計数下なので、
  `O = obj(T)` についての `<1>2` の和に現れる。
  BY <1>2, <1>3

## 10. P14 -- `borrow_ify` は RC 規律を保存する

**言明** (README の P14)。D12 の意味で RC 規律を満たし、かつ A1 と A2 を満たすプログラムを入力とすると、
`borrow_ify` の出力は D12 の意味で RC 規律を満たす。

### 10.1 示す形

**言明**。出力の各版 `V` と、`B'_V` の各実行路を辿る各活性化について (S-a)・(S-b)・(S-c) が成り立つ
ことを示せば、P14 が出る。

<1>1. 出力のプログラムの本体は、入力の各関数 `func` についての `f_own` の `body`、`borrow_versions` に載る
      各関数についての `f_borrow` の `body`、入力の各グローバル初期化子についての出力の `init` で尽きる。
  BY L6, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. D12 は、これらの本体のそれぞれが、出力の `borrowed_units` が定める割り当て (D14) の下で D11 を
      満たすことである。P13 よりその割り当ては `owned_units` が定めるものであり、第 8 節の系 3 より
      `ctx.owns_object` がパラメータ unit についてそれを答える。
  BY D12, D14, P13, 第 8 節の系 3

<1>3. QED
  以下、出力の版 `V` を 1 つ取り、第 9.1 節のとおり `B_V`、`B'_V`、`ctx`、`B'_V` の活性化と、それに
  対応する `B_V` の実行路 `ρ` と活性化 (L16a) を固定して、(S-a)、(S-b)、(S-c) を示す。D11 は本体の
  すべての実行路について 3 つの節を課し、L16 より `B'_V` の実行路は `B_V` の実行路と 1 対 1 に対応する。
  `V` と活性化は任意なので、これで `<1>2` が出る。
  BY D11, L16, L16a, <1>1, <1>2

### 10.2 `B_V` は入力の割り当ての下で D11 を満たす

**言明**。`B_V` は、すべてのパラメータ・capture の unit を所有する割り当ての下で D11 を満たす。さらに、
`B_V` の各 `App` 節点の呼び出し先 (D23) は、`V` が `f_borrow` のとき、入力の関数 `func` の本体の対応する
節点の呼び出し先と同じ `RcFunc` である。

<1>1. `V` が `f_own` かグローバル初期化子であるとき、`B_V` は入力の関数の本体または入力のグローバル
      初期化子の `init` そのものである。A1 と D12 よりそれは D11 を満たし、A1 よりその割り当ては全所有で
      ある。
  BY A1, D12, CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `V = f_borrow` であるとき、`B_V` は入力の関数 `func` の本体を `ρ_f` で一斉に付け替えたものである。
  BY P9

<1>3. 一斉の名前替えは D11 を保つ。
  D3 の実行路、D6 のスロット、D9 の消費と移動、D10 の義務集合、D11 の 3 つの節は、いずれも本体の節点の
  種類・並び・`FieldPath`・変数の型・呼び出し先の所有と、`Llvm` 節点の op が返す宣言だけから定まり、
  束縛名がどの文字列であるかを読まない。**op の宣言をここに挙げるのは、D9 の消費の表の `Llvm` の行が
  `borrows_operand(i, arg_tys, type_env)` を、D10 の生成の表の `Llvm` の行が
  `result_prov(result_ty, arg_tys, type_env)` を読み、どちらも `&self` を取るからである** --
  `rename_rhs` の `Llvm` の腕は `llvm_gen.clone()` を作って `free_vars_mut()` の名前を書き替えるので、
  複製の op は原本とは別のオブジェクトである。A3 は「`rhs.clone()` や `fresh_rename_function` が作る
  複製の op は、原本と同じ引数を渡されれば同じ `Provenance` を返す」と述べ、`borrows_operand` にも
  同じことを述べる。`rename_var` は型を残すので `arg_tys` と `result_ty` も両側で等しく、よって宣言は
  等しい。
  P9 の前半よりこの 5 つは `ρ_f` の下で保たれる -- 節点の種類・並び・`FieldPath`・`MatchArm` の `tag` は
  変わらず、`rename_var` は型を残し、`RcRhs::Closure` の `FuncRef` も変わらない。**呼び出し先 (D23) は
  `App` の callee の値が決める。** 直接呼び出しの callee は関数の名前を持つ `RcVar` であり、A6 より
  束縛名ではないので `rename_f` の鍵ではなく、`rename_var` は鍵を持たない名前をそのまま残す。間接
  呼び出しの callee は局所変数であり、`rename_rhs` の `App` の腕はそれを `rename_var` で写すが、`ρ_f` は
  本体の束縛と使用を一斉に写すので、対応する活性化が対応する位置でその変数に持つ値は等しく、呼び出し先も
  等しい。`ρ_f` は単射なので、A6 と A11 が要求する束縛と使用の対応も保たれる。`func` のパラメータ・
  capture と `f_borrow` のそれは `ρ_f` で対応し、型は等しい。
  BY A3, A6, A11, D3, D6, D9, D10, D11, D23, P9, CODE src/rc_ir/rename.rs: rename_var, rename_rhs,
     CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, LLVMGen::borrows_operand

<1>4. QED
  第 1 文は `<1>1`・`<1>2`・`<1>3` による。第 2 文は `<1>3` が呼び出し先について述べたことそのもので
  ある -- 直接呼び出しの callee は関数の名前を持つ `RcVar` で `ρ_f` の鍵ではなく、間接呼び出しの callee は
  局所変数で、`ρ_f` は本体の束縛と使用を一斉に写すので、対応する活性化が対応する位置でその変数に持つ値は
  等しい。
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
        同じ leaf について起き、L16a より対応する変数の値が等しいので inhabited な leaf も等しい。
        `Let(x, Match(scrut, arms), k)` の節点については、`Match` 節点自身は
        参照を作らず、移さず、手放さず (D9)、変位アームの payload 束縛は boxed の scrutinee のとき D10 の
        生成、unbox の scrutinee と catch-all のとき D9 の移動であり、L16 よりアームの `tag` と `payload` は
        両側で同じである。
    BY D9, D10, D16, L16, L16a
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
  <2>1. `π ∈ units(ty(v))` であり、`(v, π)` は `B_V` の site (DEF site) である。
    `B_V` が入力の本体のときは A2 がそのまま与える。`B_V` が `f_borrow` の複製本体のときは、P9 の前半より
    複製は `FieldPath` を変えず、`rename_var` は `ty` を残すので、複製の `Retain`/`Release` の path も
    その変数の型の `rc_units` の元である。DEF site は `B_V` を `for_each_node` で歩いた
    `Retain`/`Release` の節点の `(v, path)` を site とし、A15 より `grow_stack` はその歩みを変えない。
    BY A2, A15, DEF site, P9, CODE src/rc_ir/ast.rs: for_each_node,
       CODE src/rc_ir/rename.rs: rename_var, CODE src/misc.rs: grow_stack
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
  <2>2. 各引数の添字 `i` と各 `u ∈ units(ty(args[i]))` をとる。`(args[i], u)` は `B_V` の site
        (DEF site) である。
    DEF site は `Let(_, App(_, args), _)` の各 `arg` と各 `unit ∈ rc_units(ty(arg))` の対を site とする。
    BY DEF site
  <2>3. `u` の下の inhabited な各 leaf `λ` について、`T_ρ(args[i], λ)` が所有されることと
        P11 の `arg_owned(i, u)` が真であることとは同値である。
    BY L13, <2>2
  <2>4. `B_V` の `App` は、`u` の下の inhabited なすべての leaf を消費する。
    D9 の `App` の行が読む所有は D14 が `RcFunc::borrowed_units` から定めるものであり、D23 は
    「**その呼び出し先はプログラムの `funcs` の関数である**」と述べる。`B_V` は入力の関数の本体か入力の
    グローバル初期化子の `init` そのものか、入力の関数 `func` の本体を `ρ_f` で付け替えたものであり、
    後者についても 10.2 の言明よりその `App` の呼び出し先は `func.body` の対応する節点の呼び出し先と
    同じ `RcFunc` である。よってその呼び出し先は入力プログラムの `funcs` の関数であり、A1 より
    `borrowed_units` は空なので、D14 より全パラメータの全 unit を所有する。A12 より `u` は呼び出し先の
    対応するパラメータの型でも同じ unit なので、D9 の `App` の行より `u` の下の inhabited な各 leaf が
    消費される。
    BY A1, A12, D9, D14, D23, P9, 10.2 の言明, CODE src/rc_ir/borrow.rs: borrow_ify
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
    ちょうど 1 つの unit へ切り詰まるので、引数の leaf は重複なく尽くされる。P1 は **A10 を満たす**型に
    ついての言明であり、A10 はプログラムに現れる型の全体についてそれを与えるので `ty(args[i])` に当たる。
    BY A10, D9, D10, L16, P1, DEF 由来ごとの義務, <2>1, <2>6, <2>7

<1>6. QED
  D2 より `RcExpr` は 6 種であり、`Let` を `App`・それ以外に分けると `<1>3`・`<1>4`・`<1>5` が尽くす
  (`Let(x, Match(scrut, arms), k)` は `<1>3` に入る -- `Match` の節点自身は参照を作らず、移さず、
  手放さない (D9))。`<1>2` を基底、`<1>3`-`<1>5` を段とする、`ρ` の上の位置についての帰納である。各段は
  その節点の入口で INV を仮定して、その節点の**出口** -- DEF 対応する位置 の次の位置 -- で INV を示す。
  終端の `Ret` の出口は「終端の `Ret` の消費の後」であり、その節点は `<1>3` の場合なので、最後の位置も
  この帰納が覆う。
  BY D2, D9, <1>2, <1>3, <1>4, <1>5

### 10.4 `n_out` は非負であり、塊の中でも下回らない

**言明**。第 9.1 節の A19 (ii-a) の (a) と (a') の下で、`B'_V` の実行路の各時点 (DEF 時点。塊の中を
含む) で、
各計数下 (D26) の由来 `T` について `n_out ≥ 0` である。さらに、`n_out` を 1 減らす各事象の直前で、その
事象が減らす由来 `T` について `n_out(T) ≥ 1` である。

<1>0. D10 の生成が値を与える変数は `V` のパラメータでも capture でもなく、生成が増やす由来は所有される。
  D10 の生成の表の 5 行が値を与えるのは、`Let` の束縛変数、`Destructure` のフィールド変数、`Match` の
  変位アームの payload 変数である。`VarTable::of` が `Binding::Param` を入れるのはパラメータと capture に
  ついてだけなので、これらはそのどちらでもない。L19 より、パラメータでも capture でもない変数を持つ
  由来は所有される。
  BY D10, L19, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings

<1>0a. L16 の (K) の `Retain(v, π)`/`Release(v, π)` 節点が動かす由来は、すべて所有される。
  L16 の (K) は「`B_V` の `Retain`/`Release` 節点のうち、`V` が借用版でないか `owns_unit(v, π)` が真で
  あるもの」である。前者では L8 よりすべての由来が所有される。後者では L13 より `π` の下の inhabited な
  各 leaf の由来が所有される -- L13 が要求する「`(v, u)` を `B_V` の site (DEF site) とし」は、A2 と
  P9 より `π ∈ units(ty(v))` であり、DEF site が `B_V` を `for_each_node` で歩いた `Retain`/`Release` の
  `(v, path)` を site とすることによる (A15 より `grow_stack` はその歩みを変えない)。D10 の
  `Retain`/`Release` の行が動かすのは `π` の下の inhabited な各 leaf の由来だけである。
  BY A2, A15, D10, DEF site, L8, L13, L16, P9,
     CODE src/rc_ir/ast.rs: for_each_node, CODE src/misc.rs: grow_stack

<1>1. 1 つの塊の中で `n_out` を増やす事象は `B'_V` の `Retain` 節点と D10 の生成の 2 種、減らす事象は
      `B'_V` の `Release` 節点と D9 の消費の 2 種である。さらに、生成が増やす由来を同じ塊の中で減らす
      事象は無い。
  <2>1. DEF 由来ごとの義務 が挙げる行は、初期値・生成・`Retain`・`Release`・消費の 5 つであり、初期値の
        行が当たるのは活性化の開始の 1 度だけで、塊の中には無い。
    BY DEF 由来ごとの義務
  <2>2. 生成が増やす由来は、その塊の節点が束縛する変数 `x` の対 `(x, λ)` であり、その塊の入口では
        スロットではない。
    `<1>0` より生成が値を与えるのはその節点が束縛する変数である。DEF 由来の 1 歩 より、その変数の
    `Binding` は `Producer`、`Llvm` (単一の `Arg` でない)、boxed 容器の `Field`、boxed scrutinee の
    `Payload` のいずれかであって 1 歩を持たないので、`(x, λ)` は自分自身を由来とする。`x` はその塊の
    入口までに値を得ていないので、D6 よりその時点で `(x, λ)` はスロットではない。
    BY D6, D10, DEF 由来の 1 歩, <1>0
  <2>3. 塊の中で `Release` と消費が減らす由来は、その塊の入口におけるスロットである。
    `Release(v, π)` が減らすのは `π` の下の inhabited な各 leaf のスロットの由来であり、D9 の消費の
    6 行が名指すのは `App` の callee と引数、`Closure` の capture、`Llvm` のオペランド、`Destructure` の
    容器と名前の付いていないフィールド、終端の `Ret` の変数である。いずれもその塊の節点がオペランドと
    して名指す変数であり、A11 よりその位置でスコープに入っている束縛に解決するので、D6 の意味でその塊の
    入口までに値を得ている。L9 より 1 歩の先も同じ時点のスロットなので、由来もその塊の入口における
    スロットである。
    BY A11, D6, D9, D10, L9
  <2>4. QED
    `<2>1` が事象の種類を尽くす。`<2>2` と `<2>3` より、生成が増やす由来は塊の入口のスロットではなく、
    塊の中で減る由来は塊の入口のスロットなので、両者は相異なる。
    BY <2>1, <2>2, <2>3

<1>2. 所有されない由来 `T` を増やす事象は (A-前) の `Retain` だけであり、1 つの塊の中で `T` に当たる
      (A-前) の増分の総和は、同じ塊の `App` の消費が `T` に当たる減分の総和に等しく、増分はすべて減分より
      前に置かれる。
  <2>1. (K) の `Retain(v, π)` は所有されない由来を増やさない。
    BY <1>0a
  <2>1a. D10 の生成は所有されない由来を増やさない。
    BY <1>0
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
    `<2>1`、`<2>1a`、`<1>1` より、所有されない由来を増やしうるのは (A-前) の `Retain` だけである。
    `<2>2` の同値より、`callee_owns` が偽の `(i, u)` では `App` は `u` の下の leaf を消費しない。よって
    所有されない由来に当たる `App` の消費は、`callee_owns` が真かつ `arg_owned` が偽の `(i, u)`、すなわち
    (A-前) に入る対の分ちょうどである。(A-後) の `Release(a, u)` が置かれるのは `arg_owned(i, u)` が
    真の対だけであり (P11)、L13 よりそのとき `u` の下の inhabited な各 leaf の由来は所有されるので、
    (A-後) も所有されない由来を減らさない。
    BY L13, P11, <1>1, <2>1, <2>1a, <2>2, <2>3

<1>3. 所有されない計数下の由来 `T` について、DEF 対応する位置 の各位置では `n_out(T) = 0` であり、塊の
      中では非負で、`T` を 1 減らす事象の直前は 1 以上である。
  <2>1. DEF 対応する位置 の各位置で `n_out(T) = 0` である。
    BY 10.3 の INV
  <2>2. `App` の塊では、`<1>2` より `T` の増分の総和と減分の総和が等しく、増分がすべて減分より前に
        起きる。よって塊の中の各時点の値は 0 以上であり、各減分の直前の値はその減分以降に残る減分の
        個数以上、すなわち 1 以上である。
    BY <1>2, <2>1
  <2>3. `App` の塊でない塊では、`T` を動かす事象が無い。
    DEF 塊 と L16 よりその塊は元の節点 1 つか空である。`<1>1` の 4 種のうち、(A-前) と (A-後) は `App` の
    塊にしか現れない。(K) の `Retain`/`Release` が動かす由来は `<1>0a` よりすべて所有され、生成が増やす
    由来は `<1>0` より所有される。消費については、この場合の節点が行いうるのは D9 の消費の 6 行のうち
    `App` の引数の行以外、すなわち DEF 所有を読まない消費 であり、L14 よりその由来は所有される。
    BY D9, DEF 塊, DEF 所有を読まない消費, L14, L16, <1>0, <1>0a, <1>1
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>4. 所有される計数下の由来 `T` について、終端の `Ret` の塊を除く各塊の入口と出口で
      `n_out = n_in ≥ 0` である。
  DEF 対応する位置 より、`B_V` の各節点の入口が位置であり、終端の `Ret` 以外の節点の塊の出口は、その路の
  次の節点の入口である。10.3 の INV よりその位置で `n_out = n_in` であり、L27 (第 9.1 節の A19 (ii-a) の
  (a)) よりその位置で `n_in ≥ 0` である。
  BY L27, DEF 対応する位置, 10.3 の INV

<1>5. 1 つの塊の中で、各由来 `T` について、`T` を増やす事象はすべて `T` を減らす事象より前にある。
  <2>1. `App` の塊。L16 と P11 よりこの塊は (A-前) の `Retain` の列、`App` の節点、(A-後) の `Release` の
        列であり、`prepend_rc` が (A-前) を `App` の外側に置くのでそれらは `App` より前に、(A-後) は
        `App` の `Let` の継続の側なので後に実行される。`<1>1` より `App` の結果の生成が増やす由来は
        この塊の中で減らされないので、`T` を増やす事象は (A-前) の `Retain` に限られ、それらは `T` を
        減らす `App` の消費と (A-後) の `Release` より前にある。
    BY L16, P11, <1>1, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, prepend_rc
  <2>2. `Retain` の塊。L16 の (K) よりこの塊は `Retain` 節点 1 つか空であり、減らす事象が無い。
    BY L16
  <2>3. `Release` の塊。L16 の (K) よりこの塊は `Release` 節点 1 つか空であり、増やす事象が無い。
    BY L16
  <2>4. 残る塊。DEF 塊 と L16 よりその塊は元の節点 1 つである。`<1>1` より、その節点の生成が増やす由来は
        その塊の中で減らされないので、増やす事象と減らす事象は相異なる由来に当たり、1 つの由来について
        見れば増分か減分のどちらかしか起きない。
    BY DEF 塊, L16, <1>1
  <2>5. QED
    D2 より `B_V` の節点は 6 種であり、`Let` を `App` とそれ以外に分けると、DEF 塊 と L16 より塊は
    `<2>1`-`<2>4` の 4 つで尽きる。
    BY D2, DEF 塊, L16, <2>1, <2>2, <2>3, <2>4

<1>5a. 終端の `Ret` の塊では、塊の出口で `n_out = 0` であり、塊の中の各時点の値は非負、各減分の直前の
       値は 1 以上である。
  <2>1. この塊は終端の `Ret` の節点 1 つであり、その事象は D9 の終端の `Ret` の行の消費だけである。
        増やす事象は無い。
    BY D9, D10, DEF 塊, L16
  <2>2. 塊の出口 -- 終端の `Ret` の消費の後 -- で、各計数下の由来 `T` について `n_out = 0` である。
    10.2 より `B_V` はすべてのパラメータ・capture の unit を所有する割り当ての下で D11 を満たすので、
    L28 より `n_in(τ_a, T) = 0` である。`τ_a` は DEF 対応する位置 が挙げる位置なので 10.3 の INV が
    成り立ち、所有される由来については `n_out = n_in = 0`、所有されない由来については `n_out = 0` で
    ある。
    BY L28, DEF 対応する位置, 10.2 の言明, 10.3 の INV
  <2>3. QED
    `<2>1` より塊の中では値が減る一方なので、各時点の値は出口の値にその時点以降に残る減分の個数を
    足したものであり、`<2>2` より出口の値は 0 である。よって各時点で非負であり、各減分の直前の値は
    1 以上である。
    BY <2>1, <2>2

<1>6. QED
  所有されない計数下の由来については `<1>3` が言明を与える。所有される計数下の由来については次のとおり
  である。終端の `Ret` の塊は `<1>5a` が与える。それ以外の塊では、`<1>4` より入口と出口の値がどちらも
  その位置の `n_in` に等しく非負であり、`<1>5` より塊の中では増分がすべて減分より前にあるので、値は
  入口から増分の分だけ上がり、そこから減分の分だけ出口まで下がる。よって塊の中の各時点の値は入口の値と
  出口の値のうち小さい方以上であり、非負である。各減分の直前の値は、出口の値にその減分以降に残る減分の
  個数を足したもの以上であり、0 + 1 = 1 以上である。DEF 塊 と L16 より `B'_V` の各時点はいずれかの塊の
  中か塊の境界にあるので、これで尽きる。
  BY DEF 塊, L16, <1>3, <1>4, <1>5, <1>5a

### 10.5 (S-a) 過剰処分が無い

<1>1. `B'_V` で `Obl` から参照を取り除く操作は、`Release` 節点と D9 の消費である。
  BY D9, D10

<1>2. そのどの操作についても、取り除かれる参照はその時点の `Obl` に入っている。
  取り除かれる参照は、その操作が名指す inhabited な各 leaf `λ` について `obj(・, λ)` への参照 1 つで
  ある (D10)。`obj(・, λ)` がグローバル状態 (D26) であるときは、その leaf は D8 の意味の参照を持たない
  ので、取り除かれる参照が無い。計数下であるとき、DEF 時点 よりその除去は 1 つの事象であり、
  DEF 由来ごとの義務 よりその事象は由来
  `T_ρ(・, λ)` の分を 1 減らす。10.4 より、その事象の直前の時点で `n_out(T_ρ(・, λ)) ≥ 1` であり、
  10.4 より他の
  どの計数下の由来の `n_out` も非負なので、L17 より `Obl(τ)(obj(T_ρ(・, λ))) ≥ 1` である。L9 より
  `obj(T_ρ(・, λ)) = obj(・, λ)` である。1 つの節点が同じオブジェクトへの参照を 2 つ以上取り除くときも、
  それぞれが別の (あるいは同じ) 由来の分を減らす 1 つの事象であり、10.4 がその各事象の直前の時点で
  `n_out ≥ 1` を与える。**2 つ目以降の除去の直前の点は節点の入口ではないが、DEF 時点 の意味の時点で
  あり、L17 の等式は各時点について立つ。**
  BY D8, D10, D26, DEF 時点, L9, L17, 10.4

<1>3. QED
  BY D11, <1>1, <1>2

### 10.6 (S-b) 漏れが無い

<1>1. `B'_V` の実行路の終端の `Ret(v)` は、`B_V` の終端の `Ret(v)` に対応する位置にあり、その後に
      `Retain`/`Release` 節点は無い。
  L16 より `B'_V` の `Retain`/`Release` は (K)、(A-前)、(A-後) の 3 種であり、(A-前) と (A-後) は `App` の
  直前・直後に立つ。`App` の節点は継続を持つので、終端の `Ret` より前にある。(K) は `B_V` の節点の位置に
  立つ。
  BY D2, D3, L16

<1>2. `B_V` の終端の `Ret(v)` の消費の後の位置 `τ_a` で、`n_in(τ_a, T)` はどの計数下の由来 `T` に
      ついても 0 である。
  10.2 より `B_V` はすべてのパラメータ・capture の unit を所有する割り当ての下で D11 を満たすので、
  L28 がこれを与える。L28 は第 9.1 節の A19 (ii-a) の (a') を読む -- (a) は節点の入口についてしか
  言わないので、この位置について何も言えない (第 9.1 節)。
  BY L28, 10.2 の言明

<1>3. QED
  「終端の `Ret` の消費の後」は DEF 対応する位置 が挙げる位置であり、10.3 の INV はそこでも成り立つ。
  よって所有される計数下の由来については `n_out(τ_a, T) = n_in(τ_a, T) = 0` (`<1>2`)、所有されない
  計数下の由来については `n_out(τ_a, T) = 0` である。L17 より各計数下オブジェクト `O` について
  `Obl(τ_a)(O) = Σ_{T : obj(T) = O} n_out(τ_a, T) = 0` であり、D8 と D26 より `Obl` はグローバル状態の
  オブジェクトへの参照を持たないので、`B'_V` の `Obl` はその時点で空である。`<1>1` よりその時点は
  `B'_V` の実行路の終端の `Ret` の消費の後である。
  BY D8, D11, D26, L17, DEF 対応する位置, 10.3 の INV, <1>1, <1>2

### 10.7 (S-c) 解放後の読みが無い

<1>1. `B'_V` の各位置で D7 の読む構文が読みうるオブジェクトと、`Retain`/`Release` が触れるオブジェクトは、
      次の 3 種で尽きる。
  - **(i)** `B_V` の読む構文が対応する位置で読みうるオブジェクト。L16 より読む構文の列は両側で等しく、
    L16a より D7 の表が名指す値も等しい。ただ 1 つの例外が `App` の callee であり、`route` が名前を
    差し替えた `App` では L18 より `leaves(ty(c)) = ∅` なので、その `App` が callee を通じて読みうる
    オブジェクトはどちらの側にも無い (D7 は「名指した値の inhabited な各 boxed leaf が指すオブジェクト」を読みうる
    先とする)。
  - **(ii)** `B_V` の `Retain(v, π)`/`Release(v, π)` のうち `B'_V` に残ったもの (L16 の (K)) が触れる
    オブジェクト。
  - **(iii)** L16 の (A-前) と (A-後) が触れるオブジェクト。
  BY D7, L16, L16a, L18

<1>1a. **示す形。** `B'_V` の読む構文が行う読みと、`Retain`/`Release` が leaf に触れる動作を 1 つ固定
      し、その**直前の点** `p` を DEF 時点 の意味で取る。D11 の (S-c) は「その活性化がその時点まで解放に
      ついて閉じている (D11a) とき」にだけ条件を課すので、以下その仮定を置き、その動作が名指す
      オブジェクトが `p` において解放されていないことを示す。
  D11 の (S-c) は「**「直前の点」は、その読み・その触れる動作が実際に起きる瞬間の直前であって、節点の
  入口ではない。**」と書く。DEF 時点 はその 2 種の点をどちらも時点に数える。読みについては、その読みの
  直前の点が時点であり、`p` はその点である。`Retain(v, π)` と `Release(v, π)` が `π` の下の inhabited な
  leaf `λ` のオブジェクトに触れる動作は DEF 時点 の意味の事象であって -- D10 のこの 2 行は `λ` ごとに
  参照を 1 つ加え・取り除き、同時に `H(obj(v, λ))` を 1 動かす -- 、`p` はその事象の直前の時点である。

  **`p` が D24 の時点でも段内の点でもないことがある。** D24 は段内の点を「段の素動作を時間の順に
  並べたときの、最初の素動作の前の点と各素動作の後の点」と定め、D21 の制限も P28 (b) も D11a の
  閉じていることもその粒度で読む。DEF 時点 が挙げる 3 種のうち、活性化の開始と各事象の直後は、その
  受け渡し・生成・処分の素動作の直後の点なので段内の点である。読みの直前の点はそうとは限らない --
  読みは素動作ではないからである。**その点への橋は D24 が持っている** -- D24 は「読みの直前の点では、
  勘定は直前の段内の点のものである。」と置き、「その点と直前の段内の点のあいだに素動作は 1 つも無いので、
  `H` も `Obl` も `held` (D34) も動かず、解放も起きない。」「よって段内の点について示した勘定は、その点へ
  そのまま移る。」と述べる。**以下、`p` における `H`・`Obl`・解放の有無は、`p` より前の最後の
  段内の点での値として読む。**
  BY D7, D10, D11, D11a, D24, DEF 時点

<1>2. グローバル状態 (D26) のオブジェクトは解放されない。
  BY A8, D26

<1>2a. `p` 以前の時点 `τ` において `H(τ)(O) ≥ 1` である計数下オブジェクト (D26) `O` は、`τ` において
       解放されていない。
  D11a は「活性化の時点 `τ` が**解放について閉じている**とは、`τ` において `H(O) ≥ 1` である各計数下
  オブジェクト (D26) `O` が、`τ` において解放されていない (D24 の (F)) ことをいう」「活性化が
  **`τ` まで閉じている**とは、`τ` 以前の各時点が解放について閉じていることをいう」と述べる。
  `<1>1a` が置いた仮定より、`p` 以前の各時点は解放について閉じている点である。
  BY D11a, D24, D26, <1>1a

<1>3. `p` 以前の時点 `τ` で `Obl(τ)(O) ≥ 1` である計数下オブジェクト `O` は、その時点で
      `H(τ)(O) ≥ 1` であり、`τ` において解放されていない。
  D21 は「活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る」と述べる。
  その不等式は
  `H(O) ≥ Σ_{C ∈ S} d(C) + [S に借用終端の類が在るならば 1]` であり、A19 (i) の本文は
  「`Σ d(C)` はその活性化の義務集合が持つ `O` への参照の個数である」と述べる。角括弧は 0 以上なので
  `H(O) ≥ Obl(O)` である。
  **`τ` がこの制限の掛かる点であること。** この制限が掛かるのは D24 の時点と段内の点である。`<1>1a` は
  DEF 時点 が挙げる 3 種のうち、活性化の開始と各事象の直後は段内の点であり、各読みの直前の点は、それより
  前の最後の段内の点とのあいだに素動作を 1 つも持たないので `H` も `Obl` も動かないことを述べる。よって
  `τ` が前 2 種のときは制限がその点で直に掛かり、`τ` が読みの直前の点のときは、その前の最後の段内の点で
  掛かる不等式の両辺が `τ` での値に等しい。**この段は A19 の「各時点」の読みを使わない** -- 使うのは
  D21 が段内の点まで掛ける制限と、`<1>1a` の橋である。
  節点の中の点についても `Obl(τ)(O)` は L17 が各時点について与えるので、両辺が同じ点で定まる。
  よって `H(τ)(O) ≥ Obl(τ)(O) ≥ 1` であり、`<1>2a` がそこから結論を与える。
  BY A19, D21, D24, D26, DEF 時点, L17, <1>1a, <1>2a

<1>4. `V` が借用する unit の下の inhabited な leaf が指す計数下のオブジェクトは、`p` において解放
      されていない。
  <2>1. この活性化を作った段は (E3) の呼び出しの段であり、呼び出し元が在る。
    `V` が D14 の意味で借用する unit を持つとする。第 8 節の系 1 より `f_own` の `borrowed_units` は
    空であり、D1 よりグローバル初期化子はパラメータも capture も持たないので unit を持たない。よって
    `V` は借用版である。L18a より、借用版の本体の活性化を作る段は (E3) の呼び出しの段だけである。
    BY D1, D14, L18a, 第 8 節の系 1
  <2>2. 呼び出し元は、その unit の下の inhabited な leaf が指す参照を、この呼び出しが返るまで処分
        しない。
    A5 は「値が保持する参照は、その型の `boxed_leaf_paths` が列挙する leaf のうち、inhabited (D16) で
    あって計数下のオブジェクト (D26) を指すものにちょうど 1 つずつある」と述べるので、その leaf には
    参照が 1 つ在る。A20 は「**`borrow_ify` の出力と、`cancel` がそれを写したプログラムの両方**に
    ついて、その関数が借用する (D14) unit の参照を、呼び出し元は呼び出しが返るまで処分しない」と
    述べる。`<2>1` よりこの活性化には呼び出し元が在る。
    BY A5, A20, D14, D16, D26, <2>1
  <2>3. QED
    A5 よりその leaf は参照を 1 つ持ち、D8 より参照は処分されるまで存在し、`H(O)` は `O` への未処分の
    参照の総数である。`<2>2` の参照はこの活性化が生きている間 未処分なので、この活性化のどの時点でも
    `H(O) ≥ 1` である。`p` はこの活性化の時点なので (`<1>1a`)、`<1>2a` が結論を与える。
    BY A5, D8, D25, <1>1a, <1>2a, <2>1, <2>2

<1>5. 固定した動作が (i) の読みか (ii) の触れる動作であるとき、それが名指すオブジェクトは `p` において
      解放されていない。
  <2>1. そのオブジェクトはあるスロット `(x, λ)` の `obj(x, λ)` である。読む構文は名指した値の inhabited な
        各 boxed leaf が指すオブジェクトを読みうる (D7)。`Retain(v, π)`/`Release(v, π)` は `π` の下の
        inhabited な各 leaf が指すオブジェクトに触れる (D7)。
    BY D7
  <2>2. `obj(x, λ)` がグローバル状態のとき、`<1>2` による。
    BY <1>2
  <2>3. `obj(x, λ)` が計数下で `T_ρ(x, λ)` が所有されるとき、`obj(x, λ)` は `p` において解放されて
        いない。
    <3>0. `p` 以前の時点 `τ` で `n_out(τ, T_ρ(x, λ)) ≥ 1` であれば、`τ` において
          `H(τ)(obj(x, λ)) ≥ 1` であり、`obj(x, λ)` は `τ` において解放されていない。
      10.4 より、他のどの計数下の由来の `n_out` も DEF 時点 の各時点で非負なので、L17 より
      `Obl(τ)(obj(T_ρ(x, λ))) ≥ 1` である。L9 より `obj(T_ρ(x, λ)) = obj(x, λ)` である。`<1>3` が
      結論を与える。
      BY L9, L17, <1>3, 10.4
    <3>1. CASE 読む構文の読みであって、その節点が `App` の節点でない。
      <4>1. `B'_V` のこの節点の塊はこの節点 1 つであり、DEF 対応する位置 よりその塊の入口 `τ'` には
            `B_V` の同じ節点の入口 `τ0` が対応する。`<1>1` より (i) の位置は `B_V` でも同じ読む構文の
            位置である。
        DEF 塊 と L16 より、`App` でない節点の塊は同じ節点 1 つか空であり、空になるのは
        `Retain`/`Release` の節点に限る。D7 はその 2 つを読む構文としない。
        BY D7, DEF 塊, DEF 対応する位置, L16, <1>1
      <4>2. `n_out(τ', T_ρ(x, λ)) ≥ 1` である。
        L27 の (b) より `n_in(τ0, T_ρ(x, λ)) ≥ 1` である。`τ0` は DEF 対応する位置 が挙げる位置なので
        10.3 の INV がそこで成り立ち、この場合の仮定 (`T_ρ(x, λ)` は所有される) より
        `n_out(τ', T_ρ(x, λ)) = n_in(τ0, T_ρ(x, λ))` である。
        BY L27, DEF 対応する位置, 10.3 の INV, <4>1
      <4>3. `τ'` から `p` までに `n_out(・, T_ρ(x, λ))` を減らす事象は無い。
        DEF 由来ごとの義務 より `n_out` を減らす事象は `Release` 節点の leaf の事象と D9 の消費の 2 種で
        ある。`<4>1` よりこの塊はこの読む構文の節点 1 つなので、`τ'` から `p` までの事象はこの節点の
        ものだけであり、`Release` 節点の事象は無い。A26 は「「手放し」は D10 の消費と `Release` の
        両方である。渡す先のある消費を含む」と述べるので、この節点が行う D9 の消費は A26 の言う手放しで
        ある (第 9.1 節の A26 の読み)。よって A26 より、この節点の記憶域からの読みは、この節点が行う
        どの参照の手放しよりも前に起きる。**A26 の主語は節点なので、この節点が複数の段を持つ場合にも
        この順序が当たる** (第 9.1 節)。D7 はオブジェクトを読むことを「そのオブジェクトが占める記憶域の
        うち、参照カウントと状態バイトを除いた部分を読むこと」と定めるので、固定した読みはその動作で
        あり、`<1>1a` より `p` はその直前の点である。
        BY A26, D7, D9, D10, DEF 由来ごとの義務, 第 9.1 節, <1>1a, <4>1
      <4>4. QED
        `<4>2` と `<4>3` より `n_out(p, T_ρ(x, λ)) ≥ 1` である。`<3>0` を `τ = p` について当てる。
        BY <3>0, <4>2, <4>3
    <3>1a. CASE 読む構文の読みであって、その節点が `Let(x', App(callee', args), k)` である。
      <4>1. `p` における `n_out` は、この節点の入口 -- 塊の (A-前) の `Retain` 節点をすべて実行した
            後の時点 -- におけるそれに等しい。
        L29 (b) より、`App` の節点が行う読みの直前の点とその節点の入口のあいだにこの活性化の事象は
        1 つも無く、その 2 点で `n^ι_C` は等しい。DEF 塊 と L16 より `App` の塊は (A-前) の `Retain` の
        列・`App` の節点・(A-後) の `Release` の列であり、`App` の節点の入口はその 3 つのうち第 1 の列を
        終えた点である。
        BY DEF 塊, DEF 時点, L16, L29
      <4>2. `n_out(p, T_ρ(x, λ)) ≥ 1` である。
        `<1>1` より (i) の位置は `B_V` でも `App` の位置である。L27 の (b) より、`B_V` のその節点の
        入口 `τ0` で `n_in(τ0, T_ρ(x, λ)) ≥ 1` である。DEF 対応する位置 より `τ0` には `B'_V` のこの
        節点の塊の入口が対応し、この場合の仮定と 10.3 の INV よりそこで
        `n_out = n_in(τ0, T_ρ(x, λ)) ≥ 1` である。塊の入口から `p` までの事象は `<4>1` と L16 より
        (A-前) の `Retain` の事象だけであり、DEF 由来ごとの義務 より `Retain` の事象はどの由来も
        減らさない。
        BY DEF 対応する位置, DEF 由来ごとの義務, L16, L27, <1>1, 10.3 の INV, <4>1
      <4>3. QED
        `<3>0` を `τ = p` について当てる。
        BY <3>0, <4>2

      **この場合は、呼び出し先の活性化が作られる点まで `H` が下がらないことも与える。** この節点が `p`
      から呼び出しまでに行う事象は D9 の `App` の行の消費だけで、D10 よりそれは渡す先のある消費であって
      `H` を変えない。その区間は D24 の (E3) の 1 つの段の中にあり、段は不可分なので (A17 (iii)) その間に
      環境も別の制御の流れも動かない。よって `<3>0` が `p` で与える `H(p)(obj(x, λ)) ≥ 1` は、呼び出し先の
      活性化が作られる点でも成り立つ。そこから先の読みは呼び出し先の本体の読む構文が行う読みであり
      (D32 の (読み-1))、その本体について D11 の (S-c) が課す条件である。
    <3>2. CASE `Retain(v, π)` が leaf `λ` のオブジェクトに触れる動作 (`x = v`)。
      <4>1. この節点は `B'_V` に在るので、DEF 塊 と L16 の (K) よりその塊はこの節点 1 つであり、
            DEF 対応する位置 よりその塊の入口 `τ'` には `B_V` の同じ `Retain` 節点の入口 `τ0` が
            対応する。`<1>1` より (ii) の位置は `B_V` でも同じ `Retain` の位置である。
        BY DEF 塊, DEF 対応する位置, L16, <1>1
      <4>2. `n_out(τ', T_ρ(v, λ)) ≥ 1` である。
        L27 の (b') より `n_in(τ0, T_ρ(v, λ)) ≥ 1` である。`τ0` は DEF 対応する位置 が挙げる位置なので
        10.3 の INV がそこで成り立ち、この場合の仮定 (`T_ρ(v, λ)` は所有される) より
        `n_out(τ', T_ρ(v, λ)) = n_in(τ0, T_ρ(v, λ))` である。
        BY L27, DEF 対応する位置, 10.3 の INV, <4>1
      <4>3. `τ'` から `p` までに `n_out(・, T_ρ(v, λ))` を減らす事象は無い。
        `<4>1` よりこの塊は `Retain` 節点 1 つであり、D10 のその行は `π` の下の inhabited な各 leaf に
        つき参照を 1 つ加えるので、DEF 由来ごとの義務 よりその事象はどの由来も減らさない。
        BY D10, DEF 由来ごとの義務, <4>1
      <4>4. QED
        `<1>1a` より `p` はその leaf の事象の直前の時点である。`<4>2` と `<4>3` より
        `n_out(p, T_ρ(v, λ)) ≥ 1` である。`<3>0` を `τ = p` について当てる。
        BY <1>1a, <3>0, <4>2, <4>3
    <3>3. CASE `Release(v, π)` が leaf `λ` のオブジェクトに触れる動作 (`x = v`)。`<1>1a` より `p` はその
          leaf の事象の直前の時点である。DEF 由来ごとの義務 よりこの事象は `n_out(・, T_ρ(v, λ))` を
          1 減らすので、10.4 より `n_out(p, T_ρ(v, λ)) ≥ 1` である。`<3>0` を `τ = p` について当てる。
      BY DEF 由来ごとの義務, <1>1a, <3>0, 10.4
    <3>4. QED
      `<1>1` より (i) と (ii) の動作はこの 4 つで尽きる -- (i) の読む構文の節点は `App` かそれ以外かで
      割れ、(ii) は `Retain` か `Release` である。
      BY <1>1, <3>1, <3>1a, <3>2, <3>3
  <2>4. `obj(x, λ)` が計数下で `T_ρ(x, λ)` が所有されないとき。L19 より、パラメータでも capture でもない
        変数を持つ由来は所有されるので、所有されない由来 `T_ρ(x, λ) = (u, σ)` の `u` は `V` の
        パラメータか capture である。D6 より `σ` は `ty(u)` の inhabited な boxed leaf であり、P1 は
        **A10 を満たす**型についての言明で、A10 はプログラムに現れる型の全体についてそれを与えるので、
        P1 より `trunc(ty(u), σ) ∈ units(ty(u))` である。第 8 節の系 3 はその仮説 -- unit であること --
        の下で立つ。よって系 3 と P7e より `owns_object(u, σ)` が偽であることは
        `V` が unit `trunc(ty(u), σ)` を D14 の意味で借用することである。`σ` はその unit の下の
        inhabited な leaf であり、L9 より `obj(x, λ) = obj(T_ρ(x, λ)) = obj(u, σ)` なので、`<1>4` より
        `p` において解放されていない。
    BY A10, D6, D14, L9, L19, P1, P7e, 第 8 節の系 3, <1>4
  <2>5. QED
    BY <2>2, <2>3, <2>4

<1>6. 固定した動作が (iii) の触れる動作、すなわち (A-前) か (A-後) の節点のそれであるとき、それが名指す
      オブジェクトは `p` において解放されていない。
  <2>0. (A-前) と (A-後) が名指す `(a, u)` について、`a` は `App` の引数であり、`(a, u)` は `B_V` の
        site (DEF site) である。`u` の下の inhabited な各 leaf `λ` について、L13 より
        `T_ρ(a, λ)` が所有されることと `arg_owned(i, u)` が真であることとは同値である。
    DEF site は `Let(_, App(_, args), _)` の各 `arg` と各 `unit ∈ rc_units(ty(arg))` の対を site とする。
    (A-前) と (A-後) の対は P11 より `args` の元と `units(ty(args[i]))` の元の対である。
    BY DEF site, L13, P11
  <2>1. (A-前) の `Retain(a, u)` が触れるオブジェクトは `p` において解放されていない。
        P11 より `arg_owned(i, u)` は偽なので、`<2>0` より `u` の下の inhabited な各 leaf `λ` の
        `T_ρ(a, λ)` は所有されない。`obj(a, λ)` がグローバル状態なら `<1>2` による。計数下ならば、
        L19 より所有されない由来 `T_ρ(a, λ) = (u', σ')` の `u'` は `V` のパラメータか capture である。
        D6 より `σ'` は `ty(u')` の inhabited な boxed leaf であり、P1 は **A10 を満たす**型についての
        言明で、A10 はプログラムに現れる型の全体についてそれを与えるので、P1 より
        `trunc(ty(u'), σ') ∈ units(ty(u'))` である。第 8 節の系 3 はその仮説の下で立ち、系 3 と P7e より
        `V` はその unit を D14 の意味で借用するので、`<1>4` による。
    BY A10, D6, D14, L19, P1, P7e, P11, 第 8 節の系 3, <1>2, <1>4, <2>0
  <2>2. (A-後) の `Release(a, u)` が触れるオブジェクトは `p` において解放されていない。
        P11 より `arg_owned(i, u)` は真なので、`<2>0` より `u` の下の inhabited な各 leaf `λ` の
        `T_ρ(a, λ)` は所有される。`<1>1a` より `p` はその leaf の事象の直前の時点であり、
        DEF 由来ごとの義務 よりこの事象は `n_out(・, T_ρ(a, λ))` を 1 減らすので、10.4 より
        `n_out(p, T_ρ(a, λ)) ≥ 1` である。10.4 より他の由来の `n_out` は `p` で非負なので、L17 と L9 より
        `Obl(p)(obj(a, λ)) ≥ 1` であり、`<1>3` が結論を与える。`obj(a, λ)` がグローバル状態なら
        `<1>2` による。
    BY L9, L17, P11, DEF 由来ごとの義務, <1>1a, <1>2, <1>3, <2>0, 10.4
  <2>3. QED
    BY <2>1, <2>2

<1>7. QED
  `<1>1` より、`B'_V` の読む構文が読みうるオブジェクトと `Retain`/`Release` が触れるオブジェクトは
  (i)・(ii)・(iii) で尽き、`<1>5` と `<1>6` よりそのいずれも `p` において解放されていない。`<1>1a` より
  `p` はその読み・その触れる動作が実際に起きる瞬間の直前の点であり、動作は任意でよい。接頭条件つきの
  この形が D11 の (S-c) である。
  BY D11, <1>1, <1>1a, <1>5, <1>6

### 10.8 P14 の QED

<1>1. 10.5、10.6、10.7 より、`B'_V` は出力の割り当ての下で D11 を満たす。
  10.5 は A19 (ii-a) の (a) と (a') を (10.4 経由で)、10.6 は (a') を (L28 経由で)、10.7 は
  A19 (ii-a) の (b)・(b') と A20 と A26 を読む。10.7 の `<1>3` は加えて、D21 が活性化に課す制限
  (A19 (i) の不等式) を読み、10.7 の `<1>2a` は D11 の (S-c) の接頭条件 (D11a) を読む。10.7 が (b) と
  (b') を読むのは節点の入口であり、そこから読み・触れる動作の直前の点へ 10.3 の INV を渡すのは、読む構文に
  ついては A26、`Retain` については `Retain` の事象がどの由来も減らさないこと (D10)、`App` の節点に
  ついては `L29` (b) である。
  この 3 つはさらに `L18` と `L18a` を経て 4 つの仮定を読む。`L18` は A23 と A13 に立ち、第 10 節では
  10.3 の `<1>5` の `<2>1`、10.4 の `<1>2` の `<2>3`、10.7 の `<1>1` がそれを引くので、A23 と A13 は
  主不変条件を経て 10.5・10.6・10.7 のすべてに載る。`L18a` は A24 と A21 と A13 に立ち、第 10 節では
  10.7 の `<1>4` の `<2>1` と、10.3 と 10.4 が引く `L15` の `<1>2` の `<2>2` の `<3>2` がそれを引くので、
  A24 と A21 も 10.5・10.6・10.7 のすべてに載る。
  BY A13, A19, A20, A21, A23, A24, A26, D10, D11, D11a, D21, L18, L18a, L29, 10.3 の INV,
     10.5, 10.6, 10.7

<1>2. QED
  10.1 の言明より、`V` と実行路と活性化は任意でよい。よって出力のすべての本体が D11 を満たし、
  D12 が成り立つ。`<1>1` が読む A19 (ii-a)・A20・A21・A23・A24・A26 と A13 は `README.md` の仮定であり、
  P14 の言明はその下で読むものである。
  BY A13, A19, A20, A21, A23, A24, A26, D12, 10.1, <1>1

## 11. P14a -- 借用する終端の類は活性化の間 参照を持つ

**言明** (README の P14a)。`borrow_ify` の出力の各本体、各実行路、各活性化について、ρ-終端が借用する
(D14) パラメータ・capture の leaf である**計数下**の別名類 (D26) は、活性化の間ずっと参照を少なくとも
1 つ持つ。

### 11.1 固定するものと、言明の読み

第 9.1 節と同じものを固定する -- 出力の版 `V`、`B_V`、`B'_V`、`ctx`、`B_V` の実行路 `ρ`、`ρ` を辿る
活性化である。L16 より `B'_V` の実行路は `B_V` のそれと 1 対 1 に対応し、L16a より `B'_V` の各活性化に
対応する `B_V` の活性化が在る。この命題が読むのは `B'_V` の側の活性化である。

**DEF 別名類** -- `ρ` の上のスロット (D6) を、`T_ρ` (DEF 由来) が等しいという関係で分けた同値類を
**別名類**と呼び、`ρ`-由来 `T` に対応する類を `C_T` と書く。すなわち `(x, λ) ∈ C_T` とは、`(x, λ)` が
`ρ` の上のスロットであって `T_ρ(x, λ) = T` であることである。L9a より これは D33 の別名類であり、`C_T` は D33 の `C_ρ(x, λ)` (`T_ρ(x, λ) = T` である任意の
スロット `(x, λ)` について) である。

**類の元は増える**。D6 のスロットは「その実行路の上でその時点までに値を得た変数」の leaf なので、`C_T` の
元は `ρ` の上で増える。増える元は参照を持ち込まない -- 新しい元が `C_T` に入るのは DEF 由来の 1 歩 の
辺によってであり、その 6 行は D20 の別名の辺すなわち D9 の移動の 6 行であって、移動は義務集合を変えず
(D9)、DEF 由来ごとの義務 も移動の構文を挙げていないからである。

**時点**は 第 9.7 節の DEF 時点 による。`C = B'_V` について読み、`n_out(τ, T)`
(DEF 由来ごとの義務、DEF n_in、n_out) はその意味の時点について読む。最後の時点は終端の `Ret` の消費の
直後であり、これは DEF 対応する位置 が挙げる最後の位置である。

**DEF 類の参照** -- **`obj(C_T)` が計数下 (D26) である** `ρ`-由来 `T` と時点 `τ` について
`held(τ, C_T) := n_out(τ, T) + β(T)` と定める。`β(T)` は、`T = (p, σ)` で `p` が `V` のパラメータか
capture であり `ctx.owns_object(p, σ)` が偽であるとき 1、そうでないとき 0 である。`obj(C_T)` が計数下で
ない `T` については定めない -- DEF 由来ごとの義務 がそのような `T` について `n^ι_C` を定めないからで
ある。L21 (a) が、これが D34 の `held_ρ(τ, C_T)` と同じものであることを示す。

**示す形**。P14a が言う「ρ-終端が借用する (D14) パラメータ・capture の leaf である**計数下**の別名類
(D26)」の ρ-終端の側は、L20 より `ctx.owns_object(T)` が偽である `ρ`-由来 `T` である。よって
`ctx.owns_object(T)` が偽であり `obj(C_T)` が計数下である `T` を 1 つ固定して、次の 3 つを示す。
`held` も `n_out` も、その `T` についてだけ定まる (DEF 類の参照、DEF 由来ごとの義務)。

- **(A)** そのような類が在るのは `V` が借用版であるときだけであり、そのとき `T` は `V` のパラメータの
  leaf である (第 11.2 節の L20)。
- **(B)** `obj(C_T)` が計数下 (D26) であるとき、`n_out(τ, T)` は DEF 対応する位置 の各位置で 0 であり、
  塊の中を含むどの時点でも非負である (第 11.6 節の INV-a)。DEF 類の参照 と L20 より これは
  `held(τ, C_T) ≥ 1` である。
- **(C)** A20 の下で、`held(τ, C_T)` はその時点で未処分の参照の個数である。よって `obj(C_T)` が計数下で
  あるとき `C_T` は、活性化が生きている (D23) 間のどの時点でも D8 の意味の参照を少なくとも 1 つ持つ
  (第 11.7 節)。DEF 時点 が挙げない点 -- この活性化が行った D10 の事象の直後でも、活性化の開始でも、
  読みの直前でもない点 -- を含む。`obj(C_T)` が計数下でないときは、README の P14a も この節も その類を
  主語にしない。

### 11.2 借用する終端の在りか

#### L20 (借用する終端は借用版のパラメータ leaf である)

**言明**。`T = (u, σ)` を `ρ`-由来とする。次の 2 つは同値である。

1. `ctx.owns_object(u, σ)` が偽である。
2. `V` は借用版 `f_borrow` であり、`u` は `V` のあるパラメータ `p` の名前であり、`V` は unit
   `trunc(ty(p), σ)` を D14 の意味で借用する。

<1>1. 1 ならば `V` は借用版である。
  BY L6, L8, DEF 出力の版
  DEF 出力の版 と L6 より、出力の本体は各 `f_own`、各 `f_borrow`、各グローバル初期化子で尽きる。L8 より
  `V` が `f_own` かグローバル初期化子であるとき `ctx.owns_object` は値を返すどの `(r, p)` についても
  真である。

<1>2. 1 ならば `u` は `V` のパラメータか capture の名前である。
  BY L19
  L19 (パラメータでない由来は所有される) の対偶である。

<1>3. `f_borrow` は capture を持たない。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, clone_func, CODE src/rc_ir/rename.rs: fresh_rename_function
  `borrow_ify` が `borrow_versions` に入れるのは `func.capture.is_none()` である関数だけであり、
  `clone_func` は `capture` を `fresh_rename_function` の `new_cap = cap.as_ref().map(..)` から取るので、
  `None` は `None` のままである。

<1>4. `σ ∈ leaves(ty(p))` であり `trunc(ty(p), σ) ∈ units(ty(p))` である。
  BY A10, D6, P1, DEF 由来の 1 歩
  DEF 由来の 1 歩 より `ρ`-由来は `ρ` の上のスロットであり、D6 より `σ` は `ty(p)` の inhabited な
  boxed leaf である。P1 は **A10 を満たす**型についての言明であり、A10 はプログラムに現れる型の全体に
  ついてそれを与えるので、`ty(p)` に当たる。P1 より各 leaf の `trunc` は `units(ty(p))` の元である。

<1>5. `ctx.owns_object(p, σ)` が偽であることと、`V` が `(p, trunc(ty(p), σ))` を D14 の意味で借用する
      こととは同値である。
  BY D14, P7e, 第 8 節の系 3, <1>4
  P7e より `owns_object(p, σ) = owns_object(p, trunc(ty(p), σ))` である。`<1>4` より
  `trunc(ty(p), σ) ∈ units(ty(p))` なので、第 8 節の系 3 よりそれは `V` がその unit を D14 の意味で所有
  することと同値である。D14 より各 unit は所有か借用のどちらかであり、両方ではない。

<1>6. QED
  1 から 2 へは `<1>1`・`<1>2`・`<1>3`・`<1>5` が与える。2 から 1 へは `<1>5` が与える。
  BY <1>1, <1>2, <1>3, <1>5

**注**。`V` のパラメータ・capture の inhabited な leaf `(p, σ)` は常に `ρ`-由来である。`VarTable::of` は
パラメータと capture に `Binding::Param` を入れ、DEF 由来の 1 歩 の表に `Binding::Param` の行は無い
(`CODE src/rc_ir/ownership.rs: VarTable::of`, `origin_inner`)。よって `V` が借用する unit の下の
inhabited な leaf はどれも L20 の 2 を満たし、P14a の主語である類はちょうどこの `T` の類である。

#### L21 (`held` は D34 の `held_ρ` である)

**言明**。`obj(C_T)` が計数下 (D26) である `ρ`-由来 `T` について、次の 2 つが成り立つ。

- **(a)** `C_T` の開始の時点 (D34) 以後の時点 `τ` について、`held(τ, C_T)` は D34 が定める
  `held_ρ(τ, C_T)` に等しい。
- **(b)** D34 の表の第 4・第 5・第 6 行の事象が `held_ρ(・, C_T)` を動かす段内の点 (D24) と、
  DEF 時点 がその事象の直後に置く時点とのあいだに、D34 の表のどの行の事象も起きない。したがって
  その段内の点における `held_ρ(・, C_T)` は、その時点における値に等しい。

D34 は計数下の類にだけ `held_ρ` を定めるので、それ以外の類については言明が無い。

**第 11.6 節と第 11.7 節が読むのは、`ctx.owns_object(T)` が偽である `T` の類だけである** -- 第 11.6 節の
INV-a はその `T` について立て、第 11.7 節の `<1>2` は L20 から `β(T) = 1` を取る。L20 よりその類の終端は
`V` が借用する unit の下のパラメータ leaf であり、D34 の第 3 行に当たる。D34 は第 3 行の開始値を
「**その活性化が生きている活性化 (D23) になる点**に置く。」と定めるので、DEF 時点 が挙げるどの時点も
その点より後にある。よってその類については、言明の条件は DEF 時点 が挙げるすべての時点に当たる。

**読みの直前の点における `held_ρ`** は、その点より前の最後の段内の点 (D24) での値として読む。D34 が
`held_ρ` を定めるのは時点と段内の点についてであり、DEF 時点 が挙げる読みの直前の点はそのどちらとも限らない。
段内の点は段の素動作の列の切れ目なので、その最後の切れ目と読みのあいだに素動作は 1 つも無く、D34 の表の
どの事象もそこでは起きない。

<1>1. 事象の 3 行 (`Retain`・`Release`・消費) は対応し、それが `held_ρ` を動かす段内の点と、
      DEF 時点 がその事象の直後に置く時点とのあいだに、D34 の表のどの行の事象も起きない ((b))。
  BY D6, D10, D24, D25, D34, DEF 時点, DEF 別名類, DEF 由来ごとの義務
  D34 の第 4 行は「`Retain(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ」につき
  「その `λ` 1 つにつき +1」であり、DEF 由来ごとの義務 の `Retain` の行は「`π` の下の inhabited な各
  leaf `λ` につき `n^ι_C(・, T_ρ(v, λ))` を 1 増やす」である。D6 より `(v, λ)` がスロットであることは
  `λ` が inhabited であることであり、DEF 別名類 より `(v, λ) ∈ C_T` は `T_ρ(v, λ) = T` である。
  第 5 行 (`Release`) と第 6 行 (`(w, μ) ∈ C_T` の消費) も同じ対応である。

  **動かす点も同じである。** D34 は「表の第 4・第 5・第 6 行の事象は、**それを運ぶ素動作の直後の段内の
  点**で `held` を動かす。」と定め、DEF 時点 はその 3 種を事象と呼んでその直後の点を時点とする。
  D24 は段内の点を「段の素動作を時間の順に並べたときの、最初の素動作の前の点と各素動作の後の点」と定め、
  素動作を「参照の受け渡し、生成、割り当て、処分、解放、グローバル化の 6 種」とする。`Retain` の事象は
  生成の素動作、`Release` の事象と捨てる消費の事象は処分の素動作、渡す先のある消費の事象は受け渡しの
  素動作である (D10 の消費の行が `H` の動きでその 2 つを分ける)。よって DEF 時点 が置く「事象の直後の
  点」は、その素動作の直後の点である。**切れ目が抑えられる場合も、そのあいだに表の事象は無い** --
  D24 が点を除くのは「処分とその処分が起こす解放のあいだ」と「素動作とそれに付随する書き込みのあいだ」の
  2 つだけであり、解放も書き込みも D34 の表のどの行の事象でもない (解放が処分する参照の持ち手は D25 の
  2 番目のオブジェクトであって、この活性化の構文が行う事象ではない)。よって D34 が名指す段内の点は
  DEF 時点 の置く時点そのものか、そのあいだに表の事象を 1 つも挟まない点である。

<1>2. `ρ`-由来 `T = (u, σ)` は次の 3 つのいずれかである。(i) `u` が `V` のパラメータか capture である、
      (ii) `u` が `ctx.vars.bindings` の鍵でない、(iii) D10 の生成の表のいずれかの行が `u` に値を与える。
  BY D10, DEF 由来の 1 歩, CODE src/rc_ir/ownership.rs: origin_inner, collect_bindings, VarTable::of
  DEF 由来の 1 歩 の表の 6 行のどれにも当たらないのは、`bindings.get(u)` が `None`、`Some(Param)`、
  `Some(Producer)`、boxed 容器の `Some(Field(..))`、boxed scrutinee の `Some(Payload(_, Some(_)))`、
  宣言が単一の `Arg` でない `Some(Llvm(..))` の 6 つである。`VarTable::of` が `Binding::Param` を入れるのは
  パラメータと capture についてだけなので第 2 の腕が (i) であり、`None` は束縛表が持たない名前なので
  (ii) である。残る 4 つの腕に `collect_bindings` が入れる変数は、順に `RcRhs::App` と `RcRhs::Closure` の
  `Let` の束縛変数、boxed 容器の `Destructure` のフィールド変数、boxed scrutinee の変位アームの payload
  変数、`RcRhs::Llvm` の `Let` の束縛変数であり、D10 の生成の表の 5 行が値を与える変数はちょうどこれらで
  ある。

<1>3. (iii) の場合、生成の行が対応する。
  BY D10, D34, DEF 時点, DEF 由来ごとの義務, L9, 4.3 の系, <1>2
  D34 の第 1 行は「`C` の終端が D10 の生成で作られる」とき 1 から始まるとし、その開始値は
  「`T_ρ(C) = (u, σ)` の変数 `u` が値を得る時点」-- `u` を束縛する節点を実行する段の直後 -- に置かれる。
  段内の点については、D34 は「第 1 行と第 2 行の開始値は、**その類の終端の参照が `Obl(a)` に入る素動作の
  直後の段内の点**に置く。」と定める。D10 の生成の各行が参照を `Obl` に加える動作がその素動作であり、
  それは DEF 由来ごとの義務 が 1 を置く事象と同じ点である。以下は、粗い方の読み -- 段の直後 -- でも
  2 つが一致することを述べる。
  DEF 由来ごとの義務 は パラメータ・capture でない `T = (u, σ)` について「D10 の生成の表のいずれかの
  行が `u` に値を与える事象で 1 になる。その前は 0 である」とする。`<1>2` よりこの 2 つは同じ場合を
  指し、同じ開始値を置く。**置く点は節点の内側でずれる** -- DEF 時点 は節点の中の事象と事象のあいだの
  点も時点に数え、生成の事象はその節点を実行する段の中にあるからである (boxed 容器の `Destructure` は
  1 つの節点の中で容器の消費の事象とフィールドの生成の事象を持つ)。**ずれる区間で `n_out(・, T)` を
  動かす事象は無い。** `u` はこの節点が束縛する変数であり、この節点が生成の事象より後に行う事象が
  当たる由来は 2 種である -- この節点が消費する変数のスロットの由来 (その変数はこの節点より前に値を
  得ており、L9 より 1 歩の先も同じ時点のスロットなので、その由来の変数も `u` ではない) と、この節点が
  束縛する別の変数の対 (4.3 の系 より出力の束縛名は互いに相異なる)。よって言明が量化する範囲 --
  D34 の開始の時点以後 -- では 2 つは一致する。このとき `β(T) = 0` である。

<1>4. (i) の場合、初期値の行が `β` の分だけ違う。
  BY D23, D34, DEF 時点, DEF n_in、n_out, DEF 由来ごとの義務, DEF 類の参照, L20
  L20 より、`ctx.owns_object(p, σ)` が偽であることは `V` がその unit を D14 の意味で借用することである。
  よって `T = (p, σ)` は、偽のとき D34 の第 3 行 (借用するパラメータ・capture の leaf) に、真のとき
  第 2 行 (所有するそれ) に当たる。どちらの行も 1 から始まる。
  **開始値を置く点は行ごとに別である** -- D34 は「第 1 行と第 2 行の開始値は、**その類の終端の参照が
  `Obl(a)` に入る素動作の直後の段内の点**に置く。」「第 3 行の開始値は、**その活性化が生きている活性化
  (D23) になる点**に置く。」と定める。第 2 行のその素動作はこの活性化の初期 `Obl` を作る受け渡しであり、
  第 3 行のその点は活性化が始まる点である。DEF 時点 が挙げる最初の時点は活性化の開始、すなわち D10 の
  初期値が置かれた直後なので、どちらの点もそれ以前にあり、DEF 時点 が挙げるどの時点でも開始値は
  置かれている。D34 は「時点においては、その段の素動作はすべて済んでいるのでこの読みは前段落と一致
  する。」と述べるので、その各時点での値は 1 である。
  一方 `ι_V` は `ctx.owns_object(p, σ)` が真の leaf に 1、偽の leaf に 0 を与え、DEF 由来ごとの義務 は
  それを `T` の初期値に取る。DEF 類の参照 より `β(T)` は真のとき 0、偽のとき 1 なので、どちらでも
  `n_out + β = 1` である。

<1>5. QED
  BY D26, D34, DEF 類の参照, DEF 由来ごとの義務, <1>1, <1>2, <1>3, <1>4
  `<1>2` の (ii) -- `u` がグローバル値の名前である場合 -- は言明の外にある。DEF 由来ごとの義務 が
  「`u` が `ctx.vars.bindings` の鍵でないとき、D26 より `obj(u, σ)` はグローバル状態なので、この由来は
  勘定の外にある」と述べるとおり、その類は計数下でないからである。残る (i) と (iii) について、`<1>3` と
  `<1>4` が開始の時点と開始値を、`<1>1` が 3 つの事象の行を与える。どちらの側も移動の構文を挙げて
  いないので、開始の時点以後の各 `τ` で残る差は無い。これが (a) である。(b) は `<1>1` の後半である。

### 11.3 類の参照を減らす事象の数え上げ

#### L22 (`n_out` を動かす事象)

**言明**。`ρ` に対応する `B'_V` の実行路の上で、活性化の開始より後に、ある `ρ`-由来 `T` について
`n_out(・, T)` を

- **減らす**事象は、(α) `Release(v, π)` 節点 -- `π` の下の inhabited な各 leaf `λ` につき
  `n_out(・, T_ρ(v, λ))` を 1 減らす -- と、(β) D9 の消費 -- 消費される inhabited な各 leaf `(w, μ)` に
  つき `n_out(・, T_ρ(w, μ))` を 1 減らす -- の 2 種だけである。
- **増やす**事象は、`Retain(v, π)` 節点と D10 の生成の 2 種だけである。

さらに、D9 の消費を行う構文は、`Let(x, App(callee, args), k)`、`Let(x, Closure(f, caps), k)`、
`Let(x, Llvm(gen, args), k)`、`Destructure(c, fs, s, k)`、関数本体の終端の `Ret(x)` の 5 つで尽きる。

<1>1. DEF 由来ごとの義務 の 5 つの行のうち、`n^ι_C` を減らすのは `Release` の行と消費の行、増やすのは
      初期値の行 (`ι` が 1 を与えるとき) と生成の行と `Retain` の行である。他のどの節点も `n^ι_C` を
      変えない。初期値の行が当たるのは活性化の開始の 1 度だけである。
  BY DEF 由来ごとの義務

<1>2. `RcExpr` の 6 種と `RcRhs` の 5 種のそれぞれについて、D9 の 2 つの表と D10 の生成の表がどの行を
      当てるかを並べる。
  <2>1. `Let(x, Var(y), k)`。D9 の移動の表の第 1 行 (`y` の参照が `x` へ) だけが当たる。消費も生成も
        行わない。
    BY D9, D10
  <2>2. `Let(x, App(callee, args), k)`。D9 の消費の表の第 1 行 (callee の全 boxed leaf と、呼び出し先が
        その位置の unit を所有する引数の leaf) と、D10 の生成の表の `App` の行 (結果の各 boxed leaf) が
        当たる。
    BY D9, D10
  <2>3. `Let(x, Closure(f, caps), k)`。D9 の消費の表の第 2 行 (各 capture の全 boxed leaf) と、D10 の
        生成の表の `Closure` の行 (capture object) が当たる。
    BY D9, D10
  <2>4. `Let(x, Llvm(gen, args), k)`。D9 の消費の表の第 3 行と、D10 の生成の表の `Llvm` の行、および
        D9 の移動の表の第 6 行 (素通し leaf) が当たる。
    BY D9, D10
  <2>5. `Let(x, Match(scrut, arms), k)`。`Match` の節点自身は参照を作らず、移さず、手放さない (D9)。
        変位アームの payload 束縛は、boxed scrutinee のとき D10 の生成の表の第 5 行、unbox scrutinee の
        とき D9 の移動の表の第 4 行であり、catch-all アームの payload 束縛は移動の表の第 5 行、アーム
        本体の `Ret` は移動の表の第 2 行である。
    BY D9, D10
  <2>6. `Retain(v, π, s, k)`。D10 の `Retain` の行が当たる。増やす側である。
    BY D10
  <2>7. `Release(v, π, s, k)`。D10 の `Release` の行が当たる。これが (α) である。
    BY D10
  <2>8. `Destructure(c, fs, s, k)`。`c` が boxed のとき D9 の消費の表の第 4 行と D10 の生成の表の第 4 行、
        `c` が unbox のとき D9 の消費の表の第 5 行 (名前の付いていないフィールドの leaf) と D9 の移動の
        表の第 3 行 (名前付きフィールド) が当たる。
    BY D9, D10
  <2>9. `Eval(v, k)`。参照を作らず、移さず、手放さない (D9)。
    BY D9
  <2>10. `Ret(v)`。関数本体の終端であるとき D9 の消費の表の第 6 行が当たる。`Match` のアーム本体の
         `Ret` であるときは `<2>5` の移動である。
    BY D9
  <2>11. QED
    D2 より `RcExpr` は `Let`・`Retain`・`Release`・`Destructure`・`Eval`・`Ret` の 6 種であり、`Let` の
    右辺 `RcRhs` は `Var`・`App`・`Closure`・`Llvm`・`Match` の 5 種である。`<2>1`-`<2>10` は
    「`Let` を右辺で 5 つに割ったもの + 残る 5 種」の 10 通りを尽くす。D9 の消費の表の 6 行は、`App` の
    行を `<2>2` が、`Closure` の行を `<2>3` が、`Llvm` の行を `<2>4` が、`Destructure` の 2 行を
    `<2>8` が、終端の `Ret` の行を `<2>10` が持つ。
    BY D2, D9, D10, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, <2>8, <2>9, <2>10,
       CODE src/rc_ir/ast.rs: RcExpr, RcRhs

<1>3. 移動は `n_out` を変えない。
  BY D9, DEF 由来ごとの義務, L9
  D9 より移動は義務集合を変えず、DEF 由来ごとの義務 も移動の構文を挙げていない。L9 より移動の前後の
  2 つのスロットの由来は同じなので、移動は `C_T` の元を増やしても `n_out(・, T)` を動かさない。

<1>4. QED
  BY <1>1, <1>2, <1>3
  `<1>1` が DEF 由来ごとの義務 の行の側の数え上げを、`<1>2` が構文の側の数え上げを与える。

### 11.4 (α) `Release` は借用する由来を減らさない

#### L23 (`Retain`/`Release` の対象と `App` の引数 unit は site である)

**言明**。次の 2 つが成り立つ。

- **(a)** `B_V` の `Retain(v, π, s, k)` 節点と `Release(v, π, s, k)` 節点について、`π ∈ units(ty(v))` で
  あり、`(v, π)` は `B_V` の site (DEF site) である。
- **(b)** `B_V` の `Let(x, App(callee, args), k)` 節点の各引数 `args[i]` と各 `u ∈ units(ty(args[i]))` に
  ついて、`(args[i], u)` は `B_V` の site である。

<1>1. (a) の `π ∈ units(ty(v))` である。
  BY A2, P9, CODE src/rc_ir/rename.rs: rename_var
  `V` が `f_own` かグローバル初期化子のとき `B_V` は入力の本体であり、A2 がそのまま与える。`V` が
  `f_borrow` のとき `B_V` は入力の本体を `ρ_f` で付け替えたものであり、P9 の前半より複製は `FieldPath` を
  変えず、`rename_var` は `ty` を残すので、複製の節点の path もその変数の型の unit である。

<1>2. QED
  BY A15, DEF site, <1>1, CODE src/rc_ir/ast.rs: for_each_node, CODE src/misc.rs: grow_stack
  `for_each_node` は本体の全節点を 1 度ずつ歩く。本体を `grow_stack` で包んで歩くので、A15 より包まない
  場合と同じ回数だけ各節点を訪れる。DEF site が site とするのは、その歩みが訪れる
  `RcExpr::Retain(v, path, _, _)` と `RcExpr::Release(v, path, _, _)` の節点の `(v, path)` ((a)) と、
  `RcExpr::Let(_, RcRhs::App(_, args), _)` の節点の各 `arg` と各 `unit ∈ rc_units(ty(arg))` の対 ((b))
  である。`B_V` は `V` が書き換える本体なので、その 2 種の節点はこの歩みが訪れる。

**関数の版では `levelled_sites` がこの集合を計算する**。`levelled_sites` は `for_each_node` で
`func.body` の全節点を歩き、`RcExpr::Retain` と `RcExpr::Release` の腕で `(v.clone(), path.clone())` を、
`RcExpr::Let(_, RcRhs::App(_, args), _)` の腕で各 `arg` と各 `unit ∈ rc_units(&arg.ty, type_env)` に
ついて `(arg.clone(), unit)` を積む (`CODE src/rc_ir/borrow.rs: levelled_sites`)。それが `&RcFunc` を
取ることと、グローバル初期化子の版について site を 1 つも挙げないことは P7d が述べる。

#### L24 (`B'_V` の `Retain`/`Release` 節点は借用する由来を減らさない)

**言明**。`T` を `ctx.owns_object(T)` が偽である `ρ`-由来とする。`B'_V` の実行路の上の
`Retain`/`Release` 節点のうち、`n_out(・, T)` を動かしうるのは L16 の (A-前) の `Retain` だけである。
とくに **`n_out(・, T)` を減らす `Release` 節点は `B'_V` に無い。**

<1>1. `V` は借用版である。
  BY L20

<1>2. L16 より `B'_V` の `Retain`/`Release` 節点は (K)・(A-前)・(A-後) の 3 種である。
  BY L16

<1>3. CASE (K) の節点 `Retain(v, π)` または `Release(v, π)`。
  <2>1. `ctx.owns_unit(v, π)` は真である。
    BY L16, <1>1
    L16 の (K) は「`B_V` の `Retain`/`Release` 節点のうち、`V` が借用版でないか `owns_unit(v, π)` が
    真であるもの」であり、`<1>1` より第 1 の場合には当たらない。
  <2>2. `π` の下の inhabited な各 leaf `λ` について `ctx.owns_object(T_ρ(v, λ))` は真である。
    BY L13, L23, <2>1
    L23 (a) より `(v, π)` は `B_V` の site (DEF site) であり、
    `π ∈ units(ty(v))` である。
  <2>3. QED
    BY D10, <2>2
    D10 の `Retain`/`Release` の行が動かすのは `π` の下の inhabited な各 leaf の由来であり、`<2>2` より
    それはすべて所有される。`ctx.owns_object(T)` は偽なので、そのどれも `T` ではない。

<1>4. CASE (A-後) の `Release(a, u)`。
  BY D10, L13, L23, P11
  P11 よりこの節点が置かれるのは `arg_owned(i, u)` すなわち `ctx.owns_unit(a, u)` が真である対に
  ついてだけであり、`(a, u)` は `args[i]` と `units(ty(args[i]))` の元の対である。L23 (b) より
  `(a, u)` は site なので、L13 より `u` の下の inhabited な各 leaf の由来は所有される。D10 より
  この節点が動かすのはその由来だけであり、`T` は所有されないので、そのどれも `T` ではない。

<1>5. CASE (A-前) の `Retain(a, u)`。この節点は `n_out` を増やす側であり (D10)、減らさない。
  BY D10

<1>6. QED
  BY <1>2, <1>3, <1>4, <1>5
  `<1>2` の 3 種を `<1>3`・`<1>4`・`<1>5` が尽くす。動かしうるのは `<1>5` の (A-前) の `Retain` だけで
  あり、`Release` 節点は `<1>3` と `<1>4` のどちらでも `T` を動かさない。

**この補題が (α) を閉じる。** L22 の (α) -- `Release` 節点 -- は、借用する由来については `B'_V` に 1 つも
現れない。落ちる先は 2 つある。`B_V` に在った `Release(v, π)` で `owns_unit(v, π)` が偽のものは
`rewrite_rc` が落とし (P10、L16 の (K) の条件)、`call_rc` が置く `Release` は `arg_owned` が真の対に
だけ置かれる (P11)。どちらの判定も `owns_unit` を通るので、L13 によって由来の所有と一致する。

### 11.5 (β) 借用する由来を減らす消費は `App` の引数の位置だけである

#### L25 (所有を読まない消費は借用する由来を減らさない)

**言明**。`T` を `ctx.owns_object(T)` が偽である `ρ`-由来とする。`B'_V` の実行路の上で、所有を読まない
消費 (DEF 所有を読まない消費) が消費するスロット `(w, μ)` は `C_T` に属さない。

<1>1. `B'_V` のこれらの節点が消費するスロットは、`B_V` の対応する節点が消費するスロットと同じである。
  BY A3, D9, L16, L18, CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand, LLVMGen::result_prov
  L16 より `Retain`/`Release` 以外の節点の列は、`App` 節点の callee の名前を除いて両側で等しい。
  callee については L18 が場合を分ける -- `route` が名前を差し替えたときは `leaves(ty(c)) = ∅` なので
  D9 の `App` の行の callee の部分はどちらの側でも何も消費せず、差し替えないときは両側の callee が同じ
  `RcVar` なので同じ leaf を消費する。`Llvm` の節点については、L16 の言明が「両側の op は別のオブジェクト
  でありながら、同じオペランドの列と同じ型を持ち、`borrows_operand` と `result_prov` に同じ値を返す」と
  述べ、D9 の消費の表の `Llvm` の行はその 2 つの宣言だけを読むので、消費される leaf も両側で同じである。

<1>2. `B_V` の節点で所有を読まない消費によって消費されるスロット `(w, μ)` について
      `ctx.owns_object(T_ρ(w, μ))` は真である。
  BY L14

<1>3. QED
  BY DEF 別名類, <1>1, <1>2
  `ctx.owns_object(T)` は偽なので `T_ρ(w, μ) ≠ T` であり、DEF 別名類 より `(w, μ) ∉ C_T` である。

#### L26 (`App` の塊は借用する由来について収支が合い、増分が先に来る)

**言明**。`T` を `ctx.owns_object(T)` が偽である `ρ`-由来とし、`B_V` の節点
`Let(x, App(callee, args), k)` の塊 (DEF 塊) を 1 つとる。この塊が `n_out(・, T)` に与える増分の総和と
減分の総和は等しく、増分の事象はすべて減分の事象より前に起きる。

<1>1. この塊は、(A-前) の `Retain` の列、`App` の節点、(A-後) の `Release` の列であり、(A-前) の節点は
      `App` の節点より前に、(A-後) の節点は後に実行される。
  BY L16, P11, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, prepend_rc
  `rewrite_inner` の `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕は `prepend_rc(before, false, app)`
  を返し、`app` は `App` の `Let` 節点である。`prepend_rc` は `before` の節点で `app` を包むので、その
  節点は `App` より前に実行される。`after` の節点は `App` の `Let` の継続の側に置かれる。

<1>2. この塊が `n_out(・, T)` を増やす事象は、(A-前) の `Retain` だけである。
  <2>1. L22 より、増やすのは `Retain` 節点と D10 の生成である。`<1>1` よりこの塊の `Retain` 節点は
        (A-前) だけである。
    BY L22, <1>1
  <2>2. `App` の結果の生成は `T` を増やさない。
    BY 4.3 の系, D10, L20, CODE src/rc_ir/ownership.rs: collect_bindings
    D10 の生成の `App` の行が値を与えるのは `Let` の束縛変数 `x` である。L20 より `T = (p, σ)` の `p` は
    `V` のパラメータの名前であり、4.3 の系 より出力の束縛名は互いに相異なるので `x ≠ p` である。よって
    この生成が 1 にする由来 `(x, ・)` は `T` ではない。
  <2>3. QED
    BY <2>1, <2>2

<1>3. この塊が `n_out(・, T)` を減らす事象は、`App` の節点が行う引数の位置の消費だけである。
  <2>1. L22 より、減らすのは `Release` 節点と D9 の消費である。`<1>1` よりこの塊の `Release` 節点は
        (A-後) だけであり、L24 よりそれは `T` を動かさない。
    BY L22, L24, <1>1
  <2>2. `App` の callee の消費は `T` を減らさない。
    BY L25
    D9 の `App` の行の callee の部分は DEF 所有を読まない消費 に入る。
  <2>3. QED
    BY D9, <2>1, <2>2
    D9 の `App` の行は callee の部分と引数の部分からなる。

<1>4. 各引数の添字 `i` と各 `u ∈ units(ty(args[i]))` について、`B'_V` の `App` が `u` の下の inhabited な
      leaf を消費するのは `callee_owns(i, u)` (P11) が真のときちょうどである。
  BY A12, D9, L15
  L15 より `callee_owns(i, u)` は、この位置で作られる活性化の本体を持つ関数 `W` が `(W のパラメータ i, u)`
  を D14 の意味で所有することと同値である。A12 より `u` は呼び出し先のパラメータの型でも同じ unit で
  ある。D9 の `App` の行は「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」を消費とする。

<1>5. `u` の下の inhabited な leaf に `C_T` の元があるならば、`arg_owned(i, u)` は偽である。
  BY L13, L23, P11
  L23 (b) より `(args[i], u)` は `B_V` の site (DEF site) である。P11 の
  `arg_owned(i, u)` は `ctx.owns_unit(args[i], u)` であり、これが真ならば L13 より `u` の下の inhabited な
  各 leaf の由来は所有される。`T` は所有されないのでそのどれも `T` ではなく、`C_T` の元は無い。対偶が
  言明である。

<1>6. `T` を減らす対 `(i, u)` はすべて P11 の `before` に入り、`before` に入る各対について、(A-前) の
      `Retain(args[i], u)` が増やす由来の多重集合と `App` がその対について減らす由来の多重集合は等しい。
  <2>1. `T` を減らす対は `callee_owns(i, u)` が真かつ `arg_owned(i, u)` が偽である。
    BY <1>4, <1>5
    `<1>4` より減分が起きるのは `callee_owns(i, u)` が真のときであり、その減分が `T` に当たるのは
    `u` の下の inhabited な leaf に `C_T` の元があるときなので、`<1>5` より `arg_owned(i, u)` は偽で
    ある。
  <2>2. その対は `before` に入る。
    BY P11, <2>1
    P11 より `before` は `callee_owns(i, u)` が真かつ `arg_owned(i, u)` が偽である対の列である。
  <2>3. `before` に入る対 `(i, u)` について、2 つの多重集合は等しい。
    BY D10, P11, <1>4
    D10 の `Retain` の行は `u` の下の inhabited な各 leaf `λ` につき `T_ρ(args[i], λ)` を 1 増やす。
    `before` に入る対は `callee_owns(i, u)` が真なので、`<1>4` より `App` は同じ `u` の下の inhabited な
    各 leaf を消費し、D10 の消費の行が同じ由来を 1 ずつ減らす。
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>7. QED
  BY <1>1, <1>2, <1>3, <1>6
  `<1>2` と `<1>3` より、この塊で `T` を動かすのは (A-前) の `Retain` と `App` の引数の位置の消費だけで
  ある。`<1>6` より、`T` を減らす対はすべて `before` に入り、`before` の各対の増分と減分は等しいので、
  総和どうしも等しい (`before` の対で `T` を増やさないものは `<1>6` の等式より `T` を減らしもしない)。
  `<1>1` より (A-前) の `Retain` はすべて `App` の節点より前に実行される。

**同じ変数を 2 つの引数位置へ渡す呼び出し**。`App(f, [a, a])` では対 `(0, u)` と `(1, u)` が別々に
数えられ、`before` に `Retain(a, u)` が 2 つ積まれ、`App` が同じ leaf を 2 度消費する。`<1>6` の対応は
対ごとなので、この場合も増分と減分は釣り合う。

### 11.6 主不変条件

**INV-a**。`T` を `ctx.owns_object(T)` が偽であり `obj(C_T)` が計数下 (D26) である `ρ`-由来とする。
DEF 対応する位置 が挙げる `ρ` の上の各位置と `B'_V` の対応する位置において `n_out(τ, T) = 0` であり、
塊の中の各時点において `n_out(τ, T) ≥ 0` である。

<1>1. 根では `n_out(τ, T) = 0` である。
  BY DEF n_in、n_out, DEF 由来ごとの義務, L20
  L20 より `T = (p, σ)` は `V` のパラメータの leaf であり `ctx.owns_object(p, σ)` は偽なので、`ι_V` は
  この leaf に 0 を与える。

<1>2. CASE `τ` の節点が `Retain` または `Release` である。
  BY DEF 塊, L16, L24
  L16 より塊はその節点 1 つ ((K)) か空である。(A-前) と (A-後) は `App` の塊の一部なのでこの塊には
  現れない。L24 より (K) の `Retain`/`Release` は `n_out(・, T)` を動かさないので、塊の入口と出口の値は
  等しく、塊の中に `T` を動かす事象は無い。

<1>3. CASE `τ` の節点が `Let(x, App(callee, args), k)` である。
  BY L26, 帰納法の仮定
  帰納法の仮定より塊の入口で `n_out(τ, T) = 0` である。L26 よりこの塊の増分の総和と減分の総和は等しく、
  増分はすべて減分より前に起きるので、塊の中の各時点の値は 0 以上で増分の総和以下であり、塊の出口では
  再び 0 である。

<1>4. CASE `τ` の節点がそれ以外である。すなわち `Let(x, rhs, k)` で `rhs` が `Var`・`Closure`・`Llvm`・
      `Match` のいずれか、`Destructure`、`Eval`、`Ret` である。
  <2>1. L16 より塊はその節点 1 つである。
    BY L16
  <2>2. この節点が `n_out(・, T)` を減らす事象は無い。
    BY L22, L25
    L22 より減らすのは `Release` 節点と D9 の消費であり、この場合の節点は `Release` ではない。L22 の
    最後の行より、この場合の節点が行いうる D9 の消費は `Closure` の行、`Llvm` の行、`Destructure` の
    2 行、終端の `Ret` の行の 5 行であり、いずれも DEF 所有を読まない消費 に入るので、L25 より
    `C_T` のスロットを消費しない。
  <2>3. この節点が `n_out(・, T)` を増やす事象は無い。
    BY 4.3 の系, D10, L20, L22, CODE src/rc_ir/ownership.rs: collect_bindings
    L22 より増やすのは `Retain` 節点と D10 の生成であり、この場合の節点は `Retain` ではない。D10 の
    生成の表の 5 行が値を与えるのは、`Let` の束縛変数、`Destructure` のフィールド変数、`Match` の
    変位アームの payload 変数のいずれかである。L20 より `T = (p, σ)` の `p` は `V` のパラメータの名前で
    あり、4.3 の系 より出力の束縛名は互いに相異なるので、この生成が 1 にする由来はどれも `T` ではない。
  <2>4. QED
    BY <2>1, <2>2, <2>3
    塊の入口と出口の値は等しく、塊の中に `T` を動かす事象は無い。

<1>5. QED
  BY D2, <1>1, <1>2, <1>3, <1>4, CODE src/rc_ir/ast.rs: RcExpr, RcRhs
  D2 より `RcExpr` は 6 種、`Let` の右辺 `RcRhs` は 5 種であり、`<1>2` (`Retain`・`Release`)、`<1>3`
  (`Let` の `App` の腕)、`<1>4` (残る `Let` の 4 つの腕、`Destructure`、`Eval`、`Ret`) がこれを尽くす。
  `<1>1` を基底、`<1>2`-`<1>4` を段とする、`ρ` の上の位置 (DEF 対応する位置) についての帰納である。
  終端の `Ret` の出口は「終端の `Ret` の消費の後」であり、その節点は `<1>4` の場合なので、最後の位置も
  この帰納が覆う。

### 11.7 P14a の QED

<1>1. どの時点も、DEF 対応する位置 が挙げる位置か、ある塊の中の点かのどちらかである。
  BY DEF 時点, DEF 塊, DEF 対応する位置, L16
  DEF 時点 の事象は `B'_V` の節点が行うものであり、DEF 塊 と L16 より `B'_V` の各節点はちょうど 1 つの
  塊に属する。塊の最後の事象の直後の点はその塊の出口であり、DEF 対応する位置 の位置である。活性化の
  開始は根の位置である。

<1>2. 各時点で `held(τ, C_T) ≥ 1` であり、その値は D34 の `held_ρ(τ, C_T)` である。
  BY D26, DEF 類の参照, INV-a, L20, L21, <1>1
  `<1>1` と INV-a より各時点で `n_out(τ, T) ≥ 0` である。L20 より `β(T) = 1` なので、DEF 類の参照 より
  `held(τ, C_T) = n_out(τ, T) + 1 ≥ 1` である。`obj(C_T)` は計数下 (D26) であり、L20 より `T` は `V` が
  借用する unit の下のパラメータ leaf -- D34 の第 3 行 -- なので、L21 (a) が付ける開始の時点の条件は
  DEF 時点 が挙げるすべての時点で満たされる (L21 の言明の直後の注)。よって L21 (a) より この値は D34 の
  `held_ρ(τ, C_T)` である。

<1>2a. 活性化が生きている (D23) 間のどの時点でも `held(τ, C_T) ≥ 1` である。DEF 時点 が挙げない点
       -- この活性化が行った D10 の事象の直後でも、活性化の開始でも、読みの直前でもない点 -- における
       `held` は、その点までにこの活性化が行った D10 の事象のうち最後のものの直後の時点の値として読む。
       入れ子の呼び出しで中断中の時点、ほかの制御の流れの段が走っている時点、D24 の段内の点のうち
       割り当てと割り当てたオブジェクトの欄を埋める受け渡しのあいだの点、初期 `Obl` を作る受け渡しの
       列の途中の点が、その形である。README の
       A19 が「中断中はその活性化の節点が走らないので `held` も `bumps` も動かず」と書くのと同じ読みで
       ある。
  <2>1. `held(・, C_T)` を動かすのは、この活性化が実行する `B'_V` の `Retain` 節点・`Release` 節点・
        D10 の生成・D9 の消費の 4 種だけである。
    DEF 類の参照 より `held(τ, C_T) = n_out(τ, T) + β(T)` であり、`β(T)` は時点によらない。L22 より
    `n_out(・, T)` を動かす事象はこの 4 種で尽きる。
    BY DEF 類の参照, L22
  <2>2. 活性化が生きている間の各時点について、その時点までにこの活性化が行った D10 の事象のうち最後の
        ものの直後の時点は DEF 時点 が挙げる時点であり、その 2 つの時点の間にこの活性化は D10 の事象を
        1 つも行わない。事象をまだ 1 つも行っていない時点については、活性化の開始をその時点に取る。
    DEF 時点 は、活性化の開始と、この活性化が行う D10 の各事象の直後の点と、D7 の読む構文が行う各読みの
    直前の点を時点とする。読みは事象ではないので、読みの直前の点における `held` は、その点より前の
    最後の事象の直後の値に等しい (DEF 時点)。D24 より段は
    不可分なので、事象は段の中で起き、時点の列の上に順に並ぶ。この活性化が段を実行していない間 --
    子の活性化が動いている間 ((E3)、(E7)、オペランドを適用する `Llvm` の段) と、ほかの制御の流れの段が
    走っている間 -- は、この活性化は事象を 1 つも行わない。**ほかの制御の流れの段がこの活性化の 1 つの段の
    途中へ割り込むことは無い** -- D24 は「段は不可分であり、複数の制御の流れ (`FFI_EXPORT` を通じて外から
    入るスレッド) がある実行では、それらの段が 1 つの列に並ぶ」と述べ、A17 (iii) は「環境の動作も D24 の段としてこの実行の
    1 つの列に並ぶ。段は不可分なので、環境が動くのは段と段のあいだである」と述べる。よってほかの流れの段は
    この活性化の段と段のあいだに並び、その区間にこの活性化の事象は無い。(F) の解放が作る活性化はその段の
    中で終わるので、その内側に D24 の時点は無い。
    BY A17, D7, D24, DEF 時点
  <2>3. QED
    `<2>1` と `<2>2` より、生きている間の各時点の `held` は DEF 時点 が挙げるある時点の値に等しく、
    `<1>2` よりそれは 1 以上である。
    BY <1>2, <2>1, <2>2

<1>3. `C_T` のすべてのスロットは 1 つのオブジェクトを指す。それを `obj(C_T)` と書く。
  BY L9, DEF 別名類
  L9 より 1 歩の前後の 2 つのスロットは同じオブジェクトを指し、`T_ρ` は有限歩で定まる。`C_T` の各
  スロットから `T` への 1 歩の列にこれを当てると、どのスロットも `obj(T)` を指す。

<1>4. `held(τ, C_T)` は、`C_T` に数えられた参照のうちその時点で未処分のものの個数である。
  <2>1. D10 の生成は `held(・, C_T)` を動かさない。
    BY 4.3 の系, D10, L20, CODE src/rc_ir/ownership.rs: collect_bindings
    D10 の生成の表の 5 行が値を与えるのは、`Let` の束縛変数、`Destructure` のフィールド変数、`Match` の
    変位アームの payload 変数のいずれかである。L20 より `T = (p, σ)` の `p` は `V` のパラメータの名前で
    あり、4.3 の系 より出力の束縛名は互いに相異なるので、生成が 1 にする由来はどれも `T` ではない。
  <2>2. D8 より参照は D10 の生成によって作られ、D10 の消費または `Release` によって処分される。
        DEF 類の参照 と L22 と `<2>1` より、`held(・, C_T)` を動かす事象は (α)・(β)・`Retain` の 3 種で
        尽き、そのそれぞれについて 1 を足し引きする。
    BY D8, D10, DEF 類の参照, L22, <2>1
  <2>3. `held` が数える単位のうち、この活性化の事象が作ったのでないものは、開始値の 1 つだけであり、
        それは実在の参照である。
    BY A5, D16, D26, DEF 類の参照, DEF 由来ごとの義務, L20, <2>2
    DEF 類の参照 より `held(τ, C_T) = n_out(τ, T) + 1` であり、DEF 由来ごとの義務 より `n_out` は
    初期値 (L20 より `ι_V` が 0 を与える) から `<2>2` の事象だけで動く。よって `held` の開始値は 1 で
    ある。A5 は「値が保持する参照は、その型の `boxed_leaf_paths` が列挙する leaf のうち、inhabited
    (D16) であって計数下のオブジェクト (D26) を指すものにちょうど 1 つずつある」と述べる。L20 より
    `T = (p, σ)` は `V` のパラメータの inhabited な leaf であり、`obj(C_T)` は計数下なので、その leaf は
    参照を 1 つ持つ。
  <2>4. その 1 つは呼び出し元が持つ参照であり、この活性化が返るまで処分されない。
    BY A5, A20, D10, D14, D24, L18a, L20, <2>3
    L20 より `V` は借用版であり、`T` は `V` が借用する unit の下の leaf である。L18a より、借用版の
    本体の活性化を作る段は (E3) の呼び出しの段だけなので、この活性化には呼び出し元が在る。D10 の
    初期値は借用する unit の下の leaf を `Obl` に入れないので、この 1 つは `V` の義務ではない。D14 より
    借用する unit の参照は呼び出し元が処分するものであり、A20 より呼び出し元はそれを呼び出しが返るまで
    処分しない。
  <2>5. QED
    BY <2>2, <2>3, <2>4
    `<2>2` と `<2>3` より、`held(τ, C_T)` は「開始値の 1 つ」と「この活性化が `Retain` で作った参照の
    個数」の和から「この活性化が (α) と (β) で処分した個数」を引いたものである。`<2>4` より開始値の
    1 つはこの活性化の間 処分されないので、この差は `C_T` に数えられた参照のうち未処分のものの個数で
    ある。

<1>5. QED
  BY D8, D23, D26, <1>2, <1>2a, <1>3, <1>4
  `obj(C_T)` が計数下 (D26) である類を取る。`<1>2a` と `<1>4` より、`C_T` は活性化が生きている (D23)
  間のどの時点でも未処分の参照を少なくとも 1 つ持ち、D8 と D26 よりそれは D8 の意味の参照である。
  `<1>4` の勘定は `held` を動かす事象の増減だけを読むので、`<1>2a` が値を延長した中断中の時点でも
  そのまま当たる。「活性化の間ずっと」とは D23 の意味で生きている間ずっとであり、これがその範囲で
  ある。`V`・`ρ`・活性化・`T` は任意であり、L20 の注より、ρ-終端が借用する (D14) パラメータ・capture の
  leaf である計数下の別名類はちょうどこの `C_T` の形である。これが README の P14a である。

**この命題が言わないこと**。`H(obj(C_T)) ≥ 1` は主張しない。`held` の数え上げと実行時の参照カウントを
結ぶのは D21 が活性化に課す制限 (A19 (i) の不等式) であり、この証明はそれを読まない。読む者
(`p40-cancel-soundness.md` の `L42`) が要るのも `held(τ, C_T) ≥ 1` の側である。

**`obj(C_T)` がグローバル状態である場合**。D26 より、グローバル状態のオブジェクトを指す leaf は D8 の
意味の参照を持たず、計数下であるかどうかは活性化の間 変わらないので、この場合は活性化の全体にかかる。
**この節はその類について何も言わない。** DEF 由来ごとの義務 は `obj(T)` が計数下でない `T` について
`n^ι_C` を定めず、DEF 類の参照 はその `n_out` の上に `held` を建てるので、`held(τ, C_T)` も定まらない。
README の P14a が「計数下の別名類」と書き、D34 が計数下の類にだけ `held_ρ` を定め、
`p40-cancel-soundness.md` の `L42` が計数下の類に限って P14a を読むのと、同じ限定である。

**A19 も D21 の制限も読まない**。第 11.6 節の INV-a は `n_out` を上下から挟むのではなく、借用する
由来については**等式** (`= 0`) で追う。増やす事象は (A-前) の `Retain` だけで、それは同じ塊の `App` の
消費とちょうど釣り合う (L26) からである。よって P14a は A19 の (ii) のどの節にも、D21 が活性化に課す
制限 (A19 (i) の不等式) にも立たない。第 11.7 節の `<1>2a` が A19 の一文を引くのは、中断中の `held` の
読み方が同じであることを言うためであり、その `BY` は A19 を挙げない。

## 12. P14b -- 借用する本体の活性化は呼び出しが作る

**言明** (README の P14b)。`borrow_ify` の出力と、`cancel` がそれを写したプログラムの両方について、
その実行 (D24) において、借用する (D14) unit を持つ本体の活性化を作る段は、(E3) の呼び出しの段に限る。

**この命題が要る形**。A20 は呼び出し元の振る舞いについての仮定である -- 「その関数が借用する (D14)
unit の参照を、呼び出し元は呼び出しが返るまで処分しない」。借用する unit を持つ本体の活性化が呼び出しで作られていなければ、A20 が
語る呼び出し元がそこに居ないので、A20 はその活性化について何も言わない。**P14 は命題の並びで P14b より
前に在るので、P14 の証明は P14b を引けない。** 同じ 1 文を要る第 10.7 節の `<1>4` の `<2>1` が L18a を
直に引くのはそのためである。

**範囲が 2 つであること**。`<1>1` から `<1>6` が `borrow_ify` の出力について示し、`<1>7` が
`cancel` の出力へ渡す。`cancel` は関数の `name` を変えないので (`<1>7` の `<2>1`)、`borrow_ify` の
出力の借用版に対応する `cancel` の出力の関数を、以下でも借用版と呼ぶ。

<1>1. 出力の版のうち、D14 の意味で借用する unit を持つのは借用版 `f_borrow` の `body` だけである。
  DEF 出力の版 より出力の版は各 `f_own`、各 `f_borrow`、各グローバル初期化子で尽きる。第 8 節の系 1 より
  `f_own` の `borrowed_units` は空なので、D14 より `f_own` は自分のパラメータ・capture の全 unit を
  所有する。グローバル初期化子の `init` は D14 の意味の unit を 1 つも持たない -- D1 より
  `RcGlobalInit` は `symbol`・`ty`・`init`・`owns_initializer`・`owns_storage` の 5 つのフィールドを
  持って `borrowed_units` の欄を持たず、その `init` はパラメータも capture も持たないので、D14 が
  所有と借用を割り当てる先が無い。
  BY D1, D14, DEF 出力の版, 第 8 節の系 1, CODE src/rc_ir/ast.rs: RcGlobalInit

<1>2. 実行が活性化を作る段は 5 種で尽きる -- (E1) 環境が活性化を作る段、(E3) 呼び出しの段、(E7) グローバル
      の初期化の段、(E2) のうちオペランドを適用する `Llvm` の段、(F) の解放が `Destructor` について作る段
      である。
  D24 の「活性化の林」は「(E1) が作る活性化を**根**、(E3) と (E7)、(E2) のうちオペランドを適用する
  `Llvm` の段、および (F) の解放が `Destructor` について作る段が作る活性化を、それを作った活性化の
  **子**と呼ぶ」「**活性化を作る段はこの 5 種で尽きる。**」と述べる。
  BY D24

<1>3. (E7) が作るのはグローバル初期化子の `init` の活性化だけであり、関数の `body` の活性化ではない。
  D24 の (E7) は「まだ初期化されていないグローバル `g` を読む者が居るとき、`g` のアクセサが `g` の
  初期化子の `init` の活性化 `b` を作る」と述べ、D24 の (E1) は「グローバル初期化子の活性化はこの段では
  作られない -- それを作るのは (E7) である」と 2 つの段の分担を述べる。`<1>1` より借用する unit を持つ
  本体は借用版の `body` であり、D23 より `body` と `init` は別の本体である。
  BY D1, D23, D24, <1>1

<1>4. 残る 3 種 -- (E1)、オペランドを適用する `Llvm` の段、(F) の解放が `Destructor` について作る段 --
      は、いずれも Fix の関数型の**値**から呼び出し先を取る。
  - (E1)。D24 の (E1) は「C のエントリ点または `FFI_EXPORT` のエントリ点が、関数の本体 `B` の活性化 `a` を
    作り」と、この段の主体を 2 つに限る。D22 はその 2 つを、C のエントリ点は「グローバル `main` を読み、
    その `IO` の runner に `apply_lambda` で `IOState` を渡す」もの、`FFI_EXPORT` のエントリ点は「C の
    呼び出し元から引数を受け取り、書き出された Fix の値に `apply_lambda` で渡す」ものと述べる。どちらも
    環境が持つ Fix の関数型の値から呼び出し先を取る。
  - オペランドを適用する `Llvm` の段。D24 の (E2) は「その op の生成コードがオペランドを関数として
    適用するとき (`LLVMGen::applies_a_function_operand` が真を宣言する op)、適用された関数の本体の
    活性化が作られる」と述べる。適用されるのはその op のオペランドの値か、その段が組み立てた値
    (`InlineLLVMFixBody` の 1 回目の適用に渡る `fix(f)`) であり、どちらも Fix の関数型の値である。
  - (F) の解放が `Destructor` について作る段。D24 の (F) は「`o` が `_dtor` の欄に持つ関数が `_value` の
    欄の値に適用され、返った `IO` の動作が走る」「適用される関数も、それが返す `IO` の動作も、
    オブジェクトの欄から来るものであって、どの構文もそれを名指さない」と述べる。この段が作る活性化は
    2 つであり (D24 の「活性化の林」)、1 つ目の呼び出し先はオブジェクトの `_dtor` の欄が保持する
    Fix の関数型の値、2 つ目の呼び出し先はその 1 つ目が返した `IO` の動作の runner、すなわち
    プログラムが作った Fix の関数型の値である。

  D23 は `App` について「`callee` の値がクロージャならその funptr の指す関数、funptr ならそれ自身で
  ある」と、関数の値から呼び出し先を取る規則を定め、その規則を `Generator::apply_lambda` に置く
  (`get_lambda_func_ptr` が返す関数ポインタを `build_indirect_call` で呼ぶ)。この 3 種はどれも
  `apply_lambda` に関数の値を渡すので、呼び出し先は同じ規則で決まる。
  BY D22, D23, D24, CODE src/generator.rs: Generator::apply_lambda

<1>5. 借用版の関数値は出力プログラムのどこにも作られず、環境 (D22) が持つ値にも、局所変数の値にも、
      オブジェクトの欄が保持する値にも入らない。
  L18a の言明の第 2 文がこれである。
  BY L18a

<1>6. `borrow_ify` の出力について言明が成り立つ。
  `<1>4` の 3 種が作る活性化の本体は、その段が取る関数の値が指す関数の `body` である (`<1>4`、D23)。
  `<1>5` よりその値は借用版の関数値ではないので、この 3 種は借用版の `body` の活性化を作らない。
  `<1>3` より (E7) も作らない。`<1>2` より段はこの 5 種で尽きるので、借用版の `body` の活性化を作る段は
  (E3) だけである。`<1>1` より借用する unit を持つ本体は借用版の `body` に限るので、これが言明である。
  BY D23, <1>1, <1>2, <1>3, <1>4, <1>5

<1>7. `cancel` が `borrow_ify` の出力を写したプログラムについても言明が成り立つ。
  <2>1. `cancel` は入力の各関数を 1 つの関数に写し、`name`・`capture`・`borrowed_units` を含む、`body`
        以外の欄を 1 つも変えない。出力の `body` は入力の `body` から `Retain`/`Release` 節点をいくつか
        取り除いたものであり、残る節点の種類・変数・path・並びは入力のものに等しい。グローバル初期化子の
        列は入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力のものに等しく、`init` も同じ形の
        取り除きである。`roots` も入力のものに等しい。
    P24 は「`roots` を変えない」「出力の各関数は入力のちょうど 1 つの関数から作られ」「出力のグローバル
    初期化子の列は入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力の第 `i` 要素のものに等しい」
    「**`cancel` は `RcFunc` の `body` 以外の欄を 1 つも変えない。** とくに `borrowed_units` と
    `capture` は入力のものに等しい」と述べる。`cancel` は `prog.funcs` の各値について `f.clone()` の `body` だけを
    `drop_nodes(body, analysis.cancelled())` に差し替えて `f.name` を鍵に積み、`prog.globals` の各要素に
    ついて `symbol` と `ty` を写して `init` を同じ形に差し替え、`roots` を clone する。A22 より
    `prog.funcs` の各鍵はその `RcFunc` の `name` に等しいので、鍵の集合はこの積み直しで変わらない。
    P22 は「`drop_nodes(B, S)` は、`B` の `NodeId` が `S` に入る `Retain`/`Release` 節点だけを取り除いた
    木を返し、他の節点の種類・変数・path・並びを変えない」と述べる。
    BY A22, P22, P24, CODE src/rc_ir/borrow.rs: cancel

  <2>2. `cancel` の出力の本体のうち、D14 の意味で借用する unit を持つのは借用版の `body` だけである。
    D14 の所有と借用の割り当ては、その本体を持つ関数のパラメータ・capture と `borrowed_units` だけから
    定まり、`<2>1` より `cancel` はそのどれも変えない。よって `<1>1` の結論がそのまま `cancel` の出力に
    ついて立つ。グローバル初期化子は D1 よりパラメータも capture も持たないので、この意味の unit を
    1 つも持たない。
    BY D1, D14, <1>1, <2>1

  <2>3. `cancel` の出力についても、借用版の関数値は環境 (D22) が持つ値にも、局所変数の値にも、
        オブジェクトの欄が保持する値にも入らない。
    <3>1. `cancel` の出力の本体で借用版の名前が現れるのは、`RcRhs::App` の callee の位置だけである。
      L18a の言明の第 1 文より、`borrow_ify` の出力の本体でそれが現れるのは `RcRhs::App` の callee の
      位置だけである。`<2>1` より `cancel` の出力の各本体は入力の本体から節点を取り除いたもので、残る
      節点の種類・変数・path・並びは等しいので、名前が現れる位置は入力のそれの部分集合である。
      BY L18a, <2>1
    <3>2. `cancel` の出力の借用版の `body` に、`InlineLLVMFixBody` の `Llvm` 節点は無い。
      `InlineLLVMFixBody` は `gc.current_function()` の関数ポインタを持つクロージャを作るので、借用版の
      `body` にその op の節点が在れば、その本体を生成するコードが借用版の関数値を作る。L18a の言明の
      第 2 文よりそれは `borrow_ify` の出力には無く、`<2>1` より `cancel` の出力の借用版の `body` の
      節点はそのときの節点の部分集合なので、そちらにも無い。
      BY L18a, <2>1, CODE src/fixstd/builtin.rs: InlineLLVMFixBody
    <3>3. QED
      A21 は「Fix の関数型の値に LLVM 関数の番地を書き込むのは、クロージャを作る段 (`build_rc_closure`)、
      funptr のグローバルを読む段 (`ValueAccessor::get` の `is_funptr` の枝)、そして
      `InlineLLVMFixBody` の 3 か所だけである。ほかのどの構文も op も、既にある関数の値を写すだけで
      ある」と述べる。1 つ目が読むのは `RcRhs::Closure` の `FuncRef` であり、`RcRhs::App` の callee の
      位置ではないので、`<3>1` よりそれは借用版の名前ではない。2 つ目が借用版の値を作りうるのは借用版の
      名前を読む位置だけであり、`<3>1` よりそれは `RcRhs::App` の callee の位置に限る。その位置で読まれた
      値は `apply_lambda` に渡されるだけで、束縛されずオブジェクトにも格納されない。3 つ目が作るのは
      `gc.current_function()` の値であり、`<3>2` よりその op を `body` に持つ関数は借用版ではない。
      よって借用版の関数値は、環境が持つ値にも、局所変数の値にも、オブジェクトの欄が保持する値にも
      入らない -- オブジェクトの欄が保持するのはプログラムが作った値だからであり (A4 -- 段がオブジェクトの
      記憶域へ書き込む内容はその段の節点とオペランドの値と D21 の 4 種の結果だけで決まる)、環境が読む
      `roots` は `<2>1` より `borrow_ify` の出力のものに等しいからである。
      BY A4, A21, D21, D22, <2>1, <3>1, <3>2,
         CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
         CODE src/generator.rs: Generator::get_scoped_obj, ValueAccessor::get, Generator::apply_lambda

  <2>4. QED
    `<1>2`・`<1>3`・`<1>4` の言明はいずれも D1・D22・D23・D24 と `Generator::apply_lambda` のコードだけに
    立ち、どのプログラムの実行についても成り立つ。`<1>4` の 3 種が作る活性化の本体は、その段が取る関数の
    値が指す関数の `body` である (`<1>4`、D23)。`<2>3` よりその値は借用版の関数値ではないので、この
    3 種は借用版の `body` の活性化を作らない。`<1>3` より (E7) も作らない。`<1>2` より段はこの 5 種で
    尽きるので、借用版の `body` の活性化を作る段は (E3) だけである。`<2>2` より借用する unit を持つ本体は
    借用版の `body` に限るので、これが言明である。
    BY D23, <1>2, <1>3, <1>4, <2>2, <2>3

<1>8. QED
  `<1>6` が `borrow_ify` の出力について、`<1>7` が `cancel` がそれを写したプログラムについて言明を
  与える。言明の量化はこの 2 つで尽きる。
  BY <1>6, <1>7

**この命題が言わないこと**。呼び出し元が借用した参照を呼び出しの間 処分しないことは言わない。それは
A20 である。この命題が言うのは、A20 が語る呼び出し元が在るということだけである。

**支えの在りか**。`<1>5` が引く L18a は A21 (Fix の関数型の値に LLVM 関数の番地を書き込む 3 か所)、
A13 (入力に現れるどの名前も、`name` を `#` で区切った最後の断片が `borrow` でない)、
A24 (`fix` の op は capture を持つ本体にだけ在る)、P9 (複製は名前替えである)、および `rewrite_inner` の
各腕と `borrow_ify` の末尾の読みに立つ。第 4 節より、A21 は検査を持たず、A24 は果たす者を持たない。
この命題の重みはその 2 つに載る。`cancel` の側 (`<1>7`) が足すのは P22 と P24 だけである。

## 13. A19 (ii-c) -- `borrow_ify` が写した本体の段内の点における非負性

**言明**。`borrow_ify` の出力の各本体の 1 回の活性化 (D21) について、その活性化の段 (D24) の各段内の点
`q` で、`held_ρ(q, C)` が定まる各計数下 (D26) の別名類 (D33) `C` について `held_ρ(q, C) ≥ 0` である。

**この節が果たすもの**。A19 の (ii-c) は「節点の実行の途中の各点 (D24 の段内の点) でも、各計数下の
別名類 `C` について `held ≥ 0` である。」であり、その果たす者について A19 は「果たす者は `insert_rc` で
あり、`p60-insert-rc.md` の `L19` (d) が `insert_rc` の出力について示す。**`borrow_ify` が写した本体に
ついては、まだ示されていない** -- `p20-borrow-ify.md` の義務である。」と書く。この節がその義務を果たす。

**固定するもの**。第 9.1 節と同じく、出力の版 `V`、`B_V`、`B'_V`、`ctx`、`B'_V` の活性化、それに対応する
`B_V` の実行路 `ρ` と活性化を固定する。`held` と `β` は 第 11.1 節の DEF 類の参照、`C_T` は同節の
DEF 別名類 による。この節は第 9.1 節の A19 (ii-a) の (a) と (a') の下で立つ -- 第 10.4 節がその下で
立つからである。

**点集合の差をどこで埋めるか**。DEF 時点 の点集合は D24 の段内の点の集合ではない。DEF 時点 が挙げるのは
活性化の開始と各事象の直後と各読みの直前であり、D24 の段内の点は受け渡し・生成・割り当て・処分・解放・
グローバル化の 6 種の素動作の切れ目である。割り当て・解放・グローバル化の切れ目と、(F) の解放がその段の
中で始めた活性化の動作の切れ目は、DEF 時点 のどの点でもない。**この節はその差を、`held_ρ` が値を変える点
の側から埋める** -- D34 の表は `held_ρ` を 6 行でしか動かさず、その 6 行が値を置く点は D34 が名指すので、
残る段内の点では `held_ρ` は直前の変化点の値のままである。

**この節が読む活性化**。段内の点は D24 が段の素動作の列の切れ目として定める。この節が読むのは D34 が
`held_ρ` を動かす点の一覧と、第 10.4 節が DEF 時点 の各時点について与える非負性だけなので、その活性化が
実行 (D24) に実現するかどうかを問わない。

<1>0. `B'_V` の活性化について D33 が定める別名類は、DEF 別名類 の `C_T` である。
  D33 は 1 つの実行路の上のスロット (D6) を `ρ` 終端が等しいという関係で分けた同値類を別名類と呼び、
  その歩みの各段は D20 の別名の辺である。D20 は D9 の移動の表の 6 行を別名の辺と呼ぶので、
  `Retain`/`Release` 節点はどの辺も定めない。L16 より `B'_V` は `B_V` から `Retain`/`Release` 節点を
  落とし・足し、`App` の callee の名前を差し替えたものであり、L5 より `ctx.rewrite` は束縛を導入も除去も
  しない。`App` の callee は D9 の移動のどの行の端でもない。`Llvm` の素通し leaf の行が読む宣言に
  ついては、L16 の言明より両側の op は同じオペランドの列と同じ型を持ち `result_prov` に同じ値を返す。
  よって両側の対応する実行路の上のスロットは
  同じ対であり、その上の D20 の辺も同じである。L16a より対応する位置で対応する変数が得る値は等しいので、
  D16 の inhabited も両側で同じである (第 9.1 節)。よって L9a より、`B'_V` の側で読む D33 の別名類は
  `C_T` である。
  BY D6, D9, D16, D20, D33, DEF 別名類, L5, L9a, L16, L16a, 第 9.1 節

<1>1. `held_ρ(・, C_T)` が値を変える段内の点は、D34 の表の 6 行が名指す点だけである。
  D34 は `held_ρ` を 6 行 -- 3 つの開始行と `Retain`・`Release`・消費の 3 行 -- で定め、値を置く点を
  「表の第 4・第 5・第 6 行の事象は、**それを運ぶ素動作の直後の段内の点**で `held` を動かす。」
  「第 1 行と第 2 行の開始値は、**その類の終端の参照が `Obl(a)` に入る素動作の直後の段内の点**に置く。」
  「第 3 行の開始値は、**その活性化が生きている活性化 (D23) になる点**に置く。」と定める。表に行を持たない
  動作は `held_ρ` を動かさない。よって 2 つの段内の点のあいだにこの 6 行が名指す点が無ければ、
  `held_ρ(・, C_T)` はその 2 点で等しい。
  BY D34

<1>2. 開始の 3 行が値を置く点で `held_ρ(・, C_T) = 1` である。
  D34 の第 1 行は「`C` の終端が D10 の生成で作られる」とき、第 2 行は「`C` の終端が、**所有する** (D14)
  パラメータ・capture の leaf である」とき、第 3 行は「`C` の終端が、**借用する** (D14) パラメータ・
  capture の leaf である」とき、いずれも「1 から始まる」と定める。README の D34 の末尾は
  「3 つの開始行は、計数下の類の終端を尽くす。」と述べる。
  BY D34

<1>3. 第 4・第 5・第 6 行の事象が `held_ρ(・, C_T)` を動かす段内の点 `q` について
      `held_ρ(q, C_T) ≥ 0` である。
  <2>1. その事象の直後の DEF 時点 の時点 `τ` は、`C_T` の開始の時点 (D34) 以後である。
    その事象は `C_T` のスロット `(v, λ)` (第 4・第 5 行) か `(w, μ)` (第 6 行) を名指すので、D6 より
    その変数は `τ` までに値を得ている。L9 より 1 歩の先も同じ時点のスロットなので、`T = (u, σ)` の `u` も
    `τ` までに値を得ている。D34 は開始値を「`T_ρ(C) = (u, σ)` の変数 `u` が値を得る時点で置かれる」と
    定めるので、`τ` はその点以後である。
    BY D6, D34, L9
  <2>2. `held_ρ(q, C_T) = n_out(τ, T) + β(T)` である。
    L21 (b) より `q` と `τ` のあいだに D34 の表のどの行の事象も起きないので、
    `held_ρ(q, C_T) = held_ρ(τ, C_T)` である。`<2>1` より `τ` は `C_T` の開始の時点以後なので、
    L21 (a) より `held_ρ(τ, C_T) = held(τ, C_T)` であり、DEF 類の参照 より
    `held(τ, C_T) = n_out(τ, T) + β(T)` である。
    BY DEF 類の参照, L21, <2>1
  <2>3. QED
    第 10.4 節より、`B'_V` の実行路の各時点 (DEF 時点。塊の中を含む) で各計数下の由来について
    `n_out ≥ 0` である。DEF 類の参照 より `β(T)` は 0 か 1 である。`<2>2` と合わせて
    `held_ρ(q, C_T) ≥ 0` である。
    BY DEF 類の参照, <2>2, 10.4

<1>4. QED
  `held_ρ(q, C_T)` が定まるのは `C_T` の開始の点以後である (D34) から、`q` 以前に `<1>2` の点が在る。
  `<1>1` より `held_ρ(q, C_T)` は `q` 以前の最後の変化点における値に等しく、その点は開始の 3 行が値を
  置く点か、第 4・第 5・第 6 行の事象が動かす点である。`<1>2` と `<1>3` よりどちらでも 0 以上なので、
  `held_ρ(q, C_T) ≥ 0` である。`<1>0` より `C_T` は `B'_V` の活性化について D33 が定める別名類を尽くす。
  DEF 出力の版 と L6 より出力のプログラムの本体は各 `f_own` の `body`、各 `f_borrow` の `body`、各
  グローバル初期化子の `init` で尽き、`V` がそれらを渡るとき `B'_V` もそれらを渡る (第 9.1 節)。`V`・
  実行路・活性化・`C` は任意なので、これが A19 (ii-c) -- 「節点の実行の途中の各点 (D24 の段内の点) でも、
  各計数下の別名類 `C` について `held ≥ 0` である。」-- の `borrow_ify` の側である。
  BY A19, D34, DEF 出力の版, L6, <1>0, <1>1, <1>2, <1>3, 第 9.1 節

**この節が言わないこと**。段内の点における `held` の下限が 1 であることは言わない。A19 の (ii-b) は
段内の点では読めず (A19 の同名の節)、この節が示すのは (ii-c) が求める非負性だけである。借用する終端の
類について `held ≥ 1` を活性化の全体で言うのは P14a (第 11 節) であり、そちらは DEF 時点 の時点の上で
立つ。
