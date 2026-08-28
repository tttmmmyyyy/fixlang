# P3 / P4 -- `origin` の健全性

対象コミット `b81cc2c8e859a00cbf007e4f43483a514c813c73`。定義・仮定・命題の番号は同ディレクトリの
`README.md` (`91e3f6bd` の版) による。

## 0. この文書の状態

**証明は完成していない。P4 の周りでコードの誤りを見つけたので、そこで止めた。**

持ち帰ったものは 4 つである。

1. **発見 1 (コードの誤り、第 4 節)。** `cancel` が retain と release を対にするために要る不変条件が破れる。
   `acted_unit_keys(v, π)` が、`acted_references(v, π)` の名指すオブジェクトを取りこぼす。取りこぼしは
   `origin` が `Join` を平坦化するときに起き、#519 (`8fb0dd79` が直したもの) と同じ形である。反例は分岐を
   要さず、入力は健全で出力は D11 の (S-c) を破る。
2. **発見 2 (定義の誤り、第 5 節)。** P3 と P4 の「対応する leaf」(残りの path を後ろに繋ぐ規則) は、
   `origin_inner` の `Llvm` の腕の一部 (`origin_from_leaves_under`) には当てはまらない。union の構築が
   反例である。
3. **突き合わせ 2 つ (第 2 節、第 3 節)。** D9 の「移動」と `origin_inner` の別名の辺、D10 の「生成」と
   `origin_inner` の `here()` の腕。移動の表に対応の無い辺が 1 つある。
4. **README への要望 (第 7 節)。**

## 1. 記法

`origin(x, π)` は `origin(vars, type_env, &x, &π)` の略記とする。`VarPath` を `(x, π)` と書く。
`leaves(τ)` は `boxed_leaf_paths(τ)`、`leaves(τ, π)` は `π` で始まる `leaves(τ)` の要素とする
(`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`, `CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under`)。

`Origin` の構成子と読み出しは次のとおりである (`CODE src/rc_ir/ownership.rs: Origin`, `Origin::identity`,
`Origin::candidates`, `Origin::acted_on`, `Origin::of_candidates`)。

- `Exactly(p)`: `identity() = p`、`candidates() = [p]`、`acted_on() = [p]`。
- `Join { identity, candidates }`: `identity() = identity`、`candidates() = candidates`、
  `acted_on() = [identity] ++ (candidates` から `identity` を除いたもの`)`。**`candidates` は `identity` を
  含むとは限らない。**
- `of_candidates(C, h)`: `C` が 1 要素なら `Exactly` (その要素)、2 要素以上なら
  `Join { identity: h, candidates: C }`。`C` が空なら panic する。

## 2. 突き合わせ 1 -- D9 の「移動」と `origin_inner` の別名の辺

**別名の辺**とは、`origin_inner` が `origin` を再帰呼び出しする先をいう
(`CODE src/rc_ir/ownership.rs: origin_inner`)。

<1>1. `origin_inner` の `match` の腕は 7 つで尽きている。
  <2>1. `vars.bindings.get(var)` は `Option<&Binding>` を返す。
    BY CODE src/rc_ir/ownership.rs: VarTable の `bindings` フィールド
  <2>2. `Binding` の構成子は `Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join` の 7 つで
        ある。
    BY CODE src/rc_ir/ownership.rs: Binding
  <2>3. QED
    BY <2>1, <2>2 -- 腕は `None | Some(Param) | Some(Producer)` を束ねた 1 本と、残り 6 本。

<1>2. 別名の辺は次の 7 つで尽きている。
  BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner

| 辺 | 腕 | 行き先 |
|---|---|---|
| E1 | `Move(y)` | `origin(y, π)` |
| E2 | `Join(rs)` | 各 `r` in `rs` について `origin(r, π)` |
| E3 | `Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)` | `origin(args[j], σ)` |
| E4 | `Llvm` かつ E3 でない | `leaves(τ, π)` の各 leaf の宣言の各 `Arg(j, σ)` について `origin(args[j], truncate_to_unit(ty(args[j]), σ))` |
| E5 | `Field(c, i)` かつ `c` が unbox | `origin(c, [i] ++ π)` |
| E6 | `Payload(s, None)` | `origin(s, π)` |
| E7 | `Payload(s, Some(t))` かつ `s` が unbox | `origin(s, [t] ++ π)` |

E4 の中身は `origin_from_leaves_under` である
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。

D9 の移動の 6 行との対応:

| D9 の移動 | 辺 |
|---|---|
| `Let(x, Var(y), k)` | E1 |
| `Match` のアーム本体の `Ret(x)` | E2 (`Binding::Join` はアーム本体の `Ret` の変数を集める。`CODE src/rc_ir/ownership.rs: collect_bindings` の `RcRhs::Match` の腕、`returned_var`) |
| unbox 容器の `Destructure` の名前付きフィールド | E5 (`collect_bindings` の `RcExpr::Destructure` の腕は名前付きフィールドにだけ `Binding::Field` を作る) |
| unbox union の変位アームの payload 束縛 | E7 |
| catch-all アームの payload 束縛 | E6 |
| `Llvm` の素通し leaf (`result_prov` が単一の `Arg(i, σ)`) | E3 (`as_arg_projection` が集合の要素数 1 を要求する。`CODE src/rc_ir/ownership.rs: as_arg_projection`) |

**E4 は D9 の移動の表に対応する行を持たない。** ずれは 3 つある。

- **(a) unit への切り上げ。** E4 の行き先は宣言が名指した leaf `σ` ではなく、`truncate_to_unit` で切り上げた
  unit path である。`π` の下の leaf 群がその unit の leaf 群を覆いきるとき、E4 は E3 の合成に等しい。
  覆いきらないとき、答えは値が持たない参照まで含む部分木を名指す。
- **(b) 単一でない宣言の `Arg` を辿る。** `origin_from_leaves_under` の `for src in sources` のループは、
  leaf の宣言の要素数によらずすべての `Arg(j, σ)` を辿る。A3 の表では複数元の宣言は「いずれの路でも
  新しい参照」であり、D9 の移動ではなく D10 の生成である。すなわちこの辺は A3 が許していない。
  ただし現在のコードでは発火しない。全 29 個の `result_prov` を読んだ結果、複数元の集合を宣言する op は
  存在しない (宣言に使われているのは `sole_origin`、`Set::default()`、`Provenance::uniform`、
  `Provenance::fresh_under`、`Provenance::build_shape` の 5 つで、どれも単一集合か空集合しか作らない。
  `CODE src/fixstd/builtin.rs` の全 `result_prov`、`CODE src/ast/inline_llvm.rs: LLVMGen::result_prov` の
  既定、`CODE src/rc_ir/provenance.rs: Provenance::uniform`, `Provenance::fresh_under`,
  `Provenance::arg_passthrough`, `Provenance::uniform_bottom`)。複数元の集合を作るのは
  `Provenance::join` (アームの合流) であり、これは解析の側であって宣言ではない。`origin_inner` が読むのは
  宣言なので、この辺には到達しない。
- **(c) 形が変わる宣言。** E4 は答えの path をオペランドの unit path にするので、結果の leaf と
  オペランドの leaf の対応は path の接頭辞の書き換えにならない。第 5 節 (発見 2)。

## 3. 突き合わせ 2 -- D10 の「生成」と `here()` の腕

`here()` は `Origin::Exactly((var, path))` を返す閉包である
(`CODE src/rc_ir/ownership.rs: origin_inner` の先頭)。`here()` と同じ答えに着く道は 7 つある。

| 道 | 着き方 | D10 での位置 |
|---|---|---|
| H1 `None` (表に無い名前) | 直接 | A8 (グローバルは線形規律の外) |
| H2 `Param` | 直接 | D10 の初期値 |
| H3 `Producer` | 直接 | 生成の表の `App` の行と `Closure` の行 (`collect_bindings` は `RcRhs::App` と `RcRhs::Closure` にだけ `Binding::Producer` を作る) |
| H4 `Field(c, i)` かつ `c` が boxed | 直接 | 生成の表の boxed 容器の `Destructure` の行 |
| H5 `Payload(s, Some(t))` かつ `s` が boxed | 直接 | 生成の表の boxed union の変位アームの行 |
| H6 `Llvm` かつ `π` の下のある leaf の宣言が `Fresh` か `Unknown` を含む | `origin_from_leaves_under` が `produced_here` を立てて `Exactly(here)` を `reached` に積む。`reached` が 1 要素ならそれが答え | 生成の表の `Llvm` の行 |
| H7 `Llvm` かつ `π` の下の leaf の宣言がすべて空集合 | `origin_from_leaves_under` が `None` を返し `unwrap_or_else(here)` | A3 より、その leaf は inhabited にならない。D10 の生成は起きない |

D10 の生成の 5 行はすべて `here()` の道を持つ (H3 が 2 行、H4、H5、H6)。逆向きには `here()` の方が 3 つ多く、
H1 は A8 が規律の外に置くもの、H2 は D10 の初期値、H7 は inhabited な leaf を持たない位置である。どれも
「新しい参照を作る」とは主張していないので、生成の表に無いことと矛盾しない。

D10 の生成の `Llvm` の行が「単一の `Arg(j, σ)` でない leaf」であるのに対し、`here()` に着く `Llvm` の道は
H6 と H7 だけである。差は複数元の宣言で、そのとき `origin_from_leaves_under` は `produced_here` を立てずに
`Arg` を辿る。第 2 節の (b) と同じずれであり、現在のコードでは発火しない。

## 4. 発見 1 -- `cancel` が要る不変条件が破れる

### 4.1 何が要るのか

`cancel` は retain と release を `unit_key` で対にする。キーが違う release は対にならないので、そのままでは
retain が生き残って別の release と対になりうる。それを止める仕組みが 1 つだけあり、release と消費が
`acted_unit_keys` の各キーについて `consume_unit` を呼んで、そのキーの pending な retain に「載っている」の
印を付ける (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の `RcExpr::Release` の腕、`consume`、
`consume_unit`)。この仕組みが働くために要るのは次の (N) である。

> **(N)** `acted_unit_keys(v, π)` は、`acted_references(v, π)` が名指すオブジェクトを、`unit_of` で写した
> 上ですべて含む。

(N) はコードが散文で述べている不変条件でもある。`unit_key` の doc は
「The units an operation on it really touches are `acted_unit_keys`」と書き、`acted_references` の doc は
「how many references of each object it bumps or un-bumps」と書く
(`CODE src/rc_ir/ownership.rs: unit_key`, `acted_references`)。

**(N) は破れる。** 以下は `origin` の静的な計算だけで確かめられる。

### 4.2 反例

型と関数:

```
type Node   = box   struct { n : I64 };
type Pair   = unbox struct { fst : Node, snd : Node };
type Choice = unbox union  { nothing : (), both : Pair };

f : Choice -> Array I64 -> I64      // 2 つの引数を読むだけで、どの leaf も消費しない
```

RC IR (`insert_rc`、`split_rc_units`、`borrow_ify` の後、`cancel` の直前):

```
let m    = match c { true => { ... ; ret p }, false => { ... ; ret q } };
retain m, [];                       // R  : insert_rc の規則 (a)
let pair = struct_make(m, w);
let u    = union_make_1(pair);
retain arr, [];                     // insert_rc の規則 (a)
let n    = f#borrow(u, arr);        // borrow_ify の route が借用版へ回した
release arr, [];                    // call_rc
release u, [];                      // L1 : call_rc
let k    = struct_get_0(m);
release m, [];                      // L2 : insert_rc の規則 (b)
... arr を使う ... ; ret ...
```

`p`、`q`、`w`、`arr` はいずれも `Binding::Producer` (呼び出しの結果) とする。

<1>1. `retain m, []` と `release m, []` は `insert_rc` が置く。
  <2>1. `struct_make` のオペランド `m` の ownership は `Own` である。
    <3>1. `rhs_operands` は `RcRhs::Llvm` のオペランド `i` を、`borrows_operand(i, ..)` が真のときだけ
          `Borrow` とする。
      BY CODE src/rc_ir/rc_insert.rs: rhs_operands
    <3>2. `InlineLLVMMakeStructBody` は `borrows_operand` を override しない。
      BY CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody
    <3>3. `LLVMGen::borrows_operand` の既定は `false` である。
      BY CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand
    <3>4. QED
      BY <3>1, <3>2, <3>3
  <2>2. `m` は `struct_make` の後でも使われる (`struct_get_0(m)`) ので、`insert_rc` は `struct_make` の
        直前に `Retain(m, [])` を置く。
    BY <2>1, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let の `retains_before`,
       build_retains
  <2>3. `struct_get_0` のオペランド `m` の ownership は `Borrow` である。
    <3>1. `InlineLLVMStructGetBody::borrows_operand` は `i == 0` かつ `borrows_container(field_ty)` の
          とき真である。
      BY CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMStructGetBody の `borrows_operand`
    <3>2. `borrows_container(τ)` は `τ.is_fully_unboxed(type_env)` である。読むフィールドの型は `I64` なので
          真である。
      BY CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_container
    <3>3. QED
      BY <3>1, <3>2, CODE src/rc_ir/rc_insert.rs: rhs_operands
  <2>4. `m` は `struct_get_0` の後で死ぬので、`insert_rc` はその直後に `Release(m, [])` を置く。
    BY <2>3, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let の `releases_after`,
       build_releases
  <2>5. QED
    BY <2>2, <2>4

<1>2. `release u, []` は `borrow_ify` の `call_rc` が置く。
  <2>1. `f` は借用版を持つ。
    BY 前提 (`f` はどの leaf も消費しない), CODE src/rc_ir/borrow.rs: infer_ownership,
       func_has_borrowable_param, borrow_ify
  <2>2. `route` はこの呼び出しを借用版へ回す。
    <3>1. `routing_is_safe` は真である。`n` はこの関数の返り値ではないので、末尾位置の呼び出しではない。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::routing_is_safe, tail_result_vars
    <3>2. `routing_saves_retain` は真である。引数 `arr` の unit `[]` について、呼び出し先は借用し、
          `arr` は呼び出しの後でも使われるので `arg_used_later` が真であり、条件
          `callee_borrows && !(owns_unit && !arg_used_later && !comes_from_a_value_used_later)` が
          成り立つ。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::routing_saves_retain, used_later
    <3>3. QED
      BY <3>1, <3>2, CODE src/rc_ir/borrow.rs: RewriteCtx::route
  <2>3. `call_rc` は `u` の unit `[]` について後置の `Release` を積む。
    <3>1. `rc_units(Choice)` は `[[]]` である。`Choice` は unbox union なので `unit_step` は `Unit` を
          返す。
      BY CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
    <3>2. 呼び出し先はこの unit を所有しない (借用版なので `borrowed_units` に入る)。
      BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: borrow_ify の `borrowed_units` の書き込み,
         CODE src/rc_ir/ownership.rs: all_owned_units
    <3>3. 呼び出し元はこの unit を所有する。`owns_unit(u, [])` は `origin(u, []).candidates()` の各要素に
          ついて `owns_object` を問い、どれも `param_tys` に無いので真になる。
      BY 4.3 の <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit, owns_object
    <3>4. QED
      BY <3>1, <3>2, <3>3, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc の
         `if !callee_owns && arg_owned` の枝
  <2>4. QED
    BY <2>3, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::App(..), k)` の
       腕, prepend_rc

<1>3. `split_rc_units` はどの RC 節点の path も変えない。
  <2>1. `rc_units(Node)` は `[[]]` である。`Node` は boxed なので `unit_step` は `Unit` を返す。
    BY CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>2. `rc_units(Array I64)` は `[[]]` である。`is_array` が真なので `unit_step` は `Unit` を返す。
    BY CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>3. QED
    BY <2>1, <2>2, 4.2 の <1>2 の <2>3 の <3>1, A2

<1>4. この RC IR は D11 の意味で健全である。
  <2>1. `struct_make(m, w)` は `m` も `w` も消費せず、D9 の移動を行う。
    <3>1. `InlineLLVMMakeStructBody::result_prov` は、unbox struct の leaf `[i] ++ rest` に
          `sole_origin(Arg(i, rest))` を宣言する。`Pair` の leaf は `[0]` と `[1]` なので、宣言は
          `Arg(0, [])` と `Arg(1, [])` である。
      BY CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody の `result_prov`,
         CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <3>2. `passthrough_arg_leaves` は `(0, [])` と `(1, [])` を集め、`rhs_consumes` の `RcRhs::Llvm` の腕は
          それらを `out` に積まない。
      BY <3>1, CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection, rhs_consumes
    <3>3. QED
      BY <3>1, <3>2, D9 の移動の表の `Llvm` の行、A3 の「単一の `Arg(j, σ)`」の行
  <2>2. `union_make_1(pair)` は `pair` を消費せず、D9 の移動を行う。
    <3>1. `InlineLLVMMakeUnionBody::result_prov` は、構築した変位の leaf `[k] ++ rest` に
          `sole_origin(Arg(0, rest))`、他の変位の leaf に空集合を宣言する。`Choice` の leaf は `[1, 0]` と
          `[1, 1]` である (変位 0 の payload は `()` で leaf を持たない)。よって宣言は `Arg(0, [0])` と
          `Arg(0, [1])` である。
      BY CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeUnionBody の `result_prov`,
         CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <3>2. QED
      BY <3>1, <2>1 の <3>2 と同じ規則
  <2>3. `u` の leaf `[1, 0]` と `[1, 1]` は inhabited であり、`obj(u, [1, 0])` は `obj(m, [])` である。
    BY <2>2 の <3>1 (タグは変位 1), D16, <2>1, <2>2, A3 の「単一の `Arg(j, σ)`」の行
  <2>4. `f#borrow(u, arr)` は `u` の leaf も `arr` の leaf も消費しない。
    BY <1>2 の <2>1 と <2>3 の <3>2, D9 の消費の表の `App` の行
  <2>5. `obj(m, [])` への参照は、この本体で 2 つ作られ 2 つ処分される。
    <3>1. アーム本体の `Ret(p)` が `p` の参照を `m` へ移し、`R` がもう 1 つ作る。
      BY D9 の移動の表の `Match` のアーム本体の行、D10 の `Retain` の行
    <3>2. `pair` と `u` は移動なので `Obl` を変えない。`u` の leaf `[1, 0]` はその 2 つのうち 1 つを持つ。
      BY <2>1, <2>2, <2>3, D9 の移動の表
    <3>3. `L1` は `u` の `[]` の下の inhabited な各 leaf、すなわち `[1, 0]` と `[1, 1]` の参照を処分する。
          前者は `obj(m, [])` への参照である。
      BY D10 の `Release` の行, <2>3
    <3>4. `L2` は `obj(m, [])` への参照をもう 1 つ処分する。
      BY D10 の `Release` の行
    <3>5. QED
      BY <3>1, <3>3, <3>4
  <2>6. QED
    BY <2>5 -- (S-a) は各処分の時点で `Obl` にその参照があること、(S-b) は終端で空になること、(S-c) は
       `struct_get_0(m)` の位置で `H(obj(m, [])) = 1` であることによる。`arr` の側は `retain arr` と
       `release arr` が対になる。

### 4.3 `origin` の計算

<1>1. `origin(m, []) = Join { identity: (m, []), candidates: {(p, []), (q, [])} }`。
  <2>1. `m` の binding は `Binding::Join([p, q])` である。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕, returned_var
  <2>2. `origin(p, []) = Exactly((p, []))`、`origin(q, []) = Exactly((q, []))`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Producer)` の腕
  <2>3. 腕は各アームの結果の `candidates()` を集め、`of_candidates(C, (m, []))` を呼ぶ。<2>2 より
        `C = {(p, []), (q, [])}` である。
    BY <2>2, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕,
       Origin::candidates
  <2>4. QED
    BY <2>3, CODE src/rc_ir/ownership.rs: Origin::of_candidates -- 要素数 2 なので `Join`。

<1>2. `origin(pair, [0]) = origin(m, [])`、`origin(pair, [1]) = Exactly((w, []))`。
  <2>1. `pair` の binding は `Binding::Llvm(struct_make, [m, w], Pair)` である。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Llvm` の腕
  <2>2. `leaf_origins_at([0])` は `{Arg(0, [])}` であり、`as_arg_projection` は `Some((0, []))` を返す。
        `leaf_origins_at([1])` は `{Arg(1, [])}` であり、`Some((1, []))` を返す。
    BY 4.2 の <1>4 の <2>1 の <3>1, CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>3. QED
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の
       `Some((j, σ))` の枝 (E3)

<1>3. `origin(u, []) = Join { identity: (u, []), candidates: {(p, []), (q, []), (w, [])} }`。
  <2>1. `[]` は `Choice` の boxed leaf ではないので `leaf_origins_at([])` は `None` を返し、
        `origin_from_leaves_under` に入る。
    BY 4.2 の <1>4 の <2>2 の <3>1 (leaf は `[1, 0]` と `[1, 1]`),
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>2. `operand_units = {(0, [0]), (0, [1])}` であり、`produced_here` は偽である。
    <3>1. `leaf_origins_under([])` は `[1, 0]` と `[1, 1]` の宣言、すなわち `{Arg(0, [0])}` と
          `{Arg(0, [1])}` を返す。
      BY 4.2 の <1>4 の <2>2 の <3>1, CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
    <3>2. `truncate_to_unit(Pair, [0], env) = [0]`、`truncate_to_unit(Pair, [1], env) = [1]`。
      BY CODE src/rc_ir/ownership.rs: truncate_to_unit, unit_step -- `Pair` は unbox struct なので
         `Fields`、添字を積んで `Node` へ降り、path を使い切る。
    <3>3. QED
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `for sources` のループ --
         `Fresh` も `Unknown` も現れないので `produced_here` は偽のまま。
  <2>3. `reached = [origin(pair, [0]), origin(pair, [1])]` であり、この 2 要素は相異なる。
    BY <2>2, <1>2, <1>1 -- 一方は `Join`、他方は `Exactly`。
  <2>4. `reached` の全要素が等しくないので、答えは `of_candidates(C, (u, []))` であり、`C` は各要素の
        `candidates()` の合併である。
    BY <2>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `reached.iter().all(..)` の後
  <2>5. `C = {(p, []), (q, []), (w, [])}`。
    BY <2>4, <1>1, <1>2, CODE src/rc_ir/ownership.rs: Origin::candidates -- `Join` の `candidates()` は
       `identity` を含まないので `(m, [])` は入らない。
  <2>6. QED
    BY <2>5, CODE src/rc_ir/ownership.rs: Origin::of_candidates -- 要素数 3 なので `Join`。

### 4.4 (N) の反例

<1>1. `acted_references(u, [])` は `(m, [])` を名指す。
  <2>1. `acted_references(u, [])` は `[]` で始まる各 boxed leaf について `origin(u, leaf).identity()` を
        数える。leaf は `[1, 0]` と `[1, 1]` である。
    BY CODE src/rc_ir/ownership.rs: acted_references, 4.2 の <1>4 の <2>2 の <3>1
  <2>2. `origin(u, [1, 0]) = origin(pair, [0]) = origin(m, [])`。
    <3>1. `leaf_origins_at([1, 0])` は `{Arg(0, [0])}` であり、`as_arg_projection` は `Some((0, [0]))` を
          返す。
      BY 4.2 の <1>4 の <2>2 の <3>1, CODE src/rc_ir/ownership.rs: as_arg_projection
    <3>2. QED
      BY <3>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の
         `Some((j, σ))` の枝 (E3), 4.3 の <1>2
  <2>3. QED
    BY <2>1, <2>2, 4.3 の <1>1 -- `origin(m, []).identity() = (m, [])`。

<1>2. `acted_unit_keys(u, [])` は `(m, [])` を含まない。
  <2>1. `acted_unit_keys(u, [])` は `origin(u, []).acted_on()` の各要素を `unit_of` で写したものである。
    BY CODE src/rc_ir/ownership.rs: acted_unit_keys
  <2>2. `origin(u, []).acted_on() = [(u, []), (p, []), (q, []), (w, [])]`。
    BY 4.3 の <1>3, CODE src/rc_ir/ownership.rs: Origin::acted_on
  <2>3. `unit_of` は根の変数を変えない。
    BY CODE src/rc_ir/ownership.rs: unit_of -- 返すのは `(root.clone(), truncated)`。
  <2>4. QED
    BY <2>1, <2>2, <2>3 -- 4 要素の根はどれも `m` ではない。

<1>3. QED
  BY <1>1, <1>2 -- (N) の反例である。

### 4.5 `cancel` の帰結

<1>1. `R` は鍵 `(m, [])` で `pending` に積まれる。
  BY 4.3 の <1>1, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain` の腕,
     CODE src/rc_ir/ownership.rs: unit_key, unit_of

<1>2. `let pair = ...`、`let u = ...`、`let n = f#borrow(u, arr)` はどれも `R` に印を付けない。
  BY 4.2 の <1>4 の <2>1, <2>2, <2>4 (どれも消費しない),
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, CODE src/rc_ir/ownership.rs: rhs_consumes

<1>3. `L1` は `R` に印を付けず、`R` を un-bump もしない。
  <2>1. `L1` の鍵は `(u, [])` である。
    BY 4.3 の <1>3, CODE src/rc_ir/ownership.rs: unit_key, unit_of
  <2>2. `L1` が `consume_unit` を呼ぶ鍵は `(p, [])`、`(q, [])`、`(w, [])` である。
    BY <2>1, 4.4 の <1>2 の <2>2, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の
       `RcExpr::Release` の腕の `for other in self.acted_unit_keys(..)` のループ
  <2>3. `pending` に鍵 `(u, [])` の項目は無いので `un_bump` は `NoBracket` を返す。
    BY <1>1, <2>1, CODE src/rc_ir/borrow.rs: un_bump
  <2>4. `check_one_key_per_object` は発火しない。
    <3>1. この検査は、鍵が違う pending な retain の `outstanding` が `un_bumped` と**等しい**ときだけ
          abort する。
      BY CODE src/rc_ir/borrow.rs: check_one_key_per_object
    <3>2. `R` の `outstanding` は `{(m, []): 1}` である。
      BY CODE src/rc_ir/ownership.rs: acted_references, 4.3 の <1>1
    <3>3. `L1` の `un_bumped` は `{(m, []): 1, (w, []): 1}` である。
      BY 4.4 の <1>1, 4.3 の <1>2 -- leaf `[1, 1]` の identity は `(w, [])`。
    <3>4. QED
      BY <3>1, <3>2, <3>3 -- 2 つは等しくない。
  <2>5. QED
    BY <2>2, <2>3, <2>4 -- `R` の鍵 `(m, [])` はどの `consume_unit` の引数でもない。

<1>4. `let k = struct_get_0(m)` は `R` に印を付けない。
  BY 4.2 の <1>1 の <2>3 (`borrows_operand(0)` が真),
     CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕の `continue`

<1>5. `L2` は `R` と対になる。
  <2>1. `L2` の鍵は `(m, [])` である。
    BY 4.3 の <1>1, CODE src/rc_ir/ownership.rs: unit_key, unit_of
  <2>2. `L2` の `un_bumped` は `{(m, []): 1}` であり、`R` の `outstanding` と等しい。
    BY CODE src/rc_ir/ownership.rs: acted_references, 4.3 の <1>1, <1>3 の <2>4 の <3>2
  <2>3. QED
    BY <2>1, <2>2, <1>1, CODE src/rc_ir/borrow.rs: un_bump -- `covers` が成り立つので `InBracket(R)`。

<1>6. `cancel` は `R` と `L2` を削除する。
  BY <1>2, <1>3, <1>4, <1>5 -- `R` は `needed_retains` に入らず、`un_bump_releases[R]` は空でない。
     CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled, drop_nodes

<1>7. 出力は D11 の (S-c) を破る。
  <2>1. 削除の後、`L1` の位置で `obj(m, [])` への未処分の参照は 1 つであり、`L1` がそれを処分する。
    BY <1>6, 4.2 の <1>4 の <2>5 -- 作られる 2 つのうち `R` が作る方が消える。
  <2>2. `H(obj(m, [])) = 0` になるので `obj(m, [])` は解放される。
    BY <2>1, D7, D8
  <2>3. `let k = struct_get_0(m)` は D7 の読む構文であり、オペランド `m` の inhabited な leaf `[]` が指す
        オブジェクト、すなわち `obj(m, [])` を読みうる。
    BY D7 の読む構文の表の `Let(x, Llvm(gen, args), k)` の行
  <2>4. QED
    BY <2>2, <2>3 -- 解放後の読みである。

### 4.6 P4 との関係、および証明のどこで止まるか

P4 の後半は「その参照が同一である 2 つの実行路の位置において、`identity` は同じ `VarPath` である」であり、
`identity` は `(x, π)` の静的な関数なので、1 つのスロットについてはこれは自動的に成り立つ。すなわち
**P4 は (N) を含意しない。** (N) が言うのは、`(u, [])` を名指す操作と `(m, [])` を名指す操作が同じ参照に
触れるとき、前者の答えが後者の名前を挙げる、という 2 つのスロットにまたがる主張である。

したがって止まり方は次のとおりである。P4 は書かれたとおりならこの反例を通す。しかし P19 から P21
(`cancel` の健全性) は (N) が要り、4.4 が (N) の反例、4.5 がその帰結の miscompilation である。P4 を
(N) の形に強めれば、その強めた P4 は偽であり、原因はコードにある。README の第 6 節 (較正) が #519 について
述べていることが、そのままこの反例にも当てはまる。

止まる場所をコードで名指すと、`origin_from_leaves_under` の末尾の 2 行である
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。

```rust
let candidates = reached
    .iter()
    .flat_map(|reached_origin| reached_origin.candidates())
```

`reached` の要素が `Join` のとき、その `identity` はここで落ちる。`8fb0dd79` が直したのは、その 1 つ上の
「`reached` の全要素が等しいとき」の枝である。全要素が等しくない枝は直っていない。

## 5. 発見 2 -- P3 と P4 の「対応する leaf」が E4 に合わない

P3 と P4 の「対応する」は「`λ` から `π` の接頭辞を除いた残りを `σ` の後ろに繋いだ path」である。この規則は
E1、E3、E5、E6、E7 では正しい (どれも path の接頭辞を書き換えるだけである) が、**E4 では成り立たない。**

反例は 4.2 と同じ型で、payload の 2 つの unit が同じ origin を持つようにしたものである。

```
let pair = struct_make(m, m);       // 宣言は Arg(0, []) と Arg(1, [])
let u    = union_make_1(pair);      // 宣言は Arg(0, [0]) と Arg(0, [1])
```

<1>1. `m` が `Binding::Producer` のとき `origin(u, []) = Exactly((m, []))`。
  <2>1. `origin(pair, [0]) = origin(pair, [1]) = Exactly((m, []))`。
    BY 4.3 の <1>2 と同じ計算 (両オペランドが `m`),
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Producer)` の腕
  <2>2. `operand_units = {(0, [0]), (0, [1])}`、`produced_here` は偽。
    BY 4.3 の <1>3 の <2>2 と同じ計算
  <2>3. `reached` の 2 要素は等しいので、答えは `first.clone()` である。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の
       `if reached.iter().all(..) { return Some(first.clone()) }`
  <2>4. QED
    BY <2>3, <2>1

<1>2. P3 の「対応する leaf」はこの答えについて存在しない。
  <2>1. `π = []`、`σ = []`、`u` の inhabited な leaf は `[1, 0]` と `[1, 1]` である。
    BY 4.2 の <1>4 の <2>2 の <3>1, D16
  <2>2. 規則が指す path は `[1, 0]` と `[1, 1]` である (`π` が空なので残りは `λ` 自身、`σ` が空なので
        繋いでも `λ` のまま)。
    BY <2>1, P3 の「対応する」の定義
  <2>3. `[1, 0]` は `ty(m) = Node` の boxed leaf ではない。`Node` は boxed なので leaf は `[]` 1 つで
        ある。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths の `is_box` の分岐
  <2>4. QED
    BY <2>2, <2>3, D6 -- 対応する path はスロットを指さない。

<1>3. 正しい対応は宣言が与えるものである。
  BY 4.4 の <1>2 の <2>2 と同じ計算 -- `u` の leaf `[1, 0]` の宣言は `Arg(0, [0])` であり、
     `origin(pair, [0])` を経て `(m, [])` に着く。A3 の「単一の `Arg(j, σ)`」の行より、この leaf が持つ
     参照は `pair` の leaf `[0]` の参照と同一であり、それは `m` の leaf `[]` の参照と同一である。

読みの向き (`struct_get`、`union_as`) では宣言が `Arg(0, [component] ++ path)` の形なので、接頭辞を繋ぐ
規則と一致する。構築の向き (`struct_make`、`union_make`) では宣言が接頭辞を**外す**形なので一致しない。
P3 と P4 の「対応する」は、path の連結ではなく宣言の辿り着く先で定義する必要がある。第 7 節に文面案を書く。

## 6. 同じ形の 2 つ目の場所 -- `Binding::Join` の腕

`origin_inner` の `Binding::Join` の腕も、アームの結果の `candidates()` を合併して `of_candidates(C, here)`
を呼ぶ (`CODE src/rc_ir/ownership.rs: origin_inner` の `Some(Binding::Join(..))` の腕)。よってアームの結果の
origin が `Join` のとき、その `identity` は答えのどこにも残らない。`origin_from_leaves_under` が
`8fb0dd79` の前に持っていた形と同じであり、4.6 で名指した 2 行と同じである。

この腕で (N) が破れるには、次の 3 つが同時に要る。

1. あるアームの結果の origin が `Join` であること。
2. その `Join` の `identity` を鍵とする `Retain` が、match の前から pending であること。
3. その `Retain` が `merge` で `needed_retains` に入らないこと。

3 のために、すべてのアームの出口で `outstanding` が一致していなければならない
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::merge`)。アームのどれかがその値を消費するか release すると、
`consume_unit` か `merge` が印を付ける。よって破れるのは、すべてのアームの結果が 1 つの値の別名であるときに
限られる。その形の RC IR を `insert_rc` が作るかどうかは確かめていない。第 4 節の反例は分岐を要さないので、
直すべき箇所としてはそちらが先に立つ。

## 7. README への要望

- **P3 と P4 の「対応する leaf」の定義**。第 5 節より、path の連結では E4 に合わない。次の形を提案する。
  「`π` の下の inhabited な leaf `λ` に**対応するスロット**とは、`origin_inner` が `(x, π)` から `(u, σ)` へ
  辿った辺の列を `λ` について辿ったときに着く leaf のスロットをいう。E1 と E6 は `λ` を変えず、E5 と E7 は
  先頭に添字を足し、E3 は宣言 `Arg(j, σ')` の `σ'` へ置き換え、E4 は `λ` の宣言 `Arg(j, σ')` の `σ'` へ
  置き換える。」この定義なら、第 5 節の反例で `u` の leaf `[1, 0]` は `(m, [])` に対応する。
- **(N) を命題として立てること**。第 4 節より、P19 から P21 が要るのは P4 の後半ではなく (N) である。
  (N) は `origin` と `acted_references` と `acted_unit_keys` だけで書けるので、実行路を量化せずに述べられる。
  現在のコードでは偽なので、立てた上で「コードを直すまで閉じない」と記録するのが正確である。
- **A3 の複数元の行の到達可能性**。第 2 節の (b) より、複数元の宣言を持つ op は現在存在しない。A3 が
  複数元の行を持つのは仮定として正しいが、`origin_from_leaves_under` はその行の意味 (いずれの路でも
  新しい参照) と食い違う扱いをする。A3 の下では、この食い違いは「宣言が実在しないので発火しない」という
  形でしか埋まらない。命題としてどこかに書き留めるなら P2 か P4 の付帯事項になる。
