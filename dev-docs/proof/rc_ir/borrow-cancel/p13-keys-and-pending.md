# P7b, P7c, P18a: 鍵と pending

この文書は README の P7b, P7c, P18a を扱う。README の定義 D1 - D19、仮定 A1 - A14、および命題
P1 - P7, P15 - P18 の**言明**の上に立つ。それらの証明は
`p10-leaves-and-units.md`、`p11-origin-soundness.md`、`p12-keys-and-consumes.md`、
`p30-cancel-walk.md` にあり、この文書はそれらの言明だけを使う。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P7b | **偽。** 反例 X が破る。制限つきの形 (L11) は証明した |
| P7c | **偽。** 同じ反例 X が破る。P5 (a) との差は第 6 節が述べる |
| P18a | **偽。** 反例 Y が破る。反例 Y は節点の削除を伴わないので、P7b の欠陥とは別に立つ |

**コードの欠陥を 1 件見つけた。** 反例 X は Fix のプログラムとして書ける。`-O max` で、`cancel` が
`Retain`/`Release` の対を消費をまたいで消し、解放済みのオブジェクトを読むコードを作る。`-O none` と
`-O basic` は正しい。第 4 節が形式的な追跡を、第 5 節が Fix の再現プログラムと計測を、第 11 節が
診断をまとめる。

**証明を止めた所。** P7b が偽であることが確かめられた時点で、P7c と P18a を「P7b を仮定して」証明
する道は残るが、その仮定は現在のコードでは成り立たない。よってこの文書は、3 つの命題それぞれに
ついて (1) 反例、(2) 反例が消えたときに残る段、の 2 つを書き、修正の方向は第 11 節で候補として挙げる
にとどめる。どの修正を採るかは、`unit_key` が何を名指す量であるかという設計判断である。

## 1. 記法

1 つの関数 (またはグローバル初期化子) の本体 `B` を固定し、`B` から作られる `VarTable` を `vars`、
プログラムの `TypeEnv` を `type_env` と書く (`CODE src/rc_ir/ownership.rs: VarTable::of`,
`CODE src/rc_ir/ownership.rs: VarTable::body_only`)。この 2 つは本体ごとに 1 つなので、以下では
`origin`、`unit_key`、`acted_references`、`acted_unit_keys` の第 1・第 2 引数を落として書く。

- `ty(x)` は変数 `x` の型、すなわち `vars.var_tys[x]` (`CODE src/rc_ir/ownership.rs: VarTable`)。
- `origin(x, p)` は `origin(vars, type_env, x, p)` (D13)。
- `id(x, p)` は `origin(x, p).identity()` (`CODE src/rc_ir/ownership.rs: Origin::identity`)。
- `unit_of(n)` は `unit_of(vars, type_env, n)` (`CODE src/rc_ir/ownership.rs: unit_of`)。
- `key(x, p)` は `unit_key(vars, type_env, x, p)` (D15)。D15 より `key(x, p) = unit_of(id(x, p))`。
- `trunc(t, p)` は `truncate_to_unit(t, p, type_env)` (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。
- `leaves(t)` は `boxed_leaf_paths(t, type_env)` (D4)。
- `units(t)` は `rc_units(t, type_env)` (D5)。
- `p <= q` は「path `p` は path `q` の前置である」、`p < q` は「真の前置である」。
- `p ++ q` は path の連結。

多重集合の記法。`References` は `Map<VarPath, usize>` を 1 つ持つ構造体であり
(`CODE src/rc_ir/ownership.rs: References`)、これを鍵を `VarPath`、値をその個数とする多重集合と
みなす。

補題の番号 `L1` - `L12` はこの文書の中だけのものである。

## 2. 局所の定義

### DEF 訪問

`walk_inner` の 1 回の呼び出しを**訪問**と呼び、その `node` 引数が指す節点を訪問した、という
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)。節点 `n` の訪問における `pending` 引数の値
(その訪問がそれに変更を加える前の値) を `pending(n)` と書き、**入口状態**と呼ぶ。P15 の後半より走査は
`B` の各位置をちょうど 1 回訪問するので、`pending(n)` は節点ごとに 1 つに定まる。

### DEF 節点の量

`Retain` 節点 `t = Retain(v, path, _, _)` と `Release` 節点 `r = Release(v, path, _, _)` について、

- `key(t) := key(v.name, path)`、`bumped(t) := acted_references(v, path)`
- `key(r) := key(v.name, path)`、`un_bumped(r) := acted_references(v, path)`
- `others(r) := acted_unit_keys(v.name, path)` の要素のうち `key(r)` と異なるもの

`CancelAnalysis::unit_key`、`CancelAnalysis::acted_unit_keys`、`CancelAnalysis::acted_references` は
`ownership` の同名関数に `self.vars` と `self.type_env` を渡して呼ぶ
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::unit_key`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_unit_keys`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`)。これらの値は `vars`、`type_env`、
および渡された `(変数, path)` だけで決まる。

### DEF 覆う

変数 `w`、path `p`、`lf` を `leaves(ty(w))` の元とする。`p` が `ty(w)` において `lf` を**覆う**とは、
`p <= lf` かつ `trunc(ty(w), p) = trunc(ty(w), lf)` であることをいう。

この語が要るのは、P7b の仮定「path は unit である」が `origin` の再帰について閉じないからである。
unbox union の変位アームの payload 束縛をたどると path の先頭に変位番号が付き、その結果は
scrutinee の型の unit ではない (P2 の言明が同じ観察を述べている)。「覆う」は閉じる (L5)。

### DEF pending が名指す個数

1 つの活性化と 1 つの実行路を固定する。その路の上の位置 `q` とオブジェクト `o` について、

`S(q, o) :=` `q` の訪問の入口状態 `pending(q)` に在る各要素 `e` について、`e.outstanding` の鍵
`(u, s)` のうち、`s` が `u` において inhabited (D16) かつ `obj(u, s) = o` であるものの個数を足し
上げたもの。

`H(q, o)` は D7 の参照カウント、`Obl(q, o)` は D10 の義務集合が持つ `o` への参照の個数を、位置 `q`
において読んだものとする。

### DEF 良い段

`origin` の再帰の 1 段を、3 つ組 `(w, p, lf)` -- 変数、問い合わせる path、その下の leaf -- で表す。
この段が**良い**とは、`vars.bindings.get(w)` の値と、その腕の中の分岐が次のいずれかであることを
いう。

- **(G1)** `None`、`Some(Binding::Param)`、`Some(Binding::Producer)`、`is_box` が真の
  `Some(Binding::Field(..))`、`is_box` が真の変位アームの `Some(Binding::Payload(..))`。すなわち
  `here()` を返す腕。ただし `w` が `vars.var_tys` に記録されていること。
- **(G2)** `Some(Binding::Move(_))`、`is_box` が偽の `Some(Binding::Field(..))`、
  `Some(Binding::Payload(_, None))`、`is_box` が偽の変位アームの `Some(Binding::Payload(..))`。
- **(G3)** `Some(Binding::Join(_))` であって、`origin(w, p)` と `origin(w, lf)` がともに
  `Origin::Join` であるか、ともに `Origin::Exactly` であるもの。
- **(G4)** `Some(Binding::Llvm(..))` であって `p = lf` であるもの。
- **(G5)** `Some(Binding::Llvm(..))` であって `p < lf` であり、`decl.leaf_origins_at(lf)` が単一の
  `LeafOrigin::Fresh` または単一の `LeafOrigin::Unknown` であるもの。
- **(G6)** `Some(Binding::Llvm(..))` であって `p < lf` であり、`decl.leaf_origins_at(lf)` が単一の
  `LeafOrigin::Arg(j, s)` であり、かつ `origin_from_leaves_under` が `p` について作る `reached` の
  元がすべて等しいもの。

ここで `decl` は `origin_inner` の `Binding::Llvm` の腕が計算する `llvm_gen.result_prov(...)` である
(`CODE src/rc_ir/ownership.rs: origin_inner`)。

3 つ組は再帰とともに次へ降りる。(G2) では `origin_inner` の腕が `origin(w', p')` を返し、対応する
leaf の側が `origin(w', lf')` を返す。その `(w', p', lf')` が次の段である。(G3) ではこの実行路が
選んだアームの結果変数 `x_i` について `(x_i, p, lf)` が、(G6) では `(args[j], U, s)` (記号は L11 の
<1>11 のもの) が次の段である。(G1)、(G4)、(G5) では次の段が無い。

## 3. 予備の補題

### L1 (boxed leaf は互いに真の前置にならない)

**言明** — 任意の型 `t` と `lf, mu` を `leaves(t)` の元とするとき、`lf < mu` は成り立たない。

**証明**

<1>1. `boxed_leaf_paths` の内部関数 `go` が `out` に path を積むのは 3 か所であり、そのいずれも
      積んだ直後に `return` する (クロージャの腕、`is_box` の腕、`is_array` の腕)。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
<1>2. `go` が `out` に積む path は、その時点の `path` 引数か、それに `CLOSURE_CAPTURE_IDX` を
      足したものである。`go` が自分を再帰的に呼ぶのは `unpunched_field_types` の繰り返しの中だけで
      あり、その各回は `path` に添字を 1 つ足してから呼ぶ。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
<1>3. QED
  <1>2 より、`lf` を真の前置に持つ path が積まれるとすれば、それは `lf` を積んだ呼び出しの部分木の
  中である。<1>1 よりその呼び出しは積んだ直後に返るので、その部分木にそれ以上の呼び出しは無い。
  BY <1>1, <1>2

### L2 (unit はその下の leaf の切り詰めである)

**言明** — 型 `t`、`pi` を `units(t)` の元、`lf` を `leaves(t)` の元とし、`pi <= lf` とする。この
とき `trunc(t, lf) = pi` である。とくに `lf := pi` と置いて `trunc(t, pi) = pi` である。

**証明**

<1>1. `pi` が `units(t)` の元であることは、`rc_units_go` が `t` から `pi` の添字を順にたどり、
      `pi` の各真の前置の位置で `unit_step` が `UnitStep::Fields` を返して降りたうえで、次の
      いずれかが成り立つことと同値である。(a) `pi` の位置で `unit_step` が `UnitStep::Unit` を
      返す。(b) `pi` の最後の添字を除いた前置の位置で `unit_step` が
      `UnitStep::Capture { capture_idx, .. }` を返し、`pi` の最後の添字が `capture_idx` である。
  <2>1. `rc_units_go` は `unit_step(ty)` で分岐し、`NoUnit` では何も積まず、`Capture` では
        `path` に `capture_idx` を足したものを積み、`Unit` では `path` をそのまま積み、`Fields`
        では各 held field について添字を足して自分を呼ぶ。
    BY CODE src/rc_ir/ownership.rs: rc_units_go
  <2>2. QED
    `rc_units(t)` は `rc_units_go(t, type_env, &mut vec![], &mut out)` の `out` である。<2>1 より
    `out` の各要素は、`Fields` の腕を 0 回以上たどった後の `Unit` の腕か `Capture` の腕が積んだ
    ものである。
    BY <2>1, CODE src/rc_ir/ownership.rs: rc_units
<1>2. `truncate_to_unit(t, lf)` は `lf` の添字を順にたどり、各位置で `unit_step(cur)` が `Fields` の
      ときだけ添字を `out` に積んで降り、`Unit` のときは何も積まずに `break`、`Capture` のときは
      添字が `capture_idx` に等しいことを表明してから積んで `break`、`NoUnit` のときは panic する。
      降りる先の型は `held_field_type(&held_fields, idx, ...)` である。
  BY CODE src/rc_ir/ownership.rs: truncate_to_unit
<1>3. `pi` の各真の前置の位置において、`lf` の添字は `pi` の添字に等しく、<1>1 よりそこでの
      `unit_step` は `Fields` である。よって `truncate_to_unit` はその位置で `pi` の添字を積み、
      `rc_units_go` が降りたのと同じ型へ降りる。
  <2>1. `rc_units_go` の `Fields` の腕が降りる型は `held_fields` の要素の第 2 成分であり、
        `truncate_to_unit` の `Fields` の腕が降りる型は同じ `held_fields` から `held_field_type` が
        添字で引いたものである。同じ添字に対して両者は同じ型である。
    BY CODE src/rc_ir/ownership.rs: rc_units_go, CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/rc_ir/ownership.rs: held_field_type
  <2>2. QED
    BY <1>1, <1>2, <2>1
<1>4. CASE <1>1 の (a)。`pi` の位置で `unit_step` は `Unit` を返す。`lf = pi` ならばそこで `lf` の
      添字が尽きて繰り返しが終わり、`lf` が `pi` より長ければ `Unit` の腕が `break` する。どちらでも
      `out = pi` である。
  BY <1>2, <1>3
<1>5. CASE <1>1 の (b)。`pi` の最後の添字を除いた前置の位置で `unit_step` は `Capture` を返す。
      `pi <= lf` よりその位置での `lf` の添字は `pi` の最後の添字であり、<1>1 の (b) よりそれは
      `capture_idx` に等しいので表明は通り、その添字を積んで `break` する。よって `out = pi` である。
  BY <1>1, <1>2, <1>3
<1>6. QED
  <1>1 の (a) と (b) が場合を尽くす。`NoUnit` の腕は <1>3、<1>4、<1>5 のどの位置でも通らない。
  BY <1>1, <1>4, <1>5

### L3 (切り詰めは前置を返し、冪等である)

**言明** — 型 `t` と、`truncate_to_unit` が panic せずに答える path `p` について、
`trunc(t, p) <= p` であり、`trunc(t, trunc(t, p)) = trunc(t, p)` である。

**証明**

<1>1. `truncate_to_unit` は `path` の添字を順に読み、`out` に積むのは読んだ添字だけであり、順序を
      変えない。よって `out` は `path` の前置である。
  BY CODE src/rc_ir/ownership.rs: truncate_to_unit
<1>2. `q := trunc(t, p)` を入力として同じ走査を行うと、`q` の各添字の位置での `unit_step(cur)` は
      `p` を入力としたときと同じである。どちらも `t` から同じ添字列をたどるので `cur` が同じ値だから
      である。
  BY <1>1, CODE src/rc_ir/ownership.rs: truncate_to_unit
<1>3. QED
  `p` を入力とした走査が `out = q` を返したのは、`q` の長さまで `Fields` の腕で積み続けた後、
  添字が尽きたか `Unit` か `Capture` の腕で止まったかである。`q` を入力とすると、同じ位置まで
  同じ添字を積み、そこで添字が尽きるので `out = q` になる。
  BY <1>1, <1>2

### L4 (`unit_of` は根を変えない)

**言明** — 任意の `(root, path)` について、`unit_of((root, path))` の第 1 成分は `root` である。
さらに `root` が `vars.var_tys` に記録されているとき
`unit_of((root, path)) = (root, trunc(ty(root), path))` であり、記録されていないとき
`unit_of((root, path)) = (root, path)` である。

**証明**

<1>1. `unit_of` は `vars.var_tys.get(root)` で分岐する。`None` の枝は `!root.is_local()` を表明して
      から `(root.clone(), path.clone())` を返す。`Some(ty)` の枝は `truncate_to_unit(ty, path, type_env)`
      を `truncated` とし、`rc_units(ty, type_env)` がそれを含むことを表明してから
      `(root.clone(), truncated)` を返す。
  BY CODE src/rc_ir/ownership.rs: unit_of
<1>2. QED
  どちらの枝も第 1 成分は `root` である。
  BY <1>1

### L5 (別名の辺は「覆う」を保つ)

**言明** — 変数 `w`、`lf` を `leaves(ty(w))` の元、`p` は `ty(w)` において `lf` を覆い `p < lf` で
あるとする。このとき次が成り立つ。

1. `vars.bindings.get(w)` が `Some(Binding::Move(y))` のとき、`p` は `ty(y)` において `lf` を覆い、
   `lf` は `leaves(ty(y))` の元である。
2. `Some(Binding::Field(c, idx))` かつ `c.ty.is_box(type_env)` が偽のとき、`[idx] ++ p` は `ty(c)`
   において `[idx] ++ lf` を覆い、`[idx] ++ lf` は `leaves(ty(c))` の元である。
3. `Some(Binding::Payload(scrut, None))` のとき、`p` は `ty(scrut)` において `lf` を覆い、`lf` は
   `leaves(ty(scrut))` の元である。
4. `Some(Binding::Payload(scrut, Some(tag)))` かつ `scrut.ty.is_box(type_env)` が偽のとき、
   `[tag] ++ p` は `ty(scrut)` において `[tag] ++ lf` を覆い、`[tag] ++ lf` は
   `leaves(ty(scrut))` の元である。

**証明**

<1>1. 1 が成り立つ。A12 より move-bind の両辺の型は等しいので `ty(y) = ty(w)` であり、path も型も
      変わらない。
  BY A12, DEF 覆う
<1>2. 2 が成り立つ。
  <2>1. A12 より `Destructure` の容器は構造体であり、`held_field_type` の doc より、参照カウントが
        扱う path が名指すフィールドは容器が保持するフィールドである。すなわち `idx` は
        `unpunched_field_types(ty(c))` が返す添字であり、その型は `ty(w)` である。
    BY A12, CODE src/rc_ir/ownership.rs: held_field_type
  <2>2. `[idx] ++ lf` は `leaves(ty(c))` の元である。
    <3>1. `is_fully_unboxed(ty(c))` は偽である。真ならば `unpunched_field_types` の各フィールドの型も
          `is_fully_unboxed` であり、`ty(w)` がそうなると `boxed_leaf_paths(ty(w))` は空になって
          `lf` の存在に反する。
      BY <2>1, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
         CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <3>2. `ty(c).is_closure()` は偽であり、`ty(c).is_box(type_env)` はこの場合の仮定より偽であり、
          `ty(c).is_array()` は偽である。A12 より容器は構造体だからである。
      BY A12
    <3>3. QED
      <3>1 と <3>2 より `boxed_leaf_paths` の `go` は `ty(c)` で `unpunched_field_types` の繰り返しに
      入り、添字 `idx` について `ty(w)` へ降りる。よって `ty(w)` の leaf `lf` は `[idx] ++ lf` として
      積まれる。
      BY <2>1, <3>1, <3>2, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `unit_step(ty(c))` は `NoUnit` ではない。`unit_step` が `NoUnit` を返すのは
        `is_fully_unboxed` が真のときだけであり、それは <2>2 の中で否定した。
    BY <2>2, CODE src/rc_ir/ownership.rs: unit_step
  <2>4. CASE `unit_step(ty(c))` が `Fields` である。`truncate_to_unit(ty(c), [idx] ++ q)` は
        `idx` を積んで `ty(w)` へ降り、そこから `q` について同じ走査を続けるので、
        `[idx] ++ trunc(ty(w), q)` に等しい。これを `q := p` と `q := lf` に適用すると、
        DEF 覆う の仮定より両者は等しい。
    BY <2>1, DEF 覆う, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>5. CASE `unit_step(ty(c))` が `Unit` である。`truncate_to_unit` は最初の添字で `break` するので、
        `[idx] ++ p` と `[idx] ++ lf` のどちらも `[]` に切り詰まる。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>6. CASE `unit_step(ty(c))` が `Capture` である。`truncate_to_unit` は最初の添字が `capture_idx`
        に等しいことを表明してからそれを積んで `break` するので、どちらも `[idx]` に切り詰まる。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>7. QED
    <2>3 より場合は <2>4、<2>5、<2>6 で尽きる。どの場合も 2 つの切り詰めは等しい。前置の関係は
    先頭に同じ添字を足しても保たれる。leaf であることは <2>2 が与える。
    BY <2>2, <2>3, <2>4, <2>5, <2>6, DEF 覆う
<1>3. 3 が成り立つ。A12 より catch-all の payload と scrutinee の型は等しいので、path も型も変わらない。
  BY A12, DEF 覆う
<1>4. 4 が成り立つ。
  <2>1. A12 より `Match` の scrutinee は union であり、その第 `tag` 変位の型は `ty(w)` である。
    BY A12
  <2>2. `[tag] ++ lf` は `leaves(ty(scrut))` の元である。`is_box` はこの場合の仮定より偽、
        `is_closure` と `is_array` は union なので偽、`is_fully_unboxed` は <1>2 の <2>2 の <3>1 と
        同じ論法で偽である。よって `boxed_leaf_paths` の `go` は `unpunched_field_types(ty(scrut))` の
        繰り返しに入り、添字 `tag` について `ty(w)` へ降りる。
    BY <2>1, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `unit_step(ty(scrut))` は `Unit` である。`is_fully_unboxed` が偽 (<2>2)、`is_closure` が偽、
        `is_box` がこの場合の仮定より偽、`is_union` が真だからである。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: unit_step
  <2>4. QED
    <2>3 より `truncate_to_unit(ty(scrut), ・)` は最初の添字で `break` するので、`[tag] ++ p` と
    `[tag] ++ lf` のどちらも `[]` に切り詰まる。
    BY <2>2, <2>3, DEF 覆う, CODE src/rc_ir/ownership.rs: truncate_to_unit
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L6 (`Origin::Join` の候補は 2 元以上である)

**言明** — `Origin::Join { candidates, .. }` の `candidates` は 2 元以上を持ち、その `acted_on()` も
2 元以上を持つ。

**証明**

<1>1. `Origin::Join` を作るのは `Origin::of_candidates` の `_ =>` の腕だけであり、その腕は
      `candidates.len()` が 1 でないときに走る。`of_candidates` は先頭で `candidates` が空でない
      ことを表明するので、`candidates` の元数は 2 以上である。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates
<1>2. QED
  `acted_on` は `identity` を先頭に置き、`candidates` のうち `identity` と異なるものを足す。
  `identity` が `candidates` の元ならば `acted_on` の元数は `candidates` の元数に等しく、
  そうでなければそれより 1 多い。どちらでも 2 以上である。
  BY <1>1, CODE src/rc_ir/ownership.rs: Origin::acted_on

## 4. 反例 X

### 4.1 本体 X

型を 2 つ置く。

- `Pair` は unbox の構造体で、フィールドは `Array I64` が 2 つである。
- `Action` は unbox の union で、変位は `pair : Pair` (変位番号 0) と `mark : I64` (変位番号 1) で
  ある。

関数を 2 つ置く。

- `drain : Action -> I64 -> I64`。第 1 パラメータの unit `[]` を所有する (D14)。
- `peek2 : Action -> Action -> I64 -> I64`。第 1・第 2 パラメータの unit `[]` を借用する (D14)。

本体 `X` は、パラメータ `pp : Pair` と `other : Action` (どちらも所有される) を持つ関数の本体で、
次の形である。`n1`, `n2`, `n3` は `I64` の定数を束縛した変数、`s` は `RcState` の値である。

```
Let(action, Llvm(union_make_0, [pp]),
Retain(action, [], s,
Let(u1, App(drain, [action, n1]),
Let(u2, App(peek2, [action, other, n2]),
Release(action, [], s,
Let(u3, App(drain, [other, n3]),
Ret(u3)))))))
```

`union_make_0` は `InlineLLVMMakeUnionBody` の `variant_index()` が 0 である実体である
(`CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody`)。

`X` は A2 を満たす。L7 の 1 より `Retain` と `Release` の path `[]` は `units(Action)` の元だから
である。

### L7 (X における `origin`)

**言明** — `X` について次が成り立つ。

1. `leaves(Action) = {[0, 0], [0, 1]}`、`units(Action) = {[]}`、`leaves(Pair) = {[0], [1]}`、
   `units(Pair) = {[0], [1]}`。
2. `origin(action, [0, 0]) = Origin::Exactly((pp, [0]))` かつ
   `origin(action, [0, 1]) = Origin::Exactly((pp, [1]))`。
3. `origin(action, []) = Origin::Join { identity: (action, []), candidates: {(pp, [0]), (pp, [1])} }`。

**証明**

<1>1. 1 の前半が成り立つ。
  <2>1. `Action` は `is_fully_unboxed` が偽 (`Array I64` を保持する)、`is_closure` が偽、`is_box` が
        偽 (unbox の union)、`is_array` が偽である。よって `boxed_leaf_paths` の `go` は
        `unpunched_field_types(Action)` の繰り返しに入る。その要素は `(0, Pair)` と `(1, I64)` で
        ある。
    BY D4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>2. `Pair` は同じ 4 つの判定がすべて偽なので、`go` は `unpunched_field_types(Pair)` の繰り返しに
        入る。その要素は `(0, Array I64)` と `(1, Array I64)` である。`Array I64` は `is_array` が
        真なので、`go` はその位置を積んで返る。
    BY D4, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>3. `I64` は `is_fully_unboxed` が真なので `go` は何も積まずに返る。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>2. 1 の後半が成り立つ。
  <2>1. `unit_step(Action)` は `Unit` である。`is_fully_unboxed` が偽、`is_closure` が偽、
        `is_box` が偽、`is_union` が真だからである。よって `rc_units_go` は `[]` を積んで返り、
        `units(Action) = {[]}` である。
    BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>2. `unit_step(Pair)` は `Fields` であり、その `held_fields` は `(0, Array I64)` と
        `(1, Array I64)` である。`unit_step(Array I64)` は `is_array` が真なので `Unit` である。
        よって `units(Pair) = {[0], [1]}` である。
    BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>3. QED
    BY <2>1, <2>2
<1>3. `vars.bindings.get(action)` は `Some(Binding::Llvm(union_make_0, [pp], Action))` である。
      `collect_bindings` は `Let(x, RcRhs::Llvm(gen, args), k)` について
      `Binding::Llvm(gen.clone(), args.clone(), x.ty.clone())` を記録するからである。
  BY CODE src/rc_ir/ownership.rs: collect_bindings
<1>4. `decl := union_make_0.result_prov(Action, [Pair], type_env)` は、leaf `[0, 0]` に
      `{LeafOrigin::Arg(0, [0])}` を、leaf `[0, 1]` に `{LeafOrigin::Arg(0, [1])}` を与える。
  <2>1. `InlineLLVMMakeUnionBody::result_prov` は
        `Provenance::build_shape(result_ty, type_env, &|path| ...)` を返す。その閉包は
        `path.split_first()` で分岐し、`None` のとき `sole_origin(Fresh)`、`Some((k, rest))` で
        `k == variant_index()` のとき `sole_origin(Arg(0, rest.to_vec()))`、それ以外のとき
        `Set::default()` を返す。
    BY CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody
  <2>2. `Provenance::build_shape` は `LeafMap::build_shape` を呼び、それは
        `boxed_leaf_paths(ty, type_env)` の各 path について閉包を 1 回呼んでその値を記録する。
    BY CODE src/rc_ir/provenance.rs: Provenance::build_shape,
       CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape
  <2>3. QED
    <1>1 より `Action` の leaf は `[0, 0]` と `[0, 1]` である。どちらも `split_first` の第 1 成分が
    0 であり、`variant_index()` は 0 なので、閉包は `Arg(0, [0])` と `Arg(0, [1])` を返す。
    BY <1>1, <2>1, <2>2
<1>5. 2 が成り立つ。
  <2>1. `origin_inner(action, [0, 0])` は `Binding::Llvm` の腕に入り、
        `decl.leaf_origins_at([0, 0]).and_then(as_arg_projection)` を計算する。
        `leaf_origins_at` は `LeafMap::get` であり、<1>4 より `Some({Arg(0, [0])})` を返す。
        `as_arg_projection` は 1 元集合の `Arg(j, q)` について `Some((j, q))` を返すので、値は
        `Some((0, [0]))` である。
    BY <1>3, <1>4, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>2. この腕は `origin(vars, type_env, &args[0].name, &[0])` すなわち `origin(pp, [0])` を返す。
        `pp` はパラメータなので `vars.bindings.get(pp)` は `Some(Binding::Param)` であり、
        `origin_inner` のその腕は `here()` すなわち `Origin::Exactly((pp, [0]))` を返す。
    BY <2>1, CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: VarTable::of
  <2>3. QED
    `[0, 1]` についても <2>1 と <2>2 と同じ計算が `Origin::Exactly((pp, [1]))` を与える。
    <1>4 が `[0, 1]` の記録を `{Arg(0, [1])}` としているからである。
    BY <1>4, <2>1, <2>2
<1>6. 3 が成り立つ。
  <2>1. `origin_inner(action, [])` は `Binding::Llvm` の腕に入り、`decl.leaf_origins_at([])` を
        計算する。<1>1 より `[]` は `Action` の boxed leaf ではないので、`LeafMap::get` は `None` を
        返し、`and_then` の値も `None` である。よってこの腕は
        `origin_from_leaves_under(vars, type_env, &decl, [pp], [], &(action, []))` を計算する。
    BY <1>1, <1>3, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at
  <2>2. `decl.leaf_origins_under([])` は `[]` を前置に持つ leaf の記録、すなわち `{Arg(0, [0])}` と
        `{Arg(0, [1])}` を渡す。よって `operand_units` は
        `{(0, trunc(Pair, [0])), (0, trunc(Pair, [1]))}` であり、`produced_here` は偽である。
    BY <1>4, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>3. `trunc(Pair, [0]) = [0]` かつ `trunc(Pair, [1]) = [1]` である。<1>2 より `[0]` と `[1]` は
        `units(Pair)` の元なので、L2 の後半が与える。
    BY <1>2, L2
  <2>4. `reached = [origin(pp, [0]), origin(pp, [1])]` であり、その 2 元は
        `Origin::Exactly((pp, [0]))` と `Origin::Exactly((pp, [1]))` である。`pp` はパラメータなので
        `origin_inner` の `Param` の腕が `here()` を返すからである。
    BY <2>2, <2>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: origin_inner
  <2>5. `reached` の 2 元は等しくないので、`origin_from_leaves_under` は
        `Origin::of_candidates(candidates, here)` を返す。ここで `candidates` は各元の `acted_on()` を
        集めたもの、すなわち `{(pp, [0]), (pp, [1])}` であり、`here = (action, [])` である。
    BY <2>4, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: Origin::acted_on
  <2>6. QED
    `candidates` は 2 元なので `of_candidates` は
    `Origin::Join { identity: (action, []), candidates }` を返す。
    BY <2>5, CODE src/rc_ir/ownership.rs: Origin::of_candidates
<1>7. QED
  BY <1>1, <1>2, <1>5, <1>6

### L8 (X における鍵)

**言明** — `X` について次が成り立つ。

1. `key(action, []) = (action, [])`。
2. `unit_of(id(action, [0, 0])) = (pp, [0])` かつ `unit_of(id(action, [0, 1])) = (pp, [1])`。
3. `acted_references(action, []) = { (pp, [0]) : 1, (pp, [1]) : 1 }`。
4. `acted_unit_keys(action, [])` の元は `(action, [])`、`(pp, [0])`、`(pp, [1])` の 3 つである。
5. `acted_unit_keys(action, [0, 0])` は `(pp, [0])` 1 元、`acted_unit_keys(action, [0, 1])` は
   `(pp, [1])` 1 元である。
6. 実行時に `[0, 0]` と `[0, 1]` は `action` において inhabited (D16) である。

**証明**

<1>1. 1 が成り立つ。L7 の 3 より `id(action, []) = (action, [])` である。`action` は `Let` が束縛する
      変数なので `vars.var_tys` に型 `Action` が記録されており、L4 より
      `unit_of((action, [])) = (action, trunc(Action, []))` である。L7 の 1 より `[]` は
      `units(Action)` の元なので、L2 の後半より `trunc(Action, []) = []` である。
  BY L2, L4, L7, CODE src/rc_ir/ownership.rs: collect_bindings
<1>2. 2 が成り立つ。L7 の 2 より `id(action, [0, 0]) = (pp, [0])` である。`pp` はパラメータなので
      `vars.var_tys` に型 `Pair` が記録されており、L4 と L2 の後半より
      `unit_of((pp, [0])) = (pp, trunc(Pair, [0])) = (pp, [0])` である。`[0, 1]` も同じである。
  BY L2, L4, L7, CODE src/rc_ir/ownership.rs: VarTable::of
<1>3. 3 が成り立つ。`acted_references(v, path)` は `boxed_leaf_paths(ty(v))` のうち `path` を前置に
      持つ leaf ごとに `origin(v, leaf).identity()` を鍵として 1 を足す。L7 の 1 より該当する leaf は
      `[0, 0]` と `[0, 1]` の 2 つ、L7 の 2 よりその identity は `(pp, [0])` と `(pp, [1])` である。
  BY L7, CODE src/rc_ir/ownership.rs: acted_references
<1>4. 4 が成り立つ。`acted_unit_keys(v, path)` は `origin(v, path).acted_on()` の各元を `unit_of` で
      写す。L7 の 3 より `acted_on()` は `(action, [])` を先頭に `(pp, [0])`、`(pp, [1])` を並べた
      ものであり、<1>1 と <1>2 よりそれぞれの `unit_of` はそれ自身である。
  BY L7, <1>1, <1>2, CODE src/rc_ir/ownership.rs: acted_unit_keys,
     CODE src/rc_ir/ownership.rs: Origin::acted_on
<1>5. 5 が成り立つ。L7 の 2 より `origin(action, [0, 0])` は `Exactly((pp, [0]))` であり、その
      `acted_on()` は `(pp, [0])` 1 元である。<1>2 よりその `unit_of` は `(pp, [0])` である。
      `[0, 1]` も同じである。
  BY L7, <1>2, CODE src/rc_ir/ownership.rs: acted_unit_keys,
     CODE src/rc_ir/ownership.rs: Origin::acted_on
<1>6. 6 が成り立つ。`union_make_0` は変位番号 0 を構築する op であり、A3 の「単一の `Arg(j, s)`」と
      「空集合」の行より、生成コードは変位 0 の leaf にだけ値を置く。A4 より unbox union に対する
      参照カウントは実行時のタグで分岐し、`Action` のタグはその値の変位番号である。`[0, 0]` と
      `[0, 1]` が通る union の節で選ぶ変位番号は 0 なので、D16 の判定は真である。
  BY A3, A4, D16, CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody
<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### L9 (X の走査)

**言明** — `cancel_body` を `X` に走らせると、`analysis.cancelled()` は `X` の `Retain` 節点 `t` と
`Release` 節点 `r` をともに含む。さらに、`Let(u1, App(drain, [action, n1]), k)` の訪問は
`key(t)` の下で `consume_unit` を呼ばない。

**証明**

<1>1. `Retain(action, [], s, k)` の訪問は、`pending` の鍵 `key(t)` の `Vec` に
      `PendingRetain { node: node_id(t), outstanding: bumped(t) }` を積む。L8 の 1 と 3 より
      `key(t) = (action, [])` かつ `bumped(t) = {(pp, [0]) : 1, (pp, [1]) : 1}` である。
  BY L8, DEF 節点の量,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
<1>2. `Let(u1, App(drain, [action, n1]), k)` の訪問は `key(t)` の下で `consume_unit` を呼ばず、
      よって `t` を `pending` から取り除かず、`needed_retains` にも入れない。
  <2>1. この訪問は `consume_rhs(&mut pending, rhs, &x.ty)` を呼び、それは `rhs_consumes` が
        `consumed` に積んだ各 `(var, leaf)` について `consume(pending, &var, &leaf)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>2. `rhs_consumes` の `RcRhs::App(callee, args)` の腕が積むのは、callee の全 boxed leaf と、
        `owns(&params[i], &leaf)` が真である引数位置の leaf である。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes
  <2>3. `drain` は第 1 パラメータの unit `[]` を所有するので、`consume_rhs` の `owns` 閉包は
        `action` の leaf `[0, 0]` と `[0, 1]` について真を返す。第 2 引数 `n1` は `I64` で boxed
        leaf を持たない。
    BY <2>2, D14, L7, L2, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>4. `consume(pending, action, [0, 0])` は `acted_unit_keys(action, [0, 0])` の各元について
        `consume_unit` を呼ぶ。L8 の 5 よりその元は `(pp, [0])` ただ 1 つである。
        `consume_unit(pending, (pp, [0]))` は `pending.remove(&(pp, [0]))` を試みる。<1>1 より
        `pending` の鍵は `(action, [])` だけなので、この呼び出しは何も取り除かない。
        `(action, [0, 1])` についても同じで、鍵は `(pp, [1])` である。
    BY <1>1, L8, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit
  <2>5. callee の leaf について `consume` が渡す鍵の第 1 成分は callee の名前である。L4 より
        `unit_of` は第 1 成分を変えないからである。callee の名前は `action` ではないので、この
        呼び出しも `(action, [])` の項目を取り除かない。
    BY L4, <2>2, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume
  <2>6. QED
    <2>3、<2>4、<2>5 より、この訪問が `consume_unit` に渡す鍵は `(pp, [0])`、`(pp, [1])`、および
    callee の名前を第 1 成分に持つ鍵だけであり、どれも `key(t) = (action, [])` ではない。
    `needed_retains` に要素を入れるのは `consume_unit` が取り除いた `Vec` の要素だけである。
    BY <2>1, <2>3, <2>4, <2>5, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit
<1>3. `Let(u2, App(peek2, [action, other, n2]), k)` の訪問は `t` を `pending` から取り除かず、
      `needed_retains` にも入れない。`peek2` は第 1・第 2 パラメータの unit `[]` を借用するので、
      `consume_rhs` の `owns` 閉包は `action` と `other` のどの leaf についても偽を返し、
      `rhs_consumes` は callee の leaf しか積まない。callee の leaf については <1>2 の <2>5 と
      同じである。
  BY D14, L4, <1>2, CODE src/rc_ir/ownership.rs: rhs_consumes,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
<1>4. `Release(action, [], s, k)` の訪問は `un_bump_releases[node_id(t)]` に `node_id(r)` を積み、
      `pending` から `t` の項目を取り除く。
  <2>1. この訪問はまず `others(r)` の各元について `consume_unit` を呼ぶ。L8 の 1 と 4 より
        `others(r)` は `(pp, [0])` と `(pp, [1])` の 2 元であり、<1>1 よりどちらも `pending` の鍵で
        ないので何も取り除かない。
    BY <1>1, L8, DEF 節点の量,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕
  <2>2. `un_bumped(r) = acted_references(action, []) = {(pp, [0]) : 1, (pp, [1]) : 1}` であり、
        <1>1 の `outstanding` に等しい。
    BY L8, DEF 節点の量
  <2>3. `un_bump(&mut pending, &(action, []), &un_bumped(r))` は、鍵 `(action, [])` の `Vec` の
        最後の要素 (<1>1 が積んだ要素である) の `outstanding` が `un_bumped(r)` を `covers` するので、
        それを引いて空にし、要素を取り除き、`UnBump::InBracket(node_id(t))` を返す。
    BY <1>1, <2>2, P17, CODE src/rc_ir/borrow.rs: un_bump
  <2>4. QED
    `InBracket(retain)` の枝は `self.un_bump_releases.entry(retain).or_default().push(node_id(node))`
    を実行する。
    BY <2>1, <2>3,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕
<1>5. `Let(u3, App(drain, [other, n3]), k)` の訪問と `Ret(u3)` の訪問は `needed_retains` に
      `node_id(t)` を入れない。<1>4 より `t` はすでに `pending` から取り除かれており、
      `consume_unit` も `Ret` の腕も `pending` に在る要素の `node` しか `needed_retains` に入れない。
  BY <1>4, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕
<1>6. QED
  `cancelled()` は `all_retains` の各要素のうち `needed_retains` に入らず `un_bump_releases` の値が
  空でないものについて、その要素と値の全要素を返す。<1>1 より `node_id(t)` は `all_retains` の元、
  <1>2、<1>3、<1>5 より `needed_retains` の元ではなく、<1>4 より
  `un_bump_releases[node_id(t)]` は `node_id(r)` を持つ。後半の主張は <1>2 である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled

### L10 (X の実行)

**言明** — `X` は D11 を満たすが、`cancel` の出力 `X'` は (S-c) を破る。

**証明** 活性化を 1 つ固定する。`pp` の leaf `[0]` が指すオブジェクトを `A`、`[1]` が指すものを `B`
とする。`other` の leaf が指す 2 つのオブジェクトは `A` とも `B` とも異なるものとする。

<1>1. `X` の入口で `Obl(A) = Obl(B) = 1` である。A1 よりすべてのパラメータの unit が所有されるので、
      D10 の初期値は所有するパラメータの inhabited な各 leaf につき参照 1 つである。
  BY A1, D10, D14
<1>2. `Let(action, Llvm(union_make_0, [pp]), ・)` は `Obl` を変えない。L7 の 4 より
      `union_make_0` の宣言は結果の leaf `[0, 0]` に単一の `Arg(0, [0])` を、`[0, 1]` に単一の
      `Arg(0, [1])` を与えるので、`pp` の 2 つの leaf は D9 の移動の表の「`Llvm` の素通し leaf」の
      行に当たる。
  BY D9, D10, L7
<1>3. `Retain(action, [])` の後、`Obl(A) = Obl(B) = 2` であり、`H(A)` と `H(B)` はそれぞれ 1 増える。
      L8 の 6 より `[0, 0]` と `[0, 1]` はどちらも inhabited であり、L8 の 3 よりそれぞれ `A` と
      `B` を指す。
  BY D10, L8, <1>1
<1>4. `App(drain, [action, n1])` は `Obl(A)` と `Obl(B)` をそれぞれ 1 減らす。`drain` がその位置の
      unit を所有するからである。呼び出しが返った後、`H(A)` と `H(B)` はそれぞれ `Retain` の前の値に
      戻る。呼び出し先が受け取った参照を処分するからである。
  BY D9, D10, D14, <1>3
<1>5. `Let(u2, App(peek2, [action, other, n2]), ・)` は `action` の inhabited な各 leaf が指す
      オブジェクトを読みうる。D7 の読む構文の表の `App` の行がそれを述べる。
  BY D7
<1>6. `X` は D11 を満たす。
  <2>1. (S-a) が成り立つ。`Obl` から参照を取り除く操作は、`App(drain, [action, n1])` の消費、
        `Release(action, [])`、`App(drain, [other, n3])` の消費、および終端の `Ret(u3)` の消費で
        ある。<1>3 の後 `Obl(A) = Obl(B) = 2` であり、`drain` の消費で 1 に、`Release` で 0 に
        なる。`other` の 2 つのオブジェクトは <1>1 より `Obl` に 1 つずつあり、`drain` の消費で
        0 になる。`Ret(u3)` は `u3 : I64` なので boxed leaf を持たず、何も取り除かない。どの
        取り除きの時点でも、取り除かれる参照は `Obl` に入っている。
    BY D9, D10, <1>1, <1>3, <1>4
  <2>2. (S-b) が成り立つ。<2>1 の数え上げより `Ret(u3)` の消費の後 `Obl` は空である。
    BY <2>1
  <2>3. (S-c) が成り立つ。D7 の読む構文は `X` に 4 つある。`Llvm(union_make_0, [pp])` は `pp` を
        読み、その位置で `H(A)` と `H(B)` は 1 以上である (<1>1 と D10 より、`Obl` に参照がある
        オブジェクトの `H` は 1 以上である)。`App(drain, [action, n1])` は `action` と callee を
        読み、その位置で `H(A) = H(B) = 2` である (<1>3)。`App(peek2, [action, other, n2])` は
        `action` と `other` と callee を読み、その位置で `H(A)` と `H(B)` は <1>4 より入口の値、
        すなわち 1 以上であり、`other` の 2 つのオブジェクトはまだ何も処分していないので 1 以上で
        ある。`App(drain, [other, n3])` は `other` と callee を読み、`other` の 2 つのオブジェクトは
        その位置でまだ処分されていない。
    BY D7, D8, D10, <1>1, <1>3, <1>4
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>7. `X'` は (S-c) を破る。
  <2>1. L9 と P22 より `X'` は `X` から `Retain(action, [])` と `Release(action, [])` を抜いた木で
        ある。
    BY L9, P22
  <2>2. `X'` において、`App(drain, [action, n1])` の消費の後 `Obl(A) = 0` であり、呼び出し先が
        その参照を処分した時点で `H(A) = 0` になる。D7 より `H(A) = 0` のオブジェクトは解放される。
    BY <1>1, <1>2, <1>4, <2>1, D7, D10
  <2>3. QED
    <1>5 より `App(peek2, [action, other, n2])` は `A` を読みうる。<2>2 よりその位置で `A` は
    解放されている。
    BY <1>5, <2>2
<1>8. QED
  BY <1>6, <1>7

## 5. 反例 X の Fix での再現

`X` の形は Fix のプログラムから出る。次のプログラムを `-O max` で走らせると、`drain` が処分した
配列を `peek2` が読む。

```fix
module Main;

// An unboxed union whose `pair` payload holds the references of two distinct objects.
type Action = unbox union { pair : (Array I64, Array I64), mark : I64 };

// Reads both arrays the payload holds and disposes of no reference: the destructure names both
// fields (a move) and element access borrows.
read_both : Action -> I64;
read_both = |x| match x {
    pair(p) => (let (a, b) = p; a.@(0) * 10 + b.@(0)),
    mark(m) => m
};

// Borrows both unions. The recursion keeps the call out of line.
peek2 : Action -> Action -> I64 -> I64;
peek2 = |a, b, n| (
    if n == 0 { read_both(a) + read_both(b) };
    peek2(a, b, n - 1)
);

// Consumes the payload: both arrays go into a fresh array, which is then dropped.
consume_pair : (Array I64, Array I64) -> I64;
consume_pair = |p| (
    let (x, y) = p;
    let z = [x, y];
    z.@size
);

// Owns its union: the payload it takes out goes to an owning position.
drain : Action -> I64 -> I64;
drain = |a, n| (
    if n == 0 {
        match a {
            pair(p) => consume_pair(p),
            mark(m) => m
        }
    };
    drain(a, n - 1)
);

run : I64 -> (Array I64, Array I64) -> Action -> I64;
run = |k, payload, other| (
    let action = Action::pair(payload);
    let u1 = drain(action, 1);
    // Two arrays of the sizes the payload had, allocated after `drain` disposed of it.
    let f1 = Array::fill(k + 2, 111);
    let f2 = Array::fill(k + 3, 222);
    let u2 = peek2(action, other, 2);
    let u3 = drain(other, 1);
    u1 * 10000 + u2 * 10 + u3 + (f1.@(0) - 111) + (f2.@(0) - 222)
);

main : IO ();
main = (
    // The sizes come from the command line, so the arrays are not constant-folded away.
    let args = *get_args;
    let k = args.@size;
    let payload = (Array::fill(k + 2, 7), Array::fill(k + 3, 9));
    let other = Action::pair((Array::fill(k + 4, 1), Array::fill(k + 5, 2)));
    let r = run(k, payload, other);
    println(r.to_string)
);
```

計測 (`1ee99ba7`、x86-64 Linux、引数なしで実行)。

| 最適化 | 出力 | valgrind memcheck |
|---|---|---|
| `-O none` | `20912` | エラー 0 件 |
| `-O basic` | `20912` | エラー 0 件 |
| `-O max` | `43432` | `Invalid read of size 8` を 2 件 |

`-O max` の 2 件はどちらも、`drain` の中で `free` されたブロックへの読みである。`f1` と `f2` の
割り当てを外すと出力は `20912` に戻ることがあるが、valgrind の 2 件は残る。解放されたブロックが
再利用されるかどうかで出力が変わるだけである。

`--emit-rc-ir all` が出す `rc_ir.pre.txt` (`insert_rc` の直後) の `main` には

```
let action = union_make_0(struct)
retain action
let u1 = drain(action, 1)
...
let u2 = peek2(action, other, 2)
```

があり、`rc_ir.post.txt` (RC IR の最適化の後) の同じ場所には `retain` が無く、`peek2` の借用版が
呼ばれ、その後に `release` も無い。すなわち `borrow_ify` が `peek2` を借用版へ振り分けて呼び出しの
後に `Release(action, [])` を置き、`cancel` がその `Release` と `retain` の対を、間にある
`drain(action, 1)` の消費をまたいで消している。これは L9 と L10 が述べたとおりである。

`develop_mode` の `check_one_key_per_object` はこれを捕まえない。その表明は、`key` と異なる鍵の下に
在る pending な `Retain` の `outstanding` が `un_bumped` に等しいことを禁じるが、`X` では
`pending` の鍵は `key(r)` ただ 1 つであり、繰り返しの本体が `continue` で飛ばされるからである
(`CODE src/rc_ir/borrow.rs: check_one_key_per_object`)。

## 6. P7b

**言明 (README)** — `pi` が `ty(v)` の RC unit であり、`lf` が `pi` の下の inhabited な boxed leaf で
あるとき、`unit_of(origin(v, lf).identity()) = unit_key(v, pi)` である。

### 6.1 P7b は偽である

**証明**

<1>1. `X` の `action` について、`[]` は `units(Action)` の元であり、`[0, 0]` は `[]` を前置に持つ
      `leaves(Action)` の元であり、実行時に inhabited である。
  BY L7, L8
<1>2. `unit_of(id(action, [0, 0])) = (pp, [0])` かつ `key(action, []) = (action, [])` である。
  BY L8
<1>3. QED
  `pp` と `action` は相異なる変数なので `(pp, [0])` と `(action, [])` は異なる。よって P7b の等式は
  `v := action`、`pi := []`、`lf := [0, 0]` で偽である。
  BY <1>1, <1>2

### L11 (P7b の制限つきの形)

**言明** — 1 つの活性化の 1 つの実行路の 1 つの位置を固定する。変数 `w`、`lf` を
`leaves(ty(w))` の元で `w` において inhabited なもの、`p` を `ty(w)` において `lf` を覆う path と
する。3 つ組 `(w, p, lf)` から始まる `origin` の再帰のすべての段が良い (DEF 良い段) ならば、
`unit_of(id(w, p)) = unit_of(id(w, lf))` である。

P7b は L11 の系である。`pi` が `units(ty(v))` の元、`lf` が `leaves(ty(v))` の元、`pi <= lf` の
とき、L2 より `trunc(ty(v), lf) = pi` であり、L2 の後半より `trunc(ty(v), pi) = pi` なので、
`pi` は `lf` を覆う。

**証明** `origin(w, p)` の再帰についての帰納法で示す。P2 より `origin` は停止するので、この再帰の
木は有限であり、帰納法は整礎である。

<1>1. 帰納法の仮定: `origin(w, p)` の再帰の中で呼ばれる各 `origin(w', p')` について、
      3 つ組 `(w', p', lf')` が言明の仮定を満たすならば、`unit_of(id(w', p')) = unit_of(id(w', lf'))`
      である。
  BY 帰納法の仮定
<1>2. CASE `p = lf`。両辺は同じ式なので等しい。
  BY DEF 覆う
<1>3. 以下 `p < lf` とする。このとき `p` は `leaves(ty(w))` の元ではない。
  BY L1
<1>4. CASE 段が (G1) である。この腕は `here()` すなわち `Origin::Exactly((w, ・))` を返すので、
      `id(w, p) = (w, p)` かつ `id(w, lf) = (w, lf)` である。(G1) の但し書きより `w` は
      `vars.var_tys` に記録されているので、L4 より
      `unit_of((w, p)) = (w, trunc(ty(w), p))` かつ `unit_of((w, lf)) = (w, trunc(ty(w), lf))` で
      あり、DEF 覆う よりこの 2 つは等しい。
  BY L4, DEF 覆う, DEF 良い段, CODE src/rc_ir/ownership.rs: origin_inner
<1>5. CASE 段が (G2) である。
  <2>1. `Some(Binding::Move(y))` のとき、この腕は `origin(y, p)` を返し、leaf の側は
        `origin(y, lf)` を返す。L5 の 1 より `p` は `ty(y)` において `lf` を覆う。D9 の移動の表の
        「`Let(x, Var(y), k)`」の行より `w` の値は `y` の値なので、D16 が読む union のタグは等しく、
        `lf` は `y` においても inhabited である。
    BY L5, D9, D16, CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `is_box` が偽の `Some(Binding::Field(c, idx))` のとき、この腕は `origin(c, [idx] ++ p)` を
        返し、leaf の側は `origin(c, [idx] ++ lf)` を返す。L5 の 2 より `[idx] ++ p` は `ty(c)` に
        おいて `[idx] ++ lf` を覆う。D9 の移動の表の
        「`Destructure(c, fs)` (`c` が unbox) の名前付きフィールド」の行より `w` の値は `c` の値の
        第 `idx` フィールドであり、D16 の判定が読む節は同じ値の同じ節なので、`[idx] ++ lf` は
        `c` において inhabited である。
    BY L5, D9, D16, CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `Some(Binding::Payload(scrut, None))` のとき、この腕は `origin(scrut, p)` を返し、leaf の
        側は `origin(scrut, lf)` を返す。L5 の 3 より `p` は `ty(scrut)` において `lf` を覆う。
        D9 の移動の表の「catch-all アームの payload 束縛」の行より payload 変数の値は scrutinee の
        値なので、`lf` は `scrut` においても inhabited である。
    BY L5, D9, D16, CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. `is_box` が偽の `Some(Binding::Payload(scrut, Some(tag)))` のとき、この腕は
        `origin(scrut, [tag] ++ p)` を返し、leaf の側は `origin(scrut, [tag] ++ lf)` を返す。
        L5 の 4 より `[tag] ++ p` は `ty(scrut)` において `[tag] ++ lf` を覆う。A11 より payload
        変数の使用はその変位アームの本体の中にあり、A4 より `Match` は実行時のタグで分岐するので、
        この実行路の上で `scrut` のタグは `tag` である。D9 の移動の表の
        「unbox union の変位アームの payload 束縛」の行より `w` の値は `scrut` の値の第 `tag` 変位の
        payload なので、D16 より `[tag] ++ lf` は `scrut` において inhabited である。
    BY L5, D9, D16, A4, A11, CODE src/rc_ir/ownership.rs: origin_inner
  <2>5. QED
    <2>1 から <2>4 のいずれでも、この腕は `origin(w', p')` の値をそのまま返し、leaf の側も
    `origin(w', lf')` の値をそのまま返す。3 つ組 `(w', p', lf')` は言明の仮定を満たすので
    <1>1 が使える。
    BY <1>1, <2>1, <2>2, <2>3, <2>4
<1>6. CASE 段が (G3) である。`Some(Binding::Join(arm_results))` であり、この腕は各 `arm_result` に
      ついて `origin(arm_result, ・).acted_on()` を集めて
      `Origin::of_candidates(candidates, (w, ・))` を返す。
  <2>1. CASE `origin(w, p)` と `origin(w, lf)` がともに `Origin::Join` である。`of_candidates` は
        `identity` に第 2 引数を据えるので `id(w, p) = (w, p)` かつ `id(w, lf) = (w, lf)` である。
        `w` は `Let(w, Match(..), k)` が束縛する変数なので `vars.var_tys` に記録されており、
        L4 と DEF 覆う より 2 つの `unit_of` は等しい。
    BY L4, DEF 覆う, CODE src/rc_ir/ownership.rs: Origin::of_candidates,
       CODE src/rc_ir/ownership.rs: collect_bindings
  <2>2. CASE `origin(w, p)` と `origin(w, lf)` がともに `Origin::Exactly` である。
    <3>1. `origin(w, lf) = Origin::Exactly((u, s))` とすると、各 `arm_result` について
          `origin(arm_result, lf)` は `Origin::Exactly((u, s))` である。
      <4>1. `of_candidates` が `Exactly` を返すのは `candidates` が 1 元のときなので、各
            `arm_result` の `origin(arm_result, lf).acted_on()` の元はすべて `(u, s)` である。
        BY CODE src/rc_ir/ownership.rs: Origin::of_candidates,
           CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. QED
        L6 より `Origin::Join` の `acted_on()` は 2 元以上を持つ。よって <4>1 より各
        `origin(arm_result, lf)` は `Origin::Exactly` であり、その `acted_on()` は identity 1 元
        なので、identity は `(u, s)` である。
        BY L6, <4>1
    <3>2. 同じ理由で、`origin(w, p) = Origin::Exactly((u2, s2))` とすると、各 `arm_result` に
          ついて `origin(arm_result, p)` は `Origin::Exactly((u2, s2))` である。
      <4>1. `of_candidates` が `Exactly` を返すのは `candidates` が 1 元のときなので、各
            `arm_result` の `origin(arm_result, p).acted_on()` の元はすべて `(u2, s2)` である。
        BY CODE src/rc_ir/ownership.rs: Origin::of_candidates,
           CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. QED
        L6 より `Origin::Join` の `acted_on()` は 2 元以上を持つので、各 `origin(arm_result, p)` は
        `Origin::Exactly((u2, s2))` である。
        BY L6, <4>1
    <3>3. この実行路が選んだアームの結果変数を `x_i` とする。A12 より `ty(x_i) = ty(w)` なので
          `lf` は `leaves(ty(x_i))` の元であり、`p` は `ty(x_i)` において `lf` を覆う。D9 の移動の
          表の「`Match` のアーム本体の `Ret(x)`」の行より `w` の値は `x_i` の値なので、D16 が読む
          タグは等しく、`lf` は `x_i` において inhabited である。
      BY A12, D9, D16, DEF 覆う
    <3>4. QED
      <3>1 より `id(w, lf) = id(x_i, lf)`、<3>2 より `id(w, p) = id(x_i, p)` である。<3>3 より
      <1>1 が 3 つ組 `(x_i, p, lf)` に使えて `unit_of(id(x_i, p)) = unit_of(id(x_i, lf))` を与える。
      BY <1>1, <3>1, <3>2, <3>3
  <2>3. QED
    (G3) はこの 2 つの場合だけを良いとする。
    BY <2>1, <2>2, DEF 良い段
<1>7. CASE 段が (G4) である。<1>3 より `p < lf` なので (G4) の条件 `p = lf` は成り立たず、この
      場合は起こらない。
  BY <1>3, DEF 良い段
<1>8. 以下 `Some(Binding::Llvm(llvm_gen, args, result_ty))` の場合を扱う。<1>3 より
      `decl.leaf_origins_at(p)` は `None` であり、`origin_inner` のこの腕は
      `origin_from_leaves_under(vars, type_env, &decl, args, p, &(w, p)).unwrap_or_else(here)` を
      計算する。
  BY <1>3, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
     CODE src/rc_ir/ownership.rs: origin_inner
<1>9. `decl.leaf_origins_at(lf)` は空集合ではない。空集合ならば A3 の「空集合」の行より生成コードは
      その leaf に何も置かず、その leaf は inhabited にならないが、`lf` は inhabited である。
  BY A3, D16
<1>10. CASE 段が (G5) である。
  <2>1. `decl.leaf_origins_at(lf)` が単一の `Fresh` または単一の `Unknown` なので、
        `as_arg_projection` は `None` を返し、`origin_inner` の leaf の側も
        `origin_from_leaves_under` を計算する。その `path` 引数は `lf` であり、
        `leaf_origins_under(lf)` は `lf` 自身の記録だけを渡す (L1 より `lf` を真の前置に持つ leaf は
        無い)。よって `operand_units` は空、`produced_here` は真、
        `reached = [Origin::Exactly((w, lf))]` であり、返り値は `Origin::Exactly((w, lf))` である。
    BY L1, CODE src/rc_ir/ownership.rs: as_arg_projection,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
  <2>2. `p` の側の `origin_from_leaves_under` (<1>8) では `produced_here` が真である。`lf` は `p` を
        前置に持つ leaf であり、その記録が `Fresh` か `Unknown` を持つからである。
    BY <1>8, <2>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>3. `id(w, p) = (w, p)` である。
    <3>1. <2>2 より `reached` は `Origin::Exactly((w, p))` を元に持つので空でない。
      BY <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>2. CASE `reached.iter().all(|r| r == first)` が真。<3>1 より `first` は
          `Origin::Exactly((w, p))` であり、返り値もそれである。
      BY <3>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>3. CASE それが偽。返り値は `Origin::of_candidates(candidates, &(w, p))` である。これが
          `Origin::Exactly` を返すのは `candidates` が 1 元のときであり、そのとき `reached` の各元の
          `acted_on()` の元はその 1 元だけになるので、L6 より各元は `Origin::Exactly` であって
          互いに等しく、`all` が真であったことになって場合の仮定に反する。よって返り値は
          `Origin::Join` であり、その identity は `(w, p)` である。
      BY L6, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates
    <3>4. QED
      BY <3>2, <3>3
  <2>4. QED
    <2>1 より `id(w, lf) = (w, lf)`、<2>3 より `id(w, p) = (w, p)` である。`w` は `Let` が束縛する
    変数なので `vars.var_tys` に記録されており、L4 と DEF 覆う より 2 つの `unit_of` は等しい。
    BY <2>1, <2>3, L4, DEF 覆う, CODE src/rc_ir/ownership.rs: collect_bindings
<1>11. CASE 段が (G6) である。`decl.leaf_origins_at(lf) = {LeafOrigin::Arg(j, s)}` とし、
       `U := trunc(ty(args[j]), s)` とする。
  <2>1. leaf の側は `origin(args[j], s)` を返す。`as_arg_projection` が `Some((j, s))` を返すからで
        ある。
    BY CODE src/rc_ir/ownership.rs: as_arg_projection, CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `s` は `leaves(ty(args[j]))` の元である。A3 は `Arg(j, s)` を「第 `j` オペランドの leaf `s`」
        と述べており、`Provenance::compose` も宣言された `Arg(j, s)` がオペランド `j` の boxed leaf を
        名指すことを前提にしている。
    BY A3, CODE src/rc_ir/provenance.rs: Provenance::compose
  <2>3. `s` は `args[j]` において inhabited である。A3 の「単一の `Arg(j, s)`」の行は、結果のその
        leaf が inhabited であることと第 `j` オペランドの leaf `s` が inhabited であることが同値で
        あると述べる。
    BY A3
  <2>4. `U` は `ty(args[j])` において `s` を覆う。L3 より `U <= s` かつ
        `trunc(ty(args[j]), U) = U` であり、`U = trunc(ty(args[j]), s)` だからである。
    BY L3, DEF 覆う
  <2>5. `origin(args[j], U)` は、<1>8 の `origin_from_leaves_under` が作る `reached` の元である。
        `origin_from_leaves_under` は `path = p` の下の各 leaf の各 source について
        `LeafOrigin::Arg(j', leaf)` を `(j', trunc(ty(args[j']), leaf))` として `operand_units` に
        入れ、`reached` をその各元についての `origin(args[j'], unit)` として作る。`lf` は `p` を
        前置に持つ leaf であり、その source は `Arg(j, s)` である。
    BY <1>8, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>6. (G6) より `reached` の元はすべて等しいので、`origin_from_leaves_under` は `reached` の先頭を
        そのまま返す。<2>5 よりそれは `origin(args[j], U)` に等しい。よって
        `id(w, p) = id(args[j], U)` である。
    BY <2>5, DEF 良い段, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>7. QED
    <2>2、<2>3、<2>4 より <1>1 が 3 つ組 `(args[j], U, s)` に使えて
    `unit_of(id(args[j], U)) = unit_of(id(args[j], s))` を与える。<2>1 より右辺は
    `unit_of(id(w, lf))` であり、<2>6 より左辺は `unit_of(id(w, p))` である。
    BY <1>1, <2>1, <2>2, <2>3, <2>4, <2>6
<1>12. QED
  `origin_inner` の `match` は `vars.bindings.get(w)` の値で分岐し、その腕は `None` と `Binding` の
  7 変位 (`Param`, `Move`, `Llvm`, `Producer`, `Field`, `Payload`, `Join`) を尽くす。DEF 良い段 は
  その腕と分岐を (G1) から (G6) に分類しており、<1>4 が (G1) を、<1>5 が (G2) を、<1>6 が (G3) を、
  <1>7 が (G4) を、<1>10 が (G5) を、<1>11 が (G6) を扱う。`p = lf` の場合は <1>2 が扱い、
  `Llvm` の腕で `decl.leaf_origins_at(lf)` が空集合になる場合は <1>9 が排除する。
  BY <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>9, <1>10, <1>11, DEF 良い段,
     CODE src/rc_ir/ownership.rs: Binding, CODE src/rc_ir/ownership.rs: origin_inner

### 6.2 良くない段の棚卸し

L11 が除いた段は 3 つある。それぞれについて、現に起きるかどうかと、起きたときに何が壊れるかを
記す。

**(B1) `Llvm` の `origin_from_leaves_under` で `reached` の元が 2 つ以上あり、`lf` の宣言が単一の
`Arg` である場合。** L11 の <1>10 の <2>3 と同じ論法で `id(w, p) = (w, p)` であり、L11 の <1>11 の
<2>1 より `id(w, lf) = id(args[j], s)` であって、`args[j]` は `w` と別の変数でありうる。このとき
L4 より 2 つの `unit_of` の第 1 成分が異なるので等式は破れる。**この場合は現に起きる** --- 反例 X が
それである。起こす op は `InlineLLVMMakeUnionBody` であり、unbox union の payload が 2 つ以上の
RC unit を持つ unbox 集約であるときに、その 2 つの unit が別々の `operand_units` を作る。

`p40-cancel-soundness.md` の第 11 節の 差し戻し 3 は、この場合の成立には「1 つの unbox union の
2 つ以上の**変位**の leaf が、相異なる origin を持つ」ことが要る、と書いている。それは要らない。
`union_make` は作らない変位の leaf を空集合と宣言するので変位をまたぐ食い違いは起きないが、
**1 つの変位の中の 2 つの leaf** が相異なる `operand_units` を作れば同じ食い違いが起きる。
`InlineLLVMMakeUnionBody::result_prov` の閉包は leaf `[k] ++ rest` に `Arg(0, rest)` を与えるので、
payload の型が unbox 集約でその 2 つのフィールドが別々の unit を担うとき、
`trunc(payload の型, rest)` は leaf ごとに違う値になる。`Pair` と `(Array I64, Array I64)` が
その形であり、payload に対を持つ `Std::Option` と `Std::Result` はこれに当たる。

**(B2) `Join` の腕で `origin(w, p)` と `origin(w, lf)` の形が食い違う場合。** 一方が
`Origin::Join`、他方が `Origin::Exactly((u, s))` のとき、前者の identity は `(w, ・)`、後者の
identity は `(u, s)` であり、`u` は `w` と別の変数でありうる。この場合が現に起きる本体は作れなかった。
P7b がすでに (B1) で偽なので、この場合の決着は命題の真偽を変えない。修正の設計にはこの場合の決着が
要る。

**(B3) `here()` を返す腕で `w` が `vars.var_tys` に記録されていない場合。** `origin_inner` の
`None` の腕はグローバルの名前に当たり (`CODE src/rc_ir/ownership.rs: origin_inner` の doc)、
その名前は `VarTable` の `bindings` にも `var_tys` にも入らない。`collect_bindings` と
`VarTable::of` は束縛のたびに同じ名前を両方に入れるからである
(`CODE src/rc_ir/ownership.rs: collect_bindings`, `CODE src/rc_ir/ownership.rs: VarTable::of`)。
L4 よりこのとき `unit_of` は path をそのまま返すので、`p < lf` ならば 2 つの鍵は異なる。
この場合が現に起きる本体も作れなかった。A8 はグローバルが到達するオブジェクトが解放されないことを
述べるので、この場合の食い違いが (S-c) を破ることは無いが、README には「`unit_of` が型を引けない根の
path は切り詰められない」ことを扱う定義も仮定も無い。

### 6.3 結論

P7b はこのコミットのコードでは偽である。L11 は真であり、それが除く段は (B1)(B2)(B3) である。
(B1) は現に起きて `cancel` に解放後の読みを作らせる。

## 7. P7c

**言明 (README)** — pending な `Retain` の `outstanding` にあるオブジェクトへの参照を処分する構文は、
その `Retain` の鍵の下で `consume_unit` または `un_bump` を呼ぶ。

### 7.1 P5 (a) との差

P5 (a) の言明は「1 つの実行路の 1 つの位置において**同じ参照**を持つ 2 つのスロットで、一方から
他方への別名の道が `Match` のアーム本体の `Ret` の辺を含まないならば、両者の `unit_key` は等しい」で
ある。P7c が要るものとの差は 2 つある。

1. **参照とオブジェクトの差。** P5 (a) は 2 つのスロットが同じ**参照**を持つ場合の主張である。
   P7c が扱うのは、pending な `Retain` が bump した参照と、後の構文が処分する参照であり、この 2 つは
   同じオブジェクトを指すが同じ参照ではない。`Retain` はまさに 2 つ目の参照を作ったからである。
   D8 は同じオブジェクトへの参照を互いに区別しないので、「同じ参照を持つ」は「同じオブジェクトを
   指す」より真に強い条件である。
2. **アーム本体の `Ret` の辺。** P5 (a) は別名の道がその辺を含まないときにだけ成り立ち、
   `p12-keys-and-consumes.md` の反例 R1 が、その辺をまたぐと `unit_key` が 2 つに分かれる本体を
   与えている。P7c はその制限を置けない。`cancel` の走査は `Match` をまたいで `pending` を運ぶ
   (`merge`) ので、`Retain` と、それが bump した参照を処分する構文とのあいだにその辺が入りうる。

すなわち P7c は P5 (a) より広い主張であり、P5 (a) から導けない。

### 7.2 P7c は偽である

**証明**

<1>1. 反例 X の `Let(u1, App(drain, [action, n1]), k)` の位置において、`pending` は
      `key(t) = (action, [])` の下に `t` を持ち、その `outstanding` は
      `{(pp, [0]) : 1, (pp, [1]) : 1}` である。
  BY L9
<1>2. `outstanding` の鍵 `(pp, [0])` が名指すオブジェクトは、L10 の記号で `A` である。L7 の 2 より
      `origin(action, [0, 0]) = Exactly((pp, [0]))` であり、`pp` の leaf `[0]` が指すのが `A` で
      あるからである。
  BY L7, L10
<1>3. `App(drain, [action, n1])` は `A` への参照を 1 つ処分する。`drain` はその位置の unit を
      所有するので、D9 の消費の表の `App` の行がその leaf を消費とし、呼び出し先がその参照を処分する。
  BY D9, D14, L10
<1>4. この構文の訪問は `key(t)` の下で `consume_unit` を呼ばず、`un_bump` も呼ばない。
  <2>1. L9 の後半より、この訪問は `key(t)` の下で `consume_unit` を呼ばない。
    BY L9
  <2>2. `un_bump` を呼ぶのは `walk_inner` の `RcExpr::Release` の腕だけであり、この節点は
        `RcExpr::Let` である。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>3. QED
    BY <2>1, <2>2
<1>5. QED
  <1>1 から <1>4 より、pending な `Retain` `t` の `outstanding` にあるオブジェクト `A` への参照を
  処分する構文が、`key(t)` の下で `consume_unit` も `un_bump` も呼ばない。
  BY <1>1, <1>2, <1>3, <1>4

### 7.3 P7b が真ならば P7c に残る段

P7b が (B1)(B2)(B3) を塞いで真になったとして、P7c の証明に残る段を書き出す。この節は証明ではなく、
残りの見取り図である。

1. **処分する構文の網羅。** `Obl` から参照を取り除く構文は `Release` と D9 の消費だけである (D10)。
   `Release` については `walk_inner` の `RcExpr::Release` の腕が `others(r)` の各元に
   `consume_unit` を、`key(r)` に `un_bump` を呼ぶ。消費については P7 の前半が
   `rhs_consumes` と `destructure_consumes` が D9 の表のとおりの leaf を報告することを与え、
   `walk_inner` の `Let` と `Destructure` の腕がその各 leaf に `consume` を呼ぶ。
2. **消費の側の鍵。** 処分する構文が名指す leaf `mu` (変数 `v_c` の下) について、`consume` は
   `acted_unit_keys(v_c, mu)` の各元に `consume_unit` を呼ぶ。P5 (c) より、その集合は
   `acted_references(v_c, mu)` が名指す inhabited な leaf 由来のオブジェクトを `unit_of` で写した
   ものをすべて含む。よって `unit_of(id(v_c, mu))` はその集合の元である。
3. **pending の側の鍵。** pending な `Retain` `t = Retain(v_t, pi_t)` の `outstanding` の鍵は、
   P6 より `pi_t` の下の leaf の identity である。P7b (修正後) より、その `unit_of` は `key(t)` に
   等しい。
4. **残る隙間。** 2 と 3 は、`(v_c, mu)` と `(v_t, lambda_t)` が**同じオブジェクトを指す**ときに
   `unit_of(id(v_c, mu)) = unit_of(id(v_t, lambda_t))` であることを要求する。これは P5 (a) が
   「同じ参照」について与えるものより広い (第 7.1 節)。この段だけが P7b では埋まらない。
   README にこの主張を置くとすれば、`origin` についての層 1 の命題として置くのが筋である。

## 8. P18a

**言明 (README)** — 走査中の各位置において、`pending` に載っている各 `Retain` の `outstanding` の
各参照は、実行時にその位置で未処分である。すなわち pending な `Retain` の bump は、値自身が
もともと持っていた参照の上に積まれている。

### 8.1 言明の形式化

D8 より同じオブジェクトへの参照は互いに区別されないので、「その `Retain` が作った参照」を個別に
追う言い方は使えない。`outstanding` の側 (`References`、`Map<VarPath, usize>`) と実行時の側
(`H`、`Obl`) を結ぶには、オブジェクトごとの個数で述べるほかない。DEF pending が名指す個数 が
その橋である。言明を次の 2 つに分ける。

- **P18a-弱**: 走査中の各位置 `q` と各オブジェクト `o` について、`H(q, o) >= S(q, o)` かつ
  `Obl(q, o) >= S(q, o)` である。
- **P18a-強**: 走査中の各位置 `q` と各オブジェクト `o` について、`S(q, o) >= 1` ならば
  `H(q, o) >= S(q, o) + 1` かつ `Obl(q, o) >= S(q, o) + 1` である。

「値自身がもともと持っていた参照の上に積まれている」が述べるのは P18a-強である。
`p40-cancel-soundness.md` の第 6 節が L22 を閉じるのに要ると書いた不等式 (S1)(S2) も P18a-強の形で
ある。**この形式化は README に足すべきものである** (第 10 節)。

### 8.2 反例 Y

型と関数は 4.1 の `Pair`、`Action`、`drain` を使う。本体 `Y` は、パラメータ `pp : Pair`
(所有される) を持つ関数の本体で、次の形である。

```
Let(action, Llvm(union_make_0, [pp]),
Retain(action, [], s,
Let(u1, App(drain, [action, n1]),
Let(u2, App(drain, [action, n2]),
Ret(u2)))))
```

### L12 (反例 Y)

**言明** — `Y` は D11 を満たすが、P18a-弱 と P18a-強 はどちらも `Y` について偽である。

**証明** L10 と同じ記号を使う。`pp` の leaf `[0]` が指すオブジェクトを `A` とし、この活性化のほかに
`A` を持つ者がいない実行を選ぶ。

<1>1. `Y` の入口で `Obl(A) = 1` かつ `H(A) = 1` である。A1 より `pp` の unit は所有され、D10 の
      初期値がその leaf の参照 1 つを入れる。D8 より `H(A)` は `A` への未処分の参照の総数である。
  BY A1, D8, D10, D14
<1>2. `Retain(action, [])` の後、`Obl(A) = 2` かつ `H(A) = 2` である。L8 の 3 と 6 より、この
      `Retain` は `A` への参照を 1 つ作る。
  BY D10, L8, <1>1
<1>3. `Let(u1, App(drain, [action, n1]), ・)` の消費の後 `Obl(A) = 1` であり、呼び出し先がその参照を
      処分した後 `H(A) = 1` である。
  BY D9, D10, D14, <1>2
<1>4. `Let(u2, App(drain, [action, n2]), ・)` の消費の後 `Obl(A) = 0` であり、呼び出し先がその参照を
      処分した後 `H(A) = 0` である。
  BY D9, D10, D14, <1>3
<1>5. `Y` は D11 を満たす。
  <2>1. (S-a) が成り立つ。`Obl(A)` から参照を取り除くのは 2 つの消費だけであり、<1>2 から <1>4 の
        数え上げより、どちらの時点でも取り除かれる参照は `Obl` に入っている。`pp` の leaf `[1]` が
        指すオブジェクトについても同じである。`Ret(u2)` は `u2 : I64` なので何も取り除かない。
    BY D9, D10, <1>2, <1>3, <1>4
  <2>2. (S-b) が成り立つ。<1>4 の後 `Obl` は空である。
    BY <1>4
  <2>3. (S-c) が成り立つ。`A` を読みうる構文は `Llvm(union_make_0, [pp])` と 2 つの `App` である。
        その 3 つの位置で `H(A)` はそれぞれ 1、2、1 であり、どれも 1 以上である。
    BY D7, <1>1, <1>2, <1>3
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>6. 走査は `Ret(u2)` の位置まで `t` を `pending` に持ち、その `outstanding` は
      `{(pp, [0]) : 1, (pp, [1]) : 1}` のままである。
  <2>1. `Retain` の訪問は L9 の <1>1 と同じく、鍵 `(action, [])` の下に `t` を
        `outstanding = {(pp, [0]) : 1, (pp, [1]) : 1}` で積む。
    BY L8, L9,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
  <2>2. 2 つの `App(drain, ・)` の訪問は、L9 の <1>2 と同じく `(pp, [0])`、`(pp, [1])`、および
        callee の名前を第 1 成分に持つ鍵にしか `consume_unit` を呼ばないので、`t` を取り除かない。
        `outstanding` を変えるのは `un_bump` と `merge` だけであり、`Y` には `Release` も `Match` も
        無い。
    BY L9, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>3. QED
    BY <2>1, <2>2
<1>7. `Ret(u2)` の位置において `S(・, A) = 1` である。<1>6 より `pending` の唯一の要素は `t` で
      あり、その `outstanding` の鍵のうち `A` を指すのは `(pp, [0])` ただ 1 つで、個数は 1 で
      ある。L8 の 6 と L7 の 2 より対応する leaf は inhabited である。
  BY <1>6, L7, L8, DEF pending が名指す個数
<1>8. QED
  <1>4 より `Ret(u2)` の位置で `H(A) = 0` かつ `Obl(A) = 0` である。<1>7 より `S = 1` なので、
  P18a-弱 の `H >= S` が破れる。P18a-強 はその位置でも破れる。さらに P18a-強 は
  `Let(u2, App(drain, [action, n2]), ・)` の位置でも破れる。そこでは <1>3 より `H(A) = 1` かつ
  `Obl(A) = 1` で、<1>6 と <1>7 と同じ計算で `S = 1` であり、`H >= S + 1` が成り立たない。
  BY <1>3, <1>4, <1>6, <1>7

**この反例には節点の削除が要らない。** `Y` には `Release` が無いので、`t` は `un_bump_releases` が
空のまま `Ret` の訪問で `needed_retains` に入り、`cancelled()` は何も返さない
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の `RcExpr::Ret(_)` の腕,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled`)。P18a はコードの出力が誤っていなくても偽で
ある。

`Y` の形も Fix から出る。第 5 節のプログラムの `Action`、`consume_pair`、`drain` はそのままに、
`run` を次に置き換えると、`-O max` の `rc_ir.post.txt` は `union_make_0` の直後に `retain` 1 つ、
その後に `drain` の呼び出し 2 つ、`release` 無し、という形になる (`1ee99ba7` で確認)。

```fix
run : (Array I64, Array I64) -> I64;
run = |payload| (
    let action = Action::pair(payload);
    let u1 = drain(action, 1);
    let u2 = drain(action, 1);
    u1 + u2
);
```

### 8.3 P7b と P7c が真ならば P18a に残る段

P7c が真ならば P18a-強 は実行路の上の位置についての帰納法で閉じる見込みがある。残る段を書き出す。
この節は証明ではなく、残りの見取り図である。

1. **`S` を動かす遷移。** `S` が増えるのは `Retain` 節点の訪問だけであり (P16 の (a) と
   `walk_inner` の `Retain` の腕)、減るのは `un_bump` と `consume_unit` と `merge` だけである
   (P17、P18)。
2. **`H` を下げる遷移。** `H` を下げるのは `Release` と、消費した参照を呼び出し先が処分することの
   2 つである (D10)。
3. **`Retain` の段。** `Retain(v, pi)` の訪問で `S(・, o)` が `k` 増えるとき、同じ位置で `H(・, o)`
   も `k` 増える (P6 と D10)。よって `H >= S + 1` は、その `Retain` の直前に `H(・, o) >= 1` で
   あることに帰着する。それは、その `Retain` が触れる leaf のスロットがその位置に在ること (A5) から
   出る。
4. **処分の段。** `H(・, o)` が下がる位置で `S(・, o) >= 1` ならば、P7c よりその構文は `key(t)` の
   下で `consume_unit` か `un_bump` を呼ぶ。前者ならば `t` は `pending` から外れて `S` が下がり、
   後者ならば `un_bumped` の分だけ `outstanding` が減って `S` が下がる (P17)。
5. **残る隙間。** 4 の「`S` が下がる量が `H` の下がる量以上である」を言うには、`un_bump` が引く
   `un_bumped(r)` の `o` の個数と、`Release(v_r, pi_r)` が実行時に処分する `o` への参照の個数とが
   一致することが要る。P6 がこれを与える。消費の側については、`consume_unit` は `t` を丸ごと外すので
   `S` は `outstanding` の分だけ一度に下がり、`H` は消費 1 つにつき高々 1 しか下がらない。
6. **`merge` の段。** `merge` が返す `pending` に残る `Retain` の `outstanding` は、すべてのアームの
   出口で一致する値である (P18)。よって `S` はどのアームを選んだ実行でもその値であり、アームの中で
   保たれた不等式がそのまま続く。

## 9. `level_ownership` の影響

`infer_ownership` に平準化の段が入った (`CODE src/rc_ir/borrow.rs: level_ownership`,
`CODE src/rc_ir/borrow.rs: levelled_sites`, `CODE src/rc_ir/borrow.rs: covered_leaves`)。この 3 つが
P7b、P7c、P18a に効くかどうかを確かめた。

**言明** — `level_ownership`、`levelled_sites`、`covered_leaves` は P7b の両辺を動かさない。

**証明**

<1>1. `level_ownership` が書き込むのは `owned_leaves` だけである。その第 4 引数は
      `&mut Set<VarPath>` であり、返り値は「変わったか」の真偽値である。`levelled_sites` は
      `func.body` を読んで `(RcVar, FieldPath)` の列を作るだけ、`covered_leaves` は型と path から
      leaf の列を作るだけであり、どちらも書き込む先を持たない。
  BY CODE src/rc_ir/borrow.rs: level_ownership, CODE src/rc_ir/borrow.rs: levelled_sites,
     CODE src/rc_ir/borrow.rs: covered_leaves
<1>2. `origin`、`unit_of`、`unit_key`、`acted_unit_keys`、`acted_references` は `owned_leaves` を
      読まない。これらの引数は `vars`、`type_env`、変数名、path だけである。
  BY CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: unit_of,
     CODE src/rc_ir/ownership.rs: unit_key, CODE src/rc_ir/ownership.rs: acted_unit_keys,
     CODE src/rc_ir/ownership.rs: acted_references
<1>3. QED
  P7b の両辺は <1>2 の関数の値だけで決まる。
  BY <1>1, <1>2

P7c と P18a には、`owned_leaves` が `borrowed_units` を経て `cancel` の `owned_units` に届き、
`consume_rhs` の `owns` 閉包がどの引数位置を消費とするかを決める道で効く
(`CODE src/rc_ir/borrow.rs: borrow_ify`, `CODE src/rc_ir/ownership.rs: all_owned_units`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs`)。`level_ownership` は所有を増やす向きにしか
動かないので、消費される leaf は増えるだけであり、増えた leaf は `consume` を通って
`acted_unit_keys` の各鍵に `consume_unit` を呼ぶ。すなわち P7c の義務 (参照を処分する構文) と、
それを果たす呼び出し (`consume_unit`) が同じだけ増える。鍵の食い違い (B1) はどちらの側にも残る。
反例 X が `level_ownership` を含むコミットで再現すること (第 5 節) がこれを裏づける。

## 10. README へ差し戻す点

### 差し戻し 1 (P7b は偽なので、直すのはコードである)

第 6 節が反例を与えた。P7b の言明はそのままで、直すべきはコードである。第 11 節が候補を挙げる。
P7b を弱めて逃げる道 (たとえば「payload が 1 つの unit しか持たない unbox union について」と
限定する) は、`Std::Option (a, b)` と `Std::Result e (a, b)` を除外できないので採れない。

### 差し戻し 2 (L11 を層 1 の命題として置く)

この文書の L11 は、`unit_key` の doc が散文で述べていることの形式化であり、`origin` についての
主張なので層 1 に属する。P7b を直したときに残る証明の骨格でもある。README に置くならこの形が
よい。DEF 覆う も一緒に要る。P7b の仮定「path は unit である」が `origin` の再帰について閉じない
からである。

### 差し戻し 3 (P18a の言明を個数で述べる)

第 8.1 節。README の P18a は「`outstanding` の各参照は未処分である」と書くが、D8 が参照を区別
しないのでこの文は述語になっていない。P18a-弱 と P18a-強 の 2 つを置き、層 4 が使うのは強い方で
あることを書くのがよい。

### 差し戻し 4 (P7c が要求する「1 つのオブジェクト、1 つの鍵」)

第 7.3 節の 4。P7c を閉じるには、「1 つの位置で同じオブジェクトを指す 2 つのスロットの
`unit_of(id(・))` は等しい」が要る。P5 (a) は「同じ参照」についてしか与えない。この主張を
層 1 の命題として置くべきである。`p40-cancel-soundness.md` の第 11 節の 差し戻し 4 が同じことを
求めている。

### 差し戻し 5 (`unit_of` が型を引けない根)

第 6.2 節の (B3)。`unit_of` は `vars.var_tys` に型が無い根の path を切り詰めない。README には
この場合を扱う定義も仮定も無い。A8 (グローバルは線形規律の外) は `H` について述べるだけで
`unit_key` については何も述べないので、そのままではこの場合を覆わない。

### 差し戻し 6 (A3 の「空集合」の行の向き)

L11 の <1>9 は、A3 の「空集合」の行の「その leaf は inhabited にならない」を、inhabited な leaf の
宣言は空集合ではない、という向きに使う。A3 の現在の文面はこの向きを与えるが、表の行として一目で
読み取れる形ではない。`p40-cancel-soundness.md` の第 11 節の 差し戻し 2 が「単一の `Arg`」の行に
ついて同じ種類の補強を求めており、「空集合」の行にも同じ補強を足すのがよい。

## 11. 気づいたコードの欠陥

**新規 (miscompile、issue 番号は未採番)。** `-O max` で、`cancel` が `Retain`/`Release` の対を、
その間にある消費をまたいで削除し、解放されたオブジェクトを読むコードを作る。第 4 節が形式的な
追跡を、第 5 節が Fix の再現プログラムと計測を与える。`-O none` と `-O basic` は正しい。

**発火の条件** (第 4 節と第 6.2 節の (B1) から)。

1. unbox union の変位の payload が、相異なる 2 つ以上の RC unit を持つ unbox 集約であること。
   `(Array I64, Array I64)` を payload に持つ `Std::Option` や `Std::Result` がこれに当たる。
2. その union が `union_make` で作られ、その値に対する `Retain`/`Release` が残ること。
3. `Retain` と `Release` のあいだに、その payload の leaf を消費する構文があること。

**なぜ既存の検査が捕まえないか。**

- `check_one_key_per_object` は、`key` と異なる鍵の下の pending な `Retain` の `outstanding` が
  `un_bumped` に**等しい**ことだけを禁じる。この欠陥では pending の鍵が 1 つしかないので、
  繰り返しの本体が `continue` で飛ばされる (`CODE src/rc_ir/borrow.rs: check_one_key_per_object`)。
- `unit_of` の表明は、切り詰めた path がその型の unit であることだけを見る。鍵の根が別の変数へ
  移ったことは見ない (`CODE src/rc_ir/ownership.rs: unit_of`)。
- `src/tests/test_union_rc_shapes.rs` の `UNION_PAYLOAD_UNITS_SOURCE` は payload が 2 つの unit を
  持つ形を測っているが、そこでは union を作った payload が呼び出しの後も生きている
  (`via_pair` の `let (first, second) = payload;`)。`origin` の候補になる変数がまだ参照を持つので、
  対を消しても解放は早まらない。欠陥が出るのは payload の側が死んで union だけが参照を持つときで
  ある。

**直す方向の候補。** どれを採るかは、`unit_key` が何を名指す量であるかという設計判断なので、
ここでは挙げるにとどめる。

1. `origin_from_leaves_under` が `reached` の食い違いで作る `Origin::Join` を、`cancel` が
   「消してはならない `Retain`」の印として扱う。すなわち `Retain(v, pi)` の `origin(v, pi)` が
   `Origin::Join` であってその identity が `(v, pi)` 自身であるとき、その `Retain` を
   `needed_retains` に入れる。もっとも小さい修正だが、`Std::Option (a, b)` を含むコードで対の削除を
   諦める。
2. 消費の鍵を広げる。`CancelAnalysis::consume` が `acted_unit_keys(v_c, mu)` に加えて、
   `unit_of((v_c, trunc(ty(v_c), mu)))` --- 消費される値自身の unit の鍵 --- でも `consume_unit` を
   呼ぶ。反例 X はこれで塞がるが、消費が `union_as` や `Match` を経た別の変数を通る形では鍵がまた
   分かれるので、これだけでは足りない。
3. `Retain`/`Release` の鍵を、unit の `origin` の identity 1 つではなく、その unit の下の leaf の
   `origin` の集合にする。`pending` の鍵の型を変えることになり、影響範囲が大きい。

**この欠陥は #519 と #529 と同じ族である。** どれも、1 つのオブジェクトが道ごとに 2 つ以上の名前を
持ち、`Retain` と `Release` が別の鍵に分かれることで起きる。#519 は `origin_from_leaves_under` が
読み出した値自身の名前で組み直していたこと、#529 は `Join` を畳むときに内側の identity を落として
いたこと、この欠陥は `reached` が 2 元以上のときに identity を `here` へ置き換えることである。
