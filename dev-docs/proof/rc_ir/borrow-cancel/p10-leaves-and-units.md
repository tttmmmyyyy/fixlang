# P1 (leaf と unit の対応) と P2 (`origin` の全域性と停止性) の証明

この文書が読んだコードのコミットは `95665b5b78d0499a1d216a887cac5f0d76a65b40` である。README が証明の
対象として名指すコミット `b6c51fb892746e493e155d9d59ea05d02d7357db` との間で、この文書が引くファイルに
変わったのは `// PROOF:` コメントと、`Validator::check_rhs` に足された検査 -- 各 `Llvm` 節点の
`result_prov` が 1 つの結果 leaf に 2 つ以上の source を宣言しないことを develop mode で確かめるもの --
だけである。README の第 1 節がこの 2 つを挙げる。

この文書が立つのは README の定義 D1、D2、D4、D5、D6 と仮定 A3、A6、A9、A10、A11、A12、A15、A28 の
上である。証明は 1 本の構造化証明で、その QED が次の 3 つである。

- **P1** (leaf と unit の対応)。README の P1 が量化する型、すなわち **A10 を満たす**任意の型に
  ついて成り立つ。`<1>1` は A10 をこの文書の記法で述べたものである。第 3 節がその対応を述べる。
- **P2** (`origin` の全域性と停止性)。README の P2 が量化する 2 つの形 -- プログラムの束縛変数と、
  `vars.bindings` に束縛を持たない名前 (D6 の第 3 の形) -- は、どちらも `<1>31` の範囲に入る。
- **P1 の系** (`<1>33`, `<1>34`)。`pi` が `ty(x)` の boxed leaf か RC unit であるとき、
  `origin(vars, E, x, pi)` の再帰が辿る対も、その返り値に現れる `VarPath` も、第 1 成分が
  `vars.var_tys` に型を持つ限り、第 2 成分が「その型の unit に届く」。すなわちそれに
  `truncate_to_unit` を当てると abort せず、値は `rc_units` の要素である。これは「unit path の
  `origin` と、その下の leaf の `origin` の関係」についての要望への答えである。

P1 と P2 は共通の命題 (型の上の walk が停止すること、`unit_step` と `boxed_leaf_paths` の内部関数
`go` の分類) を使うので、その命題を先頭の `<1>` ステップに置き、P1 と P2 をその後ろに置く。

`<1>29a` は README の P2a (`origin` の答えは memo に依らない) と同じ内容の局所命題であり、`<1>34` の
`<2>1` の `<3>5` が読む。その証明は命題を 1 つも引かない。仮定は A3 -- `result_prov` の決定性と、
共有参照で受け取る計算が値の等しさを変えないことの 2 節 -- のほか、`<1>25` と `<1>21` を経て A6 と
A11 を引き、定義 D1、D2、D6 を引く。外部の結果は `EXT Rust の可視性`、
`EXT Rust のモジュールの木`、`EXT Rust の内部可変性`、`EXT derive した PartialEq と Eq`、
`EXT HashSet の等価性`、`EXT 1 要素の集合の反復` である。

P1 は 2 つの静的な列挙 (`boxed_leaf_paths` と `rc_units`) の対応についての主張なので、D16 の
inhabited は現れない。実行時にどの leaf が参照を持つかは P1 の主張に入らない。

`<1>1` は README の A10 を、`<1>2` は A11 を A6 と D6 と合わせて、`<1>3a` は A12 と A3 の 1 段を、
この文書の記法で述べたものである。README の文面との差と、P1 の定義域については第 3 節に書く。

**A28 (組み込みの tycon の項目は組み込みが置いたもの) は `<1>3ba` が読む。**そこから `<1>3c` と
`<1>3ca` を経て、P1 (`<1>20`) と P2 (`<1>31`) の両方がこの仮定に立つ。A28 は型ではなく型環境 `E` に
掛かる条件なので、P1 の言明が量化する型の側の条件 (A10) には入らない。

## 1. 記法

型環境 `E` を 1 つ固定する。以下、型に関する関数の `type_env` 引数は `E` に固定し、書かない。

- `L(t)` := `boxed_leaf_paths(t, E)` (`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)
- `U(t)` := `rc_units(t, E)` (`CODE src/rc_ir/ownership.rs: rc_units`)
- `T(t, p)` := `truncate_to_unit(t, p, E)` (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)
- `c` := `CLOSURE_CAPTURE_IDX as usize` (`CODE src/constants.rs: CLOSURE_CAPTURE_IDX`)

型は `t`、`s`、`f` などのラテン文字で表す。path は `usize` の有限列 (`CODE src/rc_ir/ast.rs:
FieldPath`) であり、`p`、`q`、`u`、`lam` などで表す。`p[i]` は第 `i` 要素 (0 始まり)、`p[0..k]` は
先頭 `k` 要素からなる前置、`|p|` は長さ、`p ++ q` は連結、`[]` は空列とする。

**DEF F** -- 型 `t` について `F(t) := t.unpunched_field_types(E)` と書く
(`CODE src/ast/types.rs: TypeNode::unpunched_field_types`)。これは対 `(添字, 型)` の列である。

**EXT Rust の一時値のスコープ** -- Rust Reference の "Destructors" の節の "Temporary scopes" が
次を述べる。`<1>30` の `<2>1` の `<3>2` と `<3>3` がこれを引く。

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

**EXT Rust の可視性** -- Rust Reference の "Visibility and Privacy" が次を述べる。

> By default, everything is _private_, with two exceptions: Associated items in a `pub` Trait are
> public by default; Enum variants in a `pub` enum are also public by default.
>
> If an item is private, it may be accessed by the current module and its descendants.
>
> `pub(crate)` makes an item visible within the current crate.

構造体のフィールドもこの既定の下にある -- 同じ節の例が `pub struct Bar { field: i32 }` を
「a public struct with a private field」と注釈する。`<1>3b` と、`<1>29a` の `<2>1` の `<3>1` と
`<3>2` がこれを引く。

**EXT Rust のモジュールの木** -- Rust Reference の "Modules" が次を述べる。

> A module is a container for zero or more items.
>
> A _module item_ is a module, surrounded in braces, named, and prefixed with the keyword `mod`. A
> module item introduces a new, named module into the tree of modules making up a crate.

すなわち、クレートのモジュールの木の辺は `mod` の項目が作るので、あるモジュールの子は、その
モジュールの本体に置かれた `mod` の項目に限る。`<1>3b` と `<1>29a` の `<2>1` の `<3>1` がこれを引く。

**EXT Rust の内部可変性** -- Rust Reference の "Interior Mutability" が、共有参照の指す値を書き替える
ことについて次を述べる。

> This goes against the usual requirement that the value pointed to by a shared reference is not
> mutated.
>
> `std::cell::UnsafeCell<T>` type is the only allowed way to disable this requirement.

すなわち、共有参照 `&T` が指す記憶域へ書き込む道は `UnsafeCell` を通る欄だけである。`RefCell` と
`OnceLock` はその欄を持つ型である。`<1>1a` の `<2>2` と `<1>29a` の `<2>1` の `<3>3` が
これを引く。

**EXT HashSet の等価性** -- 標準ライブラリの `HashSet<T, S>` の `PartialEq` は、両者の
要素数が等しく、かつ一方のすべての要素が他方に含まれるときにだけ真を返す。すなわち `==` は集合と
しての等価性であり、反復の順序に依らない。`<1>29a` の `<2>1a` の `<3>3` と `<1>29a` の `<2>1b` が
これを引く。`crate::misc` の
`Set<T>` は `FxHashSet<T>`、すなわちハッシャだけを差し替えた `HashSet<T, S>` である
(`CODE src/misc.rs: Set`)。

**EXT RefCell の借用** -- 標準ライブラリの `RefCell<T>` について、`borrow` は返した `Ref` が drop
されるまで共有の借用を保ち、`borrow_mut` は返した `RefMut` が drop されるまで可変の借用を保つ。
`borrow` はその値が可変に借用されている間 panic し、`borrow_mut` はその値が共有または可変に借用されて
いる間 panic する。それ以外の場合はどちらも panic しない。`<1>30` の `<2>1` の `<3>4` がこれを引く。

**EXT derive した PartialEq と Eq** -- `#[derive(PartialEq)]` が enum に作る `eq` は、2 つの値の
変位が等しく、かつ対応する各成分が `==` で等しいときにだけ真を返す。`Eq` を実装する型の `==` は
同値関係であり、反射的・対称的・推移的である (`Eq` の doc がその契約を述べる)。`<1>29a` の `<2>1a` の
`<3>3` がこれを引き、その `<3>6` が推移律を使う。

**EXT 1 要素の集合の反復** -- 要素をちょうど 1 つ持つ `HashSet<T, S>` について、
`into_iter().next()` と `iter().next()` はどちらも `Some` を返し、その中身はその 1 つの要素で
ある (`into_iter` は要素そのもの、`iter` はそれへの共有参照)。`<1>29a` の `<2>1a` の `<3>6`、
`<1>29a` の `<2>1b`、
`<1>30` の `<2>6` と `<2>7` の `<3>1`・`<3>6`、`<1>34` の `<2>1` の `<3>3` がこれを引く。

**EXT Iterator の enumerate と filter** -- 標準ライブラリの `Iterator` について、`enumerate` は
もとの列の第 `i` 要素を対 `(i, 要素)` に写した列を返す。すなわち第 1 成分は 0 から始まる連続した
整数であり、相異なる。`filter(pred)` は `pred` が真を返す要素だけを、もとの順序のまま残した列を
返す。`<1>3c` の `<2>8` と `<1>12` の `<2>1` がこれを引く。

**EXT スライスの split_first** -- 標準ライブラリの `<[T]>::split_first` は、空でないスライスに
対して `Some((先頭の要素, 残り))` を返し、空のスライスに対して `None` を返す。`<1>28` の `<2>2f` の
`<3>3` と `<2>2g` の `<3>2` がこれを引く。

**EXT Iterator の map と zip** -- 標準ライブラリの `Iterator` について、`map(f)` はもとの列の各要素に
`f` を当てた列を返し、列の長さを変えない。`a.zip(b)` は `a` と `b` の同じ位置の要素の対を、短い方の
長さだけ並べた列を返す。したがって長さの等しい 2 つの列を `zip` すると、両者の各位置の対がちょうど
1 度ずつ渡される。`<1>9a` の `<2>1a` がこれを引く。

**EXT Vec の等価性** -- 標準ライブラリの `Vec<T>` の `PartialEq` は、両者の長さが等しく、かつ同じ
位置の要素どうしが `==` で等しいときにだけ真を返す。`<1>9a` の `<2>1a` がこれを引く。

**EXT Rust の評価の決定性** -- Rust の 1 つの関数呼び出しの実行は、次の 4 つが同じであれば、同じ
分岐を選び、同じ関数を値として等しい引数で同じ順に呼び、同じ値を返す。

- (i) 引数の値。
- (ii) その実行が記憶域から読む値。
- (iii) その実行が呼ぶ関数の返り値。
- (iv) その実行が比べる 2 つの番地が一致するかどうか (`Arc::ptr_eq` と、`impl PartialEq for Type` が
  節点の対について置く同じ形の比較がこれである)。

この道には並行性も乱数も外部入力も無い。`<1>9a` の `<2>7` の `<3>2`・`<3>3`・`<3>4` がこれを引く。

**DEF この道の関数** -- `truncate_to_unit(ty, path, E)` の実行が直接または間接に呼ぶ関数のうち、
`src/` に本体を持つものの全体を、**この道の関数**と呼ぶ。`<1>9a` の `<2>1` から `<2>5` が、その各々が
何を読むかを述べる。

**DEF 下位の呼び出しの列** -- ある関数呼び出しの 1 回の実行が行う「この道の関数」の呼び出しについて、
その**開始**の事象 (呼ぶ関数と引数の値を持つ) と**返り**の事象 (返る値を持つ) を、起きた時間順に
並べた列を、その実行の**下位の呼び出しの列**と呼ぶ。実行が停止しなければ無限列になりうる。
**2 つの実行の列が対応する**とは、一方の第 `i` 項が在れば他方の第 `i` 項も在り、その 2 つが同じ種の
事象であって、開始なら同じ関数を値として等しい引数で呼び、返りなら値として等しい値を返すことをいう。

**DEF 呼び出しの辺** -- 表 `vars` と型環境 `E` を固定する。対 `(u, sig)` から対 `(u', sig')` への
**呼び出しの辺**とは、`origin_inner(vars, E, u, sig)` の実行が `origin(vars, E, u', sig')` を呼ぶことを
いう。この関係が `vars.origins` の状態に依らないことは `<1>29b` が示す。

**DEF 呼び出しの下流** -- 対 `(x, pi)` から `DEF 呼び出しの辺` の辺を 0 回以上辿って着ける対の全体を、
`(x, pi)` の**呼び出しの下流**と呼ぶ。0 回の場合を含むので `(x, pi)` 自身もそこに入る。

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

### 在りかの前提

**コードのどこに何が在るかの数え上げは、段の中で行わない。** 記号を名指す `CODE` の引用はその記号の
本体しか与えないので、「ほかの記号はそれをしない」の側はそこから出ない。以下を名前つきの前提として
置き、`BY` の行ではその名前で引く。**個数は書かない** -- 一覧が在れば個数は一覧の長さである。

**果たすのは走査である。** 在りかを走らせられる字面で書き、`dev-docs/proof/proof_links.py` がその字面を
走らせて下の一覧と突き合わせる。挙がった各項目が何であるかは `--` の後に書く。走査は字面の上位近似
なので、一覧には宣言だけの項目も、署名や散文としてその字面を含む項目も入る。`#[cfg(test)]` の下の
項目は走査が除く。項目の名前は走査が呼ぶ名前である -- 自由関数がその直前の `impl` の名前を冠して
挙がる形を含む。

**前提 `TyConInfo` の値を作る在りか** --- `TyConInfo` の構造体リテラルを書く項目は次で尽きる。走査は
その型の宣言と、返り値の型としてその名前を書く署名も挙げる。

SCAN src/ `TyConInfo {`
  = src/ast/typedecl.rs: TypeDefn::tycon_info -- 構造体リテラルと、返り値の型としての署名
  = src/ast/types.rs: TyCon::TyConInfo -- 型の宣言と `impl TyConInfo` の見出し
  = src/ast/types.rs: TypeNode::toplevel_tycon_info -- 返り値の型としての署名
  = src/build/divide_program.rs: declaration_of -- 返り値の型としての署名
  = src/elaboration/desugar_opaque.rs: Program::register_opaque_tycon -- 構造体リテラル
  = src/fixstd/builtin.rs: bulitin_tycons -- 構造体リテラル
  = src/optimization/capture_struct.rs: CaptureStruct::new -- 構造体リテラル

**前提 `ty` の欄への代入の在りか** --- `ty` という名前の欄への代入を書く項目は次で尽きる。
`src/ast/types.rs` のうち `TypeNode` の `ty` の欄へ代入するのは 8 つの setter であり、どれも
`self.clone()` が作った局所の値へ代入してから `Arc::new` で包んで返す。同じファイルの `Scheme` の
3 つが替えるのは、`Scheme` が持つ `Arc<TypeNode>` の欄そのものである。ほかのファイルの項目が替える
のは、`Field`・`QualType`・`Predicate`・`Symbol` といった別の型が持つ `Arc<TypeNode>` の欄である。

SCAN src/ `.ty = `
  = src/ast/predicate.rs: Predicate::resolve_namespace -- `Predicate` の欄
  = src/ast/predicate.rs: Predicate::resolve_type_aliases -- `Predicate` の欄
  = src/ast/predicate.rs: Predicate::set_kinds -- `Predicate` の欄
  = src/ast/program.rs: TypeEnv::unwrap_newtypes -- `Field` の欄
  = src/ast/program.rs: TypeEnv::add_tycons -- `Field` の欄
  = src/ast/program.rs: Program::instantiate_symbol -- `Symbol` の欄
  = src/ast/qual_type.rs: QualType::resolve_namespace -- `QualType` の欄
  = src/ast/qual_type.rs: QualType::resolve_type_aliases -- `QualType` の欄
  = src/ast/traits.rs: TraitImpl::set_kinds_in_qual_pred_and_member_sigs -- メンバの署名の欄
  = src/ast/typedecl.rs: Field::resolve_namespace -- `Field` の欄
  = src/ast/typedecl.rs: Field::resolve_type_aliases -- `Field` の欄
  = src/ast/typedecl.rs: Field::set_kinds -- `Field` の欄
  = src/ast/types.rs: TypeNode::set_ty -- 局所の複製の `ty`
  = src/ast/types.rs: TypeNode::set_tyvar_kind -- 局所の複製の `ty`
  = src/ast/types.rs: TypeNode::set_tyvar -- 局所の複製の `ty`
  = src/ast/types.rs: TypeNode::set_tycon_tc -- 局所の複製の `ty`
  = src/ast/types.rs: TypeNode::set_tyapp_fun -- 局所の複製の `ty`
  = src/ast/types.rs: TypeNode::set_tyapp_arg -- 局所の複製の `ty`
  = src/ast/types.rs: TypeNode::set_assocty_name -- 局所の複製の `ty`
  = src/ast/types.rs: TypeNode::set_assocty_args -- 局所の複製の `ty`
  = src/ast/types.rs: Scheme::set_kinds -- `Scheme` の欄
  = src/ast/types.rs: Scheme::resolve_namespace -- `Scheme` の欄
  = src/ast/types.rs: Scheme::resolve_type_aliases -- `Scheme` の欄
  = src/elaboration/typecheck.rs: Substitution::substitute_predicate -- `Predicate` の欄
  = src/elaboration/typecheck.rs: Substitution::substitute_qualtype -- `QualType` の欄
  = src/elaboration/typecheck.rs: TypeCheckContext::reduce_predicate_noalias -- `Predicate` の欄
  = src/optimization/unwrap_newtype.rs: run_on_symbol -- `Symbol` の欄

**前提 型の節点への可変参照の在りか** --- `&mut TypeNode` の字面を含む項目も、`Arc::get_mut` の字面を
含む項目も `src/` に無い。`Arc::make_mut` を書くのは 1 つの項目だけであり、そこが可変に借りるのは
`Arc<Map<..>>` の欄 (`assumed_preds` と `assumed_eqs`) である。すなわち `Arc<TypeNode>` から
`TypeNode` の可変参照を取る道は `src/` に無い。

SCAN src/ `&mut TypeNode`

SCAN src/ `Arc::get_mut`

SCAN src/ `Arc::make_mut`
  = src/elaboration/typecheck.rs: TypeCheckContext::instantiate_scheme -- `Arc<Map<..>>` の欄を借りる

**前提 `truncate_to_unit` を呼ぶ在りか** --- `truncate_to_unit` を呼ぶ式が在る項目は次で尽きる。
走査はその宣言も挙げる。path を `origin` の答えから得るのは `owns_object` と `owns_object_yet` で
あり、残りが渡すのは `boxed_leaf_paths` が挙げる leaf か、`rhs_consumes` が報告する leaf か、
`result_prov` の宣言が名指す leaf である。

SCAN src/ `truncate_to_unit(`
  = src/rc_ir/borrow.rs: borrow_ify -- 借用版の `owned_units` を組む `boxed_leaf_paths` の leaf
  = src/rc_ir/borrow.rs: consume_rhs -- `rhs_consumes` が報告する leaf
  = src/rc_ir/borrow.rs: owns_object -- `origin` の答えの path を `units_under` が割った unit
  = src/rc_ir/borrow.rs: owns_object_yet -- 鍵の側は `origin` の答えの unit、突き合わせる側は `boxed_leaf_paths` の leaf
  = src/rc_ir/ownership.rs: origin_from_leaves_under -- `result_prov` の宣言が名指す leaf
  = src/rc_ir/ownership.rs: truncate_to_unit -- 宣言

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

   **(i) の等式は A10 が飽和の意味として直に述べている。**A10 の第 1 文は引数の個数を「その tycon に
   kind の要求するだけ」と書き、続く段落がそれを個数の等式に直す --「**飽和とは、
   `collect_type_arguments().len()` が `tycon_info.tyvars.len()` に等しいことである。**」。同じ段落が
   その根拠も添えて、「宣言の kind はその `tyvars` の個数だけ引数を要求する」ことと、「組み込みの各行も
   `kind` と `tyvars` の長さが揃う」ことを述べる。その等式の `tycon_info` は `t` の型構成子の宣言、
   すなわちこの段が上に置いた「そのとき `toplevel_tycon_info` が返すのは `E.tycons()[&tc]` である」の
   `E.tycons()[&tc]` である。よって A10 の飽和は `t.collect_type_arguments().len()` が
   `E.tycons()[&tc].tyvars.len()` に等しいこと、すなわち (i) の等式である。

   **3 つはどれも A10 である。** A10 の第 1 文のうち「プログラムに現れる型は ground であり、
   **その tycon に kind の要求するだけの引数が与えられており**、その tycon は `type_env` にあり、…」が
   (i) を、
   「`unpunched_field_types` を繰り返し取って到達する型についても、上の 3 つ -- ground、飽和、tycon が
   `type_env` にある -- がすべて成り立ち、その歩みは有限である」が (ii) を、それに続く「さらに、到達
   する各型について `instance_field_types` が行う newtype の展開 (`unwrap_newtypes_memoized`) は
   abort せず停止する」が (iii) を与える。第 3 節がこの対応を述べる。

   **A10 を果たす `validate_layouts` は elaboration で必ず走るが、最適化が作る型を再検査するのは
   develop build だけである。**これは A10 の但し書きであり、`<1>1` はそれを含めて A10 を引き継ぐ。
  BY <ref id=8412761/>, CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
     CODE src/ast/types.rs: TypeNode::collect_type_arguments,
     CODE src/ast/types.rs: TypeNode::unpunched_field_types,
     CODE src/ast/types.rs: TypeNode::instance_field_types,
     CODE src/rc_ir/ast.rs: RcVar (`ty` の doc「always concrete (monomorphic)」),
     CODE src/ast/program.rs: Program::validate_layouts

<1>1a. **(型の項は有限である)** `TypeNode` の値 `n` の**直接の部分**を、`n.ty` が
   `Type::TyApp(fun, arg)` のとき `fun` と `arg`、`Type::AssocTy(_, args)` のとき `args` の各要素、
   `Type::TyVar(_)` と `Type::TyCon(_)` のとき無しと定める。このとき、`n` から直接の部分の辺を
   0 回以上辿って着ける `TypeNode` の値は有限個である。したがって、辿る辺が直接の部分だけである
   再帰は停止する。この段は `<1>1` を読まない -- 型が RC IR に現れるかどうかに依らず、`TypeNode` の
   任意の値について成り立つ。
  <2>1. 1 つの値の直接の部分は有限個である。`Type` の変位は `TyVar`、`TyCon`、`TyApp`、`AssocTy` の
     4 つであり、はじめの 2 つは 0 個、`TyApp` は 2 個、`AssocTy` は `args` の長さだけを持つ。
    BY CODE src/ast/types.rs: Type
  <2>2. `TypeNode` の値の `ty` の欄が書かれるのは、その値が作られるときだけである。
     前提 `ty` の欄への代入の在りか が `src/` のすべての代入を挙げる。`src/ast/types.rs` の分は、
     `TypeNode` の 8 つの setter -- `set_ty`、
     `set_tyvar_kind`、`set_tyvar`、`set_tycon_tc`、`set_tyapp_fun`、`set_tyapp_arg`、
     `set_assocty_name`、`set_assocty_args` -- と、`Scheme` の `ty` の欄への 3 つ
     (`Scheme::set_kinds`、`Scheme::resolve_namespace`、`Scheme::resolve_type_aliases`) である。
     前者はどれも `self.clone()` が作った局所の値に代入してから `Arc::new` で包んで返し、後者が
     替えるのは `Scheme` が持つ `Arc<TypeNode>` そのものであって節点の中身ではない。`ty` の欄に値を
     置く残りは `TypeNode::new` と `impl Clone for TypeNode` の構造体リテラル、および `TypeNode` が
     導出する `Deserialize` であり、どれも新しい値を作る。同じ前提が挙げるほかのファイルの代入は、
     `Field`、`QualType`、`Predicate`、`Symbol` といった別の型が持つ `Arc<TypeNode>` の欄を替える
     ものである。
     代入以外の道で既に在る値の `ty` を書くには、その `TypeNode` を所有するか、`&mut TypeNode` を
     持つかのどちらかが要る。`EXT Rust の内部可変性` より、共有参照 `&TypeNode` が指す記憶域への
     書き込みは `UnsafeCell` を通る欄に限られ、`ty` は `Type` 型の素の欄だからである。
     `TypeNode` が渡されるのは `Arc<TypeNode>` を通じてであり、`Arc` が渡すのは `&TypeNode` で
     ある。前提 型の節点への可変参照の在りか より、`Arc<TypeNode>` から `TypeNode` の可変参照を
     取る道は `src/` に無い。
    BY EXT Rust の内部可変性, 前提 `ty` の欄への代入の在りか,
       前提 型の節点への可変参照の在りか,
       CODE src/ast/types.rs: TypeNode (`ty` の宣言), CODE src/ast/types.rs: TypeNode::new,
       CODE src/ast/types.rs: TypeNode::set_ty, CODE src/ast/types.rs: TypeNode::set_tyvar_kind,
       CODE src/ast/types.rs: TypeNode::set_tyvar, CODE src/ast/types.rs: TypeNode::set_tycon_tc,
       CODE src/ast/types.rs: TypeNode::set_tyapp_fun, CODE src/ast/types.rs: TypeNode::set_tyapp_arg,
       CODE src/ast/types.rs: TypeNode::set_assocty_name,
       CODE src/ast/types.rs: TypeNode::set_assocty_args,
       CODE src/ast/types.rs: impl Clone for TypeNode
  <2>3. 値が作られる時点で、その直接の部分はどれも既に在る値である。`Type::TyApp` と
     `Type::AssocTy` を組み立てるのは `type_tyapp`、`type_assocty`、および `<2>2` の
     `set_tyapp_fun`・`set_tyapp_arg`・`set_assocty_name`・`set_assocty_args` であり、どれも
     引数として渡された `Arc<TypeNode>` か、複製元の `ty` が既に持っていた `Arc<TypeNode>` を置く。
     `TypeNode` が導出する `Deserialize` も、`ty` の欄を読み終えてから節点を組み立てるので、
     部分は節点より先に作られる。
    BY <2>2, CODE src/ast/types.rs: type_tyapp, CODE src/ast/types.rs: type_assocty,
       CODE src/ast/types.rs: TypeNode (`Serialize` と `Deserialize` の導出)
  <2>4. QED
    1 回の実行で作られる `TypeNode` の値に、作られた順の番号を与える。`<2>2` と `<2>3` より、直接の
    部分の辺はこの番号を狭義に減らすので、`n` から降りる辺の道はどれも有限であり、その長さは `n` の
    番号で上から抑えられる。`<2>1` より各段の分岐は有限である。有限分岐で深さが有限の木は有限なので、
    `n` から着ける値は有限個であり、辺が直接の部分だけである再帰はそのすべてを訪れて終わる。
    BY <2>1, <2>2, <2>3

<1>2. **(H2: 変数のスコープ規律)** 本体 -- 関数の `body` またはグローバル初期化子の `init` -- を 1 つ
   取り、`vars` をそれについて `VarTable::of` または `VarTable::body_only` が作る表とする。その本体の
   各節点 `n` が使う変数はどれも、`Scope(n)` (下の `DEF Scope`) の要素であるか、`vars.bindings` の
   定義域に無い名前である。ここで `Let(x, rhs, k)` の `rhs` が使う変数は、`Scope` に `x` が入る**前**の
   集合、すなわち `Scope(Let(x, rhs, k) の節点)` で解決される。

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

   **示すのは、`vars.bindings` の定義域にある名前が `Scope(n)` の要素であることである。**`n` が使う
   変数 `v` がその定義域にあるとする。D6 より `v` は局所名である。A11 の「関数の本体の自由な局所名は、
   その関数のパラメータと capture に限る。グローバル初期化子の `init` は自由な局所名を持たない」より、
   `v` は、この本体のパラメータ・capture であるか、この本体のある節点が束縛するものである。前者なら
   `DEF Scope` の根の行が `v` を `Scope(根)` に入れ、`DEF Scope` の
   残る 3 行はどれも子の集合を親の集合に名前を 0 個以上加えたものと定めるので、`v` は `Scope(n)` の
   要素である。後者なら、A6 より `v` の束縛はプログラム全体で 1 つなので、A11 の「変数の使用は、その
   位置でスコープに入っている束縛に解決する」が言う解決先はその束縛であり、その束縛のスコープが `n` を
   含む、すなわち `v` は `Scope(n)` の要素である。

   **スコープを定めるのは D2 であり、`DEF Scope` はその規則を節点の種類
   ごとに書き下したものである。**D2 は `Let` が束縛する `x` のスコープを `k` の部分木、`Destructure`
   が束縛する各変数のスコープを `k` の部分木、`Match` のアームの `payload` のスコープをそのアームの
   `body` の部分木、パラメータと capture のスコープを本体の全体と定める。上の 4 行はこれを、根から
   節点へ下る漸化式の形に直したものである。根の場合が 2 つに分かれるのは、`VarTable::of` が関数の
   本体を取り `VarTable::body_only` がグローバル初期化子の `init` を取るからで、`init` はパラメータも
   capture も持たない (D1) ので `Scope(根)` は空集合になる。A11 の検査 `validate` の `scope` の推移も
   この形であり、関数ごとに `func.params` と `func.capture` を `bind` してから `check_expr` を呼び、
   グローバル初期化子については `bind` を 1 度も呼ばずに `check_expr` を呼ぶ。**`validate` は
   `develop_mode` のときだけ走る** -- A11 の但し書きがこれを言う。
  BY <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=a502f3e/>, <ref id=b3dfa37/>, <ref id=596a46d/>, CODE src/rc_ir/validate.rs: validate,
     CODE src/rc_ir/validate.rs: Validator::check_expr_inner,
     CODE src/rc_ir/validate.rs: Validator::check_rhs,
     CODE src/rc_ir/validate.rs: Validator::use_var,
     CODE src/rc_ir/validate.rs: Validator::bind

<1>3. A9 を満たすプログラムのどの本体についても、`RcRhs::Match(scrut, arms)` の `arms` は
   空でない。A9 の第 1 文が `borrow_ify` の入力プログラムについてこれを述べ、第 2 文が範囲を
   `cancel` の入力と出力へ伸ばす。**プログラムを名指すのが要るのは、A9 が全プログラムについての
   言明ではないからである。**
  BY <ref id=1172c08/>, CODE src/rc_ir/validate.rs: Validator::check_rhs (`RcRhs::Match` の腕の `arms.is_empty()`
     検査)

<1>3a. **(H4: 束縛の形と型が合っている)** 関数の `body` とグローバル初期化子の `init` のどちらに
   ついても、次の 8 つが成り立つ。
   - (i) `Let(x, RcRhs::Var(y), k)` について `ty(y)` は `ty(x)` に等しい。
   - (ii) `Let(x, RcRhs::Match(scrut, arms), k)` の各アームについて、`returned_var(&arm.body)` の型は
     `ty(x)` に等しい。
   - (iii) 同じ `Match` の各アームについて、`arm.tag` が `Some(k)` のとき `(k, ty(arm.payload))` は
     `F(ty(scrut))` の要素であり、`arm.tag` が `None` のとき `ty(arm.payload)` は `ty(scrut)` に
     等しい。
   - (iv) 同じ `Match` の `ty(scrut)` は union の型である。すなわち
     `ty(scrut).toplevel_tycon_info(E).variant` は `TyConVariant::Union` であり、
     `ty(scrut).is_closure()` は偽である。
   - (v) `Destructure(cont, fields, s, k)` について `ty(cont)` は構造体 (タプルを含む) の型である。
     すなわち `ty(cont).toplevel_tycon_info(E).variant` は `TyConVariant::Struct` であり、
     `ty(cont).is_closure()` は偽である。また
     各 `(i, fv)` について `(i, ty(fv))` は `F(ty(cont))` の要素である。
   - (vi) 同じ名前を持つ `RcVar` の出現はどれも同じ型を持つ。したがって `vars.var_tys` が記録する型は、
     その名前を使う側の `RcVar` の `ty` に等しい。以下ではこの型を `ty(名前)` と書く。
   - (vii) `Let(x, RcRhs::Llvm(llvm_gen, args), k)` について、`llvm_gen` は `args` の型の列と
     `ty(x)` の上で定義されている。この文書が読むのはそのうち次の 3 つである。
     - `args` の名前の列は `llvm_gen.free_vars()` に等しい。
     - `llvm_gen` が `InlineLLVMStructPunchBody` であるとき、`ty(x).is_box(E)`、`ty(x).is_array()`、
       `ty(x).is_closure()` はいずれも偽であり、`ty(x).field_types(E)` は長さ 2 の列を返す。その第
       `PUNCHED_STRUCT_FIELD` 成分の型を `pt` と書くと、`pt` は `<1>1` を満たす構造体であり、
       `pt.is_closure()` は偽であって、`pt.toplevel_tycon_info(E).fields[llvm_gen.field_idx]` の
       `is_punched` は真である。
     - `llvm_gen` が `InlineLLVMStructSetBody` か `InlineLLVMStructPlugInBody` であるとき、
       `ty(x).is_array()` は偽である。
   - (viii) `Let(x, RcRhs::Llvm(llvm_gen, args), k)` について、`llvm_gen` の `result_prov` が結果の
     ある leaf の宣言として単一の `LeafOrigin::Arg(j, sigma)` を置くとき、`j` は `args` の添字で
     あり (すなわち `args.len()` 未満であり)、`sigma` は `L(ty(args[j]))` の要素である。

   **(i) から (vii) は A12 であり、(viii) は A3 である。**(iii) と (v) が `F`、すなわち
   `unpunched_field_types` の要素で
   あることを言うのは、A12 の「`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が
   実際に持つ (punched でない) ものであること」を述べたものである。(vii) の 3 つは A12 の
   「`Llvm` 節点の型についての 4 つ」のうち、この文書が読む 3 つである。A12 の第 2 項は punched
   struct の成分を「A10 を満たす構造体」と書く。`<1>1` は A10 をこの文書の記法で述べたものなので
   (第 3 節)、(vii) はそこを
   `<1>1` と書く。**同じ項が「第 `field_idx` フィールドが穴である」と言うのは、`pt` の `TyConInfo`
   の `fields` の第 `field_idx` 成分の `is_punched` についてである。**`ty(x).is_closure()` が偽で
   あることは、A12 のその項が「**`is_closure()` も偽であり**」と `is_box`・`is_array` に並べて述べ、
   根拠を括弧に添える (「`struct_punch` が結果の型を `make_tuple_ty` で作り、tuple は構造体である」)。
   `pt.is_closure()` が偽であることは、A12 のその項が「`is_closure()` は偽である」と併せて述べる。

   **(iv) と (v) は、A12 の「`Match` の scrutinee が union であること」と「`Destructure` の容器が
   構造体であること」を、その型の `TyConInfo` の `variant` として書いたものである。**タプルもこの形に
   入る -- `tuple_defn` はタプルを `TypeDeclValue::Struct` の `TypeDefn` として宣言し、
   `TypeDefn::tycon_info` はその腕で `TyConVariant::Struct` を置く。**`is_closure()` が偽であることを
   一緒に書くのは、A12 の「この仮定が型の `variant` を述べる各節では、その型の `is_closure()` は偽で
   ある。」がその各節の一部だからである。**

   (viii) は A3 の「**単一の `Arg(j, σ)` の宣言は well-formed である。** `j` は `args` の添字で
   あり、`σ` はその型の boxed leaf である」の段落である。「その型」は第 `j` オペランドの型
   `ty(args[j])`、
   「boxed leaf」は D4 の意味、すなわち `boxed_leaf_paths(ty(args[j]))` が列挙する path であり、
   この文書の記法では `L(ty(args[j]))` の要素である。宣言の well-formedness は、`LLVMGen` の
   宣言についての仮定 A3 が述べる。
  BY <ref id=e11772a/>, <ref id=83d98e9/>, CODE src/rc_ir/ast.rs: RcRhs (`Var` の doc「Move / rename `y := x`, consuming `x`」),
     CODE src/rc_ir/ast.rs: MatchArm (`tag` と `payload` の doc),
     CODE src/rc_ir/ast.rs: RcExpr (`Destructure` の doc「Destructure a struct/tuple container into
       its fields at once ... Each `(index, var)` binds field `index` to `var`」),
     CODE src/rc_ir/ast.rs: RcVar,
     CODE src/ast/types.rs: TypeNode::unpunched_field_types,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
     CODE src/rc_ir/ownership.rs: returned_var,
     CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/ast/types.rs: TyConVariant,
     CODE src/ast/typedecl.rs: TypeDefn::tycon_info,
     CODE src/fixstd/builtin.rs: tuple_defn

<1>3b. 製品のコードが `TyConInfo` の値を作る場所は、次の 4 つの関数だけである。
   前提 `TyConInfo` の値を作る在りか が `src/` のすべての構造体リテラルを挙げ、残る 3 項目は
   型の宣言と 2 つの署名である。各関数が置く `variant` と、この証明が読むフィールドは
   次のとおりである。

   | 作る関数 | `variant` | 個数と、この証明が読むフィールド |
   |---|---|---|
   | `bulitin_tycons` | `Primitive` | 112 個。名前つきの 12 個 (`IOState`、`Ptr`、`U8`、`I8`、`U16`、`I16`、`I32`、`U32`、`I64`、`U64`、`F32`、`F64`) と、`1..=FUNPTR_ARGS_MAX` の各 arity についての `#FunPtr{n}` -- `FUNPTR_ARGS_MAX` は 100 である。名前つきの 12 個は `fields: vec![]` と `tyvars: vec![]` を持ち、`#FunPtr{n}` は `fields: vec![]` と `tyvars` が `a0` から `a{n-1}` までの相異なる名前の列である |
   | `bulitin_tycons` | `Array` | 1 個 (`Std::Array`)。`tyvars: vec![make_tyvar("a", ...)]` |
   | `bulitin_tycons` | `Arrow` | 1 個。`tyvars` は `a` と `b` |
   | `bulitin_tycons` | `DynamicObject` | 1 個 (`#DynamicObject`)。`is_unbox: false`、`tyvars: vec![]` |
   | `bulitin_tycons` | `ArrayStorage` | 1 個 (`#ArrayStorage`)。`is_unbox: false`、`tyvars: vec![make_tyvar("a", ...)]` |
   | `TypeDefn::tycon_info` | `Struct` か `Union` | union の宣言 1 つにつき 1 個、`n` フィールドの構造体の宣言 1 つにつき `n + 1` 個 (穴の無い形と、フィールドごとの穴つきの形)。`tyvars` はどの形でも `self.tyvars` |
   | `CaptureStruct::new` | `Struct` | capture 構造体ごとに 1 個。`tyvars: vec![]` |
   | `register_opaque_tycon` | `Opaque` | 不透明型ごとに 1 個。`is_unbox: false` |

   すなわち `DynamicObject`、`ArrayStorage`、`Opaque`、`Primitive` の 4 つの `variant` は、それぞれ
   ちょうど 1 か所からしか出ない。

   **`Opaque` を除く 7 行の `tyvars` の名前は相異なる。**`bulitin_tycons` の 5 行と
   `CaptureStruct::new` は上の表が名前を挙げている。`TypeDefn::tycon_info` の行は `self.tyvars` を
   そのまま置き、`TypeDefn::validate_tyvars` がその名前が重複しないことを検査する。
   `TypeDefn::tycon_info` が作る `TyConInfo` が `E` に入るのは `Program::calculate_type_env` を通って
   であり、それは `self.type_defns` の各要素について `type_decl.tycon_info(&[])` と、構造体なら
   フィールドごとの穴つきの形 `type_decl.tycon_info(&[i])` を入れる。**`E` に入る `Struct` の
   `TyConInfo` がこの関数から出るとは限らない** -- `CaptureStruct::new` が作る行は
   `closure_specialization` の `lift_all` と `realize_all`、および `defunctionalize_fix::run_one` が
   `add_tycons` で入れる。その行は `tyvars: vec![]` なので、上の表が別に片付けている。`Program::validate_type_defns` は同じ
   `self.type_defns` を渡って `validate_tyvars` を呼び、`elaborate` はそれを `?` で呼ぶので、
   elaboration を通ったプログラムではどの行の名前も相異なる。

   **`E` に登録されている各 `TyConInfo` の `variant`、`tyvars`、`is_unbox`、`fields` の長さ、
   および各 `fields[i].is_punched` は、上の 4 つのいずれかがそれを作ったときの値である。**この 5 つが、
   この証明が `TyConInfo` から値として読むものである。**`fields[i].ty` は `F(t)` の第 2 成分として
   現れるが、この証明はその値を主張せず、それが `<1>1` を満たすことだけを使う。`TypeEnv` の `tycons`
   は非公開のフィールドなので、`EXT Rust の可視性` よりそれを名前で参照できるのは、それを宣言する
   モジュール `crate::ast::program` とその子孫だけである。`EXT Rust のモジュールの木` より子孫は
   `mod` の項目が作るところに限られ、`src/ast/program.rs` は `mod` の項目を持たないので、そのモジュール
   はこの 1 ファイルで閉じている。同ファイルで `tycons` に書くのは次のとおりである。

   - `TypeEnv::default`。空の `Map` を置く。
   - `TypeEnv::new`。`Program::calculate_type_env` が、`bulitin_tycons()` に各型宣言の
     `type_decl.tycon_info(&[])` と、構造体についてはフィールドごとの穴つきの形
     `type_decl.tycon_info(&[i])` を足した `Map` を渡す。
   - `TypeEnv::add_tycons`。渡された各 `TyConInfo` の `fields` を走って `field.ty` を
     `unwrap_newtypes` の像に置き替えてから、`tycons.insert(tycon, tycon_info)` で丸ごと入れる。
     鍵が新しければ項目が増え、同名の項目が既に在ればそれが置き替わる。
   - `TypeEnv::unwrap_newtypes`。既に在る各 `TyConInfo` の `fields` を走って `field.ty` を
     同じように置き替える。
   - `TypeEnv::resolve_type_aliases_in_tycons`。既に在る各 `TyConInfo` に
     `TyConInfo::resolve_type_aliases` を当てる。それは各 `Field` に `Field::resolve_type_aliases` を
     当て、その本体は `self.ty` への 1 つの代入である。
   - `Program::resolve_namespace_not_in_expr`。既に在る各 `TyConInfo` に
     `TyConInfo::resolve_namespace` を当てる。それは各 `Field` に `Field::resolve_namespace` を
     当て、その本体は `self.syn_ty` と `self.ty` への 2 つの代入である。

   **`tycons` に `TyConInfo` の値を置くのは、この一覧のうち `TypeEnv::new` と
   `TypeEnv::add_tycons` である。**`TypeEnv::default` が置く `Map` は空なので `TyConInfo` を
   1 つも含まず、残る 3 つは既に在る `TyConInfo` の欄を書き替えるだけである (次の段落)。
   `TypeEnv::new` が置くのは
   `Program::calculate_type_env` が渡す `Map`、すなわち `bulitin_tycons()` の各行と、各型宣言に
   ついての `TypeDefn::tycon_info` の返り値である。`TypeEnv::add_tycons` が置くのは、A28 の走査
   `SCAN src/ .add_tycons(` が挙げる 4 か所が渡す `TyConInfo`、すなわち `CaptureStruct::new` が
   作った `tycon_info`
   (`closure_specialization` の `lift_all` と `realize_all` が `record_capture_list` と
   `take_new_tycons` を経て渡すもの、および `defunctionalize_fix::run_one` が渡すもの) と、
   `register_opaque_tycon` がその場で作る `TyConInfo` である。**すなわちこの 2 つが置く値はどれも、
   上の表の 4 つの関数のいずれかが作ったものである。**`add_tycons` は入れる前に各 `Field` の `ty` を
   `unwrap_newtypes` の像に置き替えるが、`fields` の長さを変えず、`variant`、`tyvars`、`is_unbox`、
   `fields[i].is_punched` にも触れない。

   **残る 3 つ -- `TypeEnv::unwrap_newtypes`、`TypeEnv::resolve_type_aliases_in_tycons`、
   `Program::resolve_namespace_not_in_expr` -- が書き替えるのは、既に在る `TyConInfo` の
   `fields[..].ty` と `fields[..].syn_ty` だけである。**どれも `fields` の長さを変えず、`variant`、
   `tyvars`、`is_unbox`、`fields[i].is_punched` にも触れない。`TyConInfo` は `Serialize` も
   `Deserialize` も導出しないので、キャッシュから読まれる `TyConInfo` も無い。前提 型の節点への
   可変参照の在りか より `Arc::get_mut` は `src/` に無く、`Arc::make_mut` を書く 1 項目が借りるのは
   `Arc<Map<..>>` の欄なので、`tycons` が包む `Map` の項目をその場で書き替える道も無い。
  BY EXT Rust の可視性, EXT Rust のモジュールの木, <ref id=3d4be43/>,
     前提 `TyConInfo` の値を作る在りか, 前提 型の節点への可変参照の在りか,
     CODE src/ast/types.rs: TyConInfo, CODE src/fixstd/builtin.rs: bulitin_tycons,
     CODE src/constants.rs: FUNPTR_ARGS_MAX, CODE src/ast/typedecl.rs: TypeDefn::tycon_info,
     CODE src/ast/typedecl.rs: TypeDefn::validate_tyvars,
     CODE src/ast/typedecl.rs: Field (`ty` / `syn_ty` / `is_punched` の宣言),
     CODE src/ast/typedecl.rs: Field::resolve_namespace,
     CODE src/ast/typedecl.rs: Field::resolve_type_aliases,
     CODE src/ast/program.rs: Program::validate_type_defns,
     CODE src/ast/program.rs: Program::calculate_type_env,
     CODE src/ast/program.rs: TypeEnv (`tycons` の宣言),
     CODE src/ast/program.rs: TypeEnv::default, CODE src/ast/program.rs: TypeEnv::new,
     CODE src/ast/program.rs: TypeEnv::add_tycons,
     CODE src/ast/program.rs: TypeEnv::unwrap_newtypes,
     CODE src/ast/program.rs: TypeEnv::resolve_type_aliases_in_tycons,
     CODE src/ast/program.rs: Program::resolve_namespace_not_in_expr,
     CODE src/ast/types.rs: TyConInfo::resolve_namespace,
     CODE src/ast/types.rs: TyConInfo::resolve_type_aliases,
     CODE src/elaboration/mod.rs: elaborate,
     CODE src/optimization/capture_struct.rs: CaptureStruct::new,
     CODE src/optimization/closure_specialization.rs: lift_all, realize_all,
         record_capture_list, take_new_tycons,
     CODE src/optimization/defunctionalize_fix.rs: run_one,
     CODE src/elaboration/desugar_opaque.rs: register_opaque_tycon

<1>3ba. `is_funptr_tycon` について次の 2 つが成り立つ。
   - (a) `is_funptr_tycon(tc)` が abort しうるのは末尾の `number.parse::<u32>().unwrap()` だけで
     ある。そこに達するのは、`tc.name.namespace` が `Std` の 1 段でありかつ `tc.name.name` が
     `FUNPTR_NAME` (`"#FunPtr"`) で始まるときに限る。
   - (b) `E.tycons()` の鍵のうち (a) の形の名前を持つのは、`bulitin_tycons` が
     `make_funptr_tycon(n)` (`n` は 1 以上 `FUNPTR_ARGS_MAX` 以下の `u32`) の下に入れるものだけで
     ある。その鍵に対して `E.tycons()` が持つ `TyConInfo` の `variant` は `TyConVariant::Primitive` で
     あり、その鍵に対する (a) の `parse::<u32>()` は成功する。
  <2>1. (a) が成り立つ。`is_funptr_tycon` は、`tc.name.namespace` が `Std` の 1 段でなければ `None`、
     `tc.name.name` が `FUNPTR_NAME` で始まらなければ `None` を返し、そのどちらでもないときにだけ
     残りの文字を `parse::<u32>()` に掛ける。ほかに abort する場所を持たない。
    BY CODE src/fixstd/builtin.rs: is_funptr_tycon, CODE src/constants.rs: FUNPTR_NAME
  <2>2. (b) の第 1 文が成り立つ。`<2>1` より (a) の形とは、`tc.name.namespace` が `Std` の 1 段で
     ありかつ `tc.name.name` が `FUNPTR_NAME` で始まることである。A28 は、`E.tycons()` の項目の
     うち鍵がその形を持つものは `bulitin_tycons()` が `make_funptr_tycon(n)` (`n` は 1 以上
     `FUNPTR_ARGS_MAX` 以下の `u32`) の鍵の下に置いた項目であると述べる。
    BY <ref id=3d4be43/>, <2>1, CODE src/fixstd/builtin.rs: bulitin_tycons,
       CODE src/fixstd/builtin.rs: make_funptr_tycon,
       CODE src/constants.rs: FUNPTR_ARGS_MAX, CODE src/constants.rs: FUNPTR_NAME
  <2>3. (b) の第 2 文が成り立つ。`<2>2` より `make_funptr_tycon(n)` を `E.tycons()` の鍵に置くのは
     `Program::calculate_type_env` が渡す `bulitin_tycons()` だけであり、`<1>3b` の表よりその
     `TyConInfo` の `variant` は `TyConVariant::Primitive` である。`<1>3b` の最後の節より、`E` に
     入った後の書き替えは `variant` を動かさない。
    BY <1>3b, <2>2, CODE src/fixstd/builtin.rs: bulitin_tycons
  <2>4. (b) の第 3 文が成り立つ。`make_funptr_tycon(n)` の名前は `make_funptr_name(n)`、すなわち
     `#FunPtr` に `n` の 10 進表記を継いだものなので、`FUNPTR_NAME` の分を落とした残りは `n` の
     10 進表記であり、`parse::<u32>()` は成功する。
    BY <2>1, <2>2, CODE src/fixstd/builtin.rs: make_funptr_name,
       CODE src/fixstd/builtin.rs: make_funptr_tycon
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4

<1>3c. `<1>1` を満たす型 `t` について、次の 4 つが成り立つ。
   - (a) `t.is_closure()`、`t.is_array()`、`t.is_funptr()`、`t.is_punched_array()` は abort せず
     真偽値を返す。
   - (b) `t.is_unbox(E)` と `t.is_box(E)` は abort せず真偽値を返す。
   - (c) `t.is_closure()` が偽であるとき、`t.toplevel_tycon_info(E)` と `t.is_union(E)` は abort せず
     値を返す。
   - (d) `t.is_closure()` と `t.is_box(E)` がどちらも偽であるとき、`t.unpunched_field_types(E)` と
     `t.field_types(E)` は abort せず停止する。`t.field_types(E)` の長さは
     `t.toplevel_tycon_info(E).fields` の長さに等しい。`F(t)` は `t.field_types(E)` の要素のうち、
     `fields` の同じ添字の成分の `is_punched` が偽であるものを、その添字とともに並べた列である。
     とくに `F(t)` の各要素 `(i, f)` について `i` は `t.field_types(E)` の長さ未満であり、`f` は
     `t.field_types(E)[i]` である。
  <2>1. (a) が成り立つ。
    <3>1. この 4 つはいずれも `toplevel_tycon_satisfies(pred)` の 1 行であり、
       `toplevel_tycon_satisfies` は `self.toplevel_tycon()` の `Option` を見て `Some` なら `pred` を、
       `None` なら偽を返す。渡される `pred` は `is_closure` が `make_arrow_name_abs()` との名前の
       等値検査、残り 3 つが `is_array_tycon`、`is_funptr_tycon`、`is_punched_array_tycon` であり、
       どれも `TyCon` の名前だけを見て `E` を引かない。
      BY CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies,
         CODE src/ast/types.rs: TypeNode::is_closure, CODE src/ast/types.rs: TypeNode::is_array,
         CODE src/ast/types.rs: TypeNode::is_funptr,
         CODE src/ast/types.rs: TypeNode::is_punched_array,
         CODE src/fixstd/builtin.rs: is_array_tycon,
         CODE src/fixstd/builtin.rs: is_punched_array_tycon
    <3>2. `is_closure` の述語は `TyCon` の名前と `make_arrow_name_abs()` の等値比較、
       `is_array_tycon` と `is_punched_array_tycon` は `TyCon` の等値比較だけを行い、abort する
       場所を持たない。
      BY CODE src/fixstd/builtin.rs: is_array_tycon,
         CODE src/fixstd/builtin.rs: is_punched_array_tycon,
         CODE src/ast/types.rs: TypeNode::is_closure
    <3>5. QED
      `<1>1` (i) より `t.toplevel_tycon()` が返す型構成子は `E.tycons()` の鍵である。`<1>3ba` (a) より
      `is_funptr_tycon` が abort しうるのはその名前が `#FunPtr` で始まる `Std` の 1 段の名前のときだけで
      あり、`<1>3ba` (b) より `E.tycons()` の鍵でその形を持つのは `make_funptr_tycon(n)` に限られて、
      そこでは `parse::<u32>()` が成功する。`<3>1` と `<3>2` と合わせて 4 つとも abort しない。
      BY <1>1, <1>3ba, <3>1, <3>2
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
       ある。`substitute_type` は型の節点についての再帰であり、`TyVar` の腕は写像を引き、当たった
       ときはその値に `set_source_if_none(ty.get_source().clone())` を当てる。`TyCon` の腕は複製、
       `TyApp` の腕は 2 つの部分に再帰してから `set_tyapp_fun`/`set_tyapp_arg` を、`AssocTy` の腕は
       各引数に再帰してから `set_assocty_args` を呼ぶ。この 3 つの setter が
       `panic!` するのは節点が対応する `Type` の腕でないときだけで、どれもその腕の中から呼ばれる。
       **`set_source_if_none` も abort しない** -- `info.source` を見て、`None` なら `set_source` が
       節点を複製して `info.source` を置き替えた新しい `Arc` を返し、`Some` なら自分の `Arc` を複製
       する。どちらの枝も場合分けと複製だけである。
       `substitute_type` が再帰する先は `TyApp` の `fun` と `arg`、および `AssocTy` の `args` の
       各要素であり、これは `<1>1a` の直接の部分の辺そのものなので、`<1>1a` より再帰は停止する。
       返る列の長さは `ti.fields` の長さである。
      BY <1>1a, CODE src/elaboration/typecheck.rs: Substitution::substitute_type,
         CODE src/ast/types.rs: TypeNode::set_tyapp_fun,
         CODE src/ast/types.rs: TypeNode::set_tyapp_arg,
         CODE src/ast/types.rs: TypeNode::set_assocty_args,
         CODE src/ast/types.rs: TypeNode::set_source_if_none,
         CODE src/ast/types.rs: TypeNode::set_source,
         CODE src/ast/types.rs: TypeNode::get_source,
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
    `<2>7` より `t.field_types(E)` は abort せず停止し、その長さは `ti.fields` の長さに等しい。
    `unpunched_field_types` は `self.toplevel_tycon_info(type_env)` を `ti` に取り、
    `self.instance_field_types(ti, type_env)` の結果に `into_iter().enumerate()` を当ててから
    `filter(|(i, _)| !ti.fields[*i].is_punched)` で絞る。`field_types` は
    `self.instance_field_types(self.toplevel_tycon_info(type_env), type_env)` の 1 文なので、
    `unpunched_field_types` が絞る列は `t.field_types(E)` そのものである。その長さは `ti.fields` の
    長さに等しいので、`EXT Iterator の enumerate と filter` より `enumerate` が渡す `i` はどれも
    `ti.fields` の添字として範囲内であり、`ti.fields[*i]` は abort しない。同じ `EXT` より `filter`
    は要素を落とすだけなので、`F(t)` は
    `t.field_types(E)` の要素のうち `ti.fields` の同じ添字の成分の `is_punched` が偽であるものを、
    その添字とともに並べた列であり、その各要素 `(i, f)` について `i` は `t.field_types(E)` の長さ
    未満で `f` は `t.field_types(E)[i]` である。
    BY <2>1, <2>2, <2>3, <2>7, EXT Iterator の enumerate と filter,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types,
       CODE src/ast/types.rs: TypeNode::field_types

<1>3ca. `<1>1` を満たす型 `t` について `t.is_closure()` が偽であるとき、`t.is_funptr()` が真ならば
   `t.toplevel_tycon_info(E).variant` は `TyConVariant::Primitive` である。対偶をとると、
   `t.toplevel_tycon_info(E).variant` が `Primitive` でなければ `t.is_funptr()` は偽である。
  <2>1. `t.is_funptr()` は `toplevel_tycon_satisfies(|tc| is_funptr_tycon(tc).is_some())` である。
     それが真であるのは `t.toplevel_tycon()` が `Some(tc)` を返し、かつ `is_funptr_tycon(tc)` が
     `Some` を返すときに限る。
    BY CODE src/ast/types.rs: TypeNode::is_funptr,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies
  <2>2. `is_funptr_tycon(tc)` が `Some` を返すとき、`tc` の名前は `<1>3ba` (a) の形である。
     `is_funptr_tycon` は 2 つの検査のどちらかに落ちたら `None` を返すからである。
    BY <1>3ba, <2>1, CODE src/fixstd/builtin.rs: is_funptr_tycon
  <2>3. `<1>1` (i) より `tc` は `E.tycons()` の鍵であり、`<1>3ba` (b) よりその `TyConInfo` の
     `variant` は `TyConVariant::Primitive` である。
    BY <1>1, <1>3ba, <2>2
  <2>4. QED
    `t.is_closure()` が偽なので `<1>3c` (c) より `t.toplevel_tycon_info(E)` は abort せず値を返し、
    その値は `E.tycons()[tc]` である。`<2>3` よりその `variant` は `Primitive` である。
    BY <1>1, <1>3c, <2>1, <2>3, CODE src/ast/types.rs: TypeNode::toplevel_tycon_info

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
  <2>1. そのような無限列があるとすると、各 `t_j` は `<1>1` を満たす。`DEF DESC` より
     `t_j DESC t_{j+1}` はある `i` について `(i, t_{j+1})` が `F(t_j)` の要素であることを含むので、
     `t_{j+1}` は `t_j` からフィールドの辺で到達できる型である。`<1>1` (ii) は `<1>1` を満たす型
     からフィールドの辺で到達できる型も `<1>1` を満たすと述べるので、`j` についての帰納で全項に
     届く。
    BY <1>1, DEF DESC
  <2>2. 各 `j` について `t_j REC t_{j+1}` である。`<2>1` より `t_j` は `<1>1` を満たし、
     `DEF DESC` より `cls(t_j)` は `UN` か `ST` なので、`<1>7` (c) を `t := t_j`、`f := t_{j+1}` に
     適用できる。
    BY <1>7, <2>1, DEF DESC
  <2>3. QED
    `<2>2` は `<1>1` を満たす型 `t_0` から始まる無限の `REC` の列であり、`<1>3d` に反する。
    BY <1>3d, <2>1, <2>2

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

<1>9a. `truncate_to_unit(ty, path, E)` は引数の値の部分関数である。すなわち、値として等しい引数を
   渡した 2 回の呼び出しは同じ振る舞いをとる -- ともに停止して等しい `FieldPath` を返すか、ともに
   abort するか、ともに停止しないかのいずれかである。`unit_step(ty, E)`、`ty.is_box(E)`、
   `ty.unpunched_field_types(E)` についても同じことが成り立つ。この段は `<1>1` を読まない。

   以下は、この道の各関数が**何を読むか**を先に数え上げ、その数え上げから 2 つの呼び出しの振る舞いが
   一致することを QED で出す形をとる。
  <2>1. `unwrap_newtypes_node(self, E, unwrapped)` が読むのは、`self.toplevel_tycon()`、
     `E.unwrapped_newtype_info(&tycon)` が返す `TyConInfo` の `tyvars` の長さと
     `fields[0].is_punched`、`self.collect_type_arguments()` の長さ、
     `self.declared_field_types(tycon_info)` の第 0 成分、`self.ty` の変位とその成分、そして
     `unwrap_newtypes_memoized` の再帰の返り値だけである。返すのは `make_unit_ty()`、その再帰の
     返り値、`self.clone()`、`self.set_tyapp_fun(new_fun_ty).set_tyapp_arg(new_arg_ty)` の
     いずれかである。
    BY CODE src/ast/types.rs: TypeNode::unwrap_newtypes_node,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon,
       CODE src/ast/types.rs: TypeNode::collect_type_arguments,
       CODE src/ast/program.rs: TypeEnv::unwrapped_newtype_info,
       CODE src/fixstd/builtin.rs: make_unit_ty
  <2>1a. 型を写す 2 つの関数 -- `unwrap_newtypes_node` と `Substitution::substitute_type` -- が置く
     `Arc::ptr_eq` の分岐は、返り値の**値**を変えない。どちらも `Type::TyApp(fun, arg)` の腕で、
     `Arc::ptr_eq(&new_fun, fun)` と `Arc::ptr_eq(&new_arg, arg)` がどちらも真のときは節点をそのまま
     複製して返し、そうでないときは `set_tyapp_fun(new_fun).set_tyapp_arg(new_arg)` を返す。真の腕に
     入るとき `new_fun` は `fun` と、`new_arg` は `arg` と同じ `Arc` なので値としても等しく、
     複製されるもとの節点の `ty` は `Type::TyApp(fun, arg)` である。偽の腕が作る節点の `ty` は
     `Type::TyApp(new_fun, new_arg)` である。`impl PartialEq for TypeNode` は `ty` だけを読み、
     `impl PartialEq for Type` は変位と成分を読むので、どちらの腕が返す値も
     `Type::TyApp(new_fun, new_arg)` を `ty` に持つ節点と等しい。

     **`substitute_type` の `Type::AssocTy(assoc_ty, args)` の腕も、返り値の値を変えない。** この腕は
     `args` の各要素に `substitute_type` を当てた `new_args` を作り、
     `new_args.iter().zip(args).all(|(new_arg, arg)| Arc::ptr_eq(new_arg, arg))` が真のときは節点を
     そのまま複製して返し、偽のときは `set_assocty_args(new_args)` を返す。分岐の条件が列の上の
     全称であるところが `Type::TyApp` の腕と異なる。`EXT Iterator の map と zip` より `map` は列の
     長さを保つので `new_args` と `args` の
     長さは等しく、`zip` は各位置の対を渡す。よって真の腕に入るとき `new_args` の各要素は `args` の
     同じ位置の要素と同じ `Arc` であり、`EXT Vec の等価性` より `Vec` の等価性は長さと同じ位置の
     要素どうしの等価性なので `new_args` は
     `args` と値としても等しい。複製されるもとの節点の `ty` は `Type::AssocTy(assoc_ty, args)` で
     ある。偽の腕が呼ぶ `set_assocty_args` は、`self.ty` が `Type::AssocTy` のときその第 1 成分を
     保って第 2 成分を `new_args` に替えるので、作る節点の `ty` は
     `Type::AssocTy(assoc_ty, new_args)` である。この腕に入るのは `self.ty` が `Type::AssocTy` の
     ときなので、`set_assocty_args` が持つ `panic!` には達しない。`impl PartialEq for Type` は変位と
     成分を読むので、どちらの腕が返す値も `Type::AssocTy(assoc_ty, new_args)` を `ty` に持つ節点と
     等しい。

     **同じ関数の `Type::TyVar` の腕が呼ぶ `set_source_if_none` も値を変えない** -- 書き替えるのは
     `info.source` であり、`impl PartialEq for TypeNode` はそれを読まない。
    BY <2>1, EXT Iterator の map と zip, EXT Vec の等価性,
       CODE src/ast/types.rs: TypeNode::set_tyapp_fun,
       CODE src/ast/types.rs: TypeNode::set_tyapp_arg,
       CODE src/ast/types.rs: TypeNode::set_assocty_args,
       CODE src/ast/types.rs: TypeNode::set_source_if_none,
       CODE src/ast/types.rs: TypeNode::set_source,
       CODE src/ast/types.rs: impl PartialEq for TypeNode,
       CODE src/ast/types.rs: impl PartialEq for Type,
       CODE src/ast/types.rs: TypeNode::unwrap_newtypes_node,
       CODE src/elaboration/typecheck.rs: Substitution::substitute_type
  <2>2. `unwrap_newtypes_memoized(self, E, unwrapped)` が読むのは、`unwrapped` を `self` を鍵に引いた
     結果と、`unwrap_newtypes_node` の返り値だけである。`Map` は `FxHashMap` なので、鍵の一致は
     `Arc<TypeNode>` の `Hash` と `Eq`、すなわち `TypeNode` の `Hash` と `PartialEq` で決まり、
     どちらも `ty` だけを読む。
    BY CODE src/ast/types.rs: TypeNode::unwrap_newtypes_memoized,
       CODE src/ast/types.rs: impl Hash for TypeNode,
       CODE src/ast/types.rs: impl PartialEq for TypeNode,
       CODE src/ast/types.rs: TypeNode::type_hash, CODE src/misc.rs: Map
  <2>3. `unpunched_field_types(self, E)` が読むのは、`self.toplevel_tycon_info(E)` が返す
     `TyConInfo` の `fields[i].is_punched` と、`self.instance_field_types(ti, E)` の返り値だけで
     ある。`instance_field_types` が読むのは `self.declared_field_types(ti)` の返り値、`ti.tyvars` の
     各 kind、そして宣言が kind `*` でない型変数を持つときの `unwrap_newtypes_memoized` の返り値で
     あり、そこへ渡す `unwrapped` はこの呼び出しがその場で作る空の `Map` である。
     `declared_field_types` が読むのは `self.collect_type_arguments()`、`ti.tyvars`、`ti.fields` の
     各 `ty` と、`Substitution::single` / `merge` / `substitute_type` の返り値だけである。
     `Substitution` は `Map<String, Arc<TypeNode>>` を 1 つ包む値であり、`single` はその 1 要素の
     写像を作り、`merge` は鍵ごとに値を `==` で突き合わせ、`substitute_type` は型の節点を降りて
     `data` を型変数の名前で引く。
     `toplevel_tycon_info` が読むのは `self.is_closure()`、`self.toplevel_tycon()`、`E.tycons()` の
     1 回の探索だけである。
    BY CODE src/ast/types.rs: TypeNode::unpunched_field_types,
       CODE src/ast/types.rs: TypeNode::instance_field_types,
       CODE src/ast/types.rs: TypeNode::declared_field_types,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
       CODE src/ast/types.rs: TypeNode::collect_type_arguments,
       CODE src/elaboration/typecheck.rs: Substitution::single,
       CODE src/elaboration/typecheck.rs: Substitution::merge,
       CODE src/elaboration/typecheck.rs: Substitution::substitute_type
  <2>4. `unit_step(ty, E)` が読むのは、`is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、
     `is_array`、`is_punched_array` の返り値、`toplevel_tycon_info(E).fields` の長さ、そして
     `unpunched_field_types(E)` の返り値だけであり、返す `UnitStep` はそれと定数
     `CLOSURE_CAPTURE_IDX` / `CLOSURE_FIELD_COUNT` から組まれる。上に挙げた述語のうち
     `is_closure`、`is_array`、`is_punched_array` は `toplevel_tycon_satisfies` を経て
     `toplevel_tycon()` が返す `TyCon` の名前だけを見る (`TyCon` は名前 1 つの構造体なので、
     `is_array_tycon` と `is_punched_array_tycon` が行う `TyCon` の等値比較も名前の比較である)。
     `is_union` が読むのは `toplevel_tycon_info(E).variant` だけである。`is_box` は `is_unbox` の
     否定であり、`is_unbox` は `is_closure() || toplevel_tycon_info(E).is_unbox` である。
     `is_fully_unboxed` が読むのは `is_box`、`is_closure`、`is_array`、`is_funptr` の返り値と、
     この 4 つがすべて偽であるときの `unpunched_field_types(E)` の第 2 成分についての再帰である。
     `is_funptr` も `toplevel_tycon_satisfies` を経て `toplevel_tycon()` が返す `TyCon` を
     `is_funptr_tycon` に渡すだけであり、`is_funptr_tycon` が読むのはその名前だけである。
     `toplevel_tycon()` が読むのは `self.ty` の変位と、`Type::TyApp` の関数側への再帰だけである。
    BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/ast/types.rs: TypeNode::is_closure, CODE src/ast/types.rs: TypeNode::is_box,
       CODE src/ast/types.rs: TypeNode::is_unbox, CODE src/ast/types.rs: TypeNode::is_union,
       CODE src/ast/types.rs: TypeNode::is_array, CODE src/ast/types.rs: TypeNode::is_funptr,
       CODE src/ast/types.rs: TypeNode::is_punched_array,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info, CODE src/ast/types.rs: TyCon,
       CODE src/fixstd/builtin.rs: is_array_tycon, is_punched_array_tycon, is_funptr_tycon,
       CODE src/constants.rs: CLOSURE_CAPTURE_IDX, CODE src/constants.rs: CLOSURE_FIELD_COUNT
  <2>5. `truncate_to_unit(ty, path, E)` が読むのは、`path` の各要素と、`unit_step(&cur, E)` と
     `held_field_type(&held_fields, idx, "truncate_to_unit")` の返り値だけである。局所変数は `out`
     と `cur` の 2 つで、その推移はその返り値と `path` の要素で決まる。`held_field_type` は
     `held_fields` を `idx` で線形に探すだけである。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/rc_ir/ownership.rs: held_field_type
  <2>6. この道で `TypeNode` の内部可変性の欄が書かれるのは 1 か所である -- `<2>2` の `Map` の探索と
     挿入が `Arc<TypeNode>` の鍵をハッシュし、`impl Hash for TypeNode` が `type_hash` を経て
     `hash_cache.get_or_init` を実行する。A3 の「**`RcProgram` から到達できる値の等しさは、それを
     共有参照で受け取る計算が変えない。** 到達できる型が内部可変性を持つ欄を持つときは、その欄は
     **一度だけ書かれる memo であって、その値はその型の `PartialEq` が読む成分の関数である**」より、
     その書き込みは型の値を変えない。
    BY <ref id=e11772a/> (`RcProgram` から到達できる値の等しさは、それを共有参照で受け取る計算が変えない), <2>2,
       CODE src/ast/types.rs: TypeNode (`hash_cache` の宣言),
       CODE src/ast/types.rs: TypeNode::type_hash,
       CODE src/ast/types.rs: impl Hash for TypeNode
  <2>6a. `TypeNode` の等価比較は、比べる 2 つの値だけで決まる真偽値を返し、abort しない。
     `impl PartialEq for TypeNode` が読むのは `ty` だけであり、`impl PartialEq for Type` の doc は
     「Compares the parts of the type expression, taking two occurrences of one node as equal on
     sight」と述べる。すなわち同じ節点の 2 つの出現を等しいと答える道を持つが、返す真偽値は型の式の
     比較のものである。その再帰が辿るのは `<1>1a` の直接の部分の辺なので停止する。この道がこの比較を
     行うのは、`<2>2` の `Map` が `Arc<TypeNode>` の鍵を突き合わせるところと、`<2>3` の
     `Substitution::merge` が同じ鍵の値を `==` で突き合わせるところである。
    BY <1>1a, <2>2, <2>3, CODE src/ast/types.rs: impl PartialEq for Type,
       CODE src/ast/types.rs: impl PartialEq for TypeNode
  <2>7. QED
    値として等しい引数を渡した 2 つの呼び出しを `C` と `C'` とし、`DEF 下位の呼び出しの列` が
    それぞれに与える列を先頭から 1 対 1 に並べる。
    <3>1. `<2>1` から `<2>5` は、この道の各関数が読むものを尽くしている。挙がるのはどれも、引数の
       値、`E` の値、`TyConInfo` の欄の値、その呼び出しがその場で作る局所の値、そして下位の
       呼び出しの返り値である。
      BY <2>1, <2>2, <2>3, <2>4, <2>5
    <3>2. `EXT Rust の評価の決定性` の (iv) が許す番地の比較が答えに漏れる場所は、この道に 3 つある。
       型を写す 2 つの関数が置く `Arc::ptr_eq` の分岐 (`<2>1a`)、`Map` の鍵をハッシュするときに
       書かれる memo (`<2>6`)、そして `TypeNode` の等価比較 (`<2>6a`) である。3 つとも、番地が
       一致するかどうかに依らず同じ値を返す。`Arc::ptr_eq` の 2 つの腕はどちらも下位の呼び出しを
       行わないので、どちらの腕を取るかは下位の呼び出しの列にも現れない。
      BY <2>1a, <2>6, <2>6a, EXT Rust の評価の決定性,
         CODE src/ast/types.rs: TypeNode::unwrap_newtypes_node,
         CODE src/elaboration/typecheck.rs: Substitution::substitute_type
    <3>3. `n` についての帰納で、`C` と `C'` の列の先頭 `n` 項が `DEF 下位の呼び出しの列` の意味で
       対応することを示す。`n = 0` では主張は空虚である。先頭 `n` 項が対応するとして、第 `n + 1` 項を
       見る。その時点で走っている最も内側の呼び出しを `K` とすると、次に起きること -- 下位の呼び出しを
       開始するか、`K` が値を返すか、`K` が abort するか、どれも起きないか -- と、開始ならその関数と
       引数の値、返りならその値は、`EXT Rust の評価の決定性` より `K` の (i) 引数の値、(ii) `K` が
       記憶域から読む値、(iii) `K` がそこまでに受け取った下位の呼び出しの返り値、(iv) `K` が比べる
       番地の一致だけで決まる。`K` の開始の事象と `K` が受け取った返りの事象はどれも先頭 `n` 項に
       在るので、(i) と (iii) は帰納法の仮定より `C` と `C'` で等しい。(ii) は `<3>1` より `E` の値と
       `TyConInfo` の欄の値であり、`C` と `C'` はその同じ値を持つ。(iv) は `<3>2` より答えを動かさ
       ない。よって第 `n + 1` 項も対応し、一方に在れば他方にも在る。
      BY <3>1, <3>2, EXT Rust の評価の決定性, DEF 下位の呼び出しの列
    <3>4. QED
      `<3>3` より `C` と `C'` の列は全体として対応する。`K` を根の呼び出しに取った `<3>3` の帰納段が、
      根についても同じ 4 つで振る舞いを決めるので、`C` と `C'` は、ともに停止して等しい値を返すか、
      ともに同じ `panic!` / `assert` / 添字付けに達するか、ともに停止しない。`<2>4` と `<2>5` は
      この結論を `unit_step`・`is_box`・`unpunched_field_types`・`truncate_to_unit` のどれについても
      同じ形で与える。
      BY <2>4, <2>5, <3>1, <3>2, <3>3, EXT Rust の評価の決定性, DEF 下位の呼び出しの列

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
    BY <ref id=9cba81c/>, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>2. CASE `cls(t) = CL`。定義より `is_fully_unboxed` は偽、`is_closure()` は真である。`unit_step`
     は 2 番目の検査で
     `UnitStep::Capture { capture_idx: CLOSURE_CAPTURE_IDX as usize, field_count: CLOSURE_FIELD_COUNT }`
     を返す。
    BY <ref id=9cba81c/>, DEF cls, CODE src/rc_ir/ownership.rs: unit_step, CODE src/constants.rs: CLOSURE_CAPTURE_IDX
  <2>3. CASE `cls(t) = BX`。定義より `is_fully_unboxed` と `is_closure` は偽、`is_box(E)` は真で
     ある。`unit_step` の 3 番目の検査は `is_box || is_union || is_array || is_punched_array` の
     選言なので真になり、`UnitStep::Unit` を返す。
    BY <ref id=9cba81c/>, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>4. CASE `cls(t) = AR`。定義より `is_fully_unboxed`、`is_closure`、`is_box` は偽、`is_array()` は
     真である。`<2>3` と同じ選言の第 3 項が真になり `UnitStep::Unit` を返す。
    BY <ref id=9cba81c/>, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>5. CASE `cls(t) = UN`。定義より `is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽、
     `is_union(E)` か `is_punched_array()` のどちらかが真である。同じ選言の第 2 項 (unbox union) か
     第 4 項 (punched array) が真になり `UnitStep::Unit` を返す。
    BY <ref id=9cba81c/>, DEF cls, CODE src/rc_ir/ownership.rs: unit_step
  <2>6. CASE `cls(t) = ST`。定義より `is_fully_unboxed`、`is_closure`、`is_box`、`is_union`、
     `is_array`、`is_punched_array` がすべて偽である。よって `unit_step` は 3 つの検査をすべて通り
     抜け、
     `UnitStep::Fields { field_count: t.toplevel_tycon_info(E).fields.len(), held_fields: t.unpunched_field_types(E) }`
     を返す。`held_fields` は `F(t)` そのものである。
    BY <ref id=9cba81c/>, DEF cls, DEF F, CODE src/rc_ir/ownership.rs: unit_step
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
    BY <ref id=0594f24/>, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>2. CASE `cls(t) = CL`。`is_fully_unboxed` は偽、`is_closure()` は真である。`go` は
     `path.push(CLOSURE_CAPTURE_IDX as usize)`、`out.push(path.clone())`、`path.pop()` を行って
     `return` する。積まれる path は `path ++ [c]` である。
    BY <ref id=0594f24/>, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`),
       CODE src/constants.rs: CLOSURE_CAPTURE_IDX
  <2>3. CASE `cls(t) = BX`。`is_fully_unboxed` と `is_closure` は偽、`is_box(E)` は真である。`go` は
     `out.push(path.clone())` を行って `return` する。
    BY <ref id=0594f24/>, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>4. CASE `cls(t) = AR`。`is_fully_unboxed`、`is_closure`、`is_box` は偽、`is_array()` は真で
     ある。`go` は `is_array` の検査で `out.push(path.clone())` を行って `return` する。
    BY <ref id=0594f24/>, DEF cls, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>5. CASE `cls(t) = UN`。`is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽である。`go`
     にはこの 4 つの検査しか無いので、そのすべてを通り抜けて最後の
     `for (i, fty) in ty.unpunched_field_types(type_env)` に進み、各要素について `path.push(i)`、
     `go(&fty, ..., path, out)`、`path.pop()` を行う。D4 の第 5 項が言うとおり、`F(t)` は穴 (punched
     field) を含まないので、穴の下へは降りない。
    BY <ref id=0594f24/>, DEF cls, DEF F, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>6. CASE `cls(t) = ST`。`is_fully_unboxed`、`is_closure`、`is_box`、`is_array` は偽である (`ST`
     はこの 4 つに加えて `is_union` と `is_punched_array` も偽である場合だが、`go` はその 2 つを
     問わない)。`<2>5` と同じ最後のループに進む。
    BY <ref id=0594f24/>, DEF cls, DEF F, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (内部関数 `go`)
  <2>7. QED
    `<1>4` より場合は尽きており排他である。`UN` と `ST` で振る舞いが同じことが、この 2 つの walk の
    違いの全部である。`unit_step` は `UN` で止まり (`<1>10`)、`go` は `UN` で降りる。
    BY <1>4, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>12. `<1>1` を満たす型 `t` で `F(t)` が定まるとき、`F(t)` に現れる添字は相異なる。したがって
   `fld(t, i)` は一価であり、`(i, f)` が `F(t)` の要素であるとき `held_field_type(F(t), i, w)` は
   abort せず `f` を返す。
  <2>1. `unpunched_field_types` は `instance_field_types(...)` の結果に `into_iter().enumerate()` を
     適用してから `filter` する。`EXT Iterator の enumerate と filter` より、`enumerate` の添字は
     0 から始まる相異なる整数の列であり、`filter` はその一部を残すだけである。
    BY EXT Iterator の enumerate と filter,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types
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
  <2>3a. `s` が `<1>1` を満たし、`p` が `s` の長さ 1 以上の ST-道であるとき `cls(s) = ST` である。
     `DEF ST-道` の条件を `j = 0` に読むとそれである。したがって `cls(s)` が `ST` でないとき、
     `s` の ST-道は `[]` だけである。
    BY DEF ST-道
  <2>3b. `s` が `<1>1` を満たし `cls(s) = ST` であるとき、`s` の ST-道は、`[]` と、`F(s)` の各
     `(i, f)` と `f` の各 ST-道 `p'` についての `[i] ++ p'` で尽きる。
     `end(s, [i] ++ p') = end(f, p')` である。
    <3>1. `[i] ++ p'` は `s` の ST-道である。`DEF ST-道` の `s_0 = s` のクラスは場合の仮定より `ST`
       であり、`fld(s, i) = f` なので `s_1 = f` であり、以後の各段は `p'` が `f` の ST-道である
       ことから `ST` である。
      BY DEF ST-道, DEF fld
    <3>2. 逆に `p` が `s` の長さ 1 以上の ST-道であるとき、`p = [p[0]] ++ p[1..]` と書くと
       `(p[0], fld(s, p[0]))` は `F(s)` の要素であり、`p[1..]` は `fld(s, p[0])` の ST-道である。
       `DEF ST-道` の `s_j` の定義を位置 1 で切ったものがそれである。
      BY DEF ST-道, DEF fld
    <3>3. QED
      `end` は `fld` の連鎖の終点なので、位置 1 で切ると `end(s, [i] ++ p') = end(f, p')` である。
      BY <3>1, <3>2, DEF ST-道, DEF fld
  <2>4. `<1>1` を満たす任意の型 `s` と任意の path `q` について、`R(s, q)` は次の 2 つの集合の合併で
     ある。
     - `{ q ++ p : p は s の ST-道で、cls(end(s, p)) が BX、AR、UN のどれか }`
     - `{ q ++ p ++ [c] : p は s の ST-道で、cls(end(s, p)) = CL }`

     `q` を渡って一般化するのは、`<2>3` の再帰式が `q ++ [i]` を渡して降りるからである。以下は
     `DESC` に関する整礎帰納法 (`<1>8`) であり、帰納法の仮定は `s DESC f` を満たす各 `f` と
     **任意の** `q` についてこの主張が成り立つことである。
    <3>1. CASE `cls(s) = NB`。`<2>3` より `R(s, q)` は空集合である。`<2>3a` より `s` の ST-道は
       `[]` だけで、`cls(end(s, [])) = cls(s) = NB` はどちらの集合の条件にも当たらないので、
       2 つの集合はどちらも空である。
      BY <2>3, <2>3a, DEF ST-道
    <3>2. CASE `cls(s) = CL`。`<2>3` より `R(s, q)` は `{q ++ [c]}` である。`<2>3a` より `s` の
       ST-道は `[]` だけで、`cls(end(s, [])) = CL` は第 2 の集合の条件に当たるので、第 1 の集合は
       空、第 2 の集合は `{q ++ [] ++ [c]} = {q ++ [c]}` である。
      BY <2>3, <2>3a, DEF ST-道
    <3>3. CASE `cls(s)` が `BX`、`AR`、`UN` のどれか。`<2>3` より `R(s, q)` は `{q}` である。
       `<2>3a` より `s` の ST-道は `[]` だけで、`cls(end(s, []))` はその 3 つのどれかなので、第 1 の
       集合は `{q ++ []} = {q}`、第 2 の集合は空である。
      BY <2>3, <2>3a, DEF ST-道
    <3>4. CASE `cls(s) = ST`。`<2>3` より `R(s, q)` は `F(s)` の各 `(i, f)` についての
       `R(f, q ++ [i])` の合併である。`<1>1` (ii) より各 `f` は `<1>1` を満たし、`DEF DESC` より
       `s DESC f` なので、帰納法の仮定を `f` と path `q ++ [i]` に適用できる。それより
       `R(f, q ++ [i])` は `{ q ++ [i] ++ p' : p' は f の ST-道で cls(end(f, p')) が BX、AR、UN の
       どれか }` と `{ q ++ [i] ++ p' ++ [c] : p' は f の ST-道で cls(end(f, p')) = CL }` の合併で
       ある。`<2>3b` より、`i` と `p'` を渡ったときの `[i] ++ p'` は `s` の長さ 1 以上の ST-道を
       ちょうど尽くし、`cls(end(s, [i] ++ p')) = cls(end(f, p'))` である。残る ST-道は `[]` だけで
       あり、`cls(end(s, [])) = cls(s) = ST` はどちらの集合の条件にも当たらないので、主張の 2 つの
       集合はこの合併に一致する。
      BY <1>1, <2>3, <2>3b, DEF DESC, DEF ST-道
    <3>5. QED
      `<1>4` より `cls(s)` は 6 つのどれかであり、`<3>1` から `<3>4` がそれを尽くしている。整礎性は
      `<1>8` が与える。
      BY <1>4, <1>8, <3>1, <3>2, <3>3, <3>4
  <2>4a. `<2>4` を `s := t`、`q := []` に適用すると、`R(t, [])` は
     `{ p : p は t の ST-道で cls(end(t, p)) が BX、AR、UN のどれか }` と
     `{ p ++ [c] : p は t の ST-道で cls(end(t, p)) = CL }` の合併である。
    BY <2>4
  <2>5. QED
    BY <2>1, <2>4a

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
  <2>3a. `s` が `<1>1` を満たし、`p` が `s` の長さ 1 以上の UNST-道であるとき `cls(s)` は `UN` か
     `ST` である。`DEF UNST-道` の条件を `j = 0` に読むとそれである。したがって `cls(s)` が `UN`
     でも `ST` でもないとき、`s` の UNST-道は `[]` だけである。
    BY DEF UNST-道
  <2>3b. `s` が `<1>1` を満たし `cls(s)` が `UN` か `ST` であるとき、`s` の UNST-道は、`[]` と、
     `F(s)` の各 `(i, f)` と `f` の各 UNST-道 `p'` についての `[i] ++ p'` で尽きる。
     `end(s, [i] ++ p') = end(f, p')` である。
    <3>1. `[i] ++ p'` は `s` の UNST-道である。`DEF UNST-道` の `s_0 = s` のクラスは場合の仮定より
       `UN` か `ST` であり、`fld(s, i) = f` なので `s_1 = f` であり、以後の各段は `p'` が `f` の
       UNST-道であることから `UN` か `ST` である。
      BY DEF UNST-道, DEF fld
    <3>2. 逆に `p` が `s` の長さ 1 以上の UNST-道であるとき、`p = [p[0]] ++ p[1..]` と書くと
       `(p[0], fld(s, p[0]))` は `F(s)` の要素であり、`p[1..]` は `fld(s, p[0])` の UNST-道である。
       `DEF UNST-道` の `s_j` の定義を位置 1 で切ったものがそれである。
      BY DEF UNST-道, DEF fld
    <3>3. QED
      `end` は `fld` の連鎖の終点なので、位置 1 で切ると `end(s, [i] ++ p') = end(f, p')` である。
      BY <3>1, <3>2, DEF UNST-道, DEF fld
  <2>4. `<1>1` を満たす任意の型 `s` と任意の path `q` について、`G(s, q)` は次の 2 つの集合の合併で
     ある。
     - `{ q ++ p : p は s の UNST-道で、cls(end(s, p)) が BX か AR }`
     - `{ q ++ p ++ [c] : p は s の UNST-道で、cls(end(s, p)) = CL }`

     `q` を渡って一般化するのは、`<2>3` の再帰式が `q ++ [i]` を渡して降りるからである。以下は
     `DESC` に関する整礎帰納法 (`<1>8`) であり、帰納法の仮定は `s DESC f` を満たす各 `f` と
     **任意の** `q` についてこの主張が成り立つことである。
    <3>1. CASE `cls(s) = NB`。`<2>3` より `G(s, q)` は空集合である。`<2>3a` より `s` の UNST-道は
       `[]` だけで、`cls(end(s, [])) = cls(s) = NB` はどちらの集合の条件にも当たらないので、
       2 つの集合はどちらも空である。
      BY <2>3, <2>3a, DEF UNST-道
    <3>2. CASE `cls(s) = CL`。`<2>3` より `G(s, q)` は `{q ++ [c]}` である。`<2>3a` より `s` の
       UNST-道は `[]` だけで、`cls(end(s, [])) = CL` は第 2 の集合の条件に当たるので、第 1 の集合は
       空、第 2 の集合は `{q ++ [] ++ [c]} = {q ++ [c]}` である。
      BY <2>3, <2>3a, DEF UNST-道
    <3>3. CASE `cls(s)` が `BX` か `AR`。`<2>3` より `G(s, q)` は `{q}` である。`<2>3a` より `s` の
       UNST-道は `[]` だけで、`cls(end(s, []))` はその 2 つのどちらかなので、第 1 の集合は
       `{q ++ []} = {q}`、第 2 の集合は空である。
      BY <2>3, <2>3a, DEF UNST-道
    <3>4. CASE `cls(s)` が `UN` か `ST`。`<2>3` より `G(s, q)` は `F(s)` の各 `(i, f)` についての
       `G(f, q ++ [i])` の合併である。`<1>1` (ii) より各 `f` は `<1>1` を満たし、`DEF DESC` より
       `s DESC f` なので、帰納法の仮定を `f` と path `q ++ [i]` に適用できる。それより
       `G(f, q ++ [i])` は `{ q ++ [i] ++ p' : p' は f の UNST-道で cls(end(f, p')) が BX か AR }` と
       `{ q ++ [i] ++ p' ++ [c] : p' は f の UNST-道で cls(end(f, p')) = CL }` の合併である。
       `<2>3b` より、`i` と `p'` を渡ったときの `[i] ++ p'` は `s` の長さ 1 以上の UNST-道を
       ちょうど尽くし、`cls(end(s, [i] ++ p')) = cls(end(f, p'))` である。残る UNST-道は `[]` だけで
       あり、`cls(end(s, [])) = cls(s)` は `UN` か `ST` なのでどちらの集合の条件にも当たらないので、
       主張の 2 つの集合はこの合併に一致する。
      BY <1>1, <2>3, <2>3b, DEF DESC, DEF UNST-道
    <3>5. QED
      `<1>4` より `cls(s)` は 6 つのどれかであり、`<3>1` から `<3>4` がそれを尽くしている。整礎性は
      `<1>8` が与える。
      BY <1>4, <1>8, <3>1, <3>2, <3>3, <3>4
  <2>4a. `<2>4` を `s := t`、`q := []` に適用すると、`G(t, [])` は
     `{ p : p は t の UNST-道で cls(end(t, p)) が BX か AR }` と
     `{ p ++ [c] : p は t の UNST-道で cls(end(t, p)) = CL }` の合併である。
    BY <2>4
  <2>5. 分解の一意性。`lam` が `L(t)` の要素で `lam = p ++ e` が `<2>4a` の分解であるとする。`q` が
     `lam` の別の前置で `t` の UNST-道であり `cls(end(t, q))` が `UN` でも `ST` でもないとする。
     ここで `s_j` は `lam` に沿う位置 `j` の型、すなわち `s_0 := t`、`s_{j+1} := fld(s_j, lam[j])` と
     する。`p` も `q` も `lam` の前置なので、`|p|` 以下・`|q|` 以下の位置については、この `s_j` は
     `p` について `DEF UNST-道` が置く `s_j` とも、`q` について同じ定義が置く `s_j` とも一致する。
     `|q|` が `|p|` 未満なら、`p` が UNST-道であることから `cls(end(t, q)) = cls(s_{|q|})` は `UN` か
     `ST` であり、仮定に反する。`|q|` が `|p|` より大きいなら、`q` が UNST-道であることから
     `cls(s_{|p|}) = cls(end(t, p))` は `UN` か `ST` であり、`<2>4a` の `cls(end(t, p))` が `BX`、
     `AR`、`CL` のどれかであることに反する。よって `|q| = |p|`、すなわち `q = p` である。
    BY <2>4a, DEF UNST-道, DEF fld
  <2>6. QED
    BY <2>1, <2>4a, <2>5

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

<1>20. **P1 が成り立つ。** すなわち A28 を満たす型環境 `E` の下で、`<1>1` を満たす型 `t` について、
   `L(t)` の各要素 `lam` の
   `T(t, lam)` は `U(t)` の要素であり、`U(t)` の各要素 `u` はある `L(t)` の要素の `T(t, ・)` で
   ある。`<1>1` は A10 をこの文書の記法で述べたものなので (第 3 節)、これは README の P1
   --「**A10 を満たす**任意の型 `τ` について」-- である。

   **A28 は型ではなく型環境に掛かるので、P1 が型に置く条件には入らない。**`<1>18` と `<1>19` は
   `<1>3e` を経て `<1>3c` を読み、`<1>3c` の `<2>1` の `<3>5` が `<1>3ba` を、`<1>3ba` の `<2>2` が
   A28 を読む。A28 は第 4 節の仮定の 1 つであり、T の前提が量化する集合はその全体なので、この文書は
   それを固定した `E` について P1 を結論する。

   A10 が型に条件を置くことが空虚でないのは `<1>19a` による。`t.is_closure()` が偽で型構成子が `E`
   に無い型については `L(t)` も `U(t)` も定まらず、P1 の言明の 2 つの辺が意味を持たない。
  BY <ref id=3d4be43/>, <1>18, <1>19, <1>19a

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
     有限の木である。`collect_bindings` が自分を呼ぶ先は、`Let` の継続と、その右辺が `Match` の
     ときの各アームの本体、`Destructure` の継続、`Retain`/`Release`/`Eval` の継続だけであり、
     どれも節点の子なので、この再帰は木の各節点をちょうど 1 度訪れて終わる。`Match` の腕が呼ぶ
     `returned_var` も停止する -- `returned_var` の本体は `grow_stack` に包まれており、A15 より
     その閉包はちょうど 1 度呼ばれ、閉包が自分を呼ぶ先は継続だけなので、D2 の有限の木の上で
     継続の鎖は有限である。D2 より式は 6 種であり、1 つの節点の訪問が行う `insert` は、`Let` で 1 つと
     各アームの payload で 1 つずつ、`Destructure` で `fields` の長さだけ、残る 4 種 --
     `Retain`、`Release`、`Eval`、`Ret` -- で 0 個であり、どれも有限である。
    BY <ref id=3e6b0e0/>, <ref id=b3dfa37/>, CODE src/rc_ir/ownership.rs: collect_bindings,
       CODE src/rc_ir/ownership.rs: returned_var, CODE src/misc.rs: grow_stack
  <2>4. `<2>1` から `<2>3` より挿入は有限列をなす。挿入される名前は、パラメータ、capture、`Let` の
     束縛変数、`Destructure` のフィールド変数、`Match` のアームの payload であり、どれもプログラムの
     束縛変数である。A6 よりこれらの名前は相異なる。
    BY <ref id=33c54dc/>, <2>1, <2>2, <2>3, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>5. QED
    BY <2>4

<1>21a. `VarTable::of(func)` または `VarTable::body_only(body)` が作る表 `vars` について、
   `vars.bindings` の定義域は `vars.var_tys` の定義域に含まれ、`u` がその定義域にあるとき
   `vars.var_tys[u]` は `u` を束縛する `RcVar` の `ty`、すなわち `<1>3a` (vi) の `ty(u)` である。
  <2>1. `VarTable::of` は各パラメータ・capture について、`p.name` を鍵とする 3 つの挿入 --
     `vars.bindings.insert(p.name, Binding::Param)`、`vars.param_tys.insert(p.name, p.ty)`、
     `vars.var_tys.insert(p.name, p.ty)` -- をこの順に行い、そののち `collect_bindings` を呼ぶ。
     この 3 つはどれも同じ名前を鍵に取るので、`bindings` に入った名前は `var_tys` にも入る。
     `VarTable::body_only` は `collect_bindings` だけを呼ぶ。
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

<1>22. `<1>21` と同じ本体と表 `vars` を取る。`collect_bindings` の各節点での挿入の順序は
   次の通りである。節点 `n` の呼び出しが行う挿入の全体を
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

<1>23. `<1>21` と同じ本体と表 `vars` を取る。その本体の各節点 `n` について、`Scope(n)` の各名前は
   `<1>21` の列に現れ、その挿入は `Ins(n)` が始まるより前である。とくに `Scope(n)` の各名前は
   `vars.bindings` の定義域にあり、`ord` の値は 0 以上である。
  <2>1. `n` が根のとき。`VarTable::of` の場合 `Scope(根)` はパラメータと capture の名前で、これらは
     `collect_bindings` を呼ぶ前に `vars.bindings` へ挿入される。`Ins(根)` は
     `collect_bindings` の呼び出しが行う挿入の全体なので、どれも `Ins(根)` の開始より前である。
     `VarTable::body_only` の場合 `Scope(根)` は空集合であり、主張は空虚に成り立つ。
    BY <1>21, <1>22, DEF Scope, CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only
  <2>2. ASSUME: `n` について主張が成り立ち、`n'` は `DEF Scope` が `Scope(n')` を `Scope(n)` から
     定める子である
     PROVE: `n'` について主張が成り立つ
    <3>1. CASE `n = Let(x, rhs, k)` かつ `n' = k`。`Scope(k)` は `Scope(n)` に `x` を加えたもので
       ある。`Scope(n)` の各名前は帰納法の仮定より `<1>21` の列に現れ、`Ins(n)` の開始より前に
       挿入されており、`Ins(n)` の開始は `Ins(k)` の開始より前である。`x` は `<1>22` より
       `Ins(k)` が始まる直前に挿入されるので、これも列に現れる。
      BY <1>22, DEF Scope
    <3>2. CASE `n = Let(x, Match(scrut, arms), k)` かつ `n'` が `arms` のあるアームの `body`。
       `Scope(arm.body)` は `Scope(n)` に `arm.payload` を加えたものである。`Scope(n)` の各名前は
       帰納法の仮定より `<1>21` の列に現れ、`Ins(n)` の開始より前に挿入されており、`Ins(n)` の開始は
       `Ins(arm.body)` の開始より前である。`arm.payload` は `<1>22` より `Ins(arm.body)` が始まる
       直前に挿入されるので、これも列に現れる。
      BY <1>22, DEF Scope
    <3>3. CASE `n = Destructure(cont, fields, s, k)` かつ `n' = k`。`Scope(k)` は `Scope(n)` に
       `fields` の各変数を加えたものである。`Scope(n)` の各名前は帰納法の仮定より `<1>21` の列に
       現れ、`Ins(n)` の開始より
       前に挿入されており、`Ins(n)` の開始は `Ins(k)` の開始より前である。`fields` の各変数は
       `<1>22` より `Ins(k)` が始まる前に挿入されるので、これも列に現れる。
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

<1>24. `<1>21` と同じ本体と表 `vars` を取る。その本体の節点 `n'` が節点 `n` の部分木に属するとき、
   `Scope(n')` の各名前は、`Scope(n)` の要素であるか、`Ins(n)` の中で挿入される。
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

<1>25. `<1>21` と同じ本体と表 `vars` を取る。`x` が `vars.bindings` の定義域の要素であり、
   `origin_inner(vars, E, x, pi)` がある `pi` について `origin(vars, E, y, pi')` を呼ぶとき、
   `ord(y) < ord(x)` である。
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
    <3>5. CASE `y.name` が `Scope(m(x))` の要素である。`<1>23` より `y.name` は `<1>21` の列に
       現れ、その挿入は `Ins(m(x))` が始まるより前である。`<1>22` より `x` は `Ins(m(x))` の中で
       挿入される。よって `ord(y.name) < ord(x)` である。
      BY <1>21, <1>22, <1>23
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
    <3>5. CASE `args[j].name` が `Scope(m(x))` の要素である。`<1>23` より `args[j].name` は
       `<1>21` の列に現れ、その挿入は `Ins(m(x))` の開始より前である。`<1>22` より `x` は
       `Ins(m(x))` の中で挿入される。よって `ord(args[j].name) < ord(x)` である。
      BY <1>21, <1>22, <1>23
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
    <3>5. CASE `container.name` が `Scope(m(x))` の要素である。`<1>23` より `container.name` は
       `<1>21` の列に現れ、その挿入は `Ins(m(x))` の開始より前である。`<1>22` より `x` は
       `Ins(m(x))` の中で挿入される。よって `ord(container.name) < ord(x)` である。
      BY <1>21, <1>22, <1>23
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
    <3>5. CASE `scrut.name` が `Scope(m(x))` の要素である。`<1>23` より `scrut.name` は `<1>21` の
       列に現れ、その挿入は `Ins(m(x))` の開始より前である。`<1>22` より `x`、すなわち
       `arm.payload.name` は `Ins(m(x))` の中で挿入される。よって `ord(scrut.name) < ord(x)` で
       ある。
      BY <1>21, <1>22, <1>23
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
      <4>3. CASE `arm_result.name` が `Scope(arm.body)` の要素である。`<1>23` より
         `arm_result.name` は `<1>21` の列に現れ、その挿入は `Ins(arm.body)` の開始より前である。
         `<4>2` より `x` の挿入は `Ins(arm.body)` が済んだ後なので、
         `ord(arm_result.name) < ord(x)` である。
        BY <1>21, <1>23, <4>2
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

<1>26. `<1>21` と同じ本体と表 `vars` を取る。`origin_inner(vars, E, x, pi)` の 1 回の実行が行う
   `origin` の呼び出しは有限個である。
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
     `Set` である。`Provenance` は `LeafMap<LeafOrigins>` を包む値、`LeafMap<T>` は
     `Map<FieldPath, T>` を包む値、`LeafOrigins` は `Set<LeafOrigin>` であり、`Map` と `Set` は
     どちらも有限の写像・集合である。よって `operand_units` は有限である。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/provenance.rs: Provenance,
       CODE src/rc_ir/provenance.rs: LeafOrigins,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under,
       CODE src/misc.rs: Map, CODE src/misc.rs: Set
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>27a. `Origin` 型の任意の値 `o` について、`o.acted_on()` は空でない列を返す。
  `acted_on` は `let mut out = vec![self.identity()];` で始め、そののち `out` を伸ばすだけなので、
  返り値は `o.identity()` を必ず含む。
  BY CODE src/rc_ir/ownership.rs: Origin::acted_on

<1>27b. 次の 4 つの関数 -- `Provenance::build_shape`、`Provenance::uniform`、
   `Provenance::uniform_bottom`、`Provenance::fresh_under` -- を `<1>1` を満たす型 `ty` に対して
   呼ぶとき、abort しうるのは `build_shape` に渡された閉包の中だけである。その閉包は
   `boxed_leaf_paths(ty, E)` が返す列の各要素についてちょうど 1 度、その要素を `path` として
   呼ばれる。とくに受け取る `path` は `L(ty)` の要素であり、**`L(ty)` が空ならば閉包は 1 度も
   呼ばれない**。閉包を引数に取らない残りの 3 つは abort せず停止する。

   **この 4 つだけを主語に取るのは、`result_prov` の呼び出しが `Provenance` を作るのにこの 4 つしか
   使わないからである。**それを与えるのは、既定の実装と 29 個の override の本体を 1 つずつ開く
   `<1>28` の数え上げである。
  <2>1. `Provenance::build_shape(ty, E, leaf)` は `LeafMap::build_shape(ty, E, leaf)` を呼ぶ。
     `LeafMap::build_shape` は `boxed_leaf_paths(ty, E)` を 1 度呼び、返った列に
     `into_iter().map(|path| { let fact = leaf(&path); (path, fact) })` を当てて `Map` に集める。
     すなわち閉包はその列の各要素についてちょうど 1 度、その要素を引数に呼ばれ、それ以外の path に
     ついては呼ばれない。`<1>9` より `boxed_leaf_paths(ty)` は abort せず停止し、`L(ty)` は有限で
     ある。よって `build_shape` 自身は abort せず停止し、abort しうるのは閉包 `leaf` の中だけで
     ある。閉包が受け取る `path` は `L(ty)` の要素であり、`L(ty)` が空ならば閉包は 1 度も
     呼ばれない。
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
    BY <ref id=e11772a/>, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov
  <2>1a. `origin_inner` の `Llvm` の腕が `result_prov` に渡す `result_ty` と `arg_tys` は、
     `Let(x, RcRhs::Llvm(llvm_gen, args), k)` の `ty(x)` と `args` の各要素の型である。
     `<1>21` と `<1>22` が `vars.bindings` への挿入をすべて挙げており、そのうち `Binding::Llvm` を
     置くのは `collect_bindings` の `RcExpr::Let(x, RcRhs::Llvm(llvm_gen, args), k)` の腕だけで
     あって、そこが `result_ty` に置くのは `x.ty` である。腕の中で `arg_tys` を作るのは
     `args.iter().map(|a| a.ty.clone())` の 1 行である。
    BY <1>21, <1>22, CODE src/rc_ir/ownership.rs: collect_bindings,
       CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding
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
       `<1>3a` (vii) はどちらも偽だと述べる。よって `path` は空でなく、
       `EXT スライスの split_first` より `arg_leaf_path` の `split_first` は `Some` を返すので、
       その `expect` は発火しない。
      BY <1>1, <1>3a, <1>14, <1>27b, DEF cls, EXT スライスの split_first,
         CODE src/fixstd/builtin.rs: InlineLLVMStructPunchBody::arg_leaf_path
    <3>3a. `cls(result_ty)` は `UN` か `ST` である。`<1>3a` (vii) より `result_ty.is_box(E)`、
       `result_ty.is_array()`、`result_ty.is_closure()` はいずれも偽なので、`DEF cls` の `CL`、`BX`、
       `AR` の 3 行はどれも当たらない。`cls(result_ty) = NB` の場合は、
       `DEF UNST-道` の条件を `j = 0` に読むと `result_ty` の UNST-道は `[]` だけであり、
       `cls(end(result_ty, [])) = NB` は `<1>14` の 2 つの集合のどちらの条件にも当たらないので
       `L(result_ty)` は空であり、`<1>27b` より `build_shape` の閉包は 1 度も呼ばれない。よって
       以下は `cls(result_ty)` が `NB` でない場合を見ればよい。残るのは `UN` と `ST` である。
      BY <1>1, <1>3a, <1>14, <1>27b, DEF cls, DEF UNST-道
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
    <3>4. `s` を `<3>1` が取り出す型 `result_ty.field_types(E)[PUNCHED_STRUCT_FIELD]` とすると、
       `L(s)` の要素で `self.field_idx` で始まるものは無い。
      <4>1. `s` は `<1>1` を満たし、`s.is_closure()` は偽であり、
         `s.toplevel_tycon_info(E).fields[self.field_idx]` の `is_punched` は真である。`<1>28` の
         前置きより `result_ty` はこの `Llvm` 節点の `ty(x)` なので、`s` は `<1>3a` (vii) の
         `pt` である。
        BY <1>3a
      <4>2. `cls(s)` は `CL` ではない。`DEF cls` の `CL` の行は `s.is_closure()` が真であることを
         要求し、`<4>1` はそれが偽だと述べる。
        BY <4>1, DEF cls
      <4>3. 長さ 1 以上の `s` の UNST-道が在れば、`cls(s)` は `UN` か `ST` である。`DEF UNST-道` の
         条件を `j = 0` に読むとそれである。
        BY DEF UNST-道
      <4>4. CASE `cls(s) = NB`。`<4>1` より `<1>14` を `s` に当てられる。`<4>3` より `s` の UNST-道は
         `[]` だけであり、`end(s, []) = s` のクラスは `NB` なので、`<1>14` の第 1 の集合の条件
         (`BX` か `AR`) にも第 2 の集合の条件 (`CL`) にも当たらない。よって `L(s)` は空であり、
         主張は空虚に成り立つ。
        BY <1>14, <4>1, <4>3, DEF UNST-道
      <4>5. CASE `cls(s)` が `BX` か `AR`。`<4>3` より `s` の UNST-道は `[]` だけであり、
         `end(s, []) = s` のクラスはこの場合の仮定より `BX` か `AR` なので、`<1>14` の第 1 の集合が
         `[]` を、第 2 の集合が何も与えない。よって `L(s)` は `{[]}` であり、空の path はどの添字でも
         始まらない。
        BY <1>14, <4>1, <4>3, DEF UNST-道
      <4>6. CASE `cls(s)` が `UN` か `ST`。`<1>11` より `go(s, [], out)` が積むのは `F(s)` の各
         `(i, f)` についての `go(f, [i], out)` が積むものの合併であり、`<1>1` (ii) より `f` も
         `<1>1` を満たすので、`<1>14a` よりそれは `{ [i] ++ r : r は L(f) の要素 }` である。よって
         `L(s)` の空でない要素はどれも `F(s)` のある添字で始まる。この場合の仮定と `DEF cls` より
         `s.is_box(E)` は偽であり、`<4>1` より `s.is_closure()` も偽なので、`<1>3c` (d) を `s` に
         当てられる。それより `F(s)` の添字は、`s.toplevel_tycon_info(E).fields` の同じ添字の成分の
         `is_punched` が偽であるものに限る。`<4>1` より第 `self.field_idx` 成分の `is_punched` は
         真なので、`self.field_idx` は `F(s)` の添字ではない。`L(s)` の空の要素はどの添字でも
         始まらない。
        BY <1>1, <1>3c, <1>11, <1>14a, <4>1, DEF cls
      <4>7. QED
        `<1>4` より `cls(s)` は 6 つのどれかであり、`<4>2` が `CL` を退け、`<4>4`、`<4>5`、`<4>6` が
        残る `NB`、`BX`、`AR`、`UN`、`ST` を尽くしている。
        BY <1>4, <4>2, <4>4, <4>5, <4>6
    <3>4a. `arg_leaf_path` の `assert_ne!` は発火しない。`assert_ne!` が発火するのは、`path` が
       `[PUNCHED_STRUCT_FIELD, self.field_idx] ++ ・` の形のときだけである。`<3>3b` より、そのような
       `path` が `L(result_ty)` に在るのは `(PUNCHED_STRUCT_FIELD, s')` が `F(result_ty)` の要素で
       あって `L(s')` が `self.field_idx` で始まる要素を持つときに限る。`<1>3a` (vii) より
       `result_ty.is_closure()` も `result_ty.is_box(E)` も偽なので、
       `<1>3c` (d) を `result_ty` に当てられる。それよりその `s'` は
       `result_ty.field_types(E)[PUNCHED_STRUCT_FIELD]`、すなわち `<3>4` の `s` である。`<3>4` より
       そのような要素は無い。
      BY <1>3a, <1>3c, <3>1, <3>3b, <3>4,
         CODE src/fixstd/builtin.rs: InlineLLVMStructPunchBody::arg_leaf_path
    <3>5. QED
      閉包の残りは `Vec` の連結と `sole_origin` だけである。
      BY <1>1, <1>27b, <3>1, <3>2, <3>3, <3>3a, <3>3b, <3>4, <3>4a,
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
       `EXT スライスの split_first` より `split_first` は `Some` を返すので、その `expect` は
       発火しない。
      BY <1>1, <1>3a, <1>14, <1>27b, DEF cls, EXT スライスの split_first
    <3>3. QED
      閉包が行うのは `split_first` の `expect` と `Vec` の複製と `sole_origin` だけである。
      BY <1>1, <1>27b, <3>1, <3>2, CODE src/fixstd/builtin.rs: replaced_field_prov,
         CODE src/fixstd/builtin.rs: InlineLLVMStructSetBody::result_prov,
         CODE src/fixstd/builtin.rs: InlineLLVMStructPlugInBody::result_prov
  <2>3. QED
    `<2>1` より場合は既定の実装と 29 個の override で尽きている。`<2>2` が既定を、`<2>2a` から
    `<2>2g` が 16 + 1 + 5 + 2 + 2 + 1 + 2 = 29 個を尽くしている。**`<1>27b` を 4 つの関数に
    ついてだけ述べて足りるのは、この数え上げによる** -- `<2>2` から `<2>2g` はどれも本体を 1 つずつ
    読んでおり、そこで `Provenance` を作るのに使われるのは `build_shape`、`uniform`、
    `uniform_bottom`、`fresh_under` の 4 つだけである。
    BY <1>27b, <2>1, <2>1a, <2>2, <2>2a, <2>2b, <2>2c, <2>2d, <2>2e, <2>2f, <2>2g

<1>28a. `origin_inner` の `Llvm` の腕が得る `decl` は、各 leaf に要素数 0 か 1 の `LeafOrigins` を
   置く。したがって `decl.leaf_origins_at(path)` が返す集合と `decl.leaf_origins_under(path)` が
   渡す集合のどれかに `LeafOrigin::Arg(j, leaf)` が現れるとき、その leaf についての宣言は**単一の**
   `Arg(j, leaf)` であり、A3 の表の「単一の `Arg(j, σ)`」の行が当たる。すなわち `j` は `args.len()`
   未満であり、`leaf` は `L(ty(args[j]))` の要素である。
  <2>1. A3 は「複数の元を宣言する op は存在しない」と述べ、その根拠を数え上げで与える --
     `impl LLVMGen for` は 78 個、`result_prov` を override するのは 29 個、その 29 個が leaf に置く
     集合はすべて要素数 0 か 1 である。
    BY <ref id=e11772a/>
  <2>2. 残る 49 個が取る既定の実装は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)`
     であり、`uniform` は各 leaf に `sole_origin(LeafOrigin::Unknown)`、すなわち 1 要素の集合を
     置く。
    BY <2>1, CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
       CODE src/rc_ir/provenance.rs: Provenance::uniform,
       CODE src/rc_ir/provenance.rs: sole_origin
  <2>3. QED
    `<2>1` と `<2>2` より、どの op の `decl` も leaf ごとに要素数 0 か 1 の集合を置く。要素数が 1 で
    その元が `Arg(j, leaf)` である leaf は、A3 の表の「単一の `Arg(j, σ)`」の行が扱うものである。
    その `j` が `args` の添字であり (すなわち `args.len()` 未満であり)、`leaf` が
    `L(ty(args[j]))` の要素であることは `<1>3a` (viii) が述べる。
    BY <ref id=e11772a/>, <1>3a, <2>1, <2>2

<1>29. `<1>21` と同じ本体と表 `vars` を取る。`origin(vars, E, x, pi)` の**呼び出しの木** -- 根を
   その呼び出しとし、各節点の子をその実行が行う `origin` の呼び出しとする木 -- は有限である。
   したがって `origin` は停止する。
  <2>1. `origin` の本体は、`vars.origins` の中に `(x, pi)` の答えがあればそれを複製して返し、無ければ
     `origin_inner` を `grow_stack` の中で呼び、その答えを `vars.origins` に記録して返す。
     `grow_stack(f)` が `f` をちょうど 1 回呼びその返り値を返すことは A15 である -- `grow_stack` の
     本体は `stacker::maybe_grow` への 1 行の委譲であり、閉包が何回呼ばれるかを決めるのは
     `stacker` crate である。
    BY <ref id=3e6b0e0/>, CODE src/rc_ir/ownership.rs: origin, CODE src/misc.rs: grow_stack
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
  <2>6. `origin` の 1 回の実行が行う、`origin` の呼び出し以外の計算は有限時間で終わる。行うのは
     次のもので尽きている。
     - `origin` の中: 鍵 `(var.clone(), path.to_vec())` の構成、`vars.origins` (`Map`) の 1 回の
       探索と 1 回の挿入、`Origin` の複製、`grow_stack` の 1 段 (A15)。
     - `origin_inner` のどの腕にも共通するもの: `vars.bindings` (`Map`) の 1 回の探索、`here()` が
       作る `Origin::Exactly` の構成、`Vec` の複製・連結・添字付け。
     - `Binding::Field` の腕の `container.ty.is_box(E)` と `Binding::Payload` の腕の
       `scrut.ty.is_box(E)`。`is_box` は `is_unbox` の否定、`is_unbox` は
       `is_closure() || toplevel_tycon_info(E).is_unbox` であり、`is_closure` は
       `toplevel_tycon_satisfies` を経て `toplevel_tycon()` を 1 回呼んで名前を 1 回比べるだけ、
       `toplevel_tycon_info` は `toplevel_tycon()` を 1 回呼んで `E.tycons()` を 1 回引くだけで
       ある。`toplevel_tycon()` が再帰する先は `Type::TyApp` の関数側だけであり、それは `<1>1a` の
       直接の部分の辺なので、`<1>1a` より停止する。
     - `Binding::Join` の腕の `origin(...).acted_on()` と `candidates` (`Set`) への挿入。
       `acted_on` は `identity()` と `candidates()` を呼んで有限の `Vec` を作り、`candidates()` は
       `Origin::Join` が持つ有限の `Set` を 1 回走査する。`arm_results` は有限である (`<1>26`)。
     - `Origin::of_candidates`。`assert!`、`candidates.len()`、および要素数 1 のときの
       `into_iter().next()` か `identity` の複製だけを行う。
     - `Binding::Llvm` の腕の `result_prov` の呼び出し (`<1>28` より abort せず `Provenance` を
       返す)、`decl.leaf_origins_at(path)` (`LeafMap::get`、`Map` の 1 回の探索)、
       `as_arg_projection` (`Set` の大きさを見て、高々 1 要素を複製する)。
     - `origin_from_leaves_under` の中: `decl.leaf_origins_under(path)` (`LeafMap::leaves_under`、
       `Provenance` が包む有限の `Map` の 1 回の走査)、渡される各 `LeafOrigins` (有限の `Set`) の
       走査、`truncate_to_unit` の呼び出し (`<1>1` と `<1>9` より停止する)、`operand_units` (`Set`)
       への挿入、`reached` (有限の `Vec`) の走査と `Origin` どうしの等価比較、各 `Origin` の
       `acted_on()` の呼び出し、`candidates` (`Set`) の構成、`Origin::of_candidates`。
    BY <ref id=3e6b0e0/>, <1>1, <1>1a, <1>9, <1>26, <1>28,
       CODE src/rc_ir/ownership.rs: origin, origin_inner, origin_from_leaves_under,
          as_arg_projection, Origin::acted_on, Origin::candidates, Origin::of_candidates,
       CODE src/ast/types.rs: TypeNode::is_box, TypeNode::is_unbox, TypeNode::is_closure,
          TypeNode::toplevel_tycon_satisfies, TypeNode::toplevel_tycon,
          TypeNode::toplevel_tycon_info,
       CODE src/rc_ir/provenance.rs: Provenance, LeafOrigins, Provenance::leaf_origins_at,
          Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap, LeafMap::get, LeafMap::leaves_under
  <2>7. QED
    BY <2>5, <2>6

<1>29a. A6 と A11 を満たすプログラムの本体 -- 関数の `body` またはグローバル初期化子の `init` -- を
   1 つ取り、`vars` をそれについて `VarTable::of` または `VarTable::body_only` が作る表とする。型環境
   `E` も 1 つ固定する。このとき `origin(vars, E, u, sig)` が値を返すならば、その値は `(u, sig)` で
   決まる。すなわち、`vars.origins` の状態がどうであれ、同じ `(u, sig)` についての 2 回の呼び出しは
   等しい値を返す。

   **`vars` をこの 2 つの構成子が作る表に限るのは、証明が `<1>25` の `ord` の減少と
   `<1>21` の挿入の有限列に立つからである。**その 2 つは `VarTable::of` と `VarTable::body_only` が
   A6 と A11 を満たす本体について作った表でしか成り立たない。**この制限は言明の一部であって、
   読む段が自分で補うものではない。**
  <2>1. `origin` の実行は、`vars.bindings` と `vars.var_tys` について次の 3 つを満たす。
     - (a) 2 つの `Map` の鍵の集合は変わらず、各鍵が対応づける `Binding` と `Arc<TypeNode>` も
       別の値に置き替えられない。
     - (b) `origin_inner` が `vars.bindings` から**値として**読むもの -- `Binding` の変位、
       `Binding` が持つ各 `RcVar` の `name` と `ty`、`Binding::Llvm` の `result_ty`、
       `Binding::Field` の `idx`、`Binding::Payload` の `variant` -- は、表を作り終えたのちは
       変わらない。`origin_inner` は `vars.bindings.get(var)` のどの腕でも、この一覧のものだけを
       読む -- `None`・`Param`・`Producer` の腕は `var` と `path` から `Origin::Exactly` を作るだけ、
       `Move(y)` の腕は `y.name`、`Join(arm_results)` の腕は各 `arm_result.name`、
       `Llvm(llvm_gen, args, result_ty)` の腕は `result_ty` と `args` の各 `ty`・`name` と
       `llvm_gen` ((c) が扱う)、`Field(container, idx)` の腕は `container.ty`・`container.name` と
       `idx`、`Payload(scrut, variant)` の腕は `variant`・`scrut.ty`・`scrut.name` である。
     - (c) `Binding::Llvm` が持つ `Box<dyn LLVMGen>` の op について、`origin_inner` がそこから読む
       のは `result_prov(result_ty, &arg_tys, E)` の返り値だけであり、その返り値は引数だけで決まる。

     **(c) を (b) と分けるのは、op が値として読まれないからである。**README の A3 は
     「`Box<dyn LLVMGen>` の op はその値の中に在るので、欄への書き込みを数え上げるだけでは
     足りない」と述べる。op が持ちうる内部可変性の欄を数え上げる代わりに、(c) は op が答えるものの
     側を A3 の決定性の節で閉じる。
    <3>1. `bindings` は `VarTable` の非公開フィールドなので、`EXT Rust の可視性` よりそれを名前で
       参照できるのは、それを宣言するモジュール `crate::rc_ir::ownership` とその子孫だけである。
       そのモジュールとその子孫はすべて `src/rc_ir/ownership.rs` の中に在る --
       `EXT Rust のモジュールの木` より子孫を作るのは `mod` の項目であり、同ファイルが持つ `mod` の項目は
       `#[cfg(test)] mod tests` だけで、それは本体を同ファイルの中に置く。同ファイルで
       `bindings` に書くのは、
       `VarTable::empty` の初期化と、`VarTable::of` の 1 か所と、`collect_bindings` の 3 か所で
       ある (残る 1 か所は `#[cfg(test)] mod tests` の中の `table` である)。`VarTable::body_only` は
       `VarTable::empty` と `collect_bindings` を呼ぶだけである。どれも表を作る間にしか走らない。
      BY EXT Rust の可視性, EXT Rust のモジュールの木,
         CODE src/rc_ir/ownership.rs: VarTable (`bindings` の宣言),
         CODE src/rc_ir/ownership.rs: VarTable::empty, CODE src/rc_ir/ownership.rs: VarTable::of,
         CODE src/rc_ir/ownership.rs: VarTable::body_only,
         CODE src/rc_ir/ownership.rs: collect_bindings
    <3>2. `var_tys` は `pub(crate)` なので、`EXT Rust の可視性` よりそれを名前で参照できるのは
       このクレートの中だけであり、数え上げる範囲は `src/` である。`src/` のうち
       `src/rc_ir/ownership.rs` の外に `var_tys` の出現は 1 つも無く、同ファイルの中で `var_tys` に
       書くのは `VarTable::empty` の初期化と、`VarTable::of` の 1 か所と、`collect_bindings` の
       3 か所である (残る 1 か所は `#[cfg(test)] mod tests` の中の `table` である)。どれも表を作る
       間にしか走らない。
      BY EXT Rust の可視性, CODE src/rc_ir/ownership.rs: VarTable (`var_tys` の宣言),
         CODE src/rc_ir/ownership.rs: VarTable::empty, CODE src/rc_ir/ownership.rs: VarTable::of,
         CODE src/rc_ir/ownership.rs: collect_bindings
    <3>3. `origin` は `&VarTable` を取るので、`EXT Rust の内部可変性` より `bindings` と `var_tys` の
       `Map` そのものを置き替えることも、その要素を可変に借りることもできない。共有参照から届くのは
       内部可変性を持つ欄だけであり、`origins` の `RefCell` がその 1 つである。

       **`bindings` と `var_tys` が持つ `Arc<TypeNode>` も内部可変性に届く。**`TypeNode` は
       `hash_cache`・`ground_cache`・`depth_cache` という `OnceLock` の欄を 3 つ持ち、
       `impl Hash for TypeNode` は `type_hash` を呼んで `hash_cache.get_or_init` を実行する。
       **`origin` の歩みはその道を実際に通る** -- `origin_inner` の `Llvm` の腕は
       `origin_from_leaves_under` を呼び、そこが `truncate_to_unit` を呼び、`truncate_to_unit` は
       `unit_step` を経て `unpunched_field_types` を呼ぶ。それが呼ぶ
       `instance_field_types` は、その宣言が kind `*` でない型変数を持つとき各フィールドの型に
       `unwrap_newtypes_memoized` を当てる。`unwrap_newtypes_memoized` は
       `Map<Arc<TypeNode>, Arc<TypeNode>>` を `Arc<TypeNode>` の鍵で引き、`Map` は `FxHashMap` なので
       その鍵をハッシュする。すなわち共有参照から `hash_cache` が書かれる。

       **A3 の値の等しさの節がこれを片付ける** -- 「**`RcProgram` から到達できる値の等しさは、それを
       共有参照で受け取る計算が変えない。** 到達できる型が内部可変性を持つ欄を持つときは、その欄は
       **一度だけ書かれる memo であって、その値はその型の `PartialEq` が読む成分の関数である**。」
       `impl PartialEq for TypeNode` が読むのは `ty` だけであり、3 つの memo の値はどれも `ty` の
       関数である。よってその欄が埋まっても、(b) が挙げるものは値として変わらない。
      BY <ref id=e11772a/> (`RcProgram` から到達できる値の等しさは、それを共有参照で受け取る計算が変えない),
         EXT Rust の内部可変性, <3>1, <3>2,
         CODE src/rc_ir/ownership.rs: VarTable,
         CODE src/rc_ir/ownership.rs: origin, origin_inner, origin_from_leaves_under,
            truncate_to_unit, unit_step,
         CODE src/ast/types.rs: TypeNode (`hash_cache` / `ground_cache` / `depth_cache` の宣言と
            `PartialEq` の実装), TypeNode::type_hash, TypeNode::unpunched_field_types,
            TypeNode::instance_field_types, TypeNode::unwrap_newtypes_memoized,
         CODE src/ast/types.rs: impl Hash for TypeNode,
         CODE src/ast/types.rs: impl PartialEq for TypeNode,
         CODE src/misc.rs: Map
    <3>3a. (c) が成り立つ。`origin_inner` の `Binding::Llvm` の腕は、`args` の各要素の `ty` を
       集めて `arg_tys` を作り、`llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を 1 度呼んで
       `decl` を得たのち、`decl`、`args`、`path`、`var` を読む。この腕にも `origin_inner` の
       ほかの腕にも、`llvm_gen` を読む式はこの呼び出し以外に無い。A3 の
       「**`result_prov` と `borrows_operand` は決定的である** -- 同じ引数に対して常に同じ値を
       返す」より、その返り値は `result_ty`、`arg_tys`、`E` だけで決まる。**op が自分の中に持つ
       内部可変性は、この段に入らない** -- `LLVMGen::result_prov` は `&self` を取るので op はそれを
       持ちうるが、決定性の節はその有無に依らず答えを引数の関数にする。
      BY <ref id=e11772a/> (`result_prov` と `borrows_operand` は決定的である),
         CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding,
         CODE src/ast/inline_llvm.rs: LLVMGen::result_prov
    <3>4. QED
      (a) は `<3>1` と `<3>2`、および `<3>3` の第 1 段落 --「`origin` は `&VarTable` を取るので、
      `EXT Rust の内部可変性` より `bindings` と `var_tys` の `Map` そのものを置き替えることも、
      その要素を可変に借りることもできない」-- である。(b) は `<3>3`、(c) は `<3>3a` である。
      BY <3>1, <3>2, <3>3, <3>3a
  <2>1a. `origin_from_leaves_under(vars, E, decl, args, path, here)` の返り値は、`decl`、`args`、
     `path`、`here`、`E`、および自分が行う `origin` の呼び出しの返り値だけで決まる。とくに
     `operand_units` の反復の順序には依らない。
    <3>1. `operand_units` は `Set<(usize, FieldPath)>` であり、その要素は
       `decl.leaf_origins_under(path)` が渡す各 `LeafOrigins` の各 `LeafOrigin::Arg(j, leaf)` に
       ついての `(j, truncate_to_unit(&args[j].ty, leaf, E))` である。`leaf_origins_under` が渡す
       `LeafOrigins` の族は `decl` と `path` で決まり、`args[j].ty` は `args` の欄であり、
       `<1>9a` より `truncate_to_unit` の返り値はその 3 つの引数の値だけで決まる。よって集合としての
       `operand_units` は `decl`、`args`、`path`、`E` で決まる。`produced_here` は同じ族に
       `LeafOrigin::Fresh` か `LeafOrigin::Unknown` が現れるかどうかなので、同じ 4 つで決まる。
      BY <1>9a, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: truncate_to_unit,
         CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
    <3>2. `reached` は、`operand_units` の各 `(j, unit)` についての
       `origin(vars, E, &args[j].name, &unit)` の返り値を並べ、`produced_here` が真ならさらに
       `Origin::Exactly(here.clone())` を末尾に置いた `Vec` である。反復の順序が変わって変わるのは、
       この `Vec` の要素の並び順だけである。
      BY <3>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>3. `Origin` の `==` は値としての等価性であり、同値関係である。`Origin` の宣言は
       `#[derive(Clone, Debug, PartialEq, Eq)]` を持つので、`EXT derive した PartialEq と Eq` より
       `==` は変位が一致することと、`Exactly` なら `VarPath` (= `(FullName, FieldPath)`) が等しい
       こと、`Join` なら `identity` が等しくかつ `candidates` (`Set<VarPath>`) が等しいことである。
       `Set<T>` は `FxHashSet<T>` であり、その等価性は `EXT HashSet の等価性` より集合としての
       等価性である。`Origin` は `Eq` を導出するので、同じ `EXT` より `==` は同値関係である。
      BY EXT HashSet の等価性, EXT derive した PartialEq と Eq,
         CODE src/rc_ir/ownership.rs: Origin,
         CODE src/rc_ir/ast.rs: VarPath, CODE src/misc.rs: Set
    <3>4. CASE `reached` が空。`reached.first()?` が `None` を返すので、
       `origin_from_leaves_under` は `None` を返す。空であるかどうかは並び順に依らない。
      BY <3>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>5. CASE `reached` が空でなく、その要素がすべて互いに等しい。どの並び順でも `first` はその
       共通の値であり、`reached.iter().all(|o| o == first)` は真になるので、返り値は
       `Some(その共通の値)` である。
      BY <3>2, <3>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>6. CASE `reached` が空でなく、互いに等しくない 2 つの要素 `a`、`b` を持つ。どの並び順でも
       `reached.iter().all(|o| o == first)` は偽である -- 真だとすると `a == first` かつ
       `b == first` であり、`<3>3` の同値関係から `a == b` になって仮定に反する。よって返り値は
       `Some(Origin::of_candidates(candidates, here))` である。`candidates` は `reached` の各要素の
       `acted_on()` を集めた `Set<VarPath>` であり、集合としては並び順に依らない。`of_candidates` は
       `candidates.is_empty()` を表明で見たのち `candidates.len()` で場合を分け、1 のときは
       `into_iter().next()` が返す元を `Origin::Exactly` に置き、それ以外のときは `candidates`
       そのものを `Origin::Join` の同名の欄に、`here` の複製を `identity` の欄に置く。要素数が 1 の
       集合の `into_iter().next()` はその元であり、要素数 2 以上のときに置かれる `Set` は並び順に
       依らず等しいので、`<3>3` の `Origin` の等価性のもとで返り値はどちらの場合も並び順に依らない。
       要素数 1 の場合に `into_iter().next()` がその 1 つの要素を返すことは
       `EXT 1 要素の集合の反復` が言う。
      BY <3>2, <3>3, EXT 1 要素の集合の反復,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::acted_on
    <3>7. QED
      `<3>4` から `<3>6` は `reached` についての 3 つの場合を尽くしている。
      BY <3>4, <3>5, <3>6
  <2>1b. `origin_inner` の `Binding::Join(arm_results)` の腕の返り値は、`arm_results`、`var`、
     `path`、および自分が行う `origin` の呼び出しの返り値だけで決まる。とくに `candidates` の反復の
     順序には依らない。
    <3>1. この腕は `arm_results` の各要素について `origin(vars, E, &arm_result.name, path)` を呼び、
       返った `Origin` の `acted_on()` の各元を `candidates` (`Set<VarPath>`) に入れ、
       `Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))` を返す。`acted_on()` は
       `identity()` を先頭に置き、それと異なる `candidates()` の元を続けた列であり、その元の集合は
       その `Origin` の値で決まる。`Set` への挿入がなす集合は挿入の順序に依らないので、`candidates` は
       `arm_results` と各子の返り値で決まる。
      BY CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: Origin::acted_on,
         CODE src/rc_ir/ownership.rs: Origin::candidates, CODE src/misc.rs: Set
    <3>2. `of_candidates` は `candidates.len()` で場合を分け、1 のときは `into_iter().next()` が返す
       元を `Origin::Exactly` に置き、それ以外のときは `candidates` そのものを `Origin::Join` の
       同名の欄に、`here` の複製を `identity` の欄に置く。`EXT 1 要素の集合の反復` より、要素数 1 の
       集合の `into_iter().next()` はその 1 つの元である。`EXT HashSet の等価性` より `Set` の等価性は
       集合としての等価性なので、要素数 2 以上のときに置かれる `Set` も反復の順序に依らず等しい。
      BY EXT 1 要素の集合の反復, EXT HashSet の等価性,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, CODE src/misc.rs: Set
    <3>3. QED
      BY <3>1, <3>2
  <2>2. `origin_inner(vars, E, u, sig)` の返り値は、`vars.bindings`、`E`、`u`、`sig`、および自分が
     行う `origin` の呼び出しの返り値だけで決まる。この関数が `vars` から読むのは
     `vars.bindings.get(var)` だけであり、その `Binding` から値として読むものは `<2>1` (b) が
     挙げる。そのほかに読むのは `E` であり、`<1>9a` より `truncate_to_unit` の返り値はその引数の
     値だけで決まる。`vars.origins` も `vars.var_tys` も読まない。
     このうち `llvm_gen` から読まれるのは `result_prov(result_ty, &arg_tys, E)` の返り値だけで
     あり、それが引数だけで決まることは `<2>1` (c) が与える。**その節が要るのは、`decl` が変われば
     `origin_inner` の `Llvm` の腕が `decl.leaf_origins_at(path).and_then(as_arg_projection)` の
     結果で別の道を選びうるからである。**残るものが表を作り終えたのちは変わらないことは `<2>1` (b)
     が与える。`Llvm` の腕が `origin_from_leaves_under` を呼ぶ道については `<2>1a` が、
     `Binding::Join` の腕については `<2>1b` がこれを与える。どちらも `Set` を組んでから
     `Origin::of_candidates` に渡すので、反復の順序に依らないことを別に言う要がある。
    BY <1>9a, <2>1, <2>1a, <2>1b,
       CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/ast/inline_llvm.rs: LLVMGen::result_prov
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

<1>29b. `<1>21` と同じ本体と表 `vars` を取り、型環境 `E` を 1 つ固定する。
   `origin_inner(vars, E, u, sig)` の実行が行う `origin` の呼び出しの引数の族は、`vars`、`E`、`u`、
   `sig` の値だけで決まり、`vars.origins` の状態に依らない。すなわち `DEF 呼び出しの辺` が定める
   関係は、`vars` と `E` を固定すれば 1 つに定まる。
  <2>1. `origin_inner` は `vars.origins` を読まない。この関数が `vars` から読むのは
     `vars.bindings.get(var)` だけである。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: VarTable
  <2>2. `origin_inner` が `origin` を呼ぶ腕はどれも、呼び先の対を先に返った `origin` の値から
     決めない。`Move` の腕は `origin(vars, E, &y.name, path)` を 1 つ、`Join` の腕は `arm_results`
     の各要素について `origin(vars, E, &arm_result.name, path)` を、`Field` の腕は
     `container.ty.is_box(E)` が偽のとき `origin(vars, E, &container.name, [idx] ++ path)` を、
     `Payload` の腕は `origin(vars, E, &scrut.name, path)` か
     `origin(vars, E, &scrut.name, [tag] ++ path)` を呼ぶ。`Join` の腕は返った `Origin` の
     `acted_on()` を `candidates` に積むだけで、それを次の呼び先に使わない。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding
  <2>3. `<2>2` が挙げた対は、`vars.bindings` が持つ `Binding` の欄 (`Move` の `y`、`Join` の
     `arm_results`、`Field` の `container` と `idx`、`Payload` の `scrut` と `variant`) と、`path`
     と、`container.ty` / `scrut.ty` と `E` から `is_box` を引いた真偽値だけで決まる。`is_box` の
     振る舞いが型の値と `E` の値だけで決まることは `<1>9a` が述べる。
    BY <1>9a, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/ast/types.rs: TypeNode::is_box
  <2>4. `Llvm` の腕の呼び先も同じ 4 つで決まる。この腕は `args` の各要素の `ty` から `arg_tys` を
     作り、`decl = llvm_gen.result_prov(result_ty, &arg_tys, E)` を 1 度呼ぶ。A3 の
     「**`result_prov` と `borrows_operand` は決定的である** -- 同じ引数に対して常に同じ値を返す」
     より `decl` は `result_ty`、`arg_tys`、`E` の値で決まる。次に
     `decl.leaf_origins_at(path).and_then(as_arg_projection)` で場合を分け、`Some((j, p))` なら
     `origin(vars, E, &args[j].name, &p)` を 1 つ呼び、`None` なら `origin_from_leaves_under` が
     `operand_units` の各 `(j, unit)` について `origin(vars, E, &args[j].name, &unit)` を呼ぶ。
     `leaf_origins_at` と `leaf_origins_under` は `decl` が包む `Map` を `path` で引くだけであり、
     `as_arg_projection` は渡された `Set` の大きさと元だけを見る。`unit` は
     `truncate_to_unit(&args[j].ty, leaf, E)` であり、`<1>9a` よりその値は引数の値だけで決まる。
    BY <ref id=e11772a/> (`result_prov` と `borrows_operand` は決定的である), <1>9a,
       CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: as_arg_projection,
       CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
  <2>5. QED
    `vars.bindings.get(var)` の結果は `Binding` の 7 つの値と `None` の 8 通りであり、`None`、
    `Param`、`Producer` の腕は `origin` を呼ばない。残る 5 つを `<2>2` と `<2>3` (`Move`、`Join`、
    `Field`、`Payload`) と `<2>4` (`Llvm`) が尽くしている。`<2>1` より `vars.origins` は読まれない
    ので、呼び出しの引数の族は `vars`、`E`、`u`、`sig` の値だけで決まる。`DEF 呼び出しの辺` は
    その族を辺の行き先とするので、関係も 1 つに定まる。
    BY <2>1, <2>2, <2>3, <2>4, DEF 呼び出しの辺,
       CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding

<1>30. `<1>21` と同じ本体と表 `vars` を取る。`origin(vars, E, x, pi)` は abort しない。
  <2>1. `origin` 自身の abort の可能性は `vars.origins` の `RefCell` の借用の衝突だけである。
    <3>1. `origin` は `vars.origins.borrow()` を
       `if let Some(known) = ... { return known.clone(); }` の走査対象で 1 回、
       `vars.origins.borrow_mut()` を最後の文で 1 回使う。`origin` の本体が行う残りは、鍵
       `(var.clone(), path.to_vec())` の構成、`Map` の 1 回の探索と 1 回の挿入、`Origin` の複製、
       および `grow_stack` の 1 段であり、そのどれも panic しない。`grow_stack` が呼ぶ
       `origin_inner` が abort しないことは、この段の主張に含まれない。
      BY <ref id=3e6b0e0/>, CODE src/rc_ir/ownership.rs: origin, CODE src/misc.rs: grow_stack,
         CODE src/misc.rs: Map
    <3>2. `if let` の走査対象が作る一時値は、その `if let` 文の終わりで落ちる。`origin_inner` を
       呼ぶのはその次の文なので、その `Ref` は既に落ちている。

       依拠するのは第 1 節の `EXT Rust の一時値のスコープ` である。その一覧のうち、
       edition 2021 で `if let` の走査対象を含む最小の場は「A statement」であり、ここではその
       `if let` 文自身である。edition 2024 は「The pattern-matching condition(s) and consequent
       body of `if` (2024 Edition)」をこの一覧に足すので、走査対象の一時値は `else` ブロックより
       前で落ちる。`Cargo.toml` の `[package]` は `edition = "2021"` を書いており、この `if let` は
       `else` を持たないので、どちらの規則でも落ちる点は同じ `if let` 文の終わりである。
      BY <3>1, EXT Rust の一時値のスコープ, CODE Cargo.toml: [package] の edition
    <3>3. `borrow_mut()` が作る一時値はその文の終わりで落ちる。その文の中で `origin` は呼ばれない
       (`answer.clone()` は既に得た `Origin` の複製である)。落ちる点を与えるのは `<3>2` と同じ
       `EXT Rust の一時値のスコープ` であり、その一覧のうちこの式を含む最小の場は「A statement」、
       すなわちその文自身である。
      BY <3>1, EXT Rust の一時値のスコープ
    <3>4. QED
      `<3>2` と `<3>3` より、`Ref` が生きている間に `borrow_mut()` は起きず、`RefMut` が生きている
      間に `borrow()` も `borrow_mut()` も起きない。`EXT RefCell の借用` より、借用が重ならなければ
      `borrow` も `borrow_mut` も panic しない。
      BY <3>2, <3>3, EXT RefCell の借用
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
     呼ぶ。`of_candidates` が abort しうるのは、`candidates` が空のときの `assert!` と、
     `candidates.len()` が 1 の腕が置く `into_iter().next().expect(..)` である。この `candidates`
     は空でなく、`expect` の側も abort しない。
    <3>1. `arm_results` は `collect_bindings` が `Let(x, RcRhs::Match(scrut, arms), k)` の腕で
       `arms` の各要素について 1 つずつ積んだものなので、その長さは `arms` の長さに等しい。
      BY CODE src/rc_ir/ownership.rs: collect_bindings
    <3>2. `<1>3` より `arms` は空でない。`<3>1` より `arm_results` も空でない。
      BY <1>3, <3>1
    <3>3. QED
      この腕は `arm_results` の各要素について `origin(...).acted_on()` の全要素を `candidates` に
      入れる。`<3>2` より要素は 1 つ以上あり、`<1>27a` よりその `acted_on()` は空でないので、
      `candidates` は空でなく `assert!` は発火しない。`candidates.len()` が 1 のときは、
      `EXT 1 要素の集合の反復` より `into_iter().next()` が `Some` を返すので `expect` も発火しない。
      BY <1>27a, <3>2, EXT 1 要素の集合の反復, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>7. `origin_inner` の `Llvm` の腕は abort しない。
    <3>1. この腕はまず `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を呼ぶ。`<1>28` より
       これは abort しない。そののち `decl.leaf_origins_at(path)` (`LeafMap::get` を経由し `Option`
       を返す) に `as_arg_projection` を `and_then` する。`as_arg_projection` は `sources.len()` が
       1 でなければ `None` を返し、1 のときは `sources.iter().next().expect(..)` で唯一の元を取り
       出してその変位で場合を分ける。`EXT 1 要素の集合の反復` より、要素をちょうど 1 つ持つ集合の
       `iter().next()` は `Some` を返すので、この `expect` は発火しない。ほかに abort する場所は
       持たない。ここで `path` は `origin` に渡された `pi` そのものであり、
       `leaf_origins_at` はそれを `Map` の鍵として引くだけなので、`pi` が `decl` の leaf でなくても
       `None` が返るだけである。
      BY <1>28, EXT 1 要素の集合の反復, CODE src/rc_ir/ownership.rs: origin_inner,
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
    <3>5. `ty(args[j])` は RC IR に現れる型なので `<1>1` を満たす。`<3>4` より `leaf` は
       `L(ty(args[j]))` の要素なので、`<1>17` (iii) を `t := ty(args[j])`、`lam := leaf` に適用でき、
       `truncate_to_unit(&args[j].ty, leaf, E)` は abort しない。とくに `UnitStep::NoUnit` の腕の
       `panic!` (「holds no reference」) には達しない。
      BY <1>1, <1>17, <3>4
    <3>6. `origin_from_leaves_under` の残りは、`Set` への挿入、`origin` の呼び出し、
       `reached.first()?` (空なら `None` を返す)、`Origin` の等価比較、そして `reached` が空でない
       ときの `Origin::of_candidates(candidates, here)` である。`candidates` は各 `reached` の
       `acted_on()` を集めたものであり、`reached` が空でなく、`<1>27a` より各 `Origin` の
       `acted_on()` が空でないので、`candidates` は空でなく `assert!` は発火しない。
       `of_candidates` の残る abort の場所は `candidates.len()` が 1 の腕が置く
       `into_iter().next().expect(..)` であり、`EXT 1 要素の集合の反復` よりそこも発火しない。
      BY <1>27a, EXT 1 要素の集合の反復, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
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

<1>31. **P2 が成り立つ。** すなわち、A3、A6、A9、A10、A11、A12、A15、A28 を満たすプログラムに
   ついて、`vars` をその 1 つの本体 -- 関数の `body` またはグローバル初期化子の `init` -- について
   `VarTable::of` または `VarTable::body_only` が作る表、`x` を任意の `FullName`、`pi` を任意の
   `FieldPath` とすると、`origin(vars, E, x, pi)` は panic せずに `Origin` の値を返し、停止する。

   **`VarTable::body_only` が作る表を範囲に入れるのは、`origin` がその上でも走るからである。**
   `borrow_ify` と `cancel` はどちらもグローバル初期化子について `VarTable::body_only(&g.init)` を
   置き、その表で `RewriteCtx` と `CancelAnalysis` を作る。README の P7a も、site を関数に限らない
   形で書く理由を「`owns_unit` がグローバル初期化子の版でも呼ばれるからである」と述べる。

   条件節がこの 8 つを挙げるのは、`<1>29` と `<1>30` がその全部の上に立つからである。`<1>1` は
   A10、`<1>2` は A11 を A6 と D6 と合わせたもの、`<1>3a` は A12 と A3 の 1 段をこの文書の記法で
   述べたものであり (第 3 節)、A3 の残りは `<1>28` と `<1>28a`、A6 は `<1>2` と `<1>21`、A9 は
   `<1>3`、A15 は `<1>29`、A28 は `<1>3ba` が読む。A28 が届く道は `<1>3ba` から `<1>3c` と
   `<1>3ca` を経て `<1>9` と `<1>10`、そして `<1>28` と `<1>30` である。

   P2 が量化するのは、`x` がプログラムの束縛変数である場合と、`x` が `vars.bindings` に束縛を
   持たない名前 (D6 の第 3 の形) である場合であり、どちらもこの主張の特別な場合である。
  <2>1. `<1>29` (停止性) と `<1>30` (abort しないこと) の言明はどちらも、`pi` に条件を置いて
     いない。P2 の「`π` を問わず」が要求するのはこれである。
    BY <1>29, <1>30
  <2>2. `pi` が型の上の walk に渡らないことは `<1>30` が場合ごとに述べている。`origin` と
     `origin_inner` は `pi` を、`Origin::Exactly` と `Origin::of_candidates` の成分として複製するか、
     前に添字を継ぎ足して再帰へ渡すか、`decl.leaf_origins_at` の鍵および `decl.leaf_origins_under` の
     絞り込みに使うかのいずれかにしかしない。`truncate_to_unit` に渡るのは宣言が名指す leaf だけで
     ある。
    BY <1>30, CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>2a. `<1>29` と `<1>30` の言明はどちらも、`x` に条件を置いていない。とくに `x` が
     `vars.bindings` に束縛を持たない名前 (D6 の第 3 の形) である場合も、その 2 つの言明の範囲に
     入る。その場合に `origin` が返す値は `Origin::Exactly((x, pi))` である -- `origin_inner` は
     `vars.bindings.get(var)` が `None` の腕で `here()` を返し、`origin` はその値を記録して返す。
    BY <ref id=596a46d/>, <1>29, <1>30, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin
  <2>2b. `<1>29` と `<1>30` の言明はどちらも、`vars` が `VarTable::of(func)` の作る表か
     `VarTable::body_only(body)` の作る表かを問わない。その 2 つの言明が `vars` に置く条件は、
     `<1>2` と `<1>21` が `vars` に置くもの -- 本体を 1 つ取り、その本体について 2 つの構成子の
     どちらかが作る表であること -- だけであり、どちらの構成子であるかで場合を分ける条件を
     持たないからである。`<1>2` と `<1>21` は言明そのものが 2 つの構成子を並べる。
    BY <1>2, <1>21, <1>29, <1>30
  <2>3. QED
    P2 が量化する 2 つの場合はどちらも `<2>2a` の範囲に入り、`pi` についての一般性は `<2>1` と
    `<2>2` が、表の 2 つの作り方についての一般性は `<2>2b` が与える。条件節の 7 つの仮定は `<1>29` と
    `<1>30` が読むものである。
    BY <ref id=e11772a/>, <ref id=33c54dc/>, <ref id=1172c08/>, <ref id=8412761/>, <ref id=3905b4e/>, <ref id=83d98e9/>, <ref id=3e6b0e0/>, <ref id=3d4be43/>, <1>29, <1>30, <2>1, <2>2, <2>2a, <2>2b,
       CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: cancel,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only

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

<1>33. **(P1 の系 1: `origin` が辿る path はどれも unit に届く)** `<1>31` と同じ 8 つの仮定 --
   A3、A6、A9、A10、A11、A12、A15、A28 -- を満たすプログラムの本体 -- 関数の `body` またはグローバル
   初期化子の `init` -- を 1 つ取り、`vars` をそれについて `VarTable::of` または
   `VarTable::body_only` が作る表とする。`x` を `ty(x)` が定まる名前、すなわちその本体に現れる
   `RcVar` の名前 (`<1>3a` (vi)) とする。このとき、`pi` が `L(ty(x))` の要素または `U(ty(x))` の
   要素であるならば、`(x, pi)` の
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
  <2>5. 呼び出しの下流の起点 `(x, pi)` について主張が成り立つ。`DEF 呼び出しの下流` は 0 回の場合を
     含むので `(x, pi)` 自身が下流に入る。`ty(x)` は `<1>1` を満たすので、
     `pi` が `L(ty(x))` の要素なら `<2>1` が、`U(ty(x))` の要素なら `<1>32a` が、`pi` が `ty(x)` の
     unit に届くことを与える。
    BY <1>1, <1>32a, <2>1, DEF 呼び出しの下流
  <2>5a. `u` が `vars.bindings` の定義域にあるならば、`u` は `vars.var_tys` に型を持ち、その型は
     `<1>3a` (vi) の `ty(u)` である。
    BY <1>21a
  <2>6. ASSUME: 対 `(u, sig)` は `(x, pi)` の呼び出しの下流にあり、`u` が `vars.var_tys` に型を
     持つならば `sig` は `ty(u)` の unit に届く
     PROVE: `(u, sig)` から呼び出しの辺で着く各対 `(u', sig')` についても、`u'` が `vars.var_tys` に
     型を持つならば `sig'` は `ty(u')` の unit に届く
    <3>1. CASE `vars.bindings[u]` が `Binding::Move(y)` である。辺の先は `(y, sig)` である。この
       場合の仮定より `u` は `vars.bindings` の定義域にあるので、`<2>5a` より帰納法の仮定の前件が
       満たされ、`sig` は `ty(u)` の unit に届く。`Binding::Move(y)` を作るのは `collect_bindings` の
       `Let(u, RcRhs::Var(y), k)` の腕だけなので、`<1>3a` (i) が当たり、(vi) と合わせて
       `ty(y) = ty(u)` である。よって `y` が `vars.var_tys` に型を持つならばその型は `ty(u)` であり、
       `sig` はその unit に届く。
      BY <1>3a, <2>5a, DEF 呼び出しの辺, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: collect_bindings
    <3>2. CASE `vars.bindings[u]` が `Binding::Join(arm_results)` である。辺の先は
       `(arm_result, sig)` である。この場合の仮定より `u` は `vars.bindings` の定義域にあるので、
       `<2>5a` より帰納法の仮定の前件が満たされ、`sig` は `ty(u)` の unit に届く。
       `Binding::Join(arm_results)` を作るのは `collect_bindings` の
       `Let(u, RcRhs::Match(scrut, arms), k)` の腕だけであり、`arm_results` の各要素は
       `returned_var(&arm.body)` なので、`<1>3a` (ii) が当たり、(vi) と合わせて
       `ty(arm_result) = ty(u)` である。よって `arm_result` が `vars.var_tys` に型を持つならば
       その型は `ty(u)` であり、`sig` はその unit に届く。
      BY <1>3a, <2>5a, DEF 呼び出しの辺, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: collect_bindings,
         CODE src/rc_ir/ownership.rs: returned_var
    <3>3. CASE `vars.bindings[u]` が `Binding::Payload(scrut, None)` である。辺の先は
       `(scrut, sig)` である。この場合の仮定より `u` は `vars.bindings` の定義域にあるので、
       `<2>5a` より帰納法の仮定の前件が満たされ、`sig` は `ty(u)` の unit に届く。
       `Binding::Payload(scrut, arm.tag)` を作るのは `collect_bindings` の
       `Let(z, RcRhs::Match(scrut, arms), k)` の腕だけであり、`u` はその `arm.payload.name` で
       `arm.tag` は `None` なので、`<1>3a` (iii) の catch-all の場合が当たり、(vi) と合わせて
       `ty(u) = ty(scrut)` である。よって `scrut` が `vars.var_tys` に型を持つならばその型は
       `ty(u)` であり、`sig` はその unit に届く。
      BY <1>3a, <2>5a, DEF 呼び出しの辺, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: collect_bindings
    <3>4. CASE `vars.bindings[u]` が `Binding::Payload(scrut, Some(tag))` で `scrut.ty.is_box(E)` が
       偽である。
      <4>1. 辺の先は `(scrut, [tag] ++ sig)` である。
        BY DEF 呼び出しの辺, CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. `Binding::Payload(scrut, arm.tag)` を作るのは `collect_bindings` の
         `Let(z, RcRhs::Match(scrut, arms), k)` の腕だけであり、`u` はその `arm.payload.name` で
         `arm.tag` は `Some(tag)` である。よって `<1>3a` (iii) の変位アームの場合と (vi) より
         `(tag, ty(u))` は `F(ty(scrut))` の要素であり、`<1>3a` (iv) より
         `ty(scrut).toplevel_tycon_info(E).variant` は `TyConVariant::Union` である。
        BY <1>3a, CODE src/rc_ir/ownership.rs: collect_bindings
      <4>2a. `ty(scrut).is_closure()` は偽であり、`ty(scrut).is_funptr()` も偽である。前者は
         `<1>3a` (iv) がその節の一部として述べる。後者は、`<4>2` より
         `ty(scrut).toplevel_tycon_info(E).variant` が `TyConVariant::Union` であり、`Union` は
         `TyConVariant::Primitive` ではないので、`<1>3ca` の対偶から出る。
        BY <1>1, <1>3a, <1>3ca, <4>2, CODE src/ast/types.rs: TyConVariant
      <4>2b. `sig` は `ty(u)` の unit に届き、`cls(ty(u))` は `NB` でない。この場合の仮定より `u` は
         `vars.bindings` の定義域にあるので、`<2>5a` より帰納法の仮定の前件が満たされ、`sig` は
         `ty(u)` の unit に届く。`ty(u)` は `vars.var_tys` が記録する型なので `<1>1` を満たし、
         `<2>4` よりそのとき `cls(ty(u))` は `NB` でなく、`DEF cls` より `is_fully_unboxed(ty(u))` は
         偽である。
        BY <1>1, <2>4, <2>5a, DEF cls
      <4>3. `cls(ty(scrut))` は `NB` でない。`is_fully_unboxed` の本体は、`is_box` が真なら偽を返し、
         `is_closure` が真なら偽を返し、`is_array` が真なら偽を返し、`is_funptr` が真なら真を返し、
         そのどれでもなければ `F` の各要素の第 2 成分についての `is_fully_unboxed` の連言を返す。
         この場合の仮定より `ty(scrut).is_box(E)` は偽、`<4>2a` より `is_closure()` と `is_funptr()`
         も偽である。`is_array()` が真ならその段で偽が返る。偽なら値は連言であり、`<4>2` より
         `ty(u)` はその 1 つで、`<4>2b` よりその `is_fully_unboxed` は偽なので、連言も偽である。
         どちらでも `is_fully_unboxed(ty(scrut))` は偽であり、`DEF cls` より `cls(ty(scrut))` は
         `NB` でない。
        BY <4>2, <4>2a, <4>2b, DEF cls, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
      <4>4. `cls(ty(scrut))` は `AR` か `UN` である。`<4>3` より `NB` でなく、`<4>2a` より `CL` でも
         なく、この場合の仮定より `is_box(E)` が偽なので `BX` でもない。`<4>2` より
         `ty(scrut).toplevel_tycon_info(E).variant` は `Union` なので `is_union(E)` は真であり、
         `DEF cls` の `ST` の行はそれが偽であることを要求するので `ST` でもない。
        BY <4>2, <4>2a, <4>3, DEF cls, CODE src/ast/types.rs: TypeNode::is_union
      <4>5. QED
        `ty(scrut)` は `<1>1` を満たし、`[tag] ++ sig` は空でないので、`<2>3` を
        `t' := ty(scrut)` に適用すると `[tag] ++ sig` は `ty(scrut)` の unit に届く。`<2>3` の仮定は
        `cls(t')` が `BX`、`AR`、`UN` のどれかであることで、`<4>4` がそれを与える。
        BY <1>1, <2>3, <4>1, <4>4
    <3>5. CASE `vars.bindings[u]` が `Binding::Field(cont, idx)` で `cont.ty.is_box(E)` が偽で
       ある。
      <4>1. 辺の先は `(cont, [idx] ++ sig)` である。
        BY DEF 呼び出しの辺, CODE src/rc_ir/ownership.rs: origin_inner
      <4>2. `Binding::Field(cont, idx)` を作るのは `collect_bindings` の
         `Destructure(cont, fields, _state, k)` の腕だけであり、`u` はその `fields` のある要素の
         変数である。よって `<1>3a` (v) と (vi) より `(idx, ty(u))` は `F(ty(cont))` の要素であり、
         `ty(cont).toplevel_tycon_info(E).variant` は `TyConVariant::Struct` である。
        BY <1>3a, CODE src/rc_ir/ownership.rs: collect_bindings
      <4>2a. `ty(cont).is_closure()` は偽であり、`ty(cont).is_funptr()` も偽である。前者は
         `<1>3a` (v) がその節の一部として述べる。後者は、`<4>2` より
         `ty(cont).toplevel_tycon_info(E).variant` が `TyConVariant::Struct` であり、`Struct` は
         `TyConVariant::Primitive` ではないので、`<1>3ca` の対偶から出る。
        BY <1>1, <1>3a, <1>3ca, <4>2, CODE src/ast/types.rs: TyConVariant
      <4>2b. `sig` は `ty(u)` の unit に届き、`cls(ty(u))` は `NB` でない。この場合の仮定より `u` は
         `vars.bindings` の定義域にあるので、`<2>5a` より帰納法の仮定の前件が満たされ、`sig` は
         `ty(u)` の unit に届く。`ty(u)` は `vars.var_tys` が記録する型なので `<1>1` を満たし、
         `<2>4` よりそのとき `cls(ty(u))` は `NB` でなく、`DEF cls` より `is_fully_unboxed(ty(u))` は
         偽である。
        BY <1>1, <2>4, <2>5a, DEF cls
      <4>3. `cls(ty(cont))` は `NB` でない。`is_fully_unboxed` の本体は、`is_box` が真なら偽を返し、
         `is_closure` が真なら偽を返し、`is_array` が真なら偽を返し、`is_funptr` が真なら真を返し、
         そのどれでもなければ `F` の各要素の第 2 成分についての `is_fully_unboxed` の連言を返す。
         この場合の仮定より `ty(cont).is_box(E)` は偽、`<4>2a` より `is_closure()` と `is_funptr()`
         も偽である。`is_array()` が真ならその段で偽が返る。偽なら値は連言であり、`<4>2` より
         `ty(u)` はその 1 つで、`<4>2b` よりその `is_fully_unboxed` は偽なので、連言も偽である。
         どちらでも `is_fully_unboxed(ty(cont))` は偽であり、`DEF cls` より `cls(ty(cont))` は
         `NB` でない。
        BY <4>2, <4>2a, <4>2b, DEF cls, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
      <4>4. `cls(ty(cont))` は `AR`、`UN`、`ST` のどれかである。`<4>3` より `NB` でなく、`<4>2a` より
         `CL` でもなく、この場合の仮定より `is_box(E)` が偽なので `BX` でもない。
        BY <4>2a, <4>3, DEF cls
      <4>5. CASE `cls(ty(cont)) = ST`。`ty(cont)` は `<1>1` を満たすので `<2>2` を
         `t' := ty(cont)`、`t := ty(u)`、`i := idx`、`q := sig` に適用できる。`<4>2` が
         `(idx, ty(u))` が `F(ty(cont))` の要素であることを、`<4>2b` が `sig` が `ty(u)` の
         unit に届くことを与える。
        BY <1>1, <2>2, <4>1, <4>2, <4>2b
      <4>6. CASE `cls(ty(cont))` が `AR` か `UN`。`ty(cont)` は `<1>1` を満たし `[idx] ++ sig` は
         空でないので、`<2>3` を `t' := ty(cont)` に適用する。`<2>3` の仮定は `cls(t')` が `BX`、
         `AR`、`UN` のどれかであることで、この場合の仮定がそれを与える。
        BY <1>1, <2>3, <4>1
      <4>7. QED
        BY <4>4, <4>5, <4>6
    <3>6. CASE `vars.bindings[u]` が `Binding::Llvm(llvm_gen, args, result_ty)` で、
       `decl.leaf_origins_at(sig)` が単一の `Arg(j, p)` である。辺の先は `(args[j], p)` であり、
       `<1>28a` より `p` は第 `j` オペランドの leaf、すなわち `L(ty(args[j]))` の要素である。
       `ty(args[j])` は `<1>1` を満たすので `<2>1` を `t := ty(args[j])` に適用できる。
      BY <1>1, <1>28a, <1>3a, <2>1, DEF 呼び出しの辺,
         CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: as_arg_projection
    <3>7. CASE `vars.bindings[u]` が `Binding::Llvm(llvm_gen, args, result_ty)` で、
       `origin_from_leaves_under` が呼ばれる。辺の先は `(args[j], unit)` であり、
       `unit = truncate_to_unit(&args[j].ty, leaf, E)` で `leaf` は宣言が名指す leaf である。
       `ty(args[j])` は `<1>1` を満たすので、`<1>28a` より `leaf` が `L(ty(args[j]))` の要素で
       あることと合わせて `<1>18` を `t := ty(args[j])` に適用でき、`unit` は `U(ty(args[j]))` の
       要素である。同じ型に `<1>32a` を適用する。
      BY <1>1, <1>28a, <1>3a, <1>18, <1>32a, DEF 呼び出しの辺,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>8. QED
      `origin_inner` が `origin` を呼ぶ場所は `<3>1` から `<3>7` の 7 つで尽きている。`None`、
      `Binding::Param`、`Binding::Producer` の腕、`Binding::Field` で容器が boxed の腕、
      `Binding::Payload` で `Some(_)` かつ scrutinee が boxed の腕は呼ばない。
      BY <3>1, <3>2, <3>3, <3>4, <3>5, <3>6, <3>7, DEF 呼び出しの辺,
         CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding
  <2>7. QED
    `<2>5` を基底、`<2>6` を帰納段とする、`(x, pi)` からの呼び出しの辺の道の長さについての帰納法。
    `<1>29b` より辺の関係は `vars` と `E` を固定すれば 1 つに定まるので、この帰納法は 1 つの集合の
    上を回る。`<1>25` より辺ごとに `ord` は狭義に減少し、`ord` の値は `-1` と `<1>21` の有限列の
    位置に限られるので、道の長さは有限で上から抑えられる。よって帰納法は呼び出しの下流の全体に届く。
    BY <1>21, <1>25, <1>29b, <2>5, <2>6, DEF 呼び出しの辺, DEF 呼び出しの下流

<1>34. **(P1 の系 2: `origin` が返す path も unit に届く)** `<1>33` と同じ 8 つの仮定と、同じ
   本体・表 `vars`・名前 `x` を取る。`pi` が `L(ty(x))` の要素または `U(ty(x))` の要素であるとき、
   `origin(vars, E, x, pi)` の返り値に現れる各 `VarPath` `(u, sig)` (identity と candidates の両方)
   について、`u` が `vars.var_tys` に型を持つならば `sig` は `ty(u)` の unit に届く。すなわち
   `T(ty(u), sig)` は abort せず、その値は `U(ty(u))` の要素である。

   **`pi` についての条件は落とせない。**`x` を型 `Std::I64` のパラメータ、`pi = [0]` とする。
   `origin_inner` の `Some(Binding::Param)` の腕は `here()` を返すので返り値は `Exactly((x, [0]))`
   であり、`VarTable::of` は各パラメータを `var_tys` に入れるので `u = x` は型を持つ。`bulitin_tycons`
   が `Std::I64` に与える `TyConInfo` は `variant: Primitive`、`is_unbox: true`、`fields: vec![]` なので、
   `is_box`、`is_closure`、`is_array`、`is_funptr` はすべて偽で `F(I64)` は空であり、
   `is_fully_unboxed(I64)` は空の連言として真になる。すなわち `cls(I64) = NB` であり、`<1>10` より
   `unit_step(I64, E)` は `UnitStep::NoUnit` で、`T(I64, [0])` はループの第 0 周で `panic!` に達する。
   **`origin` の答えの `VarPath` の第 2 成分を `units_under` と `T` に掛ける読み手は、`owns_object` と
   `owns_object_yet` である。**前提 `truncate_to_unit` を呼ぶ在りか が `src/` の呼び出し元を挙げ、
   その分類が、path を `origin` の答えから得るのはこの 2 つであり、残りが渡すのは `boxed_leaf_paths` が
   挙げる leaf か、`rhs_consumes` が報告する leaf か、`result_prov` の宣言が名指す leaf であることを
   述べる。
   `owns_object` へ対を渡すのは `owns_unit` と `check_ownership_is_levelled`、`owns_object_yet` へ
   対を渡すのは `level_ownership` であり、3 つとも問うのは site の unit についての `origin(v, u)` な
   ので、この条件を満たす。第 5 節がその site を数え上げる。
  <2>1. `origin(vars, E, u, sig)` が返す値に現れる各 `VarPath` は、`(u, sig)` の呼び出しの下流
     (`DEF 呼び出しの下流`) にある対である。
    <3>1. `origin_inner` の `None`、`Binding::Param`、`Binding::Producer` の腕、`Binding::Field` で
       容器が boxed の腕、`Binding::Payload` で `Some(_)` かつ scrutinee が boxed の腕は
       `here() = Origin::Exactly((var.clone(), path.to_vec()))` を返す。現れる `VarPath` は
       `(u, sig)` 自身である。`DEF 呼び出しの下流` は 0 回の場合を含むので、`(u, sig)` 自身は
       `(u, sig)` の下流にある。
      BY DEF 呼び出しの下流, CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Move` の腕、`Binding::Payload` で `None` の腕、`Binding::Payload` で `Some(tag)`
       かつ scrutinee が unbox の腕、`Binding::Field` で容器が unbox の腕、`Binding::Llvm` で
       `leaf_origins_at` が単一の `Arg` の腕は、辺の先の `origin` の返り値をそのまま返す。帰納法の
       仮定より、現れる `VarPath` はその辺の先の下流にあり、したがって `(u, sig)` の下流にある。
      BY DEF 呼び出しの辺, DEF 呼び出しの下流, CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. `Binding::Join` の腕は `Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))`
       を返す。`candidates` は各子の返り値の `acted_on()` の合併である。`acted_on()` が返すのは
       `identity()` と `candidates()` の元、すなわちその辺の先の返り値に現れる `VarPath` だけな
       ので、帰納法の仮定よりその各要素は辺の先の下流にある。`of_candidates` は要素数 1 のとき
       `into_iter().next()` が返す元を `Origin::Exactly` に置き -- `EXT 1 要素の集合の反復` より
       それは `candidates` のその 1 つの元である -- 、それ以外のとき
       `Origin::Join { identity: (var, path), candidates }` を返す。前者に現れるのは辺の先の下流の
       対、後者に現れるのはそれと `(u, sig)` 自身である。どちらも `(u, sig)` の下流にある。
      BY EXT 1 要素の集合の反復, DEF 呼び出しの辺, DEF 呼び出しの下流,
         CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: Origin::of_candidates,
         CODE src/rc_ir/ownership.rs: Origin::acted_on,
         CODE src/rc_ir/ownership.rs: Origin::candidates
    <3>4. `Binding::Llvm` で `origin_from_leaves_under` を呼ぶ腕。`origin_from_leaves_under` は
       `reached` を、各辺の先の返り値と、`produced_here` が真のときの
       `Origin::Exactly(here.clone())` (`here` は `(u, sig)` 自身) から作る。`reached` が空なら
       `None` を返し、`origin_inner` は `here()` を返す (`(u, sig)` 自身)。`reached` の要素が
       すべて等しければその 1 つを返す。そうでなければ `Origin::of_candidates(candidates, here)` を
       返し、`candidates` は各 `reached` の
       `acted_on()` の合併である。`acted_on()` が返すのは `identity()` と `candidates()` の元、
       すなわちその `Origin` に現れる `VarPath` だけである。いずれの場合も現れる `VarPath` は、
       辺の先の下流にある対か `(u, sig)` 自身であり、どちらも `(u, sig)` の下流にある。
      BY DEF 呼び出しの辺, DEF 呼び出しの下流,
         CODE src/rc_ir/ownership.rs: origin_inner,
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
      BY <1>21, <1>25, <1>29b, <3>1, <3>2, <3>3, <3>4, <3>5,
         DEF 呼び出しの辺, DEF 呼び出しの下流,
         CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding
  <2>2. `<2>1` を `(u, sig) := (x, pi)` に読むと、`origin(vars, E, x, pi)` の返り値に現れる各
     `VarPath` は `(x, pi)` の呼び出しの下流にある。この言明の仮定は `pi` が `L(ty(x))` の要素か
     `U(ty(x))` の要素であることなので、`<1>33` を同じ `(x, pi)` に適用でき、その各対について、
     第 1 成分が `vars.var_tys` に型を持つならば第 2 成分がその型の unit に届く。
    BY <1>33, <2>1, DEF 呼び出しの下流
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

`<1>1` (H1) は README の A10 (型の well-formedness)、`<1>2` (H2) は A11 (スコープの規律) を A6 と
D6 と合わせて読んだもの、`<1>3a` (H4) は A12 (束縛の形と型が合っている) と A3 (宣言されたモデルの
忠実さ) の 1 段である。README の文面とこの文書の記法との対応は、3 つの前提のそれぞれについて次の
とおりである。

- **`<1>1` の 3 つはどれも A10 である。**(i) は A10 の第 1 文のうち「プログラムに現れる型は ground で
  あり、**その tycon に kind の要求するだけの引数が与えられており**、その tycon は `type_env` にあり、
  …」の部分そのものである。省略記号が置き替えるのは `no_size_in_place` の降下についての節で、この
  文書はそれを読まない (この項の末尾)。
  **引数の個数を `tyvars` の長さと書き直すのは A10 の飽和の定義による** --「**飽和とは、
  `collect_type_arguments().len()` が `tycon_info.tyvars.len()` に等しいことである。**」。
  (ii) と (iii) は A10 の
  「`unpunched_field_types` を繰り返し取って到達する型についても、上の 3 つ -- ground、飽和、tycon が
  `type_env` にある -- がすべて成り立ち、その歩みは有限である。さらに、到達する各型について
  `instance_field_types` が行う newtype の展開 (`unwrap_newtypes_memoized`) は abort せず停止する」
  の、前半と後半である。

  A10 は (iii) の果たす者について「果たしうるのは `unwrap_newtype` パスと `validate_type_defns` だが、
  **証明していない**」と書く。この文書も (iii) を仮定として使い、`<1>3c` の `<2>7` がそれを読む。

  A10 の但し書き -- 果たす者 `validate_layouts` は elaboration で必ず走るが、最適化が作る型を
  再検査するのは develop build だけである -- は `<1>1` の 3 つすべてに掛かる。

  この文書は A10 の第 1 文のうち `no_size_in_place` の降下についての節を読まない。`REC` の辺も
  `DESC` の辺も `unpunched_field_types` の辺なので、整礎性は (ii) から直に出る (`<1>3d`、`<1>8`)。
- `<1>2` は A11 の「スコープに入っている束縛」を `DEF Scope` で節点の種類ごとに書き下し、その上で
  `vars.bindings` の定義域に無い名前を許す。`origin_inner` の `None` の腕がその名前を受ける
  (`CODE src/rc_ir/ownership.rs: origin_inner`)。書き下しそのものの根拠は D2 のスコープ規則であり、
  根の場合の 2 つ目 (グローバル初期化子) は D1 の「`init` はパラメータも capture も持たない」から
  出る。A11 の検査 `validate` の `scope` の推移も同じ形で、関数ごとに `func.params` と
  `func.capture` を `bind` してから本体を検査し、グローバル初期化子については何も `bind` せずに
  本体を検査する。

  **`<1>2` は本体を関数の `body` に限らない。**`origin` はグローバル初期化子の `init` について
  `VarTable::body_only` が作る表の上でも走る (`<1>31`)。A11 は本体の 2 つの形をどちらも述べる --
  「関数の本体の自由な局所名は、その関数のパラメータと capture に限る。グローバル初期化子の `init` は
  自由な局所名を持たない。」`<1>2` が「定義域に無い名前」の側を `vars.bindings` で述べるのは、D6 の
  「局所名であることと束縛を持つことは同値である」による。使われる局所名がその本体の束縛に解決すると
  言うのに A6 (名前の一意性) が要るのは、A11 が解決先を「スコープに入っている束縛」としか言わず、
  同じ名前の束縛が 2 つあればどちらとも読めるからである。
- `<1>3a` は、この文書が使う 8 つを (i) から (viii) として述べる。**(i) から (vii) は A12 に、
  (viii) は A3 に在る。**(vii) の 3 つは A12 の「`Llvm` 節点の型についての 4 つ」のうち、この文書が
  読む 3 つであり、(viii) は A3 の
  「**単一の `Arg(j, σ)` の宣言は well-formed である。** `j` は `args` の添字であり、`σ` はその型の
  boxed leaf である」の段落である。宣言の well-formedness は `LLVMGen` の宣言についての仮定であり、
  それを置くのは A3 である。`<1>28a` の `<2>3` も同じ段を A3 から引く。

  A12 の「`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が実際に持つ (punched
  でない) ものであること」は (iii) と (v) が述べており、この文書はそれを読む -- (iii) と (v) が
  `F`、すなわち `unpunched_field_types` の要素であることを言う形がそれであり、`<1>33` の `<2>6` の
  `<3>4` と `<3>5` がそこから穴の下へ降りないことを引く。A12 の残り -- `App` の引数と呼び出し先の
  パラメータの型が合っていること、`App` の結果の型が呼び出し先の返り値の型であること、`ty(callee)` が
  実行時の呼び出し先の `fn_ty` であること、束縛を持たない `RcVar` の型がその名前の記号の型で
  あること、`Llvm` 節点の型についての 4 つのうち残る 1 つ (`InlineLLVMStructGetBody` と
  `InlineLLVMUnionAsBody` の `ty(x)` が `ty(args[0])` の第 `field_idx` フィールド・変位の型で
  あること)、そして `RcFunc` の欄どうしの整合 -- は、この文書のどのステップも読まない。

  **A12 の「`Match` の scrutinee が union であること」と「`Destructure` の容器が構造体であること」
  を、(iv) と (v) はその型の `TyConInfo` の `variant` として書き、A12 の「この仮定が型の `variant` を
  述べる各節では、その型の `is_closure()` は偽である。」を一緒に再掲する。**A12 はその文を「その各節の
  一部であって、別の主張ではない」と書き、再掲する段にそれを一緒に運ぶことを求める。`<1>33` の
  `<2>6` の `<3>4` と `<3>5` が読むのはこの形であり、`is_closure()` の偽を (iv) と (v) から直に取り、
  `variant` から `is_funptr()` の偽を出す (`<1>3ca`)。

  `<1>3a` の言明は関数の `body` とグローバル初期化子の `init` の両方に掛かる。A12 が本体の形を
  限っていないのと同じである。

  A12 の第 2 の `Llvm` 節点の条件は punched struct の成分を「A10 を満たす構造体」と書く。`<1>1` は
  A10 をこの文書の記法で述べたものなので、(vii) はそこを `<1>1` と書く。同じ条件の「第 `field_idx`
  フィールドが穴である」を、(vii) はその成分の `TyConInfo` の `fields` の `is_punched` として書き、
  A12 の同じ条件が併せて述べる「`is_closure()` は偽である」を一緒に挙げる。`<1>28` の `<2>2f` の
  `<3>4` がこの 2 つを読む。同じ条件が `ty(x)` について並べる `is_box`・`is_array`・`is_closure` の
  3 つも (vii) はそのまま写し、`<1>28` の `<2>2f` の `<3>3a` がその 3 つを読む。

  (vii) が要る理由は、これが無いと `result_prov` の 29 個の override のうち 5 個が abort しうるので
  P2 が偽になることである。`InlineLLVMStructGetBody` と `InlineLLVMUnionAsBody` は `arg_tys[0]`
  で添字付けし、`InlineLLVMStructPunchBody` は `result_ty.field_types(E)[PUNCHED_STRUCT_FIELD]` で
  添字付けし、`InlineLLVMStructSetBody` と `InlineLLVMStructPlugInBody` が呼ぶ
  `replaced_field_prov` は、boxed でない結果の boxed leaf の path が空でないことを `expect` で
  要求する (`Array a` を結果に持てば発火する)。

**P1 の定義域。** `<1>20` が示すのは `<1>1` を満たす型についての P1 であり、`<1>1` は A10 を
この文書の記法で述べたものなので、それは README の P1 --「**A10 を満たす**任意の型 `τ` について」--
そのものである。A10 が型に条件を置くことが空虚でないのは `<1>19a` による -- `t.is_closure()` が偽で
`t.toplevel_tycon()` が `None` を返すか返す型構成子が `E` に無い型については、`boxed_leaf_paths` も
`rc_units` も `toplevel_tycon_info` の `unwrap` で abort し、P1 の言明の 2 つの辺が意味を持たない。

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

**`<1>33` と `<1>34`。**その unit も、その下の各 leaf も、`U(ty(v))` か `L(ty(v))` の要素なので、
どちらを `pi` に取っても `<1>33` と `<1>34` の条件を満たす。そのとき、`origin(vars, E, v, pi)` の
計算の中で起きる呼び出しの `(u, sig)` も、返り値に現れる `VarPath` `(u, sig)` も、`u` が
`vars.var_tys` に型を持つ限り `sig` が `ty(u)` の unit に届く。すなわち
`truncate_to_unit(ty(u), sig)` は abort せず `rc_units(ty(u))` の要素になる。この性質は `origin` の
再帰の各辺 (move-bind、`Match` のアームの結果、変位アームの payload、catch-all の payload、
unbox 容器のフィールド、`Llvm` の 2 つの道) が保つ。

**`origin` の答えに `units_under` と `truncate_to_unit` を当てるコードは 2 つある。**在りかを挙げるのは
前提 `truncate_to_unit` を呼ぶ在りか である。そのうち path を `origin` の答えから得るのは
`borrow.rs` の `owns_object` と `owns_object_yet` であり、残りが渡すのは、
`boxed_leaf_paths` が挙げる leaf (`owns_object_yet` の中でキーと突き合わせる側と、
`borrow_ify` が借用版の `owned_units` を組む箇所)、`rhs_consumes` が報告する leaf
(`CancelAnalysis::consume_rhs` の `owns`)、`result_prov` の宣言が名指す leaf
(`origin_from_leaves_under`) である。

`owns_unit` と `check_ownership_is_levelled` が `origin(v, unit).candidates()` の各 `(root, path)` を
`owns_object` に渡し、`level_ownership` が同じ形で `origin(v, unit).candidates()` の各 `(root, path)`
を `owns_object_yet` に渡す。どちらの関数も `root` が `vars.param_tys` にあるとき `path` を
`units_under` と `truncate_to_unit` に掛ける。その `unit` が `U(ty(v))` の要素であることは、`App` の
引数の site については P7a の site の定義が、`Retain`/`Release` 節点の site については A2
(単位への正規化) と、借用版については P9 (複製は名前替えである) が与える。`level_ownership` が受け取る
site は `infer_ownership` が入力の各関数について `levelled_sites` から作るものなので、`Retain`/`Release`
節点については A2 だけで足りる。`param_tys` に入る名前は `var_tys` にも同じ型で入るので、`<1>34` が
その `path` について「`ty(root)` の unit に届く」を与える。`owns_object` を主語とする命題は P7e、
`level_ownership` と `owns_object_yet` を組で主語とする命題は P7d である
(`CODE src/rc_ir/borrow.rs: owns_object, owns_object_yet, level_ownership, levelled_sites,
infer_ownership, RewriteCtx::owns_unit, RewriteCtx::check_ownership_is_levelled,
CancelAnalysis::consume_rhs, borrow_ify`,
`CODE src/rc_ir/ownership.rs: units_under, truncate_to_unit, rhs_consumes, origin_from_leaves_under`,
`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)。
