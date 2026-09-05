# P7e、P7d、P7a -- 所有の unit 粒度と一様性

この文書は `README.md` の P7e と P7d を証明し、P7a の 2 つの向き -- **節 1 から節 3** と
**節 2 から節 1** -- を証明する。`README.md` の命題については、**言明**だけを読む。

P7a の残る向き (節 3 から節 2、節 3 から節 1) は偽である。`Λ(u)` に inhabited (D16) な leaf が 1 つも
無い site では節 3 が空虚に真になるからで、第 6 節の R2 がその本体を挙げる。`Inh(v, u) ≠ ∅` を仮定に
足せば 3 つは同値になる。

**P7e の (b) が条件付きである理由。** README の P7e の (b) は、`truncate_to_unit(ty(r), p)` が値 `t` を
返すときにだけ `owns_object(r, t)` を主張する。この条件が要るのは `truncate_to_unit` が中断しうるから
である。`pty(r) = None` である `r` -- 例えば `I64` の値を束縛する `Binding::Producer` の変数 -- と
`p = [0]` を取ると、`owns_object(r, p)` は `owns_object` の `None` の腕が真を返すのに、
`truncate_to_unit(I64, [0])` は `unit_step(I64) = NoUnit` の腕で `panic!` する
(`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object`, `CODE src/rc_ir/ownership.rs: truncate_to_unit`,
`unit_step`)。第 3 節は (a) と (b) をこの形のまま証明する。この文書の中で P7e を読むのは L21 の `<1>1`
の中と L22 の `<1>2` であり、どちらも (a) と (b) で足りる。

この文書が読んだコードは、コミット `3ff50dc25b76cd7f71928313c647e15a20bd22ec` の版である。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P7e (`owns_object` は unit ごとに答える) | 証明した (第 3 節) |
| P7d (所有は site ごとに一様である) | 証明した (第 5 節) |
| P7a、節 1 から節 3 | 証明した (第 6 節) |
| P7a、節 2 から節 1 | 証明した (第 6 節) |
| P7a、節 3 から節 2 と節 1 | **偽** (第 6 節の R2)。`Inh(v, u) ≠ ∅` を足せば真になる |

P7e の要は L1 である。`subtree_type` と `truncate_to_unit` は同じ型の列を同じ順に歩き、同じ `unit_step` の
答えで場合を分けるので、`sub` が `None` を返す位置で `trunc` が何をするかは、その位置の `step` だけで
決まる -- `Unit` の行と、添字が `capture_idx` に等しい `Capture` の行では `break` し、`NoUnit` の行では
`panic!`、添字が `capture_idx` と異なる `Capture` の行では `assert_eq!` が失敗する (L1 の (b))。

P7d の要は L11 と L14 である。`level_ownership` は候補 `(r, p)` について `covered_leaves(ty(r), p)` を
所有へ倒し、`owns_object_yet` は `units_under(ty(r), p)` の各 unit に「同じ鍵を持つ所有された leaf」を
要求する。この 2 つが噛み合うのは `covered_leaves(ty(r), p)` が空でないときであり (L11)、空でないことは
候補の path が `origin` の再帰で作られる形に限られることから出る (L14)。

P7a の 2 つの向きは、`origin` の再帰についての 2 つの帰納である。L21 が静的な向き (unit が所有ならその下の
**すべての** leaf も所有) を、L22 が実行時の向き (unit が非所有ならその下の **inhabited な** leaf も
非所有) を与える。どちらも `(x, π)` が「unit を覆う」(L18) ことを仮定に運ぶ。実行時の向きが読む
「その位置までに値を得ている」は L20a が与える -- A11 は使用がどの束縛に解決するかしか言わず、その
束縛の変数が既に値を得ていることは D2 のスコープの規則と D3 の実行路の進み方から出る。この 2 つと
L12 が回す帰納の尺度は `|Reach(・)|` であり、それが狭義に減ることは L11a が与える -- 減るのは
`origin` の再帰に閉路が無いからであり、閉路があれば `origin` は停止しない。**`union_as` が作る隔たりは
この 2 つの帰納のどちらにも現れない**ので、宣言についての追加の仮定は要らない。第 6 節の最後の小節が
その理由を述べる。

この帰納が固定した出力版の 2 つの本体 (第 1 節の `Pre(V)` と `Post(V)`) を追えるのは、どちらも
A6・A11・A12 の性質を持つからである。README は A6 と A11 の範囲を `borrow_ify` の入力に限ると書く。
A12 に範囲の記述は無いが、この文書は出力版についてもこの 3 つを読むので、いずれも借用版へそれを渡す段が
要る。L0 が `Pre(V)` について、L0a が `Post(V)` についてその段であり、DEF 再帰で訪れる対 が前者を、
L20a・L20b・L21a・L22 が後者を読む。L21 と L22 が `Binding::Llvm` の腕で読む
L19 と L20 は `rty` について語るので、それを `Λ_{ty(x)}(π)` と `Inh_x(π)` に読み替える `rty = ty(x)` を
L18a が与える。

L22 が inhabited に限るのは 1 か所である。`Binding::Llvm` の腕で、`result_prov` が `⊥` (空集合) と宣言した
leaf を落とすのに A3 の表の第 1 行を使う (L22 の `Binding::Llvm` の場合)。inhabited でない leaf が参照を持たないことを
述べるのは A5 であり、これは P10 と P14 へ渡すところで使う (`P7a の 2 つの向き` の最後の段落と R2)。

## 1. 記法

1 つの関数 (またはグローバル初期化子) の 1 つの出力版を固定し、それを `V` と書く。その `RewriteCtx` を
`ctx`、`ctx.type_env` を `type_env`、`ctx.vars` を `vars`、`ctx.owned_units` を `OU` と書く
(`CODE src/rc_ir/borrow.rs: RewriteCtx`)。以下では型環境を引数から落として書く。

**`V` は 2 つの本体を持つ。** `V` が書き換える本体 -- 関数の版ならその関数の `body`、グローバル初期化子の
版ならその `init` -- を **`Pre(V)`**、`ctx.rewrite` がそれを写した本体を **`Post(V)`** と書く。
`borrow_ify` が出力に据えるのは `Post(V)` である。`ctx` は `Pre(V)` を本体とする `RcFunc` から
(グローバル初期化子の版では `Pre(V)` から) 作られるので、**`vars` は `Pre(V)` の表である**
(`CODE src/rc_ir/borrow.rs: borrow_ify`, `RewriteCtx::new`, `RewriteCtx::rewrite`)。
**`V` の site (第 6 節の DEF site) は `Pre(V)` から集め、活性化 (D21) は `Post(V)` のものを取る。**
この 2 つを繋ぐのは L0a である。**`V` の本体と書かず、`Pre(V)` か `Post(V)` を名指す。**

- `step(τ)` は `unit_step(τ, type_env)` (`CODE src/rc_ir/ownership.rs: unit_step`)。
- `units(τ)` は `rc_units(τ, type_env)`、`leaves(τ)` は `boxed_leaf_paths(τ, type_env)`
  (`CODE src/rc_ir/ownership.rs: rc_units`, `CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)。
- `trunc(τ, π)` は `truncate_to_unit(τ, π, type_env)`、`under(τ, π)` は `units_under(τ, π, type_env)`、
  `sub(τ, π)` は `subtree_type(τ, π, type_env)`
  (`CODE src/rc_ir/ownership.rs: truncate_to_unit`, `units_under`, `subtree_type`)。
- `pty(r)` は `vars.param_tys.get(r)` (`CODE src/rc_ir/ownership.rs: VarTable`)。
- `owns(r, p)` は `ctx.owns_object(r, p)` (`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object`)。
- `ty(x)` は `x` が得る値の型である。D6 より `RcVar` の `ty` の欄はその変数に束縛された値の型を表し、
  A12 より同じ名前の `RcVar` の `ty` の欄は互いに一致するので、これは `x` の出現によらない。
  `pty(x)` とこの型の関係は L1e が述べる。

`FieldPath` は `Vec<usize>` である (`CODE src/rc_ir/ast.rs: FieldPath`)。`π[0..k]` は `π` の先頭 `k` 個の
要素からなる path、`|π|` は `π` の長さである。

**ファイル全体についての数え上げを引くときは、`CODE <ファイル>: <数え上げた集合の名前>` と書く。**
主張がそのファイルの中のある集合の全体についてのものであることを、この形が示す。L15 の `<1>5` の `<2>2`
と L17 の `<1>1` がこの形を使う。

**EXT Rust の可視性**
Rust では、可視性の修飾子 (`pub`、`pub(crate)`、`pub(in ...)`) を持たない項目 -- struct のフィールド、
`impl` ブロックの中の関連関数 -- は private であり、それを宣言したモジュールとその下位モジュールの
中からだけ名指せる。1 つのファイルが 1 つのモジュールを成し、そのファイルが `mod` 宣言を持たなければ
そのモジュールは下位モジュールを持たない。したがって、そのような項目に触れるコードは、それを宣言する
ファイルの中だけに在る。読む者は L17 の `<1>1` である。

関数呼び出しについて、**値を返す**とその **中断する** (`panic!` / `assert!` / `unreachable!` に到達する) を
区別する。この文書の等式「`f = g`」は、README が P7e (a) の等号について置く読みで読む --
「両辺が同時に値を返してその値が等しいか、同時に中断するかのどちらかであることをいう」。

**A6・A11・A12 を `Pre(V)` について読む段は L0 に、`Post(V)` について読む段は L0a に立つ。** README は
A6 と A11 の範囲を `borrow_ify` の**入力**に限ると書く。A12 に範囲の記述は無いが、この文書は `Pre(V)` に
ついても `Post(V)` についてもこの 3 つを読む。`V` が借用版であるとき、`Pre(V)` は入力の関数の本体の
束縛変数を付け替えた複製である (P9)。L0 が 3 つの性質を各版の `Pre(V)` へ、L0a がそこから `Post(V)` へ
渡す。**この 2 つを要る段の見分け方はこうである** -- `Pre(V)` か `Post(V)` の束縛名・スコープ・型の
整合を言う段が、それである。

**DEF 現れる名前**
入力の関数 `func` に**現れる名前**とは、`func` のパラメータ・capture の名前と、`func.body` に現れる
`RcVar` の名前 (`Let` の束縛変数、`Var` の右辺の変数、`Destructure` の容器とフィールド変数、`Match` の
scrutinee とアームの payload 変数、`App` の callee と各引数、`Closure` の各 capture、`Llvm` の各
オペランド、`Retain` / `Release` / `Eval` / `Ret` が名指す変数) の全体である
(`CODE src/rc_ir/ast.rs: for_each_var`, `for_each_var_of_node`, `for_each_var_of_rhs`,
`for_each_node`, `for_each_node_inner`)。

**DEF 扱う型**
次の 3 種を**根の型**と呼ぶ。**関数のパラメータ・capture が宣言する型** (`RcFunc::params` と
`RcFunc::capture` の `RcVar` の型)、`Pre(V)` に現れる `RcVar` の型、`Llvm` 節点の結果の型 `rty` で
ある。パラメータ・capture の型を数えるのは、本体が一度も読まないパラメータの型がその本体に現れない
からである。根の型と、根の型から `unpunched_field_types(・)` が返す対の第 2 成分を有限回取って到達する型
とを合わせて**扱う型**と呼ぶ。
**以下の命題が量化する型 -- `τ`、`σ`、および `ty(・)` の形で現れる型 -- は、すべて扱う型を渡る。**
扱う型が A10 を満たすことは L1b が与え、この文書が P1 を当てるのはその上である。
**`Post(V)` に現れる `RcVar` の型も根の型である** -- L0a (c) が、`Post(V)` の各 `RcVar` は `App` の
callee を除いて `Pre(V)` のものそのものであり、callee の 2 つも型が等しいと述べる。

**DEF 歩み**
型 `τ` と path `π` に対し、型の列 `cur_0, cur_1, ...` を次で定める。`cur_0 = τ`。`i < |π|` かつ `cur_i` が
定まっているとき、`step(cur_i)` が `UnitStep::Fields { held_fields, .. }` であり、かつ `held_fields` が
添字 `π[i]` の対を含むならば、`cur_{i+1}` をその対の型とする。それ以外のとき `cur_{i+1}` は定めない。
`cur_m` が定まる最大の `m` を **`τ` の `π` に沿う歩みの長さ**と呼ぶ。`m ≤ |π|` である。

この定義は `held_field_type(held_fields, idx, _)` が「`held_fields` の中で第 1 成分が `idx` である対の
第 2 成分を返し、そのような対が無ければ中断する」ことに対応する
(`CODE src/rc_ir/ownership.rs: held_field_type`)。

第 2 節の L11 から先で、次を足して使う。

- `covered(τ, p)` は `covered_leaves(τ, p, type_env)`、すなわち `leaves(τ)` のうち `p ⊑ λ` または
  `λ ⊑ p` を満たす `λ` の集合である。ここで `σ ⊑ π` は「`σ` は `π` の接頭辞である」
  (`CODE src/rc_ir/borrow.rs: covered_leaves`)。
- `Λ_τ(π)` は `leaves(τ)` のうち `π ⊑ λ` を満たす `λ` の集合である。P7a の `Λ(u)` は `Λ_{ty(v)}(u)` である。
- `OL` は `infer_ownership` が持ち回る `owned_leaves`、`yet(r, p)` は
  `owns_object_yet(vars, type_env, r, p, OL)` である (`CODE src/rc_ir/borrow.rs: owns_object_yet`)。
  `OL` がどの時点のものかは、使う場所で述べる。
- `cand(x, π)` は `origin(x, π).candidates()` を集合とみなしたもの、`act(x, π)` は
  `origin(x, π).acted_on()` を集合とみなしたもの、`id(x, π)` は `origin(x, π).identity()` である
  (`CODE src/rc_ir/ownership.rs: Origin::candidates`, `Origin::acted_on`, `Origin::identity`)。
  `origin` を `vars` 以外の変数表について読むときは `origin(vars', x, π)` のように表を明示し、
  そこから作る 3 つも `cand(vars', x, π)` のように書く。
- `owns_unit(v, u)` は「`cand(v, u)` のすべての元 `(r, p)` について `owns(r, p)` が真」である
  (`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit`)。

**入力の側の変数表。** 固定した出力版が入力の関数 `func` から作られたものであるとき (`f_own` の版と
借用版がそうである)、入力の側の表を `vars_f = VarTable::of(func)` と書き、
`pty_f(r) = vars_f.param_tys.get(r)`、`yet_f(r, p) = owns_object_yet(vars_f, type_env, r, p, OL)` と
書く (`CODE src/rc_ir/ownership.rs: VarTable::of`, `CODE src/rc_ir/borrow.rs: owns_object_yet`)。
`vars = ctx.vars` は出力版の表なので、固定した版が借用版であれば `vars ≠ vars_f` である -- 複製が
束縛名を付け替えるからである (P9)。`infer_ownership` は入力の関数の表と site を読むので、その水準の
主張は `vars_f` の側で書き、出力版の `owns` へ渡すのは L15 と L16 である。

## 2. 型と変数表についての命題

### L0 (`Pre(V)` は A6・A11・A12 を満たす) <!--#9cef509-->

**言明**。第 1 節の `Pre(V)` -- `V` が書き換える本体 -- について、A6・A11・A12 が述べる性質が
成り立つ。すなわち、束縛変数の名前は互いに異なって
どの関数の名前とも異なり、変数の使用はその位置でスコープに入っている束縛に解決し、自由な局所名は
その版のパラメータと capture に限り、A12 が対にする各組の型は一致し、`Match` の scrutinee は union、
`Destructure` の容器は構造体であって、`Destructure` が名指すフィールドと `Match` が名指す変位は
その型が実際に持つ (punched でない) ものである。**A12 が型の `variant` を述べる各節が伴う節 --
その型の `is_closure()` は偽である -- も一緒に渡る。**

`borrow_ify` が作る出力版は 3 種である。入力の各関数の全所有版 `f_own`、借用版を持つ関数の借用版、
および各グローバル初期化子のものである (`CODE src/rc_ir/borrow.rs: borrow_ify`)。この命題を読むのは
DEF 再帰で訪れる対 であり、それを主語にする L11a・L12・L14 と、その上に立つ L18・L21・L22 と
`P7a の 2 つの向き` が第 1 節の `vars` について読む。`Post(V)` へ渡すのは L0a である。

<1>1. `f_own` の版とグローバル初期化子の版の `Pre(V)` は、入力プログラムの本体そのものである。
  `borrow_ify` は `f_own` について `func.clone()` を作ってその `body` を書き換え、グローバルについて
  `g.init` を書き換える。どちらも書き換える前の本体は入力のものである。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. 借用版の `Pre(V)`・パラメータ・capture は、入力の関数のそれの束縛変数を `rename` で一斉に付け替えた
      ものであり、それ以外の違いを持たない。`rename` の像の名前は互いに異なり、入力プログラムに
      現れるどの名前とも、出力の `funcs` のどの鍵とも異なる。
  前半は P9 である。`fresh_rename_function` は 1 つの `counter` を `&mut` で持ち回り、各束縛名に
  ついて `assign_fresh_name` を 1 度だけ呼んで `name#b<counter>` を作るので、像の名前は `#` で
  区切った最後の断片 `b<counter>` が相異なり、互いに異なる。その断片は `b` の後に 10 進数字だけが
  続く形なので、A13 より入力プログラムに現れるどの名前とも異なる。出力の `funcs` の鍵は入力の関数の
  名前か、`borrow_funcref` が作る `<元の名前>#borrow` であり、後者の最後の断片は `borrow` なので、
  これとも異なる。
  BY <ref id=cb35ab1/>, <ref id=63eadd9/>, CODE src/rc_ir/borrow.rs: clone_func, borrow_funcref,
     CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name

<1>2a. `rename_expr(node, renaming)` が返す木は、`node` の木と次の点で一致する。節点の種類とその並び、
       `Retain`/`Release` の path と state、`Destructure` が名指すフィールドの添字とその state、
       `Eval` と `Ret` の位置、`Let` の右辺の構成子、`Llvm` の op の値と `args` の長さ、`Closure` の
       `FuncRef`、`Match` の各アームの `tag` と `payload_state`。相違は、各 `RcVar` の名前が
       `renaming` の像に差し替わっていることだけであり、`RcVar` の型の欄は残る。
  `rename_expr` の本体は `grow_stack(|| rename_expr_inner(node, renaming))` であり、A15 より
  `grow_stack` は閉包をちょうど 1 回呼んでその返り値を返すので、`rename_expr` が返すのは
  `rename_expr_inner(node, renaming)` である。`rename_expr_inner` は D2 の 6 種の節点それぞれについて
  同じ構成子を組み直し、`Let` の束縛変数・`Retain`/`Release` が名指す変数・`Eval` と `Ret` が名指す
  変数に `rename_var` を掛け、`path` と `state` をそのまま写し、継続に `rename_expr` を掛ける。
  `Destructure` の腕は容器に `rename_var` を掛け、各対 `(i, v)` の添字 `i` をそのまま写して `v` に
  `rename_var` を掛ける。`Let` の右辺には `rename_rhs` を掛ける。`rename_rhs` は `RcRhs` の 5 種の
  構成子を保ち、`Var` の変数・`App` の callee と各引数・`Closure` の各 capture・`Llvm` の各オペランド
  に `rename_var` を掛け、`Closure` の `FuncRef` をそのまま写し、`Llvm` の op は `clone()` して
  `free_vars_mut()` が返す名前だけを差し替え、`Match` は scrutinee に `rename_var` を掛けて各アームの
  `tag` と `payload_state` をそのまま写し、`payload` に `rename_var` を、`body` に `rename_expr` を
  掛ける。`rename_var` は名前を `renaming` で引いて差し替えるだけで、型の欄を残す。
  BY <ref id=3e6b0e0/>, <ref id=b3dfa37/>, CODE src/rc_ir/rename.rs: rename_expr, rename_expr_inner,
     rename_rhs, rename_var

<1>3. 借用版の `Pre(V)` について A6 の性質が成り立つ。
  A6 より入力の関数の束縛名は互いに異なる。`<1>2` より `rename` の像の名前は互いに異なるので、その
  像も互いに異なる。`<1>2` より像はどちらのプログラムのどの関数の名前とも異なる。
  BY <ref id=33c54dc/>, <1>2

<1>4. 借用版の `Pre(V)` について A11 の性質が成り立つ。
  `fresh_rename_function` は束縛の位置に `assign_fresh_name` を掛けて `renaming` を組み、本体に
  `rename_expr` を掛ける。`<1>2a` より `rename_expr` は本体の各 `RcVar` の名前を同じ `renaming` で
  引く。`<1>2` より像の名前は入力に現れるどの名前とも異なり、像の中では互いに
  異なるので、鍵でない名前を恒等に写す延長は入力に現れる名前の上で単射である。`<1>2` と `<1>2a` より
  木の形は変わらないので、D2 のスコープの規則が定める入れ子も同じである。よって借用版のある位置の使用が
  スコープに見る束縛は、入力の対応する位置の使用が見る束縛の像ちょうどである。入力が A11 を満たすので、
  借用版でも使用は自分の位置でスコープに入っている束縛に解決し、自由な局所名はパラメータと capture の
  像に限る。
  BY <ref id=3905b4e/>, <1>2, <1>2a, <ref id=b3dfa37/>, CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name

<1>5. 借用版の `Pre(V)` について A12 の性質が成り立つ。
  `<1>2a` より、名前替えは `RcVar` の名前だけを差し替えて型を残し、右辺の構成子も `Llvm` の op も
  `Destructure` が名指すフィールドの添字も `Match` のアームの `tag` も変えない。A12 が対にする各組 --
  move-bind の
  両辺、アームの結果と `Match` の束縛変数、payload と変位、catch-all の payload と scrutinee、
  `Destructure` のフィールド変数とフィールド、`App` の各引数と呼び出し先のパラメータ、`App` の結果、
  同じ名前の `RcVar`、束縛を持たない `RcVar`、そして `Llvm` 節点の型についての 4 つ -- は、どちらの側も
  型が変わらないまま対応するので、入力で成り立つ一致は借用版でも成り立つ。呼び出し先については 2 通り
  ある。直接呼び出しの callee と、束縛を持たない `RcVar` の名前は最上位の記号の名前であり、
  A13 と D6 より局所名でないので `renaming` の鍵ではなく、名前も型も動かない -- 名指す関数も
  その `params` も入力のものである。局所変数を経由する間接呼び出しでは callee は鍵でありうるが、
  `<1>2a` より型を残すので `ty(callee)` は動かない。`RcRhs::Closure` の `FuncRef` も
  `<1>2a` よりそのまま写る。`RcFunc` の欄どうしの整合は、`clone_func` が `params` と `capture` に
  `rename_var` を掛け `fn_ty` と `ret_ty` を写すことによる。
  A12 の残る 3 つの節 -- `Match` の scrutinee が union であること、`Destructure` の容器が構造体で
  あること、`Destructure` が名指すフィールドと `Match` が名指す変位がその型が実際に持つ (punched でない)
  ものであること -- も同じ理由で移る。`<1>2a` より `Match` の scrutinee と `Destructure` の容器には
  `rename_var` が掛かるだけなので `ty(scrut)` と `ty(container)` は入力のものであり、
  名指すフィールドの添字もアームの `tag` もそのまま写るからである。A12 がこの 3 つの節に伴わせる
  「その型の `is_closure()` は偽である」も、型が動かないので同じく移る。
  BY <ref id=83d98e9/>, <ref id=cb35ab1/>, <ref id=596a46d/>, <1>2, <1>2a,
     CODE src/rc_ir/borrow.rs: clone_func, CODE src/rc_ir/rename.rs: rename_var

<1>6. QED
  `<1>1` の 2 種については、A6・A11・A12 が入力の本体に直接当たる。借用版については `<1>3`・`<1>4`・
  `<1>5` が 3 つの性質を与える。
  BY <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=83d98e9/>, <1>1, <1>3, <1>4, <1>5

### L0a (`Post(V)` は `Pre(V)` と束縛を共有し、A6・A11・A12 を満たす) <!--#3c337ed-->

**言明**。`Post(V)` は `Pre(V)` から、次の 3 つの書き換えだけで得られる。

- **(書1)** 各 `Retain(v, π, s, k)` / `Release(v, π, s, k)` の節点を、同じ変数 `v` を名指す
  `Retain`/`Release` 節点の (空でありうる) 列に置き換える。
- **(書2)** 各 `Let(x, App(callee, args), k)` の `callee` を、名前だけが違う `RcVar` に替える。
- **(書3)** その `Let` の節点の直前と、その継続の先頭に、`args` の変数を名指す `Retain` / `Release`
  節点の列を挿す。

`Retain`/`Release` 以外の節点については、種類とその並び、`Let` の束縛変数、`Let` の右辺 ((書2) の名前を
除く)、`Match` のアームの構成、`Destructure` の容器とフィールド、`Eval` と `Ret` が名指す変数が、
`Pre(V)` のものに等しい。したがって次の 3 つが成り立つ。

- **(a)** `Post(V)` に `collect_bindings` を掛けて得られる `bindings` は `vars.bindings` に等しく、
  `Post(V)` を本体とする版の `param_tys` は `vars.param_tys` に等しい。
- **(b)** `Post(V)` について、L0 が `Pre(V)` について挙げる A6・A11・A12 の性質が成り立つ。
- **(c)** `Post(V)` に現れる各 `RcVar` は、`App` の callee を除いて `Pre(V)` に現れる `RcVar` そのもの
  である。callee の 2 つは名前だけが違い、型は等しい。したがって callee 以外の各名前について
  `ty(・)` は 2 つの本体で同じである。

**この命題が要るのは、site を集める本体と活性化を走らせる本体が別だからである** (第 1 節)。site は
`Pre(V)` から集まり、`vars` も `Pre(V)` の表であるのに対し、L20a・L20b・L21a・L22 と第 6 節の
節 2・節 3 は `Post(V)` の活性化について読む。

<1>1. (書1)・(書2)・(書3) の形が成り立つ。
  `rewrite` の本体は `grow_stack(|| self.rewrite_inner(node))` であり、A15 より `grow_stack` は閉包を
  ちょうど 1 回呼んでその返り値を返すので、`rewrite` が返すのは `rewrite_inner(node)` である。
  `rewrite_inner` は節点の種類で分岐する。
  `Let(x, App(callee, args), k)` の腕は `route` の返り値を callee に据え、`call_rc` が返す 2 つの列を
  `prepend_rc` で、第 1 の列はこの節点の直前に、第 2 の列は書き換えた継続の先頭に置く。`prepend_rc` が
  `rc_node` で作るのは `Retain`/`Release` の節点であり、`call_rc` が返す対の第 1 成分は `args` の要素で
  ある。`route` は `callee.clone()` を返すか、その `name` だけを借用版の名前に替えたものを返す。
  `Retain`/`Release` の腕は `rewrite_rc` を呼び、それは継続を書き換えたうえで、`is_borrow_version` が
  偽なら同じ `(v, path, state)` の節点を 1 つ、真なら `units_under(ty(v), path)` のうち `owns_unit` が
  真である unit ごとに `v` を名指す節点を並べた列を返す。`Let(x, Match(scrut, arms), k)` の腕は各アームを
  `arm.with_body(self.rewrite(&arm.body))` に、継続を `self.rewrite(k)` に替えて、同じ `x` と同じ
  `scrut` で組み直す。`MatchArm::with_body` は `body` の欄だけを差し替えて残りの欄を `self.clone()` から
  取るので、アームの `tag`・`payload`・`payload_state` は `Pre(V)` のものである。
  `Let(x, rhs, k)` の残りの腕は `rhs.clone()` を据え、`Destructure`・`Eval`・`Ret` の腕も同じ内容で
  組み直す。
  BY <ref id=3e6b0e0/>, CODE src/rc_ir/borrow.rs: prepend_rc, rc_node, expr_node, RewriteCtx::rewrite,
     RewriteCtx::rewrite_inner, RewriteCtx::rewrite_rc, RewriteCtx::call_rc, RewriteCtx::route,
     CODE src/rc_ir/ast.rs: MatchArm::with_body

<1>2. (a) が成り立つ。
  `VarTable::of` は `params` と `capture` の各 `p` について `param_tys` に `(p.name, p.ty)` を入れ、
  続けて `collect_bindings` を本体に掛ける。`VarTable::body_only` は `collect_bindings` だけを呼ぶ。
  `borrow_ify` は書き換えで `body` (グローバル初期化子では `init`) だけを差し替えるので、`params` と
  `capture` は `Pre(V)` の側のものであり、`param_tys` は等しい。`collect_bindings` が `bindings` に
  入れるのは、`Let` の右辺が `Var` / `Llvm` / `Closure` / `App` / `Match` である腕と、`Destructure` の
  腕である。`App` の腕が入れるのは `Binding::Producer` で、callee を読まない。`Retain`/`Release`/`Eval`
  の腕は継続へ進むだけである。`Binding::Join` の `arm_results` は `returned_var` が各アーム本体の継続の
  鎖を辿って着く `Ret` の変数であり、`returned_var` は `Retain`/`Release` の節点を素通りするので、
  `<1>1` より `Pre(V)` のときと同じ変数を返す。よって (書1)・(書2)・(書3) はどの `Binding` も変えない。
  BY <1>1, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var, VarTable::of,
     VarTable::body_only, CODE src/rc_ir/borrow.rs: borrow_ify

<1>3. `Post(V)` について A6 の性質が成り立つ。
  `<1>1` より `Post(V)` の束縛変数は `Pre(V)` のものと同じであり、書き換えは関数の名前の集合を
  動かさない。L0 が `Pre(V)` についてこの性質を与える。
  BY <1>1, <ref id=9cef509/>

<1>4. `Post(V)` について A11 の性質が成り立つ。
  `<1>1` より 2 つの木の差は、`Retain`/`Release` 節点の足し引きと `App` の callee の名前だけである。
  D2 より `Retain` と `Release` は変数を束縛しないので、`Pre(V)` の各束縛のスコープは `Post(V)` の
  対応する部分木であり、`Pre(V)` に在った各使用は `Post(V)` の対応する位置で同じ束縛に解決する (L0)。
  新たに現れる使用は (書1) と (書3) が置く節点が名指す変数だけである。(書1) の節点は元の
  `Retain`/`Release` と同じ変数を、元の節点が在った位置で名指す。(書3) の節点はその `Let` の引数の
  変数を名指し、その `Let` の節点が在った位置か、その継続の先頭に立つ。その `Let` はその変数を使用する
  ので、L0 の A11 よりその位置でスコープに入っている束縛に解決する。D2 のスコープはどれも節点の
  部分木なので、その `Let` を含むスコープはその `Let` の継続の部分木も含む。よって新たな使用も同じ
  束縛に解決し、自由な局所名の集合は増えない。
  BY <1>1, <ref id=b3dfa37/>, <ref id=9cef509/>

<1>5. `Post(V)` について A12 の性質が成り立つ。
  `<1>1` より書き換えが動かすのは `Retain`/`Release` 節点と `App` の callee の名前だけである。A12 が
  対にする組で `Retain`/`Release` を主語にするものは無いので、残るのは callee である。`route` が返すのは
  元の `RcVar` か、その `name` を借用版の名前に替えたものであり、型の欄は動かない。借用版は `clone_func`
  が `fresh_rename_function` で作り、`rename_var` は名前だけを差し替えて型を残すので、借用版の `fn_ty`
  と `ret_ty` と `params` の型の列は元の関数のものに等しい (P9)。よって A12 が `App` について課す
  3 つ -- 各引数と呼び出し先の対応するパラメータの型、結果の型、`ty(callee)` が呼び出し先の `fn_ty` で
  あること -- は、呼び出し先を元の関数と読んでも借用版と読んでも同じく成り立つ。`borrow_ify` は借用版を
  `borrow_funcref(func.name)` の鍵で出力の `funcs` に入れるので、替えた名前の記号はその借用版であり、
  束縛を持たない `RcVar` の型がその名前の記号の型であることも移る。残る節が主語にする構文は書き換えが
  触れないので、L0 がそのまま与える。
  BY <1>1, <ref id=83d98e9/>, <ref id=63eadd9/>, <ref id=9cef509/>,
     CODE src/rc_ir/borrow.rs: clone_func, borrow_funcref, borrow_ify, RewriteCtx::route,
     CODE src/rc_ir/rename.rs: fresh_rename_function, rename_var

<1>6. (c) が成り立つ。
  `<1>1` より、`Post(V)` の節点のうち `Retain`/`Release` でないものは、`App` の callee を除いて
  `Pre(V)` の対応する節点と同じ `RcVar` を持つ。`Retain`/`Release` の節点が名指すのは、(書1) では
  元の節点の変数、(書3) ではその `Let` の引数であり、どちらも `Pre(V)` に現れる `RcVar` である。
  callee については `route` が名前だけを替える。`ty(・)` は第 1 節よりその名前の `RcVar` の `ty` の欄で
  あり、`<1>5` の A12 より 1 つの本体の中で同じ名前の `RcVar` の型は一致するので、callee 以外の各名前に
  ついて 2 つの本体の `ty(・)` は同じである。
  BY <1>1, <1>5, <ref id=83d98e9/>, CODE src/rc_ir/borrow.rs: RewriteCtx::route

<1>7. QED
  `<1>1` が (書1)・(書2)・(書3) の形を、`<1>2` が (a) を、`<1>3`・`<1>4`・`<1>5` が (b) を、`<1>6` が
  (c) を与える。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### L1 (2 つの歩きは同じ場合分けをする) <!--#fd9b709-->

**言明**。型 `τ` と path `π` を取り、歩みの長さを `m`、その列を `cur_0, ..., cur_m` とする。

(a) `m = |π|` のとき、`sub(τ, π)` は `Some(cur_m)` を返し、`trunc(τ, π)` は `π` を返す。

(b) `m < |π|` のとき、`s = step(cur_m)` として次が成り立つ。

| `s` | 追加の条件 | `sub(τ, π)` | `trunc(τ, π)` |
|---|---|---|---|
| `Fields { held_fields, .. }` | `held_fields` が添字 `π[m]` を含まない | 中断する | 中断する |
| `Unit` | -- | `None` を返す | `π[0..m]` を返す |
| `Capture { capture_idx, .. }` | `π[m] = capture_idx` | `None` を返す | `π[0..m+1]` を返す |
| `Capture { capture_idx, .. }` | `π[m] ≠ capture_idx` | `None` を返す | 中断する |
| `NoUnit` | -- | `None` を返す | 中断する |

<1>1. `sub(τ, π)` は、`cur` を `τ` に初期化し、`π` の要素 `idx` を順に見て、`step(cur)` が
      `Fields { held_fields, .. }` なら `cur` を `held_field_type(&held_fields, idx, "subtree_type")` に
      置き換え、`NoUnit` / `Capture` / `Unit` のいずれかなら `None` を返す。`π` を使い切ったら
      `Some(cur)` を返す。
  BY CODE src/rc_ir/ownership.rs: subtree_type

<1>2. `trunc(τ, π)` は、`out` を空、`cur` を `τ` に初期化し、`π` の要素 `idx` を順に見て、
      `step(cur)` が `Fields { held_fields, .. }` なら `out` に `idx` を積んで `cur` を
      `held_field_type(&held_fields, idx, "truncate_to_unit")` に置き換え、`Unit` なら `break` し、
      `Capture { capture_idx, .. }` なら `idx = capture_idx` を `assert_eq!` した上で `out` に `idx` を
      積んで `break` し、`NoUnit` なら `panic!` する。ループを抜けたら `out` を返す。
  BY CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>3. `step` は `(ty, type_env)` だけを読む関数であり、`held_field_type` は
      `(held_fields, idx, walk_name)` だけを読む関数である。よって同じ `cur` に対して 2 つのループが
      得る `step` の答えは同じであり、`Fields` のときに得る次の `cur` も同じである
      (`walk_name` は中断時のメッセージにしか現れない)。
  BY CODE src/rc_ir/ownership.rs: unit_step, held_field_type

<1>4. `i ≤ m` かつ `i < |π|` を満たす各 `i` について、2 つのループはどちらも第 `i` 周に入り、その周の
      初めの `cur` は歩みの `cur_i` である。さらに `i < m` のとき、どちらのループも第 `i` 周をループを
      抜けずに終え、`cur` を `cur_{i+1}` に置き換える。
  <2>1. `0 ≤ m` かつ `0 < |π|` のとき、2 つのループはどちらも第 `0` 周に入り、その `cur` は
        `τ = cur_0` である。
    どちらのループも `cur` を `τ` に初期化し、`path` の要素を順に見る。`|π| > 0` なので第 `0` 周に入る。
    BY <1>1, <1>2
  <2>2. `i < m` かつ `i < |π|` であり、2 つのループが第 `i` 周に入って `cur = cur_i` であるとする。
        このとき `step(cur_i)` は `Fields { held_fields, .. }` であり、`held_fields` は添字 `π[i]` を
        含み、どちらのループもこの周を抜けずに終えて `cur` を `cur_{i+1}` に置き換える。
    `i < m` なので DEF 歩み は `cur_{i+1}` を定めており、その条件が `step(cur_i) = Fields` かつ
    `held_fields` が `π[i]` を含むことである。`<1>1` と `<1>2` より、この条件の下でどちらのループも
    `Fields` の腕に入り、`held_field_type` は中断せずに `cur_{i+1}` を返す (`<1>3`)。`subtree_type` の
    `Fields` の腕は `cur` を置き換えるだけであり、`truncate_to_unit` の `Fields` の腕は `out` に `π[i]`
    を積んで `cur` を置き換えるだけである。
    BY <1>1, <1>2, <1>3, DEF 歩み
  <2>3. QED
    `<2>1` を基底、`<2>2` を帰納の段とする `i` についての帰納。`<2>2` は第 `i` 周の終わりに `cur` が
    `cur_{i+1}` になることを与えるので、`i + 1 ≤ m` かつ `i + 1 < |π|` ならば次の周に入る。
    BY <2>1, <2>2

<1>5. `m < |π|` のとき、`step(cur_m)` は `Fields` でないか、`Fields { held_fields, .. }` であって
      `held_fields` が添字 `π[m]` を含まないかのどちらかである。
  DEF 歩み が `cur_{m+1}` を定めないのは、`m = |π|` であるか、`step(cur_m)` が `Fields` でないか、
  `Fields` だが `held_fields` が `π[m]` を含まないかのいずれかであり、いま `m < |π|` である。
  BY DEF 歩み

<1>6. CASE `m = |π|`
  `<1>4` より、`i < |π|` の各周に 2 つのループは入り、`i < m = |π|` なのでどの周もループを抜けずに
  終わる。よってどちらのループも `π` の要素をすべて使い切ってループを抜ける。`<1>1` より
  `sub` は `Some(cur)` を返し、`<1>4` よりその `cur` は `cur_m` である。`<1>2` より `trunc` は各周で
  `π[i]` を `out` に積むので `out = π` を返す。
  BY <1>1, <1>2, <1>4

<1>7. CASE `m < |π|` かつ `step(cur_m) = Fields { held_fields, .. }` かつ `held_fields` が `π[m]` を
      含まない
  `<1>4` より第 `m` 周でどちらのループも `cur = cur_m` であり、`Fields` の腕に入って
  `held_field_type(&held_fields, π[m], _)` を呼ぶ。`held_fields` が `π[m]` を含まないので、これは
  中断する。
  BY <1>1, <1>2, <1>4, CODE src/rc_ir/ownership.rs: held_field_type

<1>8. CASE `m < |π|` かつ `step(cur_m) = Unit`
  `<1>4` より第 `m` 周でどちらのループも `cur = cur_m` である。`<1>1` より `sub` は `None` を返す。
  `<1>2` より `trunc` は `break` する。それまでに `out` に積まれたのは第 `0` 周から第 `m-1` 周までの
  `π[0], ..., π[m-1]` なので、`out = π[0..m]` である。
  BY <1>1, <1>2, <1>4

<1>9. CASE `m < |π|` かつ `step(cur_m) = Capture { capture_idx, .. }` かつ `π[m] = capture_idx`
  `<1>1` より `sub` は `None` を返す。`<1>2` より `trunc` の `assert_eq!` は通り、`out` に `π[m]` を
  積んで `break` するので `out = π[0..m+1]` である。
  BY <1>1, <1>2, <1>4

<1>10. CASE `m < |π|` かつ `step(cur_m) = Capture { capture_idx, .. }` かつ `π[m] ≠ capture_idx`
  `<1>1` より `sub` は `None` を返す。`<1>2` より `trunc` の `assert_eq!` は失敗し、中断する。
  BY <1>1, <1>2, <1>4

<1>11. CASE `m < |π|` かつ `step(cur_m) = NoUnit`
  `<1>1` より `sub` は `None` を返す。`<1>2` より `trunc` は `panic!` する。
  BY <1>1, <1>2, <1>4

<1>12. QED
  `m = |π|` か `m < |π|` かで場が尽きる。後者では `step(cur_m)` は `UnitStep` の 4 つの構成子
  `NoUnit` / `Capture` / `Unit` / `Fields` のいずれかであり (`CODE src/rc_ir/ownership.rs: UnitStep`)、
  `Fields` の場合は `<1>5` より `held_fields` が `π[m]` を含まない場合だけが残り、`Capture` の場合は
  `π[m] = capture_idx` かどうかで 2 つに分かれる。よって `<1>6` から `<1>11` は場を尽くし、それぞれが
  言明の対応する行を与える。
  BY <1>6, <1>7, <1>8, <1>9, <1>10, <1>11, <1>5, CODE src/rc_ir/ownership.rs: UnitStep

### L1a (`unit_step` の 4 つの答えと、2 つの走査の降下) <!--#cd71109-->

**言明**。型 `σ` を取り、`boxed_leaf_paths` の内部関数 `go` と `rc_units_go` が `σ` に着いた時点の path を
`path` とする。`step(σ)` の 4 つの答えは、型についての述語と 2 つの走査の振る舞いを次のように決める。

| `step(σ)` | 成り立つ述語 | `go` の振る舞い | `rc_units_go` の振る舞い |
|---|---|---|---|
| `NoUnit` | `is_fully_unboxed(σ)` が真 | 何も積まずに戻る | 何も積まない |
| `Capture { capture_idx, .. }` | `is_fully_unboxed(σ)` が偽、`is_closure(σ)` が真。`capture_idx = CLOSURE_CAPTURE_IDX` | `path ++ [CLOSURE_CAPTURE_IDX]` を積んで戻る | `path ++ [capture_idx]` を積む |
| `Unit` | `is_fully_unboxed(σ)` と `is_closure(σ)` が偽、`is_box(σ) ∨ is_union(σ) ∨ is_array(σ) ∨ is_punched_array(σ)` が真 | `is_box(σ) ∨ is_array(σ)` なら `path` を積んで戻り、そうでなければ `unpunched_field_types(σ)` の各対 `(i, φ)` について `path ++ [i]` から `φ` へ降りる | `path` を積む |
| `Fields { held_fields, .. }` | 上の 3 行の述語がどれも偽。`held_fields = unpunched_field_types(σ)` | `unpunched_field_types(σ)` の各対について降りる | `held_fields` の各対について降りる |

とくに次の 2 つが成り立つ。

- **(a)** `step(σ) = Fields { held_fields, .. }` のとき `held_fields = unpunched_field_types(σ)` であり、
  `go` と `rc_units_go` はどちらも `held_fields` の各対の下へ降りる。
- **(b)** `go` が `σ` の位置で降りるのは、`step(σ)` が `Fields` であるか、`Unit` であって `is_box(σ)` も
  `is_array(σ)` も偽であるかのどちらかのときに限る。すなわち `go` が降りる位置では `step(σ)` は `NoUnit`
  でも `Capture` でもない。

**leaf と unit がずれるのはここである。**`is_fully_unboxed(σ)` と `is_closure(σ)` が偽で、
`is_union(σ)` または `is_punched_array(σ)` が真であり、`is_box(σ)` も `is_array(σ)` も偽のとき、
`step(σ) = Unit` なので `rc_units_go` は `path` を積んで止まるのに、`go` は
`unpunched_field_types(σ)` の下へ降りる。`Std::PunchedArray a` がその形の型である。宣言は
`unbox struct { _arr : Array a, _idx : I64 }` であり (`CODE src/fixstd/std.fix: PunchedArray`)、
その最上位 tycon は `Std::PunchedArray` なので `is_punched_array` は真である
(`CODE src/ast/types.rs: TypeNode::is_punched_array`, `TypeNode::toplevel_tycon_satisfies`,
`CODE src/fixstd/builtin.rs: is_punched_array_tycon`, `make_punched_array_tycon`,
`CODE src/constants.rs: PUNCHED_ARRAY_NAME`)。

<1>1. `unit_step` は上から順に、`is_fully_unboxed(σ)` で `NoUnit` を、`is_closure(σ)` で
      `Capture { capture_idx: CLOSURE_CAPTURE_IDX, field_count: CLOSURE_FIELD_COUNT }` を、
      `is_box(σ) || is_union(σ) || is_array(σ) || is_punched_array(σ)` で `Unit` を、それ以外で
      `Fields { field_count: .., held_fields: unpunched_field_types(σ) }` を返す。
  BY CODE src/rc_ir/ownership.rs: unit_step

<1>2. `go` は上から順に、`is_fully_unboxed(σ)` で何も積まずに戻り、`is_closure(σ)` で
      `path ++ [CLOSURE_CAPTURE_IDX]` を積んで戻り、`is_box(σ)` で `path` を積んで戻り、`is_array(σ)` で
      `path` を積んで戻り、それ以外で `unpunched_field_types(σ)` の各対 `(i, φ)` について `path` に `i` を
      積んで `φ` へ降りる。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. `rc_units_go` は `step(σ)` で分岐し、`NoUnit` で何も積まず、`Capture { capture_idx, .. }` で
      `path ++ [capture_idx]` を積み、`Unit` で `path` を積み、`Fields { held_fields, .. }` で
      `held_fields` の各対 `(i, φ)` について `path` に `i` を積んで `φ` へ降りる。
  BY CODE src/rc_ir/ownership.rs: rc_units_go

<1>4. QED
  表の 4 行は、`<1>1` が与える述語と、`<1>2` および `<1>3` の分岐を突き合わせたものである。(a) は
  `<1>1` の第 4 行、`<1>2` の最後の腕、`<1>3` の `Fields` の腕による。(b) は `<1>1` と `<1>2` による --
  `go` が降りるのは `is_fully_unboxed(σ)`・`is_closure(σ)`・`is_box(σ)`・`is_array(σ)` がどれも偽の
  ときであり、そのとき `<1>1` の第 1 行と第 2 行の述語は偽なので `step(σ)` は `NoUnit` でも `Capture` でも
  なく、`Unit` (`is_union(σ) ∨ is_punched_array(σ)` による) か `Fields` である。
  BY <1>1, <1>2, <1>3

### L1b (扱う型は A10 を満たす) <!--#fb62043-->

**言明**。DEF 扱う型 の意味の扱う型はすべて A10 を満たす。さらに、`τ` が扱う型であるとき次の型も
扱う型である。

- DEF 歩み が `τ` と任意の path について定める各 `cur_i`。
- `sub(τ, π)` が `Some(σ)` を返すときの `σ`。
- `boxed_leaf_paths` の `go` と `rc_units_go` が `τ` の位置から降りて着く各型。

<1>1. 根の型は A10 を満たす。
  DEF 扱う型 より根の型はパラメータ・capture の `RcVar` の型か、`Pre(V)` に現れる `RcVar` の型か、
  `Llvm` 節点の結果の型である (D1 より `RcFunc` はパラメータの列と capture を持ち、その各 `RcVar` は
  型を持つ)。固定した出力版のこの 3 種は、入力のプログラムに現れる型である -- `f_own` の版と
  グローバル初期化子の版の `Pre(V)`・パラメータ・capture は入力のものそのものであり (`borrow_ify` は
  `func.clone()` を写し、グローバルは `g.init` を写す)、借用版のそれらは入力の関数のものの束縛変数を
  一斉に付け替えたものであって (P9)、`rename_var` は名前だけを差し替えて型を残す。A10 は、プログラムに
  現れる型が ground であり、その tycon に kind の要求するだけの引数が与えられており、その tycon が
  `type_env` にあり、`no_size_in_place` の in-place の降下が有限であると述べる。
  BY <ref id=8412761/>, <ref id=a502f3e/>, <ref id=63eadd9/>, DEF 扱う型, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func,
     CODE src/rc_ir/rename.rs: rename_var

<1>2. 扱う型はすべて A10 を満たす。
  <2>1. DEF 扱う型 の第 2 の種 -- 根の型から `unpunched_field_types` の対の第 2 成分を有限回取って
        到達する型 -- は、ground であり、その tycon に kind の要求するだけの引数が与えられており、
        その tycon は `type_env` にある。またその歩みは有限である。
    A10 の到達型の節がこの 4 つを述べ、DEF 扱う型 の第 2 の種はまさにその節が名指す型である。
    BY <ref id=8412761/>, DEF 扱う型
  <2>2. その各型 `σ` について、`no_size_in_place` の in-place の降下は有限である。
    A10 は「`no_size_in_place` の降下は unbox のフィールドだけを辿るのに対し、`unit_step` の `Fields` の
    腕と `subtree_type` / `truncate_to_unit` はフィールドの型を boxed であっても取るので、こちらの方が
    広い」と述べる。すなわち `σ` からの in-place の降下で到達する型は、`σ` から
    `unpunched_field_types` を繰り返し取って到達する型でもある。`σ` 自身が根の型からその歩みで到達する
    型なので、`σ` からの降下は根の型からのその歩みの部分であり、`<2>1` よりその歩みは有限である。
    BY <2>1, <ref id=8412761/>
  <2>3. QED
    `<1>1` が根の型について A10 の 4 つを与える。第 2 の種については `<2>1` が前の 3 つを、`<2>2` が
    4 つ目を与える。
    BY <1>1, <2>1, <2>2, DEF 扱う型

<1>3. DEF 歩み の `cur_{i+1}` は、`cur_i` から `unpunched_field_types` の対の第 2 成分を取ったもので
      ある。
  DEF 歩み は、`step(cur_i)` が `Fields { held_fields, .. }` であって `held_fields` が添字 `π[i]` の対を
  含むときに、`cur_{i+1}` をその対の型と定める。L1a (a) より
  `held_fields = unpunched_field_types(cur_i)` である。
  BY <ref id=cd71109/>, DEF 歩み

<1>4. `sub(τ, π) = Some(σ)` のとき、`σ` は DEF 歩み の `cur_{|π|}` である。
  BY <ref id=fd9b709/>

<1>5. `go` と `rc_units_go` が `τ` の位置から降りて着く型は、`unpunched_field_types(τ)` の対の第 2 成分で
      ある。
  L1a の表より、`go` が降りるのは `unpunched_field_types(τ)` の各対の下であり、`rc_units_go` が降りるのは
  `Fields { held_fields, .. }` の `held_fields` の各対の下であって、L1a (a) より
  `held_fields = unpunched_field_types(τ)` である。
  BY <ref id=cd71109/>

<1>6. QED
  `<1>2` が前半を与える。後半の 3 つは `<1>3`、`<1>4`、`<1>5` より、扱う型から
  `unpunched_field_types` の対の第 2 成分を有限回取って到達する型であり、DEF 扱う型 よりそれも扱う型で
  ある。
  BY <1>2, <1>3, <1>4, <1>5, DEF 扱う型

### L1c (`param_tys` の鍵はパラメータ・capture である) <!--#fa1a6ce-->

**言明**。`g` を、`borrow_ify` の入力の関数か、`borrow_ify` が作る出力版のうち関数の版 (`f_own` の版と
借用版) の `RcFunc` とし、`vars = VarTable::of(g)` とする。このとき `vars.param_tys.get(r)` が
`Some(τ)` であることと、`r` が `g` のパラメータか capture であることは同値であり、そのとき
`τ = ty(r)` である (`ty(・)` は `g` の本体とパラメータ・capture について読む)。本体だけから作る
`VarTable::body_only(b)` では `param_tys` は空である。

<1>1. `VarTable::of(g)` は `VarTable::empty()` から始め、`g.params` と `g.capture` の各 `p` について
      `param_tys` に `(p.name, p.ty)` を入れ、`var_tys` にも同じ `p.ty` を入れる。続けて呼ぶ
      `collect_bindings` は `bindings` と `var_tys` と `closure_targets` にしか入れない。
      `VarTable::body_only(b)` も `VarTable::empty()` から始め、`collect_bindings(b, ..)` だけを呼ぶ。
      `VarTable::empty()` の `param_tys` は空の写像である。
  BY CODE src/rc_ir/ownership.rs: collect_bindings, VarTable::of, VarTable::body_only, VarTable::empty

<1>2. QED
  `<1>1` より `param_tys` の鍵は `g` のパラメータ・capture の名前ちょうどであり、その値は `g` がその
  名前に宣言した型 `p.ty` である。`g` のパラメータ・capture の `RcVar` はその型 `p.ty` を持つ。`g` が
  `borrow_ify` の入力の関数であれば A12 が直接この一致を与える。`g` が出力版の `RcFunc` であれば、
  第 1 節の固定をその版に取って L0 が同じ性質を渡す (`f_own` の版は入力の本体そのものであり、借用版は
  L0 の `<1>5` が与える)。よってどちらの場合も `ty(r) = p.ty = τ` である。`VarTable::body_only` は
  `param_tys` に何も入れないので空である。
  BY <1>1, <ref id=83d98e9/>, <ref id=9cef509/>

### L1d (`type_env` は組み込みの宣言をそのまま持つ) <!--#33ee52f-->

**言明**。プログラムの `type_env` について、`type_env.tycons()` が `make_array_tycon()` に与える
`TyConInfo` の `variant` は `TyConVariant::Array` であり、各 `make_funptr_tycon(n)` に与える `TyConInfo` の
`variant` は `TyConVariant::Primitive` である。また、扱う型 `σ` について `is_funptr(σ)` は値を返す。
したがって、扱う型 `σ` の最上位 tycon の `TyConInfo` の `variant` が `TyConVariant::Struct` または
`TyConVariant::Union` であるとき、`is_array(σ)` も `is_funptr(σ)` も偽である。

**この命題が要るのは、`is_array` と `is_funptr` が名前の比較だからである。** `is_array(σ)` は `σ` の最上位
tycon が `make_array_tycon()` に等しいかを問い (`CODE src/ast/types.rs: TypeNode::is_array`,
`CODE src/fixstd/builtin.rs: is_array_tycon`)、`is_funptr(σ)` はその名前が namespace `Std` を持ち
`#FunPtr` に 10 進表記が続く形かを問う (`CODE src/ast/types.rs: TypeNode::is_funptr`,
`CODE src/fixstd/builtin.rs: is_funptr_tycon`)。どちらも `variant` を読まないので、`variant` が
`Struct` である tycon がこの 2 つの名前を持たないことを別に述べる者が要る。

**`is_funptr` が値を返すことを言明に持つのは、その側が中断しうるからである。** `is_funptr_tycon` は
namespace が `Std` で名前が `#FunPtr` で始まるとき、残りを `parse::<u32>().unwrap()` に掛ける。
10 進表記でない残りを持つ鍵が在れば、その型についての `is_funptr` は中断する
(`CODE src/fixstd/builtin.rs: is_funptr_tycon`)。読む者は L11 の `<1>0a` と L16 の `<1>3a` である。

<1>1. `type_env.tycons()` の項目のうち、鍵 `tc` の `tc.name.namespace` が `Std` の 1 段であって
      `tc.name.name` が `FUNPTR_NAME` (`"#FunPtr"`) で始まるものは、`bulitin_tycons()` が
      `make_funptr_tycon(n)` (`n` は 1 以上 `FUNPTR_ARGS_MAX` 以下) の鍵の下に置いた項目である。
  A28 の言明である。
  BY <ref id=3d4be43/>

<1>2. `bulitin_tycons()` は `make_array_tycon()` に `variant: TyConVariant::Array` の `TyConInfo` を、
      `1..=FUNPTR_ARGS_MAX` の各 `arity` について `make_funptr_tycon(arity)` に
      `variant: TyConVariant::Primitive` の `TyConInfo` を置く。`make_array_tycon()` の名前は `Array`
      である。`make_funptr_tycon(arity)` の namespace は `Std` の 1 段であり、その名前は `FUNPTR_NAME`
      に `arity` の 10 進表記を続けたものである。
  BY CODE src/fixstd/builtin.rs: bulitin_tycons, make_array_tycon, make_array_name,
     make_funptr_tycon, make_funptr_name,
     CODE src/constants.rs: ARRAY_NAME, FUNPTR_NAME, FUNPTR_ARGS_MAX

<1>3. `make_funptr_tycon(n)` が `type_env.tycons()` の鍵であるとき、その項目の `TyConInfo` の `variant`
      は `TyConVariant::Primitive` である。
  `<1>2` よりその鍵の namespace は `Std` の 1 段であり、その名前は `#FunPtr` で始まる。`<1>1` より
  その項目は `bulitin_tycons()` が `make_funptr_tycon(n)` の鍵の下に置いたものであり、`<1>2` より
  その `variant` は `TyConVariant::Primitive` である。
  BY <1>1, <1>2

<1>4. `type_env.tycons()` の項目のうち鍵が `make_array_tycon()` であるものは、`bulitin_tycons()` が
      その鍵の下に置いた項目である。したがってその `TyConInfo` の `variant` は `TyConVariant::Array`
      である。
  前半は A28 の言明である -- A28 は `make_array_tycon()` の項目をその「とくに」の節で名指す。
  後半は `<1>2` による。
  BY <1>2, <ref id=3d4be43/>

<1>5. 扱う型 `σ` について `is_funptr(σ)` は値を返す。
  `is_funptr` は `toplevel_tycon_satisfies` に `is_funptr_tycon(tc).is_some()` を渡す。
  `toplevel_tycon()` が `None` を返す型では、`toplevel_tycon_satisfies` は述語を呼ばずに偽を返す。
  `Some(tc)` のとき、`is_funptr_tycon` は `tc.name.namespace` が `Std` の 1 段でなければ `None` を、
  `tc.name.name` が `FUNPTR_NAME` で始まらなければ `None` を返して終わる。残るのは `tc` がその 2 つを
  満たす場合である。`σ` は扱う型なので L1b より A10 を満たし、`tc` は `type_env.tycons()` の鍵である。
  `<1>1` よりその項目は `make_funptr_tycon(n)` (`n` は 1 以上 `FUNPTR_ARGS_MAX` 以下) の鍵の下に置かれた
  ものなので `tc = make_funptr_tycon(n)` であり、`<1>2` より `FUNPTR_NAME` の後に続くのは `n` の
  10 進表記だから、`parse::<u32>()` は値を返す。
  BY <1>1, <1>2, <ref id=8412761/>, <ref id=fb62043/>, DEF 扱う型,
     CODE src/ast/types.rs: TypeNode::is_funptr, TypeNode::toplevel_tycon,
     TypeNode::toplevel_tycon_satisfies,
     CODE src/fixstd/builtin.rs: is_funptr_tycon

<1>6. QED
  言明の前半は `<1>3` と `<1>4` であり、`is_funptr` の全域性は `<1>5` である。
  `σ` を扱う型とし、その最上位 tycon `tc` の `TyConInfo` の `variant` が `TyConVariant::Struct` か
  `TyConVariant::Union` であるとする。L1b より `σ` は A10 を満たすので `tc` は `type_env.tycons()` の鍵で
  あり、その項目の `TyConInfo` がその `variant` を持つ。`<1>4` より鍵 `make_array_tycon()` の項目の
  `variant` は `TyConVariant::Array` なので `tc ≠ make_array_tycon()` であり、`is_array(σ)` はその
  等しさを問うので偽である。`is_funptr_tycon(tc)` が `Some` を返すとすると `tc.name.namespace` は `Std` の
  1 段で `tc.name.name` は `#FunPtr` で始まるので、`<1>1` より `tc` は `make_funptr_tycon(n)` であり、
  `<1>3` よりその `variant` は `TyConVariant::Primitive` になって場合の条件に反する。よって
  `is_funptr_tycon(tc)` は `None` を返し、`is_funptr(σ)` は偽である。
  BY <1>1, <1>3, <1>4, <1>5, <ref id=8412761/>, <ref id=fb62043/>, DEF 扱う型,
     CODE src/ast/types.rs: TypeNode::is_array, TypeNode::is_funptr,
     CODE src/fixstd/builtin.rs: is_array_tycon, is_funptr_tycon

### L1e (`Binding::Param` を持つ名前は、この版のパラメータ・capture である) <!--#14909ed-->

**言明**。第 1 節が固定する出力版とその `vars` について、次の 3 つは同値である。

- **(i)** `vars.bindings.get(x)` が `Some(Binding::Param)` である。
- **(ii)** `x` はこの版のパラメータか capture である。
- **(iii)** `pty(x)` が値を持つ。

3 つが成り立つとき `pty(x) = Some(ty(x))` である。

**(i) から (ii) へ渡る向きは、`collect_bindings` が `Binding::Param` を 1 度も記録しないことに立つ。**
`VarTable::of` が語るのは (ii) から (i) と (iii) を出す向きだけである。

<1>1. 固定した版が関数の版であるとき、その `vars` は `VarTable::of` がその版の `RcFunc` から作った表で
      あり、グローバル初期化子の版であるとき `VarTable::body_only(g.init)` である。
  `RewriteCtx::new` は `vars` に `VarTable::of(func)` を置き、`borrow_ify` のグローバルを写す繰り返しは
  `VarTable::body_only(&g.init)` で作った表を `RewriteCtx` に据える。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, borrow_ify

<1>2. `VarTable::of(g)` が `bindings` に `Binding::Param` を入れるのは `g` のパラメータ・capture の各
      名前についてであり、`collect_bindings` が `bindings` に入れる `Binding` は `Move`・`Llvm`・
      `Producer`・`Payload`・`Field`・`Join` の 6 種で、`Binding::Param` を 1 度も入れない。
  `collect_bindings` は `Let` の右辺で場合を分けて `Move` / `Llvm` / `Producer` / `Join` を入れ、`Match`
  のアームの payload 変数に `Payload` を、`Destructure` のフィールド変数に `Field` を入れる。
  BY CODE src/rc_ir/ownership.rs: collect_bindings, VarTable::of, VarTable::body_only

<1>3. 関数の版では 3 つは同値であり、そのとき `pty(x) = Some(ty(x))` である。
  `<1>1` よりこの版の `vars` は `VarTable::of` がその版の `RcFunc` から作った表である。`<1>2` より
  `Binding::Param` を入れるのはその `RcFunc` のパラメータ・capture についての行だけなので、(i) と (ii)
  は同値である。(ii) と (iii) の同値と、そのときの `pty(x) = Some(ty(x))` は L1c である。
  BY <1>1, <1>2, <ref id=fa1a6ce/>

<1>4. グローバル初期化子の版では 3 つとも偽である。
  `<1>1` よりこの版の `vars` は `VarTable::body_only(g.init)` である。`<1>2` より `collect_bindings` は
  `Binding::Param` を入れないので (i) は偽であり、L1c より `param_tys` は空なので (iii) も偽である。
  この版の `Pre(V)` は `init` であり、D1 より `init` はパラメータも capture も持たないので (ii) も
  偽である。
  BY <1>1, <1>2, <ref id=a502f3e/>, <ref id=fa1a6ce/>

<1>5. QED
  `<1>1` より版は 2 種であり、`<1>3` と `<1>4` がそれぞれを扱った。
  BY <1>1, <1>3, <1>4

### L2 (`units_under` の 2 つの形) <!--#9f11796-->

**言明**。`sub(τ, π)` が `Some(σ)` を返すとき `under(τ, π) = { π ++ u : u ∈ units(σ) }` であり、`None` を
返すとき `under(τ, π) = [π]` である。`sub(τ, π)` が中断するとき `under(τ, π)` は中断する。

<1>1. `under(τ, π)` は `sub(τ, π)` の答えで分岐し、`Some(sty)` のときは `rc_units(sty)` の各要素 `u` に
      `π` を前置した列を返し、`None` のときは `vec![π.clone()]` を返す。
  BY CODE src/rc_ir/ownership.rs: units_under

<1>2. QED
  `under` は `sub` を呼んでから分岐するので、`sub` が中断すれば `under` も中断する。
  BY <1>1

### L3 (`Unit` の型の unit はただ 1 つ) <!--#f221813-->

**言明**。`step(σ) = UnitStep::Unit` のとき `units(σ) = [[]]` である。

<1>1. `rc_units(σ)` は空の `path` と空の `out` で `rc_units_go(σ, ·, path, out)` を呼び、`out` を返す。
  BY CODE src/rc_ir/ownership.rs: rc_units

<1>2. `rc_units_go` は `step(σ)` で分岐し、`UnitStep::Unit` の腕で `out` に `path.clone()` を積んで
      終わる。
  BY CODE src/rc_ir/ownership.rs: rc_units_go

<1>3. QED
  `<1>1` より最初の呼び出しの `path` は空列なので、`<1>2` の腕が積むのは `[]` 1 つだけである。
  BY <1>1, <1>2

### L4 (`trunc` の答えを再び歩く) <!--#3eb1174-->

**言明**。`trunc(τ, π)` が値 `t` を返すとき、次が成り立つ。

- `trunc(τ, t) = t`。
- `under(τ, t)` は、L1 の場合 (a) では `under(τ, π)` と同じ呼び出しであり、場合 (b) の `Unit` の行と
  `Capture` の行では `[t]` である。

<1>1. CASE L1 の場合 (a) (`m = |π|`)
  L1 (a) より `t = π` であるから、`trunc(τ, t)` は `trunc(τ, π)` と同じ呼び出しで `t` を返し、
  `under(τ, t)` は `under(τ, π)` と同じ呼び出しである。
  BY <ref id=fd9b709/>

<1>2. CASE L1 の場合 (b) の `Unit` の行
  <2>1. `t = π[0..m]` であり `|t| = m` である。
    BY <ref id=fd9b709/>
  <2>2. `τ` の `t` に沿う歩みの長さは `m` であり、その列は `cur_0, ..., cur_m` である。
    `i < m` について `t[i] = π[i]` なので、DEF 歩み が `cur_1, ..., cur_m` を定める条件は `π` のときと
    同じである。`|t| = m` なので `cur_{m+1}` を定める条件の 1 つ目 (`i < |t|`) が破れる。
    BY <2>1, DEF 歩み
  <2>3. `trunc(τ, t) = t` であり、`sub(τ, t) = Some(cur_m)` である。
    `<2>2` より `t` については L1 の場合 (a) が当てはまる。
    BY <2>2, <ref id=fd9b709/>
  <2>4. QED
    `<2>3` と L2 より `under(τ, t) = { t ++ u : u ∈ units(cur_m) }` であり、この場合 `step(cur_m) = Unit`
    なので L3 より `units(cur_m) = [[]]`、すなわち `under(τ, t) = [t]` である。
    BY <2>3, <ref id=9f11796/>, <ref id=f221813/>

<1>3. CASE L1 の場合 (b) の `Capture` の行 (`π[m] = capture_idx`)
  <2>1. `t = π[0..m+1]` であり `|t| = m + 1`、`t[m] = π[m] = capture_idx` である。
    BY <ref id=fd9b709/>
  <2>2. `τ` の `t` に沿う歩みの長さは `m` であり、その列は `cur_0, ..., cur_m` である。
    `i < m` について `t[i] = π[i]` なので `cur_1, ..., cur_m` は `π` のときと同じである。
    `step(cur_m) = Capture` は `Fields` ではないので、DEF 歩み は `cur_{m+1}` を定めない。
    BY <2>1, DEF 歩み
  <2>3. QED
    `<2>2` より `t` については L1 の場合 (b) が当てはまり、`step(cur_m) = Capture` かつ
    `t[m] = capture_idx` なので、`sub(τ, t) = None` かつ `trunc(τ, t) = t[0..m+1] = t` である。
    L2 より `under(τ, t) = [t]` である。
    BY <2>1, <2>2, <ref id=fd9b709/>, <ref id=9f11796/>

<1>4. QED
  L1 の場合分けのうち、`trunc(τ, π)` が値を返すのは (a) と、(b) の `Unit` の行と `Capture`
  (`π[m] = capture_idx`) の行だけである。この 3 つを `<1>1`、`<1>2`、`<1>3` が扱った。
  BY <1>1, <1>2, <1>3, <ref id=fd9b709/>

### L5 (歩みの合成) <!--#323bb77-->

**言明**。`sub(τ, p) = Some(σ)` のとき、任意の path `q` について次が成り立つ。

- `τ` の `p ++ q` に沿う歩みは、`τ` の `p` に沿う歩み (長さ `|p|`) の後ろに `σ` の `q` に沿う歩みを
  つないだものである。
- `sub(τ, p ++ q) = sub(σ, q)` であり、`trunc(τ, p ++ q) = p ++ trunc(σ, q)` である (両辺は同時に中断する)。
- `leaves(τ)` のうち `p` を前置に持つものの全体は `{ p ++ λ : λ ∈ leaves(σ) }` である。
- `units(τ) ⊇ { p ++ u : u ∈ units(σ) }` である。

<1>1. `τ` の `p ++ q` に沿う歩みの最初の `|p| + 1` 個の型は、`τ` の `p` に沿う歩みの型であり、
      `cur_{|p|} = σ` である。
  `sub(τ, p) = Some(σ)` なので L1 より `τ` の `p` に沿う歩みの長さは `|p|` であり、その最後の型は `σ` で
  ある。DEF 歩み は添字 `i < |p|` について `p ++ q` の第 `i` 要素 (`= p[i]`) だけを読むので、最初の
  `|p| + 1` 個は一致する。
  BY <ref id=fd9b709/>, DEF 歩み

<1>2. 各 `j ≥ 0` について、`τ` の `p ++ q` に沿う歩みが `cur_{|p| + j}` を定めることと、`σ` の `q` に
      沿う歩みが `cur_j` を定めることは同値であり、定めるときその 2 つは同じ型である。
  `<1>1` より `τ` の `p ++ q` に沿う歩みの `cur_{|p|}` は `σ` であり、DEF 歩み は添字 `|p| + j` に
  ついて `(p ++ q)[|p| + j] = q[j]` を読む。よって `j` についての漸化式は `σ` の `q` に沿う歩みのものと
  同じであり、基底も一致する。
  BY <1>1, DEF 歩み

<1>3. `sub(τ, p ++ q) = sub(σ, q)` である。
  `<1>1` より `sub(τ, p ++ q)` のループの最初の `|p|` 周はすべて `Fields` の腕を通り (`sub(τ, p)` が
  `Some` を返したので、L1 の場合 (a) が `(τ, p)` に当てはまる)、`cur = σ` になる。残りの周は
  `sub(σ, q)` のループそのものである。
  BY <1>1, <1>2, <ref id=fd9b709/>, CODE src/rc_ir/ownership.rs: subtree_type

<1>4. `trunc(τ, p ++ q) = p ++ trunc(σ, q)` である。
  `<1>1` より `trunc(τ, p ++ q)` のループの最初の `|p|` 周はすべて `Fields` の腕を通り、`out` に
  `p[0], ..., p[|p|-1]` を積んで `cur = σ` になる。残りの周は `trunc(σ, q)` のループそのものであり、
  そこで `out` に積まれるもの (と `break` / 中断) は `trunc(σ, q)` と同じである。
  BY <1>1, <1>2, <ref id=fd9b709/>, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>5. `leaves(τ)` のうち `p` を前置に持つものの全体は `{ p ++ λ : λ ∈ leaves(σ) }` である。
  `sub(τ, p) = Some(σ)` なので L1 の場合 (a) が `(τ, p)` に当てはまり、`τ` の `p` に沿う歩みの長さは
  `|p|` で `cur_{|p|} = σ` である。DEF 歩み より `i < |p|` の各段で `step(cur_i)` は
  `Fields { held_fields, .. }` であって `held_fields` は添字 `p[i]` の対を含む。L1a (a) より `go` は
  その各段で `held_fields` の同じ対の下へ降りるので、`go` は `path = p` で `σ` に着く。`go` はその位置で
  `σ` について走るので、`p` を前置に持つ path として積むのは、`σ` から始めた `go` が積む path の前に
  `p` を置いたものに限る。
  BY <ref id=fd9b709/>, <ref id=cd71109/>, DEF 歩み, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>6. `units(τ) ⊇ { p ++ u : u ∈ units(σ) }` である。
  L1 の場合 (a) と DEF 歩み より、`τ` の `p` に沿う歩みの各段 `i < |p|` で `step(cur_i)` は
  `Fields { held_fields, .. }` であって `held_fields` は添字 `p[i]` の対を含む。L1a (a) より
  `rc_units_go` はその各段で同じ対の下へ降りるので、`path = p` で `cur_{|p|} = σ` に達し、そこから
  `units(σ)` の各要素を `p` の後ろに積む。
  BY <ref id=fd9b709/>, <ref id=cd71109/>, DEF 歩み, CODE src/rc_ir/ownership.rs: rc_units_go

<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### L6 (unit は自分自身へ切り詰まる) <!--#e74af85-->

**言明**。`u ∈ units(τ)` のとき `trunc(τ, u) = u` であり、`under(τ, u) = [u]` である。さらに次の 2 つの
うちちょうど 1 つが成り立つ。

- **(i)** `τ` の `u` に沿う歩みの長さは `|u|` であり、`unit_step(cur_{|u|}) = Unit` である。
- **(ii)** `u = q ++ [capture_idx]` の形であり、`τ` の `q` に沿う歩みの長さは `|q|` であって
  `unit_step(cur_{|q|}) = Capture { capture_idx, .. }` である。

<1>1. `rc_units_go` が `out` に `u` を積むのは、次の 2 つの場合だけである。(i) `path = u` の位置で
      `unit_step` が `Unit` を返した。(ii) `path = q` の位置で `unit_step` が
      `Capture { capture_idx, .. }` を返し、`u = q ++ [capture_idx]` である。どちらの場合も、`path` の
      各真の接頭辞の位置で `unit_step` は `Fields` であり、`path` の次の添字はその `held_fields` に
      含まれる。
  `rc_units_go` は `NoUnit` で何も積まず、`Unit` で `path` を積み、`Capture` で `path ++ [capture_idx]` を
  積み、`Fields` で `held_fields` の各対の添字を `path` に積んで降りる。`path` が伸びるのは `Fields` の
  腕だけである。
  BY CODE src/rc_ir/ownership.rs: rc_units_go

<1>2. 場合 (i) では、`τ` の `u` に沿う歩みの長さは `|u|` である。
  `<1>1` より `i < |u|` の各位置で `unit_step` は `Fields` であり、`u[i]` は `held_fields` に含まれる。
  よって DEF 歩み は `cur_0, ..., cur_{|u|}` を定める。
  BY <1>1, DEF 歩み

<1>3. 場合 (i) では `trunc(τ, u) = u` かつ `sub(τ, u) = Some(cur_{|u|})` であり、
      `unit_step(cur_{|u|}) = Unit` である。
  `<1>2` と L1 の場合 (a) による。`cur_{|u|}` は `path = u` の位置の型なので `unit_step` は `Unit` である。
  BY <1>1, <1>2, <ref id=fd9b709/>

<1>4. 場合 (i) では `under(τ, u) = [u]` である。
  `<1>3` と L2 より `under(τ, u) = { u ++ w : w ∈ units(cur_{|u|}) }` であり、`<1>3` と L3 より
  `units(cur_{|u|}) = [[]]` である。
  BY <1>3, <ref id=9f11796/>, <ref id=f221813/>

<1>5. 場合 (ii) では、`τ` の `u` に沿う歩みの長さは `|q| = |u| - 1` であり、`unit_step(cur_{|q|})` は
      `Capture { capture_idx, .. }` かつ `u[|q|] = capture_idx` である。
  `<1>1` より `i < |q|` の各位置で `unit_step` は `Fields` なので `cur_0, ..., cur_{|q|}` が定まり、
  `cur_{|q|}` の `unit_step` は `Capture` なので DEF 歩み は `cur_{|q|+1}` を定めない。
  BY <1>1, DEF 歩み

<1>6. 場合 (ii) では `trunc(τ, u) = u` かつ `sub(τ, u) = None`、したがって `under(τ, u) = [u]` である。
  `<1>5` と L1 の場合 (b) の `Capture` (`u[|q|] = capture_idx`) の行より
  `trunc(τ, u) = u[0..|q|+1] = u` かつ `sub(τ, u) = None` である。L2 より `under(τ, u) = [u]` である。
  BY <1>5, <ref id=fd9b709/>, <ref id=9f11796/>

<1>7. QED
  `<1>1` の 2 つの場合を `<1>3`/`<1>4` と `<1>6` が扱った。言明の (i) は `<1>2` と `<1>3`、(ii) は
  `<1>5` である。この 2 つは排他である。`<1>2` は場合 (i) で `τ` の `u` に沿う歩みの長さが `|u|` だと
  述べ、`<1>5` は場合 (ii) でその長さが `|u| - 1` だと述べる。DEF 歩み は 1 つの `(τ, u)` に 1 つの長さを
  与え、`|u| ≠ |u| - 1` なので、2 つが同時に成り立つことはない。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, DEF 歩み

### L7 (unit の下の leaf はその unit へ切り詰まる) <!--#212f2e7-->

**言明**。`u ∈ units(τ)` とし、`λ ∈ leaves(τ)` が `u` を前置に持つとする。このとき
`trunc(τ, λ) = u` である。さらに `Λ_τ(u) := { λ ∈ leaves(τ) : u ⊑ λ }` は空でない。

<1>1. L6 の (i) のとき、`trunc(τ, λ) = u` である。
  `u ⊑ λ` なので、`τ` の `λ` に沿う歩みの最初の `|u| + 1` 個の型は `τ` の `u` に沿う歩みの型と一致する
  (DEF 歩み は添字 `i < |u|` について `λ[i] = u[i]` だけを読む)。L6 の (i) より `unit_step(cur_{|u|})`
  は `Unit` である。`λ = u` なら L1 の場合 (a) より `trunc(τ, λ) = u`。`λ ≠ u` すなわち `|λ| > |u|` なら
  L1 の場合 (b) の `Unit` の行より `trunc(τ, λ) = λ[0..|u|] = u`。
  BY <ref id=fd9b709/>, <ref id=e74af85/>, DEF 歩み

<1>2. L6 の (ii) のとき、`trunc(τ, λ) = u` である。
  `u = q ++ [capture_idx]` であり、`u ⊑ λ` なので `λ[|q|] = capture_idx` である。L6 の (ii) より `τ` の
  `u` に沿う歩みの長さは `|q|` であり、`unit_step(cur_{|q|})` は `Capture { capture_idx, .. }` である。
  DEF 歩み が `cur_1, ..., cur_{|q|}` を定めるのに読むのは添字 `i < |q|` の要素だけであり、`q ⊑ u ⊑ λ`
  よりその範囲で `λ[i] = u[i]` なので、`τ` の `λ` に沿う歩みの最初の `|q| + 1` 個の型は `τ` の `u` に
  沿う歩みのものと一致する。`unit_step(cur_{|q|})` が `Fields` でないので、`τ` の `λ` に沿う歩みの長さも
  `|q|` である。`|λ| ≥ |u| = |q| + 1 > |q|` なので L1 の場合 (b) の `Capture` の行が当てはまり、
  `trunc(τ, λ) = λ[0..|q|+1] = u` である。
  BY <ref id=fd9b709/>, <ref id=e74af85/>, DEF 歩み

<1>3. `Λ_τ(u)` は空でない。
  `τ` は扱う型なので L1b より A10 を満たし、P1 の定義域に入る。P1 の後半より、`u` はある
  `λ ∈ leaves(τ)` の `trunc(τ, λ)` である。`trunc` が `out` に積むのは
  引数の path の要素を順に取ったものなので、`trunc(τ, λ)` は `λ` の接頭辞である。よって `u ⊑ λ` であり
  `λ ∈ Λ_τ(u)` である。
  BY <ref id=3597669/>, <ref id=fb62043/>, DEF 扱う型, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>4. QED
  L6 は (i) と (ii) のちょうど 1 つが成り立つと述べる。
  BY <1>1, <1>2, <1>3, <ref id=e74af85/>

### L8 (leaf と unit は同時に空になる) <!--#1d99428-->

**言明**。`leaves(τ) = ∅` と `units(τ) = ∅` と `is_fully_unboxed(τ)` は同値である。

<1>1. `is_fully_unboxed(τ)` ならば `leaves(τ) = ∅` かつ `units(τ) = ∅` である。
  `boxed_leaf_paths` の `go` は最初に `is_fully_unboxed` を見て何も積まずに戻り、`unit_step` は最初に
  `is_fully_unboxed` を見て `NoUnit` を返し、`rc_units_go` は `NoUnit` で何も積まない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go

<1>2. ASSUME (U) `is_fully_unboxed(τ)` が偽である
      PROVE  `leaves(τ) ≠ ∅` かつ `units(τ) ≠ ∅`
      `τ` から `unpunched_field_types(・)` の対の第 2 成分を取る歩みについての帰納で示す。すなわち
      `unpunched_field_types(τ)` の各対の第 2 成分について結論が成り立つことを帰納法の仮定とする。
      L1b よりその第 2 成分も扱う型であり、DEF 扱う型 と A10 よりその歩みは有限なので、この帰納は
      整礎である。
  <2>1. `is_fully_unboxed` は、`is_box` / `is_closure` / `is_array` のいずれかで偽を返し、`is_funptr` で
        真を返し、それ以外では `unpunched_field_types` の全フィールドについての再帰の全称である。
    BY CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>2. `is_closure(τ)` が偽で、`is_box(τ)` または `is_array(τ)` であるとき、`go` は `path` を積み、
        `unit_step` は `Unit` を返して `rc_units_go` は `path` を積む。
    `is_fully_unboxed` は `is_box`、`is_closure`、`is_array` の順に見て、そのいずれかが真ならば偽を返す。
    `is_box(τ)` ならば最初の行で、`is_box(τ)` が偽で `is_array(τ)` ならば -- この場合の条件より
    `is_closure(τ)` は偽なので -- `is_array` の行で偽を返す。よって `is_fully_unboxed(τ)` は偽である。
    `go` と `unit_step` はどちらも `is_fully_unboxed`、`is_closure`、`is_box`、`is_array` の順に見るので、
    最初の 2 つを抜けて、`go` は `is_box` の腕か `is_array` の腕で `path` を積んで戻り、`unit_step` は
    `is_box || is_union || is_array || is_punched_array` の行で `Unit` を返す。`rc_units_go` は `Unit` の
    腕で `path` を積む。
    BY CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step,
       rc_units_go
  <2>3. `is_closure(τ)` のとき、`go` は `path ++ [CLOSURE_CAPTURE_IDX]` を積み、`unit_step` は
        `Capture { capture_idx: CLOSURE_CAPTURE_IDX, .. }` を返して `rc_units_go` は
        `path ++ [CLOSURE_CAPTURE_IDX]` を積む。
    `is_fully_unboxed` は `is_closure` の行で偽を返すので、`go` と `unit_step` はどちらも 1 つ目の検査を
    抜けて `is_closure` の腕に入る。
    BY CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>4. ASSUME (V) `is_box(τ)`、`is_array(τ)`、`is_closure(τ)`、`is_funptr(τ)` がどれも偽である
        PROVE  `leaves(τ) ≠ ∅` かつ `units(τ) ≠ ∅`
    <3>1. `unpunched_field_types(τ)` のある対 `(idx, fty)` について `is_fully_unboxed(fty)` は偽である。
      `<2>1` と (V) より、この場合 `is_fully_unboxed(τ)` は `unpunched_field_types(τ)` の全対についての
      全称であり、(U) よりそれは偽である。
      BY <2>1
    <3>2. `go` は `τ` の位置で `unpunched_field_types(τ)` の各対の下へ降りる。
      (U) より `is_fully_unboxed(τ)` は偽であり、(V) より `is_closure(τ)`・`is_box(τ)`・`is_array(τ)` は
      偽である。L1a の `go` の欄はこの 4 つがどれも偽のとき
      `unpunched_field_types(τ)` の各対の下へ降りると述べる。
      BY <ref id=cd71109/>
    <3>3. `leaves(τ) ≠ ∅` である。
      `<3>1` の `fty` は `unpunched_field_types(τ)` の対の第 2 成分なので、`<3>1` と帰納法の仮定より
      `leaves(fty) ≠ ∅` である。`<3>2` より
      `go` は `path` に `idx` を積んで `fty` へ降りるので、`fty` から始めた走査が積む path の前に `idx` を
      置いたものが `leaves(τ)` に入る。
      BY <3>1, <3>2, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <3>4. `units(τ) ≠ ∅` である。
      `<3>2` と L1a (b) より `step(τ)` は `Unit` か `Fields` である。`Unit` のとき `rc_units_go` は
      `path` を積むので `units(τ) ≠ ∅` である。`Fields { held_fields, .. }` のとき L1a (a) より
      `held_fields = unpunched_field_types(τ)` であり `rc_units_go` は対 `(idx, fty)` の下へ降りるので、
      `<3>1` と帰納法の仮定 `units(fty) ≠ ∅` から `units(τ) ≠ ∅` が出る。
      BY <3>1, <3>2, <ref id=cd71109/>, CODE src/rc_ir/ownership.rs: rc_units_go
    <3>5. QED
      BY <3>3, <3>4
  <2>5. QED
    まず `is_closure(τ)` の真偽で場を分ける。真のときは `<2>3` が当てはまる。偽のときは残る 3 つの述語で
    分ける -- `is_box(τ) ∨ is_array(τ)` ならば `<2>2` が当てはまり、どちらも偽で `is_funptr(τ)` ならば
    `<2>1` より `is_fully_unboxed(τ)` が真になって (U) に反し、4 つがどれも偽ならば `<2>4` の ASSUME が
    満たされてその結論が出る。この分け方は場を尽くし、各場は他と重ならない。`leaves(τ)` は `path` を空列として
    始めた `go` が積んだものの全体、`units(τ)` は同じく `rc_units_go` が積んだものの全体なので、`<2>2` と
    `<2>3` の場合はどちらも 1 つ以上積まれて空でない。
    BY <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/rc_ir/ownership.rs: rc_units

<1>3. QED
  BY <1>1, <1>2

### L9 (`units_under` の要素は unit へ切り詰まる) <!--#49b83ad-->

**言明**。`trunc(τ, ・)` が `under(τ, p)` の各要素について値を返すとき、その値は `units(τ)` の要素である。

<1>1. CASE `sub(τ, p) = Some(σ)`
  L2 より `under(τ, p) = { p ++ w : w ∈ units(σ) }` である。L6 より `trunc(σ, w) = w` なので、L5 より
  `trunc(τ, p ++ w) = p ++ trunc(σ, w) = p ++ w` である。L5 より `p ++ w ∈ units(τ)` である。
  BY <ref id=9f11796/>, <ref id=323bb77/>, <ref id=e74af85/>

<1>2. CASE `sub(τ, p) = None`
  L2 より `under(τ, p) = [p]` である。L1 の場合 (b) より、`trunc(τ, p)` が値を返すのは `Unit` の行か
  `Capture` (`p[m] = capture_idx`) の行であり、値はそれぞれ `p[0..m]`、`p[0..m+1]` である。
  <2>1. `rc_units_go` は `path = p[0..m]` で `cur_m` に達する。
    `i < m` の各位置で `unit_step(cur_i)` は `Fields { held_fields, .. }` であって `p[i]` は
    `held_fields` に含まれる (DEF 歩み)。L1a (a) より `rc_units_go` はその各段で同じ対の下へ降りる。
    BY <ref id=cd71109/>, DEF 歩み, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>1a. `Unit` の行では `p[0..m] ∈ units(τ)` である。
    `<2>1` より `rc_units_go` は `path = p[0..m]` で `cur_m` に達し、`unit_step(cur_m) = Unit` なので
    `path` を積む。
    BY <2>1, <ref id=cd71109/>
  <2>2. `Capture` の行では `p[0..m+1] ∈ units(τ)` である。
    `<2>1` より `rc_units_go` は `path = p[0..m]` で `cur_m` に達し、
    `unit_step(cur_m) = Capture { capture_idx, .. }` なので `path ++ [capture_idx]` を積む。
    `p[m] = capture_idx` なのでこれは `p[0..m+1]` である。
    BY <2>1, <ref id=cd71109/>
  <2>3. QED
    BY <ref id=fd9b709/>, <2>1a, <2>2

<1>3. QED
  `sub(τ, p)` は `Some` か `None` を返すか中断する。中断するとき `under(τ, p)` も中断する (L2) ので
  言明の仮定が成り立たない。
  BY <1>1, <1>2, <ref id=9f11796/>

### L9a (leaf に沿う歩みの終わり方) <!--#24f7933-->

**言明**。`λ ∈ leaves(τ)` とし、`m_λ` を `τ` の `λ` に沿う歩みの長さ、`cur_0, ..., cur_{m_λ}` をその型の
列とする。このとき `m_λ ≤ |λ|` であり、`i < m_λ` の各位置で `step(cur_i)` は `Fields { held_fields, .. }`
であって `held_fields` は添字 `λ[i]` の対を含み、さらに次のどちらかが成り立つ。

- **(A)** `step(cur_{m_λ}) = Unit`。
- **(B)** `m_λ < |λ|` であり、`step(cur_{m_λ}) = Capture { capture_idx, .. }` かつ
  `λ[m_λ] = capture_idx` である。

すなわち `NoUnit` で止まることも、`Fields` であって次の添字を held に持たないために止まることもない。

<1>1. `go` が `λ` を積むのは次の 2 つの場合だけである。(i) `path = λ` の位置で `is_box` または
      `is_array` が真であるとき。(ii) `λ = μ ++ [CLOSURE_CAPTURE_IDX]` であり、`path = μ` の位置で
      `is_closure` が真であるとき。
  L1a の表より、`go` が path を積むのは `Capture` の行 (`path ++ [CLOSURE_CAPTURE_IDX]` を積む) と、
  `Unit` の行のうち `is_box(σ) ∨ is_array(σ)` の側 (`path` を積む) だけである。
  BY <ref id=cd71109/>

<1>2. `go` が `λ` を積む位置に至るまでの各位置で `go` は降りている。その位置の型を順に
      `g_0 = τ, g_1, ...` と書くと、`g_{i+1}` は `unpunched_field_types(g_i)` の添字 `λ[i]` の対の第 2
      成分である。降りる位置は、場合 (i) では `i < |λ|` の各 `i`、場合 (ii) では `i < |μ|` の各 `i` で
      ある。
  `go` が `path` を伸ばすのは降りるときだけであり、積む位置の `path` は場合 (i) では `λ`、場合 (ii) では
  `μ` である。よってそこに至るまでの各位置で `go` はその path の次の添字の対の下へ降りている。L1a より、
  降りる先は `unpunched_field_types` の対である。
  BY <1>1, <ref id=cd71109/>, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. `i ≤ m_λ` について `cur_i = g_i` であり、`i < m_λ` の各位置で `step(cur_i)` は
      `Fields { held_fields, .. }` であって `held_fields` は添字 `λ[i]` の対を含む。また `m_λ ≤ |λ|` で
      ある。
  DEF 歩み は `i < m_λ` の各位置で `step(cur_i) = Fields { held_fields, .. }` かつ `held_fields` が添字
  `λ[i]` の対を含むことを要求し、`cur_{i+1} = held_field_type(held_fields, λ[i], _)` と定める。L1a (a) より
  `held_fields = unpunched_field_types(cur_i)` なので、これは `<1>2` の `g` の漸化式と同じである。基底は
  `cur_0 = g_0 = τ` である。`m_λ ≤ |λ|` は DEF 歩み が `i < |λ|` のときにしか次の型を定めないことによる。
  BY <1>2, <ref id=cd71109/>, DEF 歩み, CODE src/rc_ir/ownership.rs: held_field_type

<1>4. CASE `<1>1` の場合 (i)
  <2>1. `m_λ < |λ|` のとき `step(cur_{m_λ}) = Unit` である。
    `<1>3` より `cur_{m_λ} = g_{m_λ}` であり、`<1>2` より `g_{m_λ}` は `go` が降りる位置なので、L1a (b)
    より `step(g_{m_λ})` は `Fields` か `Unit` である。`Fields { held_fields, .. }` だとすると、L1a (a)
    より `held_fields = unpunched_field_types(g_{m_λ})` であり、`<1>2` よりそれは添字 `λ[m_λ]` の対を
    含むので、DEF 歩み は `cur_{m_λ+1}` を定める。これは `m_λ` が歩みの長さであることに反する。
    BY <1>2, <1>3, <ref id=cd71109/>, DEF 歩み
  <2>2. `m_λ = |λ|` のとき `step(cur_{m_λ}) = Unit` である。
    `<1>3` より `cur_{|λ|} = g_{|λ|}` である。場合 (i) の条件よりその位置で `is_box` か `is_array` が
    真である。`is_fully_unboxed` は `is_box`・`is_closure`・`is_array` の順に見て、そのいずれかが真ならば
    偽を返すので、`is_fully_unboxed(g_{|λ|})` は偽である。`is_closure(g_{|λ|})` も偽である -- `go` は
    `is_fully_unboxed`・`is_closure`・`is_box`・`is_array` の順に見て、この位置では `is_box` の腕か
    `is_array` の腕に入るので、その手前の `is_closure` の検査を抜けている (`<1>1`)。`unit_step` は同じ
    3 つを同じ順に見るので、`NoUnit` の行と `Capture` の行を抜けて
    `is_box || is_union || is_array || is_punched_array` の行に入り、`Unit` を返す。
    BY <1>1, <1>3, <ref id=cd71109/>, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step
  <2>3. QED
    `<1>3` より `m_λ ≤ |λ|` なので `<2>1` と `<2>2` が場を尽くし、どちらも (A) を与える。
    BY <1>3, <2>1, <2>2

<1>5. CASE `<1>1` の場合 (ii)
  <2>1. `m_λ < |μ|` のとき `step(cur_{m_λ}) = Unit` であり、(A) が成り立つ。
    `<1>3` より `cur_{m_λ} = g_{m_λ}` であり、`<1>2` より `g_{m_λ}` は `go` が降りる位置なので、L1a (b)
    より `step(g_{m_λ})` は `Fields` か `Unit` である。`Fields { held_fields, .. }` だとすると、L1a (a) と
    `<1>2` より `held_fields` は添字 `λ[m_λ]` の対を含むので DEF 歩み が `cur_{m_λ+1}` を定め、`m_λ` が
    歩みの長さであることに反する。
    BY <1>2, <1>3, <ref id=cd71109/>, DEF 歩み
  <2>1a. `step(g_{|μ|}) = Capture { capture_idx: CLOSURE_CAPTURE_IDX, field_count: CLOSURE_FIELD_COUNT }`
         である。
    場合 (ii) の条件より、`go` が `g_{|μ|}` に着いた位置で `is_closure` は真である。`is_fully_unboxed` は
    `is_box`・`is_closure`・`is_array` の順に見てそのいずれかが真ならば偽を返すので、
    `is_fully_unboxed(g_{|μ|})` は偽である。`unit_step` は `is_fully_unboxed` を先に見て `NoUnit` の行を
    抜け、続く `is_closure` の行でこの値を返す。
    BY <1>1, <1>2, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/rc_ir/ownership.rs: unit_step
  <2>2. `m_λ = |μ|` のとき (B) が成り立つ。
    `<1>3` より `cur_{|μ|} = g_{|μ|}` なので、`<2>1a` より
    `step(cur_{m_λ}) = Capture { capture_idx: CLOSURE_CAPTURE_IDX, .. }` である。
    `λ = μ ++ [CLOSURE_CAPTURE_IDX]` より `m_λ = |μ| = |λ| - 1 < |λ|` かつ
    `λ[m_λ] = CLOSURE_CAPTURE_IDX = capture_idx` である。
    BY <1>3, <2>1a
  <2>3. QED
    `<1>3` より `m_λ ≤ |λ| = |μ| + 1` である。`m_λ = |μ| + 1` だとすると DEF 歩み より
    `step(cur_{|μ|}) = Fields` だが、`<1>3` より `cur_{|μ|} = g_{|μ|}` であり、`<2>1a` より
    `step(g_{|μ|}) = Capture` である。よって `m_λ ≤ |μ|` であり、`<2>1` と `<2>2` が場を尽くす。
    BY <1>3, <2>1, <2>1a, <2>2, DEF 歩み

<1>6. QED
  `<1>1` の 2 つの場合を `<1>4` と `<1>5` が扱った。`i < m_λ` についての主張と `m_λ ≤ |λ|` は `<1>3` で
  ある。
  BY <1>1, <1>3, <1>4, <1>5

### L10 (leaf に届く path では歩みが中断しない) <!--#ef258a5-->

**言明**。`covered(τ, p) ≠ ∅` のとき、`trunc(τ, p)` と `sub(τ, p)` は値を返し、`under(τ, p)` の各要素
`unit` について `trunc(τ, unit)` も値を返す。

<1>1. `λ ∈ leaves(τ)` とし、`m_λ` を `τ` の `λ` に沿う歩みの長さとする。このとき `m_λ ≤ |λ|` であり、
      `i < m_λ` の各位置で `step(cur_i)` は `Fields { held_fields, .. }` であって `held_fields` は添字
      `λ[i]` の対を含み、さらに (A) `step(cur_{m_λ}) = Unit` であるか、(B) `m_λ < |λ|` かつ
      `step(cur_{m_λ}) = Capture { capture_idx, .. }` かつ `λ[m_λ] = capture_idx` であるかの
      どちらかである。
  BY <ref id=24f7933/>

<1>2. `p` が `λ` と比較可能 (`p ⊑ λ` または `λ ⊑ p`) のとき、`trunc(τ, p)` と `sub(τ, p)` は値を返す。
  <2>1. `k = min(m_λ, |p|)` と置くと、`i < k` の各位置で `p[i] = λ[i]` であり、`τ` の `p` に沿う歩みの
        最初の `k + 1` 個の型は `τ` の `λ` に沿う歩みのものと一致する。
    `i < k` のとき `i < m_λ ≤ |λ|` かつ `i < |p|` であり、`p` と `λ` は比較可能なので短い方が長い方の
    接頭辞であって `p[i] = λ[i]` である。`<1>1` より `i < m_λ` の各位置で `step(cur_i)` は `Fields` で
    あって `held_fields` は添字 `λ[i] = p[i]` の対を含むので、DEF 歩み は `p` についても同じ型を定める。
    BY <1>1, DEF 歩み
  <2>2. CASE `|p| ≤ m_λ`
    `<2>1` より `k = |p|` であり、`τ` の `p` に沿う歩みの長さは `|p|` である。L1 の場合 (a) より
    `sub(τ, p) = Some(cur_{|p|})` かつ `trunc(τ, p) = p` であり、どちらも値である。
    BY <2>1, <ref id=fd9b709/>
  <2>3. CASE `m_λ < |p|`
    `<2>1` より `k = m_λ` であり、`τ` の `p` に沿う歩みの `cur_{m_λ}` は `λ` のものと同じである。
    `<1>1` の (A) では `step(cur_{m_λ}) = Unit` なので、`τ` の `p` に沿う歩みの長さは `m_λ < |p|` で
    あり、L1 の場合 (b) の `Unit` の行より `sub(τ, p) = None`、`trunc(τ, p) = p[0..m_λ]` である。(B) では
    `step(cur_{m_λ}) = Capture { capture_idx, .. }` かつ `λ[m_λ] = capture_idx` であり、`m_λ < |p|` かつ
    `m_λ < |λ|` で `p` と `λ` は比較可能なので `p[m_λ] = λ[m_λ] = capture_idx` である。L1 の場合 (b) の
    `Capture` (`p[m] = capture_idx`) の行より `sub(τ, p) = None`、`trunc(τ, p) = p[0..m_λ+1]` である。
    どちらも値である。
    BY <1>1, <2>1, <ref id=fd9b709/>
  <2>4. QED
    `|p| ≤ m_λ` と `m_λ < |p|` が場を尽くす。
    BY <2>2, <2>3

<1>3. `λ ⊑ p` のとき、`trunc(τ, p)` と `sub(τ, p)` は値を返す。
  `λ ⊑ p` は「`p` が `λ` と比較可能」の一方の場合である。
  BY <1>2

<1>4. `under(τ, p)` の各要素について `trunc(τ, ・)` は値を返す。
  `<1>2` と `<1>3` より `sub(τ, p)` は値を返す。`None` のとき L2 より `under(τ, p) = [p]` であり、
  `trunc(τ, p)` は値を返す。`Some(σ)` のとき L2 より要素は `p ++ w` (`w ∈ units(σ)`) の形であり、L6 より
  `trunc(σ, w) = w` は値を返すので、L5 より `trunc(τ, p ++ w) = p ++ w` も値を返す。
  BY <1>2, <1>3, <ref id=9f11796/>, <ref id=323bb77/>, <ref id=e74af85/>

<1>5. QED
  `covered(τ, p)` の元 `λ` は `p ⊑ λ` か `λ ⊑ p` を満たす。
  BY <1>2, <1>3, <1>4, CODE src/rc_ir/borrow.rs: covered_leaves

### L11 (`covered_leaves` が空でなければ、それを所有すれば足りる) <!--#ebd5605-->

**言明**。`V` を任意の `VarTable`、`r` を `V.param_tys.get(r) = Some(τ)` である名前、`p` を path とし、
`OL` を `VarPath` の集合とする。`covered(τ, p) ⊆ { λ : (r, λ) ∈ OL }` であり、かつ
`covered(τ, p) ≠ ∅` または `under(τ, p) = []` であるとき、`owns_object_yet(V, type_env, r, p, OL)` は
真である。

**表を明示するのは、この命題を `vars` と `vars_f` の両方について読むからである。** `owns_object_yet` が
表から読むのは `param_tys.get(root)` の 1 か所だけなので (`<1>1`)、言明は `τ` を通してしか表に依らない。

<1>0. `under(τ, p)` と、その各要素についての `trunc(τ, ・)` は中断しない。
  `covered(τ, p) ≠ ∅` のときは L10 による。`under(τ, p) = []` のときは、L2 より `sub(τ, p)` は値を
  返しており `under(τ, p)` も中断せず、要素が無いのでその各要素についての主張は空に成り立つ。
  BY <ref id=9f11796/>, <ref id=ef258a5/>

<1>0a. `leaves(τ)` の計算と、その各 `leaf` についての `trunc(τ, leaf)` は中断しない。
  `τ` は扱う型なので L1b より A10 を満たす。`leaves(τ)` を計算する `go` が呼ぶのは
  `is_fully_unboxed`・`is_closure`・`is_box`・`is_array` と `unpunched_field_types` であり、
  `unpunched_field_types` と `is_fully_unboxed` は最上位 tycon の宣言を `type_env` から引く。A10 より、
  `τ` から `unpunched_field_types` を繰り返し取る歩みは有限である。A10 よりその各段の型は ground で
  飽和していて tycon が `type_env` にあるので、`declared_field_types` の
  `assert_eq!(args.len(), tycon_info.tyvars.len())` と `toplevel_tycon_info` の
  `tycons().get(&tycon).unwrap()` は通る。`instance_field_types` が行う newtype の展開が abort せず
  停止することは、A10 の最後の節が別に与える -- A10 は、その面が ground・飽和・`type_env` の 3 つからは
  出ないと述べる。よってこの降下は中断せずに終わる。この降下が通る `toplevel_tycon_info` の
  `assert!(!self.is_closure())` も通る -- `unpunched_field_types` はその関数を呼び、`is_fully_unboxed` は
  `is_box` を経て `is_unbox` を呼び、`is_unbox` は `is_closure()` を先に見て短絡するので、closure 型では
  `toplevel_tycon_info` に届かない。closure でない型では表明の条件がそのまま成り立つ。`go` は
  `unpunched_field_types` を呼ぶ前に `is_closure` を見るので、`go` がその関数を呼ぶ型も closure ではない。
  `is_fully_unboxed` はさらに `is_funptr` を呼び、`is_funptr` は `is_funptr_tycon` の
  `parse::<u32>().unwrap()` で中断しうる。`is_funptr` が問われるのは、`τ` と、`τ` から
  `unpunched_field_types` の対の第 2 成分を繰り返し取って到達する型である -- `go` はその操作で降り、
  `is_fully_unboxed` も自分の再帰で同じ操作で降りるので、`go` が止まる型の下も範囲に入る。
  DEF 扱う型 はその操作について閉じており `τ` は扱う型なので、その全部が扱う型であり、L1d より
  どれについても `is_funptr` は値を返す。
  `leaf ∈ leaves(τ)` については、L9a より `τ` の `leaf` に沿う
  歩みは (A) `Unit` で終わるか、(B) `Capture` で終わり、そこで `leaf` が選ぶ添字が `capture_idx` に
  等しいかのどちらかである。(A) で歩みの長さが `|leaf|` ならば L1 の場合 (a)、それより短ければ場合 (b) の
  `Unit` の行、(B) ならば場合 (b) の `Capture` (`π[m] = capture_idx`) の行が当てはまり、どれも `trunc` は
  値を返す。
  BY <ref id=8412761/>, <ref id=fd9b709/>, <ref id=fb62043/>, <ref id=33ee52f/>, <ref id=24f7933/>, DEF 扱う型, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_box, TypeNode::is_unbox,
     TypeNode::is_closure, TypeNode::is_funptr, TypeNode::toplevel_tycon_info,
     TypeNode::unpunched_field_types, TypeNode::instance_field_types, TypeNode::declared_field_types,
     CODE src/fixstd/builtin.rs: is_funptr_tycon

<1>1. `owns_object_yet(V, type_env, r, p, OL)` は、まず `leaves(τ)` を計算し、続けて `under(τ, p)` の
      各要素 `unit` について「`trunc(τ, unit)` を鍵 `key` とし、`leaves(τ)` のうち `trunc(τ, ・) = key` を
      満たし `(r, ・) ∈ OL` である leaf が存在する」を要求する。`owns_object_yet` が第 1 引数の表を読むのは
      `vars.param_tys.get(root)` の 1 か所だけであり、その値が `Some(ty)` のときこの `ty` が `τ` で
      ある。以後の計算は `ty`・`type_env`・`root`・`path`・`owned_leaves` だけを読む。
  BY CODE src/rc_ir/borrow.rs: owns_object_yet

<1>2. CASE `under(τ, p) = []`
  `<1>0a` より `leaves(τ)` の計算とその各 leaf についての `trunc` は中断しない。`<1>1` の全称は空なので
  真である。
  BY <1>0a, <1>1

<1>3. CASE `sub(τ, p) = Some(σ)` かつ `under(τ, p) ≠ []`
  <2>1. `under(τ, p)` の要素は `p ++ w` (`w ∈ units(σ)`) の形であり、`trunc(τ, p ++ w) = p ++ w` である。
    BY <ref id=9f11796/>, <ref id=323bb77/>, <ref id=e74af85/>
  <2>2. 各 `w ∈ units(σ)` について、`λ_w ∈ leaves(σ)` で `trunc(σ, λ_w) = w` であるものが取れる。
    `τ` は扱う型であり `sub(τ, p) = Some(σ)` なので、L1b より `σ` も扱う型であって A10 を満たし、
    P1 の定義域に入る。
    BY <ref id=3597669/>, <ref id=fb62043/>, DEF 扱う型
  <2>3. `p ++ λ_w ∈ covered(τ, p)` であり `trunc(τ, p ++ λ_w) = p ++ w` である。
    L5 より `p ++ λ_w ∈ leaves(τ)` であり、`p ⊑ p ++ λ_w` なので `covered` の条件を満たす。
    L5 と `<2>2` より `trunc(τ, p ++ λ_w) = p ++ trunc(σ, λ_w) = p ++ w` である。
    BY <ref id=323bb77/>, <2>2, CODE src/rc_ir/borrow.rs: covered_leaves
  <2>4. QED
    `<1>0a` より `leaves(τ)` の計算とその各 leaf についての `trunc` は中断しない。`<2>1` の各
    `unit = p ++ w` について、`<2>3` の `p ++ λ_w` が `<1>1` の要求する leaf である。仮定より
    `covered(τ, p) ⊆ { λ : (r, λ) ∈ OL }` なので `(r, p ++ λ_w) ∈ OL` である。
    BY <1>0a, <1>1, <2>1, <2>3

<1>4. CASE `sub(τ, p) = None`
  <2>1. `under(τ, p) = [p]` であり、`key = trunc(τ, p)` である。
    BY <ref id=9f11796/>, <1>1
  <2>2. `τ` の `p` に沿う歩みの長さを `m` とすると `m < |p|` であり、`trunc(τ, p)` が値を返すので
        `unit_step(cur_m)` は `Unit` か `Capture` (`p[m] = capture_idx`) であり、`key` はそれぞれ
        `p[0..m]`、`p[0..m+1]` である。
    `<1>0` と `<2>1` より `under(τ, p) = [p]` の唯一の要素についての `trunc(τ, p)` は中断しない。
    `sub(τ, p) = None` なので L1 の場合 (a) は当てはまらず、場合 (b) の 5 行のうち `trunc` が値を返す
    のは `Unit` の行と `Capture` (`p[m] = capture_idx`) の行だけである。
    BY <1>0, <2>1, <ref id=fd9b709/>
  <2>3. `covered(τ, p)` の要素 `λ` で `|λ| ≤ m` であるものについては、`unit_step(cur_m) = Unit` であり、
        `λ = p[0..m] = key` かつ `trunc(τ, λ) = key` である。
    `<2>2` より `m < |p|` なので `|λ| ≤ m < |p|` であり、`λ` と `p` が比較可能であることは `λ ⊑ p` を
    意味して、`i < |λ|` について `λ[i] = p[i]` である。DEF 歩み より `i < m` の各位置で
    `unit_step(cur_i)` は `Fields { held_fields, .. }` であって `held_fields` は添字 `p[i]` の対を含むので、
    `i < |λ| ≤ m` の範囲で `τ` の `λ` に沿う歩みも同じ型を辿り、その長さは `|λ|` である。L9a を `λ` に
    当てると、長さが `|λ|` なので (B) は起こらず、(A) すなわち `unit_step(cur_{|λ|}) = Unit` である。
    `|λ| < m` だとすると DEF 歩み より `unit_step(cur_{|λ|}) = Fields` になって矛盾するので `|λ| = m` で
    あり、`λ = p[0..m]` である。`<2>2` よりこの場合 `key = p[0..m]` なので `λ = key` である。`τ` の `λ` に
    沿う歩みの長さは `|λ|` なので、L1 の場合 (a) より `trunc(τ, λ) = λ = key` である。
    BY <2>2, <ref id=fd9b709/>, <ref id=24f7933/>, DEF 歩み, CODE src/rc_ir/borrow.rs: covered_leaves
  <2>4. `covered(τ, p)` の各要素 `λ` について `trunc(τ, λ) = key` である。
    `|λ| ≤ m` のときは `<2>3` による。`|λ| > m` のとき、`λ` は `p` と比較可能なので `λ[0..m] = p[0..m]`
    であり、`λ[m] = p[m]` である (`|λ| > m` かつ `|p| > m` で、短い方が長い方の接頭辞)。DEF 歩み より
    `i < m` の各位置で `unit_step(cur_i)` は `Fields` であって `held_fields` は添字 `p[i] = λ[i]` の対を
    含むので、`τ` の `λ` に沿う歩みも同じ型を辿り、`unit_step(cur_m)` は `<2>2` と同じものであるから、
    その長さは `m` であって `m < |λ|` である。L1 の場合 (b) より `trunc(τ, λ)` は `Unit` の行なら
    `λ[0..m] = p[0..m]`、`Capture` の行なら `λ[0..m+1] = p[0..m+1]` であり、`<2>2` よりどちらも `key` に
    等しい。
    BY <2>2, <2>3, <ref id=fd9b709/>, DEF 歩み
  <2>5. QED
    `<1>0a` より `leaves(τ)` の計算とその各 leaf についての `trunc` は中断しない。`<2>1` より
    `under(τ, p) = [p]` は空でないので、言明の仮定の 2 つの選言肢のうち成り立つのは
    `covered(τ, p) ≠ ∅` の側である。よって `<2>4` の `λ` が 1 つ取れて `trunc(τ, λ) = key` であり、
    仮定より `(r, λ) ∈ OL` である。これが `<1>1` が `unit = p` について要求するものである。
    BY <1>0a, <1>1, <2>1, <2>4

<1>5. QED
  `<1>0` より `under(τ, p)` は中断せず、`<1>0a` より `leaves(τ)` とその各 leaf についての `trunc` も
  中断しないので、`owns_object_yet` の評価は値を返す。L2 より `sub(τ, p)` が中断すれば `under(τ, p)` も
  中断するので、
  `sub(τ, p)` は `Some` を返すか `None` を返すかのどちらかである。`under(τ, p) = []` の場合を `<1>2` が、
  `under(τ, p) ≠ []` の場合を `sub` の答えで分けて `<1>3` と `<1>4` が扱った (`sub(τ, p) = None` のとき
  L2 より `under(τ, p) = [p]` であり、これは `[]` ではない)。
  BY <1>0, <1>0a, <1>2, <1>3, <1>4, <ref id=9f11796/>

## 3. P7e の証明

**言明** (README の P7e)。任意の root `r` と path `p` について、次の 2 つが成り立つ。

- **(a)** `r` がこの版のパラメータ・capture であるとき、`owns(r, p) = owns(r, trunc(ty(r), p))` である。
- **(b)** そうでないとき、`owns(r, p)` は `p` によらず真である。さらに `trunc(ty(r), p)` が値 `t` を
  返すならば `owns(r, t)` も真である。

(a) の等号は第 1 節の読み方で読む。(b) が `trunc(ty(r), p)` に条件を付けるのは
`truncate_to_unit` が中断しうるからである。冒頭がその
`(r, p)` を挙げる。この文書の中で P7e を読むのは L21 の `<1>1` の中と L22 の `<1>2` である。

<1>0. `pty(r) = Some(τ)` であることと `r` がこの版のパラメータ・capture であることは同値であり、
      そのとき `τ = ty(r)` である。
  L1e の (ii) と (iii) の同値と、そのときの等式である。
  BY <ref id=14909ed/>

<1>1. CASE `pty(r) = None` -- `<1>0` より言明の (b) の場合
  <2>1. `owns(r, q)` は、任意の `q` について真を返す。
    `owns_object` は `self.vars.param_tys.get(root)` で分岐し、`None` の腕で `true` を返す。この腕は
    `path` を読まない。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object の `None` の腕
  <2>2. QED
    `<2>1` を `q = p` に当てると `owns(r, p)` は真である。`trunc(ty(r), p)` が値 `t` を返すならば、
    `<2>1` を `q = t` に当てて `owns(r, t)` も真である。
    BY <2>1

<1>2. CASE `pty(r) = Some(τ)` -- `<1>0` より言明の (a) の場合
  <2>1. `τ = ty(r)` である。
    BY <1>0
  <2>2. `owns(r, q) = ` 「`under(τ, q)` のすべての要素 `u` について `(r, trunc(τ, u)) ∈ OU`」である。
    `owns_object` の `Some(root_ty)` の腕は、`units_under(root_ty, path, self.type_env)` の各要素 `unit`
    について `self.owned_units.contains(&(root.clone(), truncate_to_unit(root_ty, unit, self.type_env)))`
    を要求する。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object の `Some(root_ty)` の腕
  <2>3. CASE L1 の場合 (a) が `(τ, p)` に当てはまる
    L1 (a) より `trunc(τ, p) = p` であるから、右辺は `owns(r, p)`、すなわち左辺と同じ呼び出しである。
    BY <ref id=fd9b709/>
  <2>4. CASE L1 の場合 (b) の `Unit` の行または `Capture` (`p[m] = capture_idx`) の行が `(τ, p)` に
        当てはまる
    <3>1. `t = trunc(τ, p)` は値を返す。
      BY <ref id=fd9b709/>
    <3>2. `sub(τ, p) = None` であり、`under(τ, p) = [p]` である。
      BY <ref id=fd9b709/>, <ref id=9f11796/>
    <3>3. `owns(r, p) = ((r, t) ∈ OU)` である。
      `<3>2` より `under(τ, p)` の唯一の要素は `p` であり、`<2>2` よりその要素に掛けるのは
      `trunc(τ, p) = t` である。
      BY <2>2, <3>1, <3>2
    <3>4. `under(τ, t) = [t]` であり `trunc(τ, t) = t` である。
      BY <3>1, <ref id=3eb1174/>
    <3>5. `owns(r, t) = ((r, t) ∈ OU)` である。
      `<3>4` より `under(τ, t)` の唯一の要素は `t` であり、`<2>2` よりその要素に掛けるのは
      `trunc(τ, t) = t` である。
      BY <2>2, <3>4
    <3>6. QED
      `<3>3` と `<3>5` の右辺は同じ命題である。
      BY <3>3, <3>5
  <2>5. CASE L1 の場合 (b) の残りの 3 行 (`Fields` で `π[m]` が held でない行、`Capture` で
        `p[m] ≠ capture_idx` の行、`NoUnit` の行) が `(τ, p)` に当てはまる
    <3>1. `trunc(τ, p)` は中断する。よって右辺は中断する。
      BY <ref id=fd9b709/>
    <3>2. `owns(r, p)` は中断する。
      `Fields` の行では L1 より `sub(τ, p)` が中断し、L2 より `under(τ, p)` が中断するので、`<2>2` の
      左辺の評価が中断する。残る 2 行では L1 より `sub(τ, p) = None` なので L2 より
      `under(τ, p) = [p]` であり、`<2>2` はその要素に `trunc(τ, p)` を掛ける。これは `<3>1` より
      中断する。
      BY <2>2, <3>1, <ref id=fd9b709/>, <ref id=9f11796/>
    <3>3. QED
      両辺が中断する。
      BY <3>1, <3>2
  <2>6. QED
    L1 は `(τ, p)` について場合 (a) と場合 (b) の 5 行に場を尽くして分ける。`<2>3` が (a) を、`<2>4` が
    (b) の 2 行を、`<2>5` が (b) の残り 3 行を扱った。
    BY <2>3, <2>4, <2>5, <ref id=fd9b709/>

<1>3. QED
  `pty(r)` は `Option` なので `None` か `Some(τ)` のどちらかである。`<1>0` より、前者は `r` がこの版の
  パラメータ・capture でない場合、後者はそうである場合であり、この 2 つは言明の (b) と (a) の場合分け
  そのものである。`<1>1` が前者を、`<1>2` が後者を与えた。
  BY <1>0, <1>1, <1>2


## 4. `origin` の候補についての命題

**DEF 再帰で訪れる対**
1 つの本体と、その `VarTable::of` を固定する。**固定する表は第 1 節の `vars` に限らない** -- A6・A11・
A12 を満たす本体とその `VarTable::of` ならどれでもよく、この定義と、これを主語にする L11a・L12・L14 の
言明と証明は、そのどれについても同じ文言で読む。以下では固定した表を `vars` と書き、`origin` と
同じ規約で、別の表について読むときは `Reach(vars', x, π)` のように表を明示する。L14a は入力の関数
`func` とその `vars_f` について述べる。

対 `(x, π)` について、集合 `Reach(x, π)` を、`(x, π)` を含み次の規則で
閉じた最小の集合とする。`(y, ρ) ∈ Reach(x, π)` のとき、`origin_inner(vars, type_env, y, ρ)` が `origin` を
呼ぶ相手を `Reach(x, π)` に入れる。その相手は、`vars.bindings.get(y)` に応じて次のとおりである
(`CODE src/rc_ir/ownership.rs: origin_inner`, `origin_from_leaves_under`)。

| `y` の `Binding` | 呼ぶ相手 |
|---|---|
| 無し / `Param` / `Producer` | 無し |
| `Move(w)` | `(w, ρ)` |
| `Join(arm_results)` | 各アーム結果 `a` について `(a, ρ)` |
| `Field(c, idx)`、`c` が boxed | 無し |
| `Field(c, idx)`、`c` が unbox | `(c, [idx] ++ ρ)` |
| `Payload(s, None)` | `(s, ρ)` |
| `Payload(s, Some(t))`、`s` が unbox | `(s, [t] ++ ρ)` |
| `Payload(s, Some(t))`、`s` が boxed | 無し |
| `Llvm(gen, args, rty)`、`decl.leaf_origins_at(ρ)` が単一の `Arg(j, σ)` | `(args[j], σ)` |
| `Llvm(gen, args, rty)`、それ以外 | `origin_from_leaves_under` が集める各 `(j, w) ∈ operand_units` について `(args[j], w)` |

P2 より `origin(x, π)` は停止するので `Reach(x, π)` は有限である。

**表の `Llvm` の 2 行の分かれ目は鍵から決まる。** どちらの行かを決めるのは
`decl.leaf_origins_at(ρ)` であり、`decl` は `gen.result_prov(rty, arg_tys, type_env)` である。A3 より
`result_prov` は決定的である -- 同じ引数に対して常に同じ値を返す -- ので、この分岐は対 `(y, ρ)` と
固定した表から決まり、`Reach(x, π)` は 1 つの集合として定まる
(`CODE src/rc_ir/ownership.rs: origin_inner`, `as_arg_projection`)。

**第 1 節の `vars` が条件を満たすこと。** `vars` は `Pre(V)` の表であり (第 1 節)、`Pre(V)` について
A6・A11・A12 が述べる性質が成り立つことは L0 が与える。固定した版が借用版であるとき、`Pre(V)` は入力の
関数の本体の束縛変数を付け替えた複製であり (P9)、A6 と A11 の範囲は `borrow_ify` の入力である
(A12 に範囲の記述は無いが、この文書は出力版についてもこの 3 つを読む) ので、この 1 段が要る。

**`vars_f` について読む者。** 入力の関数 `func` は `borrow_ify` の入力の関数なので A6・A11・A12 を
満たし、`ty(・)` も `func` に現れる名前について A12 が定める。よって L11a・L12・L14 を
`vars_f = VarTable::of(func)` と `func.body` について読んでよく、そのとき `Reach` は
`Reach(vars_f, ・, ・)`、`cand` と `act` と `id` は `cand(vars_f, ・, ・)` などである。読む者は L15 の
`<1>6`、P7d の `<1>1a` と `<1>7` の `<2>2` である。L14a はこの読み方の下で立つ命題である。

### L11a (`Reach` の要素数は整礎な尺度である) <!--#7bdd3d4-->

**言明**。対 `(x, π)` を取り、`(y, ρ)` を DEF 再帰で訪れる対 の表が `(x, π)` から 1 歩で進む相手とする。
このとき `Reach(y, ρ) ⊊ Reach(x, π)` であり、したがって `|Reach(y, ρ)| < |Reach(x, π)|` である。

すなわち `|Reach(・)|` は自然数値の狭義に減る尺度であり、「表が進む各相手について結論が成り立つならば
`(x, π)` について成り立つ」を示せば、すべての対について結論が出る。以下ではこれを
**`Reach` についての帰納**と呼ぶ。

<1>1. `Reach(y, ρ) ⊆ Reach(x, π)` である。
  DEF 再帰で訪れる対 より `(y, ρ) ∈ Reach(x, π)` であり、`Reach(x, π)` は表の規則で閉じている。よって
  `Reach(x, π)` は `(y, ρ)` を含み表の規則で閉じた集合である。`Reach(y, ρ)` はそのような集合のうち最小の
  ものなので、`Reach(x, π)` に含まれる。
  BY DEF 再帰で訪れる対

<1>2. `(x, π) ∉ Reach(y, ρ)` である。
  <2>1. DEF 再帰で訪れる対 の表が相手を持つ行の変数は、この本体の束縛変数である。
    表の第 1 列は `vars.bindings.get(・)` の場合分けであり、相手を持つ行の第 1 列は `Move` / `Join` /
    unbox 容器の `Field` / `Payload` / `Llvm` の 2 行のいずれかである。すなわち相手を持つ行の変数の
    `vars.bindings` は `Move` / `Join` / `Field` / `Payload` / `Llvm` のいずれかである。
    `VarTable::of` が `bindings` に入れるのはパラメータ・capture についての `Binding::Param` だけで
    あり、残る 6 種を入れるのは `collect_bindings` である。`collect_bindings` が名前を入れるのは、本体の
    `Let` の束縛変数、`Destructure` のフィールド変数、`Match` のアームの payload 変数についてである。
    BY DEF 再帰で訪れる対, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings, origin_inner
  <2>2. QED
    `(x, π) ∈ Reach(y, ρ)` と仮定する。`Reach(y, ρ)` は `(y, ρ)` を含み表の規則で閉じた最小の集合なので、
    `(y, ρ)` から表の規則を 1 歩ずつ辿って `(x, π)` に至る有限の列がある。`(x, π)` から `(y, ρ)` への
    1 歩を継ぐと、`(x, π)` から `(x, π)` へ戻る長さ 1 以上の閉路になる。`VarTable` は `origins` が空の
    状態で作られ (`VarTable::empty`)、`origin` は memo `vars.origins` を見てから `origin_inner` を呼び、
    答えを再帰から**戻った後に**記録する。よって空の memo でこの閉路の上の対を問われた `origin` は、
    どの対の答えも記録されないまま閉路を無限に潜り、停止しない。`(x, π)` は表が相手を持つ行なので
    `<2>1` より `x` はこの本体の束縛変数であり、これは P2 に反する。A11 が言うのがこの停止性の根拠で
    ある。**固定した表が第 1 節の `vars` であって固定した版が借用版であるとき、A11 の性質は L0 が、
    P2 は P9 と合わせて渡す** -- A11 も P2 も範囲は `borrow_ify` の入力である。
    BY <2>1, <ref id=3905b4e/>, <ref id=9cef509/>, <ref id=0edb0ba/>, <ref id=63eadd9/>,
       CODE src/rc_ir/ownership.rs: origin, origin_inner, VarTable::empty

<1>3. QED
  `<1>1` と `<1>2` より `Reach(y, ρ) ⊊ Reach(x, π)` である。DEF 再帰で訪れる対 より `Reach(x, π)` は
  有限なので、その要素数は自然数であり、狭義の包含は要素数を狭義に減らす。
  BY <1>1, <1>2, DEF 再帰で訪れる対

### L12 (候補は訪れた対である) <!--#44a9669-->

**言明**。`act(x, π) ⊆ Reach(x, π)` である。とくに `cand(x, π) ⊆ Reach(x, π)` である。さらに、
`origin(x, π)` が値を返すとき `cand(x, π)` は空でない。

<1>1. `Origin` の値を作るのは、`here()` すなわち `Origin::Exactly((var, path))`、
      `origin_from_leaves_under` の `Origin::Exactly(here.clone())`、`Origin::of_candidates(S, id)`、
      および部分結果をそのまま返す 6 か所 (`Binding::Move` の腕、unbox 容器の `Binding::Field` の腕、
      `Binding::Payload` の catch-all の腕、`Binding::Payload` の unbox 変位の腕、単一 `Arg` の
      `Binding::Llvm` の腕が返す `origin(...)`、および `origin_from_leaves_under` が返す
      `first.clone()`) だけである。どの作り方に入るかは鍵から決まる -- `Binding::Llvm` の腕の分かれ目
      だけが `decl = gen.result_prov(rty, arg_tys, type_env)` を読み、A3 より `result_prov` は
      決定的だからである。
  BY <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under,
     as_arg_projection, Origin::of_candidates

<1>2. `Origin::Exactly(p)` について `act = cand = {p}` であり、`Origin::Join { identity, candidates }`
      について `cand = candidates`、`act = candidates ∪ {identity}` である。
  `candidates()` は `Exactly(p)` に `vec![p]`、`Join` に `candidates` を返す。`acted_on()` は
  `identity()` を先頭に、それと異なる `candidates()` の元を続けた列を返す。
  BY CODE src/rc_ir/ownership.rs: Origin::candidates, Origin::acted_on, Origin::identity

<1>3. `of_candidates(S, id)` の返す `Origin` について `act ⊆ S ∪ {id}` である。
  `|S| = 1` のとき `Exactly` を返し `act = S`。`|S| ≥ 2` のとき `Join { identity: id, candidates: S }` を
  返し `<1>2` より `act = S ∪ {id}`。`S` が空のときは `assert!` で中断する。
  BY <1>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates

<1>3a. `origin(x, π)` が値を返すとき `cand(x, π)` は空でない。
  `<1>1` より、`origin` が返す `Origin` は `here()` か `origin_from_leaves_under` の
  `Origin::Exactly(here.clone())` が作った `Exactly` であるか、`of_candidates(S, id)` が返したもので
  ある -- 部分結果をそのまま返す 6 か所が返すのも、その先の呼び出しが作ったこの 2 つのどちらかである。
  `Exactly(p)` の `candidates()` は 1 元の列 `[p]` であり、空でない (`<1>2`)。`of_candidates(S, id)` は
  `S` が空のとき `assert!` で中断するので、`origin` が値を返した以上 `S` は空でない。そのとき
  `of_candidates` は `|S| = 1` で唯一の元の `Exactly` を、`|S| ≥ 2` で `candidates` を `S` とする
  `Join` を返すので、どちらでも `candidates()` は空でない。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates

<1>4. `Reach(x, π)` の要素 `(y, ρ)` について、`origin(y, ρ)` の `act` は `Reach(x, π)` に含まれる。
      L11a の `Reach` についての帰納で示す。すなわち、DEF 再帰で訪れる対 の表が `(y, ρ)` から進む各相手
      `(y', ρ')` について同じ結論が成り立つことを帰納法の仮定とする。`Reach(x, π)` は表の規則で閉じて
      いるので、その相手も `Reach(x, π)` の要素であり、結論を述べる資格がある
      (DEF 再帰で訪れる対)。
  <2>1. `here()` と `origin_from_leaves_under` の `Origin::Exactly(here.clone())` が作る `Origin` の
        `act` は `{(y, ρ)}` であり、`(y, ρ) ∈ Reach(x, π)` である。
    `here` はどちらも `(var.clone(), path.to_vec()) = (y, ρ)` である。
    BY <1>2, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
  <2>2. 部分結果をそのまま返す腕では、返る `Origin` は `origin(y', ρ')` の値であり、
        `(y', ρ') ∈ Reach(x, π)` である (DEF 再帰で訪れる対)。帰納法の仮定よりその `act` は
        `Reach(x, π)` に含まれる。
    `origin_from_leaves_under` の `first.clone()` は `reached` の元であり、`reached` の各元は
    `origin(args[j], unit)` (`(args[j], unit) ∈ Reach(x, π)`) か `Origin::Exactly(here.clone())`
    (`<2>1`) である。
    BY <2>1, DEF 再帰で訪れる対, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
  <2>3. `of_candidates(S, (y, ρ))` を呼ぶ 2 か所で、`S` の元はすべて `Reach(x, π)` に属する。
    `Binding::Join` の腕では `S` は各アーム結果の `origin(a, ρ).acted_on()` の和であり、
    `(a, ρ) ∈ Reach(x, π)` なので帰納法の仮定による。`origin_from_leaves_under` では `S` は `reached` の
    各元の `acted_on()` の和であり、`<2>2` がその各元の `act` を `Reach(x, π)` の中に置く。
    BY <2>2, DEF 再帰で訪れる対, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
  <2>4. QED
    `<1>1` の作り方を `<2>1`、`<2>2`、`<2>3` が尽くす。`<2>3` の場合は `<1>3` より
    `act ⊆ S ∪ {(y, ρ)}` であり、どちらも `Reach(x, π)` に属する。帰納法の仮定を使ったのは `<2>2` と
    `<2>3` であり、どちらも表が `(y, ρ)` から進む相手についてのものなので、L11a よりこの帰納は整礎で
    ある。
    BY <1>1, <1>3, <2>1, <2>2, <2>3, <ref id=7bdd3d4/>

<1>5. QED
  `<1>4` を `(y, ρ) = (x, π)` に当てる。`cand ⊆ act` は `<1>2` による。言明の最後の節は `<1>3a` である。
  BY <1>2, <1>3a, <1>4

### L13 (`Binding::Param` を持たない名前は `param_tys` の鍵でない) <!--#c7416c2-->

**言明**。`vars.bindings.get(w)` が `None` であるか、`Some(Binding::Move(..))`、
`Some(Binding::Producer)`、`Some(Binding::Llvm(..))`、`Some(Binding::Field(..))`、
`Some(Binding::Payload(..))`、`Some(Binding::Join(..))` のいずれかであるとき、`pty(w)` は `None` で
ある。したがって `owns(w, σ)` と `yet(w, σ)` はどちらも、任意の `σ` について真である。

とくに `Origin::Join` の `identity` の変数はこの形である。`Origin::Join` を作るのは `of_candidates` だけで
あり、その第 2 引数を渡すのは `Binding::Join` の腕の `(var, path)` と、`Binding::Llvm` の腕が
`origin_from_leaves_under` へ渡す `here_identity = (var, path)` の 2 か所だからである。

<1>1. `vars.param_tys` の鍵は、この版のパラメータ・capture の名前ちょうどである。また
      `vars.bindings` はその各名前を鍵に持つ。
  `VarTable::of` と `VarTable::body_only` はどちらも `VarTable::empty()` から始め、その `param_tys` は
  空の写像である。`VarTable::of` は各パラメータ・capture について `bindings` に `Binding::Param` を、
  `param_tys` にその型を入れる。`collect_bindings` は `bindings` と `var_tys` と `closure_targets` にしか
  入れず、鍵を取り除かない。`VarTable::body_only` はパラメータ・capture の行を通らないので `param_tys` は
  空のままである。
  BY CODE src/rc_ir/ownership.rs: collect_bindings, VarTable::of, VarTable::body_only, VarTable::empty

<1>2. `<1>1` のパラメータ・capture の名前は、`collect_bindings` が記録する束縛名と異なる。
  `collect_bindings` が `bindings` に入れるのは、`Pre(V)` の `Let` の束縛変数、`Destructure` の
  フィールド変数、`Match` のアームの payload 変数である。固定した版で場合を分ける。`f_own` の版と
  グローバル初期化子の版では、パラメータ・capture も `Pre(V)` も入力のものそのものである --
  `borrow_ify` は `func.clone()` を写し、グローバルは `g.init` を写す -- ので、A6 がこれらの名前が
  互いに、またパラメータ・capture の名前とも異なることを直接与える。借用版では、パラメータ・capture と
  `Pre(V)` は入力の関数のそれの束縛変数を `rename` で一斉に付け替えたものであって、それ以外の違いを
  持たない (P9、`clone_func`)。
  `fresh_rename_function` は 1 つの `counter` を `&mut` で持ち回り、各束縛名について
  `assign_fresh_name` を 1 度だけ呼んで `name#b<counter>` を作るので、`rename` の像の名前は相異なる
  `counter` の値を持ち、互いに異なる。よって、入力で互いに異なるこれらの名前の像も互いに異なる。
  BY <ref id=33c54dc/>, <ref id=63eadd9/>, CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/borrow.rs: borrow_ify, clone_func,
     CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name

<1>3. QED
  言明が挙げる場合を 2 つに分ける。`vars.bindings.get(w)` が `None` のとき、`<1>1` の後半より `w` は
  この版のパラメータ・capture ではなく、`<1>1` の前半より `param_tys` の鍵でもない。
  `Some` の 6 つの `Binding` はいずれも `collect_bindings` が入れるものなので、そのとき `w` は
  `collect_bindings` が記録する束縛名であり、`<1>2` よりパラメータ・capture の名前と異なるので、
  `<1>1` の前半より `param_tys` の鍵ではない。どちらの場合も `pty(w)` は `None` である。
  `owns_object` は `param_tys.get(root)` が `None` のとき真を返し、`owns_object_yet` も同じ条件で
  真を返す。`Origin::Join` の `identity` については、
  `of_candidates` を呼ぶ 2 か所がどちらも `vars.bindings.get(var)` の `match` の `Binding::Join` /
  `Binding::Llvm` の腕の中にある。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet,
     CODE src/rc_ir/ownership.rs: Origin::of_candidates, origin_inner, origin_from_leaves_under

### L14 (訪れた対は leaf に届く) <!--#c7d11e5-->

**言明**。`u ∈ units(ty(v))` とする。`Reach(v, u)` の各要素 `(y, ρ)` について
`covered(ty(y), ρ) ≠ ∅` である。

<1>1. 基底: `covered(ty(v), u) ≠ ∅` である。
  L7 より `Λ_{ty(v)}(u) ≠ ∅` であり、その各元 `λ` は `u ⊑ λ` を満たすので `covered(ty(v), u)` の条件を
  満たす。
  BY <ref id=212f2e7/>, CODE src/rc_ir/borrow.rs: covered_leaves

<1>2. `Binding::Move(w)` と `Binding::Payload(s, None)` の辺は性質を保つ。
  どちらも path を変えず、A12 より move-bind の両辺の型は等しく、catch-all アームの payload と
  scrutinee の型も等しいので `ty(w) = ty(y)`、`ty(s) = ty(y)` である。
  BY <ref id=83d98e9/>, <ref id=9cef509/>

<1>3. `Binding::Join(arm_results)` の辺は性質を保つ。
  path を変えず、A12 よりアームの結果と `Match` の束縛変数の型は等しい。
  BY <ref id=83d98e9/>, <ref id=9cef509/>

<1>4. `Binding::Field(c, idx)` (`c` が unbox) の辺は性質を保つ。すなわち
      ASSUME `covered(ty(y), ρ) ≠ ∅`
      PROVE  `covered(ty(c), [idx] ++ ρ) ≠ ∅`
  <2>1. `λ ∈ covered(ty(y), ρ)` ならば、`[idx] ++ λ` と `[idx] ++ ρ` は一方が他方の接頭辞である。
    `covered_leaves(ty, path, type_env)` は `leaves(ty)` のうち `leaf.starts_with(path)` または
    `path.starts_with(leaf)` を満たすものを返すので、`ρ ⊑ λ` または `λ ⊑ ρ` である。前者なら
    `[idx] ++ ρ ⊑ [idx] ++ λ`、後者なら `[idx] ++ λ ⊑ [idx] ++ ρ` である。
    BY CODE src/rc_ir/borrow.rs: covered_leaves
  <2>2. `μ ∈ leaves(ty(y))` ならば `[idx] ++ μ ∈ leaves(ty(c))` である。
    A12 より `ty(c)` は構造体であり、この腕の条件より boxed ではない。`is_struct` は
    `toplevel_tycon_info` を読むので `is_closure(ty(c))` は偽であり、その `TyConInfo` の `variant` は
    `TyConVariant::Struct` であるから、L1d より `is_array(ty(c))` も `is_funptr(ty(c))` も偽である。
    `μ ∈ leaves(ty(y))` より `leaves(ty(y)) ≠ ∅` なので L8 より
    `is_fully_unboxed(ty(y))` は偽であり、A12 より `ty(y)` は `ty(c)` の第 `idx` フィールドの型であって、
    そのフィールドは `ty(c)` が持つフィールド (`unpunched_field_types` が返すもの) である。
    `is_fully_unboxed` は `is_box` / `is_closure` / `is_array` で偽を、`is_funptr` で真を返し、それ以外
    では `unpunched_field_types` の全対についての全称であり、`ty(c)` はこの 4 つの述語をどれも満たさない
    ので全称の腕に入る。よって `is_fully_unboxed(ty(c))` も偽である。
    よって `boxed_leaf_paths` の `go` は `ty(c)` について `unpunched_field_types` の下へ降り、第 `idx`
    フィールドについて `ty(y)` から始めた `go` の結果の前に `idx` を置いたものを積む。
    BY <ref id=83d98e9/>, <ref id=9cef509/>, <ref id=33ee52f/>, <ref id=1d99428/>, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_struct,
       TypeNode::toplevel_tycon_info,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. QED
    この step の ASSUME より `covered(ty(y), ρ)` の元 `λ` が取れる。`covered(ty(y), ρ) ⊆ leaves(ty(y))`
    なので `<2>2` を `μ = λ` に当てて `[idx] ++ λ ∈ leaves(ty(c))` であり、`<2>1` より `[idx] ++ λ` と
    `[idx] ++ ρ` は一方が他方の接頭辞である。この 2 つが `covered(ty(c), [idx] ++ ρ)` の条件なので、
    その集合は空でない。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: covered_leaves

<1>5. `Binding::Payload(s, Some(t))` (`s` が unbox) の辺は性質を保つ。すなわち
      ASSUME `covered(ty(y), ρ) ≠ ∅`
      PROVE  `covered(ty(s), [t] ++ ρ) ≠ ∅`
  <2>1. `λ ∈ covered(ty(y), ρ)` ならば、`[t] ++ λ` と `[t] ++ ρ` は一方が他方の接頭辞である。
    `covered_leaves(ty, path, type_env)` は `leaves(ty)` のうち `leaf.starts_with(path)` または
    `path.starts_with(leaf)` を満たすものを返すので、`ρ ⊑ λ` または `λ ⊑ ρ` である。前者なら
    `[t] ++ ρ ⊑ [t] ++ λ`、後者なら `[t] ++ λ ⊑ [t] ++ ρ` である。
    BY CODE src/rc_ir/borrow.rs: covered_leaves
  <2>2. `μ ∈ leaves(ty(y))` ならば `[t] ++ μ ∈ leaves(ty(s))` である。
    A12 より `ty(s)` は union であり、`Match` が名指す変位はその型が実際に持つものであって、`ty(y)` は
    その第 `t` 変位の型である。`is_union` は `toplevel_tycon_info` を読むので `is_closure(ty(s))` は偽で
    あり、その `TyConInfo` の `variant` は `TyConVariant::Union` であるから、L1d より `is_array(ty(s))` も
    `is_funptr(ty(s))` も偽である。
    この腕の条件より `ty(s)` は boxed ではない。`μ ∈ leaves(ty(y))` より `leaves(ty(y)) ≠ ∅` なので L8 より
    `is_fully_unboxed(ty(y))` は偽である。`is_fully_unboxed` は `is_box` / `is_closure` / `is_array` で
    偽を、`is_funptr` で真を返し、それ以外では `unpunched_field_types` の全対についての全称である。
    `ty(s)` はこの 4 つの述語をどれも満たさないので全称の腕に入り、`ty(y)` がその 1 つなので
    `is_fully_unboxed(ty(s))` も偽である。よって `boxed_leaf_paths` の
    `go` は `ty(s)` について `unpunched_field_types` の下へ降り、第 `t` 変位について `ty(y)` から始めた
    `go` の結果の前に `t` を置いたものを積む。
    BY <ref id=83d98e9/>, <ref id=9cef509/>, <ref id=33ee52f/>, <ref id=1d99428/>, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_union,
       TypeNode::toplevel_tycon_info,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. QED
    この step の ASSUME より `covered(ty(y), ρ)` の元 `λ` が取れる。`covered(ty(y), ρ) ⊆ leaves(ty(y))`
    なので `<2>2` を `μ = λ` に当てて `[t] ++ λ ∈ leaves(ty(s))` であり、`<2>1` より `[t] ++ λ` と
    `[t] ++ ρ` は一方が他方の接頭辞である。この 2 つが `covered(ty(s), [t] ++ ρ)` の条件なので、
    その集合は空でない。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: covered_leaves

<1>6. `Binding::Llvm` の単一 `Arg(j, σ)` の辺は性質を保つ。
  A3 より、単一の `Arg(j, σ)` の宣言は第 `j` オペランドの leaf `σ` を名指す。すなわち
  `σ ∈ leaves(ty(args[j]))` である。`σ ⊑ σ` なので `σ ∈ covered(ty(args[j]), σ)` である。
  BY <ref id=e11772a/>, CODE src/rc_ir/borrow.rs: covered_leaves

<1>7. `Binding::Llvm` の `origin_from_leaves_under` の辺は性質を保つ。
  `operand_units` の元は `(j, truncate_to_unit(&args[j].ty, leaf, type_env))` の形であり、`leaf` は
  `LeafOrigin::Arg(j, leaf)` の宣言が名指す第 `j` オペランドの leaf である (A3)。`trunc` の答えは引数の
  接頭辞なので `w := trunc(ty(args[j]), leaf) ⊑ leaf` であり、`leaf ∈ leaves(ty(args[j]))` なので
  `leaf ∈ covered(ty(args[j]), w)` である。
  BY <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: origin_from_leaves_under, truncate_to_unit,
     CODE src/rc_ir/borrow.rs: covered_leaves

<1>8. QED
  DEF 再帰で訪れる対 の表は辺を尽くす。呼ぶ相手が無い 3 行は新しい対を作らない。残る 7 行を `<1>2` から
  `<1>7` が扱った (`Move` と catch-all の `Payload` を `<1>2` が、`Join` を `<1>3` が、unbox 容器の
  `Field` を `<1>4` が、unbox union の変位アームの `Payload` を `<1>5` が、`Llvm` の 2 行を `<1>6` と
  `<1>7` が)。基底は `<1>1` である。`Reach(v, u)` は最小の閉じた集合なので、この帰納が全体を覆う。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, DEF 再帰で訪れる対

### L14a (訪れた対の変数は本体に現れる) <!--#2d18d2a-->

**言明**。`func` を入力の関数、`vars_f = VarTable::of(func)` とし、`x` を `func` に現れる名前
(第 1 節の DEF 現れる名前) とする。`Reach(vars_f, x, π)` の各要素 `(y, ρ)` について、`y` も `func` に
現れる名前である。

<1>1. `collect_bindings` が `Binding` に入れる `RcVar` は、いずれも `for_each_var` が訪れる変数である。
  `collect_bindings` が `Binding` に入れる `RcVar` は、`Binding::Move(y)` の `y` (`RcRhs::Var(y)` の
  オペランド)、`Binding::Llvm(_, args, _)` の各 `args` (`RcRhs::Llvm` のオペランド)、
  `Binding::Field(container, _)` の `container` (`RcExpr::Destructure` の容器)、
  `Binding::Payload(scrut, _)` の `scrut` (`RcRhs::Match` の scrutinee)、`Binding::Join(arm_results)` の
  各元 (`returned_var` が返す、各アーム本体の終端の `Ret` が名指す変数) である。`for_each_var` は
  `for_each_node` で本体の全節点 -- アーム本体の節点も含む -- を歩き、各節点について
  `for_each_var_of_node` と `for_each_var_of_rhs` を呼ぶ。この 2 つは `RcRhs::Var` のオペランド、
  `RcRhs::Llvm` の各オペランド、`Destructure` の容器、`RcRhs::Match` の scrutinee、`Ret` が名指す変数の
  いずれも訪れる。
  BY CODE src/rc_ir/ownership.rs: collect_bindings, returned_var,
     CODE src/rc_ir/ast.rs: for_each_node, for_each_node_inner, for_each_var, for_each_var_of_node,
     for_each_var_of_rhs

<1>2. QED
  L11a の `Reach` についての帰納 (DEF 再帰で訪れる対 より `vars_f` と `func.body` について読む)。基底は
  `(x, π)` であり、`x` は仮定より `func` に現れる。段では、
  DEF 再帰で訪れる対 の表が `(y, ρ)` から進む相手の変数は、`vars_f.bindings.get(y)` が持つ `RcVar`
  (`Move` の `w`、`Join` の各アーム結果、`Field` の容器、`Payload` の scrutinee、`Llvm` の `args[j]`) の
  名前であり、`<1>1` よりそれは `for_each_var` が訪れる変数の名前、すなわち DEF 現れる名前 が挙げる
  `func` に現れる名前である。
  BY <1>1, <ref id=7bdd3d4/>, DEF 再帰で訪れる対, DEF 現れる名前

### L15 (借用版は名前替えである) <!--#c3b2aa3-->

**言明**。`func` を入力の関数、`clone` をその借用版、`rename` を `clone_func` が返す写像とし、`ρ` を
`rename` を `rename` の鍵でない名前上の恒等写像で延ばしたものとする。`vars_f = VarTable::of(func)`、
`vars_c = VarTable::of(clone)` と置く。このとき次が成り立つ。

- `ρ` は `func` に現れる名前 (第 1 節の DEF 現れる名前) の上で単射であり、`func` に現れる名前のうち
  `rename` の鍵でないものは `rename` の像に入らない。`vars_c.param_tys` の鍵は `vars_f.param_tys` の
  鍵の `ρ` による像ちょうどであって、`vars_c.param_tys[ρ(p)] = vars_f.param_tys[p]` である。
- `func` に現れる各名前 `x` と任意の path `π` について
  `origin(vars_c, type_env, ρ(x), π) = ρ(origin(vars_f, type_env, x, π))` である。
  ここで `ρ` は `VarPath` に対しその変数だけを写す。

<1>1. `clone` の本体・パラメータ・capture は、`func` のそれの束縛変数を `rename` で一斉に付け替えたもの
      であり、それ以外の違いを持たない。
  BY <ref id=63eadd9/>, CODE src/rc_ir/borrow.rs: clone_func

<1>2. `rename` の鍵は `func` のパラメータ・capture の名前と、`func.body` の `Let` の束縛変数・
      `Destructure` のフィールド変数・`Match` のアームの payload 変数の名前ちょうどである。`ρ` は
      `func` に現れる名前の上で単射であり、`func` に現れる名前のうち `rename` の鍵でないものは
      `rename` の像に入らない。
  <2>1. `rename` の鍵は言明の第 1 の節が挙げる名前ちょうどであり、いずれも `func` に現れる名前である。
    `fresh_rename_function` は `params` と `cap` の各名前について `assign_fresh_name` を呼び、続けて
    `assign_fresh_names_to_binders` が本体を歩いて `Let` の束縛変数、`Match` のアームの payload 変数、
    `Destructure` のフィールド変数について同じことをする。`renaming` に鍵が入るのはこの 2 か所だけで
    ある。
    BY CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name,
       assign_fresh_names_to_binders, assign_fresh_names_to_binders_inner
  <2>2. `rename` の像の名前は互いに異なる。
    `assign_fresh_name` は `*counter += 1` を行ってから `name#b<counter>` を作る (`clone_func` が渡す
    `pass_tag` は `"b"`)。`fresh_rename_function` は 1 つの `counter` を `&mut` で持ち回り、`<2>1` の各
    鍵について `assign_fresh_name` を 1 度だけ呼ぶので、作られる名前はそれぞれ相異なる `counter` の値を
    持つ。`#` で区切った最後の断片はその `b<counter>` なので、値が異なれば名前も異なる。
    BY <2>1, CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name,
       CODE src/rc_ir/borrow.rs: clone_func
  <2>3. `func` に現れる名前は `rename` の像に入らない。
    `rename` の像の名前は `assign_fresh_name` が作るものである。`clone_func` が渡す `pass_tag` は `"b"`
    なので、作られる `FullName` の `name` は `format!("{}#b{}", name.name, counter)` であり、`#` で
    区切った最後の断片は `b` の後に 10 進数字だけが続く形である。A13 は、`borrow_ify` の入力に現れる
    すべての名前について最後の断片がその形ではないと述べる。`func` に現れる名前 (第 1 節の
    DEF 現れる名前) はその集合に含まれるので、`rename` の像のどの名前とも異なる。
    BY <ref id=cb35ab1/>, DEF 現れる名前, CODE src/rc_ir/rename.rs: assign_fresh_name,
       CODE src/rc_ir/borrow.rs: clone_func
  <2>4. QED
    鍵についての第 1 の節は `<2>1` である。単射性は次のとおりである。`func` に現れる 2 つの相異なる
    名前 `n1`、`n2` を取る。どちらも `rename` の鍵ならば `<2>2` より `ρ(n1) ≠ ρ(n2)` である。どちらも鍵でないならば `ρ(n1) = n1 ≠ n2 = ρ(n2)` である。`n1` だけが鍵
    ならば、`ρ(n1)` は `rename` の像の元であり、`ρ(n2) = n2` は `<2>3` より像に入らないので異なる。
    言明の最後の節 (`func` に現れる名前のうち鍵でないものは像に入らない) は `<2>3` の特別な場合である。
    BY <2>1, <2>2, <2>3

<1>3. `vars_c.param_tys` は `vars_f.param_tys` の `ρ` による像であり、型は変わらない。
  `VarTable::of` は `func.params` と `func.capture` から `param_tys` を作る。`fresh_rename_function` は
  各パラメータ・capture に `rename_var` を掛け、`rename_var` は名前だけを差し替えて型 `ty` を残す。
  BY <1>1, CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/rename.rs: fresh_rename_function,
     rename_var

<1>4. `vars_f.bindings` の鍵は `rename` の鍵ちょうどであり、`vars_c.bindings` の鍵はその `ρ` による像
      ちょうどである。さらに `vars_c.bindings[ρ(x)]` は `vars_f.bindings[x]` の変数を `ρ` で写したもので
      あり、`vars_c.var_tys[ρ(x)] = vars_f.var_tys[x]` である。
  `VarTable::of` は各パラメータ・capture について `bindings` に `Binding::Param` を入れ、続けて
  `collect_bindings` が本体の `Let` の束縛変数、`Destructure` のフィールド変数、`Match` のアームの
  payload 変数について `bindings` を入れる。これは `<1>2` が挙げる `rename` の鍵ちょうどである。
  `collect_bindings` が `Binding` に記録するのは、右辺に現れる変数 (`Move` の `y`、`Field` の
  容器、`Payload` の scrutinee、`Join` のアーム結果、`Llvm` のオペランド) である。`<1>1` より `clone` の
  パラメータ・capture と本体はこれらをすべて `ρ` で写したものであり、型は変わらない。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings,
     CODE src/rc_ir/rename.rs: rename_expr, rename_var

<1>5. `func` に現れる名前 `x` について `vars_f.bindings.get(x)` が
      `Some(Binding::Llvm(gen, args, rty))` であるとき、`vars_c.bindings.get(ρ(x))` は
      `Some(Binding::Llvm(gen', args', rty))` であって、`arg_tys` を各オペランドの型の列とすると
      `gen'.result_prov(rty, arg_tys, type_env) = gen.result_prov(rty, arg_tys, type_env)` である。
  <2>1. `gen'` は `gen` の複製であって、`free_vars_mut()` が返す `&mut FullName` の欄だけが
        `rename` で差し替わっている。`args'` は `args` の各 `RcVar` の名前を差し替えたもので型は
        変わらず、第 3 成分の型も変わらない。
    `rename_rhs` の `RcRhs::Llvm` の腕は `llvm_gen.clone()` を作り、`llvm_gen.free_vars_mut()` が
    返す各 slot について `renaming` が持つ名前へ差し替え、`args` の各要素に `rename_var` を掛ける。
    `rename_var` は名前だけを差し替えて型を残す。`collect_bindings` は `Binding::Llvm` の第 3 成分に
    `Let` の束縛変数の型 `x.ty` を入れ、`rename_expr` はその束縛変数にも `rename_var` を掛けるので、
    第 3 成分の型も変わらない。
    BY <1>1, CODE src/rc_ir/rename.rs: rename_rhs, rename_var, rename_expr,
       CODE src/ast/inline_llvm.rs: LLVMGen::free_vars_mut,
       CODE src/rc_ir/ownership.rs: collect_bindings
  <2>2. `result_prov` は `self` の `FullName` の欄を読まない。
    `LLVMGen::result_prov` の既定の本体は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)`
    であり、`self` の欄を 1 つも読まない。A3 より、これを override するのは 29 個であり、すべて
    `src/fixstd/builtin.rs` に在る。そのうち本体が `self` の欄を読むのは 6 つで、読む欄はどれも添字で
    ある -- `InlineLLVMStructGetBody` は `field_index()` (欄 `field_idx : usize`)、
    `InlineLLVMStructPunchBody` は `arg_leaf_path` を通して欄 `field_idx : usize`、
    `InlineLLVMStructPlugInBody` は欄 `field_idx : usize`、`InlineLLVMStructSetBody` は欄
    `field_idx : u32`、`InlineLLVMMakeUnionBody` と `InlineLLVMUnionAsBody` は `variant_index()`
    (欄 `field_idx : usize`) である。残る 23 個の本体は `self` の欄を読まない。
    BY <ref id=e11772a/>, CODE src/fixstd/builtin.rs: 全 result_prov の override,
       CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov,
       InlineLLVMStructPunchBody::result_prov, InlineLLVMStructPunchBody::arg_leaf_path,
       InlineLLVMStructPlugInBody::result_prov, InlineLLVMStructSetBody::result_prov,
       InlineLLVMMakeUnionBody::result_prov, InlineLLVMUnionAsBody::result_prov
  <2>3. QED
    `<1>4` より `vars_c.bindings[ρ(x)]` は `vars_f.bindings[x]` の変数を `ρ` で写したものなので、
    `Binding::Llvm` の腕であることは両側で同じである。`<2>1` より `gen'` が `gen` と違うのは
    `FullName` の欄だけであり、`<2>2` よりその欄は `result_prov` の答えに現れない。`<2>1` より残る
    引数 -- 結果の型と各オペランドの型と `type_env` -- も同じものである。
    `gen` と `gen'` は別のオブジェクトなので、ここまでで揃うのは引数と、`result_prov` が読む `self` の
    欄だけである。同じものから同じ答えが出ることを与えるのは A3 の決定性の節である --
    `result_prov` は同じ引数に対して常に同じ値を返す。よって
    `gen'.result_prov(rty, arg_tys, type_env) = gen.result_prov(rty, arg_tys, type_env)` である。
    BY <1>4, <2>1, <2>2, <ref id=e11772a/>, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov

<1>6. `func` に現れる名前 `x` と任意の path `π` について
      `origin(vars_c, type_env, ρ(x), π) = ρ(origin(vars_f, type_env, x, π))` である。
      DEF 再帰で訪れる対 より `Reach` を `vars_f` について読み、L11a の `Reach` についての帰納で示す。
      すなわち、DEF 再帰で訪れる対 の表が `(x, π)` から進む各相手について結論が成り立つことを帰納法の
      仮定とする。
  <2>1. この証明が比べる `VarPath` の変数はすべて `func` に現れる名前であり、`ρ` はその上で単射で
        ある。
    L12 より `act(vars_f, x, π) ⊆ Reach(vars_f, x, π)` であり、L11a より表が進む相手 `(y, ρ')` に
    ついて `Reach(vars_f, y, ρ') ⊆ Reach(vars_f, x, π)` なので、この帰納の中で作られる `VarPath` の
    変数はすべて `Reach(vars_f, x, π)` の要素の変数である。L14a よりそれらは `func` に現れる名前であり、
    `<1>2` より `ρ` はその上で単射である。
    BY <1>2, DEF 再帰で訪れる対, <ref id=7bdd3d4/>, <ref id=44a9669/>, <ref id=2d18d2a/>
  <2>2. `vars_f.bindings.get(x)` が `None` / `Some(Binding::Param)` / `Some(Binding::Producer)` /
        boxed 容器の `Some(Binding::Field(c, idx))` / boxed scrutinee の
        `Some(Binding::Payload(s, Some(t)))` のいずれかであるとき、両側の `origin_inner` は
        `here()` を返し、その値は `Exactly((x, π))` と `Exactly((ρ(x), π))` である。
    `<1>4` より `vars_c.bindings` の鍵は `vars_f.bindings` の鍵の `ρ` による像ちょうどであり、
    `<1>2` より `ρ` は `func` に現れる名前の上で単射で、鍵でない名前の像は像に入らない。よって
    `vars_f.bindings.get(x)` が `None` であることと `vars_c.bindings.get(ρ(x))` が `None` である
    ことは同値である。残る 4 つでは `<1>4` より両側の `Binding` は同じ構成子であり、`Field` と
    `Payload` が boxed かどうかを決める `container.ty` / `scrut.ty` は `ρ` で変わらないので
    (`rename_var` は型を残す)、`is_box` の答えも同じである。`here()` は `(var, path)` を
    `Exactly` に包むだけである。
    BY <1>2, <1>4, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/rename.rs: rename_var
  <2>3. 部分結果をそのまま返す 4 つの腕 -- `Binding::Move(w)`、catch-all の
        `Binding::Payload(s, None)`、unbox 容器の `Binding::Field(c, idx)`、unbox union の
        `Binding::Payload(s, Some(t))` -- では結論が出る。
    `<1>4` と `<2>2` より両側は同じ腕に入り、`origin_inner` はそれぞれ `origin(・, w, π)`、
    `origin(・, s, π)`、`origin(・, c, [idx] ++ π)`、`origin(・, s, [t] ++ π)` を返す。`<1>4` より
    借用版の側の変数は `ρ(w)` / `ρ(s)` / `ρ(c)` であり、path は同じである。これらは
    DEF 再帰で訪れる対 の表が `(x, π)` から進む相手なので、帰納法の仮定が当てはまる。
    BY <1>4, <2>2, DEF 再帰で訪れる対, CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. `Binding::Join(arm_results)` の腕では結論が出る。
    この腕は各アーム結果について `origin(・, a, π).acted_on()` の元を `Set<VarPath>` に入れ、
    `of_candidates(candidates, &(var, path))` を返す。`<1>4` より借用版のアーム結果は `ρ` の像で
    あり、帰納法の仮定より各 `origin` は `ρ` で写る。`acted_on()` は `identity()` と
    `candidates()` を並べるだけなので、その元も `ρ` で写る。`<2>1` より `ρ` はこれらの `VarPath` の
    変数の上で単射なので、集合の重複除去は両側で同じ元を潰し、`candidates.len()` も等しい。
    `of_candidates` は `len()` が 1 か 2 以上かで分岐し、前者は唯一の元を `Exactly` に、後者は
    `identity` と `candidates` を `Join` に包むだけである。
    BY <1>4, <2>1, DEF 再帰で訪れる対, CODE src/rc_ir/ownership.rs: origin_inner,
       Origin::of_candidates, Origin::acted_on
  <2>5. `Binding::Llvm(gen, args, rty)` の腕では結論が出る。
    `<1>5` より両側の `decl` は同じ値である。`leaf_origins_at(path)` と `as_arg_projection` は
    `decl` と `path` だけを読むので、両側は同じ腕に入る。`Some((j, p))` の腕では
    `origin(・, args[j], p)` を返し、`<1>4` より借用版の側の変数は `ρ(args[j])` なので帰納法の
    仮定が当てはまる。`None` の腕では `origin_from_leaves_under` に入り、その `operand_units` は
    `decl`・`args[j].ty`・`type_env` だけから決まるので両側で同じであり、`reached` の各元は
    帰納法の仮定より `ρ` で写る。`produced_here` が真のとき積まれる `Origin::Exactly(here)` の
    `here` は `(var, path)` であり、これも `ρ` で写る。`reached.iter().all(|o| o == first)` の判定は
    `Origin` の等号であり、`<2>1` より `ρ` はそこに現れる変数の上で単射なので両側で同じ答えになる。
    `of_candidates` については `<2>4` と同じである。
    BY <1>4, <1>5, <2>1, <2>4, DEF 再帰で訪れる対,
       CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under, as_arg_projection,
       Origin::of_candidates, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       Provenance::leaf_origins_under
  <2>6. QED
    `Binding` は 7 つの構成子を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、
    `Join`)、`vars_f.bindings.get(x)` はそれらか `None` である。`Field` は容器の boxed / unbox で、
    `Payload` は `tag` の有無と scrutinee の boxed / unbox で分かれる。`<2>2` から `<2>5` はこの
    分け方を尽くす。
    `<2>2` から `<2>5` が結ぶのは `origin_inner` の腕についての等式であり、その腕は `origin` を
    再帰的に呼び、帰納法の仮定も `origin` について立てている。等式の両辺を 1 つの値として読むには、
    鍵 `(y, ρ)` が等しい 2 つの `origin` の呼び出しが等しい値を返すことが要る。それを与えるのが
    P2a である -- `vars_f` と `type_env` を第 1・第 2 引数に固定した呼び出しについて、および
    `vars_c` と `type_env` を固定した呼び出しについて、鍵の等しい 2 つの返り値は等しい。P2a が
    要求するのは、その `vars` が A6 と A11 を満たす本体について `VarTable::of` (か `body_only`) が
    作った表であることである。`vars_f = VarTable::of(func)` は `borrow_ify` の入力の関数 `func` から
    作った表であり、A6 と A11 が直接この条件を与える。`vars_c = VarTable::of(clone)` は借用版 `clone`
    から作った表であり、L0 より借用版の `Pre(V)` も A6 と A11 を満たすので、こちらもこの条件に入る。
    したがって両辺の `origin` の答えは `vars.origins` が保持する memo の状態に依らない量であり、この帰納の
    各段が結ぶ等式は 1 つの値どうしの等式である。A15 と P2 より再帰は停止するので、この対応は
    全域である。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=3e6b0e0/>, <ref id=9cef509/>, <ref id=0edb0ba/>, <ref id=b1f6e13/>, <ref id=7bdd3d4/>, DEF 再帰で訪れる対,
       CODE src/rc_ir/ownership.rs: origin, origin_inner, Binding

<1>7. QED
  第 1 の項は `<1>2` と `<1>3` が、第 2 の項は `<1>6` が与える。
  BY <1>2, <1>3, <1>6

### L16 (借用版の `owns_object` は推論の `owns_object_yet` である) <!--#90c9817-->

**言明**。`OL` を `infer_ownership` の不動点の `owned_leaves` とし、`owned_units` を `borrow_ify` が組む
集合、`ctx` を `clone` (`func` の借用版) の `RewriteCtx` とする。このとき、`func` に現れる
(第 1 節の DEF 現れる名前) 任意の名前 `r` と任意の path `p` について、
`ctx.owns_object(ρ(r), p) = owns_object_yet(vars_f, type_env, r, p, OL)` である
(両辺は同時に中断する)。

<1>1. `pty_f(r) = None` のとき、両辺とも真である。
  L15 より `vars_c.param_tys` の鍵は `vars_f.param_tys` の鍵の `ρ` による像ちょうどである。`r` は
  `func` に現れる名前であり、`vars_f.param_tys` の鍵でもない。L15 より `ρ` は `func` に現れる名前の上で
  単射であり、`vars_f.param_tys` の鍵はどれも `func` に現れる名前 (パラメータ・capture の名前) なので、
  `ρ(r)` はそれらの像のどれとも異なる。よって `ρ(r)` は `vars_c.param_tys` の鍵でない。`owns_object` は
  `param_tys.get(root)` が `None` の腕で真を返し、`owns_object_yet` も `param_tys.get(root)` が `None` の
  ときに真を返す。
  BY <ref id=c3b2aa3/>, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet

<1>2. `pty_f(r) = Some(τ)` のとき、`ctx.vars.param_tys[ρ(r)] = τ` である。
  BY <ref id=c3b2aa3/>

<1>3. `owned_units` は、`(ρ(r), k)` を含むことと、「`leaves(τ)` のある `leaf` について
      `trunc(τ, leaf) = k` かつ `(r, leaf) ∈ OL`」が成り立つことが同値である。
  <2>1. `borrow_ify` は、借用版を持つ各 `func` の各パラメータ `p` の各 `leaf ∈ boxed_leaf_paths(p.ty)` に
        ついて、`owned_leaves.owns(p.name, leaf)` が真のとき `(rename[p.name], trunc(p.ty, leaf))` を
        `owned_units` に入れる。借用版を持つ関数は capture を持たないので、`r` は `func.params` の 1 つで
        ある。これが `ρ(r)` を第 1 成分とする唯一の挿入である。
    `owned_units` へのもう 1 つの挿入は `owned_units.extend(param_capture_units(func))` であり、その
    第 1 成分は入力の関数のパラメータ・capture の名前、すなわち入力プログラムの束縛名である。P9 の後半
    より `ρ(r)` はそのどれとも異なる。別の関数 `g` の借用版の挿入の第 1 成分は、`g` のパラメータの
    `g` 用の `rename` による像である。`borrow_ify` は `let mut rename_counter: u64 = 0;` を 1 つ置き、
    `prog.funcs` を渡る繰り返しの中で `clone_func(func, borrow_version, &mut rename_counter)` を呼ぶので、
    すべての借用版の名前替えは 1 つの `counter` を共有する。`assign_fresh_name` は `*counter += 1` を
    行ってから `name#b<counter>` を作るので、この 1 回の `borrow_ify` の実行が作る名前はどれも相異なる
    `counter` の値を持ち、`#` で区切った最後の断片 `b<counter>` が相異なる。よって `g` の借用版の
    パラメータ名は `ρ(r)` と異なる。
    BY <ref id=63eadd9/>, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func,
       CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name
  <2>2. QED
    `OwnedLeaves::owns(var, path)` は `(var, path) ∈ OL` である。
    BY <2>1, CODE src/rc_ir/borrow.rs: OwnedLeaves::owns

<1>3a. `pty_f(r) = Some(τ)` のとき、`leaves(τ)` の計算と、その各 `leaf` についての `trunc(τ, leaf)` は
       中断しない。
  `<1>2` より `τ` は固定した出力版のパラメータ・capture の型でもあるので、DEF 扱う型 の根の型であり、
  L1b より A10 を満たす。`leaves(τ)` を計算する `go` が呼ぶのは `is_fully_unboxed`・`is_closure`・
  `is_box`・`is_array` と `unpunched_field_types` であり、`unpunched_field_types` と `is_fully_unboxed` は
  最上位 tycon の宣言を `type_env` から引く。A10 より、`τ` から `unpunched_field_types` を繰り返し取る
  歩みは有限である。A10 よりその各段の型は ground で飽和していて tycon が `type_env` にあるので、
  `declared_field_types` の `assert_eq!(args.len(), tycon_info.tyvars.len())` と
  `toplevel_tycon_info` の `tycons().get(&tycon).unwrap()` は通る。`instance_field_types` が行う
  newtype の展開が abort せず停止することは、A10 の最後の節が別に与える -- A10 は、その面が
  ground・飽和・`type_env` の 3 つからは出ないと述べる。よってこの降下は中断せずに終わる。
  この降下が通る `toplevel_tycon_info` の `assert!(!self.is_closure())` も通る --
  `unpunched_field_types` はその関数を呼び、`is_fully_unboxed` は `is_box` を経て `is_unbox` を呼び、
  `is_unbox` は `is_closure()` を先に見て短絡するので、closure 型では `toplevel_tycon_info` に届かない。
  closure でない型では表明の条件がそのまま成り立つ。`go` は `unpunched_field_types` を呼ぶ前に
  `is_closure` を見るので、`go` がその関数を呼ぶ型も closure ではない。
  `is_fully_unboxed` はさらに `is_funptr` を呼び、`is_funptr` は `is_funptr_tycon` の
  `parse::<u32>().unwrap()` で中断しうる。`is_funptr` が問われるのは、`τ` と、`τ` から
  `unpunched_field_types` の対の第 2 成分を繰り返し取って到達する型である -- `go` はその操作で降り、
  `is_fully_unboxed` も自分の再帰で同じ操作で降りるので、`go` が止まる型の下も範囲に入る。
  DEF 扱う型 はその操作について閉じており `τ` は扱う型なので、その全部が扱う型であり、L1d より
  どれについても `is_funptr` は値を返す。
  `leaf ∈ leaves(τ)` については、L9a より `τ` の `leaf` に沿う歩みは (A) `Unit` で終わるか、(B) `Capture`
  で終わって `leaf[m] = capture_idx` であるかのどちらかである。(A) で歩みの長さが `|leaf|` ならば L1 の
  場合 (a)、それより短ければ場合 (b) の `Unit` の行、(B) ならば場合 (b) の `Capture`
  (`π[m] = capture_idx`) の行が当てはまり、どれも `trunc` は値を返す。
  BY <1>2, <ref id=8412761/>, <ref id=fd9b709/>, <ref id=fb62043/>, <ref id=33ee52f/>, <ref id=24f7933/>, DEF 扱う型, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_box, TypeNode::is_unbox,
     TypeNode::is_closure, TypeNode::is_funptr, TypeNode::toplevel_tycon_info,
     TypeNode::unpunched_field_types, TypeNode::instance_field_types, TypeNode::declared_field_types,
     CODE src/fixstd/builtin.rs: is_funptr_tycon

<1>4. QED
  `pty_f(r)` は `Option` なので `None` か `Some(τ)` のどちらかである。`None` の場合は `<1>1` が両辺とも
  真であることを与える。`Some(τ)` の場合、`<1>2` より `owns_object(ρ(r), p)` は「`under(τ, p)` の各
  `unit` について `(ρ(r), trunc(τ, unit)) ∈ owned_units`」であり、`<1>3` よりこれは「各 `unit` について、
  `trunc(τ, leaf) = trunc(τ, unit)` かつ `(r, leaf) ∈ OL` である `leaf ∈ leaves(τ)` が在る」に等しい。
  これが `owns_object_yet(vars_f, type_env, r, p, OL)` の定義そのものである。
  中断も同時である。`owns_object` が評価するのは `under(τ, p)` と、その各要素についての `trunc(τ, ・)`
  と、`owned_units.contains` だけである。`owns_object_yet` はこれに加えて `leaves(τ)` と、その各 `leaf`
  についての `trunc(τ, leaf)` を評価するが、`<1>3a` よりその 2 つは中断しない。残る呼び出しは両辺で
  同じ引数のものである。
  BY <1>1, <1>2, <1>3, <1>3a, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet

## 5. P7d の証明

**言明** (README の P7d)。`infer_ownership` の不動点において、**P7a の意味の各 site** `(v, u)` に
ついて、`origin(v, u)` の候補は、すべて `owns_object` が真であるか、すべて偽であるかのどちらかである。

**読み方**。`owns_object` は `RewriteCtx` の method なので、この言明はある出力版の `RewriteCtx` について
読む。`borrow_ify` が作る `RewriteCtx` は 3 種である。入力の各関数の全所有版 `f_own`、借用版を持つ関数の
借用版、および各グローバル初期化子のものである (`CODE src/rc_ir/borrow.rs: borrow_ify`)。

**主語は `levelled_sites` が挙げる集合ではない。** README の P7d が述べるとおり、`levelled_sites` は
`&RcFunc` を取るので、グローバル初期化子の版については site を 1 つも挙げず、その版が `owns_unit` を
呼ぶ位置を主語から落とす。第 6 節の DEF site が P7a の意味の site を本体一般について定め、関数の版では
それが `levelled_sites` が挙げる集合と一致する (第 6 節)。以下では、`f_own` については
`levelled_sites(func)` の site を、借用版については `levelled_sites(clone)` の site を主語とし、
グローバル初期化子については「`origin(v, u)` の候補」を任意の `(v, u)` について読む。この 3 つで
P7a の意味の site の全部が覆われる。

<1>1. `levelled_sites(func)` の各 site `(v, u)` は `u ∈ units(ty(v))` を満たす。
  `levelled_sites` は 2 種の site を積む。`Retain(v, path)` / `Release(v, path)` の節点について
  `(v, path)`、および `Let(_, App(_, args), _)` の各引数 `arg` と各 `unit ∈ rc_units(arg.ty)` について
  `(arg, unit)`。後者は定義から `units(ty(arg))` の元である。前者は A2 より `path` が `ty(v)` の
  `rc_units` の元である。
  BY <ref id=8e3aff3/>, CODE src/rc_ir/borrow.rs: levelled_sites

<1>1a. `levelled_sites(func)` の各 site `(v, u)` について、`v.name` は `func` に現れる名前
       (第 1 節の DEF 現れる名前) であり、`cand(vars_f, v, u)` の各元 `(r, p)` の `r` も `func` に
       現れる名前である。
  `levelled_sites` は `for_each_node` で本体を歩き、`Retain`/`Release` の名指す変数と `RcRhs::App` の各
  引数を積む。`for_each_node` の本体は `grow_stack(|| for_each_node_inner(node, visit))` であり、A15 より
  `grow_stack` は閉包をちょうど 1 回呼ぶので、この歩きは `for_each_node_inner` の歩きである。
  `for_each_var` は同じ `for_each_node` の歩きの各節点について `for_each_var_of_node` と
  `for_each_var_of_rhs` を呼び、この 2 つは `Retain`/`Release` の名指す変数と `App` の各引数を訪れるので、
  `v.name` は `func` に現れる名前である。DEF 再帰で訪れる対 より L12 と L14a を `vars_f` について読んで
  よい。L12 より `cand(vars_f, v, u) ⊆ Reach(vars_f, v.name, u)` であり、L14a より
  `Reach(vars_f, v.name, u)` の各要素の変数は `func` に現れる名前である。
  BY DEF 再帰で訪れる対, <ref id=44a9669/>, <ref id=2d18d2a/>, <ref id=3e6b0e0/>,
     CODE src/rc_ir/borrow.rs: levelled_sites,
     CODE src/rc_ir/ast.rs: for_each_node, for_each_node_inner, for_each_var, for_each_var_of_node,
     for_each_var_of_rhs

<1>2. グローバル初期化子の `RewriteCtx` では、`owns_object` は任意の `(r, p)` について真を返す。
  その `RewriteCtx` の `vars` は `VarTable::body_only` で作られ、L1c よりその `param_tys` は空である。
  よって `owns_object` は `param_tys.get(root)` が `None` の腕に入り、`path` を読まずに真を返す。
  BY <ref id=fa1a6ce/>, CODE src/rc_ir/borrow.rs: borrow_ify のグローバルを写す繰り返し,
     CODE src/rc_ir/ownership.rs: VarTable::body_only, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>3. `f_own` の `RewriteCtx` では、`levelled_sites(func)` の各 site `(v, u)` の各候補 `(r, p)` について
      `owns_object(r, p)` は値を返し、その値は真である。
  <2>1. `f_own` の `RewriteCtx` の `vars` は `VarTable::of(f_own)` であり、`f_own` は `func` の複製なので
        その `param_tys` の鍵は `func` のパラメータ・capture の名前である。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, borrow_ify, CODE src/rc_ir/ownership.rs: VarTable::of
  <2>2. `pty(r) = None` のとき、`owns_object(r, p)` は真を返す。
    `<1>2` と同じ腕である。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
  <2>3. `pty(r) = Some(τ)` のとき、`owns_object(r, p)` は中断しない。
    `<2>1` よりこの `RewriteCtx` の `vars` は `VarTable::of(f_own)` である。この `RewriteCtx` を作る
    時点の `f_own.body` は `f_own` の版の `Pre(V)` であり、L0 よりそれについて A6・A11・A12 が述べる
    性質が成り立つ。よって DEF 再帰で訪れる対 が固定する本体と表の条件が満たされ、この本体と
    `VarTable::of(f_own)` について L12 と L14 を読んでよい。
    `<1>1` より `u ∈ units(ty(v))` であり、L12 より `(r, p) ∈ Reach(v, u)`、L14 より
    `covered(ty(r), p) ≠ ∅` である。L1c より `τ = ty(r)` であり、`covered(τ, p) ≠ ∅` である。
    L10 より `under(τ, p)` もその各要素に
    ついての `trunc(τ, ・)` も中断しない。`owns_object` が呼ぶのはこの 2 つと `owned_units.contains` だけ
    である。
    BY <1>1, <2>1, DEF 再帰で訪れる対, <ref id=9cef509/>, <ref id=fa1a6ce/>, <ref id=ef258a5/>, <ref id=44a9669/>, <ref id=c7d11e5/>,
       CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, RewriteCtx::new, borrow_ify
  <2>4. `pty(r) = Some(τ)` のとき、`owns_object(r, p)` は真である。
    `owns_object` は `under(τ, p)` の各要素 `unit` について `(r, trunc(τ, unit)) ∈ owned_units` を
    要求する。L9 より `trunc(τ, unit) ∈ units(τ)` である。`borrow_ify` は入力の各関数について
    `owned_units.extend(param_capture_units(func))` を行い、`param_capture_units` は各パラメータ・capture
    `p` と各 `unit ∈ rc_units(p.ty)` について `(p.name, unit)` を並べる。`<2>1` の表は
    `VarTable::of(f_own)` なので L1c が使えて、`pty(r) = Some(τ)` であることは `r` が `f_own` の
    パラメータか capture であることと同値であり、そのとき `τ = ty(r)` である。`f_own` は `func` の複製
    なので `r` は `func` のパラメータか capture でもあり、A12 よりその `RcVar` の型 `p.ty` は
    `ty(r) = τ` である。よって `(r, trunc(τ, unit))` はこの集合に入る。
    BY <2>1, <2>3, <ref id=83d98e9/>, <ref id=fa1a6ce/>, <ref id=49b83ad/>,
       CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, borrow_ify, param_capture_units
  <2>5. QED
    BY <2>2, <2>3, <2>4

<1>3a. `f_own` の `RewriteCtx` とグローバル初期化子の `RewriteCtx` について、言明が成り立つ。
  `<1>2` と `<1>3` よりすべての候補について `owns_object` は真である。
  BY <1>2, <1>3

<1>4. 借用版の `RewriteCtx` について、`levelled_sites(clone)` の site は
      `{ (ρ(v), u) : (v, u) ∈ levelled_sites(func) }` であり、
      `cand(vars_c, ρ(v), u) = ρ(cand(vars_f, v, u))` である。
  `levelled_sites` は本体の節点を `for_each_node` で歩き、`Retain`/`Release` の変数と path、`App` の引数と
  その型の `rc_units` を積む。`<1>1a` よりその歩きは `for_each_node_inner` の歩きであり、それは節点の
  種類だけで降り方を決める。P9 より `clone` の本体は `func` の本体の束縛変数を `ρ` で
  付け替えたものであり、変数の型は変わらないので、積まれる対は `ρ` で写ったものちょうどである。候補の
  対応は L15 の第 2 の項による。その項は「`func` に現れる各名前 `x`」についての言明であり、site の
  `v.name` がそれを満たすことは `<1>1a` が与える。
  BY <1>1a, <ref id=63eadd9/>, <ref id=c3b2aa3/>, CODE src/rc_ir/borrow.rs: levelled_sites,
     CODE src/rc_ir/ast.rs: for_each_node, for_each_node_inner

<1>5. 不動点において、各関数の各 site について `level_ownership(vars_f, type_env, (v, u), OL)` は `false` を
      返す。
  `infer_ownership` は、1 周で `changed` が偽になるまで繰り返す。最後の周では、各 site について
  `changed |= level_ownership(...)` が `changed` を変えなかったので、`level_ownership` は `false` を
  返した。その周の間 `owned_leaves` は変わらないので、その周が読んだ `owned_leaves` は不動点の `OL` で
  ある。
  BY CODE src/rc_ir/borrow.rs: infer_ownership

<1>6. CASE `level_ownership` の `owns_a_candidate` が偽である site
  <2>1. `cand(vars_f, v, u)` の各元 `(r, p)` について `yet_f(r, p)` は偽である。
    `level_ownership` の `owns_a_candidate` は、候補についての `any` として
    `owns_object_yet(vars, type_env, root, path, owned_leaves)` を問う。`<1>5` の呼び出しが渡す表は
    `vars_f` であり、`owned_leaves` は不動点の `OL` なので、これは第 1 節の `yet_f` である。
    BY <1>5, CODE src/rc_ir/borrow.rs: level_ownership
  <2>2. QED
    `<1>1a` より各候補の `r` は `func` に現れる名前なので L16 が使えて
    `ctx.owns_object(ρ(r), p) = yet_f(r, p)` であり、`<1>4` より借用版の site の候補は
    `ρ(cand(vars_f, v, u))` である。よってすべての候補について `owns_object` は偽である。
    BY <2>1, <1>1a, <1>4, <ref id=90c9817/>

<1>7. CASE `level_ownership` の `owns_a_candidate` が真である site
  <2>1. `cand(vars_f, v, u)` の各元 `(r, p)` で `pty_f(r) = Some(τ)` であるものについて、
        `covered(τ, p) ⊆ { λ : (r, λ) ∈ OL }` である。
    `owns_a_candidate` が真のとき `level_ownership` は各候補について自分が渡された表の `param_tys` を
    引き、`Some(ty)` のものについて `covered_leaves(ty, path, type_env)` の各 `leaf` を
    `owned_leaves.insert` に掛け、その返り値で `changed` を立てる。`<1>5` よりその表は `vars_f` なので、
    引く値は `pty_f(r)` である。`<1>5` より `changed` は偽なので、どの `insert` も新しい元を
    加えなかった。すなわちすべての `leaf` はすでに `OL` に在った。
    BY <1>5, CODE src/rc_ir/borrow.rs: level_ownership
  <2>2. その各元について `covered(τ, p) ≠ ∅` である。
    DEF 再帰で訪れる対 より L12 と L14 を `vars_f` について読んでよい。`<1>1` より `u ∈ units(ty(v))` で
    あり、L12 より `(r, p) ∈ Reach(vars_f, v, u)` である。L14 より
    `covered(ty(r), p) ≠ ∅` である。`vars_f = VarTable::of(func)` であり `pty_f(r) = Some(τ)` なので、
    L1c より `τ = ty(r)` であり、`covered(τ, p) ≠ ∅` である。
    BY <1>1, DEF 再帰で訪れる対, <ref id=fa1a6ce/>, <ref id=44a9669/>, <ref id=c7d11e5/>
  <2>3. その各元について `yet_f(r, p)` は真である。
    L11 は任意の `VarTable` `V` について `owns_object_yet(V, type_env, r, p, OL)` を扱うので、
    `V = vars_f` に当てる。`pty_f(r) = Some(τ)` はその仮定 `V.param_tys.get(r) = Some(τ)` であり、
    残る 2 つの仮定を `<2>1` と `<2>2` が与える。
    BY <2>1, <2>2, <ref id=ebd5605/>
  <2>4. `cand(vars_f, v, u)` の各元 `(r, p)` で `pty_f(r) = None` であるものについても
        `yet_f(r, p)` は真である。
    `owns_object_yet` は `param_tys.get(root)` が `None` のとき真を返す。`yet_f` が引く表は
    `vars_f` なので、その `param_tys.get(r)` が `pty_f(r)` である。
    BY CODE src/rc_ir/borrow.rs: owns_object_yet
  <2>5. QED
    `<2>3` と `<2>4` よりすべての候補について `yet_f` は真である。`<1>1a` より各候補の `r` は `func` に
    現れる名前なので L16 が使えて、`<1>4` と合わせてすべての候補について `owns_object` は真である。
    BY <2>3, <2>4, <1>1a, <1>4, <ref id=90c9817/>

<1>8. QED
  `<1>3a` が `f_own` とグローバル初期化子を扱った。借用版については、`<1>5` より各 site で
  `level_ownership` は `false` を返し、その中で `owns_a_candidate` は真か偽かのどちらかである。
  `<1>6` が偽の場合に「すべて偽」を、`<1>7` が真の場合に「すべて真」を与える。
  BY <1>3a, <1>5, <1>6, <1>7

**この命題を検査するコード。** `develop_mode` のとき、`borrow_ify` は借用版ごとに
`RewriteCtx::check_ownership_is_levelled` を呼ぶ。これは `levelled_sites` の各 site について
`origin(...).candidates()` の `owns_object` を並べ、最初の答えとすべてが一致することを `assert!` する
(`CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled`, `borrow_ify`)。この表明は P7d の
言明そのものである。

## 6. P7a の証明

**設定**。第 1 節が固定する出力版を `V` とする。`owns_unit` と `owns_object` は `V` の `RewriteCtx` の
ものであり、`infer_ownership` の不動点の下で読む。

**DEF site**
版 `V` の **site** とは、`Pre(V)` (第 1 節) を `for_each_node` で歩いて集めた次の対である
(`CODE src/rc_ir/ast.rs: for_each_node`, `for_each_node_inner`)。

- `Retain(v, path, ..)` / `Release(v, path, ..)` の節点について、対 `(v, path)`。
- `Let(_, App(_, args), _)` の節点について、各引数 `arg` と各 `unit ∈ units(ty(arg))` の対 `(arg, unit)`。

`levelled_sites(func, type_env)` は `&RcFunc` を取り、この歩きを `func.body` について行ったものである
(`CODE src/rc_ir/borrow.rs: levelled_sites`)。上の DEF は同じ歩きを本体一般 -- グローバル初期化子の
`init` を含む -- について述べたものであり、関数の版では `levelled_sites` が挙げる集合と一致する。
`owns_unit` はグローバル初期化子の版でも呼ばれるので (L17)、主語をこの形で取る。

site `(v, u)` と `Λ(u) = Λ_{ty(v)}(u)` について、次の 3 つの節を考える。

1. `owns_unit(v, u)` が真である。
2. `Λ(u)` の**ある inhabited な** leaf `λ` の**すべての**候補 `(r, p)` について `owns_object(r, p)` が
   真である。
3. `Λ(u)` の**すべての inhabited な** leaf のすべての候補について `owns_object` が真である。

**読み方**。節 1 は静的である。節 2 と節 3 は inhabited (D16) を含むので、`Post(V)` の 1 回の活性化
(D21) と、その活性化が辿る実行路の上の位置に相対的である。**位置は、その活性化の上で `v` が値を得て
いる (D6) 任意の位置に取る。** 以下では 1 つの活性化とその 1 つの位置を固定し、そこで inhabited な
`Λ(u)` の leaf の集合を `Inh(v, u)` と書く。

**site の節点を訪れる位置に限れない。** site は `Pre(V)` から作られ、借用版の `rewrite_rc` は
`owns_unit` が偽の unit の `Retain`/`Release` 節点を落とす (P10) ので、その節点は `Post(V)` に無いことが
あり、活性化がそれを訪れるとは限らない。**位置がどれでもよいのは、値が束縛の後に変わらないからで
ある** (D6)。`Post(V)` に残った site の節点を訪れる位置はこの範囲に入る -- site の節点は `v` を使用
するので、活性化がそこを訪れるならば L20a よりその位置で `v` は値を得ている。

**この節が証明するもの**。L17 (`owns_unit` を呼ぶ位置は site を出ない)、**節 1 から節 3**、および
**節 2 から節 1** である。README の P7a は、節 1 から節 3 が「節点を残すのが安全である」を、節 2 から
節 1 の**対偶**が「節点を落とすのが安全である」を与えると書く。

**この節が証明しないもの**。節 3 から節 2 と、節 3 から節 1 である。`Inh(v, u)` が空のとき節 3 は空虚に
真になるので、この 2 つは偽である。R2 がその本体を挙げる。`Inh(v, u) ≠ ∅` を足せば 3 つは同値になる。

R1 は、節 2 と節 3 の inhabited の限定が要ることを示す記録である。限定を外すと節 2 から節 1 へ渡れない。

### L17 (`owns_unit` を呼ぶ位置は site を出ない) <!--#9f8089c-->

**言明**。ある出力版の `RewriteCtx` が `owns_unit(v, u)` を呼ぶとき、`(v, u)` はその版の site
(DEF site) である。その版がグローバル初期化子のものであれば、さらに `owns_unit(v, u)` は真を返す。

<1>1. `owns_unit` を呼ぶのは `any_owned_unit`、`routing_saves_retain`、`call_rc`、`rewrite_rc` の 4 か所で
      ある。
  `owns_unit` は `src/rc_ir/borrow.rs` の `impl RewriteCtx` の中で `fn` として宣言されており、
  `pub` も `pub(crate)` も付かない。またこのファイルは `mod` 宣言を 1 つも持たないので、下位モジュールを
  持たない。この 2 つと EXT Rust の可視性 より、その呼び出しはこのファイルの中に
  しかない。このファイルの中で識別子 `owns_unit` が現れるのは、この宣言と、`any_owned_unit` の
  `rc_units(&arg.ty, ..).iter().any(|unit| self.owns_unit(arg, unit))`、`routing_saves_retain` の
  `!(self.owns_unit(arg, unit) && ..)`、`call_rc` の `let arg_owned = self.owns_unit(arg, &unit);`、
  `rewrite_rc` の `.filter(|unit| self.owns_unit(v, unit))`、および 2 つの doc コメントである。
  BY EXT Rust の可視性,
     CODE src/rc_ir/borrow.rs: 識別子 owns_unit の全出現,
     CODE src/rc_ir/borrow.rs: 下位モジュールの宣言の全出現,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
     RewriteCtx::any_owned_unit, RewriteCtx::routing_saves_retain,
     RewriteCtx::call_rc, RewriteCtx::rewrite_rc

<1>2. `any_owned_unit(arg)` と `routing_saves_retain(_, args, _)` と `call_rc(_, args)` は、
      `rewrite_inner` の `Let(x, App(callee, args), k)` の腕からその `args` を渡されて呼ばれ、
      `owns_unit(arg, unit)` の `unit` は `rc_units(arg.ty, type_env)` を渡る。
  `rewrite_inner` のこの腕は `self.route(x, callee, args, k)` と `self.call_rc(&callee, args)` を呼ぶ。
  `route` は `self.routing_is_safe(x, args)` と `self.routing_saves_retain(borrow_version, args, k)` を
  呼び、`routing_is_safe` は `args.iter().any(|a| self.any_owned_unit(a))` を呼ぶ。`any_owned_unit`、
  `routing_saves_retain`、`call_rc` はいずれも `rc_units(&arg.ty, self.type_env)` を渡る `unit` について
  `owns_unit(arg, unit)` を呼ぶ。`call_rc` が受け取る `args` は `route` を通した後も同じ列である。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::route, RewriteCtx::routing_is_safe,
     RewriteCtx::routing_saves_retain, RewriteCtx::any_owned_unit, RewriteCtx::call_rc

<1>3. `rewrite_rc(v, path, ..)` は `rewrite_inner` の `Retain(v, path, ..)` / `Release(v, path, ..)` の腕から
      その節点の `(v, path)` を渡されて呼ばれ、`is_borrow_version` が真のときだけ
      `owns_unit(v, unit)` を `unit ∈ units_under(ty(v), path)` について呼ぶ。A2 と L6 より
      `units_under(ty(v), path) = [path]` なので、呼ばれるのは `owns_unit(v, path)` だけである。
  `rewrite_rc` は `!self.is_borrow_version` のとき節点をそのまま返して終わる。`is_borrow_version` が
  真である版は借用版であり (`borrow_ify` が `true` を渡すのは `clones` を写す繰り返しだけである)、その
  本体は入力の関数の本体の束縛変数を一斉に付け替えたものであって、それ以外の違いを持たない (P9)。すなわち
  `Retain`/`Release` 節点の `path` も、名指す変数の型も、入力の本体のものと同じである。A2 より入力の
  `Retain`/`Release` の `path` は `units(ty(v))` の元なので、借用版の節点についてもそうである。L6 より
  `under(ty(v), path) = [path]` である。
  BY <ref id=8e3aff3/>, <ref id=e74af85/>, <ref id=63eadd9/>, CODE src/rc_ir/borrow.rs: borrow_ify, RewriteCtx::rewrite_inner, RewriteCtx::rewrite_rc

<1>4. `<1>2` と `<1>3` の `(v, u)` は、`V` の site (DEF site) である。
  `rewrite_inner` は `Pre(V)` の木を継続とアーム本体へ降りて歩くので、`<1>2` と `<1>3` の呼び出しが
  起きる節点は `Pre(V)` の節点である。DEF site の歩き `for_each_node` の本体は
  `grow_stack(|| for_each_node_inner(node, visit))` であり、A15 より `grow_stack` は閉包をちょうど 1 回
  呼ぶ。`for_each_node_inner` は各節点で `visit` を呼んだ後、`Match` の各アーム本体と継続の両方へ降りる
  ので、この歩きは `Pre(V)` のすべての節点を訪れる。`<1>2` の `(arg, unit)` は
  `Let(_, App(_, args), _)` の節点の引数と
  `rc_units(arg.ty, type_env) = units(ty(arg))` の元の対、`<1>3` の `(v, path)` は
  `Retain(v, path, ..)` / `Release(v, path, ..)` 節点の変数と path であり、DEF site はその節点について
  ちょうどこの対を挙げる。関数の版ではこの集合は `levelled_sites` が挙げるものである。
  BY <1>2, <1>3, DEF site, <ref id=3e6b0e0/>,
     CODE src/rc_ir/borrow.rs: levelled_sites, RewriteCtx::rewrite_inner,
     CODE src/rc_ir/ast.rs: for_each_node, for_each_node_inner

<1>5. グローバル初期化子の版では `owns_unit(v, u)` は真を返す。
  その `RewriteCtx` は `is_borrow_version: false` で作られるので `<1>3` の呼び出しは起きない。`<1>2` の
  呼び出しについては、`owns_unit` はまず `origin(v, u).candidates()` を評価する。`v` は `Pre(V)` --
  入力のグローバル初期化子の `init` -- の `App` の引数なので、その名前は
  `vars.bindings` に束縛を持つ (節点が束縛する変数) か、持たない (D6 の第 3 の形) かのどちらかであり、
  どちらも P2 の範囲である。よって `origin(v, u)` は中断せずに答えを返す。続いて、`vars` が
  `VarTable::body_only` で作られ、L1c よりその `param_tys` が空なので、`owns_object` はどの `(r, p)` にも
  真を返し、`owns_unit` はその全称なので真である。
  BY <1>2, <ref id=596a46d/>, <ref id=fa1a6ce/>, <ref id=0edb0ba/>, CODE src/rc_ir/borrow.rs: borrow_ify のグローバルを写す繰り返し,
     CODE src/rc_ir/ownership.rs: VarTable::body_only, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object,
     RewriteCtx::owns_unit, RewriteCtx::rewrite_rc

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### L18 (unit を覆う対と、その下の leaf の写り方) <!--#af00c3a-->

**DEF unit を覆う対**
対 `(x, π)` が **unit を覆う**とは、`Λ_{ty(x)}(π) ≠ ∅` であり、かつ `Λ_{ty(x)}(π)` の各 leaf `λ` について
`trunc(ty(x), λ) = trunc(ty(x), π)` が成り立つことをいう。

**言明**。

- **(a)** `u ∈ units(ty(v))` のとき `(v, u)` は unit を覆う。
- **(b)** `(x, π)` が unit を覆うとき、DEF 再帰で訪れる対 の表が `(x, π)` から進む各相手も unit を覆う。
  さらに、`origin_from_leaves_under` が進む相手 `(args[j], w)` については `w ∈ units(ty(args[j]))` で
  あり、残る 4 種の相手については `Λ` が次のように写る。

  | 進む相手 | `Λ` の写り方 |
  |---|---|
  | `Move(w)` の `(w, π)`、`Join` の `(a, π)`、catch-all `Payload` の `(s, π)` | `Λ_{ty(w)}(π) = Λ_{ty(x)}(π)` |
  | unbox 容器の `Field(c, idx)` の `(c, [idx] ++ π)` | `Λ_{ty(c)}([idx] ++ π) = { [idx] ++ λ : λ ∈ Λ_{ty(x)}(π) }` |
  | unbox union の `Payload(s, Some(t))` の `(s, [t] ++ π)` | `Λ_{ty(s)}([t] ++ π) = { [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` |
  | 単一 `Arg(j, σ)` の `(args[j], σ)` | `Λ_{ty(args[j])}(σ) = { σ }` |

<1>1. (a) が成り立つ。
  L7 より `Λ_{ty(v)}(u) ≠ ∅` であり、その各 leaf `λ` について `trunc(ty(v), λ) = u` である。L6 より
  `trunc(ty(v), u) = u` である。
  BY <ref id=e74af85/>, <ref id=212f2e7/>

<1>2. `Move(w)`、`Join` のアーム結果 `a`、catch-all の `Payload` の scrutinee `s` については、A12 より
      その型は `ty(x)` に等しいので、`Λ` も `trunc` も変わらず、性質はそのまま移る。
  A12 は move-bind の両辺の型、アームの結果と `Match` の束縛変数の型、catch-all アームの payload と
  scrutinee の型が一致することを述べる。
  BY <ref id=83d98e9/>, <ref id=9cef509/>

<1>3. unbox 容器の `Field(c, idx)` について、`is_fully_unboxed(ty(c))`・`is_closure(ty(c))`・
      `is_box(ty(c))`・`is_array(ty(c))`・`is_union(ty(c))` はどれも偽であり、`ty(x)` は
      `unpunched_field_types(ty(c))` の添字 `idx` の対の第 2 成分である。したがって `step(ty(c))` は、
      `is_punched_array(ty(c))` が真ならば `Unit`、偽ならば `Fields { held_fields, .. }` であって
      `held_fields = unpunched_field_types(ty(c))` は添字 `idx` の対を含む。
  A12 より `ty(c)` は構造体であり、`Destructure` が名指すフィールドはその型が実際に持つ (punched でない)
  ものであって、`ty(x)` はその第 `idx` フィールドの型である。`is_struct` は `toplevel_tycon_info` を読む
  ので `is_closure(ty(c))` は偽である。その `TyConInfo` の `variant` は `TyConVariant::Struct` なので
  `is_union(ty(c))` は偽であり、L1d より `is_array(ty(c))` も
  `is_funptr(ty(c))` も偽である。この腕の条件より `ty(c)` は boxed ではない。`Λ_{ty(x)}(π) ≠ ∅` より
  `leaves(ty(x)) ≠ ∅` なので L8 より `is_fully_unboxed(ty(x))` は偽である。`is_fully_unboxed` は
  `is_box` / `is_closure` / `is_array` で偽を、`is_funptr` で真を返し、それ以外では
  `unpunched_field_types` の全対についての全称である。`ty(c)` はこの 4 つの述語をどれも満たさないので
  全称の腕に入り、`ty(x)` がその 1 つなので `is_fully_unboxed(ty(c))` も偽である。`unit_step` は上から順に
  `is_fully_unboxed`、`is_closure`、`is_box || is_union || is_array || is_punched_array` を見るので、
  残る分かれ目は `is_punched_array(ty(c))` だけである。**この場合は実際に起こりうる** --
  `Std::PunchedArray a` は `unbox struct { _arr : Array a, _idx : I64 }` と宣言されており、
  その最上位 tycon は `Std::PunchedArray` なので `is_punched_array` が真である。
  BY <ref id=83d98e9/>, <ref id=9cef509/>, <ref id=33ee52f/>, <ref id=1d99428/>, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_struct,
     TypeNode::is_union, TypeNode::is_punched_array, TypeNode::toplevel_tycon_info,
     TypeNode::toplevel_tycon_satisfies,
     CODE src/rc_ir/ownership.rs: unit_step, CODE src/fixstd/std.fix: PunchedArray,
     CODE src/fixstd/builtin.rs: is_punched_array_tycon, make_punched_array_tycon,
     CODE src/constants.rs: PUNCHED_ARRAY_NAME

<1>3a. unbox 容器の `Field(c, idx)` について、`leaves(ty(c))` のうち `[idx]` を前置に持つものは
       `{ [idx] ++ μ : μ ∈ leaves(ty(x)) }` であり、したがって
       `Λ_{ty(c)}([idx] ++ π) = { [idx] ++ λ : λ ∈ Λ_{ty(x)}(π) }` であって空でない。
  `<1>3` より `is_fully_unboxed(ty(c))`・`is_closure(ty(c))`・`is_box(ty(c))`・`is_array(ty(c))` は
  どれも偽である。L1a の表の `go` の欄より、`go` はそのとき `unpunched_field_types(ty(c))` の各対の下へ
  降りる (`step(ty(c))` が `Unit` の場合も、`is_box` と `is_array` が偽なので `go` は降りる側である)。`go` は
  第 `idx` 対について `ty(x)` から始めた `go` の結果の前に `idx` を置いたものを積み、他の対の下で積む
  path は第 1 添字が `idx` と異なる。`Λ_{ty(x)}(π) ≠ ∅` なので像も空でない。
  BY <1>3, <ref id=cd71109/>, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>4. unbox 容器の `Field(c, idx)` について、`(c, [idx] ++ π)` は unit を覆い、`Λ` は表のとおりに写る。
  <2>1. CASE `step(ty(c)) = Fields { held_fields, .. }`
    `<1>3` より `held_fields` は添字 `idx` の対を含み、その第 2 成分は `ty(x)` なので、`sub` の第 1 周は
    `Fields` の腕を通って `cur` を `ty(x)` にし、`[idx]` を使い切る。すなわち
    `sub(ty(c), [idx]) = Some(ty(x))` である。L5 より、任意の `ρ` について
    `trunc(ty(c), [idx] ++ ρ) = [idx] ++ trunc(ty(x), ρ)` である。これと `(x, π)` が unit を覆うことから
    `trunc(ty(c), [idx] ++ λ) = [idx] ++ trunc(ty(x), λ) = [idx] ++ trunc(ty(x), π)
    = trunc(ty(c), [idx] ++ π)` が出る。
    BY <1>3, <ref id=323bb77/>, CODE src/rc_ir/ownership.rs: subtree_type, held_field_type
  <2>2. CASE `step(ty(c)) = Unit`
    `truncate_to_unit(ty(c), q, type_env)` は、`q` が空でなければ第 1 周で `Unit` の腕に入って `break`
    し、`out` が空のまま `[]` を返す。`q` が空ならループに入らず `[]` を返す。よって
    `trunc(ty(c), ・)` はどの path についても `[]` を返し、とくに
    `trunc(ty(c), [idx] ++ λ) = [] = trunc(ty(c), [idx] ++ π)` である。
    BY <1>3, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>3. QED
    `<1>3` よりこの 2 つが場を尽くす。どちらでも `Λ_{ty(c)}([idx] ++ π)` の各 leaf `[idx] ++ λ`
    (`<1>3a`) は `[idx] ++ π` と同じ path へ切り詰まり、`<1>3a` よりその集合は空でなく表のとおりである。
    BY <1>3, <1>3a, <2>1, <2>2

<1>5. unbox union の `Payload(s, Some(t))` について、`(s, [t] ++ π)` は unit を覆い、`Λ` は表のとおりに
      写る。
  <2>1. `is_fully_unboxed(ty(s))`・`is_closure(ty(s))`・`is_box(ty(s))`・`is_array(ty(s))` はどれも偽で
        あり、`ty(x)` は `unpunched_field_types(ty(s))` の添字 `t` の対の第 2 成分である。したがって
        `step(ty(s)) = Unit` である。
    A12 より `ty(s)` は union であり、`Match` が名指す変位はその型が実際に持つ (punched でない) もので
    あって、`ty(x)` はその第 `t` 変位の payload の型である。`is_union` は `toplevel_tycon_info` を読むので
    `is_closure(ty(s))` は偽であり、その `TyConInfo` の `variant` は `TyConVariant::Union` であるから、
    L1d より `is_array(ty(s))` も `is_funptr(ty(s))` も偽である。この腕の条件より `ty(s)` は boxed では
    ない。`Λ_{ty(x)}(π) ≠ ∅` より `leaves(ty(x)) ≠ ∅` なので L8 より `is_fully_unboxed(ty(x))` は偽で
    ある。`is_fully_unboxed` は `is_box` / `is_closure` / `is_array` で偽を、`is_funptr` で真を返し、
    それ以外では `unpunched_field_types` の全対についての全称であり、`ty(s)` はこの 4 つの述語をどれも
    満たさないので全称の腕に入る。`ty(x)` がその 1 つなので `is_fully_unboxed(ty(s))` も偽である。よって
    `unit_step(ty(s))` は `is_fully_unboxed` の行と `is_closure` の行を抜け、`is_union` を含む行で
    `UnitStep::Unit` を返す。
    BY <ref id=83d98e9/>, <ref id=9cef509/>, <ref id=33ee52f/>, <ref id=1d99428/>, CODE src/rc_ir/ownership.rs: unit_step,
       CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_union,
       TypeNode::toplevel_tycon_info, TypeNode::unpunched_field_types
  <2>2. `leaves(ty(s))` のうち `[t]` を前置に持つものは `{ [t] ++ μ : μ ∈ leaves(ty(x)) }` であり、
        したがって `Λ_{ty(s)}([t] ++ π) = { [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` であって空でない。
    `<2>1` より `is_fully_unboxed(ty(s))`・`is_closure(ty(s))`・`is_box(ty(s))`・`is_array(ty(s))` は
    どれも偽である。L1a の表の `go` の欄より、`go` はそのとき `unpunched_field_types(ty(s))` の各対の
    下へ降りる。`go` は第 `t` 対について `ty(x)` から始めた `go` の結果の前に `t` を置いたものを積み、
    他の対の下で積む path は第 1 添字が `t` と異なる。よって `leaves(ty(s))` のうち `[t]` を前置に持つ
    ものはこの像ちょうどであり、そのうち `[t] ++ π` を前置に持つものは `λ ∈ Λ_{ty(x)}(π)` についての
    `[t] ++ λ` ちょうどである。`Λ_{ty(x)}(π) ≠ ∅` なので像も空でない。
    BY <2>1, <ref id=cd71109/>, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `trunc(ty(s), ・)` はどの path についても `[]` を返す。
    `<2>1` より `step(ty(s)) = Unit` である。`truncate_to_unit` は空でない path については第 1 周で
    `Unit` の腕に入って `break` し、`out` が空のまま `[]` を返す。空の path についてはループに入らずに
    `[]` を返す。
    BY <2>1, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>4. QED
    `<2>2` より `Λ_{ty(s)}([t] ++ π)` は空でなく、表のとおりである。その各 leaf `[t] ++ λ` について、
    `<2>3` より `trunc(ty(s), [t] ++ λ) = [] = trunc(ty(s), [t] ++ π)` なので、
    DEF unit を覆う対 の条件が成り立つ。
    BY <2>2, <2>3, DEF unit を覆う対

<1>6. 単一 `Arg(j, σ)` の `(args[j], σ)` は unit を覆い、`Λ_{ty(args[j])}(σ) = { σ }` である。
  A3 より `σ ∈ leaves(ty(args[j]))` である。`boxed_leaf_paths` の `go` は leaf を積んだ位置で戻るので、
  1 つの leaf が別の leaf の真の接頭辞になることはない。よって `σ` を前置に持つ leaf は `σ` だけであり、
  `Λ_{ty(args[j])}(σ) = { σ }` は空でない。DEF unit を覆う対 が要求する残りは、この集合の各 leaf `λ`
  について `trunc(ty(args[j]), λ) = trunc(ty(args[j]), σ)` であることだが、`λ = σ` しかないので両辺は
  同じ呼び出しであり、第 1 節の等式の読み方より成り立つ。
  BY <ref id=e11772a/>, DEF unit を覆う対, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>7. `origin_from_leaves_under` の `(args[j], w)` は unit を覆う。
  `ty(args[j])` は `Pre(V)` に現れる `RcVar` の型なので根の型であり、L1b より A10 を満たして
  P1 の定義域に入る。`w = truncate_to_unit(&args[j].ty, leaf, type_env)` であり、A3 より
  `leaf ∈ leaves(ty(args[j]))` なので、P1 より `w ∈ units(ty(args[j]))` である。L7 より
  `Λ_{ty(args[j])}(w) ≠ ∅` であり、その各 leaf `μ` について `trunc(ty(args[j]), μ) = w` である。L6 より
  `trunc(ty(args[j]), w) = w` なので、この 2 つは等しい。
  BY <ref id=e11772a/>, <ref id=3597669/>, <ref id=fb62043/>, <ref id=e74af85/>, <ref id=212f2e7/>, DEF 扱う型, CODE src/rc_ir/ownership.rs: origin_from_leaves_under

<1>8. QED
  `<1>1` が (a) を扱った。(b) については、`<1>2` から `<1>6` が表の 4 行を、`<1>7` が
  `origin_from_leaves_under` の相手の `w ∈ units(ty(args[j]))` を与え、この 5 種が
  DEF 再帰で訪れる対 の表の進む相手を尽くす (「呼ぶ相手」が無い 3 行は新しい対を作らない)。
  BY <1>1, <1>2, <1>3, <1>3a, <1>4, <1>5, <1>6, <1>7, DEF 再帰で訪れる対

### L18a (`Binding::Llvm` の第 3 成分は束縛変数の型である) <!--#6c73144-->

**言明**。`vars.bindings.get(x)` が `Some(Binding::Llvm(gen, args, rty))` であるとき `rty = ty(x)` で
ある。したがって `leaves(rty) = leaves(ty(x))` であり、任意の `π` について
`Λ_{rty}(π) = Λ_{ty(x)}(π)` である。

**この命題が要るのは、L19 と L20 が `rty` について語り、L21 と L22 がその答えを `Λ_{ty(x)}(π)` と
`Inh_x(π)` に読み替えて使うからである。**

<1>1. `collect_bindings` は `Let(x, Llvm(gen, args), k)` の腕で `bindings` に
      `Binding::Llvm(gen.clone(), args.clone(), x.ty.clone())` を入れる。すなわち第 3 成分はその `Let` の
      束縛変数 `x` の `RcVar` が持つ型である。`bindings` に `Binding::Llvm` を入れるのはこの腕だけで
      ある。
  BY CODE src/rc_ir/ownership.rs: collect_bindings

<1>2. QED
  A12 より同じ名前の `RcVar` が持つ型は一致するので、`<1>1` の `x.ty` は `x` が得る値の型 `ty(x)` で
  ある (第 1 節の `ty`)。`leaves(・)` と `Λ_{・}(π)` は型だけの関数なので、後半が従う。
  BY <1>1, <ref id=83d98e9/>, <ref id=9cef509/>

### L19 (`Llvm` が束縛する値の leaf の `origin`) <!--#18e0a91-->

**言明**。`vars.bindings.get(x)` が `Some(Binding::Llvm(gen, args, rty))` であるとし、`decl` を
`gen.result_prov(rty, arg_tys, type_env)`、`λ ∈ leaves(rty)`、`S_λ` を `decl.leaf_origins_at(λ)` が返す
集合とする。A3 より `|S_λ| ≤ 1` であり、次の 3 つが場を尽くす。

- `S_λ = ∅` のとき `origin(x, λ) = Exactly((x, λ))` である。
- `S_λ = {Fresh}` または `S_λ = {Unknown}` のとき `origin(x, λ) = Exactly((x, λ))` である。
- `S_λ = {Arg(j, σ)}` のとき `origin(x, λ) = origin(args[j], σ)` であり、`σ ∈ leaves(ty(args[j]))` である。

<1>1. `|S_λ| ≤ 1` であり、`S_λ` の元は `Arg` か `Fresh` か `Unknown` である。また `decl` は
      `(gen, rty, arg_tys, type_env)` から決まる 1 つの値であり、`S_λ` もそうである。
  A3 は、`result_prov` を override する 29 個の宣言が leaf に置く集合の要素数がすべて 0 か 1 であり、
  複数の元を宣言する op はこのコミットのプログラムに存在しないと述べる。`LeafOrigin` は `Arg`、`Fresh`、
  `Unknown` の 3 つの構成子を持つ。A3 はさらに `result_prov` が決定的である -- 同じ引数に対して常に
  同じ値を返す -- ことを述べるので、`decl` は呼び出しの時点によらない。よってこの命題が数え上げる
  `origin_inner` の腕の分かれ目も、鍵から決まる。
  BY <ref id=e11772a/>, CODE src/rc_ir/provenance.rs: LeafOrigin, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov

<1>2. `S_λ = {Arg(j, σ)}` のとき `origin(x, λ) = origin(args[j], σ)` であり
      `σ ∈ leaves(ty(args[j]))` である。
  `origin_inner` の `Binding::Llvm` の腕は `decl.leaf_origins_at(path).and_then(as_arg_projection)` で
  分岐する。`λ` は `rty` の leaf なので `leaf_origins_at(λ)` は `Some(S_λ)` であり、`as_arg_projection` は
  要素数 1 の集合の唯一の元が `Arg(j, p)` のとき `Some((j, p))` を返す。よって `Some((j, σ))` の腕に入り
  `origin(vars, type_env, &args[j].name, &σ)` を返す。A3 より単一の `Arg(j, σ)` は第 `j` オペランドの
  leaf `σ` を名指す。
  BY <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: origin_inner, as_arg_projection,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at

<1>3. `S_λ = ∅` または `S_λ = {Fresh}` または `S_λ = {Unknown}` のとき、`as_arg_projection` は `None` を
      返し、`origin_from_leaves_under(vars, type_env, &decl, args, λ, &(x, λ))` の値 (それが `None` の
      ときは `Exactly((x, λ))`) が答えになる。
  `as_arg_projection` は要素数が 1 でない集合に `None` を返し、要素数 1 でその元が `Fresh` か `Unknown` の
  ときも `None` を返す。`origin_inner` の `None` の腕は
  `origin_from_leaves_under(...).unwrap_or_else(here)` である。
  BY CODE src/rc_ir/ownership.rs: origin_inner, as_arg_projection

<1>4. `decl.leaf_origins_under(λ)` が与えるのは `S_λ` 1 つだけである。
  `leaf_origins_under(path)` は `LeafMap::leaves_under(path)` であり、`leaf_path.starts_with(path)` を
  満たす leaf の値を並べる。`boxed_leaf_paths` の `go` は leaf を積んだ位置で戻るので、1 つの leaf が別の
  leaf の真の接頭辞になることはない。よって `λ` を前置に持つ `rty` の leaf は `λ` だけである。
  BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
     CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under, boxed_leaf_paths

<1>5. `S_λ = ∅` のとき `origin(x, λ) = Exactly((x, λ))` である。
  `<1>4` より `origin_from_leaves_under` の最初の繰り返しは空集合 1 つだけを見るので、`operand_units` は
  空、`produced_here` は偽のままである。`reached` は空になり、`let first = reached.first()?;` が `None` を
  返して関数全体が `None` を返す。`<1>3` より答えは `here()` すなわち `Exactly((x, λ))` である。
  BY <1>3, <1>4, CODE src/rc_ir/ownership.rs: origin_from_leaves_under

<1>6. `S_λ = {Fresh}` または `S_λ = {Unknown}` のとき `origin(x, λ) = Exactly((x, λ))` である。
  `<1>3` よりこの 2 つの場合には `as_arg_projection` が `None` を返し、答えは
  `origin_from_leaves_under(vars, type_env, &decl, args, λ, &(x, λ))` の値である。`<1>4` より
  `origin_from_leaves_under` は `S_λ` 1 つだけを見る。`LeafOrigin::Fresh` と `LeafOrigin::Unknown` は
  どちらも `produced_here = true` にし、`operand_units` には何も入れない。よって
  `reached = [Origin::Exactly(here.clone())]` であり、`reached.iter().all(|o| o == first)` は真になって
  `first.clone() = Exactly((x, λ))` が返る。
  BY <1>3, <1>4, CODE src/rc_ir/ownership.rs: origin_from_leaves_under

<1>7. QED
  `<1>1` が場を尽くし、`<1>2`、`<1>5`、`<1>6` がそれぞれを与える。
  BY <1>1, <1>2, <1>5, <1>6

### L20 (`origin_from_leaves_under` が返す候補) <!--#cd28c3f-->

**言明**。`vars.bindings.get(x)` が `Some(Binding::Llvm(gen, args, rty))` であり、`π` について
`decl.leaf_origins_at(π).and_then(as_arg_projection)` が `None` であるとする。
`origin_from_leaves_under(vars, type_env, &decl, args, π, &(x, π))` が組む `operand_units` と
`produced_here` について、`reached` を関数が組む `Vec<Origin>` とする。このとき次が成り立つ。

- **(a)** `operand_units = { (j, trunc(ty(args[j]), σ)) : λ ∈ Λ_{rty}(π), Arg(j, σ) ∈ S_λ }` であり、
  `produced_here` は「`Λ_{rty}(π)` のある `λ` の `S_λ` が `Fresh` か `Unknown` を含む」と同値である。
  ここで `S_λ` は `decl.leaf_origins_at(λ)` である。
- **(b)** `reached` は `{ origin(args[j], w) : (j, w) ∈ operand_units }` の元を並べたものであり、
  `produced_here` のとき末尾に `Exactly((x, π))` が付く。
- **(c)** `reached` が空のとき `origin(x, π) = Exactly((x, π))` である。
- **(d)** `reached` が空でないとき、`reached` の各元 `o` について `cand(o) ⊆ cand(x, π)` である。
- **(e)** `produced_here` が真のとき `(x, π) ∈ cand(x, π)` である。

<1>1. (a) が成り立つ。
  `origin_from_leaves_under` の最初の繰り返しは `decl.leaf_origins_under(path)` を渡り、各 `sources` の
  各元について `Arg(j, leaf)` なら `(j, truncate_to_unit(&args[j].ty, leaf, type_env))` を
  `operand_units` に入れ、`Fresh` か `Unknown` なら `produced_here` を真にする。
  `leaf_origins_under(π)` は `π` を前置に持つ `rty` の leaf の値を並べるので、渡るのは
  `{ S_λ : λ ∈ Λ_{rty}(π) }` である。A3 より `result_prov` は決定的なので `decl` は
  `(gen, rty, arg_tys, type_env)` から決まる 1 つの値であり、この命題が数え上げる `origin_inner` と
  `origin_from_leaves_under` の腕の分かれ目も鍵から決まる。
  BY <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
     CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under,
     CODE src/ast/inline_llvm.rs: LLVMGen::result_prov

<1>2. (b) が成り立つ。
  関数は `operand_units` を `origin(vars, type_env, &args[j].name, &unit)` へ写して `reached` を作り、
  `produced_here` のとき `Origin::Exactly(here.clone())` を `push` する。`here` は呼び出し元が渡す
  `(x, π)` である。
  BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under, origin_inner

<1>3. (c) が成り立つ。
  `let first = reached.first()?;` は `reached` が空のとき `None` を返して関数を終える。`origin_inner` の
  `None` の腕は `.unwrap_or_else(here)` なので、答えは `Exactly((x, π))` である。
  BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under, origin_inner

<1>4. `reached` が空でないとき、`origin(x, π)` は次のどちらかである。`reached` の元がすべて `first` に
      等しければ `first`、そうでなければ `of_candidates(⋃_{o ∈ reached} act(o), (x, π))`。
  関数は `reached.iter().all(|reached_origin| reached_origin == first)` のとき `first.clone()` を返し、
  そうでないとき `reached` の各元の `acted_on()` を集めた集合と `here` で `of_candidates` を呼ぶ。
  BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under, Origin::acted_on

<1>5. (d) が成り立つ。
  `<1>4` の前者では、`reached` の各元は `first` に等しいので `cand(o) = cand(first) = cand(x, π)` である。
  後者では、`of_candidates(S, id)` は `|S| = 1` のとき `Exactly` を、`|S| ≥ 2` のとき
  `Join { identity: id, candidates: S }` を返すので、どちらでも `candidates()` は `S` そのものである。
  よって `cand(x, π) = ⋃_{o ∈ reached} act(o)` であり、`act(o) ⊇ cand(o)` である。
  BY <1>4, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates, Origin::acted_on

<1>6. (e) が成り立つ。
  `produced_here` が真のとき、`<1>2` より `Exactly((x, π)) ∈ reached` である。`<1>4` の前者では
  `reached` の元はすべて等しいので `origin(x, π) = Exactly((x, π))` であり、
  `cand(x, π) = {(x, π)}` である。後者では `act(Exactly((x, π))) = {(x, π)}` が `cand(x, π)` に入る。
  BY <1>2, <1>4, CODE src/rc_ir/ownership.rs: Origin::acted_on, Origin::candidates

<1>7. QED
  BY <1>1, <1>2, <1>3, <1>5, <1>6

### L20a (使用される変数はその位置までに値を得ている) <!--#57d5753-->

**言明**。`Post(V)` の 1 つの活性化 (D21) とその辿る実行路 `ρ` を固定し、`ρ` の上の位置 `n` を取る。
`n` の節点が **使用**する各変数 -- `Let(x, Var(y), k)` の `y`、`App` の callee と各引数、`Closure` の各
capture、`Llvm` の各オペランド、`Match` の scrutinee、`Destructure` の容器、`Retain` / `Release` /
`Eval` / `Ret` が名指す変数 -- は、`ρ` の上で `n` までに値を得ている (D6)。`n` が `Match` のアーム本体の
中の節点であるときも同じである。

**この節点が束縛する変数は入らない。** `Let` の束縛変数、`Destructure` のフィールド変数、`Match` の
アームの payload 変数は、その節点が値を与える側であり、上の列挙に無い。

**束縛を持たない名前も上の列挙に入る。** D6 は値を得る形を 3 つ挙げ、3 つ目を `vars.bindings` に束縛を
持たない名前 -- A13 と D6 より最上位の記号の名前 -- とし、その値をその記号の値と定める。その名前は
スロットを持たず、その位置は D6 の意味の記号の位置である。`Let(x, Var(g), k)` の `g` や、
`App` の callee がグローバル値であるときの callee がこれである。

**この命題が要るのは、A11 が答えないからである。** A11 は「変数の使用は、その位置でスコープに入って
いる束縛に解決する」までしか言わず、スコープに入っている束縛の変数がその時点までに値を得ていることは
言わない。それを与えるのは D2 のスコープの規則と D3 の実行路の進み方である。

<1>1. RC IR の束縛は 4 種であり、そのスコープは次のとおりである。パラメータと capture は本体の全体、
      `Let(x, rhs, k)` の `x` は `k` の部分木、`Destructure(c, fs, s, k)` の各フィールド変数は `k` の
      部分木、`Match` のアームの `payload` はそのアームの `body` の部分木である。
  BY <ref id=b3dfa37/>

<1>2. パラメータと capture は、活性化が始まった時点で値を得ている。
  D23 より、関数の本体の活性化は各パラメータと capture に 1 つずつの値を持つ。グローバル初期化子の
  `init` はパラメータも capture も持たない (D1)。
  BY <ref id=a502f3e/>, <ref id=ff5985d/>

<1>3. 残る 3 種の束縛について、活性化がそのスコープの中の位置に居るならば、その束縛変数は既に値を
      得ている。
  D3 の実行路の作り方より、`Let(x, rhs, k)` の節点からは `k` へ進み (`rhs` が `Match` のときは選んだ
  アーム本体の実行路を辿ってから `k` へ進む)、`Destructure(c, fs, s, k)` の節点からも `k` へ進む。
  すなわち `k` の部分木の位置に至る実行路はその節点を通る。D2 の節点の表より、`Let` は `rhs` の値を
  `x` に、`Destructure` は各 `(i, x)` の `x` に第 `i` フィールドを束縛するので、その節点を通った後は
  束縛変数が値を得ている。`Match` のアームの `payload` については、D3 よりアーム本体の実行路は
  `Match` の節点から始まり、コード生成はアーム本体を評価する前に payload を `scope_push` するので、
  アーム本体の各位置ではその変数が値を得ている。D9 の移動の表の payload 束縛の 2 行と D10 の生成の表の
  boxed union の payload の行が、その変数が受け取る値を述べる。
  BY <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=9d74736/>, <ref id=f06144e/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match

<1>3a. `n` の節点が使用する変数のうち `vars.bindings` に束縛を持たないものは、`n` において値を得て
       いる。
  D6 は値を得る形を 3 つ挙げ、3 つ目を `vars.bindings` に束縛を持たない名前とし、その値をその記号の値と
  定める。A13 と D6 よりそれは最上位の記号の名前である。関数の名前については、D6 より記号の位置は
  funptr を指し、初期化の段を持たないので、実行の初めから値がある。グローバル値の名前については、D6 は
  記号の位置が値を持つ時点をその記号のグローバル化の段 (D24 の (E5)) より後に限る。`n` はその名前を
  読む節点なので、**D6 の「それでも `g` を読む節点は必ず値を読む」がこの位置に当たる** -- まだ初期化
  されていなければ、その節点の段が先に (E7) と (E5) を走らせるからである。
  BY <ref id=cb35ab1/>, <ref id=596a46d/>, <ref id=e3436e8/>

<1>4. QED
  `n` の節点が使用する変数は、`vars.bindings` に束縛を持つか持たないかのどちらかである
  (L0a (a) より `vars.bindings` は `Post(V)` に `collect_bindings` を掛けた表でもある)。持たないものは
  `<1>3a` が扱う。持つものについては、L0a (b) が `Post(V)` について A11 を与えるので、変数の使用は
  その位置でスコープに入っている束縛に解決する。`<1>1` の 4 種のうち、
  パラメータ・capture は `<1>2` により活性化の初めから値を得ており、残る 3 種は `<1>3` によりその
  スコープの中の各位置で値を得ている。D6 より変数の値はそれを束縛する節点の後は変わらないので、
  一度値を得た変数は以後の位置でも「その時点までに値を得た変数」である。
  BY <1>1, <1>2, <1>3, <1>3a, <ref id=3c337ed/>, <ref id=3905b4e/>, <ref id=596a46d/>

### L20b (名前を束縛する構文は 1 つ) <!--#2c53bd2-->

**言明**。`Post(V)` について、`vars.bindings.get(x)` が `Some(Binding::Move(..))`、
`Some(Binding::Join(..))`、`Some(Binding::Field(..))`、`Some(Binding::Payload(..))`、
`Some(Binding::Llvm(..))`、`Some(Binding::Producer)` のいずれかであるとき、`x` を束縛する構文は
`Post(V)` に 1 つしかなく、それは `collect_bindings` を `Post(V)` に掛けたときにその `Binding` を
記録した構文である。

<1>1. `Post(V)` の束縛変数の名前は互いに異なる。
  L0a (b) が `Post(V)` について A6 の性質を与える。
  BY <ref id=3c337ed/>

<1>2. QED
  L0a (a) より `vars.bindings` は `Post(V)` に `collect_bindings` を掛けた表である。`collect_bindings`
  が `bindings` に名前を入れるのは、`Let` の束縛変数、`Destructure` のフィールド変数、`Match` のアームの
  payload 変数の 3 か所であり、D2 よりこの 3 つが本体の束縛である。`<1>1` より 1 つの名前を束縛する
  構文は高々 1 つなので、`collect_bindings` が `x` について記録した `Binding` はその唯一の構文からの
  ものである。
  BY <1>1, <ref id=3c337ed/>, <ref id=b3dfa37/>, CODE src/rc_ir/ownership.rs: collect_bindings

### L21 (静的な向き -- unit が所有ならその下の leaf も所有) <!--#66922eb-->

**言明**。`(x, π)` が unit を覆う (L18 の DEF) とする。`cand(x, π)` のすべての元 `(r, p)` について
`owns(r, p)` が真ならば、`Λ_{ty(x)}(π)` の各 leaf `λ` について、`cand(x, λ)` のすべての元についても
`owns` は真である。

以下、「`(x, π)` は所有される」を「`cand(x, π)` のすべての元について `owns` が真である」の略とする。
証明は L11a の `Reach` についての帰納である。すなわち、DEF 再帰で訪れる対 の表が `(x, π)` から進む
各相手について結論が成り立つことを帰納法の仮定とする。

<1>1. CASE `vars.bindings.get(x)` が `None` / `Some(Binding::Param)` / `Some(Binding::Producer)` /
      `Some(Binding::Field(c, idx))` で `c` が boxed / `Some(Binding::Payload(s, Some(t)))` で `s` が
      boxed のいずれか
  <2>1. これらの腕は `path` を読まずに `here()` を返すので、`origin(x, π) = Exactly((x, π))` かつ
        `origin(x, λ) = Exactly((x, λ))` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. QED
    `<2>1` より `cand(x, π) = {(x, π)}` かつ `cand(x, λ) = {(x, λ)}` なので、`(x, π)` が所有されるとは
    `owns(x, π)` が真ということであり、示すのは `owns(x, λ)` が真であることである。`x` がこの版の
    パラメータ・capture であるかどうかで場合を分ける。そうでないとき、第 3 節の (b) より `owns(x, λ)` は
    `λ` によらず真である。そうであるとき、第 3 節の (a) より
    `owns(x, λ) = owns(x, trunc(ty(x), λ))`、`owns(x, π) = owns(x, trunc(ty(x), π))` である。
    DEF unit を覆う対 より `trunc(ty(x), λ) = trunc(ty(x), π)` なので、この 2 つは同じ呼び出しであり、
    仮定より真である。
    BY <2>1, <ref id=63981a3/>, DEF unit を覆う対

<1>2. CASE `Some(Binding::Move(w))`、`Some(Binding::Join(arm_results))`、
      `Some(Binding::Payload(s, None))` のいずれか
  <2>1. `Move(w)` と `Payload(s, None)` では、任意の `ρ` について `origin(x, ρ) = origin(w, ρ)`
        (resp. `origin(s, ρ)`) である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `Move(w)` と `Payload(s, None)` について結論が出る。
    L18 より進む相手は unit を覆い、`Λ` は変わらない。`<2>1` より `(x, π)` が所有されることは
    `(w, π)` (resp. `(s, π)`) が所有されることであり、帰納法の仮定よりその `Λ` の各 leaf について
    `(w, λ)` は所有される。`<2>1` より `cand(x, λ) = cand(w, λ)` である。
    BY <2>1, <ref id=af00c3a/>
  <2>3. `Join(arm_results)` では、任意の `ρ` について `cand(x, ρ) = ⋃_{a ∈ arm_results} act(a, ρ)` で
        ある。
    `Binding::Join` の腕は各アーム結果の `origin(a, ρ).acted_on()` を集めて `of_candidates` に渡し、
    `of_candidates` の返り値の `candidates()` は渡した集合そのものである。
    BY CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates, Origin::candidates,
       Origin::acted_on
  <2>4. `Join(arm_results)` について結論が出る。
    `<2>3` より `(x, π)` が所有されるならば、各アーム結果 `a` について `cand(a, π) ⊆ act(a, π) ⊆
    cand(x, π)` なので `(a, π)` は所有される。L18 より `(a, π)` は unit を覆い `Λ` は変わらないので、
    帰納法の仮定より各 `λ` について `(a, λ)` は所有される。`cand(x, λ) = ⋃_a act(a, λ)` であり、
    `act(a, λ) = cand(a, λ) ∪ {id(a, λ)}` である (第 1 節の `id`)。`id(a, λ)` は、`origin(a, λ)` が
    `Exactly` のときは `cand(a, λ)` の元であり、`Join` のときは L13 より `owns` が真である。
    BY <2>3, <ref id=c7416c2/>, <ref id=af00c3a/>, CODE src/rc_ir/ownership.rs: Origin::acted_on, Origin::identity
  <2>5. QED
    BY <2>2, <2>4

<1>3. CASE `Some(Binding::Field(c, idx))` で `c` が unbox、または `Some(Binding::Payload(s, Some(t)))` で
      `s` が unbox
  <2>1. 任意の `ρ` について `origin(x, ρ) = origin(c, [idx] ++ ρ)` (resp. `origin(s, [t] ++ ρ)`) である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `Binding::Field(c, idx)` (`c` が unbox) について結論が出る。
    L18 (b) より進む相手 `(c, [idx] ++ π)` は unit を覆い、その `Λ` は
    `{ [idx] ++ λ : λ ∈ Λ_{ty(x)}(π) }` である。`<2>1` を `ρ = π` に当てると
    `cand(x, π) = cand(c, [idx] ++ π)` なので、`(x, π)` が所有されることは `(c, [idx] ++ π)` が所有される
    ことである。帰納法の仮定より、`Λ_{ty(c)}([idx] ++ π)` の各 leaf `[idx] ++ λ` について
    `(c, [idx] ++ λ)` は所有される。`<2>1` を `ρ = λ` に当てると `cand(x, λ) = cand(c, [idx] ++ λ)` で
    ある。
    BY <2>1, <ref id=af00c3a/>
  <2>2a. `Binding::Payload(s, Some(t))` (`s` が unbox) について結論が出る。
    L18 (b) より進む相手 `(s, [t] ++ π)` は unit を覆い、その `Λ` は
    `{ [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` である。`<2>1` を `ρ = π` に当てると
    `cand(x, π) = cand(s, [t] ++ π)` なので、`(x, π)` が所有されることは `(s, [t] ++ π)` が所有される
    ことである。帰納法の仮定より、`Λ_{ty(s)}([t] ++ π)` の各 leaf `[t] ++ λ` について
    `(s, [t] ++ λ)` は所有される。`<2>1` を `ρ = λ` に当てると `cand(x, λ) = cand(s, [t] ++ λ)` で
    ある。
    BY <2>1, <ref id=af00c3a/>
  <2>2b. QED
    この CASE の 2 つの腕を `<2>2` と `<2>2a` が扱った。
    BY <2>2, <2>2a

<1>4. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `Some((j, σ))`
  この腕は `π` が `rty` の leaf であることを要求する。A3 より `result_prov` が返す `Provenance` は
  結果の型の leaf ごとに `LeafOrigin` の集合を宣言するので、その鍵の集合は `leaves(rty)` であり、
  `leaf_origins_at` は鍵でない path に `None` を返す。L18a より `rty = ty(x)` なので
  `π ∈ leaves(ty(x))` である。`boxed_leaf_paths` の `go` は leaf を積んだ位置で戻るので、1 つの leaf が
  別の leaf の真の接頭辞になることはない。よって `π` を前置に持つ leaf は `π` だけなので
  `Λ_{ty(x)}(π) = {π}` である。すなわち `λ = π` しかなく、結論は仮定そのものである。
  BY <ref id=e11772a/>, <ref id=6c73144/>, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
     CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, LeafMap::get

<1>5. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `None`
  <2>1. `reached` が空のとき、`Λ_{ty(x)}(π)` の各 `λ` について `S_λ = ∅` であり、
        `origin(x, λ) = Exactly((x, λ))` で `owns(x, λ)` は真である。
    L18a より `rty = ty(x)` なので、L19 と L20 が語る `Λ_{rty}(π)` と `leaves(rty)` はここでの
    `Λ_{ty(x)}(π)` と `leaves(ty(x))` である。L20 (a) より、`reached` が空であることは `operand_units`
    が空かつ `produced_here` が偽であることで
    あり、これは `Λ_{ty(x)}(π)` のどの `λ` の `S_λ` も `Arg` を含まず `Fresh` も `Unknown` も含まない
    こと、すなわち `S_λ = ∅` であることと同値である。L19 よりそのとき
    `origin(x, λ) = Exactly((x, λ))` である。`x` の binding は `Binding::Llvm` なので L13 より
    `owns(x, λ)` は真である。
    BY <ref id=c7416c2/>, <ref id=6c73144/>, <ref id=18e0a91/>, <ref id=cd28c3f/>
  <2>2. `reached` が空でないとき、`reached` の各元 `o` について、`o` の候補はすべて `owns` で真である。
    L20 (d) より `cand(o) ⊆ cand(x, π)` であり、仮定より `cand(x, π)` の元はすべて真である。
    BY <ref id=cd28c3f/>
  <2>3. `reached` が空でないとき、`Λ_{ty(x)}(π)` の各 `λ` について `(x, λ)` は所有される。
    L18a より `rty = ty(x)` なので `Λ_{ty(x)}(π) = Λ_{rty}(π)` であり、その各 `λ` は `leaves(rty)` の
    元である。L19 より `S_λ` は 3 つの形のいずれかである。`S_λ = ∅` と `S_λ = {Fresh}` と `S_λ = {Unknown}` では
    `origin(x, λ) = Exactly((x, λ))` であり、L13 より `owns(x, λ)` は真である。
    `S_λ = {Arg(j, σ)}` では `origin(x, λ) = origin(args[j], σ)` である。L20 (a) より
    `w := trunc(ty(args[j]), σ)` について `(j, w) ∈ operand_units` であり、L20 (b) より
    `origin(args[j], w) ∈ reached` なので、`<2>2` より `(args[j], w)` は所有される。L18 より
    `(args[j], w)` は unit を覆う。`trunc` の答えは引数の接頭辞なので `w ⊑ σ` であり、
    `σ ∈ leaves(ty(args[j]))` (L19) なので `σ ∈ Λ_{ty(args[j])}(w)` である。帰納法の仮定より
    `(args[j], σ)` は所有される。
    BY <2>2, <ref id=c7416c2/>, <ref id=af00c3a/>, <ref id=6c73144/>, <ref id=18e0a91/>, <ref id=cd28c3f/>, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>4. QED
    BY <2>1, <2>3

<1>6. QED
  `Binding` は 7 つの構成子を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join`)、
  `vars.bindings.get(x)` はそれらか `None` である。`Field` は容器の boxed / unbox で、`Payload` は
  `tag` の有無と scrutinee の boxed / unbox で、`Llvm` は `as_arg_projection` の答えで分かれる。
  A3 より `result_prov` は決定的なので、最後の分かれ目 -- `decl.leaf_origins_at(π)` を
  `as_arg_projection` に掛けた答え -- は鍵 `(x, π)` から決まり、この数え上げは呼び出しの時点に
  依らない。
  `<1>1` から `<1>5` はこの分け方を尽くす。帰納法の仮定を使ったのは `<1>2` (`Move` / `Payload(s, None)` の
  `(w, π)`・`(s, π)`、`Join` の `(a, π)`)、`<1>3` (`(c, [idx] ++ π)`・`(s, [t] ++ π)`)、`<1>5`
  (`(args[j], w)`) であり、どれも DEF 再帰で訪れる対 の表が `(x, π)` から進む相手なので、L11a より
  この帰納は整礎である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <ref id=e11772a/>, <ref id=7bdd3d4/>, DEF 再帰で訪れる対,
     CODE src/rc_ir/ownership.rs: Binding, origin_inner, as_arg_projection

### L21a (別名の辺は値を運ぶ) <!--#42f1c2e-->

**言明**。`Post(V)` の 1 つの活性化 (D21) とその辿る実行路の 1 つの位置を固定し、`x` がその位置までに
値を得ているものとする。`vars.bindings.get(x)` に応じて次が成り立つ。

- **(m1)** `Some(Binding::Move(w))` のとき、`w` もその位置までに値を得ており、`x` の値は `w` の値である。
- **(m2)** `Some(Binding::Payload(s, None))` のとき、`s` もその位置までに値を得ており、`x` の値は `s` の
  値である。
- **(m3)** `Some(Binding::Join(arm_results))` のとき、この活性化はその `Match` でちょうど 1 つのアームを
  選んでおり、そのアームの結果の変数 `a*` (`returned_var` が返す、アーム本体の終端の `Ret` が名指す変数)
  は `arm_results` の元であってその位置までに値を得ており、`x` の値は `a*` の値である。
- **(m4)** `Some(Binding::Field(c, idx))` のとき、`c` もその位置までに値を得ており、`x` の値は `c` の値の
  第 `idx` フィールドである。
- **(m5)** `Some(Binding::Payload(s, Some(t)))` で `ty(s)` が unbox の union のとき、`s` もその位置までに
  値を得ており、その位置の `s` の値の実行時のタグは `t` であって、`x` の値は `s` の値の第 `t` 変位の
  payload である。

<1>1. `collect_bindings` が各 `Binding` を記録する構文は次のとおりである。`Binding::Move(y)` は
      `Let(x, Var(y), k)`、`Binding::Join(arm_results)` は `Let(x, Match(scrut, arms), k)` で
      `arm_results[i] = returned_var(&arms[i].body)`、`Binding::Field(container, idx)` は
      `Destructure(container, fields, _, k)` の `(idx, x) ∈ fields`、`Binding::Payload(scrut, tag)` は
      `Let(_, Match(scrut, arms), _)` のアームのうち `arm.tag = tag` であるものの payload 変数である。
      L0a (a) より `vars.bindings` は `Post(V)` に `collect_bindings` を掛けた表でもあるので、
      この構文は `Post(V)` の中に在る。
  BY <ref id=3c337ed/>, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var

<1>1a. `Post(V)` において `x` を束縛する構文は 1 つであり、それは `<1>1` が `vars.bindings.get(x)` の
       値について挙げる構文である。
  L20b である。この step が扱う 5 種の `Binding` は L20b が挙げるものに含まれる。
  BY <1>1, <ref id=2c53bd2/>

<1>1b. `x` が値を得るのは、`Post(V)` の中の `<1>1a` の構文がそれを束縛する位置においてだけである。
       したがって、`x` がある位置までに値を得ているならば、活性化の辿る実行路は `Post(V)` のその節点を
       通っている。
  D6 は値を得る形を 3 つ挙げる -- 節点が束縛する変数、パラメータ・capture、`vars.bindings` に束縛を
  持たない名前である。この step が扱う 5 種の `Binding` はどれも `collect_bindings` が記録するもので
  あり、`vars.bindings.get(x)` はそれを値に持つので、`x` は第 3 の形ではない。L1e の (i) と (ii) の
  同値より、この版のパラメータ・capture であるのは `vars.bindings.get(x)` が `Some(Binding::Param)` で
  ある名前ちょうどなので、`x` は第 2 の形でもない。残るのは第 1 の形であり、`<1>1a` より `x` を束縛
  する構文は 1 つである。
  BY <1>1, <1>1a, <ref id=596a46d/>, <ref id=14909ed/>, CODE src/rc_ir/ownership.rs: collect_bindings

<1>2. `Binding::Move(w)` の `w`、`Binding::Payload(s, tag)` の `s`、`Binding::Field(c, idx)` の `c` は、
      `x` がその位置までに値を得ているならば、その位置までに値を得ている。
  `<1>1` より、この 3 種の `Binding` を記録する `Post(V)` の構文は `Let(x, Var(w), k)`、
  `Let(_, Match(s, arms), _)`、`Destructure(c, fs, _, k)` であり、どれも `w` / `s` / `c` を節点そのものに
  `RcVar` として持つ。`<1>1b` より、`x` がその位置までに値を得ているならば活性化の辿る実行路は
  この節点を通っている
  (D2 と D3 -- `Let` の束縛変数と `Destructure` のフィールド変数は
  `k` へ進むところで、`Match` のアームの payload はそのアーム本体へ入るところで値を得る)。L20a を
  その節点の位置に当てると `w` / `s` / `c` はそこまでに値を得ており、D6 より以後の位置でも値を得た
  変数である。
  BY <1>1, <1>1b, <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=596a46d/>, <ref id=57d5753/>

<1>2a. 名前 `y` が活性化のある位置までに値を得ているならば、その値は以後のどの位置でも変わらない。
  `vars.bindings.get(y)` が `Some(・)` であれば `y` は束縛を持つ名前であり、D6 の「変数の値は、それを
  束縛する節点の後は変わらない」により、その束縛する節点より後のどの位置でも値は変わらない。`None` で
  あれば `y` は記号の位置であり (D6)、D6 はその値を「その記号の値」という位置に依らない量として定めるので、
  どの位置で読んでも同じ値である。どちらの場合も、`y` がある位置までに値を得ていれば、それ以後のどの
  位置でもその値は変わらない。
  BY <ref id=596a46d/>

**アーム結果はこの step に入らない。** `Binding::Join(arm_results)` の `arm_results` は各アームの
結果の変数を並べたものであり、活性化が選ばなかったアームの本体は走らないので、その結果の変数は値を
得ていない。(m3) が要るのは選ばれたアームの結果の変数だけであり、それは (m3) の証明が別に与える。

<1>3. (m1) が成り立つ。
  D2 より `Let(x, rhs, k)` は `rhs` の値を `x` に束縛し、D7 より `Var` は値を渡すだけなので、この節点の
  実行の直後、`x` の値はその直前の `w` の値と同じである。コード生成もそうなっている -- `eval_rc_rhs` の
  `RcRhs::Var(v)` の腕は `get_scoped_obj(&v.name)` を返し、`bind_and_continue` がそれを `x` の名前で
  `scope_push` する。`<1>2a` より `x` の値も `w` の値も、この節点の実行より後、固定した位置まで変わらない
  ので、固定した位置でも `x` の値は `w` の値である。
  BY <1>1, <1>2, <1>2a, <ref id=b3dfa37/>, <ref id=56c2068/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_rhs,
     Generator::bind_and_continue, Generator::eval_rc_expr_inner

<1>4. (m2) が成り立つ。
  `eval_rc_match` は、`arm.tag` が `None` のアームの payload を `get_scoped_obj_noretain(&scrut.name)` に
  束縛する。すなわち、そのアームへ入る節点の実行の直後、`x` の値はその直前の `s` の値そのものである。
  `<1>2a` より `x` の値も `s` の値も、それ以後固定した位置まで変わらないので、固定した位置でも `x` の値は
  `s` の値である。
  BY <1>1, <1>2, <1>2a, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match

<1>5. (m3) が成り立つ。
  <2>1. この活性化はこの `Match` でちょうど 1 つのアームを選び、そのアーム本体の実行路を辿ってから
        `x` を束縛する。
    `<1>1` と `<1>1a` より `Binding::Join(arm_results)` を記録する構文は `Let(x, Match(scrut, arms), k)`
    ただ 1 つであり、`x` が値を得るのはこの節点で `k` へ進むところなので (D2、D3、`<1>1b`)、`x` が
    その位置までに値を得ているならば活性化はこの節点を通っている。D21 より活性化はその `Match` で
    ちょうど 1 つのアームを選び、D3 より辿る実行路はそのアーム本体の実行路を辿ってから `k` へ進む。
    BY <1>1, <1>1a, <1>1b, <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=c232680/>
  <2>2. そのアーム本体の実行路の最後の節点は `Ret(a*)` であり、`a*` は `arm_results` の元である。
    D2 より `Ret` は唯一の終端子であり、残る 5 種はちょうど 1 つの継続を持つ。D3 より、アーム本体の
    実行路はアーム本体の根から継続を辿って `Ret` に着いて終わる (途中の `Match` はアーム本体の実行路を
    辿ってからその継続へ進む)。`collect_bindings` は `arm_results[i]` に `returned_var(&arms[i].body)`
    を置き、`returned_var` はその継続の鎖を辿って着く `Ret` が名指す変数を返す。
    BY <1>1, <ref id=b3dfa37/>, <ref id=ca36627/>, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var
  <2>3. `a*` はこの位置までに値を得ている。
    `<2>1` と `<2>2` より `Ret(a*)` の節点は活性化の辿る実行路の上にあり、`x` が値を得る位置より前で
    ある。L20a をその節点の位置に当てると `a*` はそこまでに値を得ており、D6 より以後の位置でも値を
    得た変数である。
    BY <2>1, <2>2, <ref id=596a46d/>, <ref id=57d5753/>
  <2>4. QED
    D2 より `Let(x, Match(..), k)` は `Match` の値を `x` に束縛し、アーム本体の `Ret(v)` の値は `v` で
    ある。コード生成もそうなっている。`eval_rc_expr_inner` の `Let(x, Match(scrut, arms), k)` の腕は
    `match_tail = binding_fuses_into_return(x, k, tail)` を作って `eval_rc_match` に渡し、
    `eval_rc_match` は各アームの本体をその `match_tail` で評価する。`RcExpr::Ret(a*)` の腕は
    `get_scoped_obj(&a*.name)` を `build_tail` に渡し、`build_tail` は `tail` が偽のとき `Some(obj)` を
    返し、真のとき `build_return_object` で返して `None` を返す。
    `match_tail` が偽のときは、各アームの `Ret(a*)` が `a*` の値を返し、`eval_rc_match` がそれを
    `incomings` に積んで phi で合流させ、`bind_and_continue` がその値を `x` の名前で `scope_push` する。
    選ばれたアームを通る実行では phi の値はそのアームの `incomings` の値である。
    `match_tail` が真のときは、各アームの `Ret(a*)` が `a*` の値を直に返し、`binding_fuses_into_return`
    が真を返す条件より継続 `k` は `x` を move-bind の鎖で `Ret` へ運ぶだけである。どちらの場合も、
    `x` の値はこの活性化が選んだアームの `a*` の値である。
    BY <2>1, <2>2, <2>3, <ref id=b3dfa37/>, CODE src/rc_ir/codegen.rs: carries_var_to_return,
       Generator::eval_rc_match, Generator::eval_rc_expr_inner, Generator::bind_and_continue,
       Generator::binding_fuses_into_return,
       CODE src/generator.rs: Generator::build_tail, Generator::build_return_object

<1>6. (m4) が成り立つ。
  D2 より `Destructure(c, fs, s, k)` は容器 `c` をフィールドに分解し、各 `(i, x)` の `x` に第 `i`
  フィールドを束縛するので、この節点の実行の直後、`x` の値はその直前の `c` の値の第 `idx` フィールドである。
  コード生成もそうなっている -- `eval_rc_expr_inner` の `Destructure` の腕は
  `get_struct_fields` に `fields` の添字の列を渡し、返る 1 対 1 の列を対応するフィールド変数の名前で
  `scope_push` する。`<1>2a` より `x` の値も `c` の値も、この節点の実行より後、固定した位置まで変わらない
  ので、固定した位置でも `x` の値は `c` の値の第 `idx` フィールドである。
  BY <1>1, <1>2, <1>2a, <ref id=b3dfa37/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
     CODE src/object.rs: ObjectFieldType::get_struct_fields

<1>7. (m5) が成り立つ。
  <2>1. 活性化はこの `Match` で、`x` を payload に持つアーム (`tag` が `Some(t)`) を選んでいる。
    `<1>1` より `x` は `tag = Some(t)` のアームの payload 変数であり、`<1>1a` よりそれが `x` を束縛する
    唯一の構文である。payload 変数が値を得るのはそのアームへ入るところなので (D2 と D3、`<1>1b`、および
    `eval_rc_match` がアーム本体を評価する前に payload を `scope_push` すること)、`x` がこの位置までに
    値を得ているならば活性化はそのアームを選んでいる。
    BY <1>1, <1>1a, <1>1b, <ref id=b3dfa37/>, <ref id=ca36627/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
  <2>2. `s` はこの位置までに値を得ている。
    `<1>2` を `Binding::Payload(s, Some(t))` に当てる。
    BY <1>2
  <2>3. その位置の `s` の値の実行時のタグは `t` である。
    D21 より、活性化が選ぶアームは `s` の値の実行時のタグに `tag` が等しいアームであり、そのような
    アームが無ければコード生成の振る舞いに従う。この 2 つで場が尽きるので、順に見る。
    そのようなアームが在る場合、活性化が選ぶのはそのアームである。`<2>1` より活性化が選んだアームの
    `tag` は `Some(t)` なので、実行時のタグは `t` である。
    そのようなアームが無い場合、A16 より `arms` は catch-all アーム (`tag` が `None`) を持つ。
    `eval_rc_match` は最後のアームのブロックを `else_bb` とし、それ以外の各アームについて
    `arm.tag.expect("a non-final match arm must be a variant arm")` を switch の case に積むので、
    最後でない catch-all アームがあればコード生成は中断し、活性化は存在しない。よって catch-all は
    最後のアームであって `else_bb` はそのブロックであり、どの case のタグも実行時のタグに等しくない
    ので switch は `else_bb` へ跳ぶ (アームが 1 つだけのときは `cases` が空で `else_bb` への無条件
    分岐になり、その 1 つのアームが catch-all である)。すなわち選ばれるアームの `tag` は `None` で
    あり、`<2>1` の `Some(t)` と食い違うので、この場合は起こらない。
    BY <2>1, <ref id=f769887/>, <ref id=c232680/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
  <2>4. QED
    `eval_rc_match` は、`arm.tag` が `Some(_)` のアームの payload を
    `get_union_value_noretain_norelease(scrut_obj, &arm.payload.ty)` に束縛する。これは union の payload
    バッファを変位の型として読んだものであり、`ty(s)` が unbox なので retain も伴わない。`<2>3` より
    その変位は、そのアームへ入る節点の実行の直前の `s` の値の実行時のタグ `t` の変位であり、その節点の
    実行の直後、`x` の値はその payload である。`<1>2a` より `x` の値も `s` の値 (したがってその実行時の
    タグ) も、それ以後固定した位置まで変わらないので、固定した位置でも `s` の値の実行時のタグは `t` で
    あり、`x` の値はその第 `t` 変位の payload である。
    BY <2>1, <2>2, <2>3, <1>2a, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match,
       CODE src/object.rs: ObjectFieldType::get_union_value_noretain_norelease,
       ObjectFieldType::get_value_from_union_buf

<1>8. QED
  BY <1>3, <1>4, <1>5, <1>6, <1>7

### L22 (実行時の向き -- unit が非所有ならその下の inhabited な leaf も非所有) <!--#4e2b22e-->

**言明**。`Post(V)` の 1 つの活性化 (D21) とその実行路の 1 つの位置を固定する。`x` がその位置までに
値を得ており、`(x, π)` が unit を覆う (L18 の DEF) とする。`Inh_x(π)` を、その位置の `x` の値について inhabited (D16)
である `Λ_{ty(x)}(π)` の leaf の集合とする。`cand(x, π)` のすべての元 `(r, p)` について `owns(r, p)` が
**偽**ならば、`Inh_x(π)` の各 `λ` について `cand(x, λ)` に `owns` が偽である元がある。

以下、「`(x, π)` は全部偽である」を「`cand(x, π)` のすべての元について `owns` が偽である」の略とする。
証明は L11a の `Reach` についての帰納である。すなわち、DEF 再帰で訪れる対 の表が `(x, π)` から進む
各相手について結論が成り立つことを帰納法の仮定とする。

<1>1. CASE `vars.bindings.get(x)` が `None` / `Some(Binding::Producer)` /
      `Some(Binding::Field(c, idx))` で `c` が boxed / `Some(Binding::Payload(s, Some(t)))` で `s` が
      boxed のいずれか
  この場合は仮定と両立しない。これらの腕は `here()` を返すので `cand(x, π) = {(x, π)}` である。
  この 4 つの場合 -- `bindings.get(x)` が `None`、`Producer`、boxed 容器の `Field`、boxed scrutinee の
  `Payload` -- はどれも L13 の言明が挙げるものなので、L13 より `owns(x, π)` は真である。これは
  「全部偽」に反する。
  BY <ref id=c7416c2/>, CODE src/rc_ir/ownership.rs: origin_inner

<1>2. CASE `Some(Binding::Param)`
  この腕は `path` を読まずに `here()` を返すので `cand(x, π) = {(x, π)}` かつ
  `cand(x, λ) = {(x, λ)}` である。この腕の条件は L1e の (i) なので、L1e より `x` はこの版のパラメータ・
  capture であり、第 3 節の (a) が使えて
  `owns(x, λ) = owns(x, trunc(ty(x), λ))`、`owns(x, π) = owns(x, trunc(ty(x), π))` である。
  DEF unit を覆う対 より `trunc(ty(x), λ) = trunc(ty(x), π)` なので `owns(x, λ) = owns(x, π)` であり、
  仮定よりこれは偽である。`Inh_x(π) ⊆ Λ_{ty(x)}(π)` なので、`Inh_x(π)` の各 `λ` について
  `cand(x, λ) = {(x, λ)}` は `owns` が偽である元を持つ。
  BY <ref id=14909ed/>, <ref id=63981a3/>, DEF unit を覆う対, CODE src/rc_ir/ownership.rs: origin_inner

<1>3. CASE `Some(Binding::Move(w))` または `Some(Binding::Payload(s, None))`
  <2>1. 任意の `ρ` について `origin(x, ρ) = origin(w, ρ)` (resp. `origin(s, ρ)`) である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `x` の値は `w` の値 (resp. `s` の値) と同じ値であり、`w` (resp. `s`) はこの位置までに値を得て
        いる。
    L21a の (m1) (resp. (m2)) である。
    BY <ref id=42f1c2e/>
  <2>3. QED
    `<2>2` より `Inh_x(π) = Inh_w(π)` である (D16 は値だけを見る)。L18 より `(w, π)` は unit を覆い
    `Λ` は変わらない。`<2>1` より `(w, π)` は全部偽なので、帰納法の仮定より `Inh_w(π)` の各 `λ` に
    ついて `cand(w, λ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>1, <2>2, <ref id=66c9670/>, <ref id=af00c3a/>

<1>4. CASE `Some(Binding::Join(arm_results))`
  <2>1. `cand(x, ρ) = ⋃_{a ∈ arm_results} act(a, ρ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates, Origin::candidates,
       Origin::acted_on
  <2>2. この活性化はこの `Match` でちょうど 1 つのアームを選んでおり、そのアーム本体の終端の `Ret` が
        名指す変数を `a*` とすると、`a* ∈ arm_results` であり、`a*` はこの位置までに値を得ていて、
        `x` の値は `a*` の値と同じ値である。
    L21a の (m3) である。
    BY <ref id=42f1c2e/>
  <2>3. `(a*, π)` は全部偽である。
    `<2>2` より `a* ∈ arm_results` なので、`<2>1` より
    `cand(a*, π) ⊆ act(a*, π) ⊆ cand(x, π)` である。
    BY <2>1, <2>2
  <2>4. QED
    `<2>2` より `Inh_x(π) = Inh_{a*}(π)` である。L18 より `(a*, π)` は unit を覆い `Λ` は変わらない。
    `<2>3` と帰納法の仮定より、`Inh_{a*}(π)` の各 `λ` について `cand(a*, λ)` に `owns` が偽の元があり、
    `<2>1` より `cand(a*, λ) ⊆ cand(x, λ)` である。
    BY <2>1, <2>2, <2>3, <ref id=66c9670/>, <ref id=af00c3a/>

<1>5. CASE `Some(Binding::Field(c, idx))` で `c` が unbox
  <2>1. 任意の `ρ` について `origin(x, ρ) = origin(c, [idx] ++ ρ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `c` はこの位置までに値を得ており、`x` の値は `c` の値の第 `idx` フィールドである。
    L21a の (m4) である。
    BY <ref id=42f1c2e/>
  <2>3. `λ ∈ Inh_x(π)` と `[idx] ++ λ ∈ Inh_c([idx] ++ π)` は同値である。
    A12 より `ty(c)` は構造体なので `is_union(ty(c))` は偽であり、`[idx] ++ λ` が `ty(c)` の根で通る節は
    unbox union ではない。よって `[idx] ++ λ` が通る unbox union の節と、それぞれで選ぶ変位番号は、`λ` が
    `ty(x)` で通るものと同じである。`<2>2` より `x` の値は `c` の値のその位置の部分値なので、各節の
    タグも同じである。D16 はこの一致だけを見る。
    BY <2>2, <ref id=83d98e9/>, <ref id=3c337ed/>, <ref id=66c9670/>, CODE src/ast/types.rs: TypeNode::is_union
  <2>4. QED
    L18 より `(c, [idx] ++ π)` は unit を覆い、その `Λ` は `{ [idx] ++ λ : λ ∈ Λ_{ty(x)}(π) }` である。
    `<2>1` より `(c, [idx] ++ π)` は全部偽なので、帰納法の仮定と `<2>3` より、`Inh_x(π)` の各 `λ` に
    ついて `cand(c, [idx] ++ λ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>1, <2>3, <ref id=af00c3a/>

<1>6. CASE `Some(Binding::Payload(s, Some(t)))` で `s` が unbox
  <2>1. 任意の `ρ` について `origin(x, ρ) = origin(s, [t] ++ ρ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `s` はこの位置までに値を得ており、その位置の `s` の値の実行時のタグは `t` であって、`x` の値は
        `s` の値の第 `t` 変位の payload である。
    L21a の (m5) である。この腕の条件より `ty(s)` は boxed でなく、A12 より union なので、(m5) の前提が
    成り立つ。
    BY <ref id=83d98e9/>, <ref id=3c337ed/>, <ref id=42f1c2e/>
  <2>3. `λ ∈ Inh_x(π)` と `[t] ++ λ ∈ Inh_s([t] ++ π)` は同値である。
    A12 より `ty(s)` は union であり、この腕の条件より unbox である。よって `[t] ++ λ` が `ty(s)` の根で
    通る節は unbox union であり、そこで選ぶ変位番号は `t` である。`<2>2` よりその位置の `s` のタグは
    `t` なので、この節の条件は成り立つ。残りの節は `λ` が `ty(x)` で通るものと同じであり、`<2>2` より
    `x` の値は `s` の値の第 `t` 変位の payload なのでタグも同じである。D16 はこれらの節だけを見る。
    BY <2>2, <ref id=83d98e9/>, <ref id=3c337ed/>, <ref id=66c9670/>
  <2>4. QED
    L18 より `(s, [t] ++ π)` は unit を覆い、その `Λ` は `{ [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` である。
    `<2>1` より `(s, [t] ++ π)` は全部偽なので、帰納法の仮定と `<2>3` より、`Inh_x(π)` の各 `λ` に
    ついて `cand(s, [t] ++ λ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>1, <2>3, <ref id=af00c3a/>

<1>7. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `Some((j, σ))`
  この腕は `π` が `rty` の leaf であることを要求する。A3 より `result_prov` が返す `Provenance` は
  結果の型の leaf ごとに `LeafOrigin` の集合を宣言するので、その鍵の集合は `leaves(rty)` であり、
  `leaf_origins_at` は鍵でない path に `None` を返す。L18a より `rty = ty(x)` なので
  `π ∈ leaves(ty(x))` である。
  `boxed_leaf_paths` の `go` は leaf を積んだ位置で戻るので、`π` を前置に持つ leaf は `π` だけで
  あり、`Λ_{ty(x)}(π) = {π}` である。`Inh_x(π) ⊆ {π}` であり、`λ = π` のとき
  `cand(x, λ) = cand(x, π)` は仮定より全部偽である。`cand(x, π)` は空でない -- `x` は `vars.bindings` に
  `Binding::Llvm` を持つ束縛変数なので、P2 (固定した版が借用版のときは P9 と合わせて読む) より
  `origin(x, π)` は panic せずに答えを返し、L12 の最後の節より `cand(x, π)` は空でない。
  よって `cand(x, λ)` に `owns` が偽である元がある。
  BY <ref id=e11772a/>, <ref id=44a9669/>, <ref id=6c73144/>, <ref id=0edb0ba/>, <ref id=63eadd9/>,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
     CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, LeafMap::get

<1>8. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `None`
  <2>1. `reached` は空でない。
    L20 (c) より、空ならば `origin(x, π) = Exactly((x, π))` であり、`x` の binding は `Binding::Llvm` な
    ので L13 より `owns(x, π)` は真である。これは「全部偽」に反する。
    BY <ref id=c7416c2/>, <ref id=cd28c3f/>
  <2>2. `produced_here` は偽である。
    L20 (e) より、真ならば `(x, π) ∈ cand(x, π)` であり、L13 より `owns(x, π)` は真である。これは
    「全部偽」に反する。
    BY <ref id=c7416c2/>, <ref id=cd28c3f/>
  <2>3. `operand_units` の各 `(j, w)` について `(args[j], w)` は全部偽である。
    L20 (b) より `origin(args[j], w) ∈ reached` であり、`<2>1` より `reached` は空でないので L20 (d) より
    `cand(args[j], w) ⊆ cand(x, π)` である。
    BY <2>1, <ref id=cd28c3f/>
  <2>4. `Inh_x(π)` の各 `λ` について `S_λ = {Arg(j, σ)}` の形である。
    L18a より `rty = ty(x)` なので `Inh_x(π) ⊆ Λ_{ty(x)}(π) = Λ_{rty}(π)` であり、その各 `λ` は
    `leaves(rty)` の元である。
    L19 より `S_λ` は `∅` か `{Fresh}` か `{Unknown}` か `{Arg(j, σ)}` である。`S_λ = ∅` は起きない --
    A3 の表の第 1 行が、空集合と宣言された leaf は inhabited にならないと述べるからである。
    `S_λ = {Fresh}` と `S_λ = {Unknown}` も起きない -- L20 (a) よりそのとき `produced_here` が真になり、
    `<2>2` に反するからである。
    BY <ref id=e11772a/>, <ref id=6c73144/>, <ref id=18e0a91/>, <ref id=cd28c3f/>, <2>2
  <2>4a. `args[j]` は `vars.bindings` に束縛を持つ (記号の位置でない)。
    `vars.bindings.get(args[j])` が `None` であるとすると、`origin_inner` の `None` の腕は `here()` を
    返すので `cand(args[j], w) = {(args[j], w)}` であり、L13 より `owns(args[j], w)` は真である。しかし
    `<2>3` より `(args[j], w)` は全部偽、すなわち `cand(args[j], w)` のすべての元について `owns` が偽で
    あり、その唯一の元 `(args[j], w)` について `owns` が真かつ偽になって矛盾する。よって
    `vars.bindings.get(args[j])` は `None` でない。
    BY <2>3, <ref id=c7416c2/>, CODE src/rc_ir/ownership.rs: origin_inner
  <2>5. QED
    `λ ∈ Inh_x(π)` を取り、`S_λ = {Arg(j, σ)}` とする (`<2>4`)。L19 より
    `origin(x, λ) = origin(args[j], σ)` かつ `σ ∈ leaves(ty(args[j]))` である。L20 (a) より
    `w := trunc(ty(args[j]), σ)` について `(j, w) ∈ operand_units` である。`trunc` の答えは引数の接頭辞
    なので `w ⊑ σ`、よって `σ ∈ Λ_{ty(args[j])}(w)` である。
    L20b より `x` を束縛する構文は `collect_bindings` が `Binding::Llvm(gen, args, rty)` を記録した
    `Let(x, Llvm(gen, args), k)` ただ 1 つであり、`x` がこの位置までに値を得ているので活性化はその節点を
    通っている。その節点は `args` の各要素を `RcVar` として持つので、L20a より `args[j]` もその節点まで
    に値を得ている。
    A3 の表の「単一の `Arg(j, σ)`」の行は、結果のその leaf が inhabited であることと第 `j` オペランドの
    leaf `σ` が inhabited であることが同値だと述べる。その行が語るのはこの節点が走る瞬間の結果と
    オペランドであり、`Inh_x(π)` と `Inh_{args[j]}(w)` は固定した 1 つの位置の量である。`x` は
    `Binding::Llvm` を持つ束縛変数であり、`<2>4a` より `args[j]` も束縛を持つ名前なので、D6 より両者の
    値はそれを束縛する節点の後は変わらない。D16 の inhabited は値だけを見るので、`x` と `args[j]` の
    どちらの値もその瞬間と固定した位置とで同じであり、同じ leaf が inhabited である。よって
    `σ ∈ Inh_{args[j]}(w)` である。L18 より `(args[j], w)` は unit を覆い、`<2>3` より全部偽なので、
    帰納法の仮定より `cand(args[j], σ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>3, <2>4, <2>4a, <ref id=e11772a/>, <ref id=596a46d/>, <ref id=66c9670/>, <ref id=af00c3a/>, <ref id=6c73144/>, <ref id=18e0a91/>, <ref id=cd28c3f/>, <ref id=57d5753/>, <ref id=2c53bd2/>,
       CODE src/rc_ir/ownership.rs: truncate_to_unit, collect_bindings

<1>9. QED
  `Binding` は 7 つの構成子を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join`)、
  `vars.bindings.get(x)` はそれらか `None` である。`Field` は容器の boxed / unbox で、`Payload` は
  `tag` の有無と scrutinee の boxed / unbox で、`Llvm` は `as_arg_projection` の答えで分かれる。
  A3 より `result_prov` は決定的なので、最後の分かれ目 -- `decl.leaf_origins_at(π)` を
  `as_arg_projection` に掛けた答え -- は鍵 `(x, π)` から決まり、この数え上げは呼び出しの時点に
  依らない。
  `<1>1` から `<1>8` はこの分け方を尽くす。帰納法の仮定を使ったのは `<1>3` (`(w, π)`・`(s, π)`)、
  `<1>4` (`(a*, π)`)、`<1>5` (`(c, [idx] ++ π)`)、`<1>6` (`(s, [t] ++ π)`)、`<1>8` (`(args[j], w)`)
  であり、どれも DEF 再帰で訪れる対 の表が `(x, π)` から進む相手なので、L11a よりこの帰納は整礎である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <ref id=e11772a/>, <ref id=7bdd3d4/>, DEF 再帰で訪れる対,
     CODE src/rc_ir/ownership.rs: Binding, origin_inner, as_arg_projection

### P7a の 2 つの向き

**証明するもの**。第 1 節が固定する出力版 `V`、`V` の site `(v, u)` (DEF site)、`infer_ownership` の
不動点、`Post(V)` の 1 つの活性化と、その活性化の上で `v` が値を得ている (D6) 任意の位置を固定する。
このとき **節 1 から節 3** と **節 2 から節 1** が成り立つ。

<1>1. `(v, u)` は unit を覆う。
  DEF site より `(v, u)` は、`Pre(V)` の `Retain(v, path, ..)` / `Release(v, path, ..)` 節点の
  対 `(v, path)` か、`App` の引数 `arg` と `unit ∈ units(ty(arg))` の対 `(arg, unit)` である。後者では
  `u` は DEF site から `units(ty(v))` の元である。前者について、`V` が `f_own` の版かグローバル初期化子の
  版であれば、`Pre(V)` は入力の本体そのものである -- `borrow_ify` は `func.clone()` を写し、グローバルは
  `g.init` を写す。`V` が借用版であれば、`Pre(V)` は入力の関数の本体の束縛変数を一斉に付け替えたもので
  あって、それ以外の違いを持たない (P9)。よってどちらでも、`Retain`/`Release` 節点の `path` と名指す
  変数の型は入力の本体のものと同じであり、A2 より入力のその `path` は `units(ty(v))` の元なので、
  `Pre(V)` でもそうである。
  L18 (a) より `(v, u)` は unit を覆う。
  BY <ref id=8e3aff3/>, <ref id=63eadd9/>, <ref id=af00c3a/>, DEF site, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>1a. 固定した位置で `v` は値を得ている。
  読み方より、固定した位置は `v` が値を得ている位置として取ったものである。**`Post(V)` に残った
  site の節点を訪れる位置はその 1 つである** -- DEF site より site の節点は
  `Retain(v, path, ..)` / `Release(v, path, ..)` か、`v` を引数に持つ `Let(_, App(_, args), _)` であり、
  L20a が使用する変数として挙げる列にこの 2 つ -- 「`Retain` / `Release` / `Eval` / `Ret` が名指す変数」と
  「`App` の callee と各引数」-- はどちらも入っているので、活性化がその節点を訪れるならば `v` はそこ
  までに値を得ている。
  BY <ref id=596a46d/>, <ref id=57d5753/>, DEF site

<1>2. 節 1 から節 3 へ渡る。
  節 1 は「`cand(v, u)` のすべての元 `(r, p)` について `owns(r, p)` が真」である (第 1 節の
  `owns_unit`)。`<1>1` と L21 より `Λ(u)` の**すべての** leaf `λ` について `cand(v, λ)` のすべての元に
  ついて `owns` が真である。
  `Inh(v, u) ⊆ Λ(u)` なので、とくに inhabited な leaf についてそうである。これが節 3 である。
  BY <1>1, <ref id=af00c3a/>, <ref id=66922eb/>, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>3. 節 1 が偽ならば、`cand(v, u)` のすべての元について `owns` は偽である。
  `cand(v, u)` は空でない。`v` は `Pre(V)` に現れる `RcVar` の名前なので、その名前は
  `vars.bindings` に束縛を持つ (パラメータ・capture の `Binding::Param` か、`collect_bindings` が節点から
  記録した束縛) か、持たない (D6 の第 3 の形) かのどちらかであり、どちらも P2 の範囲である (固定した版が
  借用版のときは P9 と合わせて読む)。よって `origin(v, u)` は panic せずに答えを返し、L12 の最後の節より
  `cand(v, u)` は空でない。
  `V` が関数の版ならば `(v, u)` は DEF site より `levelled_sites` が挙げる site であり、グローバル
  初期化子の版ならば第 5 節の読み方より P7d は任意の `(v, u)` について答える。いまは `infer_ownership` の
  不動点なので、いずれも P7d よりその候補はすべて真かすべて偽である。節 1 が偽とは「すべて真」が
  成り立たないことなので、「すべて偽」である。
  BY <ref id=0edb0ba/>, <ref id=f7a1148/>, <ref id=63eadd9/>, <ref id=596a46d/>, <ref id=44a9669/>, DEF site, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>4. 節 1 が偽ならば節 2 も偽である。
  `<1>3` より `(v, u)` は L22 の意味で全部偽である。`<1>1a` より `v` はこの位置までに値を得ており、
  `<1>1` より `(v, u)` は unit を覆う。L22 より、`Inh(v, u)` の各 `λ` について `cand(v, λ)` に
  `owns` が偽の元がある。節 2 は、`Inh(v, u)` のある `λ` の**すべての**候補について `owns` が真である
  ことなので (第 6 節の読み方が `Λ(u)` の inhabited な leaf の集合を `Inh(v, u)` と書く)、これは節 2 の
  否定である。
  BY <1>1, <1>1a, <1>3, <ref id=af00c3a/>, <ref id=4e2b22e/>

<1>5. 節 2 から節 1 へ渡る。
  `<1>4` の対偶である。
  BY <1>4

<1>6. QED
  README の P7a は「成り立つのは **1 ⟹ 3** と **2 ⟹ 1** である。」と述べ、また
  「`Inh(v, u) ≠ ∅` を足せば 3 つは同値になるが、下流はそれを要らない。」とも述べる。この節が閉じるのは
  前者の 2 つの含意である -- `<1>2` が 1 ⟹ 3 を、`<1>5` が 2 ⟹ 1 を与える。残る 3 ⟹ 2 と 3 ⟹ 1 は
  偽であり (R2)、README も「**3 ⟹ 2 と 3 ⟹ 1 は偽である。**」と述べる。
  BY <1>2, <1>5, <ref id=d499f99/>

**証明しないもの**。節 3 から節 2 と、節 3 から節 1 である。`Inh(v, u) = ∅` のとき節 3 は空虚に真になり、
節 2 は偽になる。R2 がその本体を挙げる。`Inh(v, u) ≠ ∅` を仮定に足せば、`<1>2` と `<1>5` に
「節 3 から節 2」(`Inh(v, u)` の元を 1 つ取る) が加わって 3 つは同値になる。

**この 2 つの向きが P10 と P14 に与えるもの。** A5 より、値が保持する参照は、inhabited (D16) であって
計数下のオブジェクト (D26) を指す leaf にちょうど 1 つずつあり、inhabited でない leaf も、グローバル
状態のオブジェクトを指す inhabited な leaf も参照を持たない (D26)。A4 よりコード生成の `Retain(v, u)` /
`Release(v, u)` は `u` の下の inhabited な leaf の参照カウントだけを ±1 する。よって節 3 は「残した
節点が触れる参照はすべてこの版のものである」を、節 2 の否定は「落とした節点が `Pre(V)` の側で触れる先の
参照は、どれもこの版のものでない」を与える。

### L23 (類が参照を持つ点で、そのオブジェクトは解放されていない) <!--#5d2d9a7-->

**言明**。1 つの本体の活性化 (D21) と、それが辿る実行路 `ρ` を取る。`n` をその活性化の時点、段内の点
(D24)、または読み・触れる動作の直前の点 (D11 の (S-c) が条件を課す点) とする。`C` をこの活性化の
計数下 (D26) の別名類 (D33)、`O = obj(C)` とし、次の 3 つを仮定する。

- **(H-a)** `n` は `C` の開始の時点 (D34) 以後であり、`n` において `held_ρ(n, C) ≥ 1` であって、
  `C` の `ρ` 終端は借用する (D14) leaf ではない。
- **(H-b)** `obj(C') = O` であり開始の時点が `n` 以前である `C` 以外の計数下の別名類 `C'` は、どれも
  A19 (i) の `d(C') ≥ 0` を満たす。
- **(H-c)** 活性化は `n` まで解放について閉じている (D11a)。

このとき、`n` において `O` は解放されていない。

**この命題が要るのは、D11 が 1 つの活性化について課す条件だからである。** README の P28 の系は実行 (D24)
についての言明であり、その前提はプログラムが D12 を満たすことである。D11 を検査している段はそれを与件に
できないので、代わりに D21 が活性化に課す制限 (A19 (i) の不等式) から `H(O) ≥ 1` を出し、それを (H-c) と
合わせる。

**(H-c) は (S-c) の接頭条件そのものである。** (S-c) は各時点について、「その活性化がその時点まで
解放について閉じている」(D11a) ことを条件に主張するので、`n` における (S-c) を示す議論はこの仮定を
与件に持つ。**D11a が名指すとおり、`H(O) ≥ 1` から「解放されていない」は D24 の (F) だけからは
出ない** -- (F) は、一度解放された
オブジェクトのカウントが後から上がらないことを言わないからである。

**仮定が段でなく点の `held` を主語にするのは、D21 の `H` が別の制御の流れの段による増減も受けるから
である。** 「開始から `n` までにこの活性化の段が `O` への参照を処分しない」は `H(O)` の下限を与えない
-- 処分する段が別の制御の流れに在れば `H(O)` は下がる (D21、D24 が「複数の制御の流れ
(`FFI_EXPORT` を通じて外から入るスレッド) がある実行では、それらの段が 1 つの列に並ぶ」と述べる)。
D21 の制限は各時点と各段内の点で当たるので、その点の `held` だけから下限が出る。

<1>1. `n` において `H(O) ≥ 1` である。
  D21 より、活性化はその各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る。`n` が読み・
  触れる動作の直前の点であるときは、その点と直前の段内の点のあいだに素動作が 1 つも無いので `H` も
  `held` も動かない (D24)。よってどの場合も `n` でこの不等式を読んでよい。
  `n` において `obj(C') = O` であり開始の時点が `n` 以前である計数下の別名類の全体を `S` とする。
  (H-a) より `C ∈ S` であり、`C` の `ρ` 終端は借用する leaf でないので、A19 (i) の
  `d(C) = held_ρ(n, C) - 0 ≥ 1` である。(H-b) より `S` の残りの元はどれも `d(C') ≥ 0` を満たし、
  A19 (i) の角括弧の項は 0 か 1 である。よって A19 (i) の右辺は `d(C)` 以上、すなわち 1 以上であり、
  `H(O) ≥ 1` である。
  BY <ref id=9f1cf6c/>, <ref id=ef8efc4/>, <ref id=c232680/>, <ref id=e3436e8/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=9d5d254/>

<1>2. QED
  `<1>1` より `n` において `H(O) ≥ 1` であり、`O` は計数下 (D26) である。(H-c) より活性化は `n` まで
  解放について閉じているので、D11a の定めるところにより `n` において `O` は解放されていない。
  BY <1>1, <ref id=859cf84/>, <ref id=88a06de/>

### R1 (節 2 と節 3 の inhabited の限定が要ること)

**言明**。第 4 節の仮定のうち入力プログラムを縛るものをすべて満たす入力プログラムであって、その出力の
ある版のある site (DEF site) と `infer_ownership` の不動点について、節 1 が偽でありながら `Λ(u)` の
ある **inhabited でない** leaf のすべての候補が所有されるものがある。すなわち、節 2 から
inhabited の限定を外すと、節 2 から節 1 へ渡れなくなる。

<1>1. 次の 2 つの型を取る。

      ```
      type Inner = unbox union { p : Array I64, q : I64 };
      type Outer = unbox union { a : Inner, b : Array I64 };
      ```

      このとき `leaves(Outer) = {[0, 0], [1]}` かつ `units(Outer) = {[]}` である。
  `boxed_leaf_paths` の `go` は、`is_fully_unboxed` でも `is_closure` でも `is_box` でも `is_array` でも
  ない型について `unpunched_field_types` の下へ降りる。`Outer` はその 4 つのどれでもないので変位
  `a : Inner` と `b : Array I64` へ降り、`Inner` も 4 つのどれでもないので変位 `p : Array I64` と
  `q : I64` へ降りる。`Array I64` は `is_array` で自分の path を積み、`I64` は `is_fully_unboxed` で
  何も積まない。`unit_step` は `is_union` で `UnitStep::Unit` を返すので、L3 より `units(Outer) = [[]]`
  である。
  BY <ref id=f221813/>, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step

<1>2. 次の関数だけを `funcs` に持ち、`globals` が空である入力プログラムを取る。`z : Array I64` は
      `f` の唯一のパラメータであり、`w : I64` は定数を束縛する。`f.borrowed_units` は空とする。

      ```
      f(z) = Let(x, Llvm(union_make_1, [z]),          // x : Outer
               Release(x, [],
                 Let(w, Llvm(int_lit_0, []),
                   Ret(w))))
      ```

      この入力プログラムは A1 と A2 を満たす。
  A2: `Release` の path `[]` は `<1>1` より `units(Outer)` の元である。
  A1 の後半 (すべての関数の `borrowed_units` が空であること): `f` をそう取った。
  A1 の前半 (D12): `globals` は空で `funcs` は `f` だけなので、示すのは `f` の本体が D11 を満たすことで
  ある。`f.borrowed_units` が空なので D14 よりすべてのパラメータ unit が所有される。`obj(z, [])` が
  グローバル状態 (D26) である活性化では、`z` の leaf は D8 の意味の参照を持たないので義務集合は空で
  始まり、`Retain`/`Release` も生成も消費もそれを動かさず、(S-a) と (S-b) は成り立つ。以下は
  `obj(z, [])` が計数下である活性化を見る。活性化の初期の
  義務集合は `z` の leaf `[]` の参照 1 つである。`union_make_1` の宣言は結果の leaf `[1]` を単一の
  `Arg(0, [])` とするので、D9 の移動の表の `Llvm` の素通し leaf の行により、その参照は `x` の
  leaf `[1]` へ移る。`x` の変位は 1 なので `x` の leaf `[0, 0]` は inhabited でない (D16)。
  `Release(x, [])` は `[]` の下の inhabited な leaf、すなわち `[1]` の参照を 1 つ取り除くので、義務集合は
  空になる。`Ret(w)` の `w` は `I64` で boxed leaf を持たないので消費は無い。
  (S-c) について。この本体に現れる D7 の読む構文は 2 つの `Llvm` の `Let` である。`int_lit_0` は
  オペランドを持たないので読むオブジェクトが無い。`union_make_1` はオペランド `z` を読み、その inhabited な
  leaf は `[]` 1 つである。`Release(x, [])` が触れるのは `obj(x, [1])` であり、素通しの行より
  `obj(z, [])` と同じオブジェクトである。`obj(z, [])` がグローバル状態であれば A8 よりそれは解放されない。
  計数下であれば L23 による。`O = obj(z, [])` と置き、その別名類を取る -- D20 の別名の辺のうち
  この本体に在るのは `Llvm` の素通し leaf の辺 (`z` の leaf `[]` から `x` の leaf `[1]` へ) だけであり、
  `z` はパラメータなので `VarTable::of` は `vars.bindings` に `z` の `Binding::Param` を入れ、D33 が
  歩みを止める位置の第 1 の行がそれである。よってスロット `(z, [])` と `(x, [1])` が 1 つの別名類 `C` を
  なし、その `ρ` 終端は `(z, [])`、`obj(C) = obj(z, []) = O` である。この活性化のスロットはこの 2 つで
  尽きる -- 値を得る変数は `z`・`x`・`w` の 3 つで、`x` の値の変位は 1 なので leaf `[0, 0]` は
  inhabited でなく (D16)、`w` の型 `I64` は boxed leaf を持たない -- ので、`obj(C') = O` である計数下の
  別名類は `C` のほかに無く、L23 の (H-b) は空に成り立つ。
  L23 の (H-a): `z` は所有するパラメータなので、D34 の第 2 行より `C` の `held` は 1 から始まり、その
  `ρ` 終端は借用する leaf ではない。D34 よりその開始値が置かれるのは `C` の終端の参照が `Obl` に入る
  素動作の直後の段内の点、すなわち活性化を作る段の中の点なので、以下の 2 つの点はどちらもそれより後で
  ある。この本体で `held` を動かす事象は `Release(x, [])`
  だけである -- `Retain` 節点は無く、D9 の消費のうちこの本体に在るのは `Llvm` のオペランドの行と終端の
  `Ret` の行だけで、`z` の唯一の leaf は単一の `Arg(0, [])` として素通しと宣言されているので消費されず、
  `Ret(w)` の `w` は boxed leaf を持たない。D34 より第 5 行の事象が `held` を動かすのはそれを運ぶ素動作の
  直後の段内の点なので、`union_make_1` が `z` を読む直前の点でも `Release(x, [])` が触れる直前の点でも
  `held_ρ(・, C) = 1` である。
  L23 の (H-c): (S-c) は各時点について、「その活性化がその時点まで解放について閉じている」
  (D11a) ことを条件に主張する節なので、その位置の (S-c) を示しているこの議論は接頭条件を与件に持つ。
  BY <ref id=596a46d/>, <ref id=56c2068/>, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=95427eb/>, <ref id=859cf84/>, <ref id=3d96eb8/>, <ref id=ef8efc4/>, <ref id=66c9670/>, <ref id=9c7c27a/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=e11772a/>, <ref id=b6673ca/>, <ref id=5d2d9a7/>, <1>1,
     CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov, InlineLLVMIntLit

<1>2a. `origin(x, [])` は `Exactly((z, []))` である。
  `[]` は `Outer` の leaf でないので `leaf_origins_at([])` は `None` を返し、`origin_from_leaves_under`
  に入る。`leaf_origins_under([])` は leaf `[0, 0]` の空集合と leaf `[1]` の `{Arg(0, [])}` を与える
  (`union_make_1` の宣言は、path の先頭が変位番号 1 に等しい leaf に `Arg(0, rest)` を、他の変位の leaf に
  `Set::default()` を置く)。空集合は `operand_units` にも `produced_here` にも寄与しない。
  `operand_units = {(0, trunc(Array I64, []))} = {(0, [])}` であり、`reached = [origin(z, [])]` である。
  `z` はパラメータなので `origin(z, [])` は `Exactly((z, []))` であり、`reached` は 1 元なので
  `reached.iter().all(|o| o == first)` は真になり、その `Origin` がそのまま返る。
  BY CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov,
     CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under, as_arg_projection,
     truncate_to_unit

<1>2b. `VarPath` の集合 `S` が `(z, [])` を含まないとき、`owns_object_yet(vars, type_env, z, [], S)` は
       偽である。
  `pty(z) = Some(Array I64)` なので `owns_object_yet` は `units_under(Array I64, [])` の各 unit について
  「同じ鍵を持つ所有された leaf」を要求する。`unit_step(Array I64)` は `is_array` の行で
  `UnitStep::Unit` を返すので、`sub(Array I64, []) = Some(Array I64)` であり L3 より
  `units(Array I64) = [[]]`、L2 より `under(Array I64, []) = [[]]` である。鍵は
  `trunc(Array I64, []) = []` (空の path なのでループに入らない) であり、`leaves(Array I64) = {[]}` なので
  要求されるのは `(z, []) ∈ S` である。
  BY <ref id=9f11796/>, <ref id=f221813/>, CODE src/rc_ir/borrow.rs: owns_object_yet,
     CODE src/rc_ir/ownership.rs: unit_step, subtree_type, units_under, truncate_to_unit,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2c. `<1>2` の入力プログラムは、第 4 節の仮定のうち入力プログラムを縛る残りのものをすべて満たす。
  **この節の仮定とは、README 第 4 節の `A<番号>` という見出しを持つ項目の全体である。** 下の表がその
  全体を 1 行ずつ挙げる。**表の行の集合は `dev-docs/proof/proof_index.py` が README の項目と突き合わせる**
  ので、枠に仮定が 1 つ足されたときに落ちた行が出る。
  **どの仮定が入力プログラムを縛るかは、次の述語で決まる** -- **その仮定の言明の真偽が、`borrow_ify` の
  入力プログラムをどう取るかで動きうるか。** 動きうるものはこの構成について確かめる。動かないものは
  どう取っても真偽が同じなので、この構成が果たすべきものを持たない。表の「主語はこの構成が選ばない」の
  行がそれである。

| 仮定 | この構成について |
|---|---|
| A1 | `<1>2` が与えた |
| A2 | `<1>2` が与えた |
| A3 | 主語は `LLVMGen` の宣言であり、この構成はそれを選ばない |
| A4 | 主語はコード生成であり、この構成はそれを選ばない |
| A5 | 主語は値と leaf の模型であり、この構成はそれを選ばない |
| A6 | 束縛名は `z`・`x`・`w` で互いに異なり、関数の名前 `f` とも異なる |
| A7 | 主語は `resolve_callee_params` であり、この構成はそれを選ばない |
| A8 | 主語は `mark_global` であり、この構成はそれを選ばない |
| A9 | この本体に `Match` は無い |
| A10 | 現れる型は `Array I64`・`I64`・`Inner`・`Outer` である。どれも型変数を持たない ground な型で、その tycon に kind の要求するだけの引数が与えられ、tycon は `type_env` にある。`unpunched_field_types` の歩みは `Outer` から `Inner` と `Array I64` へ、`Inner` から `Array I64` と `I64` へ降りて終わるので有限であり、`no_size_in_place` の in-place の降下はその部分である。newtype を 1 つも宣言していないので、`instance_field_types` の newtype の展開は恒等である |
| A11 | 本体の自由な局所名は `z` だけで、それは `f` の唯一のパラメータである。`x` の使用は直前の `Let(x, ..)` の、`w` の使用は直前の `Let(w, ..)` のスコープに入っている |
| A12 | この本体の構文は `Llvm` の `Let` 2 つと `Release` と `Ret` だけである。`union_make_1` の `args` は `[z]` で `gen.free_vars()` に等しく、`ty(x) = Outer` はその第 1 変位の payload の型を `ty(z) = Array I64` に取る `make_union` の結果の型である。`int_lit_0` はオペランドを持たず `ty(w) = I64` である。同じ名前の `RcVar` はこの本体に 1 つずつしか現れないので型は一致し、束縛を持たない `RcVar` は無い。A12 の残りの節が名指す構文 -- move-bind、`Match`、`Destructure`、`App`、`InlineLLVMStructPunchBody` などの op -- はこの本体に無いので、それらは空に成り立つ |
| A13 | `f`・`z`・`x`・`w` はどれも `#` を含まないので、`#` で区切った最後の断片は名前自身であり、`b` の後に 10 進数字が続く形でも `borrow` でもない |
| A14 | この本体に `App` は無い |
| A15 | 主語は `stacker` crate であり、この構成はそれを選ばない |
| A16 | この本体に `Match` は無い |
| A17 | 主語は環境のコードであり、この構成はそれを選ばない |
| A18 | 下の **A18 について** の段落が確かめる |
| A19 | 下の **A19 について** の段落が確かめる |
| A20 | この本体に `App` は無く、`f` を呼ぶ本体もこのプログラムに無いので、この節は空に成り立つ |
| A21 | 主語は `builtin.rs` の op の集合であり、この構成はそれを選ばない |
| A22 | `funcs` の唯一の鍵は `f` であり、その `RcFunc` の `name` に等しい |
| A23 | この構成は RC IR のプログラムを直に据えるので、`Lowerer::lower_to_var` に着く `Expr::Lam` の節点も、`Lowerer::lower_lam` が `funcs` に入れる関数も無い。A23 の 2 つの節はその 2 つを主語にするので、空に成り立つ |
| A24 | この本体に `InlineLLVMFixBody` の `Llvm` 節点は無い |
| A25 | 主語は `insert_rc` の入力プログラムであり、この構成はそれを選ばない |
| A26 | 主語はコード生成であり、この構成はそれを選ばない |
| A26a | この本体に `FFI_CALL` の `Llvm` 節点は無く (現れる op は `union_make_1` と `int_lit_0` である)、`globals` も空である (`<1>2`)。よってこの条件は空に成り立つ |
| A27 | 主語は状態の欄を書く生成コードであり、この構成はそれを選ばない |
| A28 | この構成が型宣言として置くのは `Inner` と `Outer` であり、どちらも union なので punched の鍵も作らない。どちらの名前も `#FunPtr` で始まらないので、A28 が主語にする項目は `bulitin_tycons()` が置いたもののままである |
| A29 | 主語は `LLVMGen::applies_a_function_operand` を override する項目であり、この構成はそれを置かない |
| A30 | 主語はコード生成であり、この構成はそれを選ばない |
| A31 | この構成は `Program::symbols` を組み立てないので、その不変条件は主語を持たない |
| A32 | この構成が置く関数の名前とグローバルの記号は相異なる |

  **A18 について。** (b) は空に成り立つ -- オブジェクトがグローバル状態になるのは `mark_global` が
  印を付ける (E5) の段だけであり (D26)、(E5) が走るのはグローバル初期化子の活性化が終端の `Ret` に着くときなので (D24)、
  初期化子を 1 つも持たないこのプログラムの実行に (E5) の段は無い。よってグローバル状態のオブジェクトは
  1 つも無い。
  (a) について。このプログラムのどの実行でも、生きている (D25) オブジェクトはどの点でも 1 つも無い。
  素動作の列についての帰納で見る。最初の時点について -- 環境が持ち込む boxed な値は A17 の (i-d) より
  「このプログラムが作って環境へ番地を渡したもの」に限るが、番地を渡す op
  (`boxed_to_retained_ptr` の `InlineLLVMBoxedToRetainedPtrIOS`) はこの本体に無いので 1 つも無く、
  D25 よりその時点に生きているオブジェクトは無い。ある点まで生きているオブジェクトが 1 つも無いとする。
  `create_obj` を呼ぶのは `Let(x, RcRhs::Llvm(llvm_gen, args), k)` の腕 (`llvm_gen.generate` を経て) と
  `Closure` 節点を作る段だけであり、`Ret`・`Retain`・`Release` の腕は呼ばない
  (`CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner`)。この本体に `Closure` 節点は無く、`Llvm`
  の `Let` は `union_make_1` と `int_lit_0` の 2 つである。`InlineLLVMMakeUnionBody::generate` は結果の型
  `Outer` について、`InlineLLVMIntLit::generate` は結果の型 `I64` について `create_obj` を呼ぶ
  (`CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::generate, InlineLLVMIntLit::generate`)。`Outer`
  は unbox union と宣言されているので `ty_to_object_ty` の `Union` の枝が `object_type.is_unbox` に
  `TyConInfo` の `is_unbox` (真) をそのまま写し、`I64` はプリミティブなので同じ関数の `Primitive` の枝が
  `is_unbox` の真を課す (`CODE src/object.rs: ty_to_object_ty`)。よってどちらの呼び出しも
  `object_type.is_unbox` が真の枝を取り、`create_obj` は割り当てを行わず undef の集約を返す
  (`CODE src/object.rs: create_obj`)。したがってこの本体のどの素動作もオブジェクトを割り当てない。
  子の活性化を作る段も無い -- `globals` が空なので (E5) と (E7) の段は無く、この本体に `App` が無いので
  (E3) の段も無く、`union_make_1` と `int_lit_0` は `applies_a_function_operand` を宣言しないので (A3)
  オペランドを適用する `Llvm` の段も無く、(F) の解放が活性化を作るのは `Destructor` のオブジェクトに
  ついてだが、その点まで生きているオブジェクトが無い。環境の書き込みの段 (E8) はどの leaf がどの
  オブジェクトを指すかを変えない (A17 の (ii-b))。よって次の素動作もオブジェクトを作らない。
  したがって生きているオブジェクトのグラフはどの点でも空であり、非巡回である。
  この 2 つは `<1>2` と食い違わない。A18 が主語にするのは実行 (D24) のヒープであるのに対し、`<1>2` は
  D11 の 3 つの節を D21 の意味のすべての活性化について確かめており、その活性化は実行に実現するとは
  限らない (D21)。
  **A19 について。** `obj(z, [])` が計数下である活性化の計数下の別名類は 1 つである -- D20 の別名の辺の
  うちこの本体に在るのは `Llvm` の素通し leaf の辺 (`z` の leaf `[]` から `x` の leaf `[1]` へ) だけなので、
  スロット `(z, [])` と `(x, [1])` が 1 つの類 `C` をなす。`z` は所有するパラメータなので D34 より `C` の `held` は
  1 から始まり、`Release(x, [])` が 1 引いて 0 になる。(ii-a): `held` は各時点で 0 以上であり、この類を
  名指す構文 -- `union_make_1` の読みと `Release(x, [])` -- の時点ではどちらも 1 である。終端の `Ret(w)` の
  消費の直後も 0 である。(ii-b): この本体に `Retain` は無いので `bumps` はどの時点でも 0 であり、条件は
  空に成り立つ。(ii-c): 表 (D34) がこの類の `held` を動かす段内の点は `Release(x, [])` を運ぶ素動作の
  直後だけであり、それ以外の段内の点は直前の置き場所の値のままである (D34)。したがってこの本体のどの
  段内の点でも `held` は 1 (その素動作より前) か 0 (その素動作以後) のいずれかであり、どちらも 0 以上で
  ある。(i): これは仮定ではなく D21 が活性化に課す制限であり、D11 と D12 が条件を課す活性化は
  各時点と各段内の点でその不等式を満たすものに限られる (D21)。よってこの本体の側に果たすべきものは無い。
  `obj(z, [])` がグローバル状態である活性化では計数下の別名類が 1 つも無いので、(ii-a)・(ii-b)・(ii-c) は
  空に成り立つ。`borrow_ify` がこの本体を写した本体は、束縛名を付け替え (P9)、
  `RewriteCtx::rewrite` を掛けたものである。`rewrite_inner` が `Retain`/`Release` の腕で呼ぶ
  `rewrite_rc` は、その節点を `units_under(ty(v), path)` のうち `owns_unit` が真である unit の節点の列に
  置き換える。A2 と L6 より `under(ty(x), []) = [[]]` なので、写した本体は元の本体か、そこから
  唯一の `Release(x, [])` を落としたものである。落とした本体では `held` は 1 のまま終端に着き、`bumps` は
  0 のままなので、(ii-a)・(ii-b)・(ii-c) はやはり成り立つ (段内の点は 1 のまま動かない)。
  BY <1>1, <1>2, <1>2a, <ref id=627e117/>, <ref id=3f1bb47/>, <ref id=4f63121/>, <ref id=33c54dc/>, <ref id=4517a7a/>, <ref id=b6673ca/>, <ref id=1172c08/>, <ref id=8412761/>, <ref id=3905b4e/>, <ref id=83d98e9/>, <ref id=cb35ab1/>, <ref id=f8ae607/>, <ref id=3e6b0e0/>, <ref id=f769887/>, <ref id=680aaa9/>, <ref id=ebec376/>,
     <ref id=8d3e4af/>, <ref id=675b350/>, <ref id=d80dde9/>, <ref id=fd95f12/>, <ref id=0ab1ef4/>, <ref id=3d4be43/>, <ref id=8e3aff3/>, <ref id=e11772a/>, <ref id=c9e4cca/>, <ref id=5f74a79/>, <ref id=9f1cf6c/>, <ref id=3647480/>, <ref id=29a890a/>, <ref id=a502f3e/>, <ref id=0594f24/>, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=9c7c27a/>, <ref id=c232680/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=88a06de/>,
     <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=e74af85/>, <ref id=63eadd9/>,
     CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::rewrite_rc,
     CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
     CODE src/fixstd/builtin.rs: boxed_to_retained_ptr_ios, InlineLLVMMakeUnionBody::generate,
     InlineLLVMIntLit::generate, InlineLLVMBoxedToRetainedPtrIOS,
     CODE src/object.rs: create_obj, ty_to_object_ty

<1>3. `infer_ownership` の不動点で `owned_leaves` は空である。
  <2>1. `collect_consumes` はこの本体について何も報告しない。
    `Ret(w)` は `w` の boxed leaf を積むが `I64` は boxed leaf を持たない。`Release` は継続へ降りるだけで
    ある。`Let(x, Llvm(union_make_1, [z]), ..)` については `rhs_consumes` の `RcRhs::Llvm` の腕が働く。
    `InlineLLVMMakeUnionBody` は `borrows_operand` を override しないので既定の偽であり、`z` の各 boxed
    leaf のうち `passthrough` に無いものが積まれる。`passthrough_arg_leaves` は宣言の leaf のうち単一の
    `Arg` であるものを集めるので、`union_make_1` については `{(0, [])}` である。`leaves(Array I64) = {[]}`
    なので `z` の唯一の leaf は `passthrough` に在り、積まれない。`Let(w, Llvm(int_lit_0, []), ..)` は
    オペランドを持たない。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go, rhs_consumes, passthrough_arg_leaves,
       CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand,
       CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov
  <2>2. `levelled_sites(f)` は `(x, [])` だけを挙げる。
    本体に `App` は無く、`Retain`/`Release` は `Release(x, [])` だけである。
    BY CODE src/rc_ir/borrow.rs: levelled_sites
  <2>3. `infer_ownership` が `owned_leaves` に元を加えるのは 2 か所だけである -- `collect_consumes` が
        報告した各 `(var, path)` の `origin` の候補のうち `param_tys` の鍵を根とするものと、
        `level_ownership` が `covered_leaves` から入れる leaf である。
    BY CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership

  <2>4. `level_ownership` は site `(x, [])` で `false` を返し続ける。
    `<1>2a` より候補は `(z, [])` 1 つである。`OL` が空ならば `<1>2b` より `owns_object_yet` は偽であり、
    `owns_a_candidate` は候補についての `any` なので偽になって `level_ownership` は `false` を返す。
    BY <1>2a, <1>2b, CODE src/rc_ir/borrow.rs: level_ownership
  <2>5. QED
    `infer_ownership` は空の `owned_leaves` から始める。`<2>3` の 2 か所のうち、1 つ目は `<2>1` より
    報告が無いので何も加えず、2 つ目は `<2>2` の唯一の site について `<2>4` より発火しない。よって
    最初の周で `changed` は偽になり、不動点は空集合である。
    BY <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/borrow.rs: infer_ownership

<1>4. `f` は借用版を持ち、その借用版で `owns_object(ρ(z), [])` は偽である。
  `func_has_borrowable_param` は、あるパラメータのある leaf が `owned_leaves` に無いことを問う。`<1>3` より
  `z` の leaf `[]` は無い。`f` は capture を持たない。`funcs_observing_uniqueness` が `observing` に
  名前を入れるのは 2 か所である。走査が `llvm_gen.observes_uniqueness()` の真な `Llvm` の右辺を見つけた
  ときと、最後の不動点の繰り返しが「`observing` の元への辺を持つ関数」を足すときである。後者は
  `observing` が空ならば何も足さないので、`observing` は走査が 1 つも入れなければ空のまま返る。この
  プログラムの本体に現れる op は `InlineLLVMMakeUnionBody` と `InlineLLVMIntLit` の 2 つで、どちらも
  `observes_uniqueness` を override しないので既定の偽である。よって `observing` は空であり、
  `funcs_observing_uniqueness` は `f` を含まない。**この議論は辺の張り方に依らない** -- `callees` に
  どの辺が入っても、`observing` が空である限り不動点は空である。よって `borrow_versions` は `f` を持つ。
  `borrow_ify` が借用版の `owned_units` に入れるのは `owned_leaves` が所有する leaf の像だけなので、
  `(ρ(z), [])` は `owned_units` に無い。L16 と `<1>2b` より `owns_object(ρ(z), [])` は偽である。
  BY <1>2, <1>2b, <1>3, <ref id=90c9817/>, CODE src/rc_ir/borrow.rs: func_has_borrowable_param,
     funcs_observing_uniqueness, borrow_ify,
     CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness,
     CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody, InlineLLVMIntLit

<1>5. 節 1 は偽である。
  `<1>2a` より `origin(vars_f, x, []) = Exactly((z, []))` であり、L15 より借用版の
  `origin(vars_c, ρ(x), [])` はその `ρ` による像 `Exactly((ρ(z), []))` である。すなわち借用版の site
  `(ρ(x), [])` の候補は `{(ρ(z), [])}` である。`owns_unit` はその全称なので、`<1>4` より偽である。
  BY <1>2a, <1>4, <ref id=c3b2aa3/>, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>6. `Λ([])` の leaf `[0, 0]` のすべての候補について `owns_object` は真である。すなわち、節 2 から
      inhabited の限定を外した形はこの site で真である。
  `<1>5` よりこの site は `(ρ(x), [])` であり、`rename_var` は型を残すので `ty(ρ(x)) = Outer`、`<1>1` より
  `Λ([]) = leaves(Outer) = {[0, 0], [1]}` である。`λ = [0, 0]` を取る。
  まず入力の側の `origin(vars_f, x, [0, 0])` を求める。`Outer` の leaf `[0, 0]` の宣言は
  空集合なので、`leaf_origins_at([0, 0])` は空集合を持つ `Some` を返し、`as_arg_projection` は要素数が 1 で
  ない集合に `None` を返す。よって `origin_from_leaves_under` に入り、その `leaf_origins_under([0, 0])` は
  その空集合 1 つだけを与えるので `operand_units` は空、`produced_here` は偽、`reached` は空になり、
  `reached.first()?` が `None` を返して `unwrap_or_else(here)` が `Exactly((x, [0, 0]))` を返す。
  `x` は `func` に現れる名前 (DEF 現れる名前) なので L15 が使えて、借用版の
  `origin(vars_c, ρ(x), [0, 0])` はその `ρ` による像 `Exactly((ρ(x), [0, 0]))` である。すなわちこの leaf の
  候補は `{(ρ(x), [0, 0])}` 1 つである。
  `func` のパラメータは `z` だけなので L1c より `pty_f(x)` は値を返さず、`owns_object_yet` は
  `param_tys.get(root)` が `None` のとき真を返すので `yet_f(x, [0, 0])` は真である。L16 より
  `ctx.owns_object(ρ(x), [0, 0])` も真である。よって `λ = [0, 0]` のすべての候補について `owns_object` は
  真である。
  BY <1>1, <1>5, <ref id=fa1a6ce/>, <ref id=c3b2aa3/>, <ref id=90c9817/>, DEF 現れる名前,
     CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov,
     CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under, as_arg_projection,
     CODE src/rc_ir/borrow.rs: owns_object_yet, CODE src/rc_ir/rename.rs: rename_var

<1>7. leaf `[0, 0]` はどの活性化でも inhabited にならない。
  `union_make_1` は `x` の値の変位を 1 にする。leaf `[0, 0]` が `Outer` の根の unbox union の節で選ぶ
  変位番号は 0 なので、D16 の条件はどの時点でも成り立たない。A3 の表の第 1 行も、空集合と宣言された
  leaf が inhabited にならないと述べる。
  BY <ref id=e11772a/>, <ref id=66c9670/>, CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov

<1>8. QED
  `<1>2` と `<1>2c` より、この入力プログラムは第 4 節の仮定のうち入力プログラムを縛るものをすべて
  満たす。`<1>5` と `<1>6` より、この site で節 1 は偽でありながら leaf `[0, 0]` のすべての候補は
  所有される。`<1>7` よりその leaf は inhabited でない。
  BY <1>2, <1>2c, <1>5, <1>6, <1>7

### R2 (inhabited な leaf が無い site では、節 3 から節 2 と節 1 へ渡れない)

**言明**。第 4 節の仮定のうち入力プログラムを縛るものをすべて満たす入力プログラムであって、その出力の
ある版 `V` のある site (DEF site) と `infer_ownership` の不動点について、`Post(V)` のある活性化のある
位置で `Inh(v, u) = ∅` かつ節 1 が偽であるものがある。そこでは節 3 は空虚に真であり、節 2 と節 1 は偽である。

<1>1. 型 `Mix = unbox union { n : I64, a : Array I64 }` について
      `leaves(Mix) = {[1]}` かつ `units(Mix) = {[]}` である。
  `boxed_leaf_paths` の `go` は `Mix` について `is_fully_unboxed` (変位 `a` が `Array I64` なので偽)、
  `is_closure`、`is_box` (unbox なので偽)、`is_array` のどれでもないので `unpunched_field_types` の下へ
  降り、`I64` からは何も、`Array I64` からは `[1]` を積む。`unit_step` は `is_union` で `UnitStep::Unit`
  を返すので L3 より `units(Mix) = [[]]` である。
  BY <ref id=f221813/>, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step

<1>2. 次の関数だけを `funcs` に持ち、`globals` が空である入力プログラムを取る。`x : Mix` は `f` の唯一の
      パラメータ、`w : I64` は定数を束縛する。`f.borrowed_units` は空とする。

      ```
      f(x) = Release(x, [],
               Let(w, Llvm(int_lit_0, []),
                 Ret(w)))
      ```

      この入力プログラムは A1 と A2 を満たす。
  A2: `Release` の path `[]` は `<1>1` より `units(Mix)` の元である。
  A1 の後半 (すべての関数の `borrowed_units` が空であること): `f` をそう取った。
  A1 の前半 (D12): `globals` は空で `funcs` は `f` だけなので、示すのは `f` の本体が D11 を満たすことで
  ある。`f.borrowed_units` が空なので D14 よりすべてのパラメータ unit が所有され、活性化の初期の
  義務集合は `x` の inhabited かつ計数下 (D26) な leaf の参照である (D10)。タグが `n` (変位 0) の活性化では
  `Λ_{Mix}([]) = {[1]}` の唯一の leaf が変位 1 を選ぶので inhabited でなく (D16)、義務集合は空である。
  `Release(x, [])` は inhabited な leaf の参照を取り除くので何も取り除かず、(S-a) と (S-b) が成り立つ。
  タグが `a` (変位 1) の活性化で `obj(x, [1])` がグローバル状態であるときも、その leaf は D8 の意味の
  参照を持たないので (D26) 義務集合は空のままであり、同じく (S-a) と (S-b) が成り立つ。
  タグが `a` で `obj(x, [1])` が計数下であれば義務集合は leaf `[1]` の参照 1 つであり、
  `Release(x, [])` がそれを
  取り除くので、(S-a) と (S-b) が成り立つ。
  `Ret(w)` の `w` は `I64` で boxed leaf を持たないので消費は無い。
  (S-c) について。この本体に現れる D7 の読む構文は `Let(w, Llvm(int_lit_0, []), ..)` だけであり
  (`Release` と `Ret` は読む構文ではない)、D7 の `Llvm` の行が読むのは各オペランドだが `int_lit_0` は
  オペランドを持たないので、読まれるオブジェクトは無い。`Release(x, [])` が触れるのは、タグが `n` の
  活性化では inhabited な leaf が無いので何も無く、タグが `a` の活性化では `obj(x, [1])` である。後者が
  グローバル状態であれば A8 よりそれは解放されない。計数下であれば L23 による。`O = obj(x, [1])` と
  置き、その別名類を取る -- D20 の別名の辺はこの本体に 1 つも無く、`x` はパラメータなので
  `VarTable::of` は `vars.bindings` に `x` の `Binding::Param` を入れ、D33 が歩みを止める位置の第 1 の
  行がそれである。よってスロット `(x, [1])` は 1 つだけからなる別名類 `C` をなし、その `ρ` 終端は
  `(x, [1])` 自身であって `obj(C) = O` である。この活性化のスロットはこの 1 つで尽きる -- 値を得る変数は
  `x` と `w` の 2 つで、`<1>1` より `Λ_{Mix}([]) = {[1]}`、`w` の型 `I64` は boxed leaf を持たない --
  ので、`obj(C') = O` である計数下の別名類は `C` のほかに無く、L23 の (H-b) は空に成り立つ。
  L23 の (H-a): `x` は所有するパラメータなので、D34 の第 2 行より `C` の `held` は 1 から始まり、その
  `ρ` 終端は借用する leaf ではない。D34 よりその開始値が置かれるのは `C` の終端の参照が `Obl` に入る
  素動作の直後の段内の点、すなわち活性化を作る段の中の点なので、以下の点はそれより後である。
  この本体で `held` を動かす事象は `Release(x, [])`
  だけである -- `Retain` 節点は無く、D9 の消費のうちこの本体に在るのは終端の `Ret` の行だけで、`Ret(w)`
  の `w` は boxed leaf を持たない。D34 より第 5 行の事象が `held` を動かすのはそれを運ぶ素動作の直後の
  段内の点なので、`Release(x, [])` が触れる直前の点で `held_ρ(・, C) = 1` である。
  L23 の (H-c): (S-c) は各時点について、
  「その活性化がその時点まで解放について閉じている」(D11a) ことを条件に主張する節なので、その位置の
  (S-c) を示しているこの議論は接頭条件を与件に持つ。
  BY <ref id=596a46d/>, <ref id=56c2068/>, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=95427eb/>, <ref id=859cf84/>, <ref id=3d96eb8/>, <ref id=ef8efc4/>, <ref id=66c9670/>, <ref id=9c7c27a/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=b6673ca/>, <ref id=5d2d9a7/>, <1>1,
     CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/fixstd/builtin.rs: InlineLLVMIntLit

<1>2a. `<1>2` の入力プログラムは、第 4 節の仮定のうち入力プログラムを縛る残りのものをすべて満たす。
  **この節の仮定とは、README 第 4 節の `A<番号>` という見出しを持つ項目の全体である。** 下の表がその
  全体を 1 行ずつ挙げる。**表の行の集合は `dev-docs/proof/proof_index.py` が README の項目と突き合わせる**
  ので、枠に仮定が 1 つ足されたときに落ちた行が出る。
  **どの仮定が入力プログラムを縛るかは、次の述語で決まる** -- **その仮定の言明の真偽が、`borrow_ify` の
  入力プログラムをどう取るかで動きうるか。** 動きうるものはこの構成について確かめる。動かないものは
  どう取っても真偽が同じなので、この構成が果たすべきものを持たない。表の「主語はこの構成が選ばない」の
  行がそれである。

| 仮定 | この構成について |
|---|---|
| A1 | `<1>2` が与えた |
| A2 | `<1>2` が与えた |
| A3 | 主語は `LLVMGen` の宣言であり、この構成はそれを選ばない |
| A4 | 主語はコード生成であり、この構成はそれを選ばない |
| A5 | 主語は値と leaf の模型であり、この構成はそれを選ばない |
| A6 | 束縛名は `x` と `w` で互いに異なり、関数の名前 `f` とも異なる |
| A7 | 主語は `resolve_callee_params` であり、この構成はそれを選ばない |
| A8 | 主語は `mark_global` であり、この構成はそれを選ばない |
| A9 | この本体に `Match` は無い |
| A10 | 現れる型は `Mix`・`I64`・`Array I64` である。どれも型変数を持たない ground な型で、その tycon に kind の要求するだけの引数が与えられ、tycon は `type_env` にある。`unpunched_field_types` の歩みは `Mix` から `I64` と `Array I64` へ降りて終わるので有限であり、`no_size_in_place` の in-place の降下はその部分である。newtype を 1 つも宣言していないので、`instance_field_types` の newtype の展開は恒等である |
| A11 | 本体の自由な局所名は `x` だけで、それは `f` の唯一のパラメータである。`w` の使用は直前の `Let(w, ..)` のスコープに入っている |
| A12 | この本体の構文は `Release` と `Llvm` の `Let` と `Ret` だけである。`int_lit_0` はオペランドを持たないので `args` は空で `gen.free_vars()` に等しく、`ty(w) = I64` である。同じ名前の `RcVar` はこの本体に 1 つずつしか現れないので型は一致し、束縛を持たない `RcVar` は無い。A12 の残りの節が名指す構文 -- move-bind、`Match`、`Destructure`、`App`、`InlineLLVMStructPunchBody` などの op -- はこの本体に無いので、それらは空に成り立つ |
| A13 | `f`・`x`・`w` はどれも `#` を含まないので、`#` で区切った最後の断片は名前自身であり、`b` の後に 10 進数字が続く形でも `borrow` でもない |
| A14 | この本体に `App` は無い |
| A15 | 主語は `stacker` crate であり、この構成はそれを選ばない |
| A16 | この本体に `Match` は無い |
| A17 | 主語は環境のコードであり、この構成はそれを選ばない |
| A18 | 下の **A18 について** の段落が確かめる |
| A19 | 下の **A19 について** の段落が確かめる |
| A20 | この本体に `App` は無く、`f` を呼ぶ本体もこのプログラムに無いので、この節は空に成り立つ |
| A21 | 主語は `builtin.rs` の op の集合であり、この構成はそれを選ばない |
| A22 | `funcs` の唯一の鍵は `f` であり、その `RcFunc` の `name` に等しい |
| A23 | この構成は RC IR のプログラムを直に据えるので、`Lowerer::lower_to_var` に着く `Expr::Lam` の節点も、`Lowerer::lower_lam` が `funcs` に入れる関数も無い。A23 の 2 つの節はその 2 つを主語にするので、空に成り立つ |
| A24 | この本体に `InlineLLVMFixBody` の `Llvm` 節点は無い |
| A25 | 主語は `insert_rc` の入力プログラムであり、この構成はそれを選ばない |
| A26 | 主語はコード生成であり、この構成はそれを選ばない |
| A26a | この本体に `FFI_CALL` の `Llvm` 節点は無く (現れる op は `int_lit_0` である)、`globals` も空である (`<1>2`)。よってこの条件は空に成り立つ |
| A27 | 主語は状態の欄を書く生成コードであり、この構成はそれを選ばない |
| A28 | この構成が型宣言として置くのは `Mix` であり、union なので punched の鍵も作らない。その名前は `#FunPtr` で始まらないので、A28 が主語にする項目は `bulitin_tycons()` が置いたもののままである |
| A29 | 主語は `LLVMGen::applies_a_function_operand` を override する項目であり、この構成はそれを置かない |
| A30 | 主語はコード生成であり、この構成はそれを選ばない |
| A31 | この構成は `Program::symbols` を組み立てないので、その不変条件は主語を持たない |
| A32 | この構成が置く関数の名前とグローバルの記号は相異なる |

  **A18 について。** (b) は空に成り立つ -- オブジェクトがグローバル状態になるのは `mark_global` が
  印を付ける (E5) の段だけであり (D26)、(E5) が走るのはグローバル初期化子の活性化が終端の `Ret` に着くときなので (D24)、
  初期化子を 1 つも持たないこのプログラムの実行に (E5) の段は無い。よってグローバル状態のオブジェクトは
  1 つも無い。
  (a) について。このプログラムのどの実行でも、生きている (D25) オブジェクトはどの点でも 1 つも無い。
  素動作の列についての帰納で見る。最初の時点について -- 環境が持ち込む boxed な値は A17 の (i-d) より
  「このプログラムが作って環境へ番地を渡したもの」に限るが、番地を渡す op
  (`boxed_to_retained_ptr` の `InlineLLVMBoxedToRetainedPtrIOS`) はこの本体に無いので 1 つも無く、
  D25 よりその時点に生きているオブジェクトは無い。ある点まで生きているオブジェクトが 1 つも無いとする。
  `create_obj` を呼ぶのは `Let(x, RcRhs::Llvm(llvm_gen, args), k)` の腕 (`llvm_gen.generate` を経て) と
  `Closure` 節点を作る段だけであり、`Ret`・`Retain`・`Release` の腕は呼ばない
  (`CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner`)。この本体に `Closure` 節点は無く、`Llvm`
  の `Let` は `int_lit_0` の 1 つである。`InlineLLVMIntLit::generate` は結果の型 `I64` について
  `create_obj` を呼ぶ (`CODE src/fixstd/builtin.rs: InlineLLVMIntLit::generate`)。`I64` はプリミティブ
  なので `ty_to_object_ty` の `Primitive` の枝が `object_type.is_unbox` の真を課す
  (`CODE src/object.rs: ty_to_object_ty`)。よってこの呼び出しは `object_type.is_unbox` が真の枝を取り、
  `create_obj` は割り当てを行わず undef の集約を返す (`CODE src/object.rs: create_obj`)。したがって
  この本体のどの素動作もオブジェクトを割り当てない。
  子の活性化を作る段も無い -- `globals` が空なので (E5) と (E7) の段は無く、この本体に `App` が無いので
  (E3) の段も無く、`int_lit_0` は `applies_a_function_operand` を宣言しないので (A3) オペランドを
  適用する `Llvm` の段も無く、(F) の解放が活性化を作るのは `Destructor` のオブジェクトについてだが、
  その点まで生きているオブジェクトが無い。環境の書き込みの段 (E8) はどの leaf がどのオブジェクトを
  指すかを変えない (A17 の (ii-b))。よって次の素動作もオブジェクトを作らない。
  したがって生きているオブジェクトのグラフはどの点でも空であり、非巡回である。
  この 2 つは `<1>2` と食い違わない。A18 が主語にするのは実行 (D24) のヒープであるのに対し、`<1>2` は
  D11 の 3 つの節を D21 の意味のすべての活性化について確かめており、その活性化は実行に実現するとは
  限らない (D21)。
  **A19 について。** D20 の別名の辺はこの本体に 1 つも無いので、計数下の別名類はスロット 1 つずつから
  なる。タグが `n` の活性化では inhabited な leaf が無いので類も無く、タグが `a` で `obj(x, [1])` がグローバル状態の活性化
  でも計数下の類が無いので、どちらでも (ii-a)・(ii-b)・(ii-c) は空に成り立つ。タグが `a` で `obj(x, [1])`
  が計数下の活性化では、類は `(x, [1])` の 1 つで、`x` は所有するパラメータなので D34 より `held` は 1 から
  始まり、`Release(x, [])` が 1 引いて 0 になる。(ii-a): `held` は各時点で 0 以上であり、この類を名指す
  構文は `Release(x, [])` だけで、その時点では 1 である。終端の `Ret(w)` の消費の直後も 0 である。
  (ii-b): この本体に `Retain` は無いので `bumps` はどの時点でも 0 であり、条件は空に成り立つ。(ii-c):
  表 (D34) がこの類の `held` を動かす段内の点は `Release(x, [])` を運ぶ素動作の直後だけであり、それ以外
  の段内の点は直前の置き場所の値のままである (D34)。したがってこの本体のどの段内の点でも `held` は 1
  (その素動作より前) か 0 (その素動作以後) のいずれかであり、どちらも 0 以上である。(i): これは
  仮定ではなく D21 が活性化に課す制限であり、D11 と D12 が条件を課す活性化は各時点と各段内の点で
  その不等式を満たすものに限られる (D21)。よってこの本体の側に果たすべきものは無い。
  `borrow_ify` がこの本体を写した本体は、束縛名を付け替え (P9)、
  `RewriteCtx::rewrite` を掛けたものである。`rewrite_inner` が `Retain`/`Release` の腕で呼ぶ
  `rewrite_rc` は、その節点を `units_under(ty(v), path)` のうち `owns_unit` が真である unit の節点の列に
  置き換える。A2 と L6 より `under(ty(x), []) = [[]]` なので、写した本体は元の本体か、そこから唯一の
  `Release(x, [])` を落としたものである。落とした本体では `held` は 1 のまま終端に着き、`bumps` は
  0 のままなので、(ii-a)・(ii-b)・(ii-c) はやはり成り立つ (段内の点は 1 のまま動かない)。
  BY <1>1, <1>2, <ref id=627e117/>, <ref id=3f1bb47/>, <ref id=4f63121/>, <ref id=33c54dc/>, <ref id=4517a7a/>, <ref id=b6673ca/>, <ref id=1172c08/>, <ref id=8412761/>, <ref id=3905b4e/>, <ref id=83d98e9/>, <ref id=cb35ab1/>, <ref id=f8ae607/>, <ref id=3e6b0e0/>, <ref id=f769887/>, <ref id=680aaa9/>, <ref id=ebec376/>,
     <ref id=8d3e4af/>, <ref id=675b350/>, <ref id=d80dde9/>, <ref id=fd95f12/>, <ref id=0ab1ef4/>, <ref id=3d4be43/>, <ref id=8e3aff3/>, <ref id=e11772a/>, <ref id=c9e4cca/>, <ref id=5f74a79/>, <ref id=9f1cf6c/>, <ref id=3647480/>, <ref id=29a890a/>, <ref id=a502f3e/>, <ref id=0594f24/>, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=66c9670/>, <ref id=9c7c27a/>, <ref id=c232680/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=88a06de/>,
     <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=e74af85/>, <ref id=63eadd9/>,
     CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::rewrite_rc,
     CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
     CODE src/fixstd/builtin.rs: boxed_to_retained_ptr_ios, InlineLLVMIntLit::generate,
     InlineLLVMBoxedToRetainedPtrIOS,
     CODE src/object.rs: create_obj, ty_to_object_ty

<1>3. `infer_ownership` の不動点で `owned_leaves` は空であり、`f` は借用版を持ち、そこで
      `owns_object(ρ(x), [])` は偽である。
  `collect_consumes` はこの本体について何も報告しない -- `Ret(w)` は `w` の boxed leaf を積むが `I64` は
  持たず、`Release` と `Let(w, Llvm(int_lit_0, []), ..)` は消費を作らない (後者はオペランドを持たない)。
  `levelled_sites(f)` は `(x, [])` だけを挙げる (`App` が無く、`Retain`/`Release` はこの 1 つ)。
  `x` はパラメータなので `origin(x, [])` は `Exactly((x, []))` であり、候補は `(x, [])` 1 つである。
  `owns_object_yet(vars, type_env, x, [], ∅)` は、`under(Mix, []) = [[]]` (`sub(Mix, []) = Some(Mix)`、
  L2、`<1>1`) の唯一の unit について鍵 `trunc(Mix, []) = []` を作り、`leaves(Mix) = {[1]}` の
  `trunc(Mix, [1]) = []` が鍵に等しいので `(x, [1]) ∈ ∅` を要求する。これは偽なので `owns_a_candidate` は
  偽であり、`level_ownership` は `false` を返す。よって不動点は空集合である。
  `func_has_borrowable_param` は `x` の leaf `[1]` が所有されないので真であり、`f` は capture を持たない。
  `funcs_observing_uniqueness` が `observing` に名前を入れるのは、走査が `observes_uniqueness()` の真な
  `Llvm` の右辺を見つけたときと、不動点の繰り返しが `observing` の元への辺を持つ関数を足すときの 2 か所
  であり、後者は `observing` が空ならば何も足さない。このプログラムの本体に現れる op は
  `InlineLLVMIntLit` だけで、`observes_uniqueness` を override しないので既定の偽である。よって
  `observing` は空のまま返り、`funcs_observing_uniqueness` は `f` を含まない (この議論は `callees` に
  どの辺が入るかに依らない)。よって借用版があり、その `owned_units` に `(ρ(x), [])` は入らないので、
  L16 と上の計算より `owns_object(ρ(x), [])` は偽である。
  BY <ref id=9f11796/>, <ref id=f221813/>, <ref id=90c9817/>, <1>1, <1>2, CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership,
     owns_object_yet, func_has_borrowable_param, funcs_observing_uniqueness, borrow_ify,
     CODE src/rc_ir/ownership.rs: collect_consumes_go, origin_inner,
     CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness,
     CODE src/fixstd/builtin.rs: InlineLLVMIntLit

<1>4. 借用版を版 `V` として固定する。`(ρ(x), [])` は `V` の site (DEF site) である。`ρ(x)` の値のタグが
      `n` (変位 0) である `Post(V)` の活性化を取ると、その各位置で `Inh(ρ(x), []) = ∅` であり、節 3 は
      真、節 2 と節 1 は偽である。
  DEF site は `Pre(V)` -- 複製の本体 -- を歩いて site を集める。P9 よりその本体は入力の
  本体の束縛変数を付け替えたものなので、`Release(ρ(x), [])` を持ち、`(ρ(x), [])` はその site である。
  `ρ(x)` は `V` のパラメータなので (P9)、D6 より活性化が始まった時点で値を得ており、その値は以後
  変わらない。よって節 2 と節 3 を読む位置は活性化のどの位置に取ってもよい。
  `rename_var` は型を残すので `ty(ρ(x)) = Mix` であり、`<1>1` より `Λ([]) = {[1]}` である。その leaf が
  `Mix` の根の unbox union の節で選ぶ変位番号は 1、この活性化のタグは 0 なので D16 の条件はどの位置でも
  成り立たない。よって `Inh(ρ(x), []) = ∅` である。節 3 の全称は空なので真、節 2 の存在は空なので偽で
  ある。節 1 は `owns_unit(ρ(x), [])` であり、`<1>3` と L15 より
  `cand(ρ(x), []) = {(ρ(x), [])}` の唯一の候補について `owns_object` が偽なので偽である。
  BY <1>1, <1>3, <ref id=596a46d/>, <ref id=66c9670/>, <ref id=c3b2aa3/>, <ref id=63eadd9/>, DEF site,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
     CODE src/rc_ir/rename.rs: rename_var

<1>5. QED
  `<1>2` と `<1>2a` より、この入力プログラムは第 4 節の仮定のうち入力プログラムを縛るものをすべて満たす。
  `<1>3` と `<1>4` がその上で `Inh(ρ(x), []) = ∅` と節 1 の偽を与える。
  BY <1>2, <1>2a, <1>3, <1>4

**位置を `Release(ρ(x), [])` の節点に取らないのは、`Post(V)` にその位置が無いからである。** 節 1 が
偽なので借用版の `rewrite_rc` はこの節点を落とす (P10)。site は `Pre(V)` から集めるので site の側には
残る。第 6 節の読み方が「site の節点を訪れる位置に限れない」と述べるのがこの形である。

**この乖離は無害である。** 節 1 が偽なので借用版の `rewrite_rc` は `Release(ρ(x), [])` を落とす (P10)。
A5 より inhabited でない leaf は参照を持たず、A4 よりコード生成の `Release(ρ(x), [])` は inhabited な leaf
の参照カウントだけを ±1 するので、この活性化で落とした節点は何にも触れない。壊れているのは言明の側だけで
ある -- 節 3 が「触れる参照はすべて所有されている」を空虚に述べるので、そこから節 2 や節 1 へは渡れない。

### `level_ownership` と `union_as` の隔たり

**`Λ(u)` の leaf の側から unit の側へ直接渡る形は、`union_as` の宣言で破れる。** `union_as_k` は
unbox union のオペランドについて変位 `k` の leaf だけを名指すので
(`CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov`)、unit の側が `truncate_to_unit` で
辿り着くオペランドの unit には、どの結果 leaf も名指さない leaf が残る。

**この節が証明する 2 つの向きはその形を通らない。** 節 2 から節 1 へは対偶で渡り
(`P7a の 2 つの向き` の `<1>4`)、そこで読むのは P7d である。`union_as` の場合、節 1 が偽ならば P7d より
site の候補はすべて偽であり、L22 の `<1>8` はオペランドの unit へ降りて帰納法の仮定を使う。名指されない
leaf の所有はどこでも問われない。**したがって宣言についての追加の仮定は要らない。**
