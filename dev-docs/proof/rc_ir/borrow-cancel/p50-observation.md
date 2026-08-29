# P26 -- 一意性は悪化しない (反証)

この文書は `README.md` の P26 を扱う。立つのは `README.md` の定義 D1-D20、仮定 A1-A15、および命題 P1-P24 の
**言明**である。

## 0. 結論

**P26 は `borrow_ify` について偽である。** 反例を第 4 節に挙げ、`borrow_ify` がその入力に対して何を出力するかを
第 5 節で導出し、2 つの実行の参照カウントを第 6 節で数える。入力の実行で観測値が真である観測点があり、出力の
対応する観測点 (D19) の観測値は偽である。

`cancel` の側は着手していない。証明の途中でコードの欠陥に着き当たったときは、そこで止めて報告する規則に従う。

| 命題 | 結果 |
|---|---|
| P26 (`borrow_ify` について) | **偽。反例あり** |
| P26 (`cancel` について) | 未着手 |

反例が破るのは P26 だけであり、同じ出力は D11 を満たす (第 8 節)。すなわちこれは、README 第 6 節が
「まだ持っていない」と書いた、**P26 の節だけを破るバグ**である。

## 1. 記法

1 つの関数の本体を固定し、その本体から作られる `VarTable` を `vars`、プログラムの `TypeEnv` を `type_env` と
書く。`origin(x, π)` は `origin(vars, type_env, x, π)` の略である。

- `H(o)` はオブジェクト `o` の参照カウント (D7)。
- `Obl(F, o)` は活性化 `F` の義務集合 (D10) が持つ `o` への参照の個数。
- 型 `B` は `box struct { v : I64 }` とする。`B` は `is_box` が真なので、`boxed_leaf_paths(B) = [[]]` であり
  `rc_units(B) = [[]]` である (D4 の規則 3、D5 の `unit_step` の `is_box` の行)。以下、`B` 型の値の path は
  すべて空列 `[]` である。

## 2. 反証すべき言明

P26 の言明は次である。

> 入力の一意性の観測点 (D18) で観測値が真であるすべての実行について、出力の対応する観測点 (D19) の観測値も
> 真である。

その否定は次である。これを第 4 節から第 6 節で示す。

> 入力プログラム `P` と、`P` のある実行 `ρ` と、`ρ` 上のある一意性の観測点があって、そこでの観測値は真であり、
> かつ `borrow_ify(P)` の対応する実行の対応する観測点 (D19) での観測値は偽である。

`P` は A1 と A2 を満たす。すなわち `P` は D12 の意味で RC 規律を満たし、`P` のすべての関数の `borrowed_units`
は空である。

## 3. 観測点が読む量

D18 は観測点を `unsafe_is_unique` の演算が現れる位置と定め、観測値をその演算が返す `Bool` と定める。この節は、
その `Bool` が実行時に何を読むかを、D18 が名指す記号から辿って固定する。以下 L0 と呼ぶ。

## L0 (観測値は `H(o) = 1` である)

**言明**。`Llvm(gen, [a])` が `unsafe_is_unique` の演算であり、`a` の leaf `[]` が指すオブジェクトが `o` で
あるとき、その位置での観測値は、`o` の参照カウント状態が local か threaded ならば `H(o) = 1` と同値であり、
global ならば偽である。

<1>1. `Std::unsafe_is_unique` の本体は、`InlineLLVMIsUniqueFunctionBody` を 1 つ持つ `Llvm` 式である。
  `make_std_mod` は `FullName::from_strs(&[STD_NAME], "unsafe_is_unique")` に `is_unique_function()` を
  登録し、`is_unique_function` は `expr_abs(vec![var_local("x")], expr_llvm(Box::new(
  InlineLLVMIsUniqueFunctionBody { assume_local: false, var_name: FullName::local("x"),
  assume_unique: false }), ret_type, None), None)` を返す。
  BY CODE src/fixstd/stdlib.rs: make_std_mod, CODE src/fixstd/builtin.rs: is_unique_function

<1>2. `assume_unique` が偽のとき、`InlineLLVMIsUniqueFunctionBody::generate` が返す組の第 0 成分は、
      `gc.build_branch_by_is_unique(obj_ptr, assumed_state(self.assume_local))` が返す `unique_bb` から
      来る辺で `1`、`shared_bb` から来る辺で `0` となる phi である。ここで `obj_ptr` はオペランドの値である。
  BY CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::generate

<1>3. `build_branch_by_is_unique(obj_ptr, state)` は、`build_branch_by_refcnt_state` が返す 3 つの
      基本ブロックのそれぞれについて次のように分岐する。local のとき
      `build_is_refcnt_one(obj_ptr, false, "")` の真偽で `unique_bb` と `shared_bb` へ、threaded のとき
      `build_is_refcnt_one(obj_ptr, true, "")` の真偽で `unique_threaded_bb` (そこから `unique_bb`) と
      `shared_bb` へ、global のとき `shared_bb` へ無条件に。
  BY CODE src/generator.rs: Generator::build_branch_by_is_unique

<1>4. QED
  `<1>1` より観測点の演算は `InlineLLVMIsUniqueFunctionBody` であり、`<1>2` よりその `Bool` は
  `build_branch_by_is_unique` の 2 つの出口のどちらを通ったかで決まる。`<1>3` の 3 つの状態が
  `build_branch_by_refcnt_state` が返す全体であり、local と threaded では
  `build_is_refcnt_one` すなわちカウントが 1 であるかが選択を決め、global では `shared_bb` に決まる。
  D7 より `H(o)` はその参照カウントである。
  BY <1>1, <1>2, <1>3, D7

**観測値が振る舞いを変えること。** `Debug::assert_unique` は `let (unique, x) = x.unsafe_is_unique;
if !unique { undefined(...) }; x` であり、観測値が偽のときプログラムを止める。
`Destructor::mutate_unique_io` は `let (unique, dtor) = dtor.unsafe_is_unique;` の後、真のときは資源を
そのまま使い、偽のときは `ctor` を走らせて資源を複製する
(`CODE src/fixstd/std.fix: assert_unique`, `mutate_unique_io`)。

## 4. 反例

### 4.1 Fix プログラム

```fix
module Main;

type B = box struct { v : I64 };

// 再帰があるのでインライン化されない。
// `x` と `w` は本体で使われないので、借用推論はこの 2 つを借用に倒す。
// `y` は `unsafe_is_unique` が消費するので所有のまま残る。
observe : I64 -> B -> B -> B -> Bool;
observe = |n, x, y, w| (
    if n > 0 { observe(n - 1, x, y, w) };
    let (u, y) = y.unsafe_is_unique;
    eval y;
    u
);

main : IO ();
main = (
    let o = B { v : 1 };
    let w = B { v : 2 };
    let u = observe(0, o, o, w);   // 同じオブジェクトを第 1・第 2 引数に渡す
    eval w.@v;                     // `w` を呼び出しの後で使う
    println("unique = " + u.to_string)
);
```

このプログラムの `unsafe_is_unique` が返す値は、`-O none` と `-O basic` で `true`、`-O max` で `false` で
ある (第 7 節)。

3 つの成分がそれぞれ役目を持つ。

- **`x`**: 本体で使われないので借用に倒れ、その `Release` が借用版で落ちる。
- **`y`**: `unsafe_is_unique` が消費するので所有のまま残り、観測点のオペランドになる。実行時には `x` と同じ
  オブジェクトを指す。
- **`w`**: `main` が呼び出しの後で使うので、`routing_saves_retain` が借用版への振り分けを許す
  (第 5 節の L2 の `<1>4`)。`w` を落とすと振り分けが起きず、観測値は両方の水準で `true` になる (第 7 節)。

### 4.2 入力の RC IR

`split_rc_units` の出力、すなわち `borrow_ify` の入力にあたる 2 つの本体を書く。名前は短くしてある。

```
fn main(...):
  Let(o, Llvm(struct_make, [c1]),            // o : B
  Let(w, Llvm(struct_make, [c2]),            // w : B
  Let(n, Llvm(int, []),                      // n : I64 = 0
  Retain(w, [])
  Retain(o, [])
  Let(u, App(observe, [n, o, o, w]),
  Let(t, Llvm(struct_get_0, [w]),            // struct_get_0 は容器を借用する
  Release(w, [])
  Eval(t, ... println ...))))))

fn observe(n, x, y, w) -> Bool:
  Let(c, Llvm(int_lt, [zero, n]),
  Let(r, Match(c, [arm1, arm0]),
  Ret(r)))

  arm1 (n > 0):
    Let(n2, Llvm(int_sub, [n, one]),
    Let(a, App(observe, [n2, x, y, w]),
    Ret(a)))

  arm0 (n <= 0):
    Release(x, [])
    Release(w, [])
    Let(p, Llvm(is_unique, [y]),             // <- 観測点
    Destructure(p, [(0, u), (1, y2)],
    Eval(y2,
    Release(y2, [])
    Ret(u))))
```

この本体が A1 を満たすこと (D12 の意味で RC 規律を満たし、`borrowed_units` が空であること) は A1 が仮定する。
A2 (すべての `Retain`/`Release` の path が `rc_units` の要素であること) は、`B` の unit が `[]` だけであり、
上のすべての path が `[]` であることから成り立つ。

`Llvm(struct_get_0, [w])` の `borrows_operand(0)` は、取り出すフィールドの型が fully unboxed のとき真で
ある。`B.@v : I64` は fully unboxed なので真であり、この演算は `w` を消費しない
(`CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_operand`)。

## 5. `borrow_ify` がこの入力に何を出力するか

以下、`observe` の第 1・第 2・第 3・第 4 パラメータをそれぞれ `n`、`x`、`y`、`w` と書く。

## L1 (推論の不動点は `y` だけを所有する)

**言明**。`infer_ownership` が返す `OwnedLeaves` は、`observe` のパラメータについては `(y, [])` だけを含み、
`(x, [])` と `(w, [])` を含まない。

<1>1. `observe` の本体が D9 の意味で消費する leaf を `collect_consumes` は次のように報告する。
  <2>1. `arm0` の `Llvm(is_unique, [y])` について、`rhs_consumes` の `RcRhs::Llvm` の腕は、
        `borrows_operand(i)` が偽で `passthrough_arg_leaves` に入らない各オペランド leaf を報告する。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>2. `InlineLLVMIsUniqueFunctionBody` は `borrows_operand` を override しないので、既定の `false` を
        返す。
    BY CODE src/fixstd/builtin.rs: `impl LLVMGen for InlineLLVMIsUniqueFunctionBody`,
       CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand
  <2>3. `InlineLLVMIsUniqueFunctionBody::result_prov` は
        `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` を返す。`passthrough_arg_leaves` は
        結果の leaf の宣言が単一の `Arg(j, σ)` であるものだけを集めるので、この op については空である。
    BY CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov,
       CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection
  <2>4. よって `Llvm(is_unique, [y])` は `(y, [])` を報告する。
    BY <2>1, <2>2, <2>3
  <2>5. `arm0` の `Release(x, [])` と `Release(w, [])` は何も報告しない。`collect_consumes_go` の
        `RcExpr::Retain | RcExpr::Release | RcExpr::Eval` の腕は継続へ降りるだけである。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Retain(..) | RcExpr::Release(..) |
       RcExpr::Eval(..)` の腕
  <2>6. `arm0` の `Destructure(p, [(0, u), (1, y2)], ...)` は何も報告しない。`p` の型は unbox のタプル
        `(Bool, B)` であり、`destructure_consumes` は unbox 容器について、名前が付いていないフィールドの
        leaf だけを返す。両フィールドに名前が付いているので空である。
    BY CODE src/rc_ir/ownership.rs: destructure_consumes
  <2>7. `arm0` の `Ret(u)`、`arm1` の `Ret(a)`、本体の `Ret(r)` は何も報告しない。
        `collect_consumes_go` の `RcExpr::Ret(x)` の腕は `push_boxed_leaves(&x.name, &x.ty, ...)` を呼び、
        `u`、`a`、`r` の型はいずれも `Std::Bool` で、`boxed_leaf_paths` は空である。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Ret` の腕, D4
  <2>7a. `Llvm(int_lt, [zero, n])` と `Llvm(int_sub, [n, one])` は何も報告しない。`rhs_consumes` の
        `RcRhs::Llvm` の腕はオペランドの `boxed_leaf_paths` を渡るが、`Std::I64` は
        `is_fully_unboxed` が真なので leaf を持たない。
    BY D4, CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>7b. `Let(r, Match(c, [arm1, arm0]), k)` の `Match` 自身は何も報告しない。`collect_consumes_go` の
        `RcRhs::Match(_, arms)` の場合は各アームの本体へ降りるだけである。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcRhs::Match` の場合
  <2>8. `arm1` の `App(observe, [n2, x, y, w])` が報告するもののうち、`observe` のパラメータ leaf を
        `owned_leaves` に入れうるのは、`resolve_callee_params` で解決した各位置について
        `owns(&params[i], &leaf)` が真である引数の leaf だけである。`rhs_consumes` の `RcRhs::App` の腕は
        これに先立って `push_boxed_leaves(&callee.name, &callee.ty, ...)` も報告するが、その主語は名前
        `observe` であり、`origin_inner` の `vars.bindings.get(var)` が `None` を返す腕により
        `Exactly((observe, ・))` になる。`observe` は `vars.param_tys` に無いので `infer_ownership` は
        これについて何も挿入しない。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App` の腕, resolve_callee_params,
       origin_inner の `None` の腕, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>9. QED
    `<2>4`、`<2>5`、`<2>6`、`<2>7`、`<2>7a`、`<2>7b`、`<2>8` が `observe` の本体のすべての節点を尽くす。
    `main` の本体については `<1>2` で扱う。
    BY <2>4, <2>5, <2>6, <2>7, <2>7a, <2>7b, <2>8

<1>2. `infer_ownership` の不動点で `owned_leaves` に入る `observe` のパラメータ leaf は `(y, [])` だけで
      ある。
  <2>1. `owned_leaves` の初期値は空である。
    BY CODE src/rc_ir/borrow.rs: infer_ownership
  <2>2. 1 周目、`<1>1` の `<2>4` が `(y, [])` を報告し、`origin(y, [])` は `Exactly((y, []))` である
        (`y` は `Binding::Param`)。`y` は `vars.param_tys` にあるので `(y, [])` が挿入される。
    BY <1>1, CODE src/rc_ir/borrow.rs: infer_ownership, CODE src/rc_ir/ownership.rs: origin_inner の
       `Binding::Param` の腕
  <2>3. `<1>1` の `<2>8` の `App(observe, [n2, x, y, w])` は、`owns` が真である位置の引数だけを報告する。
        `owns` は `owned_leaves` を読むので、`(x, [])` と `(w, [])` が `owned_leaves` に入っていない限り
        位置 1 と位置 3 は報告されない。位置 2 は `(y, [])` が入っているので `(y, [])` を報告し、これは
        すでに入っている。
    BY <1>1, <2>2, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>4. `main` の本体を歩いた分は `observe` のパラメータ leaf を `owned_leaves` に入れない。
        `infer_ownership` が `owned_leaves` に入れるのは `(root_var, root_path)` のうち
        `vars.param_tys.contains_key(root_var)` を満たすものだけであり、ここでの `vars` は `main` の
        `VarTable` である。`main` のパラメータは `Std::IO::IOState` 型の 1 つだけなので、`observe` の
        パラメータ名はその `param_tys` に無い。
    BY CODE src/rc_ir/borrow.rs: infer_ownership, CODE src/rc_ir/ownership.rs: VarTable::of
  <2>5. `level_ownership` は `(x, [])` と `(w, [])` を挿入しない。`levelled_sites` が挙げる site のうち
        `x` を主語にするのは `arm0` の `Release(x, [])` と `arm1` の `App` の位置 1 であり、どちらも
        `origin(x, [])` の候補は `{(x, [])}` である。`owns_object_yet(vars, type_env, x, [], owned_leaves)`
        は、`x` が `param_tys` にあるので `units_under(B, [], type_env) = [[]]` の各 unit について
        `truncate_to_unit(B, [], type_env) = []` を鍵として `(x, []) ∈ owned_leaves` を問い、`<2>2` と
        `<2>3` よりこれは偽である。よって `owns_a_candidate` が偽になり、`level_ownership` は `false` を
        返して何も挿入しない。`w` についても同じ形の site で同じ答えになる。
    BY <2>2, <2>3, CODE src/rc_ir/borrow.rs: levelled_sites, level_ownership, owns_object_yet
  <2>6. QED
    `<2>1` から `<2>5` より、不動点に達したときの `owned_leaves` は `observe` のパラメータについて
    `(y, [])` だけを持つ。
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>3. QED
  BY <1>2

## L2 (`main` の呼び出しは借用版へ振り分けられる)

**言明**。`borrow_ify` は `observe#borrow` を作り、`main` の `App(observe, [n, o, o, w])` をその借用版へ
振り分け、呼び出しの後に `Release(o, [])` と `Release(w, [])` を置く。

<1>1. `borrow_versions` は `observe` を含む。`observe` は `capture` を持たず、
      `func_has_borrowable_param(observe, owned_leaves, type_env)` は、`x` の leaf `[]` が
      `owned_leaves.owns(x, [])` を偽にするので真である。
  BY L1, CODE src/rc_ir/borrow.rs: borrow_ify, func_has_borrowable_param

<1>2. `owned_units` は、`observe` の原本について `(n, ・)` を除く全 unit すなわち `(x, [])`、`(y, [])`、
      `(w, [])` を含み、借用版 `observe#borrow` については `(y', [])` だけを含む。ここで `y'` は
      `clone_func` が `y` に付けた新しい名前である。
  `borrow_ify` は各関数について `owned_units.extend(param_capture_units(func, type_env))` を行い、
  借用版については `owned_leaves.owns(&p.name, &leaf)` が真である leaf だけを `truncate_to_unit` で
  unit にして入れる。`I64` は `rc_units` が空なので `n` の unit は無い。
  BY L1, <1>1, CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units

<1>3. `routing_is_safe(u, [n, o, o, w])` は真である。`main` の本体は呼び出しの後に
      `Llvm(struct_get_0, [w])` を持つので、`trivially_returns(k, u)` は `RcExpr::Let(..)` の腕で偽になり、
      `u` は `tail_result_vars(main.body)` に入らない。
  BY CODE src/rc_ir/borrow.rs: routing_is_safe, tail_result_vars, mark_tail, trivially_returns

<1>4. `routing_saves_retain(observe#borrow, [n, o, o, w], k)` は真である。
  <2>1. 引数 3 (`w`) の unit `[]` について `callee_borrows` は真である。
        `!self.owned_units.contains(&(borrow_params[3].0.clone(), []))` であり、`<1>2` より借用版の
        `owned_units` は `w` の複製名を含まない。
    BY <1>2, CODE src/rc_ir/borrow.rs: routing_saves_retain
  <2>2. `arg_used_later = used_later(&w.name, k)` は真である。`k` は呼び出しの継続であり、その中の
        `Let(t, Llvm(struct_get_0, [w]), ...)` について `used_later` の `RcExpr::Let(_, rhs, k)` の腕が
        `rhs_uses(name, rhs)` を真にする。
    BY CODE src/rc_ir/borrow.rs: used_later
  <2>3. QED
        `<2>2` より `!arg_used_later` は偽であり、`self.owns_unit(arg, unit) && !arg_used_later && ...` は
        偽になる。その否定は真なので、`<2>1` と合わせて引数 3 の unit `[]` が
        `callee_borrows && !(...)` を真にし、`any` が真を返す。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: routing_saves_retain

<1>5. `route(u, observe, [n, o, o, w], k)` は `observe#borrow` を返す。
  BY <1>1, <1>3, <1>4, CODE src/rc_ir/borrow.rs: route

<1>6. `call_rc(observe#borrow, [n, o, o, w])` は `before = []`、`after = [(o, []), (w, [])]` を返す。
  <2>1. 引数 0 (`n : I64`) は `rc_units(I64, type_env)` が空なので、どちらにも入らない。
    BY D5, CODE src/rc_ir/borrow.rs: call_rc
  <2>2. `main` はパラメータとして `o` も `w` も取らないので、`owns_object` の
        `self.vars.param_tys.get(root)` は `None` を返し、`owns_unit(o, [])` と `owns_unit(w, [])` は
        ともに真である。
    BY CODE src/rc_ir/borrow.rs: owns_unit, owns_object
  <2>3. 引数 1 (`o`、位置は `x`) について `callee_owns` は偽である (`<1>2`)。`<2>2` より `arg_owned` は
        真なので、`!callee_owns && arg_owned` の枝が取られ `after` に `(o, [])` が入る。
    BY <1>2, <2>2, CODE src/rc_ir/borrow.rs: call_rc
  <2>4. 引数 2 (`o`、位置は `y`) について `callee_owns` は真である (`<1>2`)。`<2>2` より `arg_owned` も
        真なので、どちらの枝も取られない。
    BY <1>2, <2>2, CODE src/rc_ir/borrow.rs: call_rc
  <2>5. 引数 3 (`w`、位置は `w`) について `callee_owns` は偽 (`<1>2`)、`arg_owned` は真 (`<2>2`) なので
        `after` に `(w, [])` が入る。
    BY <1>2, <2>2, CODE src/rc_ir/borrow.rs: call_rc
  <2>6. QED
    BY <2>1, <2>3, <2>4, <2>5

<1>7. QED
  `rewrite_inner` の `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕は、`route` の答えを callee にし、
  `call_rc` の `after` を `prepend_rc(after, true, ...)` で継続の先頭に置き、`before` を
  `prepend_rc(before, false, ...)` で呼び出しの前に置く。`<1>5` と `<1>6` より、`main` の出力は
  `Retain(w, []) ; Retain(o, []) ; Let(u, App(observe#borrow, [n, o, o, w]), Release(o, []) ;
  Release(w, []) ; k')` である。
  BY <1>5, <1>6, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, prepend_rc

## L3 (借用版は `Release(x, [])` を落とす)

**言明**。`observe#borrow` の本体の `arm0` には `Release(x', [])` も `Release(w', [])` も無く、
`Release(y2', [])` はある。ここで `'` は `clone_func` の付けた名前を表す。

<1>1. `observe#borrow` の本体は `observe` の本体の束縛変数を一斉に付け替えたものであり、それ以外の違いを
      持たない。
  BY P9

<1>2. `RewriteCtx::rewrite_rc` は、`is_borrow_version` が真のとき、`units_under(&v.ty, path, type_env)` の
      うち `self.owns_unit(v, unit)` が真である unit だけを節点として残す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>3. `owns_unit(x', [])` は偽である。
  `origin(x', [])` は `x'` が `Binding::Param` なので `Exactly((x', []))` を返し、候補は `{(x', [])}` で
  ある。`owns_object(x', [])` は `param_tys` に `x'` があるので
  `units_under(B, [], type_env) = [[]]` の各 unit について
  `owned_units.contains(&(x', truncate_to_unit(B, [], type_env)))` すなわち
  `owned_units.contains(&(x', []))` を問い、L2 の `<1>2` よりこれは偽である。
  BY L2, CODE src/rc_ir/ownership.rs: origin_inner の `Binding::Param` の腕,
     CODE src/rc_ir/borrow.rs: owns_unit, owns_object

<1>4. `owns_unit(w', [])` は偽である。理由は `<1>3` と同じ形で、`owned_units.contains(&(w', []))` が
      L2 の `<1>2` より偽である。
  BY L2, CODE src/rc_ir/ownership.rs: origin_inner の `Binding::Param` の腕,
     CODE src/rc_ir/borrow.rs: owns_unit, owns_object

<1>5. `owns_unit(y2', [])` は真である。
  <2>1. `y2'` は `Destructure(p', [(0, u'), (1, y2')], ...)` が束縛するので `Binding::Field(p', 1)` を
        持つ。`p'` の型 `(Std::Bool, B)` は unbox のタプルなので `is_box` は偽であり、`origin_inner` の
        `Binding::Field` の腕は `origin(p', [1])` へ帰着する。
    BY CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner の `Binding::Field` の腕
  <2>2. `origin(p', [1])` は `Exactly((p', [1]))` である。`p'` は `Binding::Llvm(is_unique, [y'],
        (Std::Bool, B))` を持つ。`InlineLLVMIsUniqueFunctionBody::result_prov` は
        `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` を返すので、
        `decl.leaf_origins_at([1])` は `{Unknown}` であり `as_arg_projection` は `None` を返す。よって
        `origin_from_leaves_under(..., path = [1], here = (p', [1]))` に入り、そこでは `[1]` の下の唯一の
        leaf の源が `Unknown` なので `operand_units` は空、`produced_here` は真、`reached` は
        `[Exactly((p', [1]))]` の 1 元になり、その 1 元がそのまま返る。
    BY CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov,
       CODE src/rc_ir/ownership.rs: origin_inner の `Binding::Llvm` の腕, as_arg_projection,
       origin_from_leaves_under
  <2>3. QED
    `<2>1` と `<2>2` より `origin(y2', [])` の候補は `{(p', [1])}` である。`p'` は `param_tys` に無いので
    `owns_object(p', [1])` は `None` の枝で真を返す。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: owns_unit, owns_object

<1>6. QED
  `<1>2` と `<1>3` より `Release(x', [])` の `kept` は空になり、節点は継続に置き換わる。`<1>4` より
  `Release(w', [])` も同じである。`<1>5` より `Release(y2', [])` は残る。`<1>1` より本体の他の部分は
  原本と同じ形である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5

## 6. 2 つの実行の参照カウント

以下、`main` の 1 回の活性化を `M`、`observe` (または `observe#borrow`) の 1 回の活性化を `F` と書く。
`n = 0` なので `Match` は `arm0` を選ぶ。`o` が指すオブジェクトを `o*` と書く。

## L4 (入力の実行では観測値が真である)

**言明**。入力プログラムの実行において、観測点 `Llvm(is_unique, [y])` の位置で `H(o*) = 1` である。

<1>1. `Llvm(struct_make, [c1])` の結果の leaf `[]` は新しい参照を 1 つ作り、`H(o*)` は 1 から始まる。
      `M` の `Obl` に参照が 1 つ入る。`InlineLLVMMakeStructBody::result_prov` は、boxed な結果の根の path に
      `sole_origin(LeafOrigin::Fresh)` を置く。`B` は `is_box` が真なのでその leaf は根の path `[]` だけで
      あり (D4 の規則 3)、宣言は単一の `Arg(j, σ)` ではない。D10 の生成の表の `Llvm` の行がこれに当たり、
      A3 の表の `Fresh` の行が「新しく割り当てたオブジェクトへの新しい参照」と述べる。
  BY A3, D4, D10, CODE src/fixstd/builtin.rs: InlineLLVMMakeStructBody::result_prov

<1>2. `Retain(o, [])` の後、`H(o*) = 2` であり、`M` の `Obl` は `o*` への参照を 2 つ持つ。
  BY D10, <1>1

<1>3. `App(observe, [n, o, o, w])` は、A1 より `observe` が全パラメータを所有するので、位置 1 と位置 2 の
      引数 `o` の leaf `[]` を消費する。消費は `H` を変えないので `H(o*) = 2` のままであり、`M` の `Obl` は
      `o*` について空になり、`F` の初期 `Obl` は `x` の leaf と `y` の leaf の分で `o*` への参照を 2 つ持つ。
  BY A1, D9, D10, <1>2

<1>4. `arm0` の `Release(x, [])` の後、`H(o*) = 1` である。
  BY D10, <1>3

<1>4a. `o*` の参照カウント状態は global ではない。A8 より global の状態を持つのはグローバル値が到達する
      オブジェクトであり、`o*` は `main` の `Llvm(struct_make, [c1])` が実行時に作ったオブジェクトで、
      どのグローバル値からも到達されない。
  BY A8

<1>5. QED
  `<1>4` の直後の節点が観測点である。`<1>4a` と L0 より観測値は `H(o*) = 1` と同値であり、`<1>4` より
  それは真である。
  BY L0, <1>4, <1>4a

## L5 (出力の実行では観測値が偽である)

**言明**。`borrow_ify` の出力の実行において、対応する観測点の位置で `H(o*) = 2` である。

<1>1. 観測点の対応。L3 の `<1>1` より `observe#borrow` の本体は `observe` の本体の名前替えであり、D19 より
      借用版の本体の観測点は複製元の本体の同じ位置の観測点に対応する。`main` の本体の他の節点は L2 の
      `<1>7` が述べる 2 つの `Release` の挿入と callee の付け替えだけを受ける。
  BY D19, L2, L3

<1>2. `Llvm(struct_make, [c1])` から `Retain(o, [])` までは入力と同じ節点であり、`H(o*) = 2` であって
      `M` の `Obl` は `o*` への参照を 2 つ持つ。
  BY L4, <1>1

<1>3. `App(observe#borrow, [n, o, o, w])` は、位置 2 の引数 `o` の leaf を消費し、位置 1 の引数 `o` の
      leaf を消費しない。L2 の `<1>2` より `owned_units` は `(y', [])` を含み `(x', [])` を含まないので、
      P13 より `observe#borrow` の `borrowed_units` は `(x', [])` を含み `(y', [])` を含まない。D14 が
      その集合で所有と借用を定め、D9 の `App` の行が消費される leaf をそれで決める。消費は `H` を変えない
      ので `H(o*) = 2` のままであり、`M` の `Obl` は `o*` への参照を 1 つ残し、`F` の初期 `Obl` は `y'` の
      分の 1 つだけを持つ (D10 の初期値は借用する unit の下の leaf を入れない)。
  BY D9, D10, D14, P13, L2, <1>2

<1>4. `arm0` に `Release(x', [])` は無いので、観測点の位置で `H(o*) = 2` である。
  BY L3, <1>3

<1>5. QED
  L4 の `<1>4a` と同じ理由で `o*` の状態は global ではない。L0 より観測値は `H(o*) = 1` と同値であり、
  `<1>4` よりそれは偽である。
  BY L0, L4, <1>4

## P26 は偽である

<1>1. 第 4.2 節の 2 つの本体からなるプログラム `P` は、A1 と A2 を満たす。A1 が仮定するのは入力が D12 を
      満たし `borrowed_units` が空であることで、これは `P` に対する A1 の適用そのものである。A2 は、
      `B` の unit が `[]` だけであり `P` のすべての `Retain`/`Release` の path が `[]` であることから
      成り立つ。
  BY A1, A2, 第 4.2 節

<1>2. `P` の実行の一意性の観測点で観測値は真である。
  BY L4

<1>3. `borrow_ify(P)` の対応する観測点で観測値は偽である。
  BY L5

<1>4. QED
  `<1>1`、`<1>2`、`<1>3` は第 2 節の否定形をそのまま満たす。
  BY <1>1, <1>2, <1>3

## 7. 実行による裏づけ

第 4.1 節のプログラムを走らせた結果である。

| 最適化水準 | 出力 |
|---|---|
| `-O none` | `unique = true` |
| `-O basic` | `unique = true` |
| `-O max` | `unique = false` |

`unsafe_is_unique` を `Debug::assert_unique` に置き換えた同じ形のプログラム (`observe` の本体を
`let y = y.assert_unique(|_| "..."); y.@v` にしたもの) は、`-O none` と `-O basic` では値を印字し、
`-O max` では signal 6 で停止する。

第 4.1 節から `w` を落として `observe : I64 -> B -> B -> Bool` にしたプログラムは、`-O none` でも
`-O max` でも `unique = true` を出す。L2 の `<1>4` が述べるとおり、`w` が無いと
`routing_saves_retain` が偽になり、呼び出しは原本のまま残る。

`--emit-rc-ir Main` が書く `rc_ir.Main.pre.txt` は `insert_rc` の直後の姿である。第 4.2 節はその写しで
あり、名前を短くし、`println` が展開する節点の並びを省いてある。`rc_ir.Main.post.txt` は
`optimize_rc_program` のすべてのパスを通った後の姿なので、L2 と L3 が導出した `borrow_ify` の出力とは
`cancel` の分だけ異なる。読める点は次の 3 つである。

- `observe#borrow` は
  `(#v0 : Std::I64 {u}, #v1 : Main::B {borrow}, #v2 : Main::B {own}, #v3 : Main::B {borrow})` と注釈され、
  L2 の `<1>2` の所有と借用の割り当てに一致する。
- その `arm0` は `release` を 1 つも持たずに `is_unique` に入る。L3 の結論に一致する。
- `main` は `retain o` の後に `observe#borrow(...)` を呼び、その直後に `release o` を持つ。L2 の `<1>7` の
  結論のうち `o` についての部分に一致する。`w` についての `Retain(w, [])` と呼び出し直後の
  `Release(w, [])` は post 側に無く、`cancel` が消したものである。`o` の側の対は、呼び出しが位置 2 で `o` を
  消費するので残る。

**使った二値の版。** この節の実行に使ったのは、対象コミット `a924f115` からビルドした二値ではない。使ったのは
`cb1fe26` からビルドされた `~/.cargo/bin/fix` と、`4d3a770` の作業ツリー (dirty) からビルドされた
`target/release/fix` の 2 つで、両方が同じ結果を出す。この 2 つの版と対象コミットの間で、L1-L3 が読む関数
(`infer_ownership`、`level_ownership`、`owns_object_yet`、`route`、`routing_is_safe`、
`routing_saves_retain`、`comes_from_a_value_used_later`、`owns_unit`、`owns_object`、`call_rc`、
`rewrite_rc`、`used_later`、`levelled_sites`、`func_has_borrowable_param`、`clone_func`、`borrow_ify`、
`collect_consumes`、`collect_consumes_go`、`rhs_consumes`、`destructure_consumes`、
`resolve_callee_params`、`passthrough_arg_leaves`、`origin`、`origin_inner`、`rc_units`、`unit_step`、
`truncate_to_unit`、`units_under`) の本文は、`cb1fe26` の `infer_ownership` を除いてすべて一致する
(`cb1fe26` の `infer_ownership` は `level_ownership` の周回を持たないが、L1 の `<2>5` が示すとおり
この入力では `level_ownership` は何も挿入しない)。第 5 節と第 6 節の導出は対象コミットのソースの上で行って
おり、この節はその裏づけである。

## 8. この欠陥が P26 の節を較正すること

README 第 6 節の表は、P26 の節を「未較正 -- この節だけを破るバグをまだ持っていない」と記録している。
第 4 節の反例はその較正になる。

- **P26 を破る**: L4 と L5 が示した。
- **D11 を保つ**: 出力の実行で `o*` の参照は、`M` が 1 つ (呼び出しの後の `Release(o, [])` が処分する)、
  `F` の `y'` が 1 つ (`Release(y2', [])` が処分する) であり、過剰処分 (S-a) も漏れ (S-b) も
  解放後の読み (S-c) も起きない。`-O max` でビルドした二値を valgrind の下で走らせると、エラー 0、
  definitely lost 0 バイトである。

すなわちこれは、D11 を満たしたまま P26 だけを破る欠陥であり、README 第 6 節が求めた「この節だけを破る例」で
ある。

## 9. 欠陥の一般形

第 4 節は 1 つの入力だが、破れる形は次の 3 つが揃うことである。

1. ある関数 `f` のパラメータ leaf `x` が、`f` の本体のどこでも D9 の意味で消費されない。`infer_ownership` は
   これを借用に倒す (`Release` は消費ではないので、`x` を捨てるだけの本体は借用に倒れる)。
2. `f` の本体の、入力での `Release(x)` より後ろに一意性の観測点があり、その観測点のオペランドが実行時に
   `x` と同じオブジェクトを指す。オペランドは `x` と別の名前を持つので、`origin` は 2 つを結ばない。
3. `f` を呼ぶ側で `routing_saves_retain` が真になる。これは引数についての `any` なので、条件 1 と 2 に
   関わらない別の引数 (第 4 節の `w`) が単独で満たしてよい。

この 3 つが揃うと、借用版は `Release(x)` を落とし、呼び出し元がその参照を呼び出しの後まで持つので、
観測点でのカウントが入力より 1 大きくなる。

`call_rc` が置く `Retain` の側も同じ差を作る。呼び出し元が借用し呼び出し先が所有する unit の前に置かれる
`Retain` は、呼び出しの間だけカウントを 1 上げるので、呼び出し先の中の観測点が同じように倒れる。この形は
`route` が借用版へ振り分けなかったとき (末尾位置、または `routing_saves_retain` が偽のとき) と、呼び出し先が
局所のクロージャ変数であるとき (`call_rc` の `self.callee_params.get(&FuncRef { name: callee.name.clone() })`
が `None` を返し、`callee_owns` がすべての位置で真になる) に現れる。第 4 節はこの形の実例を含んでいない。

## 10. 修正の候補

どれを採るかは言語の契約に関わる判断なので、ここでは候補と代償を並べる。

**候補 A -- 契約を広げる。** `unsafe_is_unique` の doc は「最適化が不要な計算を取り除き、共有されていた値が
一意になることがある」として共有から一意への変化だけを認めている。逆向きも認めるように doc を書き換え、
P26 を落とす。代償は `Debug::assert_unique` が最適化水準によって止まることを言語が認める点であり、
`Destructor::mutate_unique_io` が一意な資源を複製することを認める点である。

**候補 B -- 観測点を含む関数の借用版を作らない。** `unsafe_is_unique` の演算が到達可能な関数について
`func_has_borrowable_param` を偽にする。到達可能性は呼び出しグラフ上の閉包で取る。代償は、`Destructor` や
`Array` の一意性検査を通る経路の借用がすべて止まること。`std` の広い範囲がこれに当たる。

**候補 C -- 観測点より前にある `Release` を借用版でも残す。** `rewrite_rc` が落とす `Release` のうち、
同じ本体の後方に一意性の観測点があるものを残す。残した `Release` は借りている参照を処分するので、
呼び出し側の `after` の `Release` を消す必要があり、`call_rc` と `rewrite_rc` の間に新しい取り決めが要る。
代償は、`borrow_ify` の 2 つの半分が独立でなくなること。

**候補 D -- 何もしない。** 第 4 節の形が実プログラムに現れる頻度を測り、現れないなら P26 を落として
第 9 節の 3 条件を既知の穴として記録する。

## 11. この文書が示していないこと

- **`cancel` の側**。P26 が `cancel` について成り立つかどうかは調べていない。停止の規則に従い、
  `borrow_ify` の反例を得た時点で止めた。
- **反例の最小性**。第 9 節の 3 条件が必要であることは示していない。示したのは、その 3 条件を満たす入力が
  1 つあり、それが P26 を破ることである。
- **第 9 節の後半 (`call_rc` の `Retain` の側) の実例**。導出はコードの読みに立っており、実行による裏づけを
  持たない。

## 12. この文書が読んだコードの版

作業ツリーの版を読んだ。README の対象コミット `a924f115` との差分は、生成される `// PROOF:` の注釈行だけで
あり、この文書が引用する記号の本文は対象コミットと一致する。
