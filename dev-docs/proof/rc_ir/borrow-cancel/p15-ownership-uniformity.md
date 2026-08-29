# P7e -- `owns_object` は unit ごとに答える

この文書は `README.md` の P7e を証明する。立つのは `README.md` の定義 D1-D20、仮定 A1-A15、および命題
P1-P7 の**言明**である。

この文書は P7a と P7d を証明しない。第 4 節が、P7a の証明が閉じない位置を述べる。そこで要るのは
`LLVMGen::result_prov` の宣言についての事実であり、A3 はその事実を述べていない。P7d はその後に来る
命題なので、書いていない。

読んだコードは作業ツリーの版である。README の対象コミット `a924f115` との差分は、生成される `// PROOF:`
の注釈行だけであり、この文書が引用する記号の本文は対象コミットと一致する。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P7e (`owns_object` は unit ごとに答える) | 証明した |
| P7a (unit の所有と leaf の所有は一致する) | **未証明**。第 4 節に、閉じない位置と、そこで要る事実を書く |
| P7d (所有は site ごとに一様である) | **未着手** |

P7e の要は L1 である。`subtree_type` と `truncate_to_unit` は、同じ型の列の上を同じ順に歩き、同じ
`unit_step` の答えで場合を分ける。よって片方が `None` を返す位置は、もう片方が `break` する位置である。

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

## 2. 補題

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
  終わる。よってどちらのループも `π` の要素をすべて使い切って通常どおりループを抜ける。`<1>1` より
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

## 4. P7a の証明が閉じない位置

P7a の `⟸` の向き (「`Λ(u)` の各 leaf の候補がすべて所有される」から「`owns_unit(v, u)` が真」を出す
向き) は、`origin_inner` の `Binding::Llvm` の腕で閉じない。この節は、閉じない位置と、そこで要る事実を
述べる。

### 4.1 何が要るか

`origin` の再帰の各腕について、unit path `u` の答えと、その下の leaf `λ` の答えを突き合わせる。
**以下の帰着はこの文書が証明していない。**これは、`Binding::Llvm` の腕だけが残ることを示すための
見取り図である。

- `here()` を返す腕 (`None` / `Binding::Param` / `Binding::Producer`、boxed 容器の `Binding::Field`、
  boxed union の `Binding::Payload`、`origin_from_leaves_under` が `None` を返したときの
  `unwrap_or_else(here)`) では、候補は `{(v, u)}` と `{(v, λ)}` である
  (`CODE src/rc_ir/ownership.rs: origin_inner`)。`u ∈ units(ty(v))` かつ `λ` が `u` を前置に持つ leaf の
  とき `trunc(ty(v), λ) = u` であり (L1 と `rc_units_go` から出る)、P7e より
  `owns(v, λ) = owns(v, u)` なので、2 つは同値である。
- `Binding::Move`、unbox 容器の `Binding::Field`、unbox union の変位アームの `Binding::Payload`、
  catch-all の `Binding::Payload` の腕では、`u` と `λ` に同じ写像 (path の前置) が掛かるので、
  主張は 1 段内側の同じ主張に帰着する (`CODE src/rc_ir/ownership.rs: origin_inner`)。
- `Binding::Join` の腕では、候補は各アームの結果の `acted_on()` の和である。`Origin::Join` の
  `identity` は `of_candidates` の第 2 引数であり、それを渡すのは `Binding::Join` の腕
  (`(var, path)`、`var` は `Match` の束縛変数) と `origin_from_leaves_under` (`here`、`var` は
  `Binding::Llvm` を持つ変数) の 2 か所だけなので、`Join` の `identity` の変数はこの版のパラメータでは
  なく、`owns` はそれに真を答える。よって `acted_on()` と `candidates()` は同じ答えを与え、主張は
  各アームについての同じ主張に帰着する
  (`CODE src/rc_ir/ownership.rs: Origin::of_candidates`, `Origin::acted_on`, `origin_inner`)。
- `Binding::Llvm` の腕では、unit の側は `origin_from_leaves_under` が
  `truncate_to_unit(&args[j].ty, leaf, ..)` でオペランドの **unit** へ丸めてから辿り、leaf の側は
  `as_arg_projection` がオペランドの **leaf** をそのまま辿る
  (`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`, `as_arg_projection`)。

最後の腕で、`u` の下の leaf が宣言する `Arg(j, σ)` を集めると、unit の側は `w = trunc(ty(args[j]), σ)`
に立つ。1 段内側の同値を `(args[j], w)` に当てると、unit の側は `w` の下の**すべての** leaf の所有を
要求する。leaf の側が与えるのは、`u` の下の leaf が名指した `σ` についての所有だけである。よって
`⟸` は、`w` の下の名指されなかった leaf `σ'` についても所有が言えるときにだけ閉じる。

### 4.2 名指されない leaf が実在すること (O1)

`union_as` は、unbox union のオペランドについて、変位 `field_idx` の leaf だけを名指す。

<1>1. `InlineLLVMUnionAsBody::result_prov` は、オペランドが unbox union のとき、結果の各 leaf `path` に
      `sole_origin(LeafOrigin::Arg(0, [variant_index] ++ path))` を置く。
  BY CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov

<1>2. 次の 2 つの型を取る。

      ```
      type Inner = unbox union { p : Array I64, q : I64 };
      type Outer = unbox union { a : Inner, b : Array I64 };
      ```

      このとき `leaves(Inner) = {[0]}`、`units(Inner) = {[]}`、`leaves(Outer) = {[0, 0], [1]}`、
      `units(Outer) = {[]}` である。
  `boxed_leaf_paths` は `is_fully_unboxed` でも `is_closure` でも `is_box` でも `is_array` でもない型に
  ついて `unpunched_field_types` の下へ降りるので、`Inner` の変位 `p : Array I64` から `[0]` を、変位
  `q : I64` からは `is_fully_unboxed` で何も出さない。`Outer` についても同じ降下で `[0, 0]` と `[1]` を
  出す。`unit_step` は `is_union` で `UnitStep::Unit` を返すので、`rc_units` はどちらの型についても
  `[[]]` を出す (L3)。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/rc_ir/ownership.rs: unit_step, L3

<1>3. `z : Array I64` をこの版のパラメータとし、本体に
      `Let(c, Llvm(union_make_1, [z]), Let(y, Llvm(union_as_0, [c]), ...))` があるとする
      (`ty(c) = Outer`、`ty(y) = Inner`)。このとき `origin(c, [0, 0]) = Exactly((c, [0, 0]))` であり、
      `origin(c, []) = Exactly((z, []))` である。
  `InlineLLVMMakeUnionBody::result_prov` は、結果の leaf の path の先頭が変位番号 `1` に等しいときだけ
  `Arg(0, rest)` を置き、他の変位の leaf には `Set::default()` を置く。よって `Outer` の leaf `[0, 0]`
  の宣言は空集合、leaf `[1]` の宣言は `{Arg(0, [])}` である。
  path `[0, 0]` では `leaf_origins_at` が空集合を持つ `Some` を返し、`as_arg_projection` は要素数が
  `1` でない集合に `None` を返すので
  `origin_from_leaves_under` に入り、その `leaf_origins_under([0, 0])` は空集合 1 つだけを与えるので
  `operand_units` も `produced_here` も空のまま `reached` が空になり、`reached.first()?` が `None` を
  返して `unwrap_or_else(here)` が `Exactly((c, [0, 0]))` を返す。
  path `[]` では `leaf_origins_at` が `None` を返し (`[]` は `Outer` の leaf ではない)、
  `origin_from_leaves_under` の `leaf_origins_under([])` が leaf `[0, 0]` の空集合と leaf `[1]` の
  `{Arg(0, [])}` を与えるので、`operand_units = {(0, trunc(Array I64, []))} = {(0, [])}`、
  `reached = [origin(z, [])] = [Exactly((z, []))]` となり、`reached` のすべてが `first` に等しいので
  その `Origin` がそのまま返る。
  BY CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov,
     CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under, as_arg_projection,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, leaf_origins_under, <1>2

<1>4. `origin(y, [0]) = Exactly((c, [0, 0]))` であり、`origin(y, []) = Exactly((z, []))` である。
  `<1>1` より `union_as_0` の宣言は `Inner` の leaf `[0]` に `{Arg(0, [0, 0])}` を置く。path `[0]` では
  `as_arg_projection` が `(0, [0, 0])` を返すので `origin(c, [0, 0])` が答えになる。path `[]` は
  `Inner` の leaf ではないので `origin_from_leaves_under` に入り、`leaf_origins_under([])` が
  `{Arg(0, [0, 0])}` を与えるので `operand_units = {(0, trunc(Outer, [0, 0]))} = {(0, [])}` となり
  (`unit_step` は `is_union` で `UnitStep::Unit` を返すので `trunc` は最初の周で `break` する)、
  `reached = [origin(c, [])]` の 1 元だけになり、その `Origin` がそのまま返る。
  BY <1>1, <1>2, <1>3, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under,
     as_arg_projection, truncate_to_unit, unit_step

<1>5. `Λ([]) = {[0]}` であり、その唯一の leaf の候補 `(c, [0, 0])` について `owns(c, [0, 0])` は真で
      ある。一方 `owns_unit(y, [])` は `owns(z, [])` に等しい。
  `<1>2` より `ty(y) = Inner` の `[]` の下の boxed leaf は `[0]` だけである。`VarTable::of` が
  `param_tys` に入れるのはパラメータと capture だけであり、`collect_bindings` は `param_tys` に何も
  入れない。`c` は本体の `Let` が束縛する変数であり、A6 よりその名前はパラメータの名前と異なるので、
  `c` は `param_tys` に無い。よって P7e の後半より `owns(c, [0, 0])` は真である。
  `owns_unit(y, [])` は `origin(y, []).candidates()` のすべてに `owns` を要求するもので、`<1>4` より
  その候補は `(z, [])` 1 つである。
  BY <1>2, <1>4, A6, P7e, CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings,
     CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

**O1**。この本体では、`owns_unit(y, [])` は `(z, [])` の所有を問い、`Λ([])` の leaf の候補は `(c, [0, 0])`
だけである。2 つの候補集合は根を共有しない。よって P7a の同値は、一方の候補集合をもう一方へ写す形では
言えない。

### 4.3 O1 の本体で 2 つの答えが揃う理由 (O2)

`z` は `infer_ownership` の不動点で所有される。

<1>1. `passthrough_arg_leaves` は、宣言の leaf のうち単一の `Arg` であるものを集める。`union_as_0` の
      宣言について、これは `{(0, [0, 0])}` である。
  BY CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, 4.2 の `<1>1`, 4.2 の `<1>2`

<1>2. `InlineLLVMUnionAsBody::borrows_operand(0, ..)` は、変位の payload 型が `is_fully_unboxed` で
      あるときだけ真である。`Inner` は `Array I64` を持つのでこれは偽である。
  BY CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::borrows_operand, InlineLLVMUnionAsBody::borrows_union

<1>3. `rhs_consumes` は、`union_as_0` の節点について `(c, [1])` を消費として報告する。
  `RcRhs::Llvm` の腕は、`borrows_operand(i)` が偽のオペランドの各 boxed leaf のうち `passthrough` に
  無いものを `out` に積む。`<1>1` と `<1>2` より、オペランド `c` の leaf `[0, 0]` は `passthrough` に
  あり、leaf `[1]` は無い。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕, 4.2 の `<1>2`

<1>4. `infer_ownership` は `(z, [])` を `owned_leaves` に入れる。
  消費 `(c, [1])` について `origin(c, [1])` の候補を辿り、`param_tys` にある根を `owned_leaves` に
  入れる。`union_make_1` の宣言は `Outer` の leaf `[1]` に `{Arg(0, [])}` を置くので
  (4.2 の `<1>3`)、`as_arg_projection` が `(0, [])` を返し、`origin(c, [1]) = origin(z, [])` である。
  `z` はパラメータなのでこれは `Exactly((z, []))` であり、その候補 `(z, [])` は `param_tys` にある。
  BY <1>3, CODE src/rc_ir/borrow.rs: infer_ownership, CODE src/rc_ir/ownership.rs: origin_inner,
     as_arg_projection, 4.2 の `<1>3`

<1>5. `borrow_ify` は `(rename[z], [])` を `owned_units` に入れる (`rename` は `clone_func` が返す
      借用版への名前替え)。
  借用版の `owned_units` は、`owned_leaves` が所有する各パラメータ leaf を `truncate_to_unit` で unit へ
  丸め、`rename` を掛けて入れたものである。`trunc(Array I64, []) = []` である。
  BY <1>4, CODE src/rc_ir/borrow.rs: borrow_ify の `owned_units` を組む繰り返し

**O2**。`union_as` が名指さなかったオペランド leaf は、その節点の消費になる。消費された leaf の候補で
あるパラメータ leaf は不動点で所有される。よって O1 の本体では `owns(z, [])` も真になり、P7a の 2 つの
辺は揃う。

**この経路は P7a の言明に現れない。** P7a は `owns_unit` と `owns_object` だけを主語にしており、
`infer_ownership` の不動点にも消費にも触れない。O1 と O2 は、`⟸` の向きがその 2 つの上に立つことを
示している。

### 4.4 要る事実 (未証明)

4.1 の帰着と 4.3 の経路を合わせると、`⟸` の向きは次の事実に帰着する。

**(★)** 各 `Llvm` の演算、各オペランド添字 `j`、各 unit `w ∈ units(ty(args[j]))` について、`Λ_j(w)` を
`leaves(ty(args[j]))` のうち `w` を前置に持つものの集合とする。結果のある unit `π` の下のある leaf が
`Arg(j, σ)` (`σ ∈ Λ_j(w)`) を宣言するならば、`Λ_j(w)` の各 leaf `σ'` は次のどちらかを満たす。

- `π` の下のある結果 leaf が `Arg(j, σ')` を宣言する。
- どの結果 leaf も `Arg(j, σ')` を宣言せず、かつ `borrows_operand(j)` が偽である
  (このとき `σ'` はその節点の消費になり、O2 の経路でその候補が所有される)。

A3 はこの事実を述べていない。A3 が数え上げているのは「宣言が leaf に置く集合の要素数は 0 か 1」だけで
ある。(★) が破れるのは次の 2 つの形であり、どちらも A3 の数え上げからは排除されない。

1. `Arg(j, σ')` を宣言する結果 leaf が、`π` とは別の結果 unit の下にある。この `σ'` は
   `passthrough_arg_leaves` に入るので消費にならず、O2 の経路が届かない。
2. `borrows_operand(j)` が真で、かつ結果のどの leaf も `Arg(j, σ')` を宣言しない。この `σ'` も消費に
   ならない。

この 2 つが実在しないことは、`result_prov` を override する 29 個の宣言を数え上げて示すことになる。
A3 が同じ形の数え上げを持つので、置き場所は A3 である。
