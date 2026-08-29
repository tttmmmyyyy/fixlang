# P7e (`owns_object` は unit ごとに答える) と P7d (所有は site ごとに一様である)

この文書は `README.md` の P7e と P7d を証明する。立つのは `README.md` の定義 D1-D26、仮定 A1-A18、および
命題 P1-P7 の**言明**である。

**P7a は証明しない。**第 6 節が、P7a の 3 つの節のうち 1 と 2 の同値が偽であることを、本体を 1 つ挙げて
示す。同節はあわせて、P7a が要求する「`owns_unit` を呼ぶ位置は `levelled_sites` が挙げる site を出ない」を
証明し、`level_ownership` が `union_as` の形の隔たりを埋めることを述べる。

読んだコードは作業ツリーの版である。README の対象コミット
`11abca7e83ed8d52cdd0161f22f74b43447134d4` との差分は `README.md` だけであり、この文書が引用する記号の
本文は対象コミットと一致する。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P7e (`owns_object` は unit ごとに答える) | 証明した (第 3 節) |
| P7d (所有は site ごとに一様である) | 証明した (第 5 節) |
| P7a (site の所有は、その leaf の所有と一致する) | **節 2 が偽**。第 6 節に反例 R1 と修復案を書く |

P7e の要は L1 である。`subtree_type` と `truncate_to_unit` は同じ型の列を同じ順に歩き、同じ `unit_step` の
答えで場合を分けるので、片方が `None` を返す位置がもう片方の `break` する位置になる。

P7d の要は L11 と L14 である。`level_ownership` は候補 `(r, p)` について `covered_leaves(ty(r), p)` を
所有へ倒し、`owns_object_yet` は `units_under(ty(r), p)` の各 unit に「同じ鍵を持つ所有された leaf」を
要求する。この 2 つが噛み合うのは `covered_leaves(ty(r), p)` が空でないときであり (L11)、空でないことは
候補の path が `origin` の再帰で作られる形に限られることから出る (L14)。

**`union_as` の隔たりについて。**前の版の P7a は候補集合についての双条件であり、`Λ(u)` の leaf の側から
unit の側へ**直接**渡ることを要求していた。その向きは `union_as` の宣言で破れるので、前稿はそこを埋める
仮定 ((★)、`result_prov` の宣言についての数え上げ) を要求として挙げた。いまの言明はその向きを通らず、
節 2 から節 1 へ P7d 経由で渡る。**(★) は要らない。**

**残るのは `⊥` の leaf である。**`result_prov` が空集合を宣言した leaf は、`origin` が値自身を名指すので
`owns_object` が真を答える。節 2 は「ある leaf のすべての候補が所有される」なので、この leaf 1 つで真に
なる。その leaf は実行時に inhabited (D16) にならない (A3) ので、節 1 が偽でも漏れは起きない。R1 は
この形の本体である。第 6 節は節 1 と節 3 の同値については何も言わない。

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
  `boxed_leaf_paths` の内部関数 `go` は、`is_fully_unboxed` でも `is_closure` でも `is_box` でも
  `is_array` でもない型についてだけ `unpunched_field_types` の下へ降り、それ以外では `path` を延ばさずに
  終わる。`unit_step` が `Fields { held_fields, .. }` を返すのはこの 4 つがどれも成り立たないときで
  あり、そのとき `held_fields` は `unpunched_field_types` そのものである。`sub(τ, p)` が `Some` を返す
  ので、`τ` の `p` に沿う歩みの各段で `unit_step` は `Fields` であり、`go` はその段で同じフィールドの
  下へ降りる。よって `go` が `p` を前置に持つ path を積むのは、`σ` から始めた `go` が積む path の前に
  `p` を置いたものに限る。
  BY L1, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step

<1>6. `units(τ) ⊇ { p ++ u : u ∈ units(σ) }` である。
  `rc_units_go` は `unit_step` が `Fields { held_fields, .. }` のとき `held_fields` の各フィールドの下へ
  `path` を延ばして降りる。`<1>5` と同じく、`τ` の `p` に沿う歩みの各段で `unit_step` は `Fields` なので、
  `rc_units_go` は `path = p` で `σ` に達し、そこから `units(σ)` の各要素を `p` の後ろに積む。
  BY L1, CODE src/rc_ir/ownership.rs: rc_units_go, unit_step

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
  `<1>5` である。この 2 つは排他である -- 前者では `unit_step(cur_{|u|})` が `Unit`、後者では
  `unit_step(cur_{|u|-1})` が `Capture` であって、`unit_step` は 1 つの型に 1 つの答えを返す。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

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
  `u = q ++ [capture_idx]` であり、`u ⊑ λ` なので `λ[|q|] = capture_idx` である。L6 の (ii) より
  `unit_step(cur_{|q|})` は `Capture { capture_idx, .. }` である。`|λ| ≥ |u| = |q| + 1 > |q|` なので
  L1 の場合 (b) の `Capture` の行より `trunc(τ, λ) = λ[0..|q|+1] = u` である。
  BY L1, L6

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
  <2>4. 上の 3 つでも `is_funptr` でもないとき、`<2>1` より `unpunched_field_types(τ)` のあるフィールド
        `fty` について `is_fully_unboxed(fty)` は偽である。`go` も `rc_units_go` もそのフィールドの下へ
        降りるので、型の高さについての帰納で `leaves(τ) ≠ ∅` かつ `units(τ) ≠ ∅` が出る。
    A10 より型の in-place の降下は有限なので、この帰納は基底に着く。
    BY <2>1, <2>2, <2>3, A10, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
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
  <2>1. `Unit` の行では `p[0..m] ∈ units(τ)` である。
    `i < m` の各位置で `unit_step(cur_i)` は `Fields` であって `p[i]` は `held_fields` に含まれる
    (DEF 歩み)。`rc_units_go` はその各段で同じフィールドの下へ降りるので `path = p[0..m]` で `cur_m` に
    達し、`unit_step(cur_m) = Unit` なので `path` を積む。
    BY L1, DEF 歩み, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>2. `Capture` の行では `p[0..m+1] ∈ units(τ)` である。
    `<2>1` と同じ降下で `rc_units_go` は `path = p[0..m]` で `cur_m` に達し、
    `unit_step(cur_m) = Capture { capture_idx, .. }` なので `path ++ [capture_idx]` を積む。
    `p[m] = capture_idx` なのでこれは `p[0..m+1]` である。
    BY L1, DEF 歩み, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>3. QED
    BY L1, <2>1, <2>2

<1>3. QED
  `sub(τ, p)` は `Some` か `None` を返すか中断する。中断するとき `under(τ, p)` も中断する (L2) ので
  言明の仮定が成り立たない。
  BY <1>1, <1>2, L2

### L10 (leaf に届く path では歩みが中断しない)

**言明**。`covered(τ, p) ≠ ∅` のとき、`trunc(τ, p)` と `sub(τ, p)` は値を返し、`under(τ, p)` の各要素
`unit` について `trunc(τ, unit)` も値を返す。

<1>1. `λ ∈ leaves(τ)` について、`boxed_leaf_paths` の `go` が `λ` を積むのは次の 2 つの場合だけであり、
      どちらでも `τ` の `λ` に沿う歩みは次のとおりである。
      (i) `path = λ` の位置で `is_box` または `is_array` が真: `i < |λ|` の各位置で `unit_step(cur_i)` は
      `Fields { held_fields, .. }` で `held_fields` は `λ[i]` を含み、`unit_step(cur_{|λ|}) = Unit`。
      (ii) `λ = μ ++ [CLOSURE_CAPTURE_IDX]` で `path = μ` の位置で `is_closure` が真: `i < |μ|` の各位置で
      `unit_step(cur_i)` は `Fields` で `held_fields` は `λ[i]` を含み、
      `unit_step(cur_{|μ|}) = Capture { capture_idx: CLOSURE_CAPTURE_IDX, .. }`。
  `go` は `is_fully_unboxed` で何も積まずに戻り、`is_closure` で `path ++ [CLOSURE_CAPTURE_IDX]` を積んで
  戻り、`is_box` と `is_array` で `path` を積んで戻り、それ以外で `unpunched_field_types` の下へ降りる。
  降りた各位置ではこの 4 つがどれも成り立たないので `unit_step` は `Fields` を返し、その `held_fields` は
  `unpunched_field_types` そのものである。`unit_step` は `is_fully_unboxed` で `NoUnit`、`is_closure` で
  `Capture`、`is_box`/`is_array` で `Unit` を返す。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, DEF 歩み

<1>2. `p ⊑ λ` のとき、`trunc(τ, p)` と `sub(τ, p)` は値を返す。
  `<1>1` より、`τ` の `λ` に沿う歩みで `NoUnit` の位置は無く、`Fields` の位置では次の添字が
  `held_fields` に含まれ、`Capture` の位置では次の添字が `capture_idx` に等しい。`p` は `λ` の接頭辞なので
  `τ` の `p` に沿う歩みはこの歩みの先頭部分であり、L1 の場合 (b) の中断する 3 行 (`Fields` で添字が
  held でない行、`Capture` で添字が `capture_idx` でない行、`NoUnit` の行) のどれにも当たらない。
  BY <1>1, L1

<1>3. `λ ⊑ p` のとき、`trunc(τ, p)` と `sub(τ, p)` は値を返す。
  `<1>1` の場合 (i) では `unit_step(cur_{|λ|}) = Unit` であり、`i < |λ|` の各位置は `Fields` で添字は
  held である。`|λ| ≤ |p|` なので、`τ` の `p` に沿う歩みは位置 `|λ|` までは中断せず、`|λ| < |p|` なら
  そこで L1 の `Unit` の行に入り、`|λ| = |p|` なら L1 の場合 (a) になる。場合 (ii) では
  `unit_step(cur_{|μ|}) = Capture` であり、`|μ| < |λ| ≤ |p|` なので位置 `|μ|` で `Capture` の行に入り、
  `p[|μ|] = λ[|μ|] = CLOSURE_CAPTURE_IDX = capture_idx` なので中断しない。
  BY <1>1, L1

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
    BY L1
  <2>3. `covered(τ, p)` の各要素 `λ` について `|λ| > m` である。
    `λ ∈ leaves(τ)` かつ `λ ⊑ p` かつ `|λ| ≤ m` と仮定する。`boxed_leaf_paths` の `go` が `λ` を積むのは、
    `path = λ` の位置で `is_box` か `is_array` が成り立つときか、`λ = μ ++ [CLOSURE_CAPTURE_IDX]` で
    `path = μ` の位置で `is_closure` が成り立つときである。前者では `unit_step(cur_{|λ|})` が `Unit` に
    なるので DEF 歩み より `m ≤ |λ|`、あわせて `|λ| = m`。後者では `unit_step(cur_{|μ|})` が `Capture` に
    なるので `m ≤ |μ| = |λ| - 1 ≤ m - 1` となり矛盾する。よって `|λ| = m` かつ `unit_step(cur_m) = Unit`
    であり、`λ = p[0..m] = key`。この `λ` は `|λ| > m` を満たさないが、`trunc(τ, λ) = λ = key` である。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, DEF 歩み, L1
  <2>4. `covered(τ, p)` の各要素 `λ` について `trunc(τ, λ) = key` である。
    `<2>3` は `|λ| ≤ m` のとき `λ = key` かつ `trunc(τ, λ) = key` を与える。`|λ| > m` のとき、`λ` は `p` と
    比較可能なので `λ[0..m] = p[0..m]` であり、`λ[m] = p[m]` である (`|λ| > m` かつ `|p| > m` で、短い方が
    長い方の接頭辞)。よって `τ` の `λ` に沿う歩みも長さ `m` で止まり、`unit_step(cur_m)` は `<2>2` と同じ
    ものである。L1 より `trunc(τ, λ)` は `<2>2` と同じ式で与えられ、`key` に等しい。
    BY <2>2, <2>3, L1, DEF 歩み
  <2>5. QED
    仮定より `covered(τ, p) ≠ ∅` なので、`<2>4` の `λ` が 1 つ取れて `trunc(τ, λ) = key` であり、
    仮定より `(r, λ) ∈ OL` である。これが `<1>1` が `unit = p` について要求するものである。
    BY <1>1, <2>1, <2>4

<1>5. QED
  `under(τ, p) = []` の場合を `<1>2` が、`under(τ, p) ≠ []` の場合を `sub` の答えで分けて `<1>3` と
  `<1>4` が扱った。
  BY <1>2, <1>3, <1>4

## 3. P7e の証明

**言明** (README の P7e)。任意の root `r` と path `p` について、
`owns(r, p) = owns(r, trunc(ty(r), p))` である。`r` がこの版のパラメータ・capture でないときは、両辺とも
真である。

**読み方**。`r` がこの版のパラメータ・capture でないとき、`owns(r, q)` は `q` によらず真を返す
(`<1>1`)。このとき等式は、右辺の引数 `trunc(ty(r), p)` が値を持つ限りで成り立つ。`r` がこの版の
パラメータ・capture のときは `ty(r) = pty(r)` であり、両辺は同時に値を返すか同時に中断する。

<1>1. CASE `pty(r) = None`
  <2>1. `owns(r, q)` は、任意の `q` について真を返す。
    `owns_object` は `self.vars.param_tys.get(root)` で分岐し、`None` の腕で `true` を返す。この腕は
    `path` を読まない。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object の `None` の腕
  <2>2. QED
    `<2>1` より左辺は真であり、右辺も、その引数 `trunc(ty(r), p)` が値を持つならば真である。
    BY <2>1

<1>2. CASE `pty(r) = Some(τ)`
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
  `pty(r)` は `Option` なので `None` か `Some(τ)` のどちらかである。`<1>1` が前者を、`<1>2` が後者を
  扱った。
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

<1>4. `Reach` の要素 `(y, ρ)` について、`origin(y, ρ)` の `act` が `Reach(x, π)` に含まれることを、
      `Reach(y, ρ)` の有限性についての帰納で示す。
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
    各元の `acted_on()` の和であり、`<2>2` と同じ理由による。
    BY <2>2, DEF 再帰で訪れる対, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
  <2>4. QED
    `<1>1` の作り方を `<2>1`、`<2>2`、`<2>3` が尽くす。`<2>3` の場合は `<1>3` より
    `act ⊆ S ∪ {(y, ρ)}` であり、どちらも `Reach(x, π)` に属する。
    BY <1>1, <1>3, <2>1, <2>2, <2>3

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
    A12 より `ty(c)` は構造体であり、この腕の条件より boxed ではない。よって `is_closure` も `is_array` も
    偽である。`λ ∈ leaves(ty(y))` より `leaves(ty(y)) ≠ ∅` なので L8 より `is_fully_unboxed(ty(y))` は
    偽であり、A12 より `ty(y)` は `ty(c)` の第 `idx` フィールドの型であって、そのフィールドは
    `ty(c)` が持つフィールド (`unpunched_field_types` が返すもの) である。`is_fully_unboxed` は
    `unpunched_field_types` の全フィールドについての全称なので `is_fully_unboxed(ty(c))` も偽である。
    よって `boxed_leaf_paths` の `go` は `ty(c)` について `unpunched_field_types` の下へ降り、第 `idx`
    フィールドについて `ty(y)` から始めた `go` の結果の前に `idx` を置いたものを積む。
    BY A12, L8, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. QED
    `<2>1` と `<2>2` より `[idx] ++ λ ∈ covered(ty(c), [idx] ++ ρ)` である。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: covered_leaves

<1>5. `Binding::Payload(s, Some(t))` (`s` が unbox) の辺は性質を保つ。
  A12 より `ty(s)` は union であり、`ty(y)` はその第 `t` 変位の型である。この腕の条件より `ty(s)` は
  boxed ではないので `is_closure` も `is_array` も偽である。`<1>4` の `<2>2` と同じ降下で
  `[t] ++ λ ∈ leaves(ty(s))` であり、`<1>4` の `<2>1` と同じ計算で `[t] ++ λ` は `[t] ++ ρ` と比較可能で
  ある。
  BY A12, L8, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/borrow.rs: covered_leaves

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

### L15 (借用版は名前替えである)

**言明**。`func` を入力の関数、`clone` をその借用版、`rename` を `clone_func` が返す写像とし、`ρ` を
`rename` を `rename` の鍵でない名前上の恒等写像で延ばしたものとする。`vars_f = VarTable::of(func)`、
`vars_c = VarTable::of(clone)` と置く。このとき次が成り立つ。

- `ρ` は単射であり、`vars_c.param_tys` の鍵は `vars_f.param_tys` の鍵の `ρ` による像ちょうどであって、
  `vars_c.param_tys[ρ(p)] = vars_f.param_tys[p]` である。
- 任意の `(x, π)` について `origin(vars_c, type_env, ρ(x), π) = ρ(origin(vars_f, type_env, x, π))` である。
  ここで `ρ` は `VarPath` に対しその変数だけを写す。

<1>1. `clone` の本体・パラメータ・capture は、`func` のそれの束縛変数を `rename` で一斉に付け替えたもの
      であり、それ以外の違いを持たない。
  BY P9, CODE src/rc_ir/borrow.rs: clone_func

<1>2. `ρ` は単射である。
  `assign_fresh_name` は `counter` を 1 増やしてから `name#<tag><counter>` を作るので、相異なる束縛には
  相異なる新しい名前が付く。P9 の後半より、新しい名前は入力のどの束縛名とも異なる。`ρ` は `rename` の鍵で
  ない名前を動かさないので、鍵の像と非鍵の像は交わらない。
  BY P9, CODE src/rc_ir/rename.rs: assign_fresh_name

<1>3. `vars_c.param_tys` は `vars_f.param_tys` の `ρ` による像であり、型は変わらない。
  `VarTable::of` は `func.params` と `func.capture` から `param_tys` を作る。`fresh_rename_function` は
  各パラメータ・capture に `rename_var` を掛け、`rename_var` は名前だけを差し替えて型 `ty` を残す。
  BY <1>1, CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/rename.rs: fresh_rename_function,
     rename_var

<1>4. `vars_c.bindings[ρ(x)]` は `vars_f.bindings[x]` の変数を `ρ` で写したものであり、
      `vars_c.var_tys[ρ(x)] = vars_f.var_tys[x]` である。
  `collect_bindings` は本体の形だけから `bindings` と `var_tys` を作り、記録するのは束縛変数の名前と、
  右辺に現れる変数 (`Move` の `y`、`Field` の容器、`Payload` の scrutinee、`Join` のアーム結果、`Llvm` の
  オペランド) である。`<1>1` よりこれらはすべて `ρ` で写されており、型は変わらない。
  BY <1>1, CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/rename.rs: rename_expr

<1>5. QED
  `origin_inner` が読むのは `vars.bindings`、`vars.param_tys` を経由しない情報 (`args[j].ty`、
  `container.ty`、`scrut.ty`、`result_ty`)、および `type_env` だけである。`LLVMGen::result_prov` は
  `(result_ty, arg_tys, type_env)` だけを引数に取るので、オペランドの名前を読まない。`<1>4` よりこれらは
  `ρ` の下で対応するので、`origin_inner` の各腕の答えは `ρ` で写り、`origin` の再帰も同じ形で写る。
  A15 と P2 より再帰は停止するので、この対応は全域である。
  BY <1>2, <1>3, <1>4, A15, P2, CODE src/rc_ir/ownership.rs: origin, origin_inner,
     origin_from_leaves_under, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov

### L16 (借用版の `owns_object` は推論の `owns_object_yet` である)

**言明**。`OL` を `infer_ownership` の不動点の `owned_leaves` とし、`owned_units` を `borrow_ify` が組む
集合、`ctx` を `clone` (`func` の借用版) の `RewriteCtx` とする。このとき、`func` の任意の名前 `r` と
任意の path `p` について、`ctx.owns_object(ρ(r), p) = owns_object_yet(vars_f, type_env, r, p, OL)` である
(両辺は同時に中断する)。

<1>1. `pty_f(r) = None` のとき、両辺とも真である。
  L15 より `vars_c.param_tys` の鍵は `vars_f.param_tys` の鍵の像ちょうどであり、`ρ` は単射なので
  `ρ(r)` は `vars_c.param_tys` の鍵でない。`owns_object` と `owns_object_yet` はどちらもこの場合に真を
  返す。
  BY L15, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet

<1>2. `pty_f(r) = Some(τ)` のとき、`ctx.vars.param_tys[ρ(r)] = τ` である。
  BY L15

<1>3. `owned_units` は、`(ρ(r), k)` を含むことと、「`leaves(τ)` のある `leaf` について
      `trunc(τ, leaf) = k` かつ `(r, leaf) ∈ OL`」が成り立つことが同値である。
  <2>1. `borrow_ify` は、借用版を持つ各 `func` の各パラメータ `p` の各 `leaf ∈ boxed_leaf_paths(p.ty)` に
        ついて、`owned_leaves.owns(p.name, leaf)` が真のとき `(rename[p.name], trunc(p.ty, leaf))` を
        `owned_units` に入れる。これが `ρ(r)` を第 1 成分とする唯一の挿入である。
    `owned_units` へのもう 1 つの挿入は `owned_units.extend(param_capture_units(func))` であり、その
    第 1 成分は入力の関数のパラメータ・capture の名前である。P9 より `ρ(r)` はそのどれとも異なる。
    別の関数の借用版の挿入は、その関数のパラメータの `rename` による像であり、L15 が述べる `ρ` の
    単射性より `ρ(r)` と異なる。
    BY P9, L15, CODE src/rc_ir/borrow.rs: borrow_ify
  <2>2. QED
    `OwnedLeaves::owns(var, path)` は `(var, path) ∈ OL` である。
    BY <2>1, CODE src/rc_ir/borrow.rs: OwnedLeaves::owns

<1>4. QED
  `<1>2` より `owns_object(ρ(r), p)` は「`under(τ, p)` の各 `unit` について
  `(ρ(r), trunc(τ, unit)) ∈ owned_units`」であり、`<1>3` よりこれは「各 `unit` について、
  `trunc(τ, leaf) = trunc(τ, unit)` かつ `(r, leaf) ∈ OL` である `leaf ∈ leaves(τ)` が在る」に等しい。
  これが `owns_object_yet(vars_f, type_env, r, p, OL)` の定義そのものである。両辺は `under(τ, p)` と
  `trunc(τ, ・)` を同じ引数で呼ぶので、中断も同時である。
  BY <1>2, <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object, owns_object_yet

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
    L16 より `ctx.owns_object(ρ(r), p) = yet(r, p)` であり、`<1>4` より借用版の site の候補は
    `ρ(cand(vars_f, v, u))` である。よってすべての候補について `owns_object` は偽である。
    BY <2>1, <1>4, L16

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
    `<2>3` と `<2>4` よりすべての候補について `yet` は真であり、L16 と `<1>4` よりすべての候補について
    `owns_object` は真である。
    BY <2>3, <2>4, <1>4, L16

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

## 6. P7a -- 呼び出し位置の証明と、節 2 の反例

README の P7a は、`levelled_sites` が挙げる site `(v, u)` と `Λ(u) = Λ_{ty(v)}(u)` について、
`infer_ownership` の不動点の下で次の 3 つが同値である、という言明である。

1. `owns_unit(v, u)` が真である。
2. `Λ(u)` の**ある** leaf `λ` の**すべての**候補 `(r, p)` について `owns_object(r, p)` が真である。
3. `Λ(u)` の**すべての** leaf のすべての候補について `owns_object` が真である。

この節は、言明が自分で負う義務 (`owns_unit` を呼ぶ位置が site を出ないこと) を L17 で証明し、節 1 と
節 2 の同値が偽であることを反例 R1 で示す。最後に、`level_ownership` が `union_as` の形の隔たりを
埋めるかどうかに答え、修復案を述べる。

### L17 (`owns_unit` を呼ぶ位置は site を出ない)

**言明**。ある出力版の `RewriteCtx` が `owns_unit(v, u)` を呼ぶとき、その版が関数の版であれば `(v, u)` は
その版の本体について `levelled_sites` が挙げる site である。その版がグローバル初期化子のものであれば、
`owns_unit(v, u)` は真を返す。

<1>1. `owns_unit` を呼ぶのは `any_owned_unit`、`routing_saves_retain`、`call_rc`、`rewrite_rc` の 4 か所で
      ある。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::any_owned_unit, RewriteCtx::routing_saves_retain,
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

### R1 (節 1 と節 2 は同値でない)

**言明**。P7a の仮定 (site、`infer_ownership` の不動点) をすべて満たし、節 2 が真で節 1 が偽である
入力プログラムがある。

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

<1>2. 次の関数を入力に取る。`z : Array I64` は唯一のパラメータであり、`w : I64` は定数を束縛する。

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
  持たないので消費は無い。読む構文が読むオブジェクトは、その時点で解放されていない。
  BY A1, A2, D9, D10, D16, <1>1, CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov

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
  <2>3. `origin(x, [])` は `Exactly((z, []))` である。
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
  <2>4. `level_ownership` は site `(x, [])` で `false` を返し続ける。
    `<2>3` より候補は `(z, [])` 1 つである。`owns_object_yet(vars, type_env, z, [], OL)` は、
    `pty(z) = Some(Array I64)` なので `units_under(Array I64, [])` の各 unit について「同じ鍵を持つ
    所有された leaf」を要求する。`sub(Array I64, []) = Some(Array I64)` であり `units(Array I64) = [[]]`
    (`is_array` で `UnitStep::Unit`、L3) なので `under = [[]]`、鍵は `trunc(Array I64, []) = []`、
    `leaves(Array I64) = {[]}` なので `(z, []) ∈ OL` が要る。`OL` が空ならこれは偽であり、
    `owns_a_candidate` は偽になって `level_ownership` は `false` を返す。
    BY <2>3, L2, L3, CODE src/rc_ir/borrow.rs: level_ownership, owns_object_yet
  <2>5. QED
    `infer_ownership` は空の `owned_leaves` から始め、消費 (`<2>1`) からも `level_ownership` (`<2>2`,
    `<2>4`) からも要素を加えない。よって最初の周で `changed` は偽になり、不動点は空集合である。
    BY <2>1, <2>2, <2>4, CODE src/rc_ir/borrow.rs: infer_ownership

<1>4. `f` は借用版を持ち、その借用版で `owns_object(ρ(z), [])` は偽である。
  `func_has_borrowable_param` は、あるパラメータのある leaf が `owned_leaves` に無いことを問う。`<1>3` より
  `z` の leaf `[]` は無い。`f` は capture を持たず、`union_make_1` も `int_lit_0` も
  `observes_uniqueness` を override しないので `funcs_observing_uniqueness` は `f` を含まない。よって
  `borrow_versions` は `f` を持つ。`borrow_ify` が借用版の `owned_units` に入れるのは `owned_leaves` が
  所有する leaf の像だけなので、`(ρ(z), [])` は `owned_units` に無い。L16 と `<2>4` の計算より
  `owns_object(ρ(z), [])` は偽である。
  BY <1>3, L16, CODE src/rc_ir/borrow.rs: func_has_borrowable_param, funcs_observing_uniqueness,
     borrow_ify, CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness

<1>5. 節 1 は偽である。
  L15 と `<1>3` の `<2>3` より、借用版の site `(ρ(x), [])` の候補は `{(ρ(z), [])}` である。
  `owns_unit` はその全称なので、`<1>4` より偽である。
  BY L15, <1>3, <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

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

<1>7. QED
  `<1>5` と `<1>6` より、この site で節 2 は真、節 1 は偽である。
  BY <1>5, <1>6

### R1 が壊すもの、壊さないもの

**コードは正しい。** `owns_unit(ρ(x), []) = false` なので、借用版の `rewrite_rc` は `Release(x, [])` を
落とす (P10)。これは正しい。`x` の leaf `[1]` が持つ参照は `z` のものであり、`z` は借用されているので、
その参照を処分するのは呼び出し元である。`x` の leaf `[0, 0]` は変位 0 の下にあり、`x` の変位は 1 なので
どの実行でも inhabited にならない (D16)。すなわち節 2 が数えた leaf は、実行時に参照を持たない。

**節 3 は R1 で偽である。** `λ = [1]` の候補は `(ρ(z), [])` であり、`owns_object` は偽である。よって
R1 が壊すのは節 1 と節 2 の同値だけであり、節 1 と節 3 の同値は壊れていない。

### `level_ownership` は `union_as` の隔たりを埋めるか

**(★) は要らない。**いまの言明はこの隔たりを通らないからである。前の版の言明 (候補集合についての
双条件) は、節 3 から節 1 へ直接渡ることを要求していた。その向きは `union_as` の宣言で破れる -- `union_as_k` は unbox union のオペランドについて変位 `k` の leaf
だけを名指すので (`CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov`)、unit の側が
`truncate_to_unit` で辿り着くオペランドの unit には、どの結果 leaf も名指さない leaf が残る。その leaf の
所有は leaf の側から言えない。**前稿が (★) と呼んだ「宣言についての仮定」は、この向きを直接証明しようと
した結果であって、いまの言明には要らない。** いまの言明は節 2 から節 1 へ渡り、その向きが読むのは
P7d である。節 1 が偽ならば、P7d より site の候補はすべて偽であり、`level_ownership` はその site で
発火していない。すなわち `union_as` が作る隔たりは、いまの言明の証明義務に現れない。

**この節は節 1 と節 3 の同値を証明していない。**証明したのは L17 と、節 1 と節 2 が同値でないこと
(R1) だけである。

**残る隔たりは `⊥` の leaf である。** `result_prov` が空集合を宣言した leaf は、`origin` が値自身の名前で
答え、その名前はパラメータでないので `owns_object` が真を答える。R1 はその leaf 1 つでできている。
`level_ownership` はこれを埋められない。埋めるべきものが無いからである -- その leaf は実行時に参照を
持たない。

### 修復案

節 2 と節 3 を、`Λ(u)` の leaf のうち **inhabited (D16) なもの**に限る。A3 の表の第 1 行が
「空集合と宣言された leaf は inhabited にならない」と述べるので、R1 の `[0, 0]` はこの限定で落ちる。
この限定は P14 が要るものでもある。借用版が `Retain(v, u)` / `Release(v, u)` を落とすとき漏れるかどうかを
決めるのは、`u` の下の inhabited な leaf が持つ参照だけである (A4、D10)。

この修復は言明の変更なので、この文書では行わない。README を書き換えるのはオーケストレータである。
