# P1 (leaf と unit の対応) と P2 (`origin` の全域性と停止性) の証明

対象コミットは `02805734f81a8ed842a614cd952494e954770aaa` である。

この文書が立つのは README の定義 D1、D2、D4、D5 と仮定 A3、A6、A9、A10、A11、A12、A15 の上である。証明は
1 本の構造化証明で、その QED が次の 3 つである。

- **P1** (leaf と unit の対応)。成り立つのは `<1>1` を満たす型についてである。README の P1 は
  「A10 を満たす任意の型 `τ` について」と書いており、`<1>1` は A10 に前提を 1 つ足したものである。
  第 3 節がこの差を述べる。
- **P2** (`origin` の全域性と停止性)。
- **P1 の系** (`<1>33`, `<1>34`)。`origin` の再帰が辿る path も、`origin` が返す `VarPath` の path も、
  どれも「その型の unit に届く」。すなわちそれらに `truncate_to_unit` を当てると abort せず、値は
  `rc_units` の要素である。これは「unit path の `origin` と、その下の leaf の `origin` の関係」に
  ついての要望への答えである。

P1 と P2 は共通の補題 (型の上の walk が停止すること、`unit_step` と `boxed_leaf_paths` の内部関数
`go` の分類) を使うので、その補題を先頭の `<1>` ステップに置き、P1 と P2 をその後ろに置く。

P1 は 2 つの静的な列挙 (`boxed_leaf_paths` と `rc_units`) の対応についての主張なので、D16 の
inhabited は現れない。実行時にどの leaf が参照を持つかは P1 の主張に入らない。

`<1>1`、`<1>2`、`<1>3a` は、README の A10、A11、A12 をこの文書の記法で述べたものである。README の
文面との差と、P1 の定義域については第 3 節に書く。

## 1. 記法

型環境 `E` を 1 つ固定する。以下、型に関する関数の `type_env` 引数は `E` に固定し、書かない。

- `L(t)` := `boxed_leaf_paths(t, E)` (`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)
- `U(t)` := `rc_units(t, E)` (`CODE src/rc_ir/ownership.rs: rc_units`)
- `T(t, p)` := `truncate_to_unit(t, p, E)` (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)
- `F(t)` := `t.unpunched_field_types(E)` (`CODE src/ast/types.rs: TypeNode::unpunched_field_types`)。
  これは対 `(添字, 型)` の列である。
- `c` := `CLOSURE_CAPTURE_IDX as usize` (`CODE src/constants.rs: CLOSURE_CAPTURE_IDX`)

型は `t`、`s`、`f` などのラテン文字で表す。path は `usize` の有限列 (`CODE src/rc_ir/ast.rs:
FieldPath`) であり、`p`、`q`、`u`、`lam` などで表す。`p[i]` は第 `i` 要素 (0 始まり)、`p[0..k]` は
先頭 `k` 要素からなる前置、`|p|` は長さ、`p ++ q` は連結、`[]` は空列とする。

**外部の結果 (Rust の一時値のスコープ)** -- Rust Reference の "Destructors" の節の "Temporary scopes" が
次を述べる。`<1>30` の `<2>1` がこれを引く。

> The _temporary scope_ of an expression is the scope that is used for the temporary variable that
> holds the result of that expression when used in a place context, unless it is promoted.
>
> Apart from lifetime extension, the temporary scope of an expression is the smallest scope that
> contains the expression and is one of the following:
>
> - The entire function.
> - A statement.
> - The body of an `if`, `while` or `loop` expression.
> - The `else` block of an `if` expression.
> - The non-pattern matching condition expression of an `if` or `while` expression or a
>   non-pattern-matching `match` guard condition operand.
> - The pattern-matching guard, if present, and body expression for a `match` arm.
> - Each operand of a lazy boolean expression.
> - The pattern-matching condition(s) and consequent body of `if` (2024 Edition).
> - The pattern-matching condition and loop body of `while`.
> - The entirety of the tail expression of a block (2024 Edition).
>
> **2024 Edition differences.** The 2024 edition added two new temporary scope narrowing rules:
> `if let` temporaries are dropped before the `else` block, and temporaries of tail expressions of
> blocks are dropped immediately after the tail expression is evaluated.

一時値はその temporary scope の終わりで drop される。

**DEF 呼び出しの辺** -- 表 `vars` と型環境 `E` を固定する。対 `(u, sig)` から対 `(u', sig')` への
**呼び出しの辺**とは、`origin_inner(vars, E, u, sig)` の実行が `origin(vars, E, u', sig')` を呼ぶことを
いう。`(x, pi)` から呼び出しの辺を 0 回以上辿って着ける対の全体を、`(x, pi)` の**呼び出しの下流**と呼ぶ。

この関係は memo の状態に依らない。`origin_inner` が行う `origin` の呼び出しとその引数は、`vars`、`E`、
`u`、`sig` だけで決まる -- `Join` の腕は `arm_results` の各要素について、`Llvm` の腕は `decl` と `args`
から作った対について呼ぶのであって、先に返った `origin` の値を読んで呼び先を決める腕は無い
(`CODE src/rc_ir/ownership.rs: origin_inner`, `CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。

**DEF cls** -- 型 `t` の**クラス** `cls(t)` を、次の順に最初に当たるもので定める。6 つの条件がすべて
真偽値を持つとき `cls(t)` は定まる。`<1>1` を満たす型についてそれが成り立つことは `<1>3c` と `<1>3e` が
示す。

| クラス | 条件 |
|---|---|
| `NB` | `t.is_fully_unboxed(E)` |
| `CL` | 上に当たらず `t.is_closure()` |
| `BX` | 上に当たらず `t.is_box(E)` |
| `AR` | 上に当たらず `t.is_array()` |
| `UN` | 上に当たらず (`t.is_union(E)` または `t.is_punched_array()`) |
| `ST` | 上のどれにも当たらない |

`UN` は D5 が leaf と unit のずれる 2 か所として挙げるもの、すなわち unbox union と punched array を
1 つのクラスにまとめたものである。

**DEF fld** -- `(i, f)` が `F(t)` の要素であるとき `fld(t, i) := f` と書く。この写像が一価であることは
`<1>12` で示す。

**DEF REC** -- 型の間の関係 `t REC f` を、「`t.is_box(E)`、`t.is_closure()`、`t.is_array()`、
`t.is_funptr()` がすべて偽であり、かつある `i` について `(i, f)` が `F(t)` の要素である」で定める。
これは `is_fully_unboxed` が自分を再帰呼び出しする辺そのものである。

**DEF DESC** -- 型の間の関係 `t DESC f` を、「`cls(t)` が `UN` か `ST` であり、かつある `i` について
`(i, f)` が `F(t)` の要素である」で定める。

**DEF ST-道** -- path `p` が型 `t` の **ST-道**であるとは、`s_0 := t`、`s_{j+1} := fld(s_j, p[j])` と
定めたとき、`s_0` から `s_{|p|}` までがすべて定義されており、かつ `j` が `|p|` 未満のすべてで
`cls(s_j) = ST` であることをいう。このとき `s_{|p|}` を `end(t, p)` と書く。

**DEF UNST-道** -- path `p` が型 `t` の **UNST-道**であるとは、上と同じ `s_j` について、`s_0` から
`s_{|p|}` までがすべて定義されており、かつ `j` が `|p|` 未満のすべてで `cls(s_j)` が `UN` か `ST`
であることをいう。ST-道はどれも UNST-道である。UNST-道 `p` についても `end(t, p) := s_{|p|}` と書く
(ST-道はどれも UNST-道なので、この 2 つの `end` は矛盾しない)。

**DEF unit に届く** -- path `p` が型 `t` の **unit に届く**とは、`T(t, p)` が abort せずに値を返し、
その値が `U(t)` の要素であることをいう。

## 2. 証明

<1>1. **(H1: 型の well-formedness)** RC IR に現れるすべての型 `t` について、次の 3 つが成り立つ。以下
   「`t` が `<1>1` を満たす」とは、この 3 つが `t` について成り立つことをいう。
   - (i) `t` は ground であり、`t.toplevel_tycon()` は型構成子 `tc` を返し、`E.tycons()` は `tc` を
     鍵に持ち、`t` に与えられた型引数の個数はその宣言の型変数の個数に等しい。すなわち
     `t.collect_type_arguments().len()` は `E.tycons()[&tc].tyvars.len()` に等しい。`tc` を経由して
     書くのは、`toplevel_tycon_info` がクロージャ型に対して `assert!` で止まり、クロージャ型も
     RC IR に現れるからである。
   - (ii) 型 `t'` から `t'.unpunched_field_types(E)` の各要素の第 2 成分への辺を**フィールドの辺**と
     呼ぶと、`t` から始まるフィールドの辺の無限列は存在せず、`t` からフィールドの辺を 0 回以上辿って
     到達できる型はどれも (i) を満たす。
   - (iii) (ii) が挙げる各型 `t'` について、`t'.field_types(E)` の中で `instance_field_types` が行う
     newtype の展開 (`unwrap_newtypes_memoized`) は abort せず停止する。

   (i) の「`t.toplevel_tycon()` が型構成子を返し `E.tycons()` がそれを鍵に持つ」は、
   `toplevel_tycon_info` が置く 2 つの `unwrap` (`toplevel_tycon().unwrap()` と
   `type_env.tycons().get(&tycon).unwrap()`) がどちらも成功することと同じであり、そのとき
   `toplevel_tycon_info` が返すのは `E.tycons()[&tc]` である。

   **(ii) は (i) について閉じている。**`t` が `<1>1` を満たし `t'` が `t` からフィールドの辺で到達
   できるならば、`t'` からフィールドの辺で到達できる型は `t` からも到達できるので、`t'` も `<1>1` を
   満たす。以下で `<1>1` を部分木の型に当てるのはこれによる。

   (i) と (ii) は A10 である。A10 の第 1 文が (i) を、第 3 段落「`unpunched_field_types` を繰り返し
   取って到達する型についても同じことが成り立ち、その歩みは有限である」が (ii) を与える。**(iii) は
   A10 に無く、この文書が足す前提である** (第 3 節)。

   **A10 を果たす `validate_layouts` は elaboration で必ず走るが、最適化が作る型を再検査するのは
   develop build だけである。**これは A10 の但し書きである。`borrow_ify` と `cancel` は最適化の後に
   走るので、release build ではこの 2 つが読む型のうち最適化が作ったものに `validate_layouts` は
   掛からない。
  BY A10, CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
     CODE src/ast/types.rs: TypeNode::collect_type_arguments,
     CODE src/ast/types.rs: TypeNode::unpunched_field_types,
     CODE src/ast/types.rs: TypeNode::instance_field_types,
     CODE src/rc_ir/ast.rs: RcVar (`ty` の doc「always concrete (monomorphic)」),
     CODE src/ast/program.rs: Program::validate_layouts

<1>2. **(H2: 変数のスコープ規律)** 関数本体の各節点 `n` について、`n` が使う変数はどれも、`Scope(n)`
   (下の `DEF Scope`) の要素であるか、この関数のどの束縛でもない名前 (グローバル) である。ここで
   `Let(x, rhs, k)` の `rhs` が使う変数は、`Scope` に `x` が入る**前**の集合、すなわち
   `Scope(Let(x, rhs, k) の節点)` で解決される。

   **DEF Scope** -- 本体の各節点 `n` に対する名前の集合 `Scope(n)` を次で定める。
   - `VarTable::of(func)` が作る表では、`Scope(根)` は `func.params` と `func.capture` の名前の集合。
     `VarTable::body_only(body)` が作る表では、`Scope(根)` は空集合。
   - `n = Let(x, rhs, k)` のとき、`Scope(k)` は `Scope(n)` に `x` を加えたもの。`rhs` が
     `Match(scrut, arms)` のとき、各アームについて `Scope(arm.body)` は `Scope(n)` に `arm.payload`
     を加えたもの。
   - `n = Destructure(cont, fields, s, k)` のとき、`Scope(k)` は `Scope(n)` に `fields` の各変数を
     加えたもの。
   - `n` が `Retain(v, p, s, k)`、`Release(v, p, s, k)`、`Eval(v, k)` のとき、`Scope(k)` は
     `Scope(n)` に等しい。

   主張そのものは A11 である。**スコープを定めるのは D2 であり、`DEF Scope` はその規則を節点の種類
   ごとに書き下したものである。**D2 は `Let` が束縛する `x` のスコープを `k` の部分木、`Destructure`
   が束縛する各変数のスコープを `k` の部分木、`Match` のアームの `payload` のスコープをそのアームの
   `body` の部分木、パラメータと capture のスコープを本体の全体と定める。上の 4 行はこれを、根から
   節点へ下る漸化式の形に直したものである。根の場合が 2 つに分かれるのは、`VarTable::of` が関数の
   本体を取り `VarTable::body_only` がグローバル初期化子の `init` を取るからで、`init` はパラメータも
   capture も持たない (D1) ので `Scope(根)` は空集合になる。A11 の検査 `validate` の `scope` の推移も
   この形であり、関数ごとに `func.params` と `func.capture` を `bind` してから `check_expr` を呼び、
   グローバル初期化子については `bind` を 1 度も呼ばずに `check_expr` を呼ぶ。**`validate` は
   `develop_mode` のときだけ走る** -- A11 の但し書きがこれを言う。
  BY A11, D1, D2, CODE src/rc_ir/validate.rs: validate,
     CODE src/rc_ir/validate.rs: Validator::check_expr_inner,
     CODE src/rc_ir/validate.rs: Validator::check_rhs,
     CODE src/rc_ir/validate.rs: Validator::use_var,
     CODE src/rc_ir/validate.rs: Validator::bind

<1>3. `RcRhs::Match(scrut, arms)` の `arms` は空でない。
  BY A9, CODE src/rc_ir/validate.rs: Validator::check_rhs (`RcRhs::Match` の腕の `arms.is_empty()`
     検査)

<1>3a. **(H4: 束縛の形と型が合っている)** 関数本体について次の 7 つが成り立つ。
   - (i) `Let(x, RcRhs::Var(y), k)` について `ty(y)` は `ty(x)` に等しい。
   - (ii) `Let(x, RcRhs::Match(scrut, arms), k)` の各アームについて、`returned_var(&arm.body)` の型は
     `ty(x)` に等しい。
   - (iii) 同じ `Match` の各アームについて、`arm.tag` が `Some(k)` のとき `(k, ty(arm.payload))` は
     `F(ty(scrut))` の要素であり、`arm.tag` が `None` のとき `ty(arm.payload)` は `ty(scrut)` に
     等しい。
   - (iv) 同じ `Match` の `ty(scrut)` は union の型である。すなわち
     `ty(scrut).toplevel_tycon_info(E).variant` は `TyConVariant::Union` である。
   - (v) `Destructure(cont, fields, s, k)` について `ty(cont)` は構造体 (タプルを含む) の型であり、
     各 `(i, fv)` について `(i, ty(fv))` は `F(ty(cont))` の要素である。
   - (vi) 同じ名前を持つ `RcVar` の出現はどれも同じ型を持つ。したがって `vars.var_tys` が記録する型は、
     その名前を使う側の `RcVar` の `ty` に等しい。以下ではこの型を `ty(名前)` と書く。
   - (vii) `Let(x, RcRhs::Llvm(llvm_gen, args), k)` について、`llvm_gen` は `args` の型の列と
     `ty(x)` の上で定義されている。この文書が読むのはそのうち次の 3 つである。
     - `args` の名前の列は `llvm_gen.free_vars()` に等しい。
     - `llvm_gen` が `InlineLLVMStructPunchBody` であるとき、`ty(x).is_box(E)` と
       `ty(x).is_array()` はどちらも偽であり、`ty(x).field_types(E)` は長さ 2 の列を返し、その第
       `PUNCHED_STRUCT_FIELD` 成分の型は `<1>1` を満たす構造体であって、第 `llvm_gen.field_idx`
       フィールドが穴である。
     - `llvm_gen` が `InlineLLVMStructSetBody` か `InlineLLVMStructPlugInBody` であるとき、
       `ty(x).is_array()` は偽である。

   7 つはどれも A12 である。(iii) と (v) が `F`、すなわち `unpunched_field_types` の要素で
   あることを言うのは、A12 の「`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が
   実際に持つ (punched でない) ものであること」を述べたものである。(vii) の 3 つは A12 の
   「`Llvm` 節点の型についての 3 つ」である。A12 の第 2 項は punched struct の成分を「A10 を満たす
   構造体」と書く。(vii) がそこを `<1>1` と書くのは、`<1>1` が A10 に (iii) を足したものだからで
   あり、この差は `<1>1` の (iii) と同じ 1 つの差である (第 3 節)。
  BY A12, CODE src/rc_ir/ast.rs: RcRhs (`Var` の doc「Move / rename `y := x`, consuming `x`」),
     CODE src/rc_ir/ast.rs: MatchArm (`tag` と `payload` の doc),
     CODE src/rc_ir/ast.rs: RcExpr (`Destructure` の doc「Destructure a struct/tuple container into
       its fields at once ... Each `(index, var)` binds field `index` to `var`」),
     CODE src/rc_ir/ast.rs: RcVar,
     CODE src/ast/types.rs: TypeNode::unpunched_field_types,
     CODE src/rc_ir/ownership.rs: returned_var,
     CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/rc_ir/ownership.rs: collect_bindings

<1>3b. 製品のコードが `TyConInfo` の値を作る場所は、次の 4 つの関数だけである (`ownership.rs` に残る
   2 か所は `#[cfg(test)] mod tests` の中にある)。よって `E` に登録されている `TyConInfo` はどれも
   この 4 つのいずれかが作ったものである。各関数が置く `variant` と、この証明が読むフィールドは次の
   とおりである。

   | 作る関数 | `variant` | 個数と、この証明が読むフィールド |
   |---|---|---|
   | `bulitin_tycons` | `Primitive` | 112 個。名前つきの 12 個 (`IOState`、`Ptr`、`U8`、`I8`、`U16`、`I16`、`I32`、`U32`、`I64`、`U64`、`F32`、`F64`) と、`1..=FUNPTR_ARGS_MAX` の各 arity についての `#FunPtr{n}` -- `FUNPTR_ARGS_MAX` は 100 である。名前つきの 12 個は `fields: vec![]` と `tyvars: vec![]` を持ち、`#FunPtr{n}` は `fields: vec![]` と `tyvars` が `a0` から `a{n-1}` までの相異なる名前の列である |
   | `bulitin_tycons` | `Array` | 1 個 (`Std::Array`)。`tyvars: vec![make_tyvar("a", ...)]` |
   | `bulitin_tycons` | `Arrow` | 1 個。`tyvars` は `a` と `b` |
   | `bulitin_tycons` | `DynamicObject` | 1 個 (`#DynamicObject`)。`is_unbox: false` |
   | `bulitin_tycons` | `ArrayStorage` | 1 個 (`#ArrayStorage`)。`is_unbox: false` |
   | `TypeDefn::tycon_info` | `Struct` か `Union` | 宣言ごとに 1 個 (穴つきの形を含む)。`tyvars` は `self.tyvars` |
   | `CaptureStruct::new` | `Struct` | capture 構造体ごとに 1 個。`tyvars: vec![]` |
   | `register_opaque_tycon` | `Opaque` | 不透明型ごとに 1 個。`is_unbox: false` |

   すなわち `DynamicObject`、`ArrayStorage`、`Opaque`、`Primitive` の 4 つの `variant` は、それぞれ
   ちょうど 1 か所からしか出ない。

   **`Opaque` を除く 7 行の `tyvars` の名前は相異なる。**`bulitin_tycons` の 5 行と
   `CaptureStruct::new` は上の表が名前を挙げている。`TypeDefn::tycon_info` の行は `self.tyvars` を
   そのまま置き、`TypeDefn::validate_tyvars` がその名前が重複しないことを検査する。`E` に入る
   `Struct`/`Union` の `TyConInfo` を作るのは `Program::calculate_type_env` であり、それは
   `self.type_defns` の各要素について `type_decl.tycon_info(&[])` と、構造体ならフィールドごとの
   穴つきの形 `type_decl.tycon_info(&[i])` を入れる。`Program::validate_type_defns` は同じ
   `self.type_defns` を渡って `validate_tyvars` を呼び、`elaborate` はそれを `?` で呼ぶので、
   elaboration を通ったプログラムではどの行の名前も相異なる。
  BY CODE src/ast/types.rs: TyConInfo, CODE src/fixstd/builtin.rs: bulitin_tycons,
     CODE src/constants.rs: FUNPTR_ARGS_MAX, CODE src/ast/typedecl.rs: TypeDefn::tycon_info,
     CODE src/ast/typedecl.rs: TypeDefn::validate_tyvars,
     CODE src/ast/program.rs: Program::validate_type_defns,
     CODE src/ast/program.rs: Program::calculate_type_env,
     CODE src/elaboration/mod.rs: elaborate,
     CODE src/optimization/capture_struct.rs: CaptureStruct::new,
     CODE src/elaboration/desugar_opaque.rs: register_opaque_tycon

<1>3c. `<1>1` を満たす型 `t` について、次の 4 つが成り立つ。
   - (a) `t.is_closure()`、`t.is_array()`、`t.is_funptr()`、`t.is_punched_array()` は abort せず
     真偽値を返す。
   - (b) `t.is_unbox(E)` と `t.is_box(E)` は abort せず真偽値を返す。
   - (c) `t.is_closure()` が偽であるとき、`t.toplevel_tycon_info(E)` と `t.is_union(E)` は abort せず
     値を返す。
   - (d) `t.is_closure()` と `t.is_box(E)` がどちらも偽であるとき、`t.unpunched_field_types(E)` と
     `t.field_types(E)` は abort せず停止する。この 2 つの列は長さが等しく、`F(t)` の各要素 `(i, f)`
     について `i` は `t.field_types(E)` の長さ未満であり `f` は `t.field_types(E)[i]` である。
  <2>1. (a) が成り立つ。この 4 つはいずれも `toplevel_tycon_satisfies(pred)` の 1 行であり、
     `toplevel_tycon_satisfies` は `self.toplevel_tycon()` の `Option` を見て `Some` なら `pred` を、
     `None` なら偽を返す。渡される `pred` は `make_arrow_name_abs()` との名前の比較、
     `is_array_tycon`、`is_funptr_tycon`、`is_punched_array_tycon` であり、どれも `TyCon` の名前を
     比べるだけで `E` を引かない。
    BY CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies,
       CODE src/ast/types.rs: TypeNode::is_closure, CODE src/ast/types.rs: TypeNode::is_array,
       CODE src/ast/types.rs: TypeNode::is_funptr,
       CODE src/ast/types.rs: TypeNode::is_punched_array,
       CODE src/fixstd/builtin.rs: is_array_tycon, CODE src/fixstd/builtin.rs: is_funptr_tycon,
       CODE src/fixstd/builtin.rs: is_punched_array_tycon
  <2>2. (c) が成り立つ。`toplevel_tycon_info` は `assert!(!self.is_closure())` を置き、
     `self.toplevel_tycon().unwrap()` と `type_env.tycons().get(&tycon).unwrap()` を行う。この場合の
     仮定より `assert!` は通り、`<1>1` (i) より 2 つの `unwrap` は成功する。`is_union` は
     `toplevel_tycon_info` の `variant` を見るだけである。
    BY <1>1, CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
       CODE src/ast/types.rs: TypeNode::is_union
  <2>3. (b) が成り立つ。`is_unbox` は `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox`
     である。`is_closure()` が真のときは短絡して `toplevel_tycon_info` を呼ばず真を返し、偽のときは
     `<2>2` より `toplevel_tycon_info` が値を返す。`is_box` は `is_unbox` の否定である。
    BY <2>1, <2>2, CODE src/ast/types.rs: TypeNode::is_unbox, CODE src/ast/types.rs: TypeNode::is_box
  <2>4. 以下 `t.is_closure()` と `t.is_box(E)` がどちらも偽であるとして (d) を示す。
     `ti := t.toplevel_tycon_info(E)` と置く。`<2>2` より `ti` は abort せず得られる。`ti.variant` は
     `Primitive`、`Array`、`Struct`、`Union` のいずれかである。
    <3>1. `ti.variant` は `Arrow` ではない。`TyConVariant::Arrow` を持つ `TyConInfo` は
       `bulitin_tycons` が `make_arrow_tycon()` の下に作る 1 個だけであり (`<1>3b`)、`is_closure()` は
       型構成子の名前が `make_arrow_name_abs()` に等しいかを問う述語で、`make_arrow_tycon()` は
       まさにその名前の `TyCon` である。よってそのとき `t.is_closure()` は真になり、この場合の仮定に
       反する。
      BY <1>3b, CODE src/ast/types.rs: TypeNode::is_closure,
         CODE src/fixstd/builtin.rs: bulitin_tycons,
         CODE src/fixstd/builtin.rs: make_arrow_tycon,
         CODE src/fixstd/builtin.rs: make_arrow_name_abs
    <3>2. `ti.variant` は `DynamicObject`、`ArrayStorage`、`Opaque` のいずれでもない。`<1>3b` より
       この 3 つを持つ `TyConInfo` はどれも `is_unbox: false` を持つ。`t.is_closure()` が偽なので
       `t.is_unbox(E)` は `ti.is_unbox` に等しく偽であり、`t.is_box(E)` はその否定で真になって、この
       場合の仮定に反する。
      BY <2>3, <1>3b, CODE src/ast/types.rs: TypeNode::is_unbox
    <3>3. QED
       `TyConVariant` の値は `Primitive`、`Arrow`、`Array`、`Struct`、`Union`、`DynamicObject`、
       `ArrayStorage`、`Opaque` の 8 つであり、`<3>1` と `<3>2` が 4 つを退けている。
      BY <3>1, <3>2, CODE src/ast/types.rs: TyConVariant
  <2>5. 同じ仮定の下で `ti.tyvars` に現れる名前は相異なる。`<2>4` の 4 つの `variant` を `<1>3b` の
     表で引くと、
     `Primitive` と `Array` は `bulitin_tycons` の行、`Struct` と `Union` は `TypeDefn::tycon_info` か
     `CaptureStruct::new` の行であり、`<1>3b` はその 7 行すべてについて名前が相異なることを述べる。
    BY <1>3b, <2>4
  <2>6. 同じ仮定の下で `t.declared_field_types(ti)` は abort せず停止し、長さは `ti.fields` の
     長さに等しい。
    <3>1. `declared_field_types` はまず `let args = self.collect_type_arguments();` と
       `assert_eq!(args.len(), tycon_info.tyvars.len())` を行う。`<1>1` (i) がこの等式を与えるので
       `assert_eq!` は通る。
      BY <1>1, CODE src/ast/types.rs: TypeNode::declared_field_types
    <3>2. 続く `for` は `ti.tyvars` の各 `tv` について
       `subst.merge(&Substitution::single(&tv.name, args[i].clone()))` を行い、`assert!(merge_ok)` を
       置く。`args[i]` の添字は `<3>1` の等式より範囲内である。`Substitution::single` は 1 要素の
       写像を作り、`merge` は相手の各要素について、自分が同じ鍵を持ちかつ値が異なるときにだけ偽を
       返す。`<2>5` より `tv.name` は周ごとに相異なるので、`subst` は `tv.name` をまだ持っておらず、
       `merge` は真を返す。よって `assert!` は通る。
      BY <2>5, <3>1, CODE src/ast/types.rs: TypeNode::declared_field_types,
         CODE src/elaboration/typecheck.rs: Substitution::merge,
         CODE src/elaboration/typecheck.rs: Substitution::single
    <3>3. 最後の式は `ti.fields.iter().map(|field| subst.substitute_type(&field.ty)).collect()` で
       ある。`substitute_type` は型の節点についての再帰であり、`TyVar` の腕は写像を引くだけ、
       `TyCon` の腕は複製、`TyApp` の腕は 2 つの部分に再帰してから `set_tyapp_fun`/`set_tyapp_arg`
       を、`AssocTy` の腕は各引数に再帰してから `set_assocty_args` を呼ぶ。この 3 つの setter が
       `panic!` するのは節点が対応する `Type` の腕でないときだけで、どれもその腕の中から呼ばれる。
       型は有限の項なので再帰は停止する。返る列の長さは `ti.fields` の長さである。
      BY CODE src/elaboration/typecheck.rs: Substitution::substitute_type,
         CODE src/ast/types.rs: TypeNode::set_tyapp_fun,
         CODE src/ast/types.rs: TypeNode::set_tyapp_arg,
         CODE src/ast/types.rs: TypeNode::set_assocty_args,
         CODE src/ast/types.rs: TypeNode::declared_field_types
    <3>4. QED
      BY <3>1, <3>2, <3>3
  <2>7. 同じ仮定の下で `t.field_types(E)` は abort せず停止し、長さは `ti.fields` の長さに等しい。
     `field_types` は
     `self.instance_field_types(self.toplevel_tycon_info(type_env), type_env)` の 1 文であり、
     `instance_field_types` は `declared_field_types` の結果を受け取り、`ti.tyvars` に kind が `*` で
     ない要素があるときにだけ各要素に `unwrap_newtypes_memoized` を当てる。前者は `<2>6`、後者は
     `<1>1` (iii) より abort せず停止する。どちらも列の長さを変えない。
    BY <1>1, <2>2, <2>6, CODE src/ast/types.rs: TypeNode::field_types,
       CODE src/ast/types.rs: TypeNode::instance_field_types
  <2>8. QED
    (a) は `<2>1`、(b) は `<2>3`、(c) は `<2>2` である。(d) を示す。仮定は `t.is_closure()` と
    `t.is_box(E)` がどちらも偽であることで、これは `<2>4` から `<2>7` が置いた仮定である。
    `unpunched_field_types` は `self.toplevel_tycon_info(type_env)` を `ti` に取り、
    `self.instance_field_types(ti, type_env)` の結果に `into_iter().enumerate()` を当ててから
    `filter(|(i, _)| !ti.fields[*i].is_punched)` で絞る。`<2>7` よりその列の長さは `ti.fields` の
    長さに等しいので `enumerate` が渡す `i` はどれも `ti.fields` の添字として範囲内であり、
    `ti.fields[*i]` は abort しない。`filter` は要素を落とすだけなので、残った `(i, f)` の `f` は
    `instance_field_types` の結果の第 `i` 要素、すなわち `field_types` の第 `i` 要素である。
    BY <2>1, <2>2, <2>3, <2>7, CODE src/ast/types.rs: TypeNode::unpunched_field_types,
       CODE src/ast/types.rs: TypeNode::field_types

<1>3d. `<1>1` を満たす型 `t_0` から始まる無限列 `t_0 REC t_1 REC t_2 ...` は存在しない。
  <2>1. `t REC f` であるとき、ある `i` について `(i, f)` は `F(t)` の要素である。すなわち `REC` の辺は
     `<1>1` (ii) のフィールドの辺である。
    BY DEF REC
  <2>2. QED
    `<2>1` より無限の `REC` の列は `t_0` から始まる無限のフィールドの辺の列であり、`<1>1` (ii) に
    反する。
    BY <1>1, <2>1

<1>3e. `<1>1` を満たす型 `t` について、`t.is_fully_unboxed(E)` は abort せず停止して真偽値を返す。
   したがって `DEF cls` の 6 つの条件はすべて真偽値を持つ。
  <2>1. `is_fully_unboxed(s)` の本体は、`is_box` が真なら偽を返し、`is_closure` が真なら偽を返し、
     `is_array` が真なら偽を返し、`is_funptr` が真なら真を返し、そのどれでもなければ
     `unpunched_field_types(s)` の各要素の第 2 成分についての `is_fully_unboxed` の連言を返す。
    BY CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>2. `<1>1` を満たす `s` について、`<2>1` の 4 つの検査は `<1>3c` (a)(b) より abort せず、最後の
     段に進むのは `is_box(s)` と `is_closure(s)` がどちらも偽のときなので、`<1>3c` (d) より
     `unpunched_field_types(s)` も abort せず有限列を返す。
    BY <1>3c, <2>1
  <2>3. `<2>1` の最後の段が再帰する先は `F(s)` の各要素の第 2 成分であり、そのとき `is_box`、
     `is_closure`、`is_array`、`is_funptr` はすべて偽なので、その辺は `DEF REC` の辺である。また
     `<1>1` (ii) より再帰する先も `<1>1` を満たす。
    BY <1>1, <2>1, DEF REC
  <2>4. QED
    `<2>3` より再帰の辺は `REC` の辺であり、`<1>3d` より `REC` は `<1>1` を満たす型の上で整礎、
    `<2>2` より各段の分岐は有限である。よって再帰は有限で終わり、`<2>2` より途中の各段も abort
    しない。`DEF cls` の残り 5 つの条件のうち `is_closure`、`is_array`、`is_punched_array` は
    `<1>3c` (a)、`is_box` は `<1>3c` (b) が与える。`is_union` は `<1>3c` (c) が `is_closure` の偽の
    下で与えるが、`DEF cls` が `is_union` を問うのは `CL` の行に当たらなかったとき、すなわち
    `is_closure` が偽のときだけである。
    BY <1>3c, <1>3d, <2>2, <2>3, DEF cls

<1>4. `<1>1` を満たす任意の型 `t` について `cls(t)` はちょうど 1 つの値に定まる。
  <2>1. `DEF cls` の 6 つの条件はすべて真偽値を持つ。
    BY <1>3e
  <2>2. `DEF cls` の 6 つの条件は、上から順に最初に成り立つものを採る形で書かれている。よって高々
     1 つに定まる。
    BY DEF cls
  <2>3. 最後の `ST` は「上のどれにも当たらない」なので、6 つの場合は尽きている。よって 1 つ以上に
     定まる。
    BY DEF cls
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>7. `<1>1` を満たす型 `t` について `cls(t)` が `UN` か `ST` であるとき、次の 3 つが成り立つ。
   - (a) `t.is_box(E)`、`t.is_closure()`、`t.is_array()`、`t.is_funptr()` はすべて偽である。
   - (b) `t.is_fully_unboxed(E)` の値は `F(t)` の各要素の第 2 成分についての `is_fully_unboxed` の
     連言に等しく、その連言は偽である。
   - (c) `t DESC f` は `t REC f` を含意する。
  <2>1. `DEF cls` より `t.is_fully_unboxed(E)`、`t.is_closure()`、`t.is_box(E)`、`t.is_array()` は
     いずれも偽である。
    BY DEF cls
  <2>2. `is_fully_unboxed` の本体は、`is_box` が真なら偽を返し、`is_closure` が真なら偽を返し、
     `is_array` が真なら偽を返し、`is_funptr` が真なら真を返し、そのどれでもなければ `F(t)` の各要素
     の第 2 成分についての `is_fully_unboxed` の連言を返す。`<1>3e` よりこの呼び出しは abort せず
     停止するので、この場合分けのどれか 1 つが値を与える。
    BY <1>3e, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>3. (a) が成り立つ。`<2>1` が 3 つを与える。`t.is_funptr()` が真だとすると、`<2>1` と `<2>2` に
     より `is_fully_unboxed(t)` は真を返し、`<2>1` に反する。
    BY <2>1, <2>2
  <2>4. (b) が成り立つ。`<2>2` の 5 段のうち最初の 4 段の条件は `<2>1` と `<2>3` によりすべて偽なので、
     値は連言である。`<2>1` よりその値は偽である。
    BY <2>1, <2>2, <2>3
  <2>5. (c) が成り立つ。`t DESC f` は `cls(t)` が `UN` か `ST` であることと `(i, f)` が `F(t)` の
     要素であることを言う。`<2>3` (a) が `DEF REC` の 4 つの条件を与える。
    BY <2>3, DEF REC, DEF DESC
  <2>6. QED
    BY <2>3, <2>4, <2>5

<1>8. `<1>1` を満たす型の上で `DESC` は整礎である。すなわち `<1>1` を満たす型 `t_0` から始まる
   無限列 `t_0 DESC t_1 DESC t_2 ...` は存在しない。
  BY <1>3d, <1>7

<1>9. `<1>1` を満たす型 `t` について、`is_fully_unboxed(t)`、`unit_step(t)`、`rc_units(t)`、
   `boxed_leaf_paths(t)` は abort せず停止し、`L(t)` と `U(t)` は有限集合である。また
   `truncate_to_unit(t, p)` は任意の path `p` について停止し、abort するとすれば
   `UnitStep::NoUnit` の腕の `panic!`、`UnitStep::Capture` の腕の `assert_eq!`、`held_field_type` の
   `panic!` のいずれかである。
  <2>1. `is_fully_unboxed(t)` が自分を再帰呼び出しするのは、`is_box`、`is_closure`、`is_array`、
     `is_funptr` がすべて偽のときに、`F(t)` の各要素の第 2 成分に対してだけである。すなわち再帰の辺は
     `REC` の辺そのものである。
    BY DEF REC, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>2. `is_fully_unboxed(t)` は abort せず停止する。`<2>1` が挙げる再帰の辺の上でそれが成り立つ
     ことを `<1>3e` が述べる。
    BY <1>3e, <2>1
  <2>3. `unit_step(t)` は `is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、`is_array`、
     `is_punched_array`、`toplevel_tycon_info`、`unpunched_field_types` を各 1 回呼ぶだけで、自分を
     再帰呼び出ししない。`<2>2` よりはじめの `is_fully_unboxed` は abort せず停止し、`<1>3c` (a)(b)
     より `is_closure`、`is_box`、`is_array`、`is_punched_array` も abort しない。`is_union`、
     `toplevel_tycon_info`、`unpunched_field_types` を呼ぶのは `is_closure` が偽でありかつ `is_box` が
     偽である場合に限られるので、`<1>3c` (c)(d) よりこの 3 つも abort せず停止する。
    BY <1>3c, <2>2, CODE src/rc_ir/ownership.rs: unit_step
  <2>4. `rc_units_go(s, q, out)` の再帰呼び出しは `unit_step(s)` が `UnitStep::Fields` を返したときの
     `held_fields` の各要素に対してだけ起き、そのとき `held_fields` は `F(s)` である。
    BY CODE src/rc_ir/ownership.rs: rc_units_go, CODE src/rc_ir/ownership.rs: unit_step
  <2>5. `unit_step(s)` が `Fields` を返すのは `is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、
     `is_array`、`is_punched_array` がすべて偽のとき、すなわち `cls(s) = ST` のときだけである。
    BY DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>6. `<2>4` と `<2>5` より `rc_units_go` の再帰の辺は `DESC` の辺である。`<1>1` (ii) より
     再帰する先も `<1>1` を満たすので、`<2>3` を各段に当てられる。`<1>8` と `F` の有限性より
     `rc_units_go` は abort せず停止し、`out` に積む path の個数は有限である。よって `rc_units` も
     そうであり、`U(t)` は有限である。
    BY <1>1, <1>8, <2>3, <2>4, <2>5, DEF DESC, CODE src/rc_ir/ownership.rs: rc_units
  <2>7. `boxed_leaf_paths` の内部関数 `go(s, q, out)` の再帰呼び出しは、`is_fully_unboxed`、
     `is_closure`、`is_box`、`is_array` がすべて偽のときに `F(s)` の各要素に対してだけ起きる。`go` が
     呼ぶのはこの 4 つの述語と `unpunched_field_types` だけである。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>8. `<2>7` の条件のとき `cls(s)` は `UN` か `ST` である。
    BY <2>7, DEF cls
  <2>9. `<2>7` と `<2>8` より `go` の再帰の辺は `DESC` の辺である。`<1>1` (ii) より再帰する先も
     `<1>1` を満たす。`go` が各段で呼ぶ 4 つの述語は `<1>3c` (a)(b) より abort せず、
     `unpunched_field_types` を呼ぶのは `is_closure` と `is_box` がどちらも偽のときなので `<1>3c` (d)
     より abort しない。`<1>8` と `F` の有限性より `go` は abort せず停止し、`out` に積む path の
     個数は有限である。よって `boxed_leaf_paths` もそうであり、`L(t)` は有限である。
    BY <1>1, <1>3c, <1>8, <2>7, <2>8, DEF DESC, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>10. `truncate_to_unit(t, p)` の本体は `p` の要素についての `for` ループ 1 つであり、各周で
     `unit_step` と `held_field_type` を呼ぶだけで、自分を再帰呼び出ししない。`p` は有限列であり、
     ループの各周の `cur` は `<1>1` (ii) より `<1>1` を満たすので (第 0 周は `t` 自身、以後は
     `held_field_type` が返す `F` の要素)、`<2>3` より `unit_step` は abort せず停止する。
     `held_field_type` は有限列の線形探索であり、abort するのは見つからないときの `panic!` だけで
     ある。よって残る abort の場所は `UnitStep::NoUnit` の腕の `panic!` と `UnitStep::Capture` の腕の
     `assert_eq!` である。
    BY <1>1, <2>3, CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/rc_ir/ownership.rs: held_field_type
  <2>11. QED
    BY <2>2, <2>3, <2>6, <2>9, <2>10

<1>10. `<1>1` を満たす型 `t` について、`unit_step(t, E)` の返す `UnitStep` は `cls(t)` で決まり、
   次の表の通りである。

   | `cls(t)` | `unit_step(t, E)` |
   |---|---|
   | `NB` | `UnitStep::NoUnit` |
   | `CL` | `UnitStep::Capture { capture_idx: c, .. }` |
   | `BX` | `UnitStep::Unit` |
   | `AR` | `UnitStep::Unit` |
   | `UN` | `UnitStep::Unit` |
   | `ST` | `UnitStep::Fields { held_fields: F(t), .. }` |

  <2>1. CASE `cls(t) = NB`。定義より `t.is_fully_unboxed(E)` が真であり、`unit_step` の最初の検査が
     これなので `UnitStep::NoUnit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>2. CASE `cls(t) = CL`。定義より `is_fully_unboxed` は偽、`is_closure()` は真である。`unit_step`
     は 2 番目の検査で
     `UnitStep::Capture { capture_idx: CLOSURE_CAPTURE_IDX as usize, field_count: CLOSURE_FIELD_COUNT }`
     を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step, CODE src/constants.rs: CLOSURE_CAPTURE_IDX
  <2>3. CASE `cls(t) = BX`。定義より `is_fully_unboxed` と `is_closure` は偽、`is_box(E)` は真で
     ある。`unit_step` の 3 番目の検査は `is_box || is_union || is_array || is_punched_array` の
     選言なので真になり、`UnitStep::Unit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>4. CASE `cls(t) = AR`。定義より `is_fully_unboxed`、`is_closure`、`is_box` は偽、`is_array()` は
     真である。`<2>3` と同じ選言の第 3 項が真になり `UnitStep::Unit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>5. CASE `cls(t) = UN`。定義より `is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽、
     `is_union(E)` か `is_punched_array()` のどちらかが真である。同じ選言の第 2 項 (unbox union) か
     第 4 項 (punched array) が真になり `UnitStep::Unit` を返す。
    BY D5, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>6. CASE `cls(t) = ST`。定義より `is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、
     `is_array`、`is_punched_array` がすべて偽である。よって `unit_step` は 3 つの検査をすべて通り
     抜け、
     `UnitStep::Fields { field_count: t.toplevel_tycon_info(E).fields.len(), held_fields: t.unpunched_field_types(E) }`
     を返す。`held_fields` は `F(t)` そのものである。
    BY D5, DEF cls, DEF F, CODE src/rc_ir/ownership.rs: unit_step
  <2>7. QED
    `<1>4` より場合は尽きており排他である。
    BY <1>4, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>11. `<1>1` を満たす型 `t` について、`boxed_leaf_paths` の内部関数 `go(t, path, out)` の振る舞いは
   `cls(t)` で決まり、次の通りである (`path` は呼び出し時の値、`out` への追加だけを書く)。

   | `cls(t)` | `go(t, path, out)` |
   |---|---|
   | `NB` | 何もしない |
   | `CL` | `path ++ [c]` を `out` に積む |
   | `BX` | `path` を `out` に積む |
   | `AR` | `path` を `out` に積む |
   | `UN` | `F(t)` の各 `(i, f)` について `go(f, path ++ [i], out)` |
   | `ST` | `F(t)` の各 `(i, f)` について `go(f, path ++ [i], out)` |

  <2>1. CASE `cls(t) = NB`。`go` の最初の検査 `is_fully_unboxed` が真なので即 `return` する。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>2. CASE `cls(t) = CL`。`is_fully_unboxed` は偽、`is_closure()` は真である。`go` は
     `path.push(CLOSURE_CAPTURE_IDX as usize)`、`out.push(path.clone())`、`path.pop()` を行って
     `return` する。積まれる path は `path ++ [c]` である。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`),
       CODE src/constants.rs: CLOSURE_CAPTURE_IDX
  <2>3. CASE `cls(t) = BX`。`is_fully_unboxed` と `is_closure` は偽、`is_box(E)` は真である。`go` は
     `out.push(path.clone())` を行って `return` する。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>4. CASE `cls(t) = AR`。`is_fully_unboxed`、`is_closure`、`is_box` は偽、`is_array()` は真で
     ある。`go` は `is_array` の検査で `out.push(path.clone())` を行って `return` する。
    BY D4, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>5. CASE `cls(t) = UN`。`is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽である。`go`
     にはこの 4 つの検査しか無いので、そのすべてを通り抜けて最後の
     `for (i, fty) in ty.unpunched_field_types(type_env)` に進み、各要素について `path.push(i)`、
     `go(&fty, ..., path, out)`、`path.pop()` を行う。D4 の第 5 項が言うとおり、`F(t)` は穴 (punched
     field) を含まないので、穴の下へは降りない。
    BY D4, DEF cls, DEF F, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>6. CASE `cls(t) = ST`。`is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽である (`ST`
     はこの 4 つに加えて `is_union` と `is_punched_array` も偽である場合だが、`go` はその 2 つを
     問わない)。`<2>5` と同じ最後のループに進む。
    BY D4, DEF cls, DEF F, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>7. QED
    `<1>4` より場合は尽きており排他である。`UN` と `ST` で振る舞いが同じことが、この 2 つの walk の
    違いの全部である。`unit_step` は `UN` で止まり (`<1>10`)、`go` は `UN` で降りる。
    BY <1>4, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>12. `<1>1` を満たす型 `t` で `F(t)` が定まるとき、`F(t)` に現れる添字は相異なる。したがって
   `fld(t, i)` は一価であり、`(i, f)` が `F(t)` の要素であるとき `held_field_type(F(t), i, w)` は
   abort せず `f` を返す。
  <2>1. `unpunched_field_types` は `instance_field_types(...)` の結果に `into_iter().enumerate()` を
     適用してから `filter` する。`enumerate` の添字は 0 から始まる相異なる整数の列であり、`filter` は
     その一部を残すだけである。
    BY CODE src/ast/types.rs: TypeNode::unpunched_field_types
  <2>2. `<2>1` より `F(t)` の第 1 成分は相異なる。よって「第 1 成分が `i` である要素」は高々 1 つで
     あり、`fld(t, i)` は一価である。
    BY <2>1, DEF fld
  <2>3. `held_field_type(held_fields, idx, walk_name)` は
     `held_fields.iter().find(|(i, _)| *i == idx)` を行い、見つからないときだけ `panic!` し、
     見つかったときはその第 2 成分の複製を返す。
    BY CODE src/rc_ir/ownership.rs: held_field_type
  <2>4. QED
    `(i, f)` が `F(t)` の要素なら `find` は成功し、`<2>2` よりその要素は `(i, f)` に限るので、返る値
    は `f` である。
    BY <2>2, <2>3

<1>13. `<1>1` を満たす任意の型 `t` について、`U(t)` は次の 2 つの集合の合併に等しい。
   - `{ p : p は t の ST-道で、cls(end(t, p)) が BX、AR、UN のどれか }`
   - `{ p ++ [c] : p は t の ST-道で、cls(end(t, p)) = CL }`
  <2>1. `rc_units(t)` は `out` を空にして `rc_units_go(t, E, &mut vec![], &mut out)` を呼び、`out` を
     返す。
    BY CODE src/rc_ir/ownership.rs: rc_units
  <2>2. `rc_units_go` の `UnitStep::Capture` の腕は `path.push(capture_idx)`、
     `out.push(path.clone())`、`path.pop()` を行い、`UnitStep::Unit` の腕は `out.push(path.clone())`
     を行い、`UnitStep::Fields` の腕は各フィールドについて `path.push(i)`、再帰、`path.pop()` を
     行う。`UnitStep::NoUnit` の腕は何もしない。よって、呼び出し時の `path` の値を `q` と書くと、
     再帰から戻ったとき `path` は `q` に戻っている。
    BY CODE src/rc_ir/ownership.rs: rc_units_go
  <2>3. `<1>1` を満たす型 `s` について、`rc_units_go(s, E, path, out)` が `out` に積む path の
     集合を `R(s, q)` (`q` は呼び出し時の `path` の値) と書くと、`<1>10` と `<2>2` より次が成り立つ。
     `cls(s) = ST` のとき降りる先の `f` も `<1>1` (ii) より `<1>1` を満たす。
     - `cls(s) = NB` のとき `R(s, q)` は空集合
     - `cls(s) = CL` のとき `R(s, q)` は `{q ++ [c]}`
     - `cls(s)` が `BX`、`AR`、`UN` のとき `R(s, q)` は `{q}`
     - `cls(s) = ST` のとき `R(s, q)` は `F(s)` の各 `(i, f)` についての `R(f, q ++ [i])` の合併
    BY <1>1, <1>10, <2>2, CODE src/rc_ir/ownership.rs: rc_units_go
  <2>4. `<2>3` の再帰式を `DESC` に関する整礎帰納法 (`<1>8`) で展開する。`R(t, [])` の要素は、
     `cls(s_j) = ST` (`j` は `|p|` 未満) を満たす道 `p` をたどって到達した型 `end(t, p)` のクラスが
     `NB` でも `ST` でもないところで積まれた path である。すなわち `R(t, [])` は
     `{ p : p は t の ST-道で cls(end(t, p)) が BX、AR、UN のどれか }` と
     `{ p ++ [c] : p は t の ST-道で cls(end(t, p)) = CL }` の合併である。
    BY <1>8, <2>3, DEF ST-道, DEF fld
  <2>5. QED
    BY <2>1, <2>4

<1>14. `<1>1` を満たす任意の型 `t` について、`L(t)` は次の 2 つの集合の合併に等しい。
   - `{ p : p は t の UNST-道で、cls(end(t, p)) が BX か AR }`
   - `{ p ++ [c] : p は t の UNST-道で、cls(end(t, p)) = CL }`

   さらに、`L(t)` の要素 `lam` に対するこの分解 `lam = p ++ e` は一意である。すなわち `p` は、`lam`
   の前置のうち `t` の UNST-道であって `cls(end(t, p))` が `UN` でも `ST` でもないもの、ただ 1 つで
   ある。
  <2>1. `boxed_leaf_paths(t)` は `out` を空にして `go(t, E, &mut Vec::new(), &mut out)` を呼び、`out`
     を返す。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>2. `go` はクロージャの腕で `path.push`、`out.push(path.clone())`、`path.pop()` を行い、フィールド
     のループで `path.push(i)`、再帰、`path.pop()` を行う。よって、呼び出し時の `path` の値を `q` と
     書くと、再帰から戻ったとき `path` は `q` に戻っている。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>3. `<1>1` を満たす型 `s` について、`go(s, E, path, out)` が `out` に積む path の集合を
     `G(s, q)` と書くと、`<1>11` と `<2>2` より次が成り立つ。`cls(s)` が `UN` か `ST` のとき降りる
     先の `f` も `<1>1` (ii) より `<1>1` を満たす。
     - `cls(s) = NB` のとき `G(s, q)` は空集合
     - `cls(s) = CL` のとき `G(s, q)` は `{q ++ [c]}`
     - `cls(s)` が `BX` か `AR` のとき `G(s, q)` は `{q}`
     - `cls(s)` が `UN` か `ST` のとき `G(s, q)` は `F(s)` の各 `(i, f)` についての `G(f, q ++ [i])`
       の合併
    BY <1>1, <1>11, <2>2, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>4. `<2>3` の再帰式を `DESC` に関する整礎帰納法 (`<1>8`) で展開する。`G(t, [])` の要素は、
     `cls(s_j)` が `UN` か `ST` (`j` は `|p|` 未満) を満たす道 `p` をたどって到達した型 `end(t, p)`
     のクラスが `NB` でも `UN` でも `ST` でもないところで積まれた path である。すなわち `G(t, [])` は
     `{ p : p は t の UNST-道で cls(end(t, p)) が BX か AR }` と
     `{ p ++ [c] : p は t の UNST-道で cls(end(t, p)) = CL }` の合併である。
    BY <1>8, <2>3, DEF UNST-道, DEF fld
  <2>5. 分解の一意性。`lam` が `L(t)` の要素で `lam = p ++ e` が `<2>4` の分解であるとする。`q` が
     `lam` の別の前置で `t` の UNST-道であり `cls(end(t, q))` が `UN` でも `ST` でもないとする。
     `|q|` が `|p|` 未満なら、`p` が UNST-道であることから `cls(end(t, q)) = cls(s_{|q|})` は `UN` か
     `ST` であり、仮定に反する。`|q|` が `|p|` より大きいなら、`q` が UNST-道であることから
     `cls(s_{|p|}) = cls(end(t, p))` は `UN` か `ST` であり、`<2>4` の `cls(end(t, p))` が `BX`、
     `AR`、`CL` のどれかであることに反する。よって `|q| = |p|`、すなわち `q = p` である。
    BY <2>4, DEF UNST-道
  <2>6. QED
    BY <2>1, <2>4, <2>5

<1>14a. `<1>1` を満たす型 `s` と任意の path `q` について、`go(s, E, q, out)` が `out` に積む path の
   集合は `{ q ++ r : r は L(s) の要素 }` である。
  <2>1. `boxed_leaf_paths(s, E)` は `out` を空にして `go(s, E, &mut Vec::new(), &mut out)` を呼び
     `out` を返すので、`L(s)` は `go(s, E, [], out)` が積む path の集合である。`<1>11` の表を
     `path = []` に読むと、それは `cls(s)` に応じて次のとおりである。`NB` のとき空集合、
     `CL` のとき `{[c]}`、`BX` か `AR` のとき `{[]}`、`UN` か `ST` のとき `F(s)` の各 `(i, f)` に
     ついて `go(f, E, [i], out)` が積むものの合併。
    BY <1>11, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  <2>2. CASE `cls(s)` が `NB`、`CL`、`BX`、`AR` のどれか。`<1>11` の表より `go(s, E, q, out)` が積む
     のは、`NB` のとき何も無く、`CL` のとき `q ++ [c]` だけ、`BX` か `AR` のとき `q` だけである。
     `<2>1` の `L(s)` と突き合わせると、どの場合も主張の等式が成り立つ。
    BY <1>11, <2>1
  <2>3. CASE `cls(s)` が `UN` か `ST`。`<1>11` の表より `go(s, E, q, out)` が積むのは、`F(s)` の各
     `(i, f)` についての `go(f, E, q ++ [i], out)` が積むものの合併である。`<1>1` (ii) より `f` も
     `<1>1` を満たすので、`DESC` に関する整礎帰納法 (`<1>8`) の仮定を `f` と path `q ++ [i]` に
     適用すると、それは `{ q ++ [i] ++ r' : r' は L(f) の要素 }` である。同じ仮定を `f` と path `[i]`
     に適用すると `go(f, E, [i], out)` が積むものは `{ [i] ++ r' : r' は L(f) の要素 }` であり、
     `<2>1` よりその合併が `L(s)` である。よって主張の等式が成り立つ。
    BY <1>1, <1>8, <1>11, <2>1, DEF DESC
  <2>4. QED
    `<1>4` より `cls(s)` は 6 つのどれかであり、`<2>2` と `<2>3` がそれを尽くしている。
    BY <1>4, <2>2, <2>3

<1>15. `<1>1` を満たす型 `t` について、`p` が `t` の UNST-道であり、`r` が `L(end(t, p))` の要素で
   あるとき、`p ++ r` は `L(t)` の要素である。
  <2>1. `s := end(t, p)` と置く。`s` は `t` からフィールドの辺を `|p|` 回辿って着く型なので、`<1>1`
     (ii) より `s` も `<1>1` を満たす。`<1>14` を `s` に適用すると `r = q ++ e` と書ける。ここで `q`
     は `s` の UNST-道で、`cls(end(s, q))` は `BX`、`AR`、`CL` のどれかであり、`CL` のとき `e = [c]`、
     他のとき `e = []` である。
    BY <1>1, <1>14, DEF UNST-道, DEF fld
  <2>2. CASE `|q|` が 0 より大きい。`q` が `s` の UNST-道であることを `j = 0` に適用すると `cls(s)` は
     `UN` か `ST` である。したがって `p ++ q` は `t` の UNST-道である。位置 `j` (`j` は `|p|` 未満) の
     型は `p` が `t` の UNST-道であることから `UN` か `ST`、位置 `|p|` の型は `s` で `UN` か `ST`、
     位置 `|p| + j` (`j` は 1 以上 `|q|` 未満) の型は `q` の位置 `j` の型で `UN` か `ST` である。
    BY <2>1, DEF UNST-道
  <2>3. CASE `|q|` が 0。このとき `p ++ q` は `p` に等しく、`p` は `t` の UNST-道である。
    BY DEF UNST-道
  <2>4. `<2>2` と `<2>3` のどちらの場合も `end(t, p ++ q) = end(s, q)` である。`fld` の連鎖を位置
     `|p|` で切ると、前半が `p` の連鎖、後半が `q` の連鎖になる。
    BY <2>2, <2>3, DEF UNST-道, DEF fld
  <2>5. QED
    `<2>2` と `<2>3` は `|q|` が 0 かどうかで尽きている。`<2>4` より `p ++ q` は `t` の UNST-道で、
    `cls(end(t, p ++ q))` は `<2>1` の `cls(end(s, q))` に等しく `BX`、`AR`、`CL` のどれかである。
    `<1>14` の特徴づけを `t` に適用すると `p ++ q ++ e`、すなわち `p ++ r` は `L(t)` の要素である。
    BY <1>14, <2>1, <2>2, <2>3, <2>4

<1>16. `<1>1` を満たす型 `s` について `cls(s)` が `NB` でないとき、`L(s)` は空でない。
  <2>1. CASE `cls(s) = CL`。`<1>14` で `p = []` を取ると `end(s, [])` は `s` でそのクラスは `CL` なので、
     `[c]` は `L(s)` の要素である。
    BY <1>14, DEF UNST-道
  <2>2. CASE `cls(s)` が `BX` か `AR`。`<1>14` で `p = []` を取ると `[]` は `L(s)` の要素である。
    BY <1>14, DEF UNST-道
  <2>3. CASE `cls(s)` が `UN` か `ST`。
    <3>1. `<1>7` (b) より `is_fully_unboxed(s)` の値は `F(s)` の各要素の第 2 成分についての
       `is_fully_unboxed` の連言に等しく、その連言は偽である。よってある `(i, f)` が `F(s)` の要素で
       `f.is_fully_unboxed(E)` が偽である。
      BY <1>7
    <3>2. `<3>1` の `f` は `cls(f)` が `NB` でないことを満たす。
      BY <3>1, DEF cls
    <3>3. `s DESC f` である。
      BY <3>1, DEF DESC
    <3>4. `<1>1` (ii) より `f` も `<1>1` を満たす。`DESC` に関する整礎帰納法 (`<1>8`) の仮定を `f` に
       適用すると `L(f)` は空でない。その要素を `r` とする。
      BY <1>1, <1>8, <3>2, <3>3
    <3>5. `[i]` は `s` の UNST-道であり `end(s, [i]) = f` である。`j = 0` の唯一の条件は `cls(s)` が
       `UN` か `ST` であることで、これは場合の仮定である。
      BY <3>1, DEF UNST-道, DEF fld
    <3>6. QED
      `<1>15` を `t := s`、`p := [i]`、`r := r` に適用すると、`[i] ++ r` は `L(s)` の要素である。
      よって `L(s)` は空でない。
      BY <1>15, <3>4, <3>5
  <2>4. QED
    `<1>4` より `cls(s)` は 6 つのどれかであり、仮定から `NB` は除かれる。残る 5 つを `<2>1`、`<2>2`、
    `<2>3` が尽くしている。
    BY <1>4, <2>1, <2>2, <2>3

<1>17. `t` は `<1>1` を満たし、`lam` は `L(t)` の要素であるとする。`<1>14` により `lam` は
   `lam = p ++ e` と一意に分解される
   (`p` は `t` の UNST-道、`cls(end(t, p))` は `CL`、`BX`、`AR` のどれか、`e` は `CL` のとき `[c]`、
   他のとき `[]`)。`s_j` を `lam[0..j]` の位置の型 (`j` は `|p|` 以下)、`k` を「`cls(s_j)` が `ST`
   でない `j` (`|p|` 以下) のうち最小のもの」とすると、次の 3 つが成り立つ。
   - (i) `k` は定義され、`k` は `|p|` 以下であり、`|p|` は `|lam|` 以下である。また `j` が `k` 未満の
     すべてで `cls(s_j) = ST` である。
   - (ii) `k` が `|p|` 未満のとき `cls(s_k) = UN` であり、`k = |p|` のとき `cls(s_k)` は `CL`、`BX`、
     `AR` のどれかである。
   - (iii) `T(t, lam)` は abort せずに値を返す。すなわち `<1>9` が挙げる 3 つの abort の場所 --
     `UnitStep::NoUnit` の腕の `panic!` (「holds no reference」)、`UnitStep::Capture` の腕の
     `assert_eq!`、`held_field_type` の `panic!` -- のどれにも達しない。返り値は、
     `cls(s_k)` が `UN`、`BX`、`AR` のとき `lam[0..k]`、`cls(s_k)` が `CL` のとき `lam[0..k+1]` で
     ある。
  <2>1. (i) が成り立つ。`cls(s_{|p|})` は `cls(end(t, p))` に等しく、`<1>14` より `CL`、`BX`、`AR` の
     どれかであって `ST` ではない。よって `k` を取る対象の集合は `|p|` を含み空でなく、`k` は定義
     されて `|p|` 以下である。`e` は `[]` か `[c]` なので `|p|` は `|lam|` 以下である。`k` の最小性
     から、`j` が `k` 未満のすべてで `cls(s_j) = ST` である。
    BY <1>14
  <2>2. (ii) が成り立つ。`j` が `|p|` 未満のとき、`p` が `t` の UNST-道であることから `cls(s_j)` は
     `UN` か `ST` である。`k` が `|p|` 未満なら `cls(s_k)` は `ST` でないので `UN` である。`k = |p|`
     なら `cls(s_k)` は `cls(end(t, p))` に等しく、`<1>14` より `CL`、`BX`、`AR` のどれかである。
    BY <1>14, DEF UNST-道
  <2>3. `truncate_to_unit(t, lam, E)` は `out := []`、`cur := t` から始め、`lam` の各要素 `idx` に
     ついて `unit_step(cur, E)` で場合分けする `for` ループを回し、ループを抜けたら `out` を返す。
     場合分けは `UnitStep::NoUnit` のとき `panic!`、`UnitStep::Capture { capture_idx, .. }` のとき
     `assert_eq!(idx, capture_idx, ...)` と `out.push(idx)` と `break`、`UnitStep::Unit` のとき
     `break` (`out` に積まない)、`UnitStep::Fields { held_fields, .. }` のとき `out.push(idx)` と
     `cur = held_field_type(&held_fields, idx, "truncate_to_unit")` である。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>4. `j` が 0 以上 `k` 以下のすべてについて、ループの最初の `j` 周はすべて `UnitStep::Fields` の腕
     を通って完了し、その時点で `cur = s_j` かつ `out = lam[0..j]` である。
    <3>1. `j = 0` のとき。0 周が完了した時点で `cur = t = s_0` かつ `out = [] = lam[0..0]` である。
      BY <2>3
    <3>2. ASSUME: `j` は `k` 未満であり、最初の `j` 周がすべて `Fields` の腕を通って完了し、その時点で
       `cur = s_j` かつ `out = lam[0..j]` である
       PROVE: 最初の `j+1` 周がすべて `Fields` の腕を通って完了し、その時点で `cur = s_{j+1}` かつ
       `out = lam[0..j+1]` である
      <4>1. `j` は `k` 未満で `k` は `|p|` 以下、`|p|` は `|lam|` 以下なので、`j` は `|lam|` 未満で
         ある。よってループは第 `j` 周に入る。
        BY <2>1
      <4>2. `<2>1` より `cls(s_j) = ST` である。
        BY <2>1
      <4>3. `<1>10` と `<4>2` より `unit_step(s_j, E)` は
         `UnitStep::Fields { held_fields: F(s_j), .. }` である。
        BY <1>10, <4>2
      <4>4. 第 `j` 周の `idx` は `lam[j]` である。この帰納段の仮定より `j` は `k` 未満であり、
         `<2>1` より `k` は `|p|` 以下なので、`j` は `|p|` 未満である。`lam = p ++ e` より `lam` の
         先頭 `|p|` 要素は `p` に等しいので `lam[j] = p[j]` であり、`p` が `t` の UNST-道であることから
         `(lam[j], s_{j+1})` は `F(s_j)` の要素である。
        BY <2>1, DEF UNST-道, DEF fld
      <4>5. `s_j` は `t` からフィールドの辺を `j` 回辿って着く型なので `<1>1` (ii) より `<1>1` を
         満たす。`<1>12`、`<4>3`、`<4>4` より
         `held_field_type(&F(s_j), lam[j], "truncate_to_unit")` は abort せず `s_{j+1}` を返す。
        BY <1>1, <1>12, <4>3, <4>4
      <4>6. QED
        `<2>3` の `Fields` の腕により、第 `j` 周の後 `out` は `lam[0..j] ++ [lam[j]] = lam[0..j+1]`
        になり、`cur` は `s_{j+1}` になる。
        BY <2>3, <4>1, <4>3, <4>5
    <3>3. QED
      `<3>1` を基底、`<3>2` を帰納段とする `j` についての帰納法。
      BY <3>1, <3>2
  <2>5. CASE `k = |lam|`。
    <3>1. ループは第 `k` 周に入らずに終わる。`<2>4` を `j = k` に適用すると、その時点で
       `out = lam[0..k] = lam` である。
      BY <2>4
    <3>2. `k` は `|p|` 以下で `|p|` は `|lam| = k` 以下なので `|p| = |lam|`、すなわち `e = []` かつ
       `k = |p|` である。`<2>2` より `cls(s_k)` は `CL`、`BX`、`AR` のどれかである。
      BY <2>1, <2>2
    <3>3. `cls(s_k) = CL` ではない。`cls(end(t, p)) = CL` なら `<1>14` の分解は `e = [c]` を与え、
       `<3>2` の `e = []` に反する。
      BY <1>14, <3>2
    <3>4. QED
      `<3>2` と `<3>3` より `cls(s_k)` は `BX` か `AR` であり、(iii) の第 1 の式は
      `T(t, lam) = lam[0..k] = lam` を与える。`<3>1` よりこれは実際の返り値である。ループの中で
      `panic!` にも `assert_eq!` にも達していない。
      BY <3>1, <3>2, <3>3
  <2>6. CASE `k` が `|lam|` 未満。
    <3>1. `<2>4` を `j = k` に適用すると、ループは第 `k` 周に入り、その時点で `cur = s_k`、
       `out = lam[0..k]`、`idx = lam[k]` である。
      BY <2>4
    <3>2. `cls(s_k)` は `ST` でも `NB` でもない。`ST` でないのは `k` の定義から、`NB` でないのは
       `<2>2` (値は `UN`、`CL`、`BX`、`AR` のどれか) からである。
      BY <2>2
    <3>3. CASE `cls(s_k)` が `UN`、`BX`、`AR` のどれか。`<1>10` より
       `unit_step(s_k, E) = UnitStep::Unit` であり、`<2>3` のその腕は `out` に積まずに `break` する。
       よって返り値は `lam[0..k]` であり、(iii) の第 1 の式と一致する。この腕は `panic!` も
       `assert_eq!` も持たない。
      BY <1>10, <2>3, <3>1
    <3>4. CASE `cls(s_k) = CL`。
      <4>1. `<2>2` より `k = |p|` である。`k` が `|p|` 未満なら `cls(s_k) = UN` となり、この場合の
         仮定に反するからである。
        BY <2>2
      <4>2. `<4>1` より `cls(end(t, p)) = cls(s_{|p|}) = CL` である。`<1>14` の分解はこのとき
         `e = [c]` を与えるので、`lam = p ++ [c]`、`|lam| = |p| + 1 = k + 1`、`lam[k] = c` である。
        BY <1>14, <4>1
      <4>3. `<1>10` より `unit_step(s_k, E)` は `UnitStep::Capture { capture_idx: c, .. }` である。
        BY <1>10
      <4>4. `<2>3` の `Capture` の腕は `assert_eq!(idx, capture_idx, ...)` を置き、`out.push(idx)`
         して `break` する。`<3>1` と `<4>2` より `idx = lam[k] = c` であり、`<4>3` より
         `capture_idx = c` なので `assert_eq!` は通る。
        BY <2>3, <3>1, <4>2, <4>3
      <4>5. QED
        返り値は `lam[0..k] ++ [c] = lam[0..k+1]` であり、(iii) の第 2 の式と一致する。`<4>4` より
        `assert_eq!` は通り、`panic!` の腕には達しない。
        BY <3>1, <4>4
    <3>5. QED
      `<3>2` より `cls(s_k)` は `UN`、`BX`、`AR`、`CL` のどれかで、`<3>3` と `<3>4` がそれを尽くして
      いる。`truncate_to_unit` が `panic!` する腕は `UnitStep::NoUnit`、すなわち `cls(s_k) = NB` の
      ときだけで (`<1>10`)、`<3>2` がそれを排除している。
      BY <1>10, <3>2, <3>3, <3>4
  <2>7. QED
    `k` は `|p|` 以下で `|p|` は `|lam|` 以下なので、`<2>5` と `<2>6` で場合は尽きている。`<1>9` より
    ループは停止し、abort するとすれば `UnitStep::NoUnit` の腕の `panic!`、`UnitStep::Capture` の腕の
    `assert_eq!`、`held_field_type` の `panic!` のいずれかである。3 つ目は `<2>4` が排除している --
    最初の `k` 周が `Fields` の腕を**通って完了する**と述べるからで、その腕で abort する場所は
    `held_field_type` だけである。残る 2 つは `<2>5` と `<2>6` が排除している。(i) は `<2>1`、(ii) は
    `<2>2`、(iii) は `<2>5` と `<2>6` である。
    BY <1>9, <2>1, <2>2, <2>4, <2>5, <2>6,
       CODE src/rc_ir/ownership.rs: truncate_to_unit

<1>18. **(P1 の前半)** `<1>1` を満たす任意の型 `t` と `L(t)` の任意の要素 `lam` について、`T(t, lam)`
   は `U(t)` の要素である。
  <2>1. `<1>17` の記号 (`p`、`e`、`s_j`、`k`) を使う。
    BY <1>17
  <2>2. CASE `k` が `|p|` 未満。
    <3>1. `<1>17` (ii) より `cls(s_k) = UN` である。
      BY <1>17
    <3>2. `<1>17` (iii) と `<3>1` より `T(t, lam) = lam[0..k]` である。
      BY <1>17, <3>1
    <3>3. `lam[0..k]` は `t` の ST-道である。`j` が `k` 未満のすべてで `cls(s_j) = ST` であることは
       `<1>17` (i) から従う。
      BY <1>17, DEF ST-道
    <3>4. `end(t, lam[0..k]) = s_k` であり、`<3>1` よりそのクラスは `UN` である。
      BY <3>1, <3>3, DEF ST-道
    <3>5. QED
      `<1>13` の第 1 の集合に `<3>3` と `<3>4` を当てはめると `lam[0..k]` は `U(t)` の要素である。
      `<3>2` よりこれは `T(t, lam)` に等しい。
      BY <1>13, <3>2, <3>3, <3>4
  <2>3. CASE `k = |p|`。
    <3>1. `p` は `t` の ST-道である。`j` が `|p| = k` 未満のすべてで `cls(s_j) = ST` であることは
       `<1>17` (i) から従う。
      BY <1>17, DEF ST-道
    <3>2. `end(t, p) = s_{|p|} = s_k` であり、`<1>17` (ii) よりそのクラスは `CL`、`BX`、`AR` の
       どれかである。
      BY <1>17, <3>1
    <3>3. CASE `cls(s_k)` が `BX` か `AR`。`<1>14` の分解より `e = []`、すなわち `lam = p` である。
       `<1>17` (iii) より `T(t, lam) = lam[0..k] = lam[0..|p|] = p = lam` である。`<1>13` の第 1 の
       集合に `<3>1` とこの場合の仮定を当てはめると `p` は `U(t)` の要素である。
      BY <1>13, <1>14, <1>17, <3>1
    <3>4. CASE `cls(s_k) = CL`。`<1>14` の分解より `e = [c]`、すなわち `lam = p ++ [c]` である。
       `<1>17` (iii) より `T(t, lam) = lam[0..k+1] = lam[0..|p|+1] = p ++ [c] = lam` である。`<1>13`
       の第 2 の集合に `<3>1` とこの場合の仮定を当てはめると `p ++ [c]` は `U(t)` の要素である。
      BY <1>13, <1>14, <1>17, <3>1
    <3>5. QED
      `<3>2` より `cls(s_k)` は `CL`、`BX`、`AR` のどれかで、`<3>3` と `<3>4` がそれを尽くしている。
      BY <3>2, <3>3, <3>4
  <2>4. QED
    `<1>17` (i) より `k` は `|p|` 以下なので `<2>2` と `<2>3` で場合は尽きている。
    BY <1>17, <2>1, <2>2, <2>3

<1>19. **(P1 の後半)** `<1>1` を満たす任意の型 `t` と `U(t)` の任意の要素 `u` について、
   `T(t, lam) = u` となる `L(t)` の要素 `lam` が存在する。
  <2>1. `<1>13` より `u` は次の 2 つのどちらかの形をしている。
     - (a) `u = p` で `p` は `t` の ST-道、`cls(end(t, p))` は `BX`、`AR`、`UN` のどれか。
     - (b) `u = p ++ [c]` で `p` は `t` の ST-道、`cls(end(t, p)) = CL`。
    BY <1>13
  <2>2. どちらの形でも `p` は `t` の UNST-道である。ST-道はどれも UNST-道だからである。
    BY <2>1, DEF ST-道, DEF UNST-道
  <2>3. どちらの形でも、`j` が `|p|` 未満のすべてで `cls(s'_j) = ST` である。ここで `s'_j` は
     `p[0..j]` の位置の型である。
    BY <2>1, DEF ST-道
  <2>4. CASE `u` が (a) の形で `cls(end(t, p))` が `BX` か `AR`。
    <3>1. `lam := p` と置く。`<1>14` の第 1 の集合に `<2>2` とこの場合の仮定を当てはめると `lam` は
       `L(t)` の要素である。
      BY <1>14, <2>2
    <3>2. `<1>17` を `lam` に適用する。`<1>14` の分解の一意性より、`lam` の分解の `p` はこの `p` に
       等しく `e = []` である。`lam` に沿う `s_j` は `s'_j` に等しい。
      BY <1>14, <3>1
    <3>3. `<2>3` と `<3>2` より、`j` が `|p|` 未満のすべてで `cls(s_j) = ST` であり、
       `cls(s_{|p|}) = cls(end(t, p))` はこの場合の仮定より `ST` でない。よって `k = |p|` である。
      BY <1>17, <2>3, <3>2
    <3>4. `<1>17` (iii) の第 1 の式より `T(t, lam) = lam[0..k] = lam[0..|p|] = p = u` である。
      BY <1>17, <3>3
    <3>5. QED
      BY <3>1, <3>4
  <2>5. CASE `u` が (b) の形。
    <3>1. `lam := p ++ [c]` と置く。これは `u` に等しい。`<1>14` の第 2 の集合に `<2>2` とこの場合の
       仮定を当てはめると `lam` は `L(t)` の要素である。
      BY <1>14, <2>2
    <3>2. `<1>17` を `lam` に適用する。`<1>14` の分解の一意性より、`lam` の分解の `p` はこの `p` に
       等しく `e = [c]` である。`lam` に沿う `s_j` は `s'_j` に等しい。
      BY <1>14, <3>1
    <3>3. `<2>3` と `<3>2` より、`j` が `|p|` 未満のすべてで `cls(s_j) = ST` であり、
       `cls(s_{|p|}) = cls(end(t, p)) = CL` は `ST` でない。よって `k = |p|` である。
      BY <1>17, <2>3, <3>2
    <3>4. `<1>17` (iii) の第 2 の式より `T(t, lam) = lam[0..k+1] = lam[0..|p|+1] = p ++ [c] = u` で
       ある。
      BY <1>17, <3>3
    <3>5. QED
      BY <3>1, <3>4
  <2>6. CASE `u` が (a) の形で `cls(end(t, p)) = UN`。この場合は D5 が挙げる 2 か所、すなわち unbox
     union と punched array の両方を覆う。
    <3>1. `s := end(t, p)` と置く。`s` は `t` からフィールドの辺を `|p|` 回辿って着く型なので、
       `<1>1` (ii) より `s` も `<1>1` を満たす。`<1>16` より `L(s)` は空でない。その要素を `r` と
       する。
      BY <1>1, <1>16, DEF ST-道, DEF fld
    <3>2. `r` は空でない。`<3>1` より `s` は `<1>1` を満たすので `<1>14` を `s` に適用でき、
       `r = q ++ e'` で `q` は `s` の UNST-道、
       `cls(end(s, q))` は `BX`、`AR`、`CL` のどれかである。`r = []` なら `q = []` かつ `e' = []` で
       あり、そのとき `end(s, q) = s` でそのクラスは `UN` となって `BX`、`AR`、`CL` のどれでもないので
       矛盾する。
      BY <1>14, <3>1
    <3>3. `lam := p ++ r` と置く。`<2>2` と `<3>1` から `<1>15` が使え、`lam` は `L(t)` の要素で
       ある。
      BY <1>15, <2>2, <3>1
    <3>4. `lam` に沿う位置 `j` (`j` は `|p|` 以下) の型は `s'_j` である。`lam` の先頭 `|p|` 要素は
       `p` に等しいからである。とくに `lam` に沿う位置 `|p|` の型は `s'_{|p|} = s` である。
      BY <3>3, DEF UNST-道, DEF fld
    <3>5. `lam` の `<1>14` の分解 `lam = p_lam ++ e_lam` について、`|p_lam|` は `|p|` より大きい。
       `<2>3` と場合の仮定より、`j` が `|p|` 以下のすべてで `cls(s'_j)` は `UN` か `ST` である。
       `<1>14` の一意性の言い換えより `p_lam` は「`lam` の前置のうち `t` の UNST-道であって
       `cls(end(t, p_lam))` が `UN` でも `ST` でもないもの」であり、`|p_lam|` が `|p|` 以下なら
       `cls(end(t, p_lam)) = cls(s'_{|p_lam|})` が `UN` か `ST` になって矛盾する。
      BY <1>14, <2>3, <3>3, <3>4
    <3>6. `<1>17` を `lam` に適用したときの `k` は `|p|` である。`<2>3` と `<3>4` より `j` が `|p|`
       未満のすべてで `cls(s_j) = ST` であり、`<3>4` と場合の仮定より `cls(s_{|p|}) = UN` は `ST` で
       ない。`<3>5` より `|p|` は `|p_lam|` 以下なので、`|p|` は `k` を取る対象の範囲に入る。
      BY <1>17, <2>3, <3>4, <3>5
    <3>7. `<1>17` (iii) の第 1 の式 (`cls(s_k) = UN` なので) より
       `T(t, lam) = lam[0..k] = lam[0..|p|] = p = u` である。
      BY <1>17, <3>4, <3>6
    <3>8. QED
      BY <3>3, <3>7
  <2>7. QED
    `<2>1` の 2 つの形のうち (a) を `<2>4` と `<2>6` が `cls(end(t, p))` の値で尽くし ((a) のクラスは
    `BX`、`AR`、`UN` のどれか)、(b) を `<2>5` が尽くしている。
    BY <2>1, <2>4, <2>5, <2>6

<1>19a. `t.is_closure()` が偽であり、かつ `t.toplevel_tycon()` が `None` を返すか返す型構成子が `E`
   に無いとき、`boxed_leaf_paths(t, E)` と `rc_units(t, E)` は値を返さない。すなわち `L(t)` も
   `U(t)` も定まらない。
  <2>1. `toplevel_tycon_info(t, E)` は `t.toplevel_tycon().unwrap()` と
     `type_env.tycons().get(&tycon).unwrap()` を置く。この場合の仮定はそのどちらかが発火することで
     ある。
    BY CODE src/ast/types.rs: TypeNode::toplevel_tycon_info
  <2>2. `t.is_unbox(E)` は `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` で
     ある。`is_closure()` が偽なので短絡せず、`<2>1` より abort する。`is_box` はその否定なので
     同じである。
    BY <2>1, CODE src/ast/types.rs: TypeNode::is_unbox, CODE src/ast/types.rs: TypeNode::is_box
  <2>3. `boxed_leaf_paths(t, E)` は内部関数 `go` を `t` に対して呼び、`go` の最初の文は
     `ty.is_fully_unboxed(type_env)` である。`is_fully_unboxed` の最初の文は `self.is_box(type_env)`
     なので、`<2>2` より abort する。
    BY <2>2, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`),
       CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>4. `rc_units(t, E)` は `rc_units_go(t, E, &mut vec![], &mut out)` を呼び、`rc_units_go` は
     最初に `unit_step(s, type_env)` を呼ぶ。`unit_step` の最初の文は
     `ty.is_fully_unboxed(type_env)` であり、`is_fully_unboxed` の最初の文は
     `self.is_box(type_env)` なので、`<2>2` より abort する。
    BY <2>2, CODE src/rc_ir/ownership.rs: rc_units, CODE src/rc_ir/ownership.rs: rc_units_go,
       CODE src/rc_ir/ownership.rs: unit_step, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>5. QED
    BY <2>3, <2>4

<1>20. **P1 が `<1>1` を満たす型について成り立つ。** すなわちそのような型 `t` について、`L(t)` の各
   要素 `lam` の `T(t, lam)` は `U(t)` の要素であり、`U(t)` の各要素 `u` はある `L(t)` の要素の
   `T(t, ・)` である。

   何らかの制限が要ることは `<1>19a` が言う。`t.is_closure()` が偽で型構成子が `E` に無い型に
   ついては `L(t)` も `U(t)` も定まらず、P1 の言明の 2 つの辺が意味を持たない。

   README の P1 は「A10 を満たす任意の型 `τ` について」と書く。`<1>1` は A10 の (i)(ii) に (iii) を
   足したものなので、README の P1 が量化する型のうち (iii) を満たすものについて、`<1>20` は README の
   P1 そのものを与える。第 3 節がこの差を述べる。
  BY <1>18, <1>19, <1>19a

<1>21. `VarTable::of(func)` または `VarTable::body_only(body)` が作る表 `vars` について、
   `vars.bindings` への挿入は 1 つの有限列をなし、その列に現れる名前は相異なる。`ord(y)` を、`y` が
   その列に現れるときはその位置、現れないときは `-1` と定める。
  <2>1. `VarTable::of(func)` は、`func.params` と `func.capture` の各要素について
     `vars.bindings.insert(p.name.clone(), Binding::Param)` を行い、そののち
     `collect_bindings(&func.body, &mut vars)` を呼ぶ。
    BY CODE src/rc_ir/ownership.rs: VarTable::of
  <2>2. `VarTable::body_only(body)` は `collect_bindings(body, &mut vars)` だけを呼ぶ。
    BY CODE src/rc_ir/ownership.rs: VarTable::body_only
  <2>3. `collect_bindings` は本体の木を辿り、有限個の `vars.bindings.insert` を行う。D2 より本体は
     有限の木である。
    BY D2, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>4. `<2>1` から `<2>3` より挿入は有限列をなす。挿入される名前は、パラメータ、capture、`Let` の
     束縛変数、`Destructure` のフィールド変数、`Match` のアームの payload であり、どれもプログラムの
     束縛変数である。A6 よりこれらの名前は相異なる。
    BY A6, <2>1, <2>2, <2>3, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>5. QED
    BY <2>4

<1>21a. `VarTable::of(func)` または `VarTable::body_only(body)` が作る表 `vars` について、
   `vars.bindings` の定義域は `vars.var_tys` の定義域に含まれ、`u` がその定義域にあるとき
   `vars.var_tys[u]` は `u` を束縛する `RcVar` の `ty`、すなわち `<1>3a` (vi) の `ty(u)` である。
  <2>1. `VarTable::of` は各パラメータ・capture について
     `vars.bindings.insert(p.name, Binding::Param)` と `vars.var_tys.insert(p.name, p.ty)` を
     隣り合わせで行い、そののち `collect_bindings` を呼ぶ。`VarTable::body_only` は
     `collect_bindings` だけを呼ぶ。
    BY CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only
  <2>2. `collect_bindings` が `vars.bindings.insert` を行うのは 3 か所であり、どれも同じ名前に
     ついての `vars.var_tys.insert` を伴う。`RcExpr::Let(x, rhs, k)` の腕は `x.name` を `x.ty` と
     ともに入れ、その `rhs` が `RcRhs::Match(scrut, arms)` のときは各 `arm.payload.name` を
     `arm.payload.ty` とともに入れ、`RcExpr::Destructure(container, fields, _state, k)` の腕は
     `fields` の各 `fv.name` を `fv.ty` とともに入れる。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>3. QED
    `<2>1` と `<2>2` が `vars.bindings` への挿入をすべて挙げており、そのどれもが同じ名前を
    `vars.var_tys` へ、その名前を束縛する `RcVar` の `ty` とともに入れる。`<1>3a` (vi) より、その
    名前を持つ `RcVar` の型はどの出現でも同じなので、記録される型は `ty(u)` である。
    BY <1>3a, <2>1, <2>2

<1>22. `collect_bindings` の各節点での挿入の順序は次の通りである。節点 `n` の呼び出しが行う挿入の全体を
   `Ins(n)` と書くと、`Ins(n)` は `<1>21` の列の連続する区間を占め、`n` の子の `Ins` は `Ins(n)` に
   含まれる。
   - `n = Ret(v)`: `Ins(n)` は空である。
   - `n = Let(x, Match(scrut, arms), k)`: 各アームについて順に `arm.payload` を挿入してから
     `Ins(arm.body)` を行い、それがすべて済んでから `x` を挿入し、そののち `Ins(k)` を行う。
   - `n = Let(x, rhs, k)` (`rhs` が `Match` でない): `x` を挿入し、そののち `Ins(k)` を行う。
   - `n = Destructure(cont, fields, s, k)`: `fields` の各変数を順に挿入し、そののち `Ins(k)` を行う。
   - `n` が `Retain(v, p, s, k)`、`Release(v, p, s, k)`、`Eval(v, k)`: `Ins(n)` は `Ins(k)` に
     等しい。
  <2>1. `RcExpr::Ret(_)` の腕は何もしない。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>2. `RcExpr::Let(x, rhs, k)` の腕は、まず `rhs` に応じて `binding` を作り、そののち
     `vars.bindings.insert(x.name.clone(), binding)`、`vars.var_tys.insert(...)`、
     `collect_bindings(k, vars)` を行う。`rhs` が `RcRhs::Match(scrut, arms)` のときの `binding` の
     構成は、各 `arm` について
     `vars.bindings.insert(arm.payload.name.clone(), Binding::Payload(scrut.clone(), arm.tag))`、
     `vars.var_tys.insert(...)`、`collect_bindings(&arm.body, vars)`、
     `arm_results.push(returned_var(&arm.body).clone())` を順に行い、最後に
     `Binding::Join(arm_results)` を返す。`rhs` が他の 4 つの形のときは `vars.bindings` への挿入を
     行わない。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>3. `RcExpr::Destructure(container, fields, _state, k)` の腕は、`fields` の各 `(idx, fv)` に
     ついて `vars.bindings.insert(fv.name.clone(), Binding::Field(container.clone(), *idx))` と
     `vars.var_tys.insert(...)` を行い、そののち `collect_bindings(k, vars)` を行う。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>4. `RcExpr::Retain(..)`、`RcExpr::Release(..)`、`RcExpr::Eval(..)` の腕は
     `collect_bindings(k, vars)` だけを行う。
    BY CODE src/rc_ir/ownership.rs: collect_bindings
  <2>5. `<2>1` から `<2>4` より、各節点の呼び出しは自分の挿入と子の呼び出しだけを行い、他の節点の
     呼び出しの途中に割り込まない。よって `Ins(n)` は連続区間であり、子の `Ins` はその中に含まれる。
    BY <2>1, <2>2, <2>3, <2>4
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>23. 本体の各節点 `n` について、`Scope(n)` の要素であり `<1>21` の列に現れる名前はどれも、`Ins(n)`
   が始まるより前に挿入されている。
  <2>1. `n` が根のとき。`VarTable::of` の場合 `Scope(根)` はパラメータと capture の名前で、これらは
     `collect_bindings` を呼ぶ前に挿入される。`VarTable::body_only` の場合 `Scope(根)` は空集合で
     ある。
    BY <1>21, DEF Scope, CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only
  <2>2. ASSUME: `n` について主張が成り立ち、`n'` は `DEF Scope` が `Scope(n')` を `Scope(n)` から
     定める子である
     PROVE: `n'` について主張が成り立つ
    <3>1. CASE `n = Let(x, rhs, k)` かつ `n' = k`。`Scope(k)` は `Scope(n)` に `x` を加えたもので
       ある。`Scope(n)` の名前は帰納法の仮定より `Ins(n)` の開始より前に挿入されており、`Ins(n)` の
       開始は `Ins(k)` の開始より前である。`x` は `<1>22` より `Ins(k)` が始まる直前に挿入される。
      BY <1>22, DEF Scope
    <3>2. CASE `n = Let(x, Match(scrut, arms), k)` かつ `n'` が `arms` のあるアームの `body`。
       `Scope(arm.body)` は `Scope(n)` に `arm.payload` を加えたものである。`Scope(n)` の名前は
       帰納法の仮定より `Ins(n)` の開始より前に挿入されており、`Ins(n)` の開始は `Ins(arm.body)` の
       開始より前である。`arm.payload` は `<1>22` より `Ins(arm.body)` が始まる直前に挿入される。
      BY <1>22, DEF Scope
    <3>3. CASE `n = Destructure(cont, fields, s, k)` かつ `n' = k`。`Scope(k)` は `Scope(n)` に
       `fields` の各変数を加えたものである。`Scope(n)` の名前は帰納法の仮定より `Ins(n)` の開始より
       前に挿入されており、`Ins(n)` の開始は `Ins(k)` の開始より前である。`fields` の各変数は
       `<1>22` より `Ins(k)` が始まる前に挿入される。
      BY <1>22, DEF Scope
    <3>4. CASE `n` が `Retain(v, p, s, k)`、`Release(v, p, s, k)`、`Eval(v, k)` のどれかで `n' = k`。
       `Scope(k)` は `Scope(n)` に等しく `Ins(n)` は `Ins(k)` に等しいので、帰納法の仮定がそのまま
       主張である。
      BY <1>22, DEF Scope
    <3>5. QED
      `DEF Scope` が `Scope` を定める親子の組はこの 4 つで尽きている。
      BY DEF Scope, <3>1, <3>2, <3>3, <3>4
  <2>3. QED
    `<2>1` を基底、`<2>2` を帰納段とする、根からの深さについての帰納法。
    BY <2>1, <2>2

<1>24. 本体の節点 `n'` が節点 `n` の部分木に属するとき、`Scope(n')` の各名前は、`Scope(n)` の要素で
   あるか、`Ins(n)` の中で挿入される。
  <2>1. `n' = n` のとき `Scope(n')` は `Scope(n)` に等しい。
    BY DEF Scope
  <2>2. ASSUME: `n''` は `n` の部分木に属し、`n''` について主張が成り立ち、`n'` は `DEF Scope` が
     `Scope(n')` を `Scope(n'')` から定める子である
     PROVE: `n'` について主張が成り立つ
    <3>1. `DEF Scope` の 4 つの形のどれでも、`Scope(n')` は `Scope(n'')` に、`n''` の節点が挿入する
       名前 (`Let` の `x`、`Match` のアームの `payload`、`Destructure` のフィールド変数) を 0 個
       以上加えたものである。
      BY DEF Scope
    <3>2. `<3>1` で加わる名前はどれも `Ins(n'')` の中で挿入される。
      BY <1>22, <3>1
    <3>3. `n''` は `n` の部分木に属するので `Ins(n'')` は `Ins(n)` に含まれる。
      BY <1>22
    <3>4. QED
      `Scope(n'')` の各名前は帰納法の仮定より `Scope(n)` の要素であるか `Ins(n)` の中で挿入され、
      `<3>2` で加わる名前は `<3>3` より `Ins(n)` の中で挿入される。
      BY <3>1, <3>2, <3>3
  <2>3. QED
    `<2>1` を基底、`<2>2` を帰納段とする、`n` からの深さについての帰納法。
    BY <2>1, <2>2

<1>25. `x` が `vars.bindings` の定義域の要素であり、`origin_inner(vars, E, x, pi)` がある `pi` に
   ついて `origin(vars, E, y, pi')` を呼ぶとき、`ord(y) < ord(x)` である。
  <2>1. `m(x)` を、`<1>21` の列で `x` を挿入する節点とする。`<1>22` より、`x` が `Binding::Param` で
     挿入されるときを除き、`m(x)` は次のどれかである。`x` を束縛する `Let` の節点、`x` をフィールド
     変数とする `Destructure` の節点、`x` を payload とするアームを持つ `Match` を右辺とする `Let` の
     節点。
    BY <1>21, <1>22
  <2>2. `origin_inner` は `vars.bindings.get(var)` で場合分けし、`None`、`Some(Binding::Param)`、
     `Some(Binding::Producer)` の腕では再帰呼び出しを行わない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. CASE `vars.bindings[x]` が `Binding::Move(y)` である。
    <3>1. `origin_inner` のこの腕は `origin(vars, type_env, &y.name, path)` を呼ぶ。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Move(y)` は `collect_bindings` が `Let(x, RcRhs::Var(y), k)` の腕で作る。よって
       `m(x)` はその `Let` の節点であり、`y` はその `rhs` が使う変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `y.name` は `Scope(m(x))` の要素であるか、`vars.bindings` の定義域に
       無い。
      BY <1>2, <3>2
    <3>4. CASE `y.name` が `vars.bindings` の定義域に無い。このとき `ord(y.name) = -1` であり、
       `ord(x)` は 0 以上なので `ord(y.name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `y.name` が `Scope(m(x))` の要素である。`<1>23` より `y.name` は `Ins(m(x))` が
       始まるより前に挿入されている。`<1>22` より `x` は `Ins(m(x))` の中で挿入される。よって
       `ord(y.name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>4. CASE `vars.bindings[x]` が `Binding::Llvm(llvm_gen, args, result_ty)` である。
    <3>1. `origin_inner` のこの腕が行う再帰呼び出しは 2 種類である。`decl.leaf_origins_at(path)` が
       単一の `Arg(j, p)` のときの `origin(vars, type_env, &args[j].name, &p)` と、そうでないときに
       呼ぶ `origin_from_leaves_under(vars, type_env, &decl, args, path, &here_identity)` の中の
       `origin(vars, type_env, &args[j].name, &unit)` である。どちらの呼び先も `args[j].name` で
       ある。
      BY CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>2. `Binding::Llvm(llvm_gen, args, x.ty)` は `collect_bindings` が
       `Let(x, RcRhs::Llvm(llvm_gen, args), k)` の腕で作る。よって `m(x)` はその `Let` の節点で
       あり、`args` の各要素はその `rhs` が使う変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `args[j].name` は `Scope(m(x))` の要素であるか、`vars.bindings` の定義域に
       無い。
      BY <1>2, <3>2
    <3>4. CASE `args[j].name` が `vars.bindings` の定義域に無い。`ord(args[j].name) = -1` であり
       `ord(x)` は 0 以上なので `ord(args[j].name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `args[j].name` が `Scope(m(x))` の要素である。`<1>23` より `Ins(m(x))` の開始より前に
       挿入されており、`<1>22` より `x` は `Ins(m(x))` の中で挿入される。よって
       `ord(args[j].name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>5. CASE `vars.bindings[x]` が `Binding::Field(container, idx)` である。
    <3>1. `origin_inner` のこの腕は、`container.ty.is_box(type_env)` が偽のときだけ
       `origin(vars, type_env, &container.name, &container_path)` を呼ぶ。真のときは `here()` を
       返して再帰しない。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Field(container, idx)` は `collect_bindings` が
       `Destructure(container, fields, _state, k)` の腕で作る。よって `m(x)` はその `Destructure` の
       節点であり、`container` はその節点が使う変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `container.name` は `Scope(m(x))` の要素であるか、`vars.bindings` の
       定義域に無い。
      BY <1>2, <3>2
    <3>4. CASE `container.name` が `vars.bindings` の定義域に無い。`ord(container.name) = -1` であり
       `ord(x)` は 0 以上なので `ord(container.name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `container.name` が `Scope(m(x))` の要素である。`<1>23` より `Ins(m(x))` の開始より前
       に挿入されており、`<1>22` より `x` は `Ins(m(x))` の中で挿入される。よって
       `ord(container.name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>6. CASE `vars.bindings[x]` が `Binding::Payload(scrut, variant)` である。
    <3>1. `origin_inner` のこの腕は、`variant` が `None` のとき
       `origin(vars, type_env, &scrut.name, path)` を、`Some(tag)` かつ `!scrut.ty.is_box(type_env)`
       のとき `origin(vars, type_env, &scrut.name, &scrut_path)` を呼ぶ。残りの `Some(_)` では
       `here()` を返して再帰しない。呼び先はどちらも `scrut.name` である。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Payload(scrut, arm.tag)` は `collect_bindings` が
       `Let(z, RcRhs::Match(scrut, arms), k)` の腕で、`arms` の各 `arm` について作る (`x` は
       `arm.payload.name`)。よって `m(x)` はその `Let` の節点であり、`scrut` はその `rhs` が使う
       変数である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `<1>2` (H2) より `scrut.name` は `Scope(m(x))` の要素であるか、`vars.bindings` の定義域に
       無い。
      BY <1>2, <3>2
    <3>4. CASE `scrut.name` が `vars.bindings` の定義域に無い。`ord(scrut.name) = -1` であり
       `ord(x)` は 0 以上なので `ord(scrut.name) < ord(x)` である。
      BY <1>21
    <3>5. CASE `scrut.name` が `Scope(m(x))` の要素である。`<1>23` より `Ins(m(x))` の開始より前に
       挿入されており、`<1>22` より `x`、すなわち `arm.payload.name` は `Ins(m(x))` の中で挿入
       される。よって `ord(scrut.name) < ord(x)` である。
      BY <1>22, <1>23
    <3>6. QED
      BY <3>1, <3>3, <3>4, <3>5
  <2>7. CASE `vars.bindings[x]` が `Binding::Join(arm_results)` である。
    <3>1. `origin_inner` のこの腕は `arm_results` の各要素 `arm_result` について
       `origin(vars, type_env, &arm_result.name, path)` を呼ぶ。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Join(arm_results)` は `collect_bindings` が
       `Let(x, RcRhs::Match(scrut, arms), k)` の腕で作り、`arm_results` の各要素は
       `returned_var(&arm.body)` である。よって `m(x)` はその `Let` の節点である。
      BY <1>22, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `returned_var(&arm.body)` は `arm.body` の中の `RcExpr::Ret(v)` の `v` である。すなわち
       `arm.body` の部分木にある `Ret` の節点が使う変数である。その節点を `rt` と書く。
      BY CODE src/rc_ir/ownership.rs: returned_var
    <3>4. `<1>2` (H2) より `arm_result.name` は `Scope(rt)` の要素であるか、`vars.bindings` の
       定義域に無い。
      BY <1>2, <3>3
    <3>5. CASE `arm_result.name` が `vars.bindings` の定義域に無い。`ord(arm_result.name) = -1` で
       あり `ord(x)` は 0 以上なので `ord(arm_result.name) < ord(x)` である。
      BY <1>21
    <3>6. CASE `arm_result.name` が `Scope(rt)` の要素である。
      <4>1. `rt` は `arm.body` の部分木に属するので、`<1>24` より `arm_result.name` は
         `Scope(arm.body)` の要素であるか、`Ins(arm.body)` の中で挿入される。
        BY <1>24, <3>3
      <4>2. `<1>22` より `x` の挿入は、すべてのアームの `arm.payload` の挿入と `Ins(arm.body)` が
         済んだ後に行われる。
        BY <1>22
      <4>3. CASE `arm_result.name` が `Scope(arm.body)` の要素である。`<1>23` より `Ins(arm.body)` の
         開始より前に挿入されている。`<4>2` より `x` の挿入は `Ins(arm.body)` が済んだ後なので、
         `ord(arm_result.name) < ord(x)` である。
        BY <1>23, <4>2
      <4>4. CASE `arm_result.name` が `Ins(arm.body)` の中で挿入される。`<4>2` より `x` の挿入は
         `Ins(arm.body)` が済んだ後なので、`ord(arm_result.name) < ord(x)` である。
        BY <4>2
      <4>5. QED
        BY <4>1, <4>3, <4>4
    <3>7. QED
      BY <3>1, <3>4, <3>5, <3>6
  <2>8. QED
    `Binding` の値は `Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join` の 7 つであり、
    `vars.bindings.get(var)` の結果は `None` を加えた 8 通りである。`<2>2` が `None`、`Param`、
    `Producer` を、`<2>3` から `<2>7` が残る 5 つを尽くしている。
    BY <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, CODE src/rc_ir/ownership.rs: Binding

<1>26. `origin_inner(vars, E, x, pi)` の 1 回の実行が行う `origin` の呼び出しは有限個である。
  <2>1. `None`、`Param`、`Producer` の腕は 0 個である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `Move(y)` の腕は 1 個、`Field` の腕は高々 1 個、`Payload` の腕は高々 1 個である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `Join(arm_results)` の腕は `arm_results` の長さだけ呼ぶ。`arm_results` は `collect_bindings`
     が `arms` の各要素について 1 つ積んだ有限の `Vec` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>4. `Llvm` の腕で `decl.leaf_origins_at(path)` が単一の `Arg(j, p)` のときは 1 個である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>5. `Llvm` の腕で `origin_from_leaves_under` を呼ぶときは、`operand_units` の要素数だけ呼ぶ。
     `operand_units` は `decl.leaf_origins_under(path)` が渡す各 `LeafOrigins` の各要素から作る
     `Set` である。`Provenance` は `LeafMap`、すなわち有限の `Map<FieldPath, LeafOrigins>` を包む値で
     あり、`LeafOrigins` は有限の `Set<LeafOrigin>` である。よって `operand_units` は有限である。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/provenance.rs: Provenance,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>27. `Origin` 型の任意の値 `o` について、`o.candidates()` は空でない列を返す。
  <2>1. `Origin::Exactly(p)` について `candidates()` は `vec![p]` を返す。これは長さ 1 である。
    BY CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>2. `Origin::Join { candidates, .. }` について `candidates()` は `candidates` の要素を集めた列を
     返す。
    BY CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>3. `Origin::Join` を作る場所は `Origin::of_candidates` の `_ => Origin::Join { .. }` の腕だけで
     ある。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, CODE src/rc_ir/ownership.rs: Origin
  <2>4. `of_candidates` はまず `assert!(!candidates.is_empty(), ...)` を置き、そののち
     `candidates.len()` で場合分けし、1 のとき `Origin::Exactly`、それ以外のとき `Origin::Join` を
     作る。`assert!` を通った時点で長さは 1 以上なので、`Origin::Join` を作るとき `candidates` の
     長さは 2 以上である。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>5. QED
    `Origin` の値は `Exactly` と `Join` の 2 つで、`<2>1` が前者を、`<2>2` から `<2>4` が後者を
    尽くしている。
    BY <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/ownership.rs: Origin

<1>27a. `Origin` 型の任意の値 `o` について、`o.acted_on()` は空でない列を返す。
  `acted_on` は `let mut out = vec![self.identity()];` で始め、そののち `out` を伸ばすだけなので、
  返り値は `o.identity()` を必ず含む。
  BY CODE src/rc_ir/ownership.rs: Origin::acted_on

<1>27b. `Provenance` を作る 4 つの関数 -- `Provenance::build_shape`、`Provenance::uniform`、
   `Provenance::uniform_bottom`、`Provenance::fresh_under` -- を `<1>1` を満たす型 `ty` に対して
   呼ぶとき、abort しうるのは `build_shape` に渡された閉包の中だけであり、その閉包が受け取る
   `path` は `L(ty)` の要素である。閉包を引数に取らない残りの 3 つは abort せず停止する。
  <2>1. `Provenance::build_shape(ty, E, leaf)` は `LeafMap::build_shape(ty, E, leaf)` を呼ぶ。
     `LeafMap::build_shape` は `boxed_leaf_paths(ty, E)` を 1 度呼び、返った各 `path` について
     `leaf(&path)` を呼び、対 `(path, fact)` を `Map` に積む。`<1>9` より `boxed_leaf_paths(ty)` は
     abort せず停止し、`L(ty)` は有限である。よって `build_shape` 自身は abort せず停止し、abort
     しうるのは閉包 `leaf` の中だけである。閉包が受け取る `path` は `L(ty)` の要素である。
    BY <1>9, CODE src/rc_ir/provenance.rs: Provenance::build_shape,
       CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape
  <2>2. `Provenance::uniform(ty, E, src)` は `LeafMap::uniform(ty, E, sole_origin(src))` を呼び、
     `LeafMap::uniform` は `build_shape` を閉包 `|_| fact.clone()` で呼ぶ。この閉包は引数を見ずに
     複製を返すだけである。`sole_origin` は 1 要素の `Set` を作るだけである。よって `<2>1` より
     abort しない。
    BY <2>1, CODE src/rc_ir/provenance.rs: Provenance::uniform,
       CODE src/rc_ir/leaf_map.rs: LeafMap::uniform, CODE src/rc_ir/provenance.rs: sole_origin
  <2>3. `Provenance::uniform_bottom(ty, E)` は `build_shape` を閉包 `|_| Set::default()` で呼ぶ。
     この閉包も引数を見ない。よって `<2>1` より abort しない。
    BY <2>1, CODE src/rc_ir/provenance.rs: Provenance::uniform_bottom
  <2>4. `Provenance::fresh_under(ty, E, path)` は `uniform(ty, E, LeafOrigin::Unknown)` を作り、その
     `set_leaves_under(path, LeafOrigin::Fresh)` を返す。`set_leaves_under` は
     `LeafMap::map_leaves_under` を呼び、これは `Map` の各要素について
     `leaf_path.starts_with(path)` を見て閉包を当てるか複製するかを選び、新しい `Map` を作るだけで
     ある。閉包は `|_| sole_origin(src.clone())` である。`starts_with` は `path` が `ty` の leaf で
     あるかどうかを問わない。よって `<2>2` より abort しない。
    BY <2>2, CODE src/rc_ir/provenance.rs: Provenance::fresh_under,
       CODE src/rc_ir/provenance.rs: Provenance::set_leaves_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::map_leaves_under
  <2>5. QED
    `<2>1` が `build_shape` を、`<2>2` から `<2>4` が残る 3 つを尽くしている。
    BY <2>1, <2>2, <2>3, <2>4

<1>27c. `<1>1` を満たす型 `t` について、`t.is_unbox(E)` と `t.is_box(E)` は abort せずに真偽値を
   返す。
  BY <1>3c

<1>28. `origin_inner` の `Llvm` の腕が呼ぶ `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` は
   abort せずに `Provenance` を返す。ここで `result_ty` と `arg_tys` は、その `Llvm` 節点の `ty(x)`
   と `args` の各要素の型であり、どれも RC IR に現れる型なので `<1>1` を満たす。以下の各腕で
   `<1>27b` と `<1>27c` を適用するのはこの型についてである。
  <2>1. `impl LLVMGen for` は 78 個あり、`result_prov` を override するのは 29 個である。残る 49 個は
     既定の実装を取る。
    BY A3, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov
  <2>2. 既定の実装は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` の 1 文であり、
     `<1>27b` より abort しない。
    BY <1>1, <1>27b, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov
  <2>2a. 29 個の override のうち 16 個の本体は `Provenance::uniform(result_ty, type_env, src)` の
     1 文であり、`src` は `LeafOrigin::Fresh` (13 個) か `LeafOrigin::Unknown` (3 個) である。
     `<1>27b` より abort しない。
    BY <1>1, <1>27b,
       CODE src/fixstd/builtin.rs: InlineLLVMStringBuf::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayUnsafeEmpty::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayTruncateBoundsUnchecked::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayAppendValueCapacityUnchecked::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArraySetCapacityBoundsUnchecked::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayAppendCapacityUnchecked::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayGrowSizeBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArraySetBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArraySwapBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMPunchedArrayPlugBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayLitBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMDestructorMake::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayIsStorageUniqueBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMMarkThreadedFunctionBody::result_prov
  <2>2b. 1 個 (`InlineLLVMUndefinedInternalBody`) の本体は
     `Provenance::uniform_bottom(result_ty, type_env)` の 1 文である。`<1>27b` より abort しない。
    BY <1>1, <1>27b, CODE src/fixstd/builtin.rs: InlineLLVMUndefinedInternalBody::result_prov
  <2>2c. 5 個の本体は `Provenance::fresh_under(result_ty, type_env, p)` の 1 文であり、`p` はその
     op が書いた定数の path である。`<1>27b` より abort しない。
    BY <1>1, <1>27b,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayPunchBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMUnsafeMutateBoxedInternalFunctionBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMUnsafeMutateBoxedIOSInternalBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayMutateElementsInternalBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMArrayMutateElementsIosInternalBody::result_prov
  <2>2d. 2 個 (`InlineLLVMMakeStructBody`、`InlineLLVMMakeUnionBody`) の本体は
     `Provenance::build_shape` を呼ぶ 1 つの式で終わる。`InlineLLVMMakeUnionBody` はその前に
     `let variant_idx = self.variant_index();` を置く。`variant_index` は `self.field_idx` を返す
     1 行で、abort する場所を持たない。渡す閉包は `path.split_first()` が返す `Option` で
     場合分けし、`None` の腕と `Some` の腕をすべて書いて `sole_origin(...)` か `Set::default()` を
     返す。閉包が行うのはこの場合分けと `Vec` の複製と `Set` の構成だけであり、添字付けも `unwrap`
     も `expect` も持たない。`<1>27b` より残りも abort しない。
    BY <1>1, <1>27b, CODE src/fixstd/builtin.rs: InlineLLVMMakeStructBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov,
       CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::variant_index
  <2>2e. 2 個 (`InlineLLVMStructGetBody`、`InlineLLVMUnionAsBody`) の本体は、`arg_tys[0]` を
     `is_box(type_env)` に掛けて分岐し、真なら `Provenance::uniform(result_ty, type_env,
     LeafOrigin::Unknown)` を返し、偽なら `let field_idx = self.field_index();` (`InlineLLVMUnionAsBody`
     では `let variant_idx = self.variant_index();`) を置いてから `Provenance::build_shape` を返す。
     この 2 つのメソッドはどちらも `self.field_idx` を返す 1 行で、abort する場所を持たない。
     `build_shape` に渡す閉包は 1 要素の `Vec` に `path` を継いで
     `sole_origin(LeafOrigin::Arg(0, ・))` を作るだけである。
    <3>1. この 2 つの op の `free_vars_mut` はどちらも 1 要素の `Vec` を返し、`free_vars` はその
       各要素を複製した `Vec` を返す。`<1>3a` (vii) より `args` の名前の列は `free_vars()` に
       等しいので `args` は 1 要素であり、`arg_tys` も 1 要素である。よって `arg_tys[0]` は
       範囲内である。
      BY <1>3a, CODE src/ast/inline_llvm.rs: LLVMGen::free_vars,
         CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::free_vars_mut,
         CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::free_vars_mut
    <3>2. `arg_tys[0]` は RC IR に現れる型なので `<1>1` を満たし、`<1>27c` より
       `arg_tys[0].is_box(type_env)` は abort しない。
      BY <1>1, <1>27c
    <3>3. QED
      閉包が行うのは `Vec` の連結と `sole_origin` だけであり、その前に置かれる `field_index` /
      `variant_index` はどちらも `self.field_idx` を返す 1 行である。
      BY <1>1, <1>27b, <3>1, <3>2,
         CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov,
         CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::field_index,
         CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov,
         CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::variant_index
  <2>2f. `InlineLLVMStructPunchBody::result_prov` は
     `result_ty.field_types(type_env)[PUNCHED_STRUCT_FIELD]` を取り、その `is_box(type_env)` が真の
     とき `Provenance::fresh_under(result_ty, type_env, &[PUNCHED_STRUCT_FIELD])` を返し、偽のとき
     `Provenance::build_shape` を閉包 `|path| sole_origin(LeafOrigin::Arg(0, self.arg_leaf_path(path)))`
     で返す。この 1 個は abort しない。
    <3>1. `<1>3a` (vii) は `result_ty.field_types(type_env)` が長さ 2 の列を返すと述べる。
       `PUNCHED_STRUCT_FIELD` は 1 なので添字は範囲内である。
      BY <1>3a, CODE src/ast/types.rs: TypeNode::field_types,
         CODE src/fixstd/builtin.rs: PUNCHED_STRUCT_FIELD
    <3>2. `<3>1` が取り出す型は `<1>3a` (vii) より `<1>1` を満たすので、`<1>27c` より
       その `is_box(type_env)` は abort しない。
      BY <1>3a, <1>27c, <3>1
    <3>3. `build_shape` の閉包が受け取る `path` は `L(result_ty)` の要素である (`<1>27b`)。
       `<1>14` より空 path が `L(t)` に入るのは `cls(t)` が `BX` か `AR` のときだけであり、
       `DEF cls` よりその 2 つはそれぞれ `is_box` と `is_array` が真であることを要求する。
       `<1>3a` (vii) はどちらも偽だと述べる。よって `path` は空でなく、`arg_leaf_path` の
       `split_first` の `expect` は発火しない。
      BY <1>1, <1>3a, <1>14, <1>27b, DEF cls,
         CODE src/fixstd/builtin.rs: InlineLLVMStructPunchBody::arg_leaf_path
    <3>3a. `cls(result_ty)` は `UN` か `ST` である。`<1>3a` (vii) より `result_ty.is_box(E)` と
       `result_ty.is_array()` は偽である。`result_ty.is_closure()` も偽である -- `<3>1` より
       `result_ty.field_types(E)` は値を返し、`field_types` は `toplevel_tycon_info` を呼ぶので、
       その `assert!(!self.is_closure())` が通っている。`cls(result_ty) = NB` なら `<1>11` より
       `L(result_ty)` は空で `build_shape` の閉包は 1 度も呼ばれないので、以下は
       `cls(result_ty)` が `NB` でない場合を見ればよい。残るのは `UN` と `ST` である。
      BY <1>3a, <1>11, <3>1, DEF cls, CODE src/ast/types.rs: TypeNode::field_types,
         CODE src/ast/types.rs: TypeNode::toplevel_tycon_info
    <3>3b. `L(result_ty)` の要素のうち `PUNCHED_STRUCT_FIELD` で始まるものは、
       `(PUNCHED_STRUCT_FIELD, s)` が `F(result_ty)` の要素であるとき
       `{ [PUNCHED_STRUCT_FIELD] ++ r : r は L(s) の要素 }` であり、`F(result_ty)` が
       `PUNCHED_STRUCT_FIELD` を第 1 成分に持つ要素を持たないときは 1 つも無い。
       `<3>3a` と `<1>11` より `go(result_ty, [], out)` が積むのは `F(result_ty)` の各 `(i, f)` に
       ついての `go(f, [i], out)` が積むものの合併であり、`<1>1` (ii) より `f` は `<1>1` を満たすので
       `<1>14a` よりそれは `{ [i] ++ r : r は L(f) の要素 }` である。`<1>12` より `F(result_ty)` の
       添字は相異なるので、`PUNCHED_STRUCT_FIELD` で始まる要素はその添字を持つ 1 つの `(i, f)` から
       だけ来る。
      BY <1>1, <1>11, <1>12, <1>14a, <3>3a
    <3>4. `arg_leaf_path` の `assert_ne!` が発火するのは、`path` が
       `[PUNCHED_STRUCT_FIELD, self.field_idx] ++ ・` の形のときだけである。`<3>3b` より、そのような
       `path` が `L(result_ty)` に在るのは `(PUNCHED_STRUCT_FIELD, s)` が `F(result_ty)` の要素で
       あって `L(s)` が `self.field_idx` で始まる要素を持つときに限る。`<1>3c` (d) より `s` は
       `result_ty.field_types(E)[PUNCHED_STRUCT_FIELD]`、すなわち `<3>1` が取り出す型であり、
       `<1>3a` (vii) より `s` は構造体であって第 `self.field_idx` フィールドが穴である。
       `<1>1` (ii) より `s` は `<1>1` を満たすので `<1>11` を `s` に当てられる。`L(s)` の要素で
       空でないものは、`cls(s)` が `CL` なら `[c]` だけ (`s` は構造体なので `is_closure()` は偽で
       あり、この場合は起きない)、`UN` か `ST` なら `F(s)`、すなわち `unpunched_field_types` が返す
       フィールドの添字で始まる。穴はそこに入らない。よって `L(s)` に `self.field_idx` で始まる要素は
       無く、`assert_ne!` は発火しない。
      BY <1>1, <1>3a, <1>3c, <1>11, <3>1, <3>3b, DEF cls,
         CODE src/ast/types.rs: TypeNode::unpunched_field_types,
         CODE src/fixstd/builtin.rs: InlineLLVMStructPunchBody::arg_leaf_path
    <3>5. QED
      閉包の残りは `Vec` の連結と `sole_origin` だけである。
      BY <1>1, <1>27b, <3>1, <3>2, <3>3, <3>3a, <3>3b, <3>4,
         CODE src/fixstd/builtin.rs: InlineLLVMStructPunchBody::result_prov
  <2>2g. 2 個 (`InlineLLVMStructSetBody`、`InlineLLVMStructPlugInBody`) の本体は
     `replaced_field_prov(result_ty, type_env, field_idx, struct_arg, value_arg)` の 1 文である。
     `replaced_field_prov` は `result_ty.is_box(type_env)` が真なら
     `Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)` を返し、偽なら
     `Provenance::build_shape` を返す。渡す閉包は `path.split_first()` を `expect` で開き、
     `sole_origin(LeafOrigin::Arg(・, ・))` を作る。
    <3>1. `result_ty` は RC IR に現れる型なので `<1>1` を満たし、`<1>27c` より
       `result_ty.is_box(type_env)` は abort しない。
      BY <1>1, <1>27c
    <3>2. `build_shape` の腕に入るのは `result_ty.is_box(type_env)` が偽のときである。閉包が受け取る
       `path` は `L(result_ty)` の要素であり (`<1>27b`)、`<1>14` より空 path が `L(t)` に
       入るのは `cls(t)` が `BX` か `AR` のときだけである。`BX` は場合の仮定が退け、`AR` は
       `<1>3a` (vii) の `ty(x).is_array()` が偽であることが退ける。よって `path` は空でなく、
       `split_first` の `expect` は発火しない。
      BY <1>1, <1>3a, <1>14, <1>27b, DEF cls
    <3>3. QED
      閉包が行うのは `split_first` の `expect` と `Vec` の複製と `sole_origin` だけである。
      BY <1>1, <1>27b, <3>1, <3>2, CODE src/fixstd/builtin.rs: replaced_field_prov,
         CODE src/fixstd/builtin.rs: InlineLLVMStructSetBody::result_prov,
         CODE src/fixstd/builtin.rs: InlineLLVMStructPlugInBody::result_prov
  <2>3. QED
    `<2>1` より場合は既定の実装と 29 個の override で尽きている。`<2>2` が既定を、`<2>2a` から
    `<2>2g` が 16 + 1 + 5 + 2 + 2 + 1 + 2 = 29 個を尽くしている。
    BY <2>1, <2>2, <2>2a, <2>2b, <2>2c, <2>2d, <2>2e, <2>2f, <2>2g

<1>28a. `origin_inner` の `Llvm` の腕が得る `decl` は、各 leaf に要素数 0 か 1 の `LeafOrigins` を
   置く。したがって `decl.leaf_origins_at(path)` が返す集合と `decl.leaf_origins_under(path)` が
   渡す集合のどれかに `LeafOrigin::Arg(j, leaf)` が現れるとき、その leaf についての宣言は**単一の**
   `Arg(j, leaf)` であり、A3 の表の「単一の `Arg(j, σ)`」の行が当たる。すなわち `j` は `args.len()`
   未満であり、`leaf` は `L(ty(args[j]))` の要素である。
  <2>1. A3 は「複数の元を宣言する op は、このコミットのプログラムには存在しない」と述べ、その根拠を
     数え上げで与える -- `impl LLVMGen for` は 78 個、`result_prov` を override するのは 29 個、その
     29 個が leaf に置く集合はすべて要素数 0 か 1 である。
    BY A3
  <2>2. 残る 49 個が取る既定の実装は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)`
     であり、`uniform` は各 leaf に `sole_origin(LeafOrigin::Unknown)`、すなわち 1 要素の集合を
     置く。
    BY <2>1, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
       CODE src/rc_ir/provenance.rs: Provenance::uniform,
       CODE src/rc_ir/provenance.rs: sole_origin
  <2>3. QED
    `<2>1` と `<2>2` より、どの op の `decl` も leaf ごとに要素数 0 か 1 の集合を置く。要素数が 1 で
    その元が `Arg(j, leaf)` である leaf は、A3 の表の「単一の `Arg(j, σ)`」の行が扱うものであり、
    その行が「第 `j` オペランドの leaf `σ`」と述べる。第 `j` オペランドが存在しなければこの宣言は
    意味を持たないので `j` は `args.len()` 未満であり、`leaf` は第 `j` オペランドの leaf、すなわち
    `L(ty(args[j]))` の要素である。
    BY A3, <2>1, <2>2

<1>29. `origin(vars, E, x, pi)` の**呼び出しの木** -- 根をその呼び出しとし、各節点の子をその実行が
   行う `origin` の呼び出しとする木 -- は有限である。したがって `origin` は停止する。
  <2>1. `origin` の本体は、`vars.origins` の中に `(x, pi)` の答えがあればそれを複製して返し、無ければ
     `origin_inner` を `grow_stack` の中で呼び、その答えを `vars.origins` に記録して返す。
     `grow_stack(f)` が `f` をちょうど 1 回呼びその返り値を返すことは A15 である -- `grow_stack` の
     本体は `stacker::maybe_grow` への 1 行の委譲であり、閉包が何回呼ばれるかを決めるのは
     `stacker` crate である。
    BY A15, CODE src/rc_ir/ownership.rs: origin, CODE src/misc.rs: grow_stack
  <2>2. `origin` の呼び出しの木を考える。根は最初の呼び出しであり、節点 `origin(_, _, y, _)` の子は
     その実行が行う `origin` の呼び出しである。memo が当たった呼び出しは葉である。
    BY <2>1
  <2>3. `<1>25` より、この木の各辺 `origin(_, _, x, _)` から `origin(_, _, y, _)` について、`y` が
     `vars.bindings` の定義域に無いならその子は葉であり (`origin_inner` の `None` の腕は再帰しない)、
     定義域にあるなら `ord(y) < ord(x)` である。
    BY <1>25, CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. `<2>3` より、根から葉へ向かう任意の道の上で `ord` は狭義に減少する。`ord` の値は `-1` と
     `<1>21` の有限列の位置に限られるので、道の長さは `vars.bindings` の要素数に 2 を足した値で
     上から抑えられる。
    BY <1>21, <2>3
  <2>5. `<1>26` より木の各節点の子は有限個であり、`<2>4` より木の深さは有限である。有限分岐で深さが
     有限の木は有限なので、`origin` の呼び出しは有限回で終わる。
    BY <1>26, <2>4
  <2>6. `origin` の 1 回の実行が行う、`origin` 以外の計算は有限時間で終わる。`vars.origins` の探索と
     挿入、`Origin` の複製、`result_prov` の呼び出し (`<1>28`)、`origin_inner` の中の `unit_step` と
     `truncate_to_unit` の呼び出し (`<1>9` より停止する)、`Provenance` の有限の `Map` と `Set` の
     走査 (`<1>26`)、`args` の添字付けがその全部である。
    BY <1>9, <1>26, <1>28, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>7. QED
    BY <2>5, <2>6

<1>29a. 1 つの表 `vars` と型環境 `E` を固定する。`origin(vars, E, u, sig)` が値を返すとき、その値は
   `(u, sig)` で決まる。すなわち、`vars.origins` の状態がどうであれ、同じ `(u, sig)` についての
   2 回の呼び出しは等しい値を返す。
  <2>1. `vars.bindings` と `vars.var_tys` は、表を作り終えたのちは変わらない。この 2 つは
     `VarTable` の非公開フィールドであり、書き込むのは `VarTable::of`、`VarTable::body_only`、
     `collect_bindings` だけで、どれも表を作る間にしか走らない。`origin` は `&VarTable` を取るので、
     内部可変性を持つ `origins` 以外を書き換えられない。
    BY CODE src/rc_ir/ownership.rs: VarTable, CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only,
       CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: origin
  <2>2. `origin_inner(vars, E, u, sig)` の返り値は、`vars.bindings`、`E`、`u`、`sig`、および自分が
     行う `origin` の呼び出しの返り値だけで決まる。この関数が読むのは `vars.bindings.get(var)`、
     その `Binding` が持つ `RcVar` の名前と型と `llvm_gen` と `result_ty`、および `E` であり、
     `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` と `truncate_to_unit` はその引数の関数で
     ある。`vars.origins` も `vars.var_tys` も読まない。
    BY <2>1, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>3. `vars.origins` に記録される `(u, sig)` の値は、その位置で計算した
     `origin_inner(vars, E, u, sig)` の返り値である。`origin` が `insert` するのはその値だけで
     ある。
    BY CODE src/rc_ir/ownership.rs: origin
  <2>4. `ord(u)` (`<1>21`) についての強い帰納法で主張を示す。ASSUME: `ord(u') < ord(u)` を満たす
     すべての `(u', sig')` について `origin(vars, E, u', sig')` の返り値が `(u', sig')` で決まる
     PROVE: `origin(vars, E, u, sig)` の返り値は `(u, sig)` で決まる
    <3>1. `u` が `vars.bindings` の定義域に無いとき、`origin_inner` の `None` の腕は
       `Origin::Exactly((u, sig))` を返し、`origin` を呼ばない。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `u` が定義域にあるとき、`<1>25` より `origin_inner(vars, E, u, sig)` が呼ぶ
       `origin(vars, E, y, sig')` はどれも `ord(y) < ord(u)` を満たすので、帰納法の仮定よりその
       返り値は `(y, sig')` で決まる。`<2>2` よりそれらと `(u, sig)` から
       `origin_inner(vars, E, u, sig)` の返り値が決まる。
      BY <1>25, <2>2
    <3>3. QED
      `<3>1` と `<3>2` より `origin_inner(vars, E, u, sig)` の返り値は `(u, sig)` で決まる。
      `origin` が返すのは、memo が当たったときは `<2>3` より以前に記録された
      `origin_inner(vars, E, u, sig)` の返り値、当たらなかったときはその場で計算した同じものである。
      よってどちらも同じ値である。
      BY <2>3, <3>1, <3>2
  <2>5. QED
    `ord` の値は `-1` と `<1>21` の有限列の位置に限られるので、`<2>4` の帰納法は全体に届く。
    BY <1>21, <2>4

<1>30. `origin(vars, E, x, pi)` は abort しない。
  <2>1. `origin` 自身の abort の可能性は `vars.origins` の `RefCell` の借用の衝突だけである。
    <3>1. `origin` は `vars.origins.borrow()` を
       `if let Some(known) = ... { return known.clone(); }` の走査対象で 1 回、
       `vars.origins.borrow_mut()` を最後の文で 1 回使う。
      BY CODE src/rc_ir/ownership.rs: origin
    <3>2. `if let` の走査対象が作る一時値は、その `if let` 文の終わりで落ちる。`origin_inner` を
       呼ぶのはその次の文なので、その `Ref` は既に落ちている。

       依拠するのは第 1 節の外部の結果 (Rust の一時値のスコープ) である。その一覧のうち、
       edition 2021 で `if let` の走査対象を含む最小の場は「A statement」であり、ここではその
       `if let` 文自身である。edition 2024 は「The pattern-matching condition(s) and consequent
       body of `if` (2024 Edition)」をこの一覧に足すので、走査対象の一時値は `else` ブロックより
       前で落ちる。`Cargo.toml` の `[package]` は `edition = "2021"` を書いており、この `if let` は
       `else` を持たないので、どちらの規則でも落ちる点は同じ `if let` 文の終わりである。
      BY <3>1, 外部の結果 (Rust の一時値のスコープ)
    <3>3. `borrow_mut()` が作る一時値はその文の終わりで落ちる。その文の中で `origin` は呼ばれない
       (`answer.clone()` は既に得た `Origin` の複製である)。
      BY <3>1
    <3>4. QED
      `<3>2` と `<3>3` より、`Ref` が生きている間に `borrow_mut()` は起きず、`RefMut` が生きている
      間に `borrow()` も `borrow_mut()` も起きない。
      BY <3>2, <3>3
  <2>2. `origin_inner` の `None`、`Param`、`Producer` の腕は `Origin::Exactly` を作るだけで abort
     しない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `origin_inner` の `Move` の腕は `origin` を呼ぶだけで abort しない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. `origin_inner` の `Field` の腕は `container.ty.is_box(type_env)` を呼ぶ。`is_box` は
     `is_unbox`、すなわち `is_closure() || toplevel_tycon_info(type_env).is_unbox` を呼ぶ。
     `is_closure()` が真のときは短絡して `toplevel_tycon_info` を呼ばないので、その
     `assert!(!self.is_closure())` は発火しない。偽のときは `<1>1` (i) より
     `toplevel_tycon().unwrap()` と `tycons().get(&tycon).unwrap()` が成功する。その他にこの腕が
     呼ぶのは `origin` と `Vec` の連結だけである。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner, CODE src/ast/types.rs: TypeNode::is_box,
       CODE src/ast/types.rs: TypeNode::is_unbox,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info
  <2>5. `origin_inner` の `Payload` の腕は `scrut.ty.is_box(type_env)` を呼ぶ。`is_closure()` が真の
     ときは短絡して `toplevel_tycon_info` を呼ばないので、その `assert!(!self.is_closure())` は発火
     しない。偽のときは `<1>1` (i) より `toplevel_tycon().unwrap()` と
     `tycons().get(&tycon).unwrap()` が成功する。その他にこの腕が呼ぶのは `origin` と `Vec` の連結
     だけである。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner, CODE src/ast/types.rs: TypeNode::is_box,
       CODE src/ast/types.rs: TypeNode::is_unbox,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info
  <2>6. `origin_inner` の `Join(arm_results)` の腕は `Origin::of_candidates(candidates, ...)` を
     呼ぶ。`of_candidates` は `candidates` が空のとき `assert!` で abort するが、この `candidates` は
     空でない。
    <3>1. `arm_results` は `collect_bindings` が `Let(x, RcRhs::Match(scrut, arms), k)` の腕で
       `arms` の各要素について 1 つずつ積んだものなので、その長さは `arms` の長さに等しい。
      BY CODE src/rc_ir/ownership.rs: collect_bindings
    <3>2. `<1>3` より `arms` は空でない。`<3>1` より `arm_results` も空でない。
      BY <1>3, <3>1
    <3>3. QED
      この腕は `arm_results` の各要素について `origin(...).acted_on()` の全要素を `candidates` に
      入れる。`<3>2` より要素は 1 つ以上あり、`<1>27a` よりその `acted_on()` は空でないので、
      `candidates` は空でない。
      BY <1>27a, <3>2, CODE src/rc_ir/ownership.rs: origin_inner
  <2>7. `origin_inner` の `Llvm` の腕は abort しない。
    <3>1. この腕はまず `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を呼ぶ。`<1>28` より
       これは abort しない。そののち `decl.leaf_origins_at(path)` (`LeafMap::get` を経由し `Option`
       を返す) に `as_arg_projection` を `and_then` する。`as_arg_projection` は `Set` の大きさを見て
       場合分けするだけで abort しない。ここで `path` は `origin` に渡された `pi` そのものであり、
       `leaf_origins_at` はそれを `Map` の鍵として引くだけなので、`pi` が `decl` の leaf でなくても
       `None` が返るだけである。
      BY <1>28, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: as_arg_projection,
         CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
         CODE src/rc_ir/leaf_map.rs: LeafMap::get
    <3>2. `Some((j, p))` の腕は `args[j]` で添字付けする。`<1>28a` より `j` は `args.len()` 未満
       なので、添字付けは範囲内である。
      BY <1>28a, CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. `None` の腕は
       `origin_from_leaves_under(vars, type_env, &decl, args, path, &here_identity)` を呼び、その
       結果が `None` なら `here()` を返す。`unwrap_or_else` は abort しない。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>4. `origin_from_leaves_under` は `decl.leaf_origins_under(path)` が渡す `LeafOrigin::Arg(j,
       leaf)` の各出現について `args[*j]` で添字付けし、`truncate_to_unit(&args[*j].ty, leaf,
       type_env)` を呼ぶ。`<1>28a` より、この出現の宣言は単一の `Arg(j, leaf)` であり、`j` は
       `args.len()` 未満、`leaf` は `L(ty(args[j]))` の要素である。`origin` に渡された `pi` は
       `leaf_origins_under` の絞り込みに使われるだけで、`truncate_to_unit` には渡らない。
      BY <1>28a, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
    <3>5. `<1>17` (iii) と `<3>4` より `truncate_to_unit(&args[j].ty, leaf, E)` は abort しない。
       とくに `UnitStep::NoUnit` の腕の `panic!` (「holds no reference」) には達しない。
      BY <1>17, <3>4
    <3>6. `origin_from_leaves_under` の残りは、`Set` への挿入、`origin` の呼び出し、
       `reached.first()?` (空なら `None` を返す)、`Origin` の等価比較、そして `reached` が空でない
       ときの `Origin::of_candidates(candidates, here)` である。`candidates` は各 `reached` の
       `acted_on()` を集めたものであり、`reached` が空でなく、`<1>27a` より各 `Origin` の
       `acted_on()` が空でないので、`candidates` は空でなく `assert!` は発火しない。
      BY <1>27a, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates
    <3>7. QED
      BY <3>1, <3>2, <3>3, <3>4, <3>5, <3>6
  <2>8. `origin_inner` が呼ぶ型の上の walk (`truncate_to_unit` と、その中の `unit_step`、さらにその
     中の `is_fully_unboxed` と `unpunched_field_types`) は、`<1>1` (i) と `<1>9` より停止する。
    BY <1>1, <1>9
  <2>9. QED
    `origin` の実行が触れるのは `origin` 自身 (`<2>1`)、`origin_inner` の 8 通りの腕 (`<2>2` から
    `<2>7`)、および型の上の walk (`<2>8`) だけである。`Binding` の 7 つの値と `None` の 8 通りは
    `<2>2` から `<2>7` が尽くしている。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, <2>8, CODE src/rc_ir/ownership.rs: Binding

<1>31. **P2 が成り立つ。** すなわち `<1>1`、`<1>2`、`<1>3a` を満たすプログラムについて、`vars` を
   その関数の `VarTable`、`x` を任意の `FullName`、`pi` を任意の `FieldPath` とすると、
   `origin(vars, E, x, pi)` は panic せずに `Origin` の値を返し、停止する。P2 が量化するのは `x` が
   プログラムの束縛変数である場合であり、それはこの主張の特別な場合である。
  <2>1. `<1>29` (停止性) と `<1>30` (abort しないこと) の主張も証明も、`pi` に条件を置いていない。
     P2 の「`π` を問わず」が要求するのはこれである。
    BY <1>29, <1>30
  <2>2. `pi` が型の上の walk に渡らないことは `<1>30` が場合ごとに述べている。`origin` と
     `origin_inner` は `pi` を、`Origin::Exactly` と `Origin::of_candidates` の成分として複製するか、
     前に添字を継ぎ足して再帰へ渡すか、`decl.leaf_origins_at` の鍵および `decl.leaf_origins_under` の
     絞り込みに使うかのいずれかにしかしない。`truncate_to_unit` に渡るのは宣言が名指す leaf だけで
     ある。
    BY <1>30, CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>3. QED
    BY <1>29, <1>30, <2>1, <2>2

<1>32a. `t` が `<1>1` を満たし `u` が `U(t)` の要素であるとき、`u` は `t` の unit に届き
   `T(t, u) = u` である。
  <2>1. `<1>13` より `u` は次のどちらかの形である。`u = p` で `p` は `t` の ST-道、`cls(end(t, p))`
     は `BX`、`AR`、`UN` のどれか。または `u = p ++ [c]` で `p` は `t` の ST-道、
     `cls(end(t, p)) = CL`。`s_j` を `u[0..j]` の位置の型 (`j` は `|p|` 以下) と書く。
    BY <1>13
  <2>2. `j` が 0 以上 `|p|` 以下のすべてについて、`T(t, u)` のループの最初の `j` 周はすべて
     `UnitStep::Fields` の腕を通って完了し、その時点で `cur = s_j` かつ `out = u[0..j]` である。
    <3>1. `j = 0` のとき。0 周が完了した時点で `cur = t = s_0` かつ `out = [] = u[0..0]` である。
      BY CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>2. ASSUME: `j` は `|p|` 未満であり、最初の `j` 周がすべて `Fields` の腕を通って完了し、その
       時点で `cur = s_j` かつ `out = u[0..j]` である
       PROVE: 最初の `j+1` 周がすべて `Fields` の腕を通って完了し、その時点で `cur = s_{j+1}` かつ
       `out = u[0..j+1]` である
      <4>1. `j` は `|p|` 未満で `|p|` は `|u|` 以下なので、ループは第 `j` 周に入る。
        BY <2>1
      <4>2. `p` が `t` の ST-道なので `cls(s_j) = ST` であり、`<1>10` より `unit_step(s_j, E)` は
         `UnitStep::Fields { held_fields: F(s_j), .. }` である。
        BY <1>10, <2>1, DEF ST-道
      <4>3. 第 `j` 周の `idx` は `u[j] = p[j]` であり、`p` が `t` の ST-道であることから
         `(p[j], s_{j+1})` は `F(s_j)` の要素である。`s_j` は `t` からフィールドの辺を `j` 回辿って
         着く型なので `<1>1` (ii) より `<1>1` を満たし、`<1>12` より
         `held_field_type(&F(s_j), p[j], "truncate_to_unit")` は abort せず `s_{j+1}` を返す。
        BY <1>1, <1>12, <2>1, DEF ST-道, DEF fld
      <4>4. QED
         `Fields` の腕は `out.push(idx)` と `cur = held_field_type(...)` を行うので、`out` は
         `u[0..j+1]`、`cur` は `s_{j+1}` になる。
        BY <4>1, <4>2, <4>3, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>3. QED
      `<3>1` を基底、`<3>2` を帰納段とする `j` についての帰納法。
      BY <3>1, <3>2
  <2>3. CASE `u = p` で `cls(end(t, p))` が `BX`、`AR`、`UN` のどれか。`|u| = |p|` なのでループは第
     `|p|` 周に入らずに終わり、`<2>2` を `j = |p|` に適用すると `out = u[0..|p|] = u` である。abort
     する腕には達していない。`<2>1` より `u` は `U(t)` の要素である。
    BY <2>1, <2>2
  <2>4. CASE `u = p ++ [c]` で `cls(end(t, p)) = CL`。`|u| = |p| + 1` なのでループは第 `|p|` 周に
     入る。`<2>2` を `j = |p|` に適用すると `cur = s_{|p|} = end(t, p)`、`out = p`、
     `idx = u[|p|] = c` である。`<1>10` より `unit_step(cur, E)` は
     `UnitStep::Capture { capture_idx: c, .. }` なので `assert_eq!(idx, capture_idx, ...)` は通り、
     `out.push(c)` して `break` する。よって `T(t, u) = p ++ [c] = u` である。
    BY <1>10, <2>1, <2>2, CODE src/rc_ir/ownership.rs: truncate_to_unit
  <2>5. QED
    `<2>1` の 2 つの形を `<2>3` と `<2>4` が尽くしている。どちらも `T(t, u) = u` を与え、`u` は
    `U(t)` の要素である。
    BY <2>1, <2>3, <2>4, DEF unit に届く

<1>33. **(P1 の系 1: `origin` が辿る path はどれも unit に届く)** `<1>1`、`<1>2`、`<1>3a` を満たす
   プログラムについて、`pi` が `L(ty(x))` の要素または `U(ty(x))` の要素であるとき、`(x, pi)` の
   呼び出しの下流 (`DEF 呼び出しの下流`) にあるすべての対 `(u, sig)` について、`u` が
   `vars.var_tys` に型を持つならば `sig` は `ty(u)` の unit に届く (`DEF unit に届く`)。

   以下、`vars.var_tys` が記録する型と `args` の各要素の `ty` は RC IR に現れる型なので、`<1>1` に
   より `<1>1` を満たす。各ステップが `<1>1` を引くのはこの形である。
  <2>1. `t` が `<1>1` を満たし `lam` が `L(t)` の要素であるとき、`lam` は `t` の unit に届く。
     `<1>17` (iii) が `T(t, lam)` の abort しないことを、`<1>18` がその値が `U(t)` の要素であることを
     与える。
    BY <1>17, <1>18, DEF unit に届く
  <2>2. `t'` が `<1>1` を満たし、`q` が `t` の unit に届き、`cls(t') = ST` かつ `(i, t)` が `F(t')`
     の要素であるとき、`[i] ++ q` は `t'` の unit に届く。`<1>1` (ii) より `t` も `<1>1` を満たす。
    <3>1. `T(t', [i] ++ q)` のループの第 0 周は、`<1>10` より
       `unit_step(t') = UnitStep::Fields { held_fields: F(t'), .. }` を見て `out.push(i)` を行い、
       `<1>12` より `held_field_type(&F(t'), i, "truncate_to_unit")` が abort せず `t` を返すので
       `cur = t` になる。`<1>10` と `<1>12` を `t'` に当てられるのは、この場合の仮定が `t'` は
       `<1>1` を満たすと置いているからである。
      BY <1>10, <1>12, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>2. ループの各周の振る舞いは `cur` と `idx` だけで決まり、`out` には後ろに継ぎ足すことしか
       しない。`<3>1` の後の状態は `cur = t`、残りの入力が `q`、`out = [i]` であり、`T(t, q)` の
       初期状態は `cur = t`、残りの入力が `q`、`out = []` である。よって `T(t', [i] ++ q)` は abort
       せず、その値は `[i] ++ T(t, q)` である。
      BY <3>1, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>3. `t` は `t'` からフィールドの辺で着く型なので `<1>1` (ii) より `<1>1` を満たす。`T(t, q)`
       は `U(t)` の要素なので、`<1>13` を `t` に適用すると `T(t, q) = r` (`r` は `t` の ST-道で
       `cls(end(t, r))` が `BX`、`AR`、`UN` のどれか) か `T(t, q) = r ++ [c]` (`r` は `t` の ST-道で
       `cls(end(t, r)) = CL`) である。
      BY <1>1, <1>13, DEF unit に届く
    <3>4. `[i] ++ r` は `t'` の ST-道であり `end(t', [i] ++ r) = end(t, r)` である。`cls(t') = ST`
       であり `fld(t', i) = t` だからである。
      BY <3>3, DEF ST-道, DEF fld
    <3>5. QED
      `<3>3` と `<3>4` を `t'` についての `<1>13` に当てはめると `[i] ++ T(t, q)` は `U(t')` の
      要素である。`<3>2` と合わせて `[i] ++ q` は `t'` の unit に届く。
      BY <1>13, <3>2, <3>3, <3>4, DEF unit に届く
  <2>3. `t'` が `<1>1` を満たし、`cls(t')` が `BX`、`AR`、`UN` のどれかで `q'` が空でない path で
     あるとき、`q'` は `t'` の unit に届き `T(t', q') = []` である。
    <3>1. `<1>10` より `unit_step(t', E) = UnitStep::Unit` であり、その腕は `out` に積まずに `break`
       する。`q'` は空でないのでループは第 0 周に入る。よって `T(t', q') = []` であり abort しない。
      BY <1>10, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>2. `[]` は `t'` の長さ 0 の ST-道であり `cls(end(t', [])) = cls(t')` は `BX`、`AR`、`UN` の
       どれかなので、`<1>13` の第 1 の集合より `[]` は `U(t')` の要素である。
      BY <1>13, DEF ST-道
    <3>3. QED
      BY <3>1, <3>2, DEF unit に届く
  <2>4. `ty(u)` が `<1>1` を満たし `sig` が `ty(u)` の unit に届くならば、`cls(ty(u))` は `NB` で
     ない。
    <3>1. `cls(ty(u)) = NB` とする。`<1>10` より `unit_step(ty(u), E) = UnitStep::NoUnit` である。
      BY <1>10
    <3>2. `sig` が空でなければ、ループは第 0 周に入り `NoUnit` の腕の `panic!` に達する。これは
       `sig` が unit に届くことに反する。
      BY <3>1, DEF unit に届く, CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>3. `sig` が空なら `T(ty(u), sig) = []` である。`<1>13` より `U(ty(u))` の要素はどれも
       `ty(u)` の ST-道 `p` (末尾のクラスが `BX`、`AR`、`UN`) か `p ++ [c]` (末尾のクラスが `CL`) の
       形であるが、`cls(ty(u)) = NB` なので長さ 0 の ST-道の末尾のクラスは `NB` であり、`cls(ty(u))`
       は `ST` でないので長さ 1 以上の ST-道は無い。よって `U(ty(u))` は空集合であり、`[]` はその
       要素でない。これは `sig` が unit に届くことに反する。
      BY <1>13, <3>1, DEF ST-道, DEF unit に届く
    <3>4. QED
      BY <3>2, <3>3
  <2>5. 呼び出しの下流の起点 `(x, pi)` について主張が成り立つ。`ty(x)` は `<1>1` を満たすので、
     `pi` が `L(ty(x))` の要素なら `<2>1` が、`U(ty(x))` の要素なら `<1>32a` が、`pi` が `ty(x)` の
     unit に届くことを与える。
    BY <1>1, <1>32a, <2>1
  <2>6. ASSUME: 対 `(u, sig)` は `(x, pi)` の呼び出しの下流にあり、`u` が `vars.var_tys` に型を
     持つならば `sig` は `ty(u)` の unit に届く
     PROVE: `(u, sig)` から呼び出しの辺で着く各対 `(u', sig')` についても、`u'` が `vars.var_tys` に
     型を持つならば `sig'` は `ty(u')` の unit に届く
    <3>1. CASE `vars.bindings[u]` が `Binding::Move(y)` である。辺の先は `(y, sig)` であり、
       `<1>3a` (i) と (vi) より `ty(y) = ty(u)` なので、帰納法の仮定がそのまま主張である。
      BY <1>3a, CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. CASE `vars.bindings[u]` が `Binding::Join(arm_results)` である。辺の先は
       `(arm_result, sig)` であり、`<1>3a` (ii) と (vi) より `ty(arm_result) = ty(u)` なので、
       帰納法の仮定がそのまま主張である。
      BY <1>3a, CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. CASE `vars.bindings[u]` が `Binding::Payload(scrut, None)` である。辺の先は
       `(scrut, sig)` であり、`<1>3a` (iii) と (vi) より `ty(u) = ty(scrut)` なので、帰納法の仮定が
       そのまま主張である。
      BY <1>3a, CODE src/rc_ir/ownership.rs: origin_inner
    <3>4. CASE `vars.bindings[u]` が `Binding::Payload(scrut, Some(tag))` で `scrut.ty.is_box(E)` が
       偽である。
      <4>1. 辺の先は `(scrut, [tag] ++ sig)` である。
        BY CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. `<1>3a` (iii) と (vi) より `(tag, ty(u))` は `F(ty(scrut))` の要素であり、`<1>3a` (iv)
         より `ty(scrut)` は union の型である。
        BY <1>3a
      <4>3. `cls(ty(scrut))` は `NB` でない。`ty(scrut)` は union の型なので `is_closure()` は偽
         (union の型構成子の variant は `Union`、closure の型構成子の variant は `Arrow`)、
         `is_array()` は偽 (`Std::Array` の variant は `Array`)、`is_funptr()` は偽 (`#FunPtr{n}` の
         variant は `Primitive`) であり、この場合の仮定より `is_box(E)` も偽である。よって
         `is_fully_unboxed(ty(scrut))` の値は `F(ty(scrut))` の各要素の第 2 成分についての
         `is_fully_unboxed` の連言である。`<4>2` より `ty(u)` はその 1 つである。この場合の仮定は
         `vars.bindings[u]` が `Binding::Payload(scrut, Some(tag))` であることなので `u` は
         `vars.bindings` の定義域にあり、`<1>21a` より `vars.var_tys` にも型 `ty(u)` を持つ。
         したがって帰納法の仮定の前件が満たされ、その帰結と `<2>4` より `cls(ty(u))` は `NB` で
         ない、すなわち `is_fully_unboxed(ty(u))` は偽なので、連言は偽である。
        BY <1>21a, <2>4, <4>2, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
           CODE src/ast/types.rs: TyConVariant, CODE src/fixstd/builtin.rs: bulitin_tycons
      <4>4. `cls(ty(scrut)) = UN` である。`<4>3` より `NB` でなく、`<4>3` の中で `is_closure`、
         `is_box`、`is_array` が偽であることを示したので `CL`、`BX`、`AR` でもない。`ty(scrut)` は
         union の型なので `is_union(E)` が真であり、`DEF cls` より `UN` である。
        BY <4>2, <4>3, DEF cls
      <4>5. QED
        `ty(scrut)` は `<1>1` を満たし、`[tag] ++ sig` は空でないので、`<2>3` を
        `t' := ty(scrut)` に適用すると `[tag] ++ sig` は `ty(scrut)` の unit に届く。
        BY <1>1, <2>3, <4>1, <4>4
    <3>5. CASE `vars.bindings[u]` が `Binding::Field(cont, idx)` で `cont.ty.is_box(E)` が偽で
       ある。
      <4>1. 辺の先は `(cont, [idx] ++ sig)` である。
        BY CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. `<1>3a` (v) と (vi) より `(idx, ty(u))` は `F(ty(cont))` の要素であり、`ty(cont)` は
         構造体の型である。
        BY <1>3a
      <4>3. `cls(ty(cont))` は `NB` でない。`ty(cont)` は構造体の型なので `is_closure()` は偽
         (構造体の型構成子の variant は `Struct`)、`is_array()` は偽、`is_funptr()` は偽であり、この
         場合の仮定より `is_box(E)` も偽である。よって `is_fully_unboxed(ty(cont))` の値は
         `F(ty(cont))` の各要素の第 2 成分についての `is_fully_unboxed` の連言である。`<4>2` より
         `ty(u)` はその 1 つである。この場合の仮定は `vars.bindings[u]` が
         `Binding::Field(cont, idx)` であることなので `u` は `vars.bindings` の定義域にあり、
         `<1>21a` より `vars.var_tys` にも型 `ty(u)` を持つ。したがって帰納法の仮定の前件が
         満たされ、その帰結と `<2>4` より `is_fully_unboxed(ty(u))` は偽なので、連言は偽である。
        BY <1>21a, <2>4, <4>2, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
           CODE src/ast/types.rs: TyConVariant, CODE src/fixstd/builtin.rs: bulitin_tycons
      <4>4. `cls(ty(cont))` は `UN` か `ST` である。`<4>3` より `NB`、`CL`、`BX`、`AR` のどれでも
         ない。
        BY <4>3, DEF cls
      <4>5. CASE `cls(ty(cont)) = ST`。`ty(cont)` は `<1>1` を満たすので `<2>2` を
         `t' := ty(cont)`、`t := ty(u)`、`i := idx`、`q := sig` に適用できる。`<4>2` が
         `(idx, ty(u))` が `F(ty(cont))` の要素であることを、帰納法の仮定が `sig` が `ty(u)` の
         unit に届くことを与える。
        BY <1>1, <2>2, <4>1, <4>2
      <4>6. CASE `cls(ty(cont)) = UN`。`ty(cont)` は `<1>1` を満たし `[idx] ++ sig` は空でないので
         `<2>3` を `t' := ty(cont)` に適用する。
        BY <1>1, <2>3, <4>1
      <4>7. QED
        BY <4>4, <4>5, <4>6
    <3>6. CASE `vars.bindings[u]` が `Binding::Llvm(llvm_gen, args, result_ty)` で、
       `decl.leaf_origins_at(sig)` が単一の `Arg(j, p)` である。辺の先は `(args[j], p)` であり、
       `<1>28a` より `p` は第 `j` オペランドの leaf、すなわち `L(ty(args[j]))` の要素である。
       `ty(args[j])` は `<1>1` を満たすので `<2>1` を `t := ty(args[j])` に適用できる。
      BY <1>1, <1>28a, <1>3a, <2>1, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: as_arg_projection
    <3>7. CASE `vars.bindings[u]` が `Binding::Llvm(llvm_gen, args, result_ty)` で、
       `origin_from_leaves_under` が呼ばれる。辺の先は `(args[j], unit)` であり、
       `unit = truncate_to_unit(&args[j].ty, leaf, E)` で `leaf` は宣言が名指す leaf である。
       `ty(args[j])` は `<1>1` を満たすので、`<1>28a` より `leaf` が `L(ty(args[j]))` の要素で
       あることと合わせて `<1>18` を `t := ty(args[j])` に適用でき、`unit` は `U(ty(args[j]))` の
       要素である。同じ型に `<1>32a` を適用する。
      BY <1>1, <1>28a, <1>3a, <1>18, <1>32a,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>8. QED
      `origin_inner` が `origin` を呼ぶ場所は `<3>1` から `<3>7` の 7 つで尽きている。`None`、
      `Binding::Param`、`Binding::Producer` の腕、`Binding::Field` で容器が boxed の腕、
      `Binding::Payload` で `Some(_)` かつ scrutinee が boxed の腕は呼ばない。
      BY <3>1, <3>2, <3>3, <3>4, <3>5, <3>6, <3>7,
         CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding
  <2>7. QED
    `<2>5` を基底、`<2>6` を帰納段とする、`(x, pi)` からの呼び出しの辺の道の長さについての帰納法。
    `<1>25` より辺ごとに `ord` は狭義に減少し、`ord` の値は `-1` と `<1>21` の有限列の位置に限られる
    ので、道の長さは有限で上から抑えられる。よって帰納法は呼び出しの下流の全体に届く。
    BY <1>21, <1>25, <2>5, <2>6

<1>34. **(P1 の系 2: `origin` が返す path も unit に届く)** `<1>33` の条件の下で、
   `origin(vars, E, x, pi)` の返り値に現れる各 `VarPath` `(u, sig)` (identity と candidates の両方)
   について、`u` が `vars.var_tys` に型を持つならば `sig` は `ty(u)` の unit に届く。すなわち
   `T(ty(u), sig)` は abort せず、その値は `U(ty(u))` の要素である。
  <2>1. `origin(vars, E, u, sig)` が返す値に現れる各 `VarPath` は、`(u, sig)` の呼び出しの下流
     (`DEF 呼び出しの下流`) にある対である。
    <3>1. `origin_inner` の `None`、`Binding::Param`、`Binding::Producer` の腕、`Binding::Field` で
       容器が boxed の腕、`Binding::Payload` で `Some(_)` かつ scrutinee が boxed の腕は
       `here() = Origin::Exactly((var.clone(), path.to_vec()))` を返す。現れる `VarPath` は
       `(u, sig)` 自身である。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Move` の腕、`Binding::Payload` で `None` の腕、`Binding::Payload` で `Some(tag)`
       かつ scrutinee が unbox の腕、`Binding::Field` で容器が unbox の腕、`Binding::Llvm` で
       `leaf_origins_at` が単一の `Arg` の腕は、辺の先の `origin` の返り値をそのまま返す。帰納法の
       仮定より、現れる `VarPath` はその辺の先の下流にあり、したがって `(u, sig)` の下流にある。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. `Binding::Join` の腕は `Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))`
       を返す。`candidates` は各子の返り値の `acted_on()` の合併である。`acted_on()` が返すのは
       `identity()` と `candidates()` の元、すなわちその辺の先の返り値に現れる `VarPath` だけな
       ので、帰納法の仮定よりその各要素は辺の先の下流にある。`of_candidates` は要素数 1 のとき
       `Origin::Exactly(その要素)` を、それ以外のとき
       `Origin::Join { identity: (var, path), candidates }` を返す。前者に現れるのは辺の先の下流の
       対、後者に現れるのはそれと `(u, sig)` 自身である。どちらも `(u, sig)` の下流にある。
      BY CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates,
         CODE src/rc_ir/ownership.rs: Origin::acted_on,
         CODE src/rc_ir/ownership.rs: Origin::candidates
    <3>4. `Binding::Llvm` で `origin_from_leaves_under` を呼ぶ腕。`origin_from_leaves_under` は
       `reached` を、各辺の先の返り値と、`produced_here` が真のときの
       `Origin::Exactly(here.clone())` (`here` は `(u, sig)` 自身) から作る。`reached` が空なら
       `None` を返し、`origin_inner` は `here()` を返す (`(u, sig)` 自身)。`reached` の要素が
       すべて等しければその 1 つを返す。そうで
       なければ `Origin::of_candidates(candidates, here)` を返し、`candidates` は各 `reached` の
       `acted_on()` の合併である。`acted_on()` が返すのは `identity()` と `candidates()` の元、
       すなわちその `Origin` に現れる `VarPath` だけである。いずれの場合も現れる `VarPath` は、
       辺の先の下流にある対か `(u, sig)` 自身であり、どちらも `(u, sig)` の下流にある。
      BY CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates,
         CODE src/rc_ir/ownership.rs: Origin::acted_on,
         CODE src/rc_ir/ownership.rs: Origin::candidates
    <3>5. memo が当たった呼び出しが返す値も `<3>1` から `<3>4` の場合分けが覆う。`origin` は
       `vars.origins` にある `(u, sig)` の答えを複製して返すが、`<1>29a` よりその値は
       `origin_inner(vars, E, u, sig)` をその場で計算した値に等しく、`<3>1` から `<3>4` はその値に
       ついての言明だからである。memo をどの呼び出しが埋めたかは問わなくてよい。
      BY <1>29a, CODE src/rc_ir/ownership.rs: origin
    <3>6. QED
      `<3>1` から `<3>5` を帰納段とする、`ord(u)` (`<1>21`) についての強い帰納法。`<1>25` より
      呼び出しの辺の先の `ord` は狭義に小さく、`ord` の値は `-1` と `<1>21` の有限列の位置に限られる。
      `vars.bindings.get(u)` の結果は `Binding` の 7 つの値と `None` の 8 通りであり、`<3>1` が
      `None`、`Param`、`Producer`、`Field` で容器が boxed、`Payload` で `Some(_)` かつ scrutinee が
      boxed の 5 通りを、`<3>2` から `<3>4` が残る `Move`、`Join`、`Payload` の残り、`Field` の残り、
      `Llvm` の 2 つの道を尽くしている。
      BY <1>21, <1>25, <3>1, <3>2, <3>3, <3>4, <3>5,
         CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding
  <2>2. `<2>1` を `(u, sig) := (x, pi)` に読むと、`origin(vars, E, x, pi)` の返り値に現れる各
     `VarPath` は `(x, pi)` の呼び出しの下流にある。`<1>33` はその各対について、第 1 成分が
     `vars.var_tys` に型を持つならば第 2 成分がその型の unit に届くことを与える。
    BY <1>33, <2>1
  <2>3. 「`sig` が `ty(u)` の unit に届く」とは、`DEF unit に届く` より「`T(ty(u), sig)` は abort
     せず、その値は `U(ty(u))` の要素である」ことである。
    BY DEF unit に届く
  <2>4. QED
    BY <2>2, <2>3

<1>37. QED
  結論の 3 つは順に `<1>20` (`<1>1` を満たす型についての P1)、`<1>31` (P2)、`<1>33` と `<1>34`
  (P1 の系) である。
  BY <1>20, <1>31, <1>33, <1>34

## 3. 入力についての 3 つの前提と README の仮定、および P1 の定義域

`<1>1` (H1)、`<1>2` (H2)、`<1>3a` (H4) は、README の A10 (型の well-formedness)、A11 (スコープの
規律)、A12 (束縛の形と型が合っている) である。README の文面との差は、3 つの前提のそれぞれに
ついて次のとおりである。

- `<1>1` の (i) と (ii) は A10 である。(i) は A10 の第 1 文 --「プログラムに現れる型は ground で
  あり、その tycon に kind の要求するだけの引数が与えられており、その tycon は `type_env` にある」--
  そのものであり、(ii) は A10 の第 3 段落 --「`unpunched_field_types` を繰り返し取って到達する型に
  ついても同じことが成り立ち、その歩みは有限である」-- である。**「同じこと」は第 1 文の 3 つ、
  すなわち ground であることと飽和していることと tycon が `type_env` にあること、と読む。**
  A10 の第 2 段落は `no_size_in_place` の in-place 降下について 2 つ (ground と `type_env`) しか
  挙げていないが、`unpunched_field_types` の歩みは到達する各型で `declared_field_types` を呼び、
  その `assert_eq!(args.len(), tycon_info.tyvars.len())` が飽和を要求するので、飽和を落とすと
  A10 が「abort しないことを言う議論はこの節を読む」と書いた役目を果たせない。

  `<1>1` は A10 に (iii) を足す。(ii) が挙げる各型について、`instance_field_types` が行う newtype の
  展開 (`unwrap_newtypes_memoized`) が abort せず停止すること。`field_types` は tycon が kind が
  `*` でない型変数を持つときにこの展開を走らせ、その中の `tycon_info.fields[0]` の添字付けと
  `Type::AssocTy` の `unimplemented!` と展開の停止性は、A10 の 3 つからは出ない。

  A10 の但し書き -- 果たす者 `validate_layouts` は elaboration で必ず走るが、最適化が作る型を
  再検査するのは develop build だけである -- は `<1>1` の 3 つすべてに掛かる。

  この文書は A10 の第 1 文の `no_size_in_place` の節と第 2 段落を読まない。`REC` の辺も `DESC` の
  辺も `unpunched_field_types` の辺なので、整礎性は (ii) から直に出る (`<1>3d`、`<1>8`)。
- `<1>2` は A11 の「スコープに入っている束縛」を `DEF Scope` で節点の種類ごとに書き下し、さらに
  この関数のどの束縛でもない名前 (グローバル) を許す。`origin_inner` の `None` の腕がその名前を
  受ける (`CODE src/rc_ir/ownership.rs: origin_inner`)。書き下しそのものの根拠は D2 のスコープ規則
  であり、根の場合の 2 つ目 (グローバル初期化子) は D1 の「`init` はパラメータも capture も持た
  ない」から出る。A11 の検査 `validate` の `scope` の推移も同じ形で、関数ごとに `func.params` と
  `func.capture` を `bind` してから本体を検査し、グローバル初期化子については何も `bind` せずに
  本体を検査する。
- `<1>3a` は A12 の項目のうちこの文書が使う 7 つを (i) から (vii) として述べる。**7 つはどれも
  A12 に在る。**(vii) の 3 つは A12 の「`Llvm` 節点の型についての 3 つ」である。

  A12 の「`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が実際に持つ (punched
  でない) ものであること」は (iii) と (v) が述べており、この文書はそれを読む -- (iii) と (v) が
  `F`、すなわち `unpunched_field_types` の要素であることを言う形がそれであり、`<1>33` の `<2>6` の
  `<3>4` と `<3>5` がそこから穴の下へ降りないことを引く。A12 の残り -- `App` の引数と呼び出し先の
  パラメータの型が合っていること、`App` の結果の型が呼び出し先の返り値の型であること、束縛を持たない
  `RcVar` の型がその名前の記号の型であること -- は、この文書のどのステップも読まない。

  A12 の第 2 の `Llvm` 節点の条件は punched struct の成分を「A10 を満たす構造体」と書くが、(vii) は
  そこを `<1>1` と書く。差は `<1>1` の (iii) の 1 つであり、上に挙げたものと同じである。

  (vii) が要る理由は、これが無いと `result_prov` の 29 個の override のうち 5 個が abort しうるので
  P2 が偽になることである。`InlineLLVMStructGetBody` と `InlineLLVMUnionAsBody` は `arg_tys[0]`
  で添字付けし、`InlineLLVMStructPunchBody` は `result_ty.field_types(E)[PUNCHED_STRUCT_FIELD]` で
  添字付けし、`InlineLLVMStructSetBody` と `InlineLLVMStructPlugInBody` が呼ぶ
  `replaced_field_prov` は、boxed でない結果の boxed leaf の path が空でないことを `expect` で
  要求する (`Array a` を結果に持てば発火する)。

**P1 の定義域。** `<1>20` が示すのは、`<1>1` を満たす型についての P1 である。README の P1 は
「**A10 を満たす**任意の型 `τ` について」と書く。`<1>1` は A10 の (i)(ii) に (iii) を足したもの
なので、A10 を満たし (iii) も満たす型については `<1>20` が README の P1 そのものを与える。何らかの
制限が要ること自体は空虚ではない -- `<1>19a` が、`<1>1` の (i) を満たさない型については
`boxed_leaf_paths` も `rc_units` も `toplevel_tycon_info` の `unwrap` で abort し、P1 の言明の
2 つの辺が意味を持たないことを示す。

## 4. leaf と unit がずれる 2 か所が P1 に効いた場所

`unit_step` は `is_union` と `is_punched_array` で `UnitStep::Unit` を返して止まり、
`boxed_leaf_paths` の内部関数 `go` はその 2 つを問わずにフィールドへ降りる (`<1>10` と `<1>11` の表の
`UN` の行)。D5 が挙げる 2 か所は、この 1 つのクラス `UN` にまとまる。

- **unbox union**: `Choice = unbox union { l : Array I64, r : I64 }` の `rc_units` は `[[]]`、
  `boxed_leaf_paths` は `[[0]]` である。
- **punched array**: `PunchedArray a = unbox struct { _arr : Array a, _idx : I64 }` は
  `is_punched_array` が真なので `cls` は `UN` であり、`rc_units` は `[[]]`、`boxed_leaf_paths` は
  `[[0]]` である。

この差は P1 の 2 つの向きにそれぞれ別の形で効く。どちらの向きも `cls` が `UN` であることだけを使うので、
上の 2 か所を同時に扱う。

- **前半 (`<1>18`)**: leaf `lam` の道の途中に `UN` の型があると、`truncate_to_unit` はそこで
  `UnitStep::Unit` を見て `break` し、`lam` を最初の `UN` の位置で切る (`<1>17` の `k` の定義と
  (iii))。切った結果が `U(t)` の要素になるのは、`rc_units_go` がその同じ位置まで `ST` の型だけを降りて
  `UnitStep::Unit` を見て path を積むからである (`<1>13` の第 1 の集合)。すなわち、切り詰めが止まる
  位置と unit が置かれる位置が、どちらも「最初に `ST` でなくなる型」で一致している。
- **後半 (`<1>19`)**: `cls(s) = UN` の位置に置かれた unit `p` を証拠立てる leaf が要る。`go` は `UN`
  で降りるので、`p` の下の leaf は `p` より真に長い (`<1>19`)。真に長いことが効くのは、
  `truncate_to_unit` の `for` ループが位置 `|p|` の周に入ってはじめて `UnitStep::Unit` を見て
  `break` するからである (`<1>17` の (iii) が `cls(s_k)` に応じて場を分ける形である)。そして `p` の下に
  leaf が 1 つ以上あることは、`cls(s) = UN` なら `is_fully_unboxed(s)` が偽で `is_funptr(s)` も偽な
  ので `F(s)` のどれかのフィールドが `is_fully_unboxed` でない、という `<1>7` (b) と `<1>16` から
  出る。

つまり P1 は、`UN` の型が (a) `truncate_to_unit` で切り詰めが止まる位置であり、かつ (b) その下に必ず
leaf を持つ、という 2 つの事実の上に立っている。どちらか一方でも崩れると P1 は偽になる。

## 5. unit path の `origin` と、その下の leaf の `origin` の関係

`Retain`/`Release` の path が leaf でない unit (unbox union、punched array) のとき、その unit の
`origin` と、その下の各 leaf の `origin` について言えるのは次の 1 つである。

**`<1>33` と `<1>34`。** どちらの `origin` についても、その計算の中で起きる呼び出しの `(u, sig)` も、
返り値に現れる `VarPath` `(u, sig)` も、`sig` が `ty(u)` の unit に届く。すなわち
`truncate_to_unit(ty(u), sig)` は abort せず `rc_units(ty(u))` の要素になる。この性質は `origin` の
再帰の各辺 (move-bind、`Match` のアームの結果、変位アームの payload、catch-all の payload、
unbox 容器のフィールド、`Llvm` の 2 つの道) が保つ。

`origin` の答えに `truncate_to_unit` を当てるコードは `borrow.rs` の `owns_object` である。
`owns_unit` と `check_ownership_is_levelled` が `origin(v, unit).candidates()` の各 `(root, path)` を
`owns_object` に渡し、`owns_object` は `root` が `vars.param_tys` にあるとき `path` を `units_under`
と `truncate_to_unit` に掛ける。`param_tys` に入る名前は `var_tys` にも同じ型で入るので、`<1>34` が
その `path` について「`ty(root)` の unit に届く」を与える。`owns_object` を主語とする命題は P7e で
ある。
