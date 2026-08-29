# P7e、P7d、P7a -- 所有の unit 粒度と一様性

この文書は `README.md` の P7d を証明し、P7e については下の**言明の訂正**が述べる形を証明し、P7a の
2 つの向き -- **節 1 から節 3** と **節 2 から節 1** -- を証明する。立つのは `README.md` の定義と仮定、
命題 P1-P7 の**言明**、および第 1 節の仮定 **(N)** である。

P7a の残る向き (節 3 から節 2、節 3 から節 1) は偽である。`Λ(u)` に inhabited (D16) な leaf が 1 つも
無い site では節 3 が空虚に真になるからで、第 6 節の R2 がその本体を挙げる。`Inh(v, u) ≠ ∅` を仮定に
足せば 3 つは同値になる。

**P7e の言明の訂正。** README の P7e は「任意の root `r` と path `p` について
`owns_object(r, p) = owns_object(r, truncate_to_unit(ty(r), p))`」と述べる。第 1 節が定める `=` は
「両辺が値を返してその値が等しいか、両辺が中断するか」なので、左辺が値を返して右辺が中断する `(r, p)` が
1 つでもあればこの等式は偽である。そのような `(r, p)` は在る。`pty(r) = None` である `r` -- 例えば
`I64` の値を束縛する `Binding::Producer` の変数 -- と `p = [0]` を取ると、左辺は `owns_object` の
`None` の腕が真を返し、右辺は `truncate_to_unit(I64, [0])` が `unit_step(I64) = NoUnit` の腕で `panic!`
する (`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object`,
`CODE src/rc_ir/ownership.rs: truncate_to_unit`, `unit_step`)。第 3 節が証明するのは `pty(r)` で場合を
分けた形であり、その言明は第 3 節の冒頭に置く。この文書の中で P7e を読む 2 か所 (L21 の `<1>1`、
L22 の `<1>2`) は、どちらも訂正した形で足りる。

この文書が読んだコードは、コミット `10bb7e118ce88fbed8b66d20a7b8d8fea2e82e92` の版である。

**この文書が使う仮定 (N)。** A13 と P9 の後半が言うのは**束縛名**についてだけである。L15 と L16 が要るのは
より広い形 -- 複製が作る名前が、束縛名でない名前 (直接呼び出しが名指す関数の名前と、グローバル値を読む
`RcVar` の名前) とも異なること -- であり、第 1 節がそれを **(N)** として置く。衝突すると `origin` が複製の
変数を原本の束縛へ辿る。第 6 節の最後の小節がこの差を記録する。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P7e (`owns_object` は unit ごとに答える) | **訂正した形**を証明した (第 3 節)。README の形は偽である |
| P7d (所有は site ごとに一様である) | 証明した (第 5 節) |
| P7a、節 1 から節 3 | 証明した (第 6 節) |
| P7a、節 2 から節 1 | 証明した (第 6 節) |
| P7a、節 3 から節 2 と節 1 | **偽** (第 6 節の R2)。`Inh(v, u) ≠ ∅` を足せば真になる |

P7e の要は L1 である。`subtree_type` と `truncate_to_unit` は同じ型の列を同じ順に歩き、同じ `unit_step` の
答えで場合を分けるので、片方が `None` を返す位置がもう片方の `break` する位置になる。

P7d の要は L11 と L14 である。`level_ownership` は候補 `(r, p)` について `covered_leaves(ty(r), p)` を
所有へ倒し、`owns_object_yet` は `units_under(ty(r), p)` の各 unit に「同じ鍵を持つ所有された leaf」を
要求する。この 2 つが噛み合うのは `covered_leaves(ty(r), p)` が空でないときであり (L11)、空でないことは
候補の path が `origin` の再帰で作られる形に限られることから出る (L14)。

P7a の 2 つの向きは、`origin` の再帰についての 2 つの帰納である。L21 が静的な向き (unit が所有ならその下の
**すべての** leaf も所有) を、L22 が実行時の向き (unit が非所有ならその下の **inhabited な** leaf も
非所有) を与える。どちらも `(x, π)` が「unit を覆う」(L18) ことを仮定に運ぶ。この 2 つと L12 が回す帰納の
尺度は `|Reach(・)|` であり、それが狭義に減ることは L11a が与える -- 減るのは `origin` の再帰に閉路が
無いからであり、閉路があれば `origin` は停止しない。**`union_as` が作る隔たりはこの 2 つの帰納の
どちらにも現れない** -- 前の版の言明が要求していた「宣言についての仮定」(前稿の (★)) は要らない。
第 6 節の最後の 2 つの小節がその答えと、見つかった点を述べる。

L22 が inhabited に限るのは 1 か所である。`Binding::Llvm` の腕で、`result_prov` が `⊥` (空集合) と宣言した
leaf を落とすのに A3 の表の第 1 行を使う (L22 の `Binding::Llvm` の場合)。inhabited でない leaf が参照を持たないことを
述べるのは A5 であり、これは P10 と P14 へ渡すところで使う (`P7a の 2 つの向き` の最後の段落と R2)。

## 1. 記法

1 つの関数 (またはグローバル初期化子) の 1 つの出力版を固定する。その `RewriteCtx` を `ctx`、`ctx.type_env`
を `type_env`、`ctx.vars` を `vars`、`ctx.owned_units` を `OU` と書く
(`CODE src/rc_ir/borrow.rs: RewriteCtx`)。以下では型環境を引数から落として書く。

- `step(τ)` は `unit_step(τ, type_env)` (`CODE src/rc_ir/ownership.rs: unit_step`)。
- `units(τ)` は `rc_units(τ, type_env)`、`leaves(τ)` は `boxed_leaf_paths(τ, type_env)`
  (`CODE src/rc_ir/ownership.rs: rc_units`, `CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)。
- `trunc(τ, π)` は `truncate_to_unit(τ, π, type_env)`、`under(τ, π)` は `units_under(τ, π, type_env)`、
  `sub(τ, π)` は `subtree_type(τ, π, type_env)`
  (`CODE src/rc_ir/ownership.rs: truncate_to_unit`, `units_under`, `subtree_type`)。
- `pty(r)` は `vars.param_tys.get(r)` (`CODE src/rc_ir/ownership.rs: VarTable`)。
- `owns(r, p)` は `ctx.owns_object(r, p)` (`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object`)。
- `ty(x)` は `x` が得る値の型である。A12 より同じ名前の `RcVar` が持つ型は一致するので、これは `x` の
  出現によらない。`x` がこの版のパラメータ・capture のとき、`VarTable::of` が `param_tys` と `var_tys` の
  両方に同じ型を入れるので `pty(x) = Some(ty(x))` である
  (`CODE src/rc_ir/ownership.rs: VarTable::of`)。

`FieldPath` は `Vec<usize>` である (`CODE src/rc_ir/ast.rs: FieldPath`)。`π[0..k]` は `π` の先頭 `k` 個の
要素からなる path、`|π|` は `π` の長さである。

関数呼び出しについて、**値を返す**とその **中断する** (`panic!` / `assert!` / `unreachable!` に到達する) を
区別する。等式「`f = g`」は、両辺が値を返してその値が等しいか、両辺が中断するかのどちらかであることを
いう。

**(N) (複製の名前は入力のどの名前とも異なる)**
入力の関数 `func` に**現れる名前**とは、`func` のパラメータ・capture の名前と、`func.body` に現れる
`RcVar` の名前 (`Let` の束縛変数、`Destructure` の容器とフィールド変数、`Match` の scrutinee とアームの
payload 変数、`App` の callee と各引数、`Closure` の各 capture、`Llvm` の各オペランド、`Retain` /
`Release` / `Eval` / `Ret` が名指す変数) の全体である
(`CODE src/rc_ir/ast.rs: for_each_var`, `for_each_var_of_node`, `for_each_var_of_rhs`)。
**(N)** とは、`clone_func` が `func` から作る借用版について、`fresh_rename_function` が返す写像
`rename` の像のどの名前も、`func` に現れるどの名前とも異なることをいう。

果たす者: lowering (A13 と同じ、名前の付け方)。検査: `check_clone_names_are_fresh`、ただし
`develop_mode` のときだけ走る。これは入力の全関数・全グローバル初期化子のパラメータ・capture と
`for_each_var` の全変数を `bound` に集め、複製が作る各名前が `bound` に入らないことを `assert!` する
(`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`)。すなわち検査が見ている集合が (N) の言う集合を
含む。

A13 と P9 の後半では足りない。どちらも**束縛名**についてだけ述べるが、`func` に現れる名前には束縛名で
ないものがある -- 直接呼び出しが名指す関数の名前と、グローバル値を読む `RcVar` の名前である
(`CODE src/rc_ir/ownership.rs: origin_inner` の `None` の腕)。

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
  `origin(x, π).acted_on()` を集合とみなしたものである
  (`CODE src/rc_ir/ownership.rs: Origin::candidates`, `Origin::acted_on`)。
- `owns_unit(v, u)` は「`cand(v, u)` のすべての元 `(r, p)` について `owns(r, p)` が真」である
  (`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit`)。

## 2. 型の歩みについての補題

### L1 (2 つの歩きは同じ場合分けをする)

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

### L1a (`unit_step` の 4 つの答えと、2 つの走査の降下)

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

**leaf と unit がずれるのはここである。**`is_union(σ)` または `is_punched_array(σ)` が真で `is_box(σ)` も
`is_array(σ)` も偽のとき、`step(σ) = Unit` なので `rc_units_go` は `path` を積んで止まるのに、`go` は
`unpunched_field_types(σ)` の下へ降りる。`Std::PunchedArray a` は `unbox struct { _arr : Array a, _idx : I64 }`
であり (`CODE src/fixstd/std.fix: PunchedArray`)、`is_punched_array` が真なのでこの形になる。

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

### L2 (`units_under` の 2 つの形)

**言明**。`sub(τ, π)` が `Some(σ)` を返すとき `under(τ, π) = { π ++ u : u ∈ units(σ) }` であり、`None` を
返すとき `under(τ, π) = [π]` である。`sub(τ, π)` が中断するとき `under(τ, π)` は中断する。

<1>1. `under(τ, π)` は `sub(τ, π)` の答えで分岐し、`Some(sty)` のときは `rc_units(sty)` の各要素 `u` に
      `π` を前置した列を返し、`None` のときは `vec![π.clone()]` を返す。
  BY CODE src/rc_ir/ownership.rs: units_under

<1>2. QED
  `under` は `sub` を呼んでから分岐するので、`sub` が中断すれば `under` も中断する。
  BY <1>1

### L3 (`Unit` の型の unit はただ 1 つ)

**言明**。`step(σ) = UnitStep::Unit` のとき `units(σ) = [[]]` である。

<1>1. `rc_units(σ)` は空の `path` と空の `out` で `rc_units_go(σ, ·, path, out)` を呼び、`out` を返す。
  BY CODE src/rc_ir/ownership.rs: rc_units

<1>2. `rc_units_go` は `step(σ)` で分岐し、`UnitStep::Unit` の腕で `out` に `path.clone()` を積んで
      終わる。
  BY CODE src/rc_ir/ownership.rs: rc_units_go

<1>3. QED
  `<1>1` より最初の呼び出しの `path` は空列なので、`<1>2` の腕が積むのは `[]` 1 つだけである。
  BY <1>1, <1>2

### L4 (`trunc` の答えを再び歩く)

**言明**。`trunc(τ, π)` が値 `t` を返すとき、次が成り立つ。

- `trunc(τ, t) = t`。
- `under(τ, t)` は、L1 の場合 (a) では `under(τ, π)` と同じ呼び出しであり、場合 (b) の `Unit` の行と
  `Capture` の行では `[t]` である。

<1>1. CASE L1 の場合 (a) (`m = |π|`)
  L1 (a) より `t = π` であるから、`trunc(τ, t)` は `trunc(τ, π)` と同じ呼び出しで `t` を返し、
  `under(τ, t)` は `under(τ, π)` と同じ呼び出しである。
  BY L1

<1>2. CASE L1 の場合 (b) の `Unit` の行
  <2>1. `t = π[0..m]` であり `|t| = m` である。
    BY L1
  <2>2. `τ` の `t` に沿う歩みの長さは `m` であり、その列は `cur_0, ..., cur_m` である。
    `i < m` について `t[i] = π[i]` なので、DEF 歩み が `cur_1, ..., cur_m` を定める条件は `π` のときと
    同じである。`|t| = m` なので `cur_{m+1}` を定める条件の 1 つ目 (`i < |t|`) が破れる。
    BY <2>1, DEF 歩み
  <2>3. `trunc(τ, t) = t` であり、`sub(τ, t) = Some(cur_m)` である。
    `<2>2` より `t` については L1 の場合 (a) が当てはまる。
    BY <2>2, L1
  <2>4. QED
    `<2>3` と L2 より `under(τ, t) = { t ++ u : u ∈ units(cur_m) }` であり、この場合 `step(cur_m) = Unit`
    なので L3 より `units(cur_m) = [[]]`、すなわち `under(τ, t) = [t]` である。
    BY <2>3, L2, L3

<1>3. CASE L1 の場合 (b) の `Capture` の行 (`π[m] = capture_idx`)
  <2>1. `t = π[0..m+1]` であり `|t| = m + 1`、`t[m] = π[m] = capture_idx` である。
    BY L1
  <2>2. `τ` の `t` に沿う歩みの長さは `m` であり、その列は `cur_0, ..., cur_m` である。
    `i < m` について `t[i] = π[i]` なので `cur_1, ..., cur_m` は `π` のときと同じである。
    `step(cur_m) = Capture` は `Fields` ではないので、DEF 歩み は `cur_{m+1}` を定めない。
    BY <2>1, DEF 歩み
  <2>3. QED
    `<2>2` より `t` については L1 の場合 (b) が当てはまり、`step(cur_m) = Capture` かつ
    `t[m] = capture_idx` なので、`sub(τ, t) = None` かつ `trunc(τ, t) = t[0..m+1] = t` である。
    L2 より `under(τ, t) = [t]` である。
    BY <2>1, <2>2, L1, L2

<1>4. QED
  L1 の場合分けのうち、`trunc(τ, π)` が値を返すのは (a) と、(b) の `Unit` の行と `Capture`
  (`π[m] = capture_idx`) の行だけである。この 3 つを `<1>1`、`<1>2`、`<1>3` が扱った。
  BY <1>1, <1>2, <1>3, L1

### L5 (歩みの合成)

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
  BY L1, DEF 歩み

<1>2. `i ≥ |p|` について、`τ` の `p ++ q` に沿う歩みの `cur_{|p| + j}` は `σ` の `q` に沿う歩みの `cur_j`
      である。
  `<1>1` より `cur_{|p|} = σ` であり、DEF 歩み は添字 `|p| + j` について `(p ++ q)[|p| + j] = q[j]` を
  読む。よって漸化式は `σ` の `q` に沿う歩みのものと同じである。
  BY <1>1, DEF 歩み

<1>3. `sub(τ, p ++ q) = sub(σ, q)` である。
  `<1>1` より `sub(τ, p ++ q)` のループの最初の `|p|` 周はすべて `Fields` の腕を通り (`sub(τ, p)` が
  `Some` を返したので、L1 の場合 (a) が `(τ, p)` に当てはまる)、`cur = σ` になる。残りの周は
  `sub(σ, q)` のループそのものである。
  BY <1>1, <1>2, L1, CODE src/rc_ir/ownership.rs: subtree_type

<1>4. `trunc(τ, p ++ q) = p ++ trunc(σ, q)` である。
  `<1>1` より `trunc(τ, p ++ q)` のループの最初の `|p|` 周はすべて `Fields` の腕を通り、`out` に
  `p[0], ..., p[|p|-1]` を積んで `cur = σ` になる。残りの周は `trunc(σ, q)` のループそのものであり、
  そこで `out` に積まれるもの (と `break` / 中断) は `trunc(σ, q)` と同じである。
  BY <1>1, <1>2, L1, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>5. `leaves(τ)` のうち `p` を前置に持つものの全体は `{ p ++ λ : λ ∈ leaves(σ) }` である。
  `sub(τ, p) = Some(σ)` なので L1 の場合 (a) が `(τ, p)` に当てはまり、`τ` の `p` に沿う歩みの長さは
  `|p|` で `cur_{|p|} = σ` である。DEF 歩み より `i < |p|` の各段で `step(cur_i)` は
  `Fields { held_fields, .. }` であって `held_fields` は添字 `p[i]` の対を含む。L1a (a) より `go` は
  その各段で `held_fields` の同じ対の下へ降りるので、`go` は `path = p` で `σ` に着く。`go` はその位置で
  `σ` について走るので、`p` を前置に持つ path として積むのは、`σ` から始めた `go` が積む path の前に
  `p` を置いたものに限る。
  BY L1, L1a, DEF 歩み, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>6. `units(τ) ⊇ { p ++ u : u ∈ units(σ) }` である。
  L1 の場合 (a) と DEF 歩み より、`τ` の `p` に沿う歩みの各段 `i < |p|` で `step(cur_i)` は
  `Fields { held_fields, .. }` であって `held_fields` は添字 `p[i]` の対を含む。L1a (a) より
  `rc_units_go` はその各段で同じ対の下へ降りるので、`path = p` で `cur_{|p|} = σ` に達し、そこから
  `units(σ)` の各要素を `p` の後ろに積む。
  BY L1, L1a, DEF 歩み, CODE src/rc_ir/ownership.rs: rc_units_go

<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### L6 (unit は自分自身へ切り詰まる)

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
  BY <1>1, <1>2, L1

<1>4. 場合 (i) では `under(τ, u) = [u]` である。
  `<1>3` と L2 より `under(τ, u) = { u ++ w : w ∈ units(cur_{|u|}) }` であり、`<1>3` と L3 より
  `units(cur_{|u|}) = [[]]` である。
  BY <1>3, L2, L3

<1>5. 場合 (ii) では、`τ` の `u` に沿う歩みの長さは `|q| = |u| - 1` であり、`unit_step(cur_{|q|})` は
      `Capture { capture_idx, .. }` かつ `u[|q|] = capture_idx` である。
  `<1>1` より `i < |q|` の各位置で `unit_step` は `Fields` なので `cur_0, ..., cur_{|q|}` が定まり、
  `cur_{|q|}` の `unit_step` は `Capture` なので DEF 歩み は `cur_{|q|+1}` を定めない。
  BY <1>1, DEF 歩み

<1>6. 場合 (ii) では `trunc(τ, u) = u` かつ `sub(τ, u) = None`、したがって `under(τ, u) = [u]` である。
  `<1>5` と L1 の場合 (b) の `Capture` (`u[|q|] = capture_idx`) の行より
  `trunc(τ, u) = u[0..|q|+1] = u` かつ `sub(τ, u) = None` である。L2 より `under(τ, u) = [u]` である。
  BY <1>5, L1, L2

<1>7. QED
  `<1>1` の 2 つの場合を `<1>3`/`<1>4` と `<1>6` が扱った。言明の (i) は `<1>2` と `<1>3`、(ii) は
  `<1>5` である。この 2 つは排他である。`<1>2` は場合 (i) で `τ` の `u` に沿う歩みの長さが `|u|` だと
  述べ、`<1>5` は場合 (ii) でその長さが `|u| - 1` だと述べる。DEF 歩み は 1 つの `(τ, u)` に 1 つの長さを
  与え、`|u| ≠ |u| - 1` なので、2 つが同時に成り立つことはない。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, DEF 歩み

### L7 (unit の下の leaf はその unit へ切り詰まる)

**言明**。`u ∈ units(τ)` とし、`λ ∈ leaves(τ)` が `u` を前置に持つとする。このとき
`trunc(τ, λ) = u` である。さらに `Λ_τ(u) := { λ ∈ leaves(τ) : u ⊑ λ }` は空でない。

<1>1. L6 の (i) のとき、`trunc(τ, λ) = u` である。
  `u ⊑ λ` なので、`τ` の `λ` に沿う歩みの最初の `|u| + 1` 個の型は `τ` の `u` に沿う歩みの型と一致する
  (DEF 歩み は添字 `i < |u|` について `λ[i] = u[i]` だけを読む)。L6 の (i) より `unit_step(cur_{|u|})`
  は `Unit` である。`λ = u` なら L1 の場合 (a) より `trunc(τ, λ) = u`。`λ ≠ u` すなわち `|λ| > |u|` なら
  L1 の場合 (b) の `Unit` の行より `trunc(τ, λ) = λ[0..|u|] = u`。
  BY L1, L6, DEF 歩み

<1>2. L6 の (ii) のとき、`trunc(τ, λ) = u` である。
  `u = q ++ [capture_idx]` であり、`u ⊑ λ` なので `λ[|q|] = capture_idx` である。L6 の (ii) より `τ` の
  `u` に沿う歩みの長さは `|q|` であり、`unit_step(cur_{|q|})` は `Capture { capture_idx, .. }` である。
  DEF 歩み が `cur_1, ..., cur_{|q|}` を定めるのに読むのは添字 `i < |q|` の要素だけであり、`q ⊑ u ⊑ λ`
  よりその範囲で `λ[i] = u[i]` なので、`τ` の `λ` に沿う歩みの最初の `|q| + 1` 個の型は `τ` の `u` に
  沿う歩みのものと一致する。`unit_step(cur_{|q|})` が `Fields` でないので、`τ` の `λ` に沿う歩みの長さも
  `|q|` である。`|λ| ≥ |u| = |q| + 1 > |q|` なので L1 の場合 (b) の `Capture` の行が当てはまり、
  `trunc(τ, λ) = λ[0..|q|+1] = u` である。
  BY L1, L6, DEF 歩み

<1>3. `Λ_τ(u)` は空でない。
  P1 の後半より、`u` はある `λ ∈ leaves(τ)` の `trunc(τ, λ)` である。`trunc` が `out` に積むのは
  引数の path の要素を順に取ったものなので、`trunc(τ, λ)` は `λ` の接頭辞である。よって `u ⊑ λ` であり
  `λ ∈ Λ_τ(u)` である。
  BY P1, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>4. QED
  L6 は (i) と (ii) のちょうど 1 つが成り立つと述べる。
  BY <1>1, <1>2, <1>3, L6

### L8 (leaf と unit は同時に空になる)

**言明**。`leaves(τ) = ∅` と `units(τ) = ∅` と `is_fully_unboxed(τ)` は同値である。

<1>1. `is_fully_unboxed(τ)` ならば `leaves(τ) = ∅` かつ `units(τ) = ∅` である。
  `boxed_leaf_paths` の `go` は最初に `is_fully_unboxed` を見て何も積まずに戻り、`unit_step` は最初に
  `is_fully_unboxed` を見て `NoUnit` を返し、`rc_units_go` は `NoUnit` で何も積まない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go

<1>2. `is_fully_unboxed(τ)` が偽ならば `leaves(τ) ≠ ∅` かつ `units(τ) ≠ ∅` である。
  <2>1. `is_fully_unboxed` は、`is_box` / `is_closure` / `is_array` のいずれかで偽を返し、`is_funptr` で
        真を返し、それ以外では `unpunched_field_types` の全フィールドについての再帰の全称である。
    BY CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>2. `is_box(τ)` または `is_array(τ)` のとき、`go` は `path` を積み、`unit_step` は `Unit` を返して
        `rc_units_go` は `path` を積む。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>3. `is_closure(τ)` のとき、`go` は `path ++ [CLOSURE_CAPTURE_IDX]` を積み、`unit_step` は
        `Capture { capture_idx: CLOSURE_CAPTURE_IDX, .. }` を返して `rc_units_go` は
        `path ++ [CLOSURE_CAPTURE_IDX]` を積む。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>4. `is_box(τ)`、`is_array(τ)`、`is_closure(τ)`、`is_funptr(τ)` がどれも偽のとき、
        `leaves(τ) ≠ ∅` かつ `units(τ) ≠ ∅` である。
    <3>1. `unpunched_field_types(τ)` のある対 `(idx, fty)` について `is_fully_unboxed(fty)` は偽である。
      `<2>1` より、この場合 `is_fully_unboxed(τ)` は `unpunched_field_types(τ)` の全対についての全称で
      あり、`<1>2` の仮定よりそれは偽である。
      BY <2>1
    <3>2. `go` は `τ` の位置で `unpunched_field_types(τ)` の各対の下へ降りる。
      `is_fully_unboxed(τ)` は `<1>2` の仮定より偽であり、`is_closure(τ)`・`is_box(τ)`・`is_array(τ)` は
      `<2>4` の場合の条件より偽である。L1a の `go` の欄はこの 4 つがどれも偽のとき
      `unpunched_field_types(τ)` の各対の下へ降りると述べる。
      BY L1a
    <3>3. `leaves(τ) ≠ ∅` である。
      `<3>1` と型の in-place の降下についての帰納法の仮定より `leaves(fty) ≠ ∅` である。`<3>2` より
      `go` は `path` に `idx` を積んで `fty` へ降りるので、`fty` から始めた走査が積む path の前に `idx` を
      置いたものが `leaves(τ)` に入る。
      BY <3>1, <3>2, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <3>4. `units(τ) ≠ ∅` である。
      `<3>2` と L1a (b) より `step(τ)` は `Unit` か `Fields` である。`Unit` のとき `rc_units_go` は
      `path` を積むので `units(τ) ≠ ∅` である。`Fields { held_fields, .. }` のとき L1a (a) より
      `held_fields = unpunched_field_types(τ)` であり `rc_units_go` は対 `(idx, fty)` の下へ降りるので、
      `<3>1` と帰納法の仮定 `units(fty) ≠ ∅` から `units(τ) ≠ ∅` が出る。
      BY <3>1, <3>2, L1a, CODE src/rc_ir/ownership.rs: rc_units_go
    <3>5. QED
      A10 より型の in-place の降下は有限なので、この帰納は基底に着く。
      BY <3>3, <3>4, A10
  <2>5. QED
    `is_funptr(τ)` のときは `is_fully_unboxed(τ)` が真なので、この場合は仮定に反する。残りを `<2>2`、
    `<2>3`、`<2>4` が扱った。
    BY <2>1, <2>2, <2>3, <2>4

<1>3. QED
  BY <1>1, <1>2

### L9 (`units_under` の要素は unit へ切り詰まる)

**言明**。`trunc(τ, ・)` が `under(τ, p)` の各要素について値を返すとき、その値は `units(τ)` の要素である。

<1>1. CASE `sub(τ, p) = Some(σ)`
  L2 より `under(τ, p) = { p ++ w : w ∈ units(σ) }` である。L6 より `trunc(σ, w) = w` なので、L5 より
  `trunc(τ, p ++ w) = p ++ trunc(σ, w) = p ++ w` である。L5 より `p ++ w ∈ units(τ)` である。
  BY L2, L5, L6

<1>2. CASE `sub(τ, p) = None`
  L2 より `under(τ, p) = [p]` である。L1 の場合 (b) より、`trunc(τ, p)` が値を返すのは `Unit` の行か
  `Capture` (`p[m] = capture_idx`) の行であり、値はそれぞれ `p[0..m]`、`p[0..m+1]` である。
  <2>1. `rc_units_go` は `path = p[0..m]` で `cur_m` に達する。
    `i < m` の各位置で `unit_step(cur_i)` は `Fields { held_fields, .. }` であって `p[i]` は
    `held_fields` に含まれる (DEF 歩み)。L1a (a) より `rc_units_go` はその各段で同じ対の下へ降りる。
    BY L1a, DEF 歩み, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>1a. `Unit` の行では `p[0..m] ∈ units(τ)` である。
    `<2>1` より `rc_units_go` は `path = p[0..m]` で `cur_m` に達し、`unit_step(cur_m) = Unit` なので
    `path` を積む。
    BY <2>1, L1a
  <2>2. `Capture` の行では `p[0..m+1] ∈ units(τ)` である。
    `<2>1` より `rc_units_go` は `path = p[0..m]` で `cur_m` に達し、
    `unit_step(cur_m) = Capture { capture_idx, .. }` なので `path ++ [capture_idx]` を積む。
    `p[m] = capture_idx` なのでこれは `p[0..m+1]` である。
    BY <2>1, L1a
  <2>3. QED
    BY L1, <2>1a, <2>2

<1>3. QED
  `sub(τ, p)` は `Some` か `None` を返すか中断する。中断するとき `under(τ, p)` も中断する (L2) ので
  言明の仮定が成り立たない。
  BY <1>1, <1>2, L2

### L9a (leaf に沿う歩みの終わり方)

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
  BY L1a

<1>2. `go` が `λ` を積む位置に至るまでの各位置で `go` は降りている。その位置の型を順に
      `g_0 = τ, g_1, ...` と書くと、`g_{i+1}` は `unpunched_field_types(g_i)` の添字 `λ[i]` の対の第 2
      成分である。降りる位置は、場合 (i) では `i < |λ|` の各 `i`、場合 (ii) では `i < |μ|` の各 `i` で
      ある。
  `go` が `path` を伸ばすのは降りるときだけであり、積む位置の `path` は場合 (i) では `λ`、場合 (ii) では
  `μ` である。よってそこに至るまでの各位置で `go` はその path の次の添字の対の下へ降りている。L1a より、
  降りる先は `unpunched_field_types` の対である。
  BY <1>1, L1a, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. `i ≤ m_λ` について `cur_i = g_i` であり、`i < m_λ` の各位置で `step(cur_i)` は
      `Fields { held_fields, .. }` であって `held_fields` は添字 `λ[i]` の対を含む。また `m_λ ≤ |λ|` で
      ある。
  DEF 歩み は `i < m_λ` の各位置で `step(cur_i) = Fields { held_fields, .. }` かつ `held_fields` が添字
  `λ[i]` の対を含むことを要求し、`cur_{i+1} = held_field_type(held_fields, λ[i], _)` と定める。L1a (a) より
  `held_fields = unpunched_field_types(cur_i)` なので、これは `<1>2` の `g` の漸化式と同じである。基底は
  `cur_0 = g_0 = τ` である。`m_λ ≤ |λ|` は DEF 歩み が `i < |λ|` のときにしか次の型を定めないことによる。
  BY <1>2, L1a, DEF 歩み, CODE src/rc_ir/ownership.rs: held_field_type

<1>4. CASE `<1>1` の場合 (i)
  <2>1. `m_λ < |λ|` のとき `step(cur_{m_λ}) = Unit` である。
    `<1>3` より `cur_{m_λ} = g_{m_λ}` であり、`<1>2` より `g_{m_λ}` は `go` が降りる位置なので、L1a (b)
    より `step(g_{m_λ})` は `Fields` か `Unit` である。`Fields { held_fields, .. }` だとすると、L1a (a)
    より `held_fields = unpunched_field_types(g_{m_λ})` であり、`<1>2` よりそれは添字 `λ[m_λ]` の対を
    含むので、DEF 歩み は `cur_{m_λ+1}` を定める。これは `m_λ` が歩みの長さであることに反する。
    BY <1>2, <1>3, L1a, DEF 歩み
  <2>2. `m_λ = |λ|` のとき `step(cur_{m_λ}) = Unit` である。
    `<1>3` より `cur_{|λ|} = g_{|λ|}` であり、場合 (i) の条件よりその位置で `is_box` か `is_array` が
    真なので、L1a より `step(cur_{m_λ}) = Unit` である。
    BY <1>3, L1a
  <2>3. QED
    `<1>3` より `m_λ ≤ |λ|` なので `<2>1` と `<2>2` が場を尽くし、どちらも (A) を与える。
    BY <1>3, <2>1, <2>2

<1>5. CASE `<1>1` の場合 (ii)
  <2>1. `m_λ < |μ|` のとき `step(cur_{m_λ}) = Unit` であり、(A) が成り立つ。
    `<1>3` より `cur_{m_λ} = g_{m_λ}` であり、`<1>2` より `g_{m_λ}` は `go` が降りる位置なので、L1a (b)
    より `step(g_{m_λ})` は `Fields` か `Unit` である。`Fields { held_fields, .. }` だとすると、L1a (a) と
    `<1>2` より `held_fields` は添字 `λ[m_λ]` の対を含むので DEF 歩み が `cur_{m_λ+1}` を定め、`m_λ` が
    歩みの長さであることに反する。
    BY <1>2, <1>3, L1a, DEF 歩み
  <2>2. `m_λ = |μ|` のとき (B) が成り立つ。
    `<1>3` より `cur_{|μ|} = g_{|μ|}` であり、場合 (ii) の条件よりその位置で `is_closure` が真なので、
    L1a より `step(cur_{m_λ}) = Capture { capture_idx: CLOSURE_CAPTURE_IDX, .. }` である。
    `λ = μ ++ [CLOSURE_CAPTURE_IDX]` より `m_λ = |μ| = |λ| - 1 < |λ|` かつ
    `λ[m_λ] = CLOSURE_CAPTURE_IDX = capture_idx` である。
    BY <1>3, L1a
  <2>3. QED
    `<1>3` より `m_λ ≤ |λ| = |μ| + 1` である。`m_λ = |μ| + 1` だとすると DEF 歩み より
    `step(cur_{|μ|}) = Fields` だが、`<1>3` より `cur_{|μ|} = g_{|μ|}` であり、場合 (ii) の条件より
    その位置で `is_closure` が真なので L1a より `step(cur_{|μ|}) = Capture` である。よって `m_λ ≤ |μ|` で
    あり、`<2>1` と `<2>2` が場を尽くす。
    BY <1>3, <2>1, <2>2, L1a, DEF 歩み

<1>6. QED
  `<1>1` の 2 つの場合を `<1>4` と `<1>5` が扱った。`i < m_λ` についての主張と `m_λ ≤ |λ|` は `<1>3` で
  ある。
  BY <1>1, <1>3, <1>4, <1>5

### L10 (leaf に届く path では歩みが中断しない)

**言明**。`covered(τ, p) ≠ ∅` のとき、`trunc(τ, p)` と `sub(τ, p)` は値を返し、`under(τ, p)` の各要素
`unit` について `trunc(τ, unit)` も値を返す。

<1>1. `λ ∈ leaves(τ)` とし、`m_λ` を `τ` の `λ` に沿う歩みの長さとする。このとき `m_λ ≤ |λ|` であり、
      `i < m_λ` の各位置で `step(cur_i)` は `Fields { held_fields, .. }` であって `held_fields` は添字
      `λ[i]` の対を含み、さらに (A) `step(cur_{m_λ}) = Unit` であるか、(B) `m_λ < |λ|` かつ
      `step(cur_{m_λ}) = Capture { capture_idx, .. }` かつ `λ[m_λ] = capture_idx` であるかの
      どちらかである。
  BY L9a

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
    BY <2>1, L1
  <2>3. CASE `m_λ < |p|`
    `<2>1` より `k = m_λ` であり、`τ` の `p` に沿う歩みの `cur_{m_λ}` は `λ` のものと同じである。
    `<1>1` の (A) では `step(cur_{m_λ}) = Unit` なので、`τ` の `p` に沿う歩みの長さは `m_λ < |p|` で
    あり、L1 の場合 (b) の `Unit` の行より `sub(τ, p) = None`、`trunc(τ, p) = p[0..m_λ]` である。(B) では
    `step(cur_{m_λ}) = Capture { capture_idx, .. }` かつ `λ[m_λ] = capture_idx` であり、`m_λ < |p|` かつ
    `m_λ < |λ|` で `p` と `λ` は比較可能なので `p[m_λ] = λ[m_λ] = capture_idx` である。L1 の場合 (b) の
    `Capture` (`p[m] = capture_idx`) の行より `sub(τ, p) = None`、`trunc(τ, p) = p[0..m_λ+1]` である。
    どちらも値である。
    BY <1>1, <2>1, L1
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
  BY <1>2, <1>3, L2, L5, L6

<1>5. QED
  `covered(τ, p)` の元 `λ` は `p ⊑ λ` か `λ ⊑ p` を満たす。
  BY <1>2, <1>3, <1>4, CODE src/rc_ir/borrow.rs: covered_leaves

### L11 (`covered_leaves` が空でなければ、それを所有すれば足りる)

**言明**。`r` を `pty(r) = Some(τ)` である名前、`p` を path とし、`OL` を `VarPath` の集合とする。
`covered(τ, p) ⊆ { λ : (r, λ) ∈ OL }` であり、かつ `covered(τ, p) ≠ ∅` または `under(τ, p) = []` である
とき、`owns_object_yet(vars, type_env, r, p, OL)` は真である。

<1>0. `under(τ, p)` と、その各要素についての `trunc(τ, ・)` は中断しない。
  `covered(τ, p) ≠ ∅` のときは L10 による。`under(τ, p) = []` のときは、L2 より `sub(τ, p)` が
  `Some` を返しており、L5 と L6 よりその要素の `trunc` も中断しない -- 要素が無いので空に成り立つ。
  BY L2, L5, L6, L10

<1>1. `owns_object_yet(vars, type_env, r, p, OL)` は、`under(τ, p)` の各要素 `unit` について
      「`trunc(τ, unit)` を鍵 `key` とし、`leaves(τ)` のうち `trunc(τ, ・) = key` を満たし `(r, ・) ∈ OL`
      である leaf が存在する」を要求する。
  BY CODE src/rc_ir/borrow.rs: owns_object_yet

<1>2. CASE `under(τ, p) = []`
  `<1>1` の全称は空なので真である。
  BY <1>1

<1>3. CASE `sub(τ, p) = Some(σ)` かつ `under(τ, p) ≠ []`
  <2>1. `under(τ, p)` の要素は `p ++ w` (`w ∈ units(σ)`) の形であり、`trunc(τ, p ++ w) = p ++ w` である。
    BY L2, L5, L6
  <2>2. 各 `w ∈ units(σ)` について、`λ_w ∈ leaves(σ)` で `trunc(σ, λ_w) = w` であるものが取れる。
    BY P1
  <2>3. `p ++ λ_w ∈ covered(τ, p)` であり `trunc(τ, p ++ λ_w) = p ++ w` である。
    L5 より `p ++ λ_w ∈ leaves(τ)` であり、`p ⊑ p ++ λ_w` なので `covered` の条件を満たす。
    L5 と `<2>2` より `trunc(τ, p ++ λ_w) = p ++ trunc(σ, λ_w) = p ++ w` である。
    BY L5, <2>2, CODE src/rc_ir/borrow.rs: covered_leaves
  <2>4. QED
    `<2>1` の各 `unit = p ++ w` について、`<2>3` の `p ++ λ_w` が `<1>1` の要求する leaf である。仮定より
    `covered(τ, p) ⊆ { λ : (r, λ) ∈ OL }` なので `(r, p ++ λ_w) ∈ OL` である。
    BY <1>1, <2>1, <2>3

<1>4. CASE `sub(τ, p) = None`
  <2>1. `under(τ, p) = [p]` であり、`key = trunc(τ, p)` である。
    BY L2, <1>1
  <2>2. `τ` の `p` に沿う歩みの長さを `m` とすると `m < |p|` であり、`trunc(τ, p)` が値を返すので
        `unit_step(cur_m)` は `Unit` か `Capture` (`p[m] = capture_idx`) であり、`key` はそれぞれ
        `p[0..m]`、`p[0..m+1]` である。
    `<1>0` と `<2>1` より `under(τ, p) = [p]` の唯一の要素についての `trunc(τ, p)` は中断しない。
    `sub(τ, p) = None` なので L1 の場合 (a) は当てはまらず、場合 (b) の 5 行のうち `trunc` が値を返す
    のは `Unit` の行と `Capture` (`p[m] = capture_idx`) の行だけである。
    BY <1>0, <2>1, L1
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
    BY <2>2, L1, L9a, DEF 歩み, CODE src/rc_ir/borrow.rs: covered_leaves
  <2>4. `covered(τ, p)` の各要素 `λ` について `trunc(τ, λ) = key` である。
    `|λ| ≤ m` のときは `<2>3` による。`|λ| > m` のとき、`λ` は `p` と比較可能なので `λ[0..m] = p[0..m]`
    であり、`λ[m] = p[m]` である (`|λ| > m` かつ `|p| > m` で、短い方が長い方の接頭辞)。DEF 歩み より
    `i < m` の各位置で `unit_step(cur_i)` は `Fields` であって `held_fields` は添字 `p[i] = λ[i]` の対を
    含むので、`τ` の `λ` に沿う歩みも同じ型を辿り、`unit_step(cur_m)` は `<2>2` と同じものであるから、
    その長さは `m` であって `m < |λ|` である。L1 の場合 (b) より `trunc(τ, λ)` は `Unit` の行なら
    `λ[0..m] = p[0..m]`、`Capture` の行なら `λ[0..m+1] = p[0..m+1]` であり、`<2>2` よりどちらも `key` に
    等しい。
    BY <2>2, <2>3, L1, DEF 歩み
  <2>5. QED
    `<2>1` より `under(τ, p) = [p]` は空でないので、言明の仮定の 2 つの選言肢のうち成り立つのは
    `covered(τ, p) ≠ ∅` の側である。よって `<2>4` の `λ` が 1 つ取れて `trunc(τ, λ) = key` であり、
    仮定より `(r, λ) ∈ OL` である。これが `<1>1` が `unit = p` について要求するものである。
    BY <1>1, <2>1, <2>4

<1>5. QED
  `<1>0` より `under(τ, p)` は中断せず、L2 より `sub(τ, p)` が中断すれば `under(τ, p)` も中断するので、
  `sub(τ, p)` は `Some` を返すか `None` を返すかのどちらかである。`under(τ, p) = []` の場合を `<1>2` が、
  `under(τ, p) ≠ []` の場合を `sub` の答えで分けて `<1>3` と `<1>4` が扱った (`sub(τ, p) = None` のとき
  L2 より `under(τ, p) = [p]` であり、これは `[]` ではない)。
  BY <1>0, <1>2, <1>3, <1>4, L2

## 3. P7e の証明

**言明** (README の P7e を訂正した形)。任意の root `r` と path `p` について、次の 2 つが成り立つ。

- **(a)** `pty(r) = Some(τ)` のとき `τ = ty(r)` であり、`owns(r, p) = owns(r, trunc(ty(r), p))` である。
  すなわち両辺は同時に値を返してその値が等しいか、同時に中断する。
- **(b)** `pty(r) = None` のとき、`owns(r, p)` は `p` によらず真である。さらに `trunc(ty(r), p)` が値 `t`
  を返すならば `owns(r, t)` も真である。

**README の形との違い**。README の P7e は (a) と (b) を 1 つの等式にまとめて述べる。その形は
`pty(r) = None` かつ `trunc(ty(r), p)` が中断する `(r, p)` で偽である -- 左辺は値 (真) を返し、右辺は
引数の評価で中断するので、第 1 節の `=` の条件をどちらの向きにも満たさない。第 0 節の冒頭がその `(r, p)` を
挙げる。この文書の中で P7e を読むのは L21 の `<1>1` と L22 の `<1>2` であり、どちらも (a) と (b) で
足りる。

<1>1. CASE `pty(r) = None` -- 言明の (b)
  <2>1. `owns(r, q)` は、任意の `q` について真を返す。
    `owns_object` は `self.vars.param_tys.get(root)` で分岐し、`None` の腕で `true` を返す。この腕は
    `path` を読まない。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object の `None` の腕
  <2>2. QED
    `<2>1` を `q = p` に当てると `owns(r, p)` は真である。`trunc(ty(r), p)` が値 `t` を返すならば、
    `<2>1` を `q = t` に当てて `owns(r, t)` も真である。
    BY <2>1

<1>2. CASE `pty(r) = Some(τ)` -- 言明の (a)
  <2>1. `τ = ty(r)` である。
    `VarTable::of` は各パラメータ・capture について `param_tys` と `var_tys` に同じ型 `p.ty` を入れ、
    `collect_bindings` は `param_tys` に何も入れない。よって `pty(r) = Some(τ)` なら `r` はこの版の
    パラメータか capture であり、`τ` はその宣言された型、すなわち `ty(r)` である。
    BY CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings
  <2>2. `owns(r, q) = ` 「`under(τ, q)` のすべての要素 `u` について `(r, trunc(τ, u)) ∈ OU`」である。
    `owns_object` の `Some(root_ty)` の腕は、`units_under(root_ty, path, self.type_env)` の各要素 `unit`
    について `self.owned_units.contains(&(root.clone(), truncate_to_unit(root_ty, unit, self.type_env)))`
    を要求する。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object の `Some(root_ty)` の腕
  <2>3. CASE L1 の場合 (a) が `(τ, p)` に当てはまる
    L1 (a) より `trunc(τ, p) = p` であるから、右辺は `owns(r, p)`、すなわち左辺と同じ呼び出しである。
    BY L1
  <2>4. CASE L1 の場合 (b) の `Unit` の行または `Capture` (`p[m] = capture_idx`) の行が `(τ, p)` に
        当てはまる
    <3>1. `t = trunc(τ, p)` は値を返す。
      BY L1
    <3>2. `sub(τ, p) = None` であり、`under(τ, p) = [p]` である。
      BY L1, L2
    <3>3. `owns(r, p) = ((r, t) ∈ OU)` である。
      `<3>2` より `under(τ, p)` の唯一の要素は `p` であり、`<2>2` よりその要素に掛けるのは
      `trunc(τ, p) = t` である。
      BY <2>2, <3>1, <3>2
    <3>4. `under(τ, t) = [t]` であり `trunc(τ, t) = t` である。
      BY <3>1, L4
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
      BY L1
    <3>2. `owns(r, p)` は中断する。
      `Fields` の行では L1 より `sub(τ, p)` が中断し、L2 より `under(τ, p)` が中断するので、`<2>2` の
      左辺の評価が中断する。残る 2 行では L1 より `sub(τ, p) = None` なので L2 より
      `under(τ, p) = [p]` であり、`<2>2` はその要素に `trunc(τ, p)` を掛ける。これは `<3>1` より
      中断する。
      BY <2>2, <3>1, L1, L2
    <3>3. QED
      両辺が中断する。
      BY <3>1, <3>2
  <2>6. QED
    L1 は `(τ, p)` について場合 (a) と場合 (b) の 5 行に場を尽くして分ける。`<2>3` が (a) を、`<2>4` が
    (b) の 2 行を、`<2>5` が (b) の残り 3 行を扱った。
    BY <2>3, <2>4, <2>5, L1

<1>3. QED
  `pty(r)` は `Option` なので `None` か `Some(τ)` のどちらかである。`<1>1` が前者について言明の (b) を、
  `<1>2` が後者について言明の (a) を与えた。
  BY <1>1, <1>2


## 4. `origin` の候補についての補題

**DEF 再帰で訪れる対**
1 つの本体とその `vars` を固定する。対 `(x, π)` について、集合 `Reach(x, π)` を、`(x, π)` を含み次の規則で
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

### L11a (`Reach` の要素数は整礎な尺度である)

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
    表で「呼ぶ相手」が無いのは `vars.bindings.get(・)` が `None` / `Param` / `Producer` の行なので、
    相手を持つ行の変数の `vars.bindings` は `Move` / `Join` / `Field` / `Payload` / `Llvm` のいずれかで
    ある。`VarTable::of` が `bindings` に入れるのはパラメータ・capture についての `Binding::Param` だけで
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
    ある。
    BY <2>1, A11, P2, CODE src/rc_ir/ownership.rs: origin, origin_inner, VarTable::empty

<1>3. QED
  `<1>1` と `<1>2` より `Reach(y, ρ) ⊊ Reach(x, π)` である。DEF 再帰で訪れる対 より `Reach(x, π)` は
  有限なので、その要素数は自然数であり、狭義の包含は要素数を狭義に減らす。
  BY <1>1, <1>2, DEF 再帰で訪れる対

### L12 (候補は訪れた対である)

**言明**。`act(x, π) ⊆ Reach(x, π)` である。とくに `cand(x, π) ⊆ Reach(x, π)` である。

<1>1. `Origin` の値を作るのは、`here()` すなわち `Origin::Exactly((var, path))`、
      `origin_from_leaves_under` の `Origin::Exactly(here.clone())`、`Origin::of_candidates(S, id)`、
      および部分結果をそのまま返す 3 か所 (`Move` / `Field` / `Payload` / 単一 `Arg` の `Llvm` の腕が返す
      `origin(...)`、`origin_from_leaves_under` が返す `first.clone()`) だけである。
  BY CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under, Origin::of_candidates

<1>2. `Origin::Exactly(p)` について `act = cand = {p}` であり、`Origin::Join { identity, candidates }`
      について `cand = candidates`、`act = candidates ∪ {identity}` である。
  `candidates()` は `Exactly(p)` に `vec![p]`、`Join` に `candidates` を返す。`acted_on()` は
  `identity()` を先頭に、それと異なる `candidates()` の元を続けた列を返す。
  BY CODE src/rc_ir/ownership.rs: Origin::candidates, Origin::acted_on, Origin::identity

<1>3. `of_candidates(S, id)` の返す `Origin` について `act ⊆ S ∪ {id}` である。
  `|S| = 1` のとき `Exactly` を返し `act = S`。`|S| ≥ 2` のとき `Join { identity: id, candidates: S }` を
  返し `<1>2` より `act = S ∪ {id}`。`S` が空のときは `assert!` で中断する。
  BY <1>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates

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
    BY <1>1, <1>3, <2>1, <2>2, <2>3, L11a

<1>5. QED
  `<1>4` を `(y, ρ) = (x, π)` に当てる。`cand ⊆ act` は `<1>2` による。
  BY <1>2, <1>4

### L13 (本体が束縛する変数はパラメータではない)

**言明**。`vars.bindings.get(w)` が `Some(Binding::Move(..))`、`Some(Binding::Producer)`、
`Some(Binding::Llvm(..))`、`Some(Binding::Field(..))`、`Some(Binding::Payload(..))`、
`Some(Binding::Join(..))` のいずれかであるとき、`pty(w)` は `None` である。したがって `owns(w, σ)` と
`yet(w, σ)` はどちらも、任意の `σ` について真である。

とくに `Origin::Join` の `identity` の変数はこの形である。`Origin::Join` を作るのは `of_candidates` だけで
あり、その第 2 引数を渡すのは `Binding::Join` の腕の `(var, path)` と、`Binding::Llvm` の腕が
`origin_from_leaves_under` へ渡す `here_identity = (var, path)` の 2 か所だからである。

<1>1. `vars.param_tys` の鍵は、この本体のパラメータ・capture の名前ちょうどであり、その各名前の
      `vars.bindings` は `Binding::Param` である。
  `VarTable::of` は各パラメータ・capture について `bindings` に `Binding::Param` を、`param_tys` にその型を
  入れる。`collect_bindings` は `bindings` と `var_tys` にしか入れず、`Binding::Param` を作らない。
  `VarTable::body_only` はパラメータを持たないので `param_tys` は空である。
  BY CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only, collect_bindings

<1>2. `<1>1` のパラメータ・capture の名前は、`collect_bindings` が記録する束縛名と異なる。
  `collect_bindings` が `bindings` に入れるのは、本体の `Let` の束縛変数、`Destructure` のフィールド変数、
  `Match` のアームの payload 変数である。A6 よりこれらの名前は互いに、またパラメータ・capture の名前とも
  異なる。
  BY A6, CODE src/rc_ir/ownership.rs: collect_bindings

<1>3. QED
  言明の 6 つの `Binding` はいずれも `collect_bindings` が入れるものであり、`Binding::Param` ではない。
  `<1>1` と `<1>2` より `w` は `param_tys` の鍵ではない。`owns_object` は `param_tys.get(root)` が
  `None` のとき真を返し、`owns_object_yet` も同じ条件で真を返す。`Origin::Join` の `identity` については、
  `of_candidates` を呼ぶ 2 か所がどちらも `vars.bindings.get(var)` の `match` の `Binding::Join` /
  `Binding::Llvm` の腕の中にある。
  BY <1>1, <1>2, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet,
     CODE src/rc_ir/ownership.rs: Origin::of_candidates, origin_inner, origin_from_leaves_under

### L14 (訪れた対は leaf に届く)

**言明**。`u ∈ units(ty(v))` とする。`Reach(v, u)` の各要素 `(y, ρ)` について
`covered(ty(y), ρ) ≠ ∅` である。

<1>1. 基底: `covered(ty(v), u) ≠ ∅` である。
  L7 より `Λ_{ty(v)}(u) ≠ ∅` であり、その各元 `λ` は `u ⊑ λ` を満たすので `covered(ty(v), u)` の条件を
  満たす。
  BY L7, CODE src/rc_ir/borrow.rs: covered_leaves

<1>2. `Binding::Move(w)` と `Binding::Payload(s, None)` の辺は性質を保つ。
  どちらも path を変えず、A12 より move-bind の両辺の型は等しく、catch-all アームの payload と
  scrutinee の型も等しいので `ty(w) = ty(y)`、`ty(s) = ty(y)` である。
  BY A12

<1>3. `Binding::Join(arm_results)` の辺は性質を保つ。
  path を変えず、A12 よりアームの結果と `Match` の束縛変数の型は等しい。
  BY A12

<1>4. `Binding::Field(c, idx)` (`c` が unbox) の辺は性質を保つ。
  <2>1. `covered(ty(y), ρ)` から `λ` を取る。`[idx] ++ λ` は `[idx] ++ ρ` と比較可能である。
    `ρ ⊑ λ` なら `[idx] ++ ρ ⊑ [idx] ++ λ`、`λ ⊑ ρ` なら `[idx] ++ λ ⊑ [idx] ++ ρ` である。
    BY 仮定
  <2>2. `[idx] ++ λ ∈ leaves(ty(c))` である。
    A12 より `ty(c)` は構造体であり、この腕の条件より boxed ではない。`is_struct` は
    `toplevel_tycon_info` を読むので `is_closure(ty(c))` は偽であり、構造体の `TyConInfo` の `variant` は
    `TyConVariant::Struct` で、`Std::Array` の tycon の `variant` は `TyConVariant::Array`、
    `Std::#FunPtr{n}` の tycon の `variant` は `TyConVariant::Primitive` なので、`is_array(ty(c))` も
    `is_funptr(ty(c))` も偽である。`λ ∈ leaves(ty(y))` より `leaves(ty(y)) ≠ ∅` なので L8 より
    `is_fully_unboxed(ty(y))` は偽であり、A12 より `ty(y)` は `ty(c)` の第 `idx` フィールドの型であって、
    そのフィールドは `ty(c)` が持つフィールド (`unpunched_field_types` が返すもの) である。
    `is_fully_unboxed` は `is_box` / `is_closure` / `is_array` で偽を、`is_funptr` で真を返し、それ以外
    では `unpunched_field_types` の全対についての全称であり、`ty(c)` はこの 4 つの述語をどれも満たさない
    ので全称の腕に入る。よって `is_fully_unboxed(ty(c))` も偽である。
    よって `boxed_leaf_paths` の `go` は `ty(c)` について `unpunched_field_types` の下へ降り、第 `idx`
    フィールドについて `ty(y)` から始めた `go` の結果の前に `idx` を置いたものを積む。
    BY A12, L8, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_struct,
       TypeNode::is_array, TypeNode::is_funptr, CODE src/fixstd/builtin.rs: bulitin_tycons,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. QED
    `<2>1` と `<2>2` より `[idx] ++ λ ∈ covered(ty(c), [idx] ++ ρ)` である。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: covered_leaves

<1>5. `Binding::Payload(s, Some(t))` (`s` が unbox) の辺は性質を保つ。
  <2>1. `covered(ty(y), ρ)` から `λ` を取る。`[t] ++ λ` は `[t] ++ ρ` と比較可能である。
    `ρ ⊑ λ` なら `[t] ++ ρ ⊑ [t] ++ λ`、`λ ⊑ ρ` なら `[t] ++ λ ⊑ [t] ++ ρ` である。
    BY 仮定
  <2>2. `[t] ++ λ ∈ leaves(ty(s))` である。
    A12 より `ty(s)` は union であり、`Match` が名指す変位はその型が実際に持つものであって、`ty(y)` は
    その第 `t` 変位の型である。`is_union` は `toplevel_tycon_info` を読むので `is_closure(ty(s))` は偽で
    ある。union の `TyConInfo` の `variant` は `TyConVariant::Union` であり、`Std::Array` の tycon の
    `variant` は `TyConVariant::Array`、`Std::#FunPtr{n}` の tycon の `variant` は
    `TyConVariant::Primitive` なので、`is_array(ty(s))` も `is_funptr(ty(s))` も偽である。
    この腕の条件より `ty(s)` は boxed ではない。`λ ∈ leaves(ty(y))` より `leaves(ty(y)) ≠ ∅` なので L8 より
    `is_fully_unboxed(ty(y))` は偽である。`is_fully_unboxed` は `is_box` / `is_closure` / `is_array` で
    偽を、`is_funptr` で真を返し、それ以外では `unpunched_field_types` の全対についての全称である。
    `ty(s)` はこの 4 つの述語をどれも満たさないので全称の腕に入り、`ty(y)` がその 1 つなので
    `is_fully_unboxed(ty(s))` も偽である。よって `boxed_leaf_paths` の
    `go` は `ty(s)` について `unpunched_field_types` の下へ降り、第 `t` 変位について `ty(y)` から始めた
    `go` の結果の前に `t` を置いたものを積む。
    BY A12, L8, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_union,
       TypeNode::is_array, TypeNode::is_funptr, CODE src/fixstd/builtin.rs: bulitin_tycons,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. QED
    `<2>1` と `<2>2` より `[t] ++ λ ∈ covered(ty(s), [t] ++ ρ)` である。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: covered_leaves

<1>6. `Binding::Llvm` の単一 `Arg(j, σ)` の辺は性質を保つ。
  A3 より、単一の `Arg(j, σ)` の宣言は第 `j` オペランドの leaf `σ` を名指す。すなわち
  `σ ∈ leaves(ty(args[j]))` である。`σ ⊑ σ` なので `σ ∈ covered(ty(args[j]), σ)` である。
  BY A3, CODE src/rc_ir/borrow.rs: covered_leaves

<1>7. `Binding::Llvm` の `origin_from_leaves_under` の辺は性質を保つ。
  `operand_units` の元は `(j, truncate_to_unit(&args[j].ty, leaf, type_env))` の形であり、`leaf` は
  `LeafOrigin::Arg(j, leaf)` の宣言が名指す第 `j` オペランドの leaf である (A3)。`trunc` の答えは引数の
  接頭辞なので `w := trunc(ty(args[j]), leaf) ⊑ leaf` であり、`leaf ∈ leaves(ty(args[j]))` なので
  `leaf ∈ covered(ty(args[j]), w)` である。
  BY A3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under, truncate_to_unit,
     CODE src/rc_ir/borrow.rs: covered_leaves

<1>8. QED
  DEF 再帰で訪れる対 の表は辺を尽くす。呼ぶ相手が無い 4 行は新しい対を作らない。残る 7 行を `<1>2` から
  `<1>7` が扱った (`Move` と catch-all の `Payload` を `<1>2` が、`Join` を `<1>3` が、unbox 容器の
  `Field` を `<1>4` が、unbox union の変位アームの `Payload` を `<1>5` が、`Llvm` の 2 行を `<1>6` と
  `<1>7` が)。基底は `<1>1` である。`Reach(v, u)` は最小の閉じた集合なので、この帰納が全体を覆う。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, DEF 再帰で訪れる対

### L14a (訪れた対の変数は本体に現れる)

**言明**。`x` を `func` に現れる名前 (第 1 節の (N)) とする。`Reach(x, π)` の各要素 `(y, ρ)` について、
`y` も `func` に現れる名前である。

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
  L11a の `Reach` についての帰納。基底は `(x, π)` であり、`x` は仮定より `func` に現れる。段では、
  DEF 再帰で訪れる対 の表が `(y, ρ)` から進む相手の変数は、`vars.bindings.get(y)` が持つ `RcVar`
  (`Move` の `w`、`Join` の各アーム結果、`Field` の容器、`Payload` の scrutinee、`Llvm` の `args[j]`) の
  名前であり、`<1>1` よりそれは `func` に現れる名前である。
  BY <1>1, L11a, DEF 再帰で訪れる対

### L15 (借用版は名前替えである)

**言明**。`func` を入力の関数、`clone` をその借用版、`rename` を `clone_func` が返す写像とし、`ρ` を
`rename` を `rename` の鍵でない名前上の恒等写像で延ばしたものとする。`vars_f = VarTable::of(func)`、
`vars_c = VarTable::of(clone)` と置く。このとき次が成り立つ。

- `ρ` は `func` に現れる名前 (第 1 節の (N)) の上で単射であり、`func` に現れる名前のうち `rename` の鍵で
  ないものは `rename` の像に入らない。`vars_c.param_tys` の鍵は `vars_f.param_tys` の鍵の `ρ` による像
  ちょうどであって、`vars_c.param_tys[ρ(p)] = vars_f.param_tys[p]` である。
- `func` に現れる各名前 `x` と任意の path `π` について
  `origin(vars_c, type_env, ρ(x), π) = ρ(origin(vars_f, type_env, x, π))` である。
  ここで `ρ` は `VarPath` に対しその変数だけを写す。

<1>1. `clone` の本体・パラメータ・capture は、`func` のそれの束縛変数を `rename` で一斉に付け替えたもの
      であり、それ以外の違いを持たない。
  BY P9, CODE src/rc_ir/borrow.rs: clone_func

<1>2. `ρ` は `func` に現れる名前の上で単射であり、`func` に現れる名前のうち `rename` の鍵でないものは
      `rename` の像に入らない。
  <2>1. `rename` の鍵は `func` のパラメータ・capture の名前と、`func.body` の `Let` の束縛変数・
        `Destructure` のフィールド変数・`Match` のアームの payload 変数の名前ちょうどであり、いずれも
        `func` に現れる名前である。
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
    (N) は、`rename` の像のどの名前も `func` に現れるどの名前とも異なると述べる。
    BY (N)
  <2>4. QED
    `func` に現れる 2 つの相異なる名前 `n1`、`n2` を取る。どちらも `rename` の鍵ならば `<2>2` より
    `ρ(n1) ≠ ρ(n2)` である。どちらも鍵でないならば `ρ(n1) = n1 ≠ n2 = ρ(n2)` である。`n1` だけが鍵
    ならば、`ρ(n1)` は `rename` の像の元であり、`ρ(n2) = n2` は `<2>3` より像に入らないので異なる。
    言明の後半 (`func` に現れる名前のうち鍵でないものは像に入らない) は `<2>3` の特別な場合である。
    BY <2>2, <2>3

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
  payload 変数について `bindings` を入れる。これは `<1>2` の `<2>1` が挙げる `rename` の鍵ちょうどで
  ある。`collect_bindings` が `Binding` に記録するのは、右辺に現れる変数 (`Move` の `y`、`Field` の
  容器、`Payload` の scrutinee、`Join` のアーム結果、`Llvm` のオペランド) である。`<1>1` より `clone` の
  パラメータ・capture と本体はこれらをすべて `ρ` で写したものであり、型は変わらない。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings,
     CODE src/rc_ir/rename.rs: rename_expr, rename_var

<1>5. QED
  `origin_inner` が読むのは `vars.bindings`、`vars.param_tys` を経由しない情報 (`args[j].ty`、
  `container.ty`、`scrut.ty`、`result_ty`)、および `type_env` だけである。`LLVMGen::result_prov` は
  `(result_ty, arg_tys, type_env)` だけを引数に取るので、オペランドの名前を読まない。`func` に現れる
  名前 `x` について、`<1>4` より `vars_f.bindings.get(x)` が `None` であることと
  `vars_c.bindings.get(ρ(x))` が `None` であることは同値であり (`vars_c.bindings` の鍵は
  `vars_f.bindings` の鍵の `ρ` による像ちょうどで、`<1>2` より `ρ` は `func` に現れる名前の上で単射で
  あって、鍵でない名前の像は像に入らない)、その腕はどちらも `here()` を返す。`Some` の腕でも `<1>4` より
  読む値が `ρ` の下で対応するので、`origin_inner` の各腕の答えは `ρ` で写り、`origin` の再帰も同じ形で
  写る。A15 と P2 より再帰は停止するので、この対応は全域である。
  BY <1>2, <1>3, <1>4, A15, P2, CODE src/rc_ir/ownership.rs: origin, origin_inner,
     origin_from_leaves_under, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov

### L16 (借用版の `owns_object` は推論の `owns_object_yet` である)

**言明**。`OL` を `infer_ownership` の不動点の `owned_leaves` とし、`owned_units` を `borrow_ify` が組む
集合、`ctx` を `clone` (`func` の借用版) の `RewriteCtx` とする。このとき、`func` に現れる (第 1 節の
(N)) 任意の名前 `r` と任意の path `p` について、
`ctx.owns_object(ρ(r), p) = owns_object_yet(vars_f, type_env, r, p, OL)` である
(両辺は同時に中断する)。

<1>1. `pty_f(r) = None` のとき、両辺とも真である。
  L15 より `vars_c.param_tys` の鍵は `vars_f.param_tys` の鍵の `ρ` による像ちょうどである。`r` は
  `func` に現れる名前であり、`vars_f.param_tys` の鍵でもない。L15 より `ρ` は `func` に現れる名前の上で
  単射であり、`vars_f.param_tys` の鍵はどれも `func` に現れる名前 (パラメータ・capture の名前) なので、
  `ρ(r)` はそれらの像のどれとも異なる。よって `ρ(r)` は `vars_c.param_tys` の鍵でない。`owns_object` は
  `param_tys.get(root)` が `None` の腕で真を返し、`owns_object_yet` も `param_tys.get(root)` が `None` の
  ときに真を返す。
  BY L15, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet

<1>2. `pty_f(r) = Some(τ)` のとき、`ctx.vars.param_tys[ρ(r)] = τ` である。
  BY L15

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
    BY P9, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func,
       CODE src/rc_ir/rename.rs: fresh_rename_function, assign_fresh_name
  <2>2. QED
    `OwnedLeaves::owns(var, path)` は `(var, path) ∈ OL` である。
    BY <2>1, CODE src/rc_ir/borrow.rs: OwnedLeaves::owns

<1>4. QED
  `pty_f(r)` は `Option` なので `None` か `Some(τ)` のどちらかである。`None` の場合は `<1>1` が両辺とも
  真であることを与える。`Some(τ)` の場合、`<1>2` より `owns_object(ρ(r), p)` は「`under(τ, p)` の各
  `unit` について `(ρ(r), trunc(τ, unit)) ∈ owned_units`」であり、`<1>3` よりこれは「各 `unit` について、
  `trunc(τ, leaf) = trunc(τ, unit)` かつ `(r, leaf) ∈ OL` である `leaf ∈ leaves(τ)` が在る」に等しい。
  これが `owns_object_yet(vars_f, type_env, r, p, OL)` の定義そのものである。両辺は `under(τ, p)` と
  `trunc(τ, ・)` を同じ引数で呼ぶので、中断も同時である。
  BY <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet

## 5. P7d の証明

**言明** (README の P7d)。`infer_ownership` の不動点において、`levelled_sites` が挙げる各 site `(v, u)` に
ついて、`origin(v, u)` の候補は、すべて `owns_object` が真であるか、すべて偽であるかのどちらかである。

**読み方**。`owns_object` は `RewriteCtx` の method なので、この言明はある出力版の `RewriteCtx` について
読む。`borrow_ify` が作る `RewriteCtx` は 3 種である。入力の各関数の全所有版 `f_own`、借用版を持つ関数の
借用版、および各グローバル初期化子のものである
(`CODE src/rc_ir/borrow.rs: borrow_ify`)。`levelled_sites` は `prog.funcs` の各関数について計算される
(`CODE src/rc_ir/borrow.rs: infer_ownership`)。以下では、`f_own` については `levelled_sites(func)` の
site を、借用版については `levelled_sites(clone)` の site を主語とし、グローバル初期化子については
「`origin(v, u)` の候補」を任意の `(v, u)` について読む。

<1>1. `levelled_sites(func)` の各 site `(v, u)` は `u ∈ units(ty(v))` を満たす。
  `levelled_sites` は 2 種の site を積む。`Retain(v, path)` / `Release(v, path)` の節点について
  `(v, path)`、および `Let(_, App(_, args), _)` の各引数 `arg` と各 `unit ∈ rc_units(arg.ty)` について
  `(arg, unit)`。後者は定義から `units(ty(arg))` の元である。前者は A2 より `path` が `ty(v)` の
  `rc_units` の元である。
  BY A2, CODE src/rc_ir/borrow.rs: levelled_sites

<1>1a. `levelled_sites(func)` の各 site `(v, u)` について、`v.name` は `func` に現れる名前 (第 1 節の
       (N)) であり、`cand(vars_f, v, u)` の各元 `(r, p)` の `r` も `func` に現れる名前である。
  `levelled_sites` は `for_each_node` で本体を歩き、`Retain`/`Release` の名指す変数と `RcRhs::App` の各
  引数を積む。`for_each_var` は同じ `for_each_node` の歩きの各節点について `for_each_var_of_node` と
  `for_each_var_of_rhs` を呼び、この 2 つは `Retain`/`Release` の名指す変数と `App` の各引数を訪れるので、
  `v.name` は `func` に現れる名前である。L12 より `cand(vars_f, v, u) ⊆ Reach(v.name, u)` であり、L14a より
  `Reach(v.name, u)` の各要素の変数は `func` に現れる名前である。
  BY L12, L14a, CODE src/rc_ir/borrow.rs: levelled_sites,
     CODE src/rc_ir/ast.rs: for_each_node, for_each_var, for_each_var_of_node, for_each_var_of_rhs

<1>2. グローバル初期化子の `RewriteCtx` では、`owns_object` は任意の `(r, p)` について真を返す。
  その `RewriteCtx` の `vars` は `VarTable::body_only` で作られ、その `param_tys` は空である。よって
  `owns_object` は `param_tys.get(root)` が `None` の腕に入り、`path` を読まずに真を返す。
  BY CODE src/rc_ir/borrow.rs: borrow_ify のグローバルを写す繰り返し,
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
    `<1>1` より `u ∈ units(ty(v))` であり、L12 より `(r, p) ∈ Reach(v, u)`、L14 より
    `covered(τ, p) ≠ ∅` である。L10 より `under(τ, p)` もその各要素についての `trunc(τ, ・)` も
    中断しない。`owns_object` が呼ぶのはこの 2 つと `owned_units.contains` だけである。
    BY <1>1, L10, L12, L14, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
  <2>4. `pty(r) = Some(τ)` のとき、`owns_object(r, p)` は真である。
    `owns_object` は `under(τ, p)` の各要素 `unit` について `(r, trunc(τ, unit)) ∈ owned_units` を
    要求する。L9 より `trunc(τ, unit) ∈ units(τ)` である。`borrow_ify` は入力の各関数について
    `owned_units.extend(param_capture_units(func))` を行い、`param_capture_units` は各パラメータ・capture
    `p` と各 `unit ∈ rc_units(p.ty)` について `(p.name, unit)` を並べる。`<2>1` より `r` は `func` の
    パラメータか capture であり `τ = ty(r)` なので、`(r, trunc(τ, unit))` はこの集合に入る。
    BY <2>1, <2>3, L9, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, borrow_ify, param_capture_units
  <2>5. QED
    BY <2>2, <2>3, <2>4

<1>3a. `f_own` の `RewriteCtx` とグローバル初期化子の `RewriteCtx` について、言明が成り立つ。
  `<1>2` と `<1>3` よりすべての候補について `owns_object` は真である。
  BY <1>2, <1>3

<1>4. 借用版の `RewriteCtx` について、`levelled_sites(clone)` の site は
      `{ (ρ(v), u) : (v, u) ∈ levelled_sites(func) }` であり、
      `cand(vars_c, ρ(v), u) = ρ(cand(vars_f, v, u))` である。
  `levelled_sites` は本体の節点を `for_each_node` で歩き、`Retain`/`Release` の変数と path、`App` の引数と
  その型の `rc_units` を積む。P9 より `clone` の本体は `func` の本体の束縛変数を `ρ` で
  付け替えたものであり、変数の型は変わらないので、積まれる対は `ρ` で写ったものちょうどである。候補の
  対応は L15 による。
  BY P9, L15, CODE src/rc_ir/borrow.rs: levelled_sites, CODE src/rc_ir/ast.rs: for_each_node

<1>5. 不動点において、各関数の各 site について `level_ownership(vars_f, type_env, (v, u), OL)` は `false` を
      返す。
  `infer_ownership` は、1 周で `changed` が偽になるまで繰り返す。最後の周では、各 site について
  `changed |= level_ownership(...)` が `changed` を変えなかったので、`level_ownership` は `false` を
  返した。その周の間 `owned_leaves` は変わらないので、その周が読んだ `owned_leaves` は不動点の `OL` で
  ある。
  BY CODE src/rc_ir/borrow.rs: infer_ownership

<1>6. CASE `level_ownership` の `owns_a_candidate` が偽である site
  <2>1. `cand(vars_f, v, u)` の各元 `(r, p)` について `yet(r, p)` は偽である。
    `owns_a_candidate` は候補についての `any` である。
    BY CODE src/rc_ir/borrow.rs: level_ownership
  <2>2. QED
    `<1>1a` より各候補の `r` は `func` に現れる名前なので L16 が使えて
    `ctx.owns_object(ρ(r), p) = yet(r, p)` であり、`<1>4` より借用版の site の候補は
    `ρ(cand(vars_f, v, u))` である。よってすべての候補について `owns_object` は偽である。
    BY <2>1, <1>1a, <1>4, L16

<1>7. CASE `level_ownership` の `owns_a_candidate` が真である site
  <2>1. `cand(vars_f, v, u)` の各元 `(r, p)` で `pty_f(r) = Some(τ)` であるものについて、
        `covered(τ, p) ⊆ { λ : (r, λ) ∈ OL }` である。
    `owns_a_candidate` が真のとき `level_ownership` は各候補について `param_tys` を引き、`Some(ty)` の
    ものについて `covered_leaves(ty, path, type_env)` の各 `leaf` を `owned_leaves.insert` に掛け、
    その返り値で `changed` を立てる。`<1>5` より `changed` は偽なので、どの `insert` も新しい元を
    加えなかった。すなわちすべての `leaf` はすでに `OL` に在った。
    BY <1>5, CODE src/rc_ir/borrow.rs: level_ownership
  <2>2. その各元について `covered(τ, p) ≠ ∅` である。
    `<1>1` より `u ∈ units(ty(v))` であり、L12 より `(r, p) ∈ Reach(v, u)` である。L14 より
    `covered(ty(r), p) ≠ ∅` であり、`ty(r) = τ` である。
    BY <1>1, L12, L14
  <2>3. その各元について `yet(r, p)` は真である。
    BY <2>1, <2>2, L11
  <2>4. `cand(vars_f, v, u)` の各元 `(r, p)` で `pty_f(r) = None` であるものについても `yet(r, p)` は
        真である。
    `owns_object_yet` は `param_tys.get(root)` が `None` のとき真を返す。
    BY CODE src/rc_ir/borrow.rs: owns_object_yet
  <2>5. QED
    `<2>3` と `<2>4` よりすべての候補について `yet` は真である。`<1>1a` より各候補の `r` は `func` に
    現れる名前なので L16 が使えて、`<1>4` と合わせてすべての候補について `owns_object` は真である。
    BY <2>3, <2>4, <1>1a, <1>4, L16

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

README の P7a は、`levelled_sites` が挙げる site `(v, u)` と `Λ(u) = Λ_{ty(v)}(u)` について、
`infer_ownership` の不動点の下で次の 3 つが同値である、という言明である。

1. `owns_unit(v, u)` が真である。
2. `Λ(u)` の**ある inhabited な** leaf `λ` の**すべての**候補 `(r, p)` について `owns_object(r, p)` が
   真である。
3. `Λ(u)` の**すべての inhabited な** leaf のすべての候補について `owns_object` が真である。

**読み方**。節 1 は静的である。節 2 と節 3 は inhabited (D16) を含むので、1 回の活性化 (D21) とその
実行路の 1 つの位置に相対的である。以下では活性化とその位置を固定し、`v` がその位置までに値を得ている
ものとする。その位置で inhabited な `Λ(u)` の leaf の集合を `Inh(v, u)` と書く。

**この節が証明するもの**。L17 (`owns_unit` を呼ぶ位置は site を出ない)、**節 1 から節 3**、および
**節 2 から節 1** である。この 2 つが README の解説が挙げる 2 つの役割 -- 「節点を残すのが安全である」と
「節点を落とすのが安全である」 -- を与える。

**この節が証明しないもの**。節 3 から節 2 と、節 3 から節 1 である。`Inh(v, u)` が空のとき節 3 は空虚に
真になるので、この 2 つは偽である。R2 がその本体を挙げる。`Inh(v, u) ≠ ∅` を足せば 3 つは同値になる。

R1 は、節 2 と節 3 の inhabited の限定が要ることを示す記録である。限定を外すと節 2 から節 1 へ渡れない。

### L17 (`owns_unit` を呼ぶ位置は site を出ない)

**言明**。ある出力版の `RewriteCtx` が `owns_unit(v, u)` を呼ぶとき、その版が関数の版であれば `(v, u)` は
その版の本体について `levelled_sites` が挙げる site である。その版がグローバル初期化子のものであれば、
`owns_unit(v, u)` は真を返す。

<1>1. `owns_unit` を呼ぶのは `any_owned_unit`、`routing_saves_retain`、`call_rc`、`rewrite_rc` の 4 か所で
      ある。
  `owns_unit` は `src/rc_ir/borrow.rs` の `impl RewriteCtx` の中で `fn` として宣言されている --
  `pub` も `pub(crate)` も付かない -- ので、Rust の可視性の規則により、その呼び出しはこのモジュールの中に
  しかない。このモジュールの中で識別子 `owns_unit` が現れるのは、この宣言と、`any_owned_unit` の
  `rc_units(&arg.ty, ..).iter().any(|unit| self.owns_unit(arg, unit))`、`routing_saves_retain` の
  `!(self.owns_unit(arg, unit) && ..)`、`call_rc` の `let arg_owned = self.owns_unit(arg, &unit);`、
  `rewrite_rc` の `.filter(|unit| self.owns_unit(v, unit))`、および 2 つの doc コメントである。
  BY CODE src/rc_ir/borrow.rs, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
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
  `rewrite_rc` は `!self.is_borrow_version` のとき節点をそのまま返して終わる。A2 より入力の
  `Retain`/`Release` の `path` は `units(ty(v))` の元であり、L6 より `under(ty(v), path) = [path]` である。
  BY A2, L6, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::rewrite_rc

<1>4. 関数の版では、`<1>2` と `<1>3` の `(v, u)` は `levelled_sites` が挙げる site である。
  `levelled_sites` は `for_each_node` で本体の全節点を歩き、`Retain(v, path, ..)` / `Release(v, path, ..)`
  について `(v, path)` を、`Let(_, App(_, args), _)` について各 `arg` と各
  `unit ∈ rc_units(arg.ty, type_env)` の対を積む。`for_each_node` は継続とアーム本体の両方へ降りるので、
  本体のすべての節点を訪れる。`rewrite_inner` は同じ木を継続とアーム本体へ降りて歩く。
  BY <1>2, <1>3, CODE src/rc_ir/borrow.rs: levelled_sites, RewriteCtx::rewrite_inner,
     CODE src/rc_ir/ast.rs: for_each_node

<1>5. グローバル初期化子の版では `owns_unit(v, u)` は真を返す。
  その `RewriteCtx` は `is_borrow_version: false` で作られるので `<1>3` の呼び出しは起きない。`<1>2` の
  呼び出しについては、`vars` が `VarTable::body_only` で作られ `param_tys` が空なので、`owns_object` は
  どの `(r, p)` にも真を返し、`owns_unit` はその全称なので真である。
  BY <1>2, CODE src/rc_ir/borrow.rs: borrow_ify のグローバルを写す繰り返し,
     CODE src/rc_ir/ownership.rs: VarTable::body_only, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object,
     RewriteCtx::owns_unit, RewriteCtx::rewrite_rc

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### L18 (unit を覆う対と、その下の leaf の写り方)

**DEF unit を覆う対**
対 `(x, π)` が **unit を覆う**とは、`Λ_{ty(x)}(π) ≠ ∅` であり、かつ `Λ_{ty(x)}(π)` の各 leaf `λ` について
`trunc(ty(x), λ) = trunc(ty(x), π)` が成り立つことをいう。

**言明**。

- **(a)** `u ∈ units(ty(v))` のとき `(v, u)` は unit を覆う。
- **(b)** `(x, π)` が unit を覆うとき、DEF 再帰で訪れる対 の表が `(x, π)` から進む各相手も unit を覆う。
  さらに `Λ` は次のように写る。

  | 進む相手 | `Λ` の写り方 |
  |---|---|
  | `Move(w)` の `(w, π)`、`Join` の `(a, π)`、catch-all `Payload` の `(s, π)` | `Λ_{ty(w)}(π) = Λ_{ty(x)}(π)` |
  | unbox 容器の `Field(c, idx)` の `(c, [idx] ++ π)` | `Λ_{ty(c)}([idx] ++ π) = { [idx] ++ λ : λ ∈ Λ_{ty(x)}(π) }` |
  | unbox union の `Payload(s, Some(t))` の `(s, [t] ++ π)` | `Λ_{ty(s)}([t] ++ π) = { [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` |
  | 単一 `Arg(j, σ)` の `(args[j], σ)` | `Λ_{ty(args[j])}(σ) = { σ }` |
  | `origin_from_leaves_under` の `(args[j], w)` | `w ∈ units(ty(args[j]))` |

<1>1. (a) が成り立つ。
  L7 より `Λ_{ty(v)}(u) ≠ ∅` であり、その各 leaf `λ` について `trunc(ty(v), λ) = u` である。L6 より
  `trunc(ty(v), u) = u` である。
  BY L6, L7

<1>2. `Move(w)`、`Join` のアーム結果 `a`、catch-all の `Payload` の scrutinee `s` については、A12 より
      その型は `ty(x)` に等しいので、`Λ` も `trunc` も変わらず、性質はそのまま移る。
  A12 は move-bind の両辺の型、アームの結果と `Match` の束縛変数の型、catch-all アームの payload と
  scrutinee の型が一致することを述べる。
  BY A12

<1>3. unbox 容器の `Field(c, idx)` について、`is_fully_unboxed(ty(c))`・`is_closure(ty(c))`・
      `is_box(ty(c))`・`is_array(ty(c))`・`is_union(ty(c))` はどれも偽であり、`ty(x)` は
      `unpunched_field_types(ty(c))` の添字 `idx` の対の第 2 成分である。したがって `step(ty(c))` は、
      `is_punched_array(ty(c))` が真ならば `Unit`、偽ならば `Fields { held_fields, .. }` であって
      `held_fields = unpunched_field_types(ty(c))` は添字 `idx` の対を含む。
  A12 より `ty(c)` は構造体であり、`Destructure` が名指すフィールドはその型が実際に持つ (punched でない)
  ものであって、`ty(x)` はその第 `idx` フィールドの型である。`is_struct` は `toplevel_tycon_info` を読む
  ので `is_closure(ty(c))` は偽である。構造体の `TyConInfo` の `variant` は `TyConVariant::Struct` なので
  `is_union(ty(c))` は偽であり、`Std::Array` の tycon の `variant` は `TyConVariant::Array`、
  `Std::#FunPtr{n}` の tycon の `variant` は `TyConVariant::Primitive` なので、`is_array(ty(c))` も
  `is_funptr(ty(c))` も偽である。この腕の条件より `ty(c)` は boxed ではない。`Λ_{ty(x)}(π) ≠ ∅` より
  `leaves(ty(x)) ≠ ∅` なので L8 より `is_fully_unboxed(ty(x))` は偽である。`is_fully_unboxed` は
  `is_box` / `is_closure` / `is_array` で偽を、`is_funptr` で真を返し、それ以外では
  `unpunched_field_types` の全対についての全称である。`ty(c)` はこの 4 つの述語をどれも満たさないので
  全称の腕に入り、`ty(x)` がその 1 つなので `is_fully_unboxed(ty(c))` も偽である。`unit_step` は上から順に
  `is_fully_unboxed`、`is_closure`、`is_box || is_union || is_array || is_punched_array` を見るので、
  残る分かれ目は `is_punched_array(ty(c))` だけである。**`Std::PunchedArray a` は
  `unbox struct { _arr : Array a, _idx : I64 }` なので、この場合は実際に起こりうる**
  (`CODE src/fixstd/std.fix: PunchedArray`)。
  BY A12, L8, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_struct, TypeNode::is_union,
     TypeNode::is_array, TypeNode::is_funptr, TypeNode::is_punched_array,
     CODE src/fixstd/builtin.rs: bulitin_tycons, CODE src/rc_ir/ownership.rs: unit_step

<1>3a. unbox 容器の `Field(c, idx)` について、`leaves(ty(c))` のうち `[idx]` を前置に持つものは
       `{ [idx] ++ μ : μ ∈ leaves(ty(x)) }` であり、したがって
       `Λ_{ty(c)}([idx] ++ π) = { [idx] ++ λ : λ ∈ Λ_{ty(x)}(π) }` であって空でない。
  `<1>3` より `is_fully_unboxed(ty(c))`・`is_closure(ty(c))`・`is_box(ty(c))`・`is_array(ty(c))` は
  どれも偽である。L1a (b) の欄より `go` はそのとき `unpunched_field_types(ty(c))` の各対の下へ降りる
  (`step(ty(c))` が `Unit` の場合も、`is_box` と `is_array` が偽なので `go` は降りる側である)。`go` は
  第 `idx` 対について `ty(x)` から始めた `go` の結果の前に `idx` を置いたものを積み、他の対の下で積む
  path は第 1 添字が `idx` と異なる。`Λ_{ty(x)}(π) ≠ ∅` なので像も空でない。
  BY <1>3, L1a, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>4. unbox 容器の `Field(c, idx)` について、`(c, [idx] ++ π)` は unit を覆い、`Λ` は表のとおりに写る。
  <2>1. CASE `step(ty(c)) = Fields { held_fields, .. }`
    `<1>3` より `held_fields` は添字 `idx` の対を含み、その第 2 成分は `ty(x)` なので、`sub` の第 1 周は
    `Fields` の腕を通って `cur` を `ty(x)` にし、`[idx]` を使い切る。すなわち
    `sub(ty(c), [idx]) = Some(ty(x))` である。L5 より、任意の `ρ` について
    `trunc(ty(c), [idx] ++ ρ) = [idx] ++ trunc(ty(x), ρ)` である。これと `(x, π)` が unit を覆うことから
    `trunc(ty(c), [idx] ++ λ) = [idx] ++ trunc(ty(x), λ) = [idx] ++ trunc(ty(x), π)
    = trunc(ty(c), [idx] ++ π)` が出る。
    BY <1>3, L5, CODE src/rc_ir/ownership.rs: subtree_type, held_field_type
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
  A12 より `ty(s)` は union であり、`Match` が名指す変位はその型が実際に持つものであって、`ty(x)` はその
  第 `t` 変位の型である。`is_union` は `toplevel_tycon_info` を読むので `is_closure(ty(s))` は偽である。
  union の `TyConInfo` の `variant` は `TyConVariant::Union` であり、`Std::Array` の tycon の `variant` は
  `TyConVariant::Array`、`Std::#FunPtr{n}` の tycon の `variant` は `TyConVariant::Primitive` なので、
  `is_array(ty(s))` も `is_funptr(ty(s))` も偽である。この腕の条件より `ty(s)` は boxed ではない。
  `Λ_{ty(x)}(π) ≠ ∅` より `leaves(ty(x)) ≠ ∅` なので L8 より `is_fully_unboxed(ty(x))` は偽である。
  `is_fully_unboxed` は `is_box` / `is_closure` / `is_array` で偽を、`is_funptr` で真を返し、それ以外では
  `unpunched_field_types` の全対についての全称であり、`ty(s)` はこの 4 つの述語をどれも満たさないので
  全称の腕に入る。`ty(x)` がその 1 つなので `is_fully_unboxed(ty(s))` も偽である。よって
  `unit_step(ty(s))` は `is_union` の行で `UnitStep::Unit` を返す。`truncate_to_unit` は
  空でない path については第 1 周で `Unit` の腕に入って `break` し、空の path についてはループに入らない
  ので、`trunc(ty(s), ・)` はどの path についても `[]` を返す。とくに `[t] ++ π` と `[t] ++ λ` はどちらも
  `[]` へ切り詰まるので、unit を覆う条件は成り立つ。L1a (b) の欄より `go` は
  `is_fully_unboxed`・`is_closure`・`is_box`・`is_array` がどれも偽の `ty(s)` について
  `unpunched_field_types` の下へ降り、第 `t` 変位について `ty(x)` から始めた `go` の結果の前に `t` を
  置いたものを積む。よって `Λ_{ty(s)}([t] ++ π) = { [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` であり、これは
  空でない。
  BY A12, L1a, L8, CODE src/rc_ir/ownership.rs: unit_step, truncate_to_unit,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, TypeNode::is_union, TypeNode::is_array,
     TypeNode::is_funptr, CODE src/fixstd/builtin.rs: bulitin_tycons

<1>6. 単一 `Arg(j, σ)` の `(args[j], σ)` は unit を覆い、`Λ_{ty(args[j])}(σ) = { σ }` である。
  A3 より `σ ∈ leaves(ty(args[j]))` である。`boxed_leaf_paths` の `go` は leaf を積んだ位置で戻るので、
  1 つの leaf が別の leaf の真の接頭辞になることはない。よって `σ` を前置に持つ leaf は `σ` だけである。
  `trunc` の条件は 1 元集合について自明に成り立つ。
  BY A3, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>7. `origin_from_leaves_under` の `(args[j], w)` は unit を覆う。
  `w = truncate_to_unit(&args[j].ty, leaf, type_env)` であり、A3 より `leaf ∈ leaves(ty(args[j]))` なので、
  P1 より `w ∈ units(ty(args[j]))` である。L7 より `Λ_{ty(args[j])}(w) ≠ ∅` であり、その各 leaf `μ` に
  ついて `trunc(ty(args[j]), μ) = w` である。L6 より `trunc(ty(args[j]), w) = w` なので、この 2 つは
  等しい。
  BY A3, P1, L6, L7, CODE src/rc_ir/ownership.rs: origin_from_leaves_under

<1>8. QED
  `<1>1` が (a) を、`<1>2` から `<1>7` が (b) の表の 5 行を尽くして扱った (DEF 再帰で訪れる対 の表の
  うち「呼ぶ相手」が無い 4 行は新しい対を作らない)。
  BY <1>1, <1>2, <1>3, <1>3a, <1>4, <1>5, <1>6, <1>7, DEF 再帰で訪れる対

### L19 (`Llvm` が束縛する値の leaf の `origin`)

**言明**。`vars.bindings.get(x)` が `Some(Binding::Llvm(gen, args, rty))` であるとし、`decl` を
`gen.result_prov(rty, arg_tys, type_env)`、`λ ∈ leaves(rty)`、`S_λ` を `decl.leaf_origins_at(λ)` が返す
集合とする。A3 より `|S_λ| ≤ 1` であり、次の 3 つが場を尽くす。

- `S_λ = ∅` のとき `origin(x, λ) = Exactly((x, λ))` である。
- `S_λ = {Fresh}` または `S_λ = {Unknown}` のとき `origin(x, λ) = Exactly((x, λ))` である。
- `S_λ = {Arg(j, σ)}` のとき `origin(x, λ) = origin(args[j], σ)` であり、`σ ∈ leaves(ty(args[j]))` である。

<1>1. `|S_λ| ≤ 1` であり、`S_λ` の元は `Arg` か `Fresh` か `Unknown` である。
  A3 は、`result_prov` を override する 29 個の宣言が leaf に置く集合の要素数がすべて 0 か 1 であり、
  複数の元を宣言する op はこのコミットのプログラムに存在しないと述べる。`LeafOrigin` は `Arg`、`Fresh`、
  `Unknown` の 3 つの構成子を持つ。
  BY A3, CODE src/rc_ir/provenance.rs: LeafOrigin

<1>2. `S_λ = {Arg(j, σ)}` のとき `origin(x, λ) = origin(args[j], σ)` であり
      `σ ∈ leaves(ty(args[j]))` である。
  `origin_inner` の `Binding::Llvm` の腕は `decl.leaf_origins_at(path).and_then(as_arg_projection)` で
  分岐する。`λ` は `rty` の leaf なので `leaf_origins_at(λ)` は `Some(S_λ)` であり、`as_arg_projection` は
  要素数 1 の集合の唯一の元が `Arg(j, p)` のとき `Some((j, p))` を返す。よって `Some((j, σ))` の腕に入り
  `origin(vars, type_env, &args[j].name, &σ)` を返す。A3 より単一の `Arg(j, σ)` は第 `j` オペランドの
  leaf `σ` を名指す。
  BY A3, CODE src/rc_ir/ownership.rs: origin_inner, as_arg_projection,
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

### L20 (`origin_from_leaves_under` が返す候補)

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
  `{ S_λ : λ ∈ Λ_{rty}(π) }` である。
  BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
     CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under

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

### L21 (静的な向き -- unit が所有ならその下の leaf も所有)

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
    `owns(x, π)` が真ということであり、示すのは `owns(x, λ)` が真であることである。`pty(x)` で場合を
    分ける。`pty(x) = None` のとき、第 3 節の (b) より `owns(x, λ)` は真である。`pty(x) = Some(τ)` の
    とき、第 3 節の (a) より `τ = ty(x)` であり
    `owns(x, λ) = owns(x, trunc(ty(x), λ))`、`owns(x, π) = owns(x, trunc(ty(x), π))` である。
    DEF unit を覆う対 より `trunc(ty(x), λ) = trunc(ty(x), π)` なので、この 2 つは同じ呼び出しであり、
    仮定より真である。
    BY <2>1, P7e, DEF unit を覆う対

<1>2. CASE `Some(Binding::Move(w))`、`Some(Binding::Join(arm_results))`、
      `Some(Binding::Payload(s, None))` のいずれか
  <2>1. `Move(w)` と `Payload(s, None)` では、任意の `ρ` について `origin(x, ρ) = origin(w, ρ)`
        (resp. `origin(s, ρ)`) である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `Move(w)` と `Payload(s, None)` について結論が出る。
    L18 より進む相手は unit を覆い、`Λ` は変わらない。`<2>1` より `(x, π)` が所有されることは
    `(w, π)` (resp. `(s, π)`) が所有されることであり、帰納法の仮定よりその `Λ` の各 leaf について
    `(w, λ)` は所有される。`<2>1` より `cand(x, λ) = cand(w, λ)` である。
    BY <2>1, L18
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
    `act(a, λ) = cand(a, λ) ∪ {id(a, λ)}` である。`id(a, λ)` は、`origin(a, λ)` が `Exactly` のときは
    `cand(a, λ)` の元であり、`Join` のときは L13 より `owns` が真である。
    BY <2>3, L13, L18, CODE src/rc_ir/ownership.rs: Origin::acted_on, Origin::identity
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
    BY <2>1, L18
  <2>2a. `Binding::Payload(s, Some(t))` (`s` が unbox) について結論が出る。
    L18 (b) より進む相手 `(s, [t] ++ π)` は unit を覆い、その `Λ` は
    `{ [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` である。`<2>1` を `ρ = π` に当てると
    `cand(x, π) = cand(s, [t] ++ π)` なので、`(x, π)` が所有されることは `(s, [t] ++ π)` が所有される
    ことである。帰納法の仮定より、`Λ_{ty(s)}([t] ++ π)` の各 leaf `[t] ++ λ` について
    `(s, [t] ++ λ)` は所有される。`<2>1` を `ρ = λ` に当てると `cand(x, λ) = cand(s, [t] ++ λ)` で
    ある。
    BY <2>1, L18
  <2>2b. QED
    この CASE の 2 つの腕を `<2>2` と `<2>2a` が扱った。
    BY <2>2, <2>2a

<1>4. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `Some((j, σ))`
  この腕は `π` が `rty` の leaf であることを要求する (`leaf_origins_at` は leaf の path にだけ `Some` を
  返す)。`boxed_leaf_paths` の `go` は leaf を積んだ位置で戻るので、1 つの leaf が別の leaf の真の接頭辞に
  なることはない。よって `π` を前置に持つ leaf は `π` だけなので `Λ_{ty(x)}(π) = {π}` である。すなわち
  `λ = π` しかなく、結論は仮定そのものである。
  BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, Provenance::build_shape,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, LeafMap::get, LeafMap::build_shape

<1>5. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `None`
  <2>1. `reached` が空のとき、`Λ_{ty(x)}(π)` の各 `λ` について `S_λ = ∅` であり、
        `origin(x, λ) = Exactly((x, λ))` で `owns(x, λ)` は真である。
    L20 (a) より、`reached` が空であることは `operand_units` が空かつ `produced_here` が偽であることで
    あり、これは `Λ_{ty(x)}(π)` のどの `λ` の `S_λ` も `Arg` を含まず `Fresh` も `Unknown` も含まない
    こと、すなわち `S_λ = ∅` であることと同値である。L19 よりそのとき
    `origin(x, λ) = Exactly((x, λ))` である。`x` の binding は `Binding::Llvm` なので L13 より
    `owns(x, λ)` は真である。
    BY L13, L19, L20
  <2>2. `reached` が空でないとき、`reached` の各元 `o` について、`o` の候補はすべて `owns` で真である。
    L20 (d) より `cand(o) ⊆ cand(x, π)` であり、仮定より `cand(x, π)` の元はすべて真である。
    BY L20
  <2>3. `reached` が空でないとき、`Λ_{ty(x)}(π)` の各 `λ` について `(x, λ)` は所有される。
    L19 より `S_λ` は 3 つの形のいずれかである。`S_λ = ∅` と `S_λ = {Fresh}` と `S_λ = {Unknown}` では
    `origin(x, λ) = Exactly((x, λ))` であり、L13 より `owns(x, λ)` は真である。
    `S_λ = {Arg(j, σ)}` では `origin(x, λ) = origin(args[j], σ)` である。L20 (a) より
    `w := trunc(ty(args[j]), σ)` について `(j, w) ∈ operand_units` であり、L20 (b) より
    `origin(args[j], w) ∈ reached` なので、`<2>2` より `(args[j], w)` は所有される。L18 より
    `(args[j], w)` は unit を覆う。`trunc` の答えは引数の接頭辞なので `w ⊑ σ` であり、
    `σ ∈ leaves(ty(args[j]))` (L19) なので `σ ∈ Λ_{ty(args[j])}(w)` である。帰納法の仮定より
    `(args[j], σ)` は所有される。
    BY <2>2, L13, L18, L19, L20, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>4. QED
    BY <2>1, <2>3

<1>6. QED
  `Binding` は 7 つの構成子を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join`)、
  `vars.bindings.get(x)` はそれらか `None` である。`Field` は容器の boxed / unbox で、`Payload` は
  `tag` の有無と scrutinee の boxed / unbox で、`Llvm` は `as_arg_projection` の答えで分かれる。
  `<1>1` から `<1>5` はこの分け方を尽くす。帰納法の仮定を使ったのは `<1>2` (`Move` / `Payload(s, None)` の
  `(w, π)`・`(s, π)`、`Join` の `(a, π)`)、`<1>3` (`(c, [idx] ++ π)`・`(s, [t] ++ π)`)、`<1>5`
  (`(args[j], w)`) であり、どれも DEF 再帰で訪れる対 の表が `(x, π)` から進む相手なので、L11a より
  この帰納は整礎である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, L11a, DEF 再帰で訪れる対,
     CODE src/rc_ir/ownership.rs: Binding, origin_inner

### L21a (別名の辺は値を運ぶ)

**言明**。1 つの活性化 (D21) とその辿る実行路の 1 つの位置を固定し、`x` がその位置までに値を得ている
ものとする。`vars.bindings.get(x)` に応じて次が成り立つ。

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
  BY CODE src/rc_ir/ownership.rs: collect_bindings, returned_var

<1>2. どの場合も、`x` の binding が名指す変数 (`w` / `s` / `c` / 各アーム結果) は、`x` がその位置までに
      値を得ているならば、その位置までに値を得ている。
  A11 より変数の使用はその位置でスコープに入っている束縛に解決する。`<1>1` の 4 つの構文はいずれも、
  名指す変数を `x` を束縛する節点そのもの (またはそのアーム本体) で読むので、その変数はその節点より前で
  束縛されている。
  BY <1>1, A11

<1>3. (m1) が成り立つ。
  D2 より `Let(x, rhs, k)` は `rhs` の値を `x` に束縛し、D7 より `Var` は値を渡すだけである。コード生成も
  そうなっている -- `eval_rc_rhs` の `RcRhs::Var(v)` の腕は `get_scoped_obj(&v.name)` を返し、
  `bind_and_continue` がそれを `x` の名前で `scope_push` する。
  BY <1>1, <1>2, D2, D7, CODE src/rc_ir/codegen.rs: Generator::eval_rc_rhs,
     Generator::bind_and_continue, Generator::eval_rc_expr_inner

<1>4. (m2) が成り立つ。
  `eval_rc_match` は、`arm.tag` が `None` のアームの payload を `get_scoped_obj_noretain(&scrut.name)` に
  束縛する。すなわち scrutinee の値そのものである。
  BY <1>1, <1>2, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match

<1>5. (m3) が成り立つ。
  D21 より活性化はその `Match` でちょうど 1 つのアームを選ぶ。`x` がその位置までに値を得ているので、
  その `Match` は実行された。D2 より `Let(x, Match(..), k)` は `Match` の値を `x` に束縛し、D2 より
  アーム本体の `Ret(v)` の値は `v` である。コード生成もそうなっている -- `eval_rc_expr_inner` の
  `RcExpr::Ret(x)` の腕は `get_scoped_obj(&x.name)` を返し、`eval_rc_match` は各アームについて
  `eval_rc_expr(&arm.body, ..)` の値を `incomings` に積んで phi で合流させ、`bind_and_continue` が
  その値を `x` の名前で `scope_push` する。選ばれたアームを通る実行では phi の値はそのアームの
  `incomings` の値である。`<1>1` よりそのアームの結果の変数は `arm_results` の元である。
  BY <1>1, <1>2, D2, D21, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match,
     Generator::eval_rc_expr_inner, Generator::bind_and_continue

<1>6. (m4) が成り立つ。
  D2 より `Destructure(c, fs, s, k)` は容器 `c` をフィールドに分解し、各 `(i, x)` の `x` に第 `i`
  フィールドを束縛する。コード生成もそうなっている -- `eval_rc_expr_inner` の `Destructure` の腕は
  `get_struct_fields` に `fields` の添字の列を渡し、返る 1 対 1 の列を対応するフィールド変数の名前で
  `scope_push` する。
  BY <1>1, <1>2, D2, CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
     CODE src/object.rs: ObjectFieldType::get_struct_fields

<1>7. (m5) が成り立つ。
  `x` は `tag = Some(t)` のアームの payload 変数であり (`<1>1`)、`x` がその位置までに値を得ているので、
  活性化はそのアームを選んでいる。D21 より活性化が選ぶアームは `s` の値の実行時のタグに `tag` が等しい
  アームであり、A16 よりそのようなアームが在るので、選ばれたアームのタグ `t` は `s` のタグに等しい。
  `eval_rc_match` は、`arm.tag` が `Some(_)` のアームの payload を
  `get_union_value_noretain_norelease(scrut_obj, &arm.payload.ty)` に束縛する。これは union の payload
  バッファを変位の型として読んだものであり、`ty(s)` が unbox なので retain も伴わない。
  BY <1>1, <1>2, A16, D21, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match,
     CODE src/object.rs: ObjectFieldType::get_union_value_noretain_norelease,
     ObjectFieldType::get_value_from_union_buf

<1>8. QED
  BY <1>3, <1>4, <1>5, <1>6, <1>7

### L22 (実行時の向き -- unit が非所有ならその下の inhabited な leaf も非所有)

**言明**。1 つの活性化 (D21) とその実行路の 1 つの位置を固定する。`x` がその位置までに値を得ており、
`(x, π)` が unit を覆う (L18 の DEF) とする。`Inh_x(π)` を、その位置の `x` の値について inhabited (D16)
である `Λ_{ty(x)}(π)` の leaf の集合とする。`cand(x, π)` のすべての元 `(r, p)` について `owns(r, p)` が
**偽**ならば、`Inh_x(π)` の各 `λ` について `cand(x, λ)` に `owns` が偽である元がある。

以下、「`(x, π)` は全部偽である」を「`cand(x, π)` のすべての元について `owns` が偽である」の略とする。
証明は L11a の `Reach` についての帰納である。すなわち、DEF 再帰で訪れる対 の表が `(x, π)` から進む
各相手について結論が成り立つことを帰納法の仮定とする。

<1>1. CASE `vars.bindings.get(x)` が `None` / `Some(Binding::Producer)` /
      `Some(Binding::Field(c, idx))` で `c` が boxed / `Some(Binding::Payload(s, Some(t)))` で `s` が
      boxed のいずれか
  この場合は仮定と両立しない。これらの腕は `here()` を返すので `cand(x, π) = {(x, π)}` である。
  `bindings.get(x)` が `None` のとき、`x` は `param_tys` の鍵でない -- `VarTable::of` は各パラメータ・
  capture について `bindings` に `Binding::Param` を、`param_tys` にその型を入れ、`collect_bindings` は
  `param_tys` に何も入れないので、`param_tys` の鍵はどれも `bindings` に `Binding::Param` を持つから
  である。`owns_object` は `param_tys.get(root)` が `None` の腕で真を返すので `owns(x, π)` は真である。
  残る 3 つの binding (`Producer`、boxed 容器の `Field`、boxed scrutinee の `Payload`) は
  `collect_bindings` が入れるものなので L13 より `owns(x, π)` は真である。どちらも「全部偽」に反する。
  BY L13, CODE src/rc_ir/ownership.rs: origin_inner, VarTable::of, collect_bindings,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>2. CASE `Some(Binding::Param)`
  この腕は `path` を読まずに `here()` を返すので `cand(x, π) = {(x, π)}` かつ
  `cand(x, λ) = {(x, λ)}` である。`x` はこの版のパラメータ・capture なので第 1 節より
  `pty(x) = Some(ty(x))` であり、第 3 節の (a) が使えて
  `owns(x, λ) = owns(x, trunc(ty(x), λ))`、`owns(x, π) = owns(x, trunc(ty(x), π))` である。
  DEF unit を覆う対 より `trunc(ty(x), λ) = trunc(ty(x), π)` なので `owns(x, λ) = owns(x, π)` であり、
  仮定よりこれは偽である。`Inh_x(π) ⊆ Λ_{ty(x)}(π)` なので、`Inh_x(π)` の各 `λ` について
  `cand(x, λ) = {(x, λ)}` は `owns` が偽である元を持つ。
  BY P7e, DEF unit を覆う対, CODE src/rc_ir/ownership.rs: origin_inner, VarTable::of

<1>3. CASE `Some(Binding::Move(w))` または `Some(Binding::Payload(s, None))`
  <2>1. 任意の `ρ` について `origin(x, ρ) = origin(w, ρ)` (resp. `origin(s, ρ)`) である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `x` の値は `w` の値 (resp. `s` の値) と同じ値であり、`w` (resp. `s`) はこの位置までに値を得て
        いる。
    L21a の (m1) (resp. (m2)) である。
    BY L21a
  <2>3. QED
    `<2>2` より `Inh_x(π) = Inh_w(π)` である (D16 は値だけを見る)。L18 より `(w, π)` は unit を覆い
    `Λ` は変わらない。`<2>1` より `(w, π)` は全部偽なので、帰納法の仮定より `Inh_w(π)` の各 `λ` に
    ついて `cand(w, λ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>1, <2>2, D16, L18

<1>4. CASE `Some(Binding::Join(arm_results))`
  <2>1. `cand(x, ρ) = ⋃_{a ∈ arm_results} act(a, ρ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates, Origin::candidates,
       Origin::acted_on
  <2>2. この活性化はこの `Match` でちょうど 1 つのアームを選んでおり、そのアーム本体の終端の `Ret` が
        名指す変数を `a*` とすると、`a* ∈ arm_results` であり、`a*` はこの位置までに値を得ていて、
        `x` の値は `a*` の値と同じ値である。
    L21a の (m3) である。
    BY L21a
  <2>3. `(a*, π)` は全部偽である。
    `<2>1` より `cand(a*, π) ⊆ act(a*, π) ⊆ cand(x, π)` である。
    BY <2>1
  <2>4. QED
    `<2>2` より `Inh_x(π) = Inh_{a*}(π)` である。L18 より `(a*, π)` は unit を覆い `Λ` は変わらない。
    `<2>3` と帰納法の仮定より、`Inh_{a*}(π)` の各 `λ` について `cand(a*, λ)` に `owns` が偽の元があり、
    `<2>1` より `cand(a*, λ) ⊆ cand(x, λ)` である。
    BY <2>1, <2>2, <2>3, D16, L18

<1>5. CASE `Some(Binding::Field(c, idx))` で `c` が unbox
  <2>1. 任意の `ρ` について `origin(x, ρ) = origin(c, [idx] ++ ρ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `c` はこの位置までに値を得ており、`x` の値は `c` の値の第 `idx` フィールドである。
    L21a の (m4) である。
    BY L21a
  <2>3. `λ ∈ Inh_x(π)` と `[idx] ++ λ ∈ Inh_c([idx] ++ π)` は同値である。
    A12 より `ty(c)` は構造体なので `is_union(ty(c))` は偽であり、`[idx] ++ λ` が `ty(c)` の根で通る節は
    unbox union ではない。よって `[idx] ++ λ` が通る unbox union の節と、それぞれで選ぶ変位番号は、`λ` が
    `ty(x)` で通るものと同じである。`<2>2` より `x` の値は `c` の値のその位置の部分値なので、各節の
    タグも同じである。D16 はこの一致だけを見る。
    BY <2>2, A12, D16, CODE src/ast/types.rs: TypeNode::is_union
  <2>4. QED
    L18 より `(c, [idx] ++ π)` は unit を覆い、その `Λ` は `{ [idx] ++ λ : λ ∈ Λ_{ty(x)}(π) }` である。
    `<2>1` より `(c, [idx] ++ π)` は全部偽なので、帰納法の仮定と `<2>3` より、`Inh_x(π)` の各 `λ` に
    ついて `cand(c, [idx] ++ λ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>1, <2>3, L18

<1>6. CASE `Some(Binding::Payload(s, Some(t)))` で `s` が unbox
  <2>1. 任意の `ρ` について `origin(x, ρ) = origin(s, [t] ++ ρ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `s` はこの位置までに値を得ており、その位置の `s` の値の実行時のタグは `t` であって、`x` の値は
        `s` の値の第 `t` 変位の payload である。
    L21a の (m5) である。この腕の条件より `ty(s)` は boxed でなく、A12 より union なので、(m5) の前提が
    成り立つ。
    BY A12, L21a
  <2>3. `λ ∈ Inh_x(π)` と `[t] ++ λ ∈ Inh_s([t] ++ π)` は同値である。
    A12 より `ty(s)` は union であり、この腕の条件より unbox である。よって `[t] ++ λ` が `ty(s)` の根で
    通る節は unbox union であり、そこで選ぶ変位番号は `t` である。`<2>2` よりその位置の `s` のタグは
    `t` なので、この節の条件は成り立つ。残りの節は `λ` が `ty(x)` で通るものと同じであり、`<2>2` より
    `x` の値は `s` の値の第 `t` 変位の payload なのでタグも同じである。D16 はこれらの節だけを見る。
    BY <2>2, A12, D16
  <2>4. QED
    L18 より `(s, [t] ++ π)` は unit を覆い、その `Λ` は `{ [t] ++ λ : λ ∈ Λ_{ty(x)}(π) }` である。
    `<2>1` より `(s, [t] ++ π)` は全部偽なので、帰納法の仮定と `<2>3` より、`Inh_x(π)` の各 `λ` に
    ついて `cand(s, [t] ++ λ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>1, <2>3, L18

<1>7. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `Some((j, σ))`
  この腕は `π` が `rty` の leaf であることを要求する (`leaf_origins_at` は leaf の path にだけ `Some` を
  返す)。`boxed_leaf_paths` の `go` は leaf を積んだ位置で戻るので、`π` を前置に持つ leaf は `π` だけで
  あり、`Λ_{ty(x)}(π) = {π}` である。`Inh_x(π) ⊆ {π}` であり、`λ = π` のとき
  `cand(x, λ) = cand(x, π)` は仮定より全部偽なので、偽の元がある (`cand` は空でない -- `Exactly` は
  1 元を持ち、`of_candidates` は空集合に `assert!` で中断する)。
  BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, Provenance::build_shape,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, LeafMap::get, LeafMap::build_shape,
     CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates

<1>8. CASE `Some(Binding::Llvm(gen, args, rty))` で `decl.leaf_origins_at(π).and_then(as_arg_projection)`
      が `None`
  <2>1. `reached` は空でない。
    L20 (c) より、空ならば `origin(x, π) = Exactly((x, π))` であり、`x` の binding は `Binding::Llvm` な
    ので L13 より `owns(x, π)` は真である。これは「全部偽」に反する。
    BY L13, L20
  <2>2. `produced_here` は偽である。
    L20 (e) より、真ならば `(x, π) ∈ cand(x, π)` であり、L13 より `owns(x, π)` は真である。これは
    「全部偽」に反する。
    BY L13, L20
  <2>3. `operand_units` の各 `(j, w)` について `(args[j], w)` は全部偽である。
    L20 (b) より `origin(args[j], w) ∈ reached` であり、`<2>1` より `reached` は空でないので L20 (d) より
    `cand(args[j], w) ⊆ cand(x, π)` である。
    BY <2>1, L20
  <2>4. `Inh_x(π)` の各 `λ` について `S_λ = {Arg(j, σ)}` の形である。
    L19 より `S_λ` は `∅` か `{Fresh}` か `{Unknown}` か `{Arg(j, σ)}` である。`S_λ = ∅` は起きない --
    A3 の表の第 1 行が、空集合と宣言された leaf は inhabited にならないと述べるからである。
    `S_λ = {Fresh}` と `S_λ = {Unknown}` も起きない -- L20 (a) よりそのとき `produced_here` が真になり、
    `<2>2` に反するからである。
    BY A3, L19, L20, <2>2
  <2>5. QED
    `λ ∈ Inh_x(π)` を取り、`S_λ = {Arg(j, σ)}` とする (`<2>4`)。L19 より
    `origin(x, λ) = origin(args[j], σ)` かつ `σ ∈ leaves(ty(args[j]))` である。L20 (a) より
    `w := trunc(ty(args[j]), σ)` について `(j, w) ∈ operand_units` である。`trunc` の答えは引数の接頭辞
    なので `w ⊑ σ`、よって `σ ∈ Λ_{ty(args[j])}(w)` である。A3 の表の「単一の `Arg(j, σ)`」の行は、
    結果のその leaf が inhabited であることと第 `j` オペランドの leaf `σ` が inhabited であることが同値だ
    と述べるので、`σ ∈ Inh_{args[j]}(w)` である (`args[j]` は A11 よりこの位置までに値を得ている)。
    L18 より `(args[j], w)` は unit を覆い、`<2>3` より全部偽なので、帰納法の仮定より
    `cand(args[j], σ) = cand(x, λ)` に `owns` が偽の元がある。
    BY <2>3, <2>4, A3, A11, L18, L19, L20, CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>9. QED
  `Binding` は 7 つの構成子を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join`)、
  `vars.bindings.get(x)` はそれらか `None` である。`Field` は容器の boxed / unbox で、`Payload` は
  `tag` の有無と scrutinee の boxed / unbox で、`Llvm` は `as_arg_projection` の答えで分かれる。
  `<1>1` から `<1>8` はこの分け方を尽くす。帰納法の仮定を使ったのは `<1>3` (`(w, π)`・`(s, π)`)、
  `<1>4` (`(a*, π)`)、`<1>5` (`(c, [idx] ++ π)`)、`<1>6` (`(s, [t] ++ π)`)、`<1>8` (`(args[j], w)`)
  であり、どれも DEF 再帰で訪れる対 の表が `(x, π)` から進む相手なので、L11a よりこの帰納は整礎である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, L11a, DEF 再帰で訪れる対,
     CODE src/rc_ir/ownership.rs: Binding, origin_inner

### P7a の 2 つの向き

**証明するもの**。site `(v, u)`、`infer_ownership` の不動点、1 つの活性化とその実行路の 1 つの位置を
固定する。このとき **節 1 から節 3** と **節 2 から節 1** が成り立つ。

<1>1. `(v, u)` は unit を覆う。
  `levelled_sites` は `Retain(v, path)` / `Release(v, path)` について `(v, path)` を積み、A2 より
  その `path` は `units(ty(v))` の元である。`Let(_, App(_, args), _)` については各 `arg` と各
  `unit ∈ rc_units(arg.ty, type_env)` の対を積むので、これも `units(ty(arg))` の元である。L18 (a) より
  `(v, u)` は unit を覆う。
  BY A2, L18, CODE src/rc_ir/borrow.rs: levelled_sites

<1>2. 節 1 から節 3 へ渡る。
  節 1 は「`cand(v, u)` のすべての元について `owns` が真」である (`owns_unit` の定義)。`<1>1` と L21 より
  `Λ(u)` の**すべての** leaf `λ` について `cand(v, λ)` のすべての元について `owns` が真である。
  `Inh(v, u) ⊆ Λ(u)` なので、とくに inhabited な leaf についてそうである。これが節 3 である。
  BY <1>1, L18, L21, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>3. 節 1 が偽ならば、`cand(v, u)` のすべての元について `owns` は偽である。
  `cand(v, u)` は空でない -- `Origin::Exactly` の `candidates()` は 1 元の列であり、`of_candidates` は
  空集合に `assert!` で中断する。`(v, u)` は `levelled_sites` が挙げる site であり、いまは
  `infer_ownership` の不動点なので、P7d よりその候補はすべて真かすべて偽である。節 1 が偽とは
  「すべて真」が成り立たないことなので、「すべて偽」である。
  BY P7d, CODE src/rc_ir/ownership.rs: Origin::candidates, Origin::of_candidates,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>4. 節 1 が偽ならば節 2 も偽である。
  `<1>3` より `(v, u)` は L22 の意味で全部偽である。`v` はこの位置までに値を得ている (節 2 と節 3 を読む
  前提)。`<1>1` より `(v, u)` は unit を覆う。L22 より、`Inh(v, u)` の各 `λ` について `cand(v, λ)` に
  `owns` が偽の元がある。節 2 は「`Inh(v, u)` のある `λ` の**すべての**候補について `owns` が真」なので、
  これは節 2 の否定である。
  BY <1>1, <1>3, L18, L22

<1>5. 節 2 から節 1 へ渡る。
  `<1>4` の対偶である。
  BY <1>4

<1>6. QED
  README の P7a は「成り立つのは **1 ⟹ 3** と **2 ⟹ 1** の 2 つであり、この 2 つで足りる」と述べる。
  この節が閉じるのはその 2 つの含意である -- `<1>2` が 1 ⟹ 3 を、`<1>5` が 2 ⟹ 1 を与える。残る
  3 ⟹ 2 と 3 ⟹ 1 は偽であり (R2)、README も偽であると述べる。
  BY <1>2, <1>5

**証明しないもの**。節 3 から節 2 と、節 3 から節 1 である。`Inh(v, u) = ∅` のとき節 3 は空虚に真になり、
節 2 は偽になる。R2 がその本体を挙げる。`Inh(v, u) ≠ ∅` を仮定に足せば、`<1>2` と `<1>5` に
「節 3 から節 2」(`Inh(v, u)` の元を 1 つ取る) が加わって 3 つは同値になる。

**この 2 つの向きが P10 と P14 に与えるもの。** A5 より、値が保持する参照は inhabited な leaf に
ちょうど 1 つずつあり、inhabited でない leaf は参照を持たない。A4 よりコード生成の `Retain(v, u)` /
`Release(v, u)` は `u` の下の inhabited な leaf の参照カウントだけを ±1 する。よって節 3 は「残した
節点が触れる参照はすべてこの版のものである」を、節 2 の否定は「落とした節点が触れたはずの参照は
どれもこの版のものでない」を与える。

### R1 (節 2 と節 3 の inhabited の限定が要ること)

**言明**。P7a の仮定 (site、`infer_ownership` の不動点) をすべて満たし、節 1 が偽でありながら、`Λ(u)` の
ある **inhabited でない** leaf のすべての候補が所有される入力プログラムがある。すなわち、節 2 から
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
  BY L3, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step

<1>2. 次の関数だけを `funcs` に持ち、`globals` が空である入力プログラムを取る。`z : Array I64` は
      `f` の唯一のパラメータであり、`w : I64` は定数を束縛する。

      ```
      f(z) = Let(x, Llvm(union_make_1, [z]),          // x : Outer
               Release(x, [],
                 Let(w, Llvm(int_lit_0, []),
                   Ret(w))))
      ```

      この本体は A1 と A2 を満たす。
  A2: `Release` の path `[]` は `<1>1` より `units(Outer)` の元である。
  A1: 入力ではすべてのパラメータ unit が所有されるので、活性化の初期の義務集合は `z` の leaf `[]` の
  参照 1 つである。`union_make_1` の宣言は結果の leaf `[1]` を単一の `Arg(0, [])` とするので、D9 の移動の
  表の `Llvm` の素通し leaf の行により、その参照は `x` の leaf `[1]` へ移る。`x` の変位は 1 なので
  `x` の leaf `[0, 0]` は inhabited でない (D16)。`Release(x, [])` は `[]` の下の inhabited な leaf、
  すなわち `[1]` の参照を 1 つ取り除くので、義務集合は空になる。`Ret(w)` の `w` は `I64` で boxed leaf を
  持たないので消費は無い。
  (S-c) について。この本体に現れる D7 の読む構文は 2 つの `Llvm` の `Let` である。`int_lit_0` は
  オペランドを持たないので読むオブジェクトが無い。`union_make_1` はオペランド `z` を読み、その inhabited な
  leaf は `[]` 1 つで、その時点の義務集合がその参照を持つので D8 より `H(obj(z, []))` は 1 以上であり、
  解放されていない。`Release(x, [])` が触れるのは `obj(x, [1])` であり、素通しの行より `obj(z, [])` と
  同じオブジェクトで、その時点の義務集合がその参照を持つので同じく解放されていない。
  BY A1, A2, D7, D8, D9, D10, D11, D16, <1>1,
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
  BY L2, L3, CODE src/rc_ir/borrow.rs: owns_object_yet,
     CODE src/rc_ir/ownership.rs: unit_step, subtree_type, units_under, truncate_to_unit,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

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
  <2>4. `level_ownership` は site `(x, [])` で `false` を返し続ける。
    `<1>2a` より候補は `(z, [])` 1 つである。`OL` が空ならば `<1>2b` より `owns_object_yet` は偽であり、
    `owns_a_candidate` は候補についての `any` なので偽になって `level_ownership` は `false` を返す。
    BY <1>2a, <1>2b, CODE src/rc_ir/borrow.rs: level_ownership
  <2>5. QED
    `infer_ownership` は空の `owned_leaves` から始め、消費 (`<2>1`) からも `level_ownership` (`<2>2`,
    `<2>4`) からも要素を加えない。よって最初の周で `changed` は偽になり、不動点は空集合である。
    BY <2>1, <2>2, <2>4, CODE src/rc_ir/borrow.rs: infer_ownership

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
  BY <1>2, <1>2b, <1>3, L16, CODE src/rc_ir/borrow.rs: func_has_borrowable_param,
     funcs_observing_uniqueness, borrow_ify,
     CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness,
     CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody, InlineLLVMIntLit

<1>5. 節 1 は偽である。
  `<1>2a` より `origin(vars_f, x, []) = Exactly((z, []))` であり、L15 より借用版の
  `origin(vars_c, ρ(x), [])` はその `ρ` による像 `Exactly((ρ(z), []))` である。すなわち借用版の site
  `(ρ(x), [])` の候補は `{(ρ(z), [])}` である。`owns_unit` はその全称なので、`<1>4` より偽である。
  BY <1>2a, <1>4, L15, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>6. 節 2 は真である。
  `Λ([]) = leaves(Outer) = {[0, 0], [1]}` であり、`λ = [0, 0]` を取る。`Outer` の leaf `[0, 0]` の宣言は
  空集合なので、`leaf_origins_at([0, 0])` は空集合を持つ `Some` を返し、`as_arg_projection` は要素数が 1 で
  ない集合に `None` を返す。よって `origin_from_leaves_under` に入り、その `leaf_origins_under([0, 0])` は
  その空集合 1 つだけを与えるので `operand_units` は空、`produced_here` は偽、`reached` は空になり、
  `reached.first()?` が `None` を返して `unwrap_or_else(here)` が `Exactly((x, [0, 0]))` を返す。
  `x` は本体の `Let` が `Llvm` の右辺で束縛する変数なので `vars.bindings.get(x)` は
  `Some(Binding::Llvm(..))` であり、L13 より `owns_object(ρ(x), [0, 0])` は真である。よって
  `λ = [0, 0]` のすべての候補について `owns_object` は真である。
  BY <1>1, L13, L15, CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov,
     CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner, origin_from_leaves_under,
     as_arg_projection

<1>7. leaf `[0, 0]` はどの活性化でも inhabited にならない。
  `union_make_1` は `x` の値の変位を 1 にする。leaf `[0, 0]` が `Outer` の根の unbox union の節で選ぶ
  変位番号は 0 なので、D16 の条件はどの時点でも成り立たない。A3 の表の第 1 行も、空集合と宣言された
  leaf が inhabited にならないと述べる。
  BY A3, D16, CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov

<1>8. QED
  `<1>5` と `<1>6` より、この site で節 1 は偽でありながら leaf `[0, 0]` のすべての候補は所有される。
  `<1>7` よりその leaf は inhabited でない。
  BY <1>5, <1>6, <1>7

### R2 (inhabited な leaf が無い site では、節 3 から節 2 と節 1 へ渡れない)

**言明**。P7a の仮定をすべて満たし、ある活性化のある位置で `Inh(v, u) = ∅` かつ節 1 が偽である入力
プログラムがある。そこでは節 3 は空虚に真であり、節 2 と節 1 は偽である。

<1>1. 型 `Mix = unbox union { n : I64, a : Array I64 }` について
      `leaves(Mix) = {[1]}` かつ `units(Mix) = {[]}` である。
  `boxed_leaf_paths` の `go` は `Mix` について `is_fully_unboxed` (変位 `a` が `Array I64` なので偽)、
  `is_closure`、`is_box` (unbox なので偽)、`is_array` のどれでもないので `unpunched_field_types` の下へ
  降り、`I64` からは何も、`Array I64` からは `[1]` を積む。`unit_step` は `is_union` で `UnitStep::Unit`
  を返すので L3 より `units(Mix) = [[]]` である。
  BY L3, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step

<1>2. 次の関数だけを `funcs` に持ち、`globals` が空である入力プログラムを取る。`x : Mix` は `f` の唯一の
      パラメータ、`w : I64` は定数を束縛する。

      ```
      f(x) = Release(x, [],
               Let(w, Llvm(int_lit_0, []),
                 Ret(w)))
      ```

      この本体は A1 と A2 を満たす。
  A2: `Release` の path `[]` は `<1>1` より `units(Mix)` の元である。
  A1: 入力ではすべてのパラメータ unit が所有されるので、活性化の初期の義務集合は `x` の inhabited な
  leaf の参照である (D10)。タグが `n` (変位 0) の活性化では `Λ_{Mix}([]) = {[1]}` の唯一の leaf が変位 1 を
  選ぶので inhabited でなく (D16)、義務集合は空である。`Release(x, [])` は inhabited な leaf の参照を
  取り除くので何も取り除かず、(S-a) と (S-b) が成り立つ。タグが `a` (変位 1) の活性化では義務集合は
  leaf `[1]` の参照 1 つであり、`Release(x, [])` がそれを取り除くので、(S-a) と (S-b) が成り立つ。
  `Ret(w)` の `w` は `I64` で boxed leaf を持たないので消費は無い。
  (S-c) について。この本体に現れる D7 の読む構文は `Let(w, Llvm(int_lit_0, []), ..)` だけであり
  (`Release` と `Ret` は読む構文ではない)、D7 の `Llvm` の行が読むのは各オペランドだが `int_lit_0` は
  オペランドを持たないので、読まれるオブジェクトは無い。`Release(x, [])` が触れるのは、タグが `n` の
  活性化では inhabited な leaf が無いので何も無く、タグが `a` の活性化では `obj(x, [1])` である。後者では
  その時点の義務集合がその参照を持っており、D8 より `H(obj(x, [1])) ≥ 1` なので、そのオブジェクトは
  解放されていない。
  BY A1, A2, D7, D8, D10, D11, D16, <1>1, CODE src/fixstd/builtin.rs: InlineLLVMIntLit

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
  BY L2, L3, L16, <1>1, <1>2, CODE src/rc_ir/borrow.rs: infer_ownership, level_ownership,
     owns_object_yet, func_has_borrowable_param, funcs_observing_uniqueness, borrow_ify,
     CODE src/rc_ir/ownership.rs: collect_consumes_go, origin_inner,
     CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness,
     CODE src/fixstd/builtin.rs: InlineLLVMIntLit

<1>4. タグが `n` の活性化の `Release(x, [])` の位置で、`Inh(x, []) = ∅` であり、節 3 は真、節 2 と節 1 は
      偽である。
  `<1>1` より `Λ([]) = {[1]}` であり、その leaf が `Mix` の根の unbox union の節で選ぶ変位番号は 1、
  その位置のタグは 0 なので D16 の条件は成り立たない。よって `Inh(x, []) = ∅` である。節 3 の全称は空なので
  真、節 2 の存在は空なので偽である。節 1 は `owns_unit(ρ(x), [])` であり、`<1>3` より
  `cand(ρ(x), []) = {(ρ(x), [])}` の唯一の候補について `owns_object` が偽なので偽である。
  BY <1>1, <1>3, D16, L15, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>5. QED
  BY <1>2, <1>3, <1>4

**この乖離は無害である。** 節 1 が偽なので借用版の `rewrite_rc` は `Release(x, [])` を落とす (P10)。
A5 より inhabited でない leaf は参照を持たず、A4 よりコード生成の `Release(x, [])` は inhabited な leaf の
参照カウントだけを ±1 するので、この活性化で落とした節点は何にも触れない。壊れているのは言明の側だけで
ある -- 節 3 が「触れる参照はすべて所有されている」を空虚に述べるので、そこから節 2 や節 1 へは渡れない。

### `level_ownership` と `union_as` の隔たり

**`union_as` の形の隔たりは、いまの言明の証明義務に現れない。**前の版の P7a は候補集合についての双条件で
あり、`Λ(u)` の leaf の側から unit の側へ**直接**渡ることを要求していた。その向きは `union_as` の宣言で
破れる -- `union_as_k` は unbox union のオペランドについて変位 `k` の leaf だけを名指すので
(`CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov`)、unit の側が `truncate_to_unit` で
辿り着くオペランドの unit には、どの結果 leaf も名指さない leaf が残る。

いまの言明はその向きを通らない。節 2 から節 1 へは対偶で渡り (`P7a の 2 つの向き` の `<1>4`)、そこで
読むのは P7d である。`union_as` の場合、節 1 が偽ならば P7d より site の候補はすべて偽であり、L22 の
`<1>8` はオペランドの unit へ降りて帰納法の仮定を使う。名指されない leaf の所有はどこでも問われない。
**前稿が (★) と呼んだ「宣言についての仮定」は要らない。**

### 記録

この文書が対象のコードに見つけたものは無い。見つかったのは言明と仮定の側の 3 点である。

- **R1**: 節 2 と節 3 が inhabited な leaf に限らなければならないこと。この限定は README に入っている。
- **R2**: `Inh(v, u) = ∅` のとき節 3 から節 2 と節 1 へ渡れないこと。3 つを同値と述べるには
  `Inh(v, u) ≠ ∅` が要る。README を書き換えるのはオーケストレータである。
- **(N)**: A13 と P9 の後半が**束縛名**だけを覆うこと。L15 と L16 が要るのは、複製が作る名前が入力に
  現れるどの名前とも異なることであり、束縛名でない名前 -- 直接呼び出しが名指す関数の名前と、グローバル値
  を読む `RcVar` の名前 -- はそこに入らない。第 1 節が (N) としてその形を置く。A13 の脇の「検査: 無し」も
  実際とずれている -- `check_clone_names_are_fresh` が `develop_mode` のとき (N) の集合そのものを検査する
  (`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`)。README を書き換えるのはオーケストレータで
  ある。
