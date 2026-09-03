# P2a, P15 - P18: `cancel` の走査

この文書は README の層 1 の命題 P2a と、層 3 の 4 命題 P15, P16, P17, P18 を証明する。README の
「定義」の節が置く定義と「仮定」の節が置く仮定の上に立つ。P2a を除く層 1 の命題と、層 2 の命題は引用しない。

この文書が読んだコードのコミットは `e5f2b9c79460be3ae60b46e3900267c36ac98692` である。README が証明の
対象として名指すコミット `b6c51fb892746e493e155d9d59ea05d02d7357db` との間で、この文書の `CODE` 引用が
名指すファイルは 19 個ある。そのうち 18 個 --- `src/rc_ir/borrow.rs`、`src/rc_ir/ownership.rs`、
`src/rc_ir/ast.rs`、`src/rc_ir/provenance.rs`、`src/rc_ir/leaf_map.rs`、`src/rc_ir/rename.rs`、
`src/ast/types.rs`、`src/ast/typedecl.rs`、`src/ast/name.rs`、`src/ast/program.rs`、
`src/elaboration/typecheck.rs`、`src/elaboration/desugar_opaque.rs`、`src/misc.rs`、
`src/build/build_object_files.rs`、`src/main.rs`、`src/object.rs`、`src/tool/log_file.rs`、
`src/tests/test_util.rs` --- に変わったのは `// PROOF:` コメントだけである。

残る 1 個 `src/rc_ir/validate.rs` には、コメント以外の変更がある。`Validator::check_rhs` の署名が
`(&mut self, x: &RcVar, rhs: &RcRhs)` になり、その `Llvm` の腕が `llvm_gen.result_prov(&x.ty, &arg_tys,
self.type_env)` を呼んで、1 つの結果 leaf に宣言された source の個数が 2 以上のとき `panic!` する検査が
入った。README の「対象」の節がこの変更を挙げ、A3 の「**複数の元を宣言する op は存在しない。**」を果たす者と
して数える。この文書が `validate` を引くのは L2b の `<1>2` と `<1>3` の 2 か所であり、どちらもこの検査を
含む `validate` について読む。

P15 の言明は `cancel` の入力を「`borrow_ify` の出力」に限る。P16 - P18 もその入力に対する走査についての
言明なので、この文書は全体を通じて、`cancel` の引数 `prog` が `borrow_ify` の 1 回の呼び出しの返り値で
ある場合を扱う。この仮説を `ASSUME` の形で明示するのは L3、L4、L8、L8a、L9、L11、L13 の 7 本である。
**L2b がそれを `cancel` のすべての呼び出しについて無条件に果たす**ので、P16 と P18 は README が書くとおり
の、仮説を持たない言明として立つ。

## 0. この文書が使う記法

README の「証明の記法」の節に、次の 3 つを加える。

- **局所の定義**。この文書の中だけで使う語を定め、`BY` の行では `DEF <名前>` で引用する。定義は第 1 節に
  置く。定めるものが 1 つに決まることを補題が与えるときは、その補題の後に置く --- `DEF 節点の量` は
  L0c の後である。
- **局所の補題**。この文書の中だけで使う補題を `L0` - `L13` と番号を付けて述べ、`BY` の行では
  `L<n>` で引用する。あいだに挟む補題には `L8a` のように枝番を振り、既存の番号は振り直さない。各補題は、
  それより前に置かれた補題と命題と、README の D/A だけを引用する。
- **外部の結果**。`BY` の行では `EXT <名前>` として引く。Rust の言語と標準ライブラリの契約、および
  `Cargo.toml` が宣言するビルド構成のうちこの文書が使うものを、第 1 節の「外部の結果」で
  完全な言明とともに述べる。ビルド構成をここに置くのは、`CODE` が名指せるのが `.rs` と `.fix` の
  ファイルの記号だからである。

`CODE` の引用はファイル名と記号の道で書く。引用するコードはすべてこのリポジトリの中にある。

## 1. 局所の定義

### DEF 本文

関数 (またはメソッド) の**本文**とは、その定義に書かれた式と文の全体をいう。**ある本文の中に書かれた
クロージャ式の本文は、その本文の一部である。** 「関数 `f` が関数 `g` を**呼ぶ**」とは、`f` の本文に `g` の
呼び出しが書かれていることをいう。

この約束のもとでは、`borrow.rs` の中で書かれたクロージャを標準ライブラリの関数へ渡しても、その
クロージャの本文は `borrow.rs` の関数の本文の一部のままであって、受け取った標準ライブラリの関数の本文の
一部にはならない。「呼び出しが `borrow.rs` の中にしか書けない」も、この意味の本文について読む。

**Rust の側を「本文」、RC IR の側を「本体」と呼んで分ける。** D2 の**本体**は式の節点の木であり、D23 の
**本体**は活性化の主語 --- ある関数の `body` か、あるグローバル初期化子の `init` --- である。この文書は
その 2 つの意味でも「本体」を使うので、Rust の関数の側にはこの語を当てない。`match` の腕のように関数より
小さい単位について「本文」と書くときも、指すのはその腕に書かれた式と文の全体である。

### DEF このクレート

この文書が「**このクレート**」と書くとき指すのは、EXT このリポジトリのターゲット が挙げる 2 つの
クレート --- lib の `fixlang` と bin の `fix` --- のいずれか 1 つである。

**数え上げの走査の範囲を `src/` の全体と定め、走査が挙げた一覧を、どちらのクレートで読んでも実際の
集合の上位集合として読む。** この文書の数え上げはどれも「挙げた以外の場所には無い」と言う向きに使うので、
上位集合で読んで結論が保たれる。

### DEF 部分木

D2 の意味での本体の木の位置を**節点**と呼ぶ。節点 `n` の**子**を次で定める。

| `n` の式 | 子 |
|---|---|
| `Let(_, Match(_, arms), k)` | `arms` の各 `arm.body`、および `k` |
| `Let(_, rhs, k)` (`rhs` は `Match` でない) | `k` |
| `Retain(_, _, _, k)` | `k` |
| `Release(_, _, _, k)` | `k` |
| `Destructure(_, _, _, k)` | `k` |
| `Eval(_, k)` | `k` |
| `Ret(_)` | 無し |

節点 `n` の**部分木** `N(n)` を、`n` と、`n` の各子 `c` についての `N(c)` との合併とする。

**節点の道**を、本体の根からその節点まで、上の表が挙げる子のどれを選んだかを順に並べた列とする。

### DEF 訪問

`walk_inner` の 1 回の呼び出しを**訪問**と呼び、その `node` 引数が指す節点を訪問した、という
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)。呼び出しの時間順を**訪問順序**と呼ぶ。

節点 `n` の訪問における `pending` 引数の値 (その訪問がそれに変更を加える前の値) を `pending(n)` と書き、
**入口状態**と呼ぶ。その訪問の戻り値を `pending_out(n)` と書き、**出口状態**と呼ぶ。

節点 `n` が時点 `τ` までに**訪問された**とは、`n` の訪問がその時点までに始まっていることをいう。節点 `m`
が節点 `n` より**前に訪問された**とは、`m` の訪問が `n` の訪問より前に始まっていることをいう。

### DEF 参照の多重集合

`References` の値を、その `Map<VarPath, usize>` の鍵を**位置** (`VarPath`)、値をその位置について数えた
参照の個数とする多重集合とみなす (`CODE src/rc_ir/ownership.rs: References`)。次の記法を使う。

- `R2 ⊆ R1` とは、各位置について `R2` の個数が `R1` の個数以下であることをいう。
- `R1 - R2` とは、各位置の個数の差である (`R2 ⊆ R1` のときだけ書く)。
- **空**とは、参照を 1 つも持たないことをいう。
- 2 つの `References` の値が**等しい**とは、`PartialEq` が真を返すことをいう。

### DEF 割り当て

`Arc::new(v)` の 1 回の呼び出しは、アロケータから 1 つのメモリブロックを取る。このブロックを**割り当て**と
呼ぶ。相異なる 2 つの `Arc::new` の呼び出しが取るブロックを、相異なる割り当てという。割り当ての**占める
番地**とは、そのブロックが占める番地の集合の元である。`Arc::new` が返した値と、`Arc::clone` がそれから
作った値を、その割り当ての**ハンドル**と呼ぶ。

ハンドルが 1 つ以上存在する間、その割り当ては**生存している**という。ブロックがアロケータへ返ることを、
その**割り当てが解放される**という。

**この 2 つの語は D7 の語とは別の概念を指す。**「割り当てが解放される」「生存している」は、コンパイラの
プロセスの中の Rust の `Arc` についての語である。D7 の「オブジェクトが解放される」は、コンパイルされた
プログラムを実行したときのヒープオブジェクトの参照カウントが 0 になることをいう。この文書に現れるのは
前者だけである。

### DEF 引数で決まる関数

関数の返り値が**引数で決まる**とは、引数の値が等しい 2 回の呼び出しが同じ値を返すことをいう。返り値が
`Set` の反復から作られる列であるように、決まるのが並びではなく元の集合であるときは、**元の集合が引数で
決まる**という。

### 外部の結果

この文書が使う Rust の言語と標準ライブラリの契約を、名前を付けて述べる。

**EXT 呼び出しの入れ子**
関数の呼び出しは入れ子である。呼び出し `c` の中で始まった呼び出しは、`c` が返るより前に返る。よって
2 つの呼び出しの実行区間は、交わらないか、一方が他方に含まれるかのどちらかである。また、返る呼び出しの
中で始まる呼び出しは有限個である。

**EXT 参照は引数を通ってだけ届く**
safe Rust で書かれた関数の本文が名指せる値は、自分の引数 (`self` を含む) から到達できる値、自分が作った
値、および `static` 項目の値だけである。よって、呼び出し先の本文が呼び出し元の局所変数に届くのは、その値
かそれへの参照が引数として渡ったときに限る。

**EXT static は Sync を要る**
safe Rust の `static` 項目の型は `Sync` でなければならない。`Sync` は auto trait であり、`RefCell<T>` は
`Sync` を実装しないので、`RefCell` の欄を持つ構造体も `Sync` でない。

**EXT 可視性と私有性**
Rust Reference の "Visibility and Privacy" が次を述べる。

> By default, everything is *private*, with two exceptions: Associated items in a `pub` Trait are
> public by default; Enum variants in a `pub` enum are also public by default.

> With the notion of an item being either public or private, Rust allows item accesses in two cases:
>
> 1. If an item is public, then it can be accessed externally from some module `m` if you can access
>    all the item's ancestor modules from `m`. You can also potentially be able to name the item
>    through re-exports. See below.
> 2. If an item is private, it may be accessed by the current module and its descendants.

同じ節が `pub(crate)` について次を述べる。

> `pub(crate)` makes an item visible within the current crate.

すなわち、`pub` の付かない項目 --- 自由関数、型、inherent な `impl` の中のメソッド、構造体のフィールド
--- を名指せるのは、それを宣言したモジュールとその子孫のモジュールの中だけである。`pub(crate)` の付いた
項目を名指せるのは、そのクレートの中だけである。

**EXT モジュールは `mod` が導入する**
Rust Reference の "Modules" が次を述べる。

> A module is a container for zero or more items.

> A *module item* is a module, surrounded in braces, named, and prefixed with the keyword `mod`. A
> module item introduces a new, named module into the tree of modules making up a crate.

> Modules can nest arbitrarily.

すなわち、1 つのファイルの本体が `mod` の項目を 1 つも書かなければ、そのファイルのモジュールは子孫の
モジュールを持たない。EXT 可視性と私有性 と合わせると、そのファイルが宣言する非公開の項目を名指せるのは
そのファイルの中だけである。書かれた `mod` の項目が `#[cfg(test)] mod tests` の 1 つだけであれば、
子孫はそのモジュールだけである。

**EXT 文は書かれた順に実行される**
Rust Reference の "Block expressions" が次を述べる。

> When evaluating a block expression, each statement, except for item declaration statements, is
> executed sequentially.

> Then the final operand is executed, if given.

すなわち、ブロックの中の 2 つの文は書かれた順に実行され、末尾式はすべての文の後に評価される。

**EXT 演算対象は式より先に評価される**
Rust Reference の "Evaluation order of operands" が、呼び出し式 (call expression) とメソッド呼び出し式
(method call expression) を含む一覧を挙げたうえで、次を述べる。

> The operands of these expressions are evaluated prior to applying the effects of the expression.
> Expressions taking multiple operands are evaluated left to right as written in the source code.

同じ節が次の注を置く。

> Since this is applied recursively, these expressions are also evaluated from innermost to
> outermost, ignoring siblings until there are no inner subexpressions.

すなわち、呼び出しの引数はその呼び出しより先に、左から右の順に評価され、入れ子になった呼び出しは
内側から順に評価される。

**EXT このリポジトリのターゲット**
`Cargo.toml` は 3 つのターゲットを宣言する --- `[lib] name = "fixlang", path = "src/lib.rs"`、
`[[bin]] name = "fix", path = "src/main.rs"`、`[[bench]] name = "typecheck", harness = false`
(既定のパス `benches/typecheck.rs`) である。`src/lib.rs` と `src/main.rs` は同じ 28 個のモジュールを
`mod` で導入し、`src/main.rs` はさらに `#[cfg(test)] mod tests` を導入する。すなわち `src/` の 1 つの
ファイルは lib と bin の 2 つのクレートに属し、どちらのクレートの項目も `src/` のファイルが宣言する。
ベンチのターゲットは第 3 のクレートであり、`fixlang` を外部クレートとして読む。

**EXT 共有参照は代入を許さない**
Rust Reference の "Pointer types" が共有参照について次を述べる。

> When a shared reference to a value is created, it prevents direct mutation of the value. Interior
> mutability provides an exception for this in certain circumstances. As the name suggests, any
> number of shared references to a value may exist. A shared reference type is written `&type`, or
> `&'a type` when you need to specify an explicit lifetime.

すなわち `&T` を通じて `T` の欄へ代入することはできない。`&T` から値を動かす道として残るのは、その型が
`UnsafeCell` を通じて持つ内部可変性だけである。

**EXT 型のサイズ**
型の値が占める記憶域の大きさを、その型の**サイズ**という。構造体の各フィールドと、enum の 1 つの変位が
保持する各値は、その値の記憶域の中に互いに重ならずに置かれる。よって、構造体のサイズはその各フィールドの
型のサイズ以上であり、enum のサイズはその各変位が保持する各値の型のサイズ以上である。

**EXT bool のサイズ**
`bool` のサイズは 1 である。

**EXT Arc の契約**
`Arc::new(v)` は、`v` を保持するメモリブロックをアロケータから取り、そのブロックへのハンドルを 1 つ返す。
`<Arc<T> as Clone>::clone` は同じブロックへのハンドルをもう 1 つ作り、強参照カウントを 1 増やす。
`<Arc<T> as Drop>::drop` は強参照カウントを 1 減らす。ブロックがアロケータへ返るのは強参照カウントが 0 に
なった後なので、ハンドルが 1 つでも在る間、そのブロックはアロケータへ返らない。

`<Arc<T> as AsRef<T>>::as_ref` は、そのハンドルが指すブロックの中に置かれた `T` の値への共有参照を返す。
`T` のサイズが 0 でないとき、その `T` の値が占める番地はそのブロックが占める番地であり、とくに返る参照の
番地はそのブロックの占める番地の 1 つである。

**EXT アロケータの契約**
アロケータが返した 2 つのメモリブロックが同時に *currently allocated* である (どちらもまだアロケータへ
返っていない) とき、その 2 つは記憶域を共有しない。すなわち、一方の占める番地の集合と他方の占める番地の
集合は交わらず、とくに先頭アドレスが相異なる。

**EXT Vec::default**
`<Vec<T> as Default>::default()` は、要素を 1 つも持たない `Vec` を返す。

**EXT Vec::push**
`v.push(x)` は `v` の末尾に要素 `x` を 1 つ加える。既存の要素の値と添字は変わらず、長さは 1 増える。

**EXT Vec::remove**
`v.remove(i)` は添字 `i` の要素を取り除いて返し、それより後ろの要素を 1 つずつ前へ詰める。残る要素の値と
相対順序は変わらない。`i` が長さ以上のとき panic する。

**EXT Vec::retain**
`v.retain(f)` は、`f(&e)` が偽を返す要素 `e` をすべて取り除く。`f` は元の並びの順に各要素についてちょうど
1 回呼ばれ、残る要素の値と相対順序は変わらない。

**EXT Vec::iter と slice::iter**
`v.iter()` は、`Vec<T>` またはスライス `&[T]` の値 `v` の各要素への共有参照を、先頭から順にちょうど
1 度ずつ渡す反復子である。

**EXT スライスの接頭と先頭**
`s.starts_with(p)` は、`s` の長さが `p` の長さ以上であり、`s` の先頭からの `p` の長さ個の要素が `p` の
要素と順に等しいとき真、そうでないとき偽である。`s.first()` は、`s` が空でないときその第 0 要素への共有
参照を `Some` で返し、空のとき `None` を返す。

**EXT Iterator::all と any**
`it.all(f)` は、`it` が渡すすべての要素について `f` が真を返すとき真であり、`f` が偽を返す要素が 1 つでも
あれば偽である。`it.any(f)` は、`it` が渡すいずれかの要素について `f` が真を返すとき真であり、どの要素に
ついても偽なら偽である。どちらも判定を先頭から順に行い、答えが決まった時点で止める。

**EXT Iterator::map と collect**
`it.map(f)` は、`it` の各要素に `f` を適用した結果を並べる反復子である。`Vec` への `collect` は反復子の
要素をすべて順に取り出して `Vec` に並べるので、`it.map(f).collect::<Vec<_>>()` の長さは `it` の要素数に
等しく、第 `i` 要素は `it` の第 `i` 要素に `f` を適用した値であり、`f` は各要素についてちょうど 1 回、
先頭から順に呼ばれる。

**EXT Iterator::filter_map**
`it.filter_map(f)` は、`it` の各要素 `x` について `f(x)` が `Some(y)` のとき `y` を、`None` のとき何も
生じない反復子である。`f` は各要素についてちょうど 1 回、先頭から順に呼ばれ、生じる要素の順序は `it` の
順序である。

**EXT Iterator::fold と rev**
`it.fold(init, f)` は、累積値を `init` から始め、`it` が渡す各要素 `x` について累積値を
`f(累積値, x)` で置き換え、最後の累積値を返す。`f` は各要素についてちょうど 1 回、`it` の順序で呼ばれる。
`it.rev()` は、両端から取り出せる反復子 `it` の要素を末尾から先頭へ 1 度ずつ渡す反復子である。
`Vec<T>` の `into_iter()` はその要素を先頭から順に 1 度ずつ渡し、両端から取り出せる。

**EXT Iterator::rposition**
`it.rposition(f)` は、`f` を満たす要素が在るとき、そのうち先頭から数えた添字が最大のものの添字を `Some` で
返し、無いとき `None` を返す。判定は後ろから行われ、最初に真を返した時点で止まる。

**EXT IntoIterator と for**
`for x in e { ... }` は `IntoIterator::into_iter(e)` で反復子を作り、`Iterator::next` が `Some(x)` を
返す限り波括弧の中を 1 回ずつ実行する。`Vec<T>` の `<Vec<T> as IntoIterator>::into_iter` はその要素を先頭から
順に 1 度ずつ渡し、`&Vec<T>` の `<&Vec<T> as IntoIterator>::into_iter` はその要素への共有参照を先頭から
順に 1 度ずつ渡す。どちらも要素を落とさず、重複させない。

**EXT Map と Set**
`Map<K, V>` は `FxHashMap<K, V>`、`Set<K>` は `FxHashSet<K>` の別名である
(`CODE src/misc.rs: Map`, `CODE src/misc.rs: Set`)。次の契約を使う。

- `m.get(&k)` は、`m` が鍵 `k` を持つときその値への共有参照を `Some` で返し、持たないとき `None` を返す。
- `m.insert(k, v)` は、`m` が鍵 `k` を持たないとき `(k, v)` を加え、持つときその鍵の値を `v` で置き換える。
  どちらの場合も、`k` 以外の鍵とその値は変わらず、鍵が失われることはない。
- `m.contains_key(&k)`、`s.contains(&k)` は、その鍵 (要素) を持つことと同値である。
- `s.insert(k)` は、`s` が `k` を持たないときそれを加え、持つとき何もしない。どちらの場合も、ほかの要素は
  変わらず、要素が失われることはない。
- `m.get_mut(&k)` は、`m` が鍵 `k` を持つときその値への可変参照を `Some` で返し、持たないとき `None` を
  返す。
- `m.remove(&k)` は、`m` が鍵 `k` を持つときその鍵とその値を取り除き、持たないとき何も変えない。どちらの
  場合も、`k` 以外の鍵とその値は変わらない。
- `m.entry(k).or_default()` は、`m` が鍵 `k` を持たないとき `(k, V::default())` を加え、いずれの場合も
  `k` の値への可変参照を返す。ほかの鍵とその値は変わらず、鍵が失われることはない。
- `m.keys()` は `m` の各鍵への共有参照をちょうど 1 度ずつ渡す反復子であり、`m.keys().cloned()` は
  その各鍵の複製をちょうど 1 度ずつ渡す。順序は定めない。
- `&m` の反復と `m.iter()` はどちらも `m` の各 (鍵, 値) の対をちょうど 1 度ずつ渡す。順序は定めない。

**EXT collect into Map と Set**
`it.collect::<Map<K, V>>()` は、空の `Map` に `it` の各要素 `(k, v)` を `insert` で順に加えたものである。
よってその鍵の集合は `it` の要素の鍵の集合に等しく、鍵を共有する要素が複数あるとき、その鍵の値は最後の
要素の値である。`it.collect::<Set<K>>()` も同じく、空の `Set` に `it` の各要素を `insert` で順に加えた
ものであり、その要素の集合は `it` の要素の集合に等しい。

**EXT Clone**
`<Vec<T> as Clone>::clone` と `<[T]>::to_vec` は、元と同じ長さの新しい `Vec` を作り、その第 `i` 要素を
元の第 `i` 要素の `<T as Clone>::clone` とする。`<Set<K> as Clone>::clone` は、元の各要素の `clone` を
要素とする `Set` を返す。`<Map<K, V> as Clone>::clone` は、元の各鍵の `clone` を鍵に持ち、各鍵の値が元の
値の `clone` である `Map` を返す。`<usize as Clone>::clone`、`<bool as Clone>::clone`、
`<String as Clone>::clone` は同じ値を返す。組 `(A, B)` の `clone` は各成分の `clone` の組である。
`#[derive(Clone)]` が作る実装は、構造体については各フィールドをその型の `clone` で写した値を返し、
enum については元と同じ変位で、その変位が保持する各値をその型の `clone` で写した値を返す。

**この規則は等しさで閉じる。** 上に挙げた基底の型では `clone` は同じ値を返し、`Vec`・`Set`・`Map`・組・
`#[derive(PartialEq)]` を持つ構造体と enum の等しさは成分ごとの等しさで決まるので、成分の `clone` が元と
等しければ全体も元と等しい。**`PartialEq` を手書きで実装する型については、その実装が読む成分について
同じことを確かめる。** 次の段落が `NameSpace` についてそれを行う。

`Origin` は `Clone` と `PartialEq` を derive した enum であり、変位 `Exactly` は `VarPath` を 1 つ、変位
`Join` は `VarPath` と `Set<VarPath>` を保持する。`VarPath` は組 `(FullName, FieldPath)`、`FieldPath` は
`Vec<usize>`、`FullName` は `Clone` と `PartialEq` を derive した構造体でそのフィールドは `NameSpace` と
`String`、`NameSpace` は `Clone` を derive した構造体でそのフィールドは `Vec<String>` と `bool` である。
`NameSpace` の `PartialEq` は手書きであり、その本文は `self.names == other.names` なので、`NameSpace` の
2 つの値が等しいことは `names` が等しいことである。`clone` は `names` を `<Vec<String> as Clone>::clone`
で写すので複製の `names` は元と等しく、したがって `NameSpace` の値の `clone` は元と等しい。
よって `Origin` の値の `clone` も `VarPath` の値の `clone` も元と等しい
(`CODE src/rc_ir/ownership.rs: Origin`, `CODE src/rc_ir/ast.rs: VarPath`, `CODE src/ast/name.rs: FullName`,
`CODE src/ast/name.rs: NameSpace`)。

`PendingRetain` は `Clone` を derive し、そのフィールドは `node: NodeId` (`usize` の別名) と
`outstanding: References` である。`References` も `Clone` を derive し、そのフィールドは `Map` 1 つで
ある。よって `pending.clone()` の値は、元の `PendingRetains` と同じ長さで、第 `i` 要素の `node` は元の
第 `i` 要素の `node` と等しく、第 `i` 要素の `outstanding` は元の `outstanding` と等しい (DEF 参照の
多重集合) (`CODE src/rc_ir/borrow.rs: PendingRetain`, `CODE src/rc_ir/ownership.rs: References`)。

### DEF 基本操作

走査が `PendingRetains` の値を作る操作のうち、次の 6 つに名を与える。

| 名 | 作られ方 |
|---|---|
| 初期 | `cancel` の `cancel_body` の `analysis.walk(body, PendingRetains::default(), true)` の第 2 引数 |
| 複製 | `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の `pending.clone()` |
| 追加 | `RcExpr::Retain(v, path, _, k)` の腕の `pending.push(PendingRetain { node: retain, outstanding })` |
| 消費 | `CancelAnalysis::consume_objects` の `pending.retain(...)` |
| 引き | `RcExpr::Release(v, path, _, k)` の腕の `un_bump(&mut pending, &un_bumped)` |
| 併合 | `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の `self.merge(&pending, &arm_exits)` |

(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`, `CODE src/rc_ir/borrow.rs: un_bump`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::merge`)

走査が作る `PendingRetains` の値を**状態**と呼ぶ。各状態は、それを作る基本操作が実行される時点で生じる。
状態 `Q` が状態 `P` より**前に作られた**とは、`Q` の生じる時点が `P` の生じる時点より前であることをいい、
この関係を**生成順序**と呼ぶ。

基本操作の**入力**と**出力**を次で定める。「初期」に入力は無く、出力は `PendingRetains::default()` の値で
ある。「複製」の入力は複製される 1 つの状態、出力はその複製である。「追加」「消費」「引き」の入力は
その操作が書き換える `Vec` の書き換え前の値 1 つ、出力は書き換え後の値である。「併合」の入力は
`pending_in` と各 `arm_exits[j]` の全部、出力は `merge` の返り値である。

### DEF 除去事象

基本操作 1 つの入力の状態の集まり `Q1, ..., Qm` (DEF 基本操作) のある `Qi` に `node` が `x` である要素が
あり、その操作の出力 `Q'` にはそれが無いとき、この操作を `x` の**除去事象**と呼ぶ。

## 2. 予備の補題と P2a

### L0 (`origin` の返り値は memo に依らない)

1 つの `VarTable` の値 `vars` を固定する。以下この補題の中では、**呼び出し**も「`origin` の呼び出し」も、
`vars` を第 1 引数として行われる `origin` の呼び出しを指す。**第 2 引数は問わない。** memo の鍵は
`(var.clone(), path.to_vec())` であって `TypeEnv` を含まないので、1 つの `vars` の表は、第 2 引数が
相異なる呼び出しのあいだでも共有される。次が成り立つ。

**鍵 `(x, π)` が等しい 2 つの呼び出しがどちらも値を返すならば、その 2 つの返り値は等しい。**

したがって、1 つの `TypeEnv` の値 `type_env` を固定すると、
`ownership::acted_references(vars, type_env, v, π)` の返り値と、
`CancelAnalysis::other_objects(v, π)` が返す `Vec` の元の集合も、走査のどの時点で読んでも同じである。

**証明**

<1>0. `origin` の呼び出しがこのクレートの中に書かれているのは 17 か所であり、それを含む関数は 12 個で
      ある。12 個は、`ownership.rs` の `origin_inner` (6 か所)、`origin_from_leaves_under`
      (1 か所)、`acted_references` (1 か所)、その `#[cfg(test)] mod tests` の `origin_of` と
      `a_unit_read_out_of_a_container_keeps_the_containers_origin` (各 1 か所)、`borrow.rs` の
      `infer_ownership`、`level_ownership`、`RewriteCtx::comes_from_a_value_used_later`、
      `RewriteCtx::owns_unit`、`RewriteCtx::check_ownership_is_levelled`、`CancelAnalysis::consume`、
      `CancelAnalysis::other_objects` (各 1 か所) である。
  <2>1. `origin` の呼び出しが書かれるのは `src/` のファイルの中だけである。`origin` は `ownership.rs` の
        `pub(crate)` の自由関数なので、EXT 可視性と私有性 よりその呼び出しはそれを項目として持つ
        クレートの中にしか書けない。EXT このリポジトリのターゲット より、そのクレートは lib の
        `fixlang` と bin の `fix` の 2 つであり、どちらの項目も `src/` のファイルが宣言する。ベンチの
        ターゲットは `fixlang` を外部クレートとして読むので、`pub(crate)` の `origin` を名指せない。
    BY CODE src/rc_ir/ownership.rs: origin, EXT 可視性と私有性, EXT このリポジトリのターゲット
  <2>2. `src/` の全体を `origin(` で走査すると、`ownership.rs` の関数定義の頭を除いて 17 か所が挙がり、
        それを含む関数は上に挙げた 12 個である。DEF 本文 より、閉包の中に書かれた呼び出しは
        それを書いた関数の呼び出しとして数える --- 渡す先が標準ライブラリであっても `grow_stack` のような
        このクレートの関数であっても同じなので、この数え上げはそれを含んでいる。
    BY CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: acted_references, CODE src/rc_ir/borrow.rs: infer_ownership,
       CODE src/rc_ir/borrow.rs: level_ownership,
       CODE src/rc_ir/borrow.rs: RewriteCtx::comes_from_a_value_used_later,
       CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
       CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects, DEF 本文, DEF このクレート
  <2>3. QED
    <2>1 より呼び出しは `src/` の中にしか書けず、<2>2 がその走査を尽くす。DEF このクレート より、
    `src/` の走査が挙げた一覧は、どちらのクレートで読んでもその呼び出しの集合の上位集合である。
    BY <2>1, <2>2, DEF このクレート
<1>0a. `VarTable` のどの値 `t` についても、`t` への参照を引数 (`self` を含む) として受け取らず `t` を
       自分で作りもしない関数の本文は、`t` に届かない。したがって `origin(vars, ・, ・, ・)` の 1 回の
       呼び出しの中で、**第 1 引数が `vars` である `origin` の呼び出しが書かれている本文**は、`origin`、
       `origin_inner`、`origin_from_leaves_under` の 3 つだけである。
  <2>1. 前半が成り立つ。EXT 参照は引数を通ってだけ届く より、関数の本文が名指せる値は、自分の引数
        (`self` を含む) から到達できる値、自分が作った値、および `static` 項目の値だけである。この 3 つ目
        から `VarTable` の値には届かない --- `src/` の全体を `static` で走査して挙がるのは 4 つ
        (`src/main.rs` の `GLOBAL: MiMalloc`、`src/object.rs` の `FIELDS_BY_NAME: OnceLock<Map<FullName,
        Vec<ObjectFieldType>>>`、`src/tests/test_util.rs` の `BUILD_FIX: Once`、`src/tool/log_file.rs` の
        `LOG_FILE: Lazy<Mutex<File>>`) であり、DEF このクレート よりこの 4 つはどちらのクレートの
        `static` 項目の集合の上位集合でもある。`MiMalloc` は欄を持たず、`Once` は
        内部の状態語だけを持ち、`Lazy<Mutex<File>>` はファイルハンドルだけを持つ。`FIELDS_BY_NAME` の
        値の型は `Map<FullName, Vec<ObjectFieldType>>` であり、`FullName` は `NameSpace` と `String`、
        `ObjectFieldType` の 19 変位が保持するのは `Arc<TypeNode>`・`Vec<Arc<TypeNode>>`・`bool` だけで
        ある。どの型からも `VarTable` に到達できない。`VarTable` 自身を
        `static` に置けないことも別に出る --- `VarTable` は `origins: RefCell<Map<VarPath, Origin>>` の
        欄を持つので `Sync` ではなく、EXT static は Sync を要る がそれを禁じる。よって関数の本文が
        `VarTable` のある値に届くのは、その値への参照が引数 (`self` を含む) として渡ったときか、自分で
        その値を作ったときに限る。
    BY CODE src/rc_ir/ownership.rs: VarTable, CODE src/main.rs: GLOBAL,
       CODE src/object.rs: FIELDS_BY_NAME, CODE src/object.rs: ObjectFieldType,
       CODE src/ast/name.rs: FullName, CODE src/ast/name.rs: NameSpace,
       CODE src/tests/test_util.rs: BUILD_FIX,
       CODE src/tool/log_file.rs: LOG_FILE, EXT static は Sync を要る,
       EXT 参照は引数を通ってだけ届く, DEF このクレート
  <2>1a. `origin` の本文は `grow_stack` へ `vars` を捕捉した閉包を渡すので、`grow_stack` の本文とその中で
         走る `stacker` の本文は、`vars` に到達できる値を引数として受け取る。しかしこの 2 つのどちらにも
         `origin` の呼び出しは書かれていない --- <1>0 の 12 個にどちらも入らず、`stacker` はこのクレートの
         外なので EXT 可視性と私有性 より `pub(crate)` の `origin` を名指せない。同じことが、この文書が
         数え上げる本文が引数として渡すどの閉包についても言える --- <1>0 の 12 個の外の本文には `origin`
         の呼び出しが書かれていないからである。
    BY <1>0, CODE src/misc.rs: grow_stack, CODE src/rc_ir/ownership.rs: origin, DEF 本文,
       EXT 可視性と私有性
  <2>2. `origin` の本文が `vars` を渡すのは `origin_inner(vars, type_env, var, path)` の 1 か所だけで
        ある。ほかに `vars` が現れるのは `vars.origins.borrow()` と `vars.origins.borrow_mut()` で、
        どちらも `RefCell` の欄への参照を渡すだけである。`origin_inner` の呼び出しは `grow_stack` へ渡す
        閉包の中に書かれているが、DEF 本文 よりその閉包の本文は `origin` の本文の一部であり、A15 より
        `grow_stack` はその閉包をちょうど 1 回呼ぶ。
    BY CODE src/rc_ir/ownership.rs: origin, DEF 本文, A15
  <2>3. `origin_inner` の本文が `vars` を渡すのは、6 か所の `origin(vars, ...)` と 1 か所の
        `origin_from_leaves_under(vars, ...)` だけである。`origin_from_leaves_under` の本文が `vars` を
        渡すのは 1 か所の `origin(vars, ...)` だけである。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>4. QED
    後半を、呼び出しの入れ子の深さについての帰納法で示す (EXT 呼び出しの入れ子)。根の呼び出し
    `origin(vars, ・, ・, ・)` はこの 3 つの 1 つであり、<2>2 と <2>3 より、この 3 つの本文が `vars` を
    引数として渡す先はこの 3 つと、`vars` を捕捉した閉包を受け取る `grow_stack` だけである。<2>1 より、
    `vars` を引数として受け取らない本文は `vars` に届かない --- 自分で作った `VarTable` の値は `vars` では
    ない --- ので、その中の呼び出しの引数に `vars` は現れない。<2>1a より、`grow_stack` とその先の本文には
    `origin` の呼び出しが書かれていない。よって、第 1 引数が `vars` である `origin` の呼び出しが書かれて
    いる本文はこの 3 つだけである。
    BY <2>1, <2>1a, <2>2, <2>3, EXT 呼び出しの入れ子
<1>0b. 次の関数の返り値は引数で決まる (DEF 引数で決まる関数)。`Map::get`、`Set`・`Vec`・`Map` の操作、
       `<[T]>::starts_with`、`<[T]>::first`、`Clone::clone`、`Option` と `Iterator` の組み合わせ子、
       `LLVMGen::result_prov`、`TypeNode::is_box`、`Provenance::leaf_origins_at`、
       `Provenance::leaf_origins_under`、`as_arg_projection`、`truncate_to_unit`、`boxed_leaf_paths`、
       `Origin::identity`、`Origin::candidates`。ただし `Provenance::leaf_origins_under` と
       `Origin::candidates` については、引数で決まるのは並びではなく元の集合である。
  <2>1. 標準ライブラリの操作 --- `Map::get`、`Set`・`Vec`・`Map` の操作、`<[T]>::starts_with`、
        `<[T]>::first`、`Clone::clone`、`Option` と `Iterator` の組み合わせ子 --- については、外部の結果が
        それを述べる。`Set` の反復の順序は定めないので、`Set` から作られるのは要素の集合であって並びでは
        ない。
    BY DEF 引数で決まる関数, EXT Map と Set, EXT collect into Map と Set, EXT スライスの接頭と先頭,
       EXT Clone, EXT Vec::iter と slice::iter, EXT Iterator::all と any,
       EXT Iterator::map と collect, EXT Iterator::filter_map, EXT Iterator::rposition
  <2>2. `LLVMGen::result_prov` については A3 が述べる。A3 は「**`result_prov` と `borrows_operand` は
        決定的である** -- 同じ引数に対して常に同じ値を返す。」と書き、その果たす者を
        `impl LLVMGen for` の 78 個の通読としている。
    BY A3, DEF 引数で決まる関数
  <2>3. 残る 8 つ --- `TypeNode::is_box`、`Provenance::leaf_origins_at`、
        `Provenance::leaf_origins_under`、`as_arg_projection`、`truncate_to_unit`、`boxed_leaf_paths`、
        `Origin::identity`、`Origin::candidates` --- が引数に取るのは、型・path・`TypeEnv`・
        `Provenance`・`Set<LeafOrigin>`・`Origin` の値だけであり、`VarTable` も走査の状態も引数に
        取らない。
    BY CODE src/ast/types.rs: TypeNode::is_box,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/ownership.rs: as_arg_projection,
       CODE src/rc_ir/ownership.rs: truncate_to_unit, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/rc_ir/ownership.rs: Origin::identity, CODE src/rc_ir/ownership.rs: Origin::candidates
  <2>4. <2>3 の 8 つのうち 6 つ --- `TypeNode::is_box`、`Provenance::leaf_origins_at`、
        `Provenance::leaf_origins_under`、`as_arg_projection`、`Origin::identity`、
        `Origin::candidates` --- は内部可変性を持つ値に触れない。その本文は引数から到達できる値だけを
        読み、可変な静的変数にも触れない。`is_box` は `is_unbox` を経て `toplevel_tycon_info` の
        `type_env.tycons().get(&tycon)` に落ちるが、その鍵の型 `TyCon` は `FullName` の欄を 1 つ持つだけ
        であり、`FullName` と `NameSpace` の手書きの `Hash` が読むのは `Vec<String>` と `String` だけ
        なので、内部可変性を持たない。残る 5 つは引数の `Provenance`・`Set<LeafOrigin>`・`Origin` を
        読むだけである。よってこの 6 つの返り値は引数の値で決まる。**そのうち 2 つは、並びではなく元の
        集合が決まる。** `Provenance::leaf_origins_under` の doc は、渡す要素を `in no particular order`
        と述べるので、引数で決まるのは渡す要素の集合である。`Origin::candidates` は `Join` の変位に
        ついて `Set` の反復から `Vec` を作り、EXT Map と Set は `Set` の反復の順序を定めないので、
        引数で決まるのはその元の集合である。
    BY <2>3, EXT Map と Set, CODE src/ast/types.rs: TypeNode::is_box,
       CODE src/ast/types.rs: TypeNode::is_unbox,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info, CODE src/ast/types.rs: TyCon,
       CODE src/ast/name.rs: FullName, CODE src/ast/name.rs: NameSpace,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/ownership.rs: as_arg_projection, CODE src/rc_ir/ownership.rs: Origin::identity,
       CODE src/rc_ir/ownership.rs: Origin::candidates, DEF 引数で決まる関数
  <2>5. 残る 2 つ --- `boxed_leaf_paths` と `truncate_to_unit` --- は触れる。この 2 つの呼び出しの中で
        走る関数は、`boxed_leaf_paths` の内部関数 `go`、`is_fully_unboxed`、`is_box`、`is_unbox`、
        `is_closure`、`is_array`、`is_funptr`、`is_punched_array`、`toplevel_tycon_satisfies`、
        `toplevel_tycon`、`toplevel_tycon_info`、`unpunched_field_types`、`instance_field_types`、
        `declared_field_types`、`collect_type_arguments`、`unwrap_newtypes_memoized`、
        `unwrap_newtypes_node`、`type_node_eq`、`TypeEnv::tycons`、`TypeEnv::unwrapped_newtype_info`、
        `make_unit_ty`、`Substitution::default`・`single`・`merge`・`substitute_type`、`get_source`、
        `set_source_if_none`、`set_source`、`set_tyapp_fun`、`set_tyapp_arg`、`set_assocty_args`、
        `unit_step`、`held_field_type`、および標準ライブラリの操作で尽きる (`truncate_to_unit` は
        `unit_step` と `held_field_type` から `unpunched_field_types` を経て同じ道に入る)。
        **このうち共有参照から書ける欄に触れるのは、`OnceLock` の `get_or_init` を呼ぶ 3 つのメソッド ---
        `TypeNode::type_hash`、`TypeNode::is_ground`、`TypeNode::depth` --- だけである。** A3 は
        「**その 3 つを共有参照から埋めるのは、`<欄>.get_or_init` を呼ぶメソッドである**」と書き、
        「**在りかは `_cache.get_or_init` の全出現で決める**」と述べる。`src/` の全体を
        `get_or_init` で走査すると 4 か所が挙がり、うち 3 か所がこの 3 つのメソッド、残る 1 か所は
        `src/object.rs` の `static FIELDS_BY_NAME` である。この道はその `static` に触れない。
    <3>1. `unwrap_newtypes_memoized` の memo `unwrapped` は、呼び出しを跨いで持ち越されない。その型は
          `Map<Arc<TypeNode>, Arc<TypeNode>>` であり、鍵を引くたびに `impl Hash for TypeNode` が
          `TypeNode::type_hash` を呼んで `hash_cache` の `OnceLock` を共有参照から書くが、`unwrapped`
          自身は `instance_field_types` の 1 回の呼び出しごとに `Map::default()` で空から作られる。
      BY CODE src/ast/types.rs: TypeNode::instance_field_types,
         CODE src/ast/types.rs: TypeNode::unwrap_newtypes_memoized,
         CODE src/ast/types.rs: TypeNode::type_hash, CODE src/ast/types.rs: TypeNode, EXT Map と Set
    <3>2. `set_source_if_none` は既に在る `TypeNode` の欄を書かない。その本文は、自分の `info.source` が
          `None` のとき `set_source` を呼び、そうでないとき `self.clone()` を返す。`set_source` は
          `self.clone()` で新しい `TypeNode` を作ってからその `info.source` を書き、`Arc::new` で包んで
          返す。`impl Clone for TypeNode` は 3 つの `OnceLock` を `OnceLock::new()` で空にするので、
          この複製は原本の memo も引き継がない。
      BY CODE src/ast/types.rs: TypeNode::set_source_if_none, CODE src/ast/types.rs: TypeNode::set_source,
         CODE src/ast/types.rs: TypeNode::get_source, CODE src/ast/types.rs: TypeNode
    <3>3. `toplevel_tycon_info` は `type_env` の表を読むだけで、何も書かない。その本文は
          `type_env.tycons().get(&tycon).unwrap()` であり、`TypeEnv::tycons` は表への共有参照を返す。
          返る `TyConInfo` の 8 欄は `Arc<Kind>`、`TyConVariant`、`bool`、`Vec<Arc<TyVar>>`、
          `Vec<Field>`、`Option<Span>`、`Option<String>`、`Option<TyCon>` であり、`get_or_init` を
          呼ぶ欄を直接には持たない。`Field` の `ty` を通じて `TypeNode` の 3 つの `OnceLock` には届くが、
          それは上の 3 つのメソッドを経る道である。
      BY CODE src/ast/types.rs: TypeNode::toplevel_tycon_info, CODE src/ast/program.rs: TypeEnv::tycons,
         CODE src/ast/types.rs: TyConInfo, CODE src/ast/typedecl.rs: Field
    <3>4. QED
      <3>1・<3>2・<3>3 が、この道で共有参照から書かれうる欄を尽くす。残るのは A3 が名指す 3 つの
      `OnceLock` である。
      BY <3>1, <3>2, <3>3, A3, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
         CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
         CODE src/ast/types.rs: TypeNode::unpunched_field_types,
         CODE src/ast/types.rs: TypeNode::instance_field_types,
         CODE src/ast/types.rs: TypeNode::declared_field_types,
         CODE src/elaboration/typecheck.rs: Substitution::substitute_type,
         CODE src/ast/types.rs: TypeNode::is_ground, CODE src/ast/types.rs: TypeNode::depth,
         CODE src/ast/types.rs: TypeNode::type_hash, CODE src/object.rs: FIELDS_BY_NAME,
         CODE src/rc_ir/ownership.rs: truncate_to_unit, CODE src/rc_ir/ownership.rs: unit_step,
         CODE src/rc_ir/ownership.rs: held_field_type, DEF このクレート
  <2>6. `TypeNode` の 3 つの `OnceLock` の欄が埋まっても、`Arc<TypeNode>` の値の等しさも、そのハッシュも
        動かない。**この節は `type_env` を通じてだけ現れる `TypeNode` にも当たる。**
    <3>1. A3 は「**その 3 つは一度だけ書かれる memo であり、`impl PartialEq for TypeNode` は `ty` だけを
          読み、3 つの memo の値はどれも `ty` の関数である**」と述べ、「**`impl Hash for TypeNode` は
          `type_hash` を呼ぶので `hash_cache` を読み、かつ書く。**」「反映されるのは `ty` だけなので、
          等しい 2 つの値は等しくハッシュされる。」と続ける。
      BY A3
    <3>2. <3>1 の 3 文は、`TypeNode` という**型**と、その型が持つ 3 つの欄と 2 つの実装についての言明で
          ある。主語は `TypeNode` の値がどこから到達できるかを条件にしていないので、`RcProgram` から
          到達できない `TypeNode` の値 --- `type_env` の `TyConInfo` が持つ欄の型と、そこから
          `declared_field_types` の代入が作る型 --- にも当たる。
      BY <3>1, A3, CODE src/ast/types.rs: TypeNode, CODE src/ast/types.rs: TypeNode::instance_field_types
    <3>3. QED
      <3>1 の 3 文より、3 つの欄の値は `ty` の関数であり、`PartialEq` は `ty` だけを読み、`Hash` が
      反映するのも `ty` だけである。よって欄が埋まっても値の等しさもハッシュも動かない。<3>2 より
      これは `type_env` を通じてだけ現れる値にも当たる。
      BY <3>1, <3>2
  <2>7. `unwrap_newtypes_node` の `Type::TyApp` の腕は `Arc::ptr_eq` を読むが、その真偽は返る `Arc` の
        同一性を変えるだけで、返る値を変えない。
    <3>1. その腕は、`Arc::ptr_eq(&new_fun_ty, fun_ty) && Arc::ptr_eq(&new_arg_ty, arg_ty)` が真のとき
          `self.clone()` を返し、偽のとき `self.set_tyapp_fun(new_fun_ty).set_tyapp_arg(new_arg_ty)` を
          返す。`self.clone()` は `self` と同じ割り当てのハンドルであり、その `ty` は
          `Type::TyApp(fun_ty, arg_ty)` である。`set_tyapp_fun` と `set_tyapp_arg` は `self` の複製の
          `ty` の成分を差し替えるので、後者の `ty` は `Type::TyApp(new_fun_ty, new_arg_ty)` である。
      BY CODE src/ast/types.rs: TypeNode::unwrap_newtypes_node, CODE src/ast/types.rs: Type,
         CODE src/ast/types.rs: TypeNode::set_tyapp_fun, CODE src/ast/types.rs: TypeNode::set_tyapp_arg
    <3>2. `impl PartialEq for Type` の `TyApp` の腕は、2 つの成分に `type_node_eq` を掛ける。
          `type_node_eq(lhs, rhs)` の本文は `Arc::ptr_eq(lhs, rhs) || lhs.ty == rhs.ty` であり、
          `Arc::ptr_eq` が真ならば 2 つは同じ値なので `lhs.ty == rhs.ty` も真である。よって
          `type_node_eq(lhs, rhs)` が真であることと `lhs.ty == rhs.ty` は同値であり、`Type` の等しさは
          値で決まる。
      BY CODE src/ast/types.rs: Type, CODE src/ast/types.rs: type_node_eq
    <3>3. QED
      `Arc::ptr_eq(&new_fun_ty, fun_ty)` が真のとき `new_fun_ty` と `fun_ty` は同じ割り当てのハンドル
      なので値が等しく、`new_arg_ty` と `arg_ty` についても同じである。よって <3>1 の 2 つの枝が返す値の
      `ty` は、<3>2 の意味でどちらも `Type::TyApp(new_fun_ty, new_arg_ty)` と等しい。`TypeNode` の
      `PartialEq` は `ty` だけを読むので (<2>6)、2 つの枝は等しい値を返す。
      BY <3>1, <3>2, <2>6, CODE src/ast/types.rs: TypeNode
  <2>7a. `declared_field_types` は `Substitution::substitute_type` を呼ぶ。その本文は `Arc::ptr_eq` を
         2 か所で読むが、どちらの読みも返る値を変えず、返り値の値は引数の値で決まる。
    <3>1. `declared_field_types` の本文は、`collect_type_arguments` の結果を `tycon_info.tyvars` に
          対応させた `Substitution` を `Substitution::default`・`single`・`merge` で組み立て、
          `tycon_info.fields` の各 `field` について `subst.substitute_type(&field.ty)` を並べて返す。
          `Substitution` の欄は `data: Map<Name, Arc<TypeNode>>` の 1 つであり、`single` はその表に
          1 対を入れ、`merge` は `other` の各対について自分が持つ値と `PartialEq` で比べてから入れる。
      BY CODE src/ast/types.rs: TypeNode::declared_field_types,
         CODE src/elaboration/typecheck.rs: Substitution,
         CODE src/elaboration/typecheck.rs: Substitution::single,
         CODE src/elaboration/typecheck.rs: Substitution::merge,
         CODE src/elaboration/typecheck.rs: Substitution::substitute_type
    <3>2. `substitute_type` の本文は `ty.ty` についての 4 つの腕である。`Type::TyVar(tyvar)` の腕は
          `self.data.get(&tyvar.name)` を引き、`None` なら `ty.clone()`、`Some(sub)` なら
          `sub.set_source_if_none(ty.get_source().clone())` を返す。`Type::TyCon(_)` の腕は
          `ty.clone()` を返す。`Type::TyApp(fun, arg)` の腕は 2 つの成分に自分を再帰させ、
          `Arc::ptr_eq(&new_fun, fun) && Arc::ptr_eq(&new_arg, arg)` が真のとき `ty.clone()`、偽のとき
          `ty.set_tyapp_fun(new_fun).set_tyapp_arg(new_arg)` を返す。`Type::AssocTy(_, args)` の腕は
          各引数に自分を再帰させ、すべての対に `Arc::ptr_eq` が真のとき `ty.clone()`、偽のとき
          `ty.set_assocty_args(new_args)` を返す。`Arc::ptr_eq` の読みはこの 2 か所だけである。
      BY CODE src/elaboration/typecheck.rs: Substitution::substitute_type,
         CODE src/ast/types.rs: Type, CODE src/ast/types.rs: TypeNode::set_source_if_none,
         CODE src/ast/types.rs: TypeNode::set_tyapp_fun, CODE src/ast/types.rs: TypeNode::set_tyapp_arg,
         CODE src/ast/types.rs: TypeNode::set_assocty_args
    <3>3. `Type::TyApp` の腕の 2 つの枝は、条件が真のとき等しい値を返す。条件が真であるとき、
          `new_fun` と `fun` は同じ割り当てのハンドルなので同じ値であり、`new_arg` と `arg` についても
          同じである。真の枝が返す `ty.clone()` の `ty` は `Type::TyApp(fun, arg)`、偽の枝が返す値の
          `ty` は `Type::TyApp(new_fun, new_arg)` である。`impl PartialEq for Type` の `TyApp` の腕は
          2 つの成分に `type_node_eq` を掛け、`type_node_eq(lhs, rhs)` の本文は
          `Arc::ptr_eq(lhs, rhs) || lhs.ty == rhs.ty` なので、この 2 つは等しい。
          `impl PartialEq for TypeNode` は `ty` だけを読む (<2>6)。`Type::AssocTy` の腕も同じ形で
          あり、`impl PartialEq for Type` の `AssocTy` の腕は名前の一致と各引数の `type_node_eq` である。
      BY <3>2, <2>6, CODE src/ast/types.rs: Type, CODE src/ast/types.rs: type_node_eq,
         CODE src/ast/types.rs: TypeNode, EXT Arc の契約
    <3>4. QED
      <3>3 より、`Arc::ptr_eq` の 2 つの読みはどちらも、返る値を選び替えるだけで値を変えない。残る腕が
      読むのは `data` の引き (鍵は `Name` すなわち `String`)、`get_source`、`set_source_if_none` である。
      `get_source` は `self.info.source` への共有参照を返し、`set_source_if_none` は `set_source` を
      経て `self.clone()` の `info.source` を書いた新しいノードを返すか `self.clone()` を返すかであり、
      どちらも既存のノードの欄を書かない。`set_tyapp_fun`・`set_tyapp_arg`・`set_assocty_args` は
      `self.clone()` の `ty` の成分を差し替えて `Arc::new` で包む。よって `substitute_type` の返り値の
      `ty` は、その再帰の上の帰納で引数の値から決まり、<2>6 より `TypeNode` の値の等しさは `ty` で
      決まる。<3>1 より、`declared_field_types` が組み立てる `Substitution` も引数の値で決まる。
      BY <3>1, <3>2, <3>3, <2>6, DEF 引数で決まる関数, EXT Map と Set,
         CODE src/elaboration/typecheck.rs: Substitution::substitute_type,
         CODE src/ast/types.rs: TypeNode::set_source_if_none, CODE src/ast/types.rs: TypeNode::set_source,
         CODE src/ast/types.rs: TypeNode::get_source, CODE src/ast/types.rs: TypeNode
  <2>8. QED
    <2>5 の 2 つの返り値は引数の値で決まる。`unwrap_newtypes_memoized` の再帰について、その返り値の値と
    それが `unwrapped` に加える対の値が、引数の値と `unwrapped` の値で決まることを、その再帰の上の
    帰納で示す --- 再帰が停止することは A10 の「さらに、到達する各型について `instance_field_types` が
    行う newtype の展開 (`unwrap_newtypes_memoized`) は abort せず停止する。」が与える。memo の引きは
    `Hash` と `PartialEq` で行われ、<2>6 よりその 2 つは値で決まる。`unwrap_newtypes_node` の腕のうち
    `Arc::ptr_eq` を読むのは `Type::TyApp` の腕だけであり、<2>7 よりその読みは返る値を変えない。残る腕が
    読むのは `toplevel_tycon`・`TypeEnv::unwrapped_newtype_info`・`collect_type_arguments`・
    `declared_field_types` と、そこから得た `TyConInfo` の欄、および引数を取らない `make_unit_ty` で
    あって、`declared_field_types` は <2>7a より引数の値で決まり、残りも引数の値から決まる。よって
    `instance_field_types` の返り値の値は、`unwrapped` を空から作るので引数の値で決まり、
    `unpunched_field_types`・`is_fully_unboxed`・`unit_step`・`truncate_to_unit`・`boxed_leaf_paths` の
    返り値もそれぞれの再帰の上の帰納で引数の値から決まる。

    **`Arc` の同一性を読む場所は、クレート全体を `Arc::ptr_eq` で走査して決める。** `src/` の全体に
    `Arc::ptr_eq` を含む行は 10 行あり、うち 1 行は `src/elaboration/typecheck.rs` のコメントである。
    残る 9 行のうち 3 行はテスト (`src/elaboration/typecheck.rs` の `#[cfg(test)] mod tests` に 1 行、
    `src/tests/test_type_node_identity.rs` に 2 行) にあり、<2>5 が挙げた関数の中には無い。
    製品のコードの 6 行のうち、<2>5 が挙げた関数の中に在るのは 4 行である ---
    `unwrap_newtypes_node` の 1 行 (<2>7)、`type_node_eq` の 1 行、
    `substitute_type` の 2 行 (<2>7a) である。`type_node_eq` の本文は
    `Arc::ptr_eq(lhs, rhs) || lhs.ty == rhs.ty` であり、`Arc::ptr_eq` が真ならば 2 つは同じ割り当ての
    ハンドルなので `lhs.ty == rhs.ty` も真である。よってこの読みは `PartialEq` の答えを変えない。
    残る 2 行は
    `src/elaboration/desugar_opaque.rs` の `resolve_opaque_type_in_type` と
    `src/elaboration/typecheck.rs` の `TypeCheckContext::unify` にあり、どちらも <2>5 が挙げた関数では
    ない。`leaf_map.rs`・`ownership.rs`・`provenance.rs` には 1 行も無い。
    <2>1、<2>2、<2>4 と合わせて、挙げた関数はすべて引数で決まる。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>7, <2>7a, A10, DEF 引数で決まる関数, DEF このクレート,
       CODE src/ast/types.rs: TypeNode::unwrap_newtypes_memoized,
       CODE src/ast/types.rs: TypeNode::unwrap_newtypes_node,
       CODE src/ast/types.rs: TypeNode::instance_field_types,
       CODE src/ast/types.rs: TypeNode::declared_field_types,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types,
       CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/ast/types.rs: TypeNode::toplevel_tycon,
       CODE src/ast/types.rs: TypeNode::collect_type_arguments,
       CODE src/ast/program.rs: TypeEnv::unwrapped_newtype_info,
       CODE src/elaboration/typecheck.rs: Substitution::substitute_type,
       CODE src/elaboration/typecheck.rs: TypeCheckContext::unify,
       CODE src/elaboration/desugar_opaque.rs: resolve_opaque_type_in_type,
       CODE src/rc_ir/ownership.rs: unit_step,
       CODE src/rc_ir/ownership.rs: truncate_to_unit, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/ast/types.rs: type_node_eq
<1>1. `origin(vars, type_env, x, π)` の本文は 4 つの文と末尾式である。
      `let key = (x.clone(), π.to_vec());`、
      `if let Some(known) = vars.origins.borrow().get(&key) { return known.clone(); }`、
      `let answer = grow_stack(|| origin_inner(vars, type_env, x, π));`、
      `vars.origins.borrow_mut().insert(key, answer.clone());`、そして末尾式 `answer` である。
      EXT 文は書かれた順に実行される より、この 4 つの文はこの順に実行され、末尾式はその後に評価される。
      第 2 の文で返る呼び出しを**当たり**、第 2 の文で返らずに第 3 の文へ進む呼び出しを**外れ**と呼ぶ。
      この 2 つは呼び出しを尽くす。当たりの呼び出しは `vars.origins` の鍵 `key` の値の
      複製を返し、`origin` も `origin_inner` も呼ばない。外れの呼び出しは A15 より `origin_inner` を
      ちょうど 1 回呼び、その値を鍵 `key` に `insert` してから返す。`key` は `(x.clone(), π.to_vec())`
      であり、`insert` に渡るのは `answer.clone()` で、当たりが返すのは `known.clone()` である。
      EXT Clone より、`key` は `(x, π)` と等しく、`insert` に渡る値は返る値と等しく、当たりが返す値は
      表が持つ値と等しい。
  BY CODE src/rc_ir/ownership.rs: origin, A15, EXT Map と Set, EXT Clone,
     EXT 文は書かれた順に実行される
<1>2. `vars.origins` の鍵の集合は増えるだけであり、鍵 `k` が入るのは、鍵が `k` である外れの呼び出しが
      `origin_inner` から戻った後に限る。`origins` は `VarTable` の非公開の欄であり、`ownership.rs` は
      `mod` 宣言を `#[cfg(test)] mod tests` の 1 つしか持たないので、この欄を名指す式が書けるのは
      `ownership.rs` の中だけである。そこで --- その `#[cfg(test)] mod tests` を含めて --- この欄に
      触れるのは `VarTable::empty` の `RefCell::default()` と <1>1 の `get` と `insert` の 3 か所だけで
      ある。EXT Map と Set より `insert` は鍵を失わせない。
  BY CODE src/rc_ir/ownership.rs: VarTable, CODE src/rc_ir/ownership.rs: VarTable::empty,
     CODE src/rc_ir/ownership.rs: origin, <1>1, EXT Map と Set, EXT 可視性と私有性,
     EXT モジュールは `mod` が導入する
<1>2a. `vars` を第 1 引数とする `origin` の呼び出しが起きるどの時点でも、`vars.bindings` は同じ値で
       ある。すなわち鍵の集合が同じであり、各鍵の `Binding` の変位が同じで、その変位が保持する値が
       等しく、`Binding::Llvm` の `Box<dyn LLVMGen>` は同じ引数に同じ宣言を返す。
  <2>1. `bindings` は `VarTable` の非公開の欄であり、`ownership.rs` は `mod` 宣言を
        `#[cfg(test)] mod tests` の 1 つしか持たないので、この欄を名指す式が書けるのは `ownership.rs`
        とそのモジュールの中だけである。そこでこの欄に触れる関数は 5 つだけである。書き手は 4 つ ---
        `VarTable::empty` の `Map::default()`、`VarTable::of` の `vars.bindings.insert`、
        `collect_bindings` の 3 つの `vars.bindings.insert`、`#[cfg(test)] mod tests` の `table` の
        `vars.bindings.insert` --- であり、読み手は `origin_inner` の `vars.bindings.get(var)` 1 つで
        ある。`ownership.rs` の外で `VarTable` を名指すのは `borrow.rs` だけであり、欄が非公開なので
        そこからは触れない。
    BY CODE src/rc_ir/ownership.rs: VarTable, CODE src/rc_ir/ownership.rs: VarTable::empty,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: collect_bindings,
       CODE src/rc_ir/ownership.rs: origin_inner, EXT 可視性と私有性,
       EXT モジュールは `mod` が導入する
  <2>2. `bindings` への書き込みは、`VarTable::of`、`VarTable::body_only`、または
        `#[cfg(test)] mod tests` の `table` の 1 回の呼び出しの実行区間の中でだけ起きる。<2>1 の 4 つの
        書き手のうち `VarTable::empty` と `collect_bindings` は `ownership.rs` の非公開の項目であり、
        `ownership.rs` の `mod` 宣言は `#[cfg(test)] mod tests` の 1 つだけなので、その呼び出しは
        `ownership.rs` とそのモジュールの中にしか書けない。そこで `VarTable::empty` を呼ぶのはこの
        3 つだけ、`collect_bindings` を呼ぶのはこの 3 つのうち `VarTable::of` と `VarTable::body_only`、
        および `collect_bindings` 自身だけである。
    BY <2>1, CODE src/rc_ir/ownership.rs: VarTable::empty, CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only,
       CODE src/rc_ir/ownership.rs: collect_bindings, EXT 呼び出しの入れ子, DEF 本文,
       EXT 可視性と私有性, EXT モジュールは `mod` が導入する
  <2>3. `VarTable` の値が作られるのは `VarTable::empty` の 1 か所だけなので、どの `VarTable` の値も
        <2>2 の 3 つのいずれかの 1 回の呼び出しの中で作られる。その呼び出しの中で、その表を第 1 引数と
        する `origin` の呼び出しは起きない --- <1>0a の前半より、その表に届く本文はそれを作った
        `VarTable::of` (または `VarTable::body_only`、`table`) と、その表を引数として渡された
        `collect_bindings` だけであり、<1>0 よりそのどれにも `origin` の呼び出しは書かれていない。
    BY <1>0, <1>0a, <2>1, <2>2, CODE src/rc_ir/ownership.rs: VarTable::empty,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only,
       CODE src/rc_ir/ownership.rs: collect_bindings
  <2>3a. `bindings` が保持する値の等しさは、それを共有参照で受け取る計算が変えない。また
         `Binding::Llvm` の `Box<dyn LLVMGen>` は、同じ引数に同じ宣言を返す。
    <3>1. `bindings` が保持するのは、`Box<dyn LLVMGen>`、`RcVar`、`Vec<RcVar>`、`Arc<TypeNode>`、
          `usize`、`Option<usize>` の値である。`Binding` の 7 変位のうち `Param` と `Producer` は何も
          保持せず、`Move` は `RcVar`、`Llvm` は `Box<dyn LLVMGen>` と `Vec<RcVar>` と `Arc<TypeNode>`、
          `Field` は `RcVar` と `usize`、`Payload` は `RcVar` と `Option<usize>`、`Join` は
          `Vec<RcVar>` を保持する。この 6 つの型はどれも、D1 の `RcProgram` の 3 つの欄から辿って現れる ---
          `Box<dyn LLVMGen>` と `Vec<RcVar>` は `RcRhs::Llvm` の 2 つの成分の型、`RcVar` は `RcRhs::Var`
          の成分と `Let` の束縛変数の型、`Arc<TypeNode>` は `RcVar` の `ty` の型、`usize` と
          `Option<usize>` は `Destructure` のフィールド添字と `MatchArm` の `tag` の型である。
          `VarTable::of` と `collect_bindings` が `bindings` に入れるのは、その位置から複製した値である。
      BY CODE src/rc_ir/ownership.rs: Binding, CODE src/rc_ir/ownership.rs: VarTable::of,
         CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ast.rs: RcRhs,
         CODE src/rc_ir/ast.rs: RcVar, CODE src/rc_ir/ast.rs: MatchArm, CODE src/rc_ir/ast.rs: RcExpr, D1
    <3>2. A3 の「**`RcProgram` から到達できる値の等しさは、それを共有参照で受け取る計算が変えない。**」の
          節が、この 6 つの型の値に当たる。その節は「到達できる型が内部可変性を持つ欄を持つときは、その欄は
          **一度だけ書かれる memo であって、その値はその型の `PartialEq` が読む成分の関数である**」と、
          型についての言明として述べるので、その型の値である複製にも当たる。よって、その欄が埋まっても
          `bindings` が保持する値の等しさは動かない。
      BY <3>1, A3
    <3>3. QED
      A3 の「**`result_prov` と `borrows_operand` は決定的である** -- 同じ引数に対して常に同じ値を返す。」
      より、`Binding::Llvm` の `Box<dyn LLVMGen>` は同じ引数に同じ宣言を返す。
      BY <3>1, <3>2, A3
  <2>4. QED
    <2>3 より、`vars` を第 1 引数とする `origin` の呼び出しはどれも、`vars` を作った <2>2 の呼び出しが
    返った後に起きる。<2>2 よりその後 `vars.bindings` への書き込みは無いので、鍵の集合と、各鍵の
    `Binding` の変位と、その変位が保持する値は動かない。<2>3a より、保持する値の等しさも、
    `Box<dyn LLVMGen>` が同じ引数に返す宣言も動かない。
    BY <2>2, <2>3, <2>3a
<1>2b. `origin(vars, τ, ・, ・)` の 1 回の呼び出しの中で起きる呼び出しは、どれも第 2 引数が `τ` である。
  <2>1. `origin` の本文は自分の第 2 引数を `origin_inner(vars, type_env, var, path)` の第 2 引数として
        渡す。`origin_inner` の本文に書かれた 6 か所の `origin(vars, type_env, ...)` と 1 か所の
        `origin_from_leaves_under(vars, type_env, ...)`、および `origin_from_leaves_under` の本文に
        書かれた 1 か所の `origin(vars, type_env, ...)` も、どれも自分の第 2 引数をそのまま第 2 引数と
        して渡す。
    BY CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>2. QED
    第 1 引数が `vars` である `origin` の呼び出しは、`vars` を引数として受け取った本文の中に書かれた
    ものである。<1>0a より、この呼び出しの中でそのような本文は `origin`、`origin_inner`、
    `origin_from_leaves_under` の 3 つに限り、<2>1 よりこの 3 つはどれも、自分が受け取った第 2 引数を
    そのまま渡す。この呼び出し自身の第 2 引数は `τ` なので、呼び出しの入れ子の深さについての帰納法に
    より、その中で起きる呼び出しはどれも第 2 引数が `τ` である。
    BY <1>0a, <2>1, EXT 呼び出しの入れ子
<1>3. `origin_inner(vars, type_env, x, π)` の 1 回の呼び出しが直に行う、第 1 引数が `vars` である
      `origin` の呼び出しの鍵の集合は、`vars.bindings`、`type_env`、`(x, π)` だけで決まる。とくにこの
      集合は `vars.origins` の状態にも、その呼び出しが受け取る `origin` の返り値にも依らない。
  <2>0. `origin_inner(vars, type_env, x, π)` の 1 回の呼び出しの中で、第 1 引数が `vars` である `origin`
        の呼び出しが直に (別の `origin` の呼び出しの中でなく) 起きるのは、`origin_inner` の本文に書かれた
        6 か所と、それが呼ぶ `origin_from_leaves_under` の本文に書かれた 1 か所を通ってだけである。
    <3>1. 第 1 引数が `vars` である `origin` の呼び出しは、`vars` を引数として受け取った本文の中に
          書かれたものである。<1>0a より、その本文は `origin`、`origin_inner`、
          `origin_from_leaves_under` の 3 つに限る。
      BY <1>0a
    <3>2. <1>0 の 17 か所のうち、この 3 つの本文に在るのは `origin_inner` の 6 か所と
          `origin_from_leaves_under` の 1 か所である。`origin` の本文に `origin` の呼び出しは書かれて
          いない。
      BY <1>0
    <3>3. `origin_from_leaves_under` は `ownership.rs` の非公開の自由関数であり、`ownership.rs` は
          `mod` 宣言を `#[cfg(test)] mod tests` の 1 つしか持たないので、その呼び出しはこのファイルと
          そのモジュールの中にしか書けない。そこでそれを呼ぶのは `origin_inner` の 1 か所と
          `#[cfg(test)] mod tests` の 2 か所だけである。
      BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: origin_inner, DEF 本文, EXT 可視性と私有性,
         EXT モジュールは `mod` が導入する
    <3>4. QED
      <3>1 と <3>2 より、第 1 引数が `vars` である `origin` の呼び出しはこの 7 か所を通ってだけ起きる。
      <3>3 より、この `origin_inner` の呼び出しの中で走る `origin_from_leaves_under` は、その本文が
      呼んだものである。
      BY <3>1, <3>2, <3>3
  <2>1. 本文は `vars.bindings.get(x)` による場合分けである。`None`、`Binding::Param`、
        `Binding::Producer` の腕、`Binding::Field(container, idx)` の `container` が boxed の枝、
        `Binding::Payload(scrut, Some(_))` の `scrut` が boxed の枝は、いずれも `here()` を返して
        `origin` を呼ばない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. `Binding::Move(y)` の腕は鍵 `(y.name, π)` について、`Binding::Join(arm_results)` の腕は
        各 `arm_result` について鍵 `(arm_result.name, π)` について、`Binding::Field(container, idx)` の
        unbox の枝は鍵 `(container.name, [idx] ++ π)` について、`Binding::Payload(scrut, None)` の枝は
        鍵 `(scrut.name, π)` について、`Binding::Payload(scrut, Some(tag))` の unbox の枝は鍵
        `(scrut.name, [tag] ++ π)` について、`origin` を呼ぶ。どの鍵も `vars.bindings.get(x)` が返した
        束縛と `π` から作られ、`origin` の返り値を読まない。`Binding::Join` の腕は各 `arm_result` に
        ついて必ず 1 回呼び、返り値は `candidates` を積むのに使うだけである。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `Binding::Llvm(gen, args, result_ty)` の腕は、`args` の各要素の `ty` を並べた `arg_tys` と
        `decl = gen.result_prov(result_ty, &arg_tys, type_env)` を作る。`decl.leaf_origins_at(π)` に
        `as_arg_projection` を掛けた結果が `Some((j, p))` のときは鍵 `(args[j].name, p)` について
        `origin` を 1 回呼び、`None` のときは `origin_from_leaves_under(vars, type_env, &decl, args, π, ・)`
        を呼ぶ。後者は `decl.leaf_origins_under(π)` と `args` と `type_env` から `Set` の値
        `operand_units` を作り、その各元 `(j, unit)` について鍵 `(args[j].name, unit)` の `origin` を
        1 回呼ぶ。<1>0b より、`decl` も `operand_units` も `vars.bindings.get(x)` が返した束縛と
        `type_env` と `π` で決まる --- `decl` については `LLVMGen::result_prov` の行が、`operand_units`
        については `arg_tys`・`leaf_origins_at`・`leaf_origins_under`・`as_arg_projection`・
        `truncate_to_unit` の行が与える。`operand_units` は `Set` なので反復の順序は定まらないが、呼ぶ
        鍵の集合は定まる。どの鍵も `origin` の返り値を読まない。
    BY <1>0b, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: as_arg_projection, DEF 引数で決まる関数
  <2>4. QED
    <1>0b より `vars.bindings.get(x)` の値は `vars.bindings` と `x` で決まる。`Binding` の
    変位は `Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join` の 7 つであり、<2>1 から
    <2>3 がこの 7 つと、`get` が `None` を返す場合を尽くす。<2>0 より、第 1 引数が `vars` であって直に
    起きる `origin` の呼び出しは、この 2 つの本文に書かれたものだけである。
    BY <1>0b, <2>0, <2>1, <2>2, <2>3, CODE src/rc_ir/ownership.rs: Binding, DEF 引数で決まる関数
<1>4. 値を返す `origin` の呼び出し `c` について、`c` の中には、鍵が等しく一方が他方に真に含まれる 2 つの
      外れの `origin` の呼び出しは無い。ここで外側の候補には `c` 自身も数える。この 2 つ組を**入れ子の対**
      と呼ぶ。
  <2>1. `c` の中に入れ子の対が在ると仮定する。
    BY 背理法の仮定
  <2>2. EXT 呼び出しの入れ子 より `c` の中で始まる呼び出しは有限個なので、`c` の中の入れ子の対も有限個で
        ある。<2>1 よりそれは空でないので、外側の呼び出しが始まる時刻が最も遅い対を取れる。それを
        `(a, b)` とし、その鍵を `k` とする。
    BY <2>1, EXT 呼び出しの入れ子
  <2>3. `a` から `b` へ至る `origin` の呼び出しの鎖 `a = d_0, d_1, ..., d_m = b` (`m ≥ 1`) が在る。
        EXT 呼び出しの入れ子 より、`b` を含み `a` に含まれる `origin` の呼び出しは包含について線形に
        並ぶので、それを外側から並べたものがこの鎖であり、各 `d_{i+1}` は `d_i` の中で、間に別の
        `origin` の呼び出しを挟まずに始まる。<1>1 より当たりの呼び出しは `origin` を呼ばないので、
        `origin` の呼び出しを中に持つ `d_0` から `d_{m-1}` までは外れである。`d_m = b` も入れ子の対の
        定義より外れである。よって `d_1` は外れであり (`m = 1` なら `d_1 = b`、`m > 1` なら
        `1 ≤ m - 1`)、`a` の中で始まるので `a` より始まる時刻が遅い。
    BY <1>1, <2>2, EXT 呼び出しの入れ子
  <2>4. `b` の中で、鍵が `d_1` の鍵 `k_1` である `origin` の呼び出し `b_1` が直に始まる。`a` と `b` は
        どちらも外れであり鍵が `k` で、<1>2a より `a` の時点と `b` の時点で `vars.bindings` は等しく、
        `b` は `a` に真に含まれるので <1>2b より `b` の第 2 引数は `a` の第 2 引数と等しい。よって
        <1>3 より 2 つが直に呼ぶ `origin` の鍵の集合は等しい。`d_1` は
        `a` が直に呼ぶものなので、`k_1` はその集合の元であり、したがって `b` も鍵 `k_1` の `origin` を
        直に呼ぶ。`b_1` は `b` に真に含まれ、`b` は `d_1` に含まれる (`m = 1` のとき `b = d_1`) ので、
        `b_1` は `d_1` に真に含まれる。
    BY <1>2a, <1>2b, <1>3, <2>2, <2>3, EXT 呼び出しの入れ子
  <2>5. CASE `b_1` が外れである。`(d_1, b_1)` は `c` の中の入れ子の対であり、<2>3 よりその外側 `d_1` が
        始まる時刻は `a` より遅い。これは <2>2 の取り方に反する。
    BY <2>2, <2>3, <2>4
  <2>6. CASE `b_1` が当たりである。<1>1 より、`b_1` が始まる時点で `vars.origins` は鍵 `k_1` を持つ。
        <2>3 より `d_1` は外れなので、`d_1` が始まる時点で `vars.origins` は `k_1` を持たない。<1>2 より
        鍵 `k_1` が入るのは鍵 `k_1` の外れの呼び出しが `insert` を実行するときだけなので、そのような
        呼び出し `f` の `insert` が `d_1` の始まりと `b_1` の始まりの間にある。<1>1 より `insert` は `f` の
        末尾式の直前の文であり、EXT 文は書かれた順に実行される より `f` はその後に末尾式を評価して返る。
        よって `f` は `d_1` の実行区間の中で返る。EXT 呼び出しの入れ子 より `f` と `d_1` の
        実行区間は交わらないか一方が他方に含まれるかであり、交わるので後者である。`d_1` が `f` に含まれる
        なら `f` は `d_1` より後に返るが、`f` は `d_1` の実行区間の中で返るのでそれは無い。よって `f` は
        `d_1` に含まれ、`b_1` が始まる時点で `d_1` はまだ返っていないので `f ≠ d_1`、すなわち `f` は
        `d_1` に真に含まれる。よって `(d_1, f)` は `c` の中の入れ子の対であり、<2>3 よりその外側 `d_1` が
        始まる時刻は `a` より遅い。これは <2>2 の取り方に反する。
    BY <1>1, <1>2, <2>2, <2>3, <2>4, EXT 呼び出しの入れ子, EXT 文は書かれた順に実行される
  <2>7. QED (矛盾)
    <2>5 と <2>6 は `b_1` について場合を尽くす。
    BY <2>5, <2>6
<1>5. 鍵 `k` について、値を返す外れの呼び出しは高々 1 つである。
  <2>1. 値を返す外れの呼び出しが 2 つ在るとし、`c_1`、`c_2` とする。EXT 呼び出しの入れ子 より、2 つの
        実行区間は交わらないか、一方が他方に含まれる。
    BY 背理法の仮定, EXT 呼び出しの入れ子
  <2>2. 一方が他方に真に含まれることは無い。含まれるとすると、外側の呼び出しは値を返すのに、その中に
        入れ子の対を持つことになり、<1>4 に反する。
    BY <1>4, <2>1
  <2>3. QED (矛盾)
    <2>1 と <2>2 より 2 つの実行区間は交わらないので、一方 --- `c_1` としてよい --- が他方より前に返る。
    <1>1 より `c_1` は返る前に鍵 `k` を `vars.origins` に入れ、<1>2 より鍵は失われないので、`c_2` が
    始まる時点で `vars.origins` は `k` を持つ。よって `c_2` は当たりであり、外れであることに反する。
    BY <1>1, <1>2, <2>1, <2>2
<1>6. QED
  鍵 `k` について値を返す呼び出しを考える。<1>5 より外れのものは高々 1 つであり、在るならその返り値を
  `A` とする。当たりのものは <1>1 より `vars.origins` の鍵 `k` の値と等しい値を返し、<1>2 よりその値は
  鍵 `k` の外れの呼び出しが `insert` で入れたもの、すなわち `A` と等しい (EXT Clone)。よって鍵 `k` に
  ついて値を返す呼び出しの返り値はどれも `A` と等しい。当たりのものが在って外れのものが無いことは、
  <1>2 より無い。

  1 つの `TypeEnv` の値 `type_env` を固定する。
  `ownership::acted_references(vars, type_env, v, π)` は、`boxed_leaf_paths(&v.ty, type_env)` のうち
  `π` を接頭辞に持つ各 leaf について `origin(vars, type_env, &v.name, &leaf)` の `identity` を数えたもので
  あり、`CancelAnalysis::other_objects(v, π)` は同じ leaf について同じ `origin` を呼び、その `candidates`
  のうち `identity` と異なるものを並べる。<1>0b より `boxed_leaf_paths` と `Origin::identity` は引数で
  決まるので、`acted_references` の返り値は `vars`、`type_env`、`v`、`π` だけで決まる。
  `Origin::candidates` については引数で決まるのは元の集合だけなので (<1>0b)、`other_objects` について
  決まるのも、返る `Vec` の元の集合だけである。
  BY <1>0b, <1>1, <1>2, <1>5, CODE src/rc_ir/ownership.rs: acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects, DEF 引数で決まる関数, EXT Clone

### P2a (`origin` の答えは memo に依らない)

**言明** --- **1 つの `VarTable` の値 `vars` と 1 つの `TypeEnv` の値を固定する。** その 2 つを第 1・
第 2 引数とし、鍵 `(x, π)` が等しい 2 つの `origin` の呼び出しがどちらも値を返すならば、その 2 つの
返り値は等しい。すなわち答えは `vars.origins` が保持する memo の状態に依らない。

**`vars` は、A6 と A11 を満たすプログラムの本体について `VarTable::of` か `VarTable::body_only` が
作った表である。** 製品のコードが作る表はこの 2 つの構成子を通るものだけであり (`VarTable::empty` を直に
呼ぶ残る 1 か所は `#[cfg(test)]` の下の作り手である)、この命題を引く段が扱うのもそれである。**この制限は
言明の一部であって、読む段が自分で補うものではない。**

**L0 はこの制限を持たない。** L0 が量化するのは `VarTable` のどの値でもある `vars` であり、言明の
`vars` はその特別な場合である。

**証明**

<1>1. QED
  L0 は 1 つの `VarTable` の値 `vars` を固定し、`vars` を第 1 引数として行われる `origin` の呼び出しに
  ついて、鍵 `(x, π)` が等しい 2 つがどちらも値を返すならばその 2 つの返り値が等しいことを述べる。
  L0 は `vars` の作られ方を問わないので、言明が制限する `vars` --- A6 と A11 を満たすプログラムの本体に
  ついて `VarTable::of` か `VarTable::body_only` が作った表 --- もその範囲にある。P2a が量化するのは、
  その `vars` を第 1 引数とし、固定した 1 つの `TypeEnv` の値を第 2 引数とする `origin` の呼び出しで
  あり、これは L0 が量化する呼び出しの一部である。よって L0 の言明は P2a の言明を含む。
  BY L0

### L0a (部分木の形)

DEF 部分木 の節点・子・部分木・節点の道について、次の 4 つが成り立つ。

1. 各節点はその道でちょうど 1 つに定まり、相異なる節点の道は相異なる。
2. `N(n)` は、`n` の道を接頭として持つ道の節点の全体である。
3. `n` の相異なる 2 つの子 `c`、`c'` について、`N(c)` と `N(c')` は交わらない。
4. `n` はどの子の部分木にも入らない。したがって `N(n)` の上の構造帰納法 --- `n` の各子について成り
   立つことから `n` について結論する形 --- は整礎である。

**証明**

<1>1. 1 が成り立つ。D2 は本体を式の節点の有限の**木**と定め、木の位置が相異なれば節点も相異なるものと
      定める。木の位置は根からの選択の列で一意に定まるので、各節点はその道でちょうど 1 つに定まり、
      相異なる節点の道は相異なる。
  BY D2, DEF 部分木
<1>1a. 節点の道の長さには最大値 `M` が在り、道の長さが `M` の節点は子を持たない。**したがって、節点に
       ついての言明を、道の長さが `M` の節点から 0 の節点へ下る帰納で示せる** --- 節点 `n` の各子の道は
       `n` の道に選択を 1 つ継ぎ足したものなので、長さが 1 大きい。この帰納は、`n` の各子について
       成り立つことから `n` について結論する形であり、DEF 部分木 の `N(n)` の上の構造帰納法と同じもので
       ある。D2 より本体は有限の木なので節点は有限個であり、<1>1 より相異なる節点の道は相異なるから、
       道も有限個で、その長さには最大値が在る。
  BY <1>1, D2, DEF 部分木
<1>2. 2 が成り立つ。<1>1a の帰納による --- `n` 自身の道は `n` の道を接頭として持ち、`n` の各子 `c` の
      道は `n` の道に選択を 1 つ継ぎ足したものなので、帰納法の仮定より `N(c)` は `c` の道を、したがって
      `n` の道を接頭として持つ道の節点からなる。逆に `n` の道を真の接頭として持つ道は、`n` のどれか
      1 つの子の道を接頭として持つ。
  BY <1>1, <1>1a, DEF 部分木
<1>3. 3 が成り立つ。`n` の相異なる 2 つの子の道は、`n` の道に相異なる選択を 1 つずつ継ぎ足したもので
      あるから、その両方を接頭として持つ道は無い。<1>2 より `N(c)` と `N(c')` の元はそれぞれ `c` の道と
      `c'` の道を接頭として持つので、2 つは交わらない。
  BY <1>1, <1>2, DEF 部分木
<1>4. QED
  4 が成り立つ。`n` の道は各子の道より 1 つ短いので、子の道を接頭として持たない。<1>2 より `n` は
  どの `N(c)` にも入らない。構造帰納法が整礎であることは <1>1a が与える。D2 より本体は有限の木なので、
  `N(n)` は有限集合である。
  BY <1>1, <1>1a, <1>2, <1>3, D2, DEF 部分木

### L0b (走査は `vars` と `type_env` の欄を動かさない)

1 回の `cancel_body` の実行を通じて、その `CancelAnalysis` の値の `vars` の欄と `type_env` の欄は
同じ値である。すなわち、その 2 つの欄が指す `VarTable` と `TypeEnv` は最初から最後まで同じ値である。

**証明**

<1>1. この 2 つの欄は、`cancel` の `cancel_body` の閉包が `CancelAnalysis` の値を構築するときに
      置かれ、その値は閉包の局所変数 `analysis` である。閉包の本文がその値を名指すのは
      `analysis.walk(body, PendingRetains::default(), true)` と `analysis.cancelled()` の 2 か所だけで
      ある。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: CancelAnalysis
<1>2. `CancelAnalysis` は `borrow.rs` の非公開の型であり、その 7 つの欄はいずれも非公開である。
      `borrow.rs` は `mod` 宣言を 1 つも持たないので子孫のモジュールも無く、その欄を名指す式は
      `borrow.rs` の中にしか書けない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis, EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>3. <1>1 の値の欄を名指せる本文は、`CancelAnalysis` のメソッドの本文と `cancel_body` の閉包の本文
      だけである。<1>2 よりその欄を名指す式は `borrow.rs` の中にしか書けず、
      EXT 参照は引数を通ってだけ届く より、ある本文がこの値に届くのは、その値かそれへの参照が引数
      (`self` を含む) として渡ったときか、自分でその値を作ったときに限る。`borrow.rs` で
      `CancelAnalysis` の値かそれへの参照を引数に取るのは、その `impl` ブロックのメソッドの `self` だけ
      である --- `borrow.rs` に `CancelAnalysis` という語が現れるのは、<1>1 の構築、型の宣言、その
      `impl` ブロックの見出しの 3 か所だけであり、ほかにこの型を引数に書いた署名は無い。この値を作るのは
      <1>1 の 1 か所である。
  BY <1>1, <1>2, CODE src/rc_ir/borrow.rs: CancelAnalysis, CODE src/rc_ir/borrow.rs: cancel,
     EXT 参照は引数を通ってだけ届く
<1>4. `CancelAnalysis` のメソッドは 9 つあり、`&mut self` を取るのは `walk`、`walk_inner`、
      `consume_rhs`、`consume`、`consume_objects`、`merge` の 6 つ、`&self` を取るのは
      `acted_references`、`other_objects`、`cancelled` の 3 つである。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled
<1>5. QED
  <1>4 の 6 つの本文と `cancel_body` の閉包の本文で `vars` と `type_env` の欄が現れるのは、どれも値を
  読んで別の関数へ渡す位置であり、どちらの欄への代入も書かれていない。残る 3 つは `self` を共有参照で
  受け取るので、EXT 共有参照は代入を許さない よりこの値のどの欄へも代入できない。その例外の内部可変性が
  与えるのは、欄が指す先を書き換える道だけであり、欄そのもの --- どの `VarTable` と `TypeEnv` を指すか
  --- を差し替える道ではない。<1>2 と <1>3 より、これ以外に欄を動かしうる本文は無い。
  BY <1>1, <1>2, <1>3, <1>4, CODE src/rc_ir/borrow.rs: cancel,
     CODE src/rc_ir/borrow.rs: CancelAnalysis, EXT 共有参照は代入を許さない

### L0c (`ActRefs` は節点で決まる)

1 回の `cancel_body` の実行を固定し、その `CancelAnalysis` の値の `vars` の欄が指す表を `vars`、
`type_env` の欄が指す値を `type_env` と書く。このとき次の 3 つが成り立つ。

1. `vars` と `type_env` は、その実行のあいだ同じ値である。
2. `Retain` 節点 `t = Retain(v, path, _, _)` と `Release` 節点 `r = Release(v, path, _, _)` について、
   `ownership::acted_references(vars, type_env, v, path)` の値は走査のどの時点で読んでも同じであり、
   `vars`、`type_env`、`v`、`path` で決まる。
3. `CancelAnalysis::acted_references(v, path)` は、値を返すときその値を返す。

**証明**

<1>1. 1 が成り立つ。
  BY L0b
<1>2. 2 が成り立つ。L0 の後半が `ownership::acted_references(vars, type_env, v, π)` の返り値について
      それを述べる。<1>1 より、その実行のあいだ第 1・第 2 引数は同じ値である。
  BY L0, <1>1
<1>3. QED
  3 が成り立つ。`CancelAnalysis::acted_references(v, path)` の本文は
  `ownership::acted_references(self.vars, self.type_env, &v.name, path)` を呼び、その値が空のときは
  表明が発火してコンパイラが停止し、そうでないときその値を返す。<1>1 より `self.vars` と
  `self.type_env` はその実行のあいだ同じ値である。
  BY <1>1, <1>2, CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references

### DEF 節点の量

1 回の `cancel_body` の実行を固定し、その `CancelAnalysis` の値の `vars` の欄が指す表を `vars`、
`type_env` の欄が指す値を `type_env` と書く。`Retain` 節点 `t = Retain(v, path, _, _)` と `Release`
節点 `r = Release(v, path, _, _)` について次の値を定める。

- `ActRefs(t) :=` `ownership::acted_references(vars, type_env, v, path)` の値、
  `ActRefs(r) :=` `ownership::acted_references(vars, type_env, v, path)` の値。

この値が節点だけで決まることは L0c が示す。定義をここに置くのはそのためである。`ActRefs(t)` は D15 の
`ActRefs(v, path)` である。

### L1 (`walk` と `rewrite` は内側を 1 回呼ぶ)

`CancelAnalysis::walk(node, pending, returns_from_func)` の 1 回の呼び出しは
`CancelAnalysis::walk_inner(node, pending, returns_from_func)` をちょうど 1 回呼んでその値を返し、
`RewriteCtx::rewrite(node)` の 1 回の呼び出しは `RewriteCtx::rewrite_inner(node)` をちょうど 1 回呼んで
その値を返す。

**証明**

<1>3. `walk` の本文は `grow_stack(|| self.walk_inner(node, pending, returns_from_func))` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk
<1>4. `rewrite` の本文は `grow_stack(|| self.rewrite_inner(node))` である。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite
<1>5. QED
  A15 より `grow_stack(f)` は `f` をちょうど 1 回呼び、その返り値を返す。<1>3 の閉包は `walk_inner` を
  1 回呼んでその値を返し、<1>4 の閉包は `rewrite_inner` を 1 回呼んでその値を返す。
  BY A15, <1>3, <1>4

### L2 (`References` の表現)

`CancelAnalysis` の走査が扱う `References` の値は、どの鍵についてもその値が 1 以上である。よってこの
6 つが成り立つ。以下 `R`、`R1`、`R2` は走査が扱う `References` の値である。

1. `R.is_empty()` が真であることと `R` が空 (DEF 参照の多重集合) であることは同値である。
2. `R1.covers(&R2)` が真であることと `R2 ⊆ R1` であることは同値である。
3. `R1.shares_an_object(&R2)` が真であることと、`R1` と `R2` の双方が参照を持つ位置が在ることは
   同値である。また `R1.names(o)` が真であることと、`R1` が位置 `o` の参照を持つことは同値である。
   `R1.objects()` は `R1` が参照を持つ位置をちょうど 1 度ずつ並べた列である。
4. `R2 ⊆ R1` のとき `R1.subtract(&R2)` は `R1` を `R1 - R2` に書き換え、panic しない。また
   `R1 - R2 ⊆ R1` である。
5. `CancelAnalysis::acted_references(v, path)` が値を返すとき、その値は空でない。
6. `R1` と `R2` が等しい (DEF 参照の多重集合) ことと、各位置についての 2 つの個数が一致することは
   同値である。

**証明**

<1>1. `ownership::acted_references(vars, type_env, v, path)` が返す `References` の各鍵の値は 1 以上で
      ある。鍵の値が増えるのは `*references.entry(object).or_default() += 1` の 1 か所だけであり、鍵は
      その場で作られるので、値が 0 の鍵は残らない。
  BY CODE src/rc_ir/ownership.rs: acted_references
<1>2. `CancelAnalysis::acted_references(v, path)` は `ownership::acted_references` の値を返す。ただし
      その値が空のときは `assert!(!references.is_empty(), ...)` が発火してコンパイラが停止する。よって
      この関数が値を返すとき、その値は空でなく、各鍵の値は 1 以上である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references, <1>1
<1>3. **`R1.covers(&R2)` が真であるとき**、`R1.subtract(&R2)` は panic せず、`R2` の各鍵についての `R1` の
      値をその分だけ減らし、減らした結果が 0 になった鍵を取り除く。`R2` の鍵でない `R1` の鍵の値は変わら
      ない。よって、各鍵の値が 1 以上の `R1` を `covers` が真である `R2` で `subtract` した結果も各鍵の
      値が 1 以上であり、その値は各位置の個数を `R2` の分だけ減らしたものである。

      `subtract` の本文は `for (object, count) in &other.0 { let held_count = self.0.get_mut(object)
      .expect(...); *held_count -= count; if *held_count == 0 { self.0.remove(object); } }` である。
      `covers` の本文は `other.0.iter().all(|(object, count)| self.0.get(object)
      .is_some_and(|held_count| held_count >= count))` なので、`covers` が真であれば `other` のどの鍵に
      ついても `self` はその鍵を持ちその値以上である。よって `get_mut` の `expect` は発火せず、
      `*held_count -= count` は underflow しない。**この 2 つの `expect` と減算が、`covers` を仮説に
      要求する理由である。** `covers` を落とすと、`R1` が持たない鍵を `R2` が持つとき `expect` が発火し、
      `R2` の値が `R1` の値を超えるとき減算が underflow する。
  BY CODE src/rc_ir/ownership.rs: References::subtract, CODE src/rc_ir/ownership.rs: References::covers,
     EXT Map と Set, EXT Iterator::all と any, EXT IntoIterator と for
<1>4. `References::covers(other)` は、`other` のどの鍵についても、自分がその鍵を持ちその値以上であることを
      言う。各鍵の値が 1 以上のとき、これは各位置の個数の不等式、すなわち `other ⊆ self` と同値で
      ある (`other` が持たない位置の個数は 0 で、不等式は自動的に成り立つ)。よって 2 が成り立つ。
      また `covers` が真のとき、<1>3 より引き算は panic せず結果は `self - other` である。`R1 - R2` の各
      位置の個数は `R1` のそれ以下なので `R1 - R2 ⊆ R1` である。よって 4 が成り立つ。
  BY CODE src/rc_ir/ownership.rs: References::covers, <1>3, EXT Map と Set, EXT Iterator::all と any
<1>5. `References::is_empty()` は内側の `Map` が空であることを言う。各鍵の値が 1 以上の `References` に
      ついて、これは参照を 1 つも持たないことと同値である。よって 1 が成り立つ。
  BY CODE src/rc_ir/ownership.rs: References::is_empty, <1>1, <1>3
<1>6. `References::shares_an_object(other)` の本文は `other.0.keys().any(|object| self.0.contains_key(object))`
      であり、`other` の鍵のいずれかが自分の鍵であることを言う。`References::names(object)` の本文は
      `self.0.contains_key(object)` であり、`object` が自分の鍵であることを言う。`References::objects()` の
      本文は `self.0.keys().cloned().collect()` であり、EXT Map と Set より自分の各鍵の複製をちょうど
      1 度ずつ渡す反復子を `Vec` に並べたもの、すなわち自分の鍵を 1 度ずつ並べた列を返す。各鍵の値が 1 以上の
      `References` について、鍵であることとその位置の参照を 1 つ以上持つことは同値である。よって 3 が
      成り立つ。
  BY CODE src/rc_ir/ownership.rs: References::shares_an_object,
     CODE src/rc_ir/ownership.rs: References::names, CODE src/rc_ir/ownership.rs: References::objects,
     <1>1, <1>3, EXT Map と Set, EXT Iterator::all と any, EXT Iterator::map と collect
<1>7. 走査が扱う `References` の値は、`CancelAnalysis::acted_references` が返したもの、それを `subtract`
      で減らしたもの、およびそれらの複製だけである。`References` のフィールドは非公開なので、
      EXT 可視性と私有性 より `References` の値を構築できるのは `ownership.rs` の中だけであり、
      `ownership.rs` は `mod` 宣言を `#[cfg(test)] mod tests` の 1 つしか持たない。
  BY CODE src/rc_ir/ownership.rs: References (フィールドは非公開なので `ownership` の外では作れない),
     CODE src/rc_ir/ownership.rs: acted_references (`References(references)` がこのモジュールでの唯一の
     構築点), CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
     (`outstanding` は `self.acted_references` の値), CODE src/rc_ir/borrow.rs: un_bump
     (`innermost.outstanding.subtract(un_bumped)`), CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
     (`outstanding.clone()`), EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>7a. 走査が `References::subtract` を呼ぶのは `un_bump` の中の
       `innermost.outstanding.subtract(un_bumped)` の 1 か所だけであり、その文に到達するのは直前の
       `if !innermost.outstanding.covers(un_bumped) { return UnBump::OutsideBracket; }` を通り抜けたとき、
       すなわち `innermost.outstanding.covers(un_bumped)` が真のときに限る。よって <1>3 の仮説は、走査が
       行うすべての `subtract` の呼び出しで満たされる。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>7
<1>7b. 6 が成り立つ。`References` は `PartialEq` を derive し、そのフィールドは `Map` (`FxHashMap`)
       1 つなので、2 つの値が等しいことは 2 つの `Map` が等しいこと、すなわち鍵と値の対の集合が一致する
       ことである。各鍵の値が 1 以上のとき、鍵と値の対の集合の一致は各位置についての個数の一致と同値で
       ある --- 一方が持たない位置の個数は 0 だからである。
  BY CODE src/rc_ir/ownership.rs: References, CODE src/misc.rs: Map, DEF 参照の多重集合, EXT Map と Set,
     EXT Clone
<1>8. QED
  <1>7 が走査の扱う値の出どころを尽くし、<1>1、<1>2 が `acted_references` の値について、<1>3 と <1>7a が
  `subtract` で減らした値について、各鍵の値が 1 以上であることを与える。複製は EXT Clone より元と同じ鍵と
  値を持つ。1 から 4 は <1>4、<1>5、<1>6 が、5 は <1>2 が、6 は <1>7b が与える。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>7a, <1>7b, EXT Clone

### L2a (根の値が在るあいだ、木の割り当てはすべて生存している)

`RcExprNode` の値 `n` が在るあいだ、`n` を根とする木のすべての位置の `Arc<RcExpr>` のハンドルが 1 つ以上
存在し、DEF 割り当て よりその割り当ては生存している。

**証明**

<1>1. `RcExprNode` は `expr: Arc<RcExpr>` を 1 つ持つ。`RcExpr` の各変位は、DEF 部分木 の子の表が挙げる
      子の `RcExprNode` を値として保持する --- `Let(_, rhs, k)` は `k` を、`rhs` が `RcRhs::Match(_, arms)`
      のときは各 `MatchArm` の `body` も、`Retain`/`Release`/`Destructure`/`Eval` は継続 `k` を保持し、
      `Ret(_)` は `RcExprNode` を保持しない。
  BY CODE src/rc_ir/ast.rs: RcExprNode, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
     CODE src/rc_ir/ast.rs: MatchArm, DEF 部分木
<1>2. QED
  木 `N(n)` の構造についての帰納法で示す。L0a の 4 よりこの帰納法は整礎である。`n` の値が在れば、その
  `expr` の欄が `Arc<RcExpr>` のハンドルを 1 つ持つので、DEF 割り当て よりその割り当ては生存している。
  EXT Arc の契約 より、ハンドルが 1 つでも在るあいだそのブロックはアロケータへ返らないので、その
  ハンドルが指すブロックの中に `RcExpr` の値が在る。<1>1 よりその中に `n` の各子の `RcExprNode` の値が
  在るので、帰納法の仮定が各子の部分木について同じことを与える。
  BY <1>1, L0a, DEF 部分木, DEF 割り当て, EXT Arc の契約

### L2b (`cancel` の呼び出しは `borrow_ify` の出力を受け取る)

`cancel(prog, type_env)` のすべての呼び出しについて、`prog` が指すのは `borrow_ify` の 1 回の呼び出しが
返した値**そのもの**である。とくに、その値の各位置の `Arc<RcExpr>` は `borrow_ify` が置いたハンドルと
同じ割り当てを指す。

**言明が述べるのは同一性であって、D1a の等しさではない。** D1a は「`Arc` の参照カウントも、木が式を共有
する度合いも、成分ではない」と述べるので、D1a の等号は 2 つの `RcProgram` の `Arc` の番地について何も
言わない。P15 の前半は `node_id` --- `Arc<RcExpr>` の割り当ての占める番地 --- を主語にするので、そこへ
渡すには同一性が要る。

**証明**

<1>1. `cancel` は `pub(crate)` なので、EXT 可視性と私有性 よりその呼び出しはこのクレートの中にしか
      書けない。EXT このリポジトリのターゲット より、そのクレートの項目は `src/` のファイルが宣言する。
      `src/` の全体を `cancel(` で走査すると、関数定義の頭を除いて 1 か所が挙がる ---
      `build_object_files.rs` の `optimize_rc_program` である。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/build/build_object_files.rs: optimize_rc_program,
     EXT 可視性と私有性, EXT このリポジトリのターゲット, DEF このクレート
<1>2. `optimize_rc_program` の局所変数 `prog` は、`prog = borrow_ify(&prog, type_env,
      config.develop_mode);` の文の後、`prog = cancel(&prog, type_env);` の文が `cancel` を呼ぶ時点まで、
      `borrow_ify` が返した値を保持する。この 2 つの文のあいだに在るのは
      `validate(&prog, "after borrow_ify");` の 1 文だけであり、EXT 文は書かれた順に実行される より
      ほかの文はこのあいだに実行されない。`validate` は `prog` を共有参照で受け取るので、
      EXT 共有参照は代入を許さない より `prog` そのものへも `RcProgram` の欄へも代入できない。
      `cancel` に渡る `&prog` は、EXT 演算対象は式より先に評価される より第 3 の文の代入より先に
      評価される。
  BY <1>1, CODE src/build/build_object_files.rs: optimize_rc_program,
     CODE src/rc_ir/validate.rs: validate, EXT 文は書かれた順に実行される,
     EXT 演算対象は式より先に評価される, EXT 共有参照は代入を許さない
<1>3. QED
  <1>2 より、`cancel` が受け取るのは `borrow_ify` が返した値そのものである。残るのは内部可変性を
  通した書き込みの道だけであり、それは `Arc<RcExpr>` の割り当てを動かさない ---
  `Validator::check_rhs` は `llvm_gen.result_prov(&x.ty, &arg_tys, self.type_env)` を呼び、その
  `Arc<TypeNode>` は `OnceLock` の欄を 3 つ持つ。A3 は「**`RcProgram` から到達できる値の
  等しさは、それを共有参照で受け取る計算が変えない。**」と述べ、「`validate` がその 1 つで
  あり」とこの関数を名指す。同じ節は、その欄が「**一度だけ書かれる memo であって、その値はその型の
  `PartialEq` が読む成分の関数である**」と述べる。書かれるのは `TypeNode` の欄であって
  `RcExprNode` の `expr` の欄ではないので、木の各位置の `Arc<RcExpr>` のハンドルが指す割り当ては
  動かない。
  BY <1>1, <1>2, CODE src/build/build_object_files.rs: optimize_rc_program,
     CODE src/rc_ir/validate.rs: validate, CODE src/rc_ir/validate.rs: Validator::check_rhs,
     CODE src/rc_ir/ast.rs: RcExprNode, CODE src/ast/types.rs: TypeNode, A3

### L3 (走査する本体は `RewriteCtx::rewrite` の出力である)

ASSUME  NEW `prog`: `RcProgram`,
        `prog` は `borrow_ify` の 1 回の呼び出しが返した値である
PROVE   `cancel(prog, type_env)` が `cancel_body` に渡す本体 --- `prog.funcs` の各 `f` の `f.body` と
        `prog.globals` の各 `g` の `g.init` --- は、いずれも `RewriteCtx::rewrite` の 1 回の呼び出しが
        返した木である。

仮定は P15 の言明の「`cancel` の入力すなわち `borrow_ify` の出力」である。

**証明**

<1>1. 本補題の仮定より `prog` は `borrow_ify` の 1 回の呼び出しが返した値である。L2b より、この仮定は
      `cancel` のすべての呼び出しで満たされる。
  BY 本補題の仮定, L2b
<1>2. `borrow_ify` が返す `RcProgram` の `funcs` の各値の `body` は `ctx.rewrite(&f_own.body)` または
      `ctx.rewrite(&clone.body)` の値であり、`globals` の各値の `init` は `ctx.rewrite(&g.init)` の値で
      ある。返す直前に走る `for func in funcs.values_mut()` のループは `borrowed_units` だけを書き換える。
  BY CODE src/rc_ir/borrow.rs: borrow_ify
<1>3. `cancel` は `prog.funcs.values()` の各 `f` について `cancel_body(&vars, &f.body)` を呼び、
      `prog.globals` の各 `g` について `cancel_body(&vars, &g.init)` を呼ぶ。ほかに `cancel_body` を
      呼ばない。
  BY CODE src/rc_ir/borrow.rs: cancel
<1>4. QED
  <1>1 の `prog` について、<1>2 よりその `funcs` の各値の `body` と `globals` の各値の `init` は
  `RewriteCtx::rewrite` の 1 回の呼び出しが返した木である。<1>3 より `cancel` が `cancel_body` に渡すのは
  その木だけである。
  BY <1>1, <1>2, <1>3

### L3a (`rewrite_inner` の 1 回の呼び出しの形)

`RewriteCtx::rewrite_inner(node)` の 1 回の呼び出しについて、次の 3 つが成り立つ。

1. この呼び出しは `self.rewrite` を `node` の各子 (DEF 部分木) についてちょうど 1 回ずつ呼び、ほかに
   `self.rewrite` を呼ばない。
2. 返す木の位置は、それらの `self.rewrite` の呼び出しが返した木の位置と、この呼び出しの中で `expr_node`
   (`rc_node` と `prepend_rc` が呼ぶものを含む) が作った節点の**全体**である。すなわち、受け取った木も
   作った節点も 1 つ残らず返す木に入り、返す木にはそれ以外の位置が無い。
3. それらの `self.rewrite` の呼び出しの実行区間は互いに交わらず、この呼び出しの中の `expr_node` の実行は
   どの `self.rewrite` の実行区間の中にも無い。

**証明**

<1>0. `expr_node` の呼び出しと `RewriteCtx::rewrite` の呼び出しは `borrow.rs` の中にしか書けない。
      `borrow.rs` の中で `expr_node` を呼ぶのは、`RewriteCtx::rewrite_inner`、`rc_node`、
      `split_body_inner`、`drop_nodes_inner` の 4 つの本文だけであり、`RewriteCtx::rewrite` を呼ぶのは
      `borrow_ify`、`RewriteCtx::rewrite_inner`、`RewriteCtx::rewrite_rc` の 3 つの本文だけである。
      `expr_node` は `borrow.rs` の非公開の自由関数、`RewriteCtx::rewrite` は `RewriteCtx` の非公開の
      メソッドであり、`borrow.rs` は `mod` 宣言を 1 つも持たないので子孫のモジュールも無い。
  BY CODE src/rc_ir/borrow.rs: expr_node, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite,
     CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/borrow.rs: rc_node,
     CODE src/rc_ir/borrow.rs: split_body_inner, CODE src/rc_ir/borrow.rs: drop_nodes_inner,
     CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc,
     DEF 本文, EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>1. `expr_node(expr, source)` は `Arc::new(expr)` を 1 つ作り、それを `expr` フィールドに持つ
      `RcExprNode` を返す。
  BY CODE src/rc_ir/borrow.rs: expr_node
<1>1a. `route`、`call_rc`、`RewriteCtx::owns_unit`、`units_under` の 1 回の呼び出しの中では、`expr_node`
       も `RewriteCtx::rewrite` も走らない。よってその呼び出しは `RcExprNode` の値を 1 つも作らず、
       `self.rewrite` の呼び出しも 1 つも起こさない。
  <2>1. `route` または `call_rc` の 1 回の呼び出しの中で走る `borrow.rs` の関数は、この 2 つと
        `RewriteCtx::routing_is_safe`、`RewriteCtx::routing_saves_retain`、
        `RewriteCtx::any_owned_unit`、`RewriteCtx::owns_unit`、`RewriteCtx::owns_object`、
        `RewriteCtx::comes_from_a_value_used_later`、`used_later`、`rhs_uses` で尽きる。
        `borrow.rs` の関数としては、`route` は `routing_is_safe` と `routing_saves_retain` を、
        `call_rc` は `owns_unit` を、`routing_is_safe` は `any_owned_unit` を、
        `routing_saves_retain` は `used_later`、`owns_unit`、`comes_from_a_value_used_later` を、
        `any_owned_unit` は `owns_unit` を、`owns_unit` は `owns_object` を、
        `comes_from_a_value_used_later` は `used_later` を、`used_later` は `rhs_uses` と自身を、
        `rhs_uses` は `used_later` を呼ぶ。`owns_object` が呼ぶ `borrow.rs` の関数は無い。
        **この 10 個の本文に書かれた呼び出しのうち、`borrow.rs` の項目に解決するのは上に挙げた
        ものだけであり、残りはすべて `borrow.rs` の外で定義された項目に解決する。** DEF 本文 より、
        この 10 個が引数として渡すクロージャの本文はそれを書いた関数の本文の一部なので、その中に
        書かれた呼び出しもこの数え上げに入っている --- 渡す先が標準ライブラリであっても
        `borrow.rs` の外の項目であっても同じである。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::route, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc,
       CODE src/rc_ir/borrow.rs: RewriteCtx::routing_is_safe,
       CODE src/rc_ir/borrow.rs: RewriteCtx::routing_saves_retain,
       CODE src/rc_ir/borrow.rs: RewriteCtx::any_owned_unit,
       CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
       CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object,
       CODE src/rc_ir/borrow.rs: RewriteCtx::comes_from_a_value_used_later,
       CODE src/rc_ir/borrow.rs: used_later, CODE src/rc_ir/borrow.rs: rhs_uses, DEF 本文
  <2>2. `RewriteCtx::owns_unit` の 1 回の呼び出しの中で走る `borrow.rs` の関数は、`owns_unit` と
        `owns_object` の 2 つだけである。<2>1 の数え上げで `owns_unit` が呼ぶ `borrow.rs` の関数は
        `owns_object` だけであり、`owns_object` が呼ぶ `borrow.rs` の関数は無い。
    BY <2>1, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit,
       CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
  <2>3. `units_under` の 1 回の呼び出しの中で走る `borrow.rs` の関数は 1 つも無い。`units_under` の中で
        走る `ownership.rs` の関数は `subtree_type`、`rc_units`、`rc_units_go`、`unit_step`、
        `held_field_type` で尽き、残りは `src/ast/types.rs` の型についての関数と標準ライブラリの関数で
        ある。この 5 つの本文にも `units_under` の本文にも `borrow.rs` の項目への参照は無い。
    BY CODE src/rc_ir/ownership.rs: units_under, CODE src/rc_ir/ownership.rs: subtree_type,
       CODE src/rc_ir/ownership.rs: rc_units, CODE src/rc_ir/ownership.rs: rc_units_go,
       CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/ownership.rs: held_field_type, DEF 本文
  <2>4. QED
    <1>0 が挙げる本文 --- `expr_node` を呼ぶ 4 つと `RewriteCtx::rewrite` を呼ぶ 3 つ、重複する
    `rewrite_inner` を 1 つと数えて 6 つ --- は、
    <2>1 の 10 個にも <2>2 の 2 つにも入らず、<2>3 より `units_under` の中では `borrow.rs` の関数が
    1 つも走らない。`borrow.rs` の外で定義された関数の本文は `borrow.rs` の中に無いので、<1>0 より
    `expr_node` も `RewriteCtx::rewrite` も呼べない。よってこの 4 つの呼び出しの中で `expr_node` も
    `RewriteCtx::rewrite` も走らない。`RcExprNode` の値を `Arc::new` から作るのは `expr_node` だけで
    ある (<1>1)。
    BY <1>0, <1>1, <2>1, <2>2, <2>3, DEF 本文
<1>2. `rc_node(is_release, var, path, state, k, source)` は `expr_node` を 1 回呼んでその値を返す。
      `prepend_rc(units, is_release, k)` は `units` を逆順にたたみ込み、`units` の要素ごとに `rc_node`
      を 1 回呼ぶ。`units` が空なら `k` をそのまま返す。どちらも `self.rewrite` を呼ばない。
  BY CODE src/rc_ir/borrow.rs: rc_node, CODE src/rc_ir/borrow.rs: prepend_rc, <1>0, <1>1,
     EXT Iterator::fold と rev
<1>3. `RewriteCtx::rewrite_rc(v, path, state, is_release, k, source)` は `self.rewrite(k)` をちょうど
      1 回呼び、ほかに `self.rewrite` を呼ばず、その値の上に `rc_node` で 0 個以上の節点を積んだものを
      返す。返す木の位置は、その `self.rewrite(k)` が返した木の位置と、`rc_node` が積んだ節点の
      **全体**である。**`self.rewrite(k)` はこの本文の最初の文であり、`rc_node` の実行はすべて
      それが返った後に起きる。**
  <2>1. 本文は最初に `let k = self.rewrite(k);` を実行する。`self.is_borrow_version` が偽のときは
        `rc_node(is_release, v.clone(), path.clone(), state, k, source)` を返す。真のときは
        `kept.into_iter().rev().fold(k, |cont, unit| rc_node(is_release, v.clone(), unit, state, cont, source))`
        を返す。この `fold` の値は、`k` の上に `kept` の要素ごとに `rc_node` の節点を 1 つ積んだ木で
        ある。EXT 文は書かれた順に実行される より、どちらの枝の `rc_node` も、最初の文が返った後に
        実行される。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, EXT Iterator::fold と rev,
       EXT 文は書かれた順に実行される
  <2>2. `kept` は `units_under(&v.ty, path, self.type_env)` を `self.owns_unit(v, unit)` で絞った
        `Vec<FieldPath>` である。`units_under` の返り値の型は `Vec<FieldPath>`、`owns_unit` の返り値の
        型は `bool`、`FieldPath` は `Vec<usize>` であり、どれも `RcExprNode` を含まない。よって `kept`
        は木の節点を持ち込まない。<1>1a より、この 2 つの呼び出しの中で `self.rewrite` も `expr_node` も
        走らない。
    BY <1>1a, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc,
       CODE src/rc_ir/ownership.rs: units_under,
       CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit, CODE src/rc_ir/ast.rs: FieldPath
  <2>3. QED
    <2>1 の 2 つの枝はどちらも、`self.rewrite(k)` が返した木の上に `rc_node` の節点を 0 個以上積んだ木を
    返す。積むのは `fold` なので、作った節点は 1 つ残らずその木に入る。<1>2 より `rc_node` は
    `expr_node` で 1 節点を作り、`self.rewrite` を呼ばない。<2>2 よりほかの値は木の節点にならず、
    `kept` を作る 2 つの呼び出しも `self.rewrite` を呼ばない。よって `self.rewrite` の呼び出しは
    最初の文の 1 回だけであり、EXT 文は書かれた順に実行される より `rc_node` の実行はその後である。
    BY <1>1a, <1>2, <2>1, <2>2, EXT Iterator::fold と rev, EXT 文は書かれた順に実行される
<1>4. `rewrite_inner(node)` の 8 つの腕はいずれも、`self.rewrite` を `node` の各子についてちょうど
      1 回ずつ呼び、ほかに `self.rewrite` を呼ばず、その戻り値の上に `expr_node` / `rc_node` /
      `prepend_rc` で有限個の節点を積んだ木を返す。返す木の位置は、それらの `self.rewrite` の呼び出しが
      返した木の位置と、この呼び出しの中で `expr_node` (`rc_node` と `prepend_rc` が呼ぶものを含む) が
      作った節点の**全体**である。すなわち、受け取った木も作った節点も 1 つ残らず返す木に入り、返す木に
      はそれ以外の位置が無い。**さらに、それらの `self.rewrite` の呼び出しの実行区間は互いに交わらず、
      この呼び出しの中の `expr_node` の実行はどの `self.rewrite` の実行区間の中にも無い。**
  <2>1. `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕が `self.rewrite` を呼ぶのは
        `self.rewrite(k)` の 1 回だけであり、その上に `prepend_rc(after, true, ...)`、
        `expr_node(RcExpr::Let(...))`、`prepend_rc(before, false, ...)` で節点を積む。この腕に落ちる
        節点の子は継続 `k` だけである (DEF 部分木)。返す木の位置は、その `self.rewrite(k)` が返した
        木の位置と、これらが積んだ節点の全体である。この腕の `expr_node` の実行はすべて
        `self.rewrite(k)` が返った後に起きる。
    <3>1. この腕の本文は 5 つの文である。`let callee = self.route(x, callee, args, k);`、
          `let (before, after) = self.call_rc(&callee, args);`、
          `let k = prepend_rc(after, true, self.rewrite(k));`、
          `let app = expr_node(RcExpr::Let(x.clone(), RcRhs::App(callee, args.clone()), k), &node.source);`、
          `prepend_rc(before, false, app)`。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕
    <3>2. `route` の返り値の型は `RcVar` であり、`call_rc` の返り値の型は
          `(Vec<(RcVar, FieldPath)>, Vec<(RcVar, FieldPath)>)` である。`RcVar` の 5 つのフィールドの型は
          `FullName`、`Arc<TypeNode>`、`Option<Span>`、`Option<Name>`、`bool` であり、`FieldPath` は
          `Vec<usize>` である。いずれの型も `RcExprNode` を含まないので、`callee`、`before`、`after`、
          `x.clone()`、`args.clone()` は木の節点を持ち込まない。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::route, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc,
         CODE src/rc_ir/ast.rs: RcVar, CODE src/rc_ir/ast.rs: FieldPath
    <3>3. QED
      <3>1 の第 1 文と第 2 文の `route` と `call_rc` は節点を作らず `self.rewrite` も呼ばない
      (<1>1a) ので、この腕が `self.rewrite` を呼ぶのは第 3 文の `self.rewrite(k)` の 1 回だけであり、
      この腕がこの呼び出しの中で作る節点は第 3 文から第 5 文のものだけである。DEF 部分木 より
      この腕に落ちる節点の子は継続 `k` だけなので、`self.rewrite` はこの節点の各子についてちょうど
      1 回ずつ呼ばれている。第 3 文から第 5 文が木を組み立て、その材料は `self.rewrite(k)` が返した
      木と、`prepend_rc` と `expr_node` が作る節点だけである (<1>2、<3>2)。第 3 文の
      `prepend_rc(after, ...)` が作る節点は `k` に、第 4 文の `expr_node` が作る節点は `app` に、
      第 5 文の `prepend_rc(before, ...)` が作る節点は返り値に入り、`k` は `app` の中に、`app` は
      返り値の中にあるので、作った節点は 1 つ残らず返り値の木に入る。第 3 文では
      EXT 演算対象は式より先に評価される より `self.rewrite(k)` が引数として先に評価され、その値を
      受け取ってから `prepend_rc` が走る。EXT 文は書かれた順に実行される より第 4 文と第 5 文は
      第 3 文の後に実行される。よってこの腕の `expr_node` の実行はすべて `self.rewrite(k)` が返った
      後である。
      BY <1>1a, <1>2, <3>1, <3>2, DEF 部分木, EXT 文は書かれた順に実行される,
         EXT 演算対象は式より先に評価される
  <2>2. `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕は、`arms` の各 `arm` について
        `self.rewrite(&arm.body)` を 1 回ずつ呼び、`self.rewrite(k)` を 1 回呼び、`expr_node` で
        1 節点を積む。アームの列は `arms.iter().map(|arm| arm.with_body(self.rewrite(&arm.body))).collect()`
        で作られるので、EXT Iterator::map と collect より閉包は `arms` の各要素についてちょうど 1 回、
        先頭から順に呼ばれ、作られる列の第 `i` 要素は `arms[i]` について `with_body` が返したアームで
        ある。すなわち `self.rewrite(&arm.body)` の呼び出しはアームごとにちょうど 1 回であり、閉包が
        順に呼ばれるので 2 つのアームの呼び出しの実行区間は交わらない。`arm.with_body(body)` は
        `body` をそのアームの本体に据えたアームを返す。`MatchArm` の残る 3 フィールド `tag`、
        `payload`、`payload_state` は `RcExprNode` を持たないので、この節点の子は `self.rewrite` の
        各呼び出しが返した木の根だけである (DEF 部分木)。EXT 文は書かれた順に実行される より
        アームの列を作る文が先に走り、EXT 演算対象は式より先に評価される より続く `expr_node(...)` の
        引数として `self.rewrite(k)` が評価されてから `expr_node` が呼ばれる。よって
        この腕の `self.rewrite(k)` の実行区間はどのアームの呼び出しとも交わらず、`expr_node` の実行は
        どの `self.rewrite` の実行区間の中にも無い。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
       CODE src/rc_ir/ast.rs: MatchArm::with_body, CODE src/rc_ir/ast.rs: MatchArm, DEF 部分木,
       EXT Vec::iter と slice::iter, EXT Iterator::map と collect, EXT 呼び出しの入れ子,
       EXT 文は書かれた順に実行される, EXT 演算対象は式より先に評価される
  <2>3. `RcExpr::Let(x, rhs, k)` の腕は `self.rewrite(k)` を 1 回呼び、`expr_node` で 1 節点を積む。
        `rhs.clone()` は木の位置を持ち込まない。この腕に落ちる `rhs` は `RcRhs::Match` ではなく
        (`match` の腕はこの順に並んでいる)、`Match` でない右辺を持つ `Let` 節点の子は継続だけだから
        である (DEF 部分木)。EXT 演算対象は式より先に評価される より `expr_node` の引数として
        `self.rewrite(k)` が先に評価されるので、`expr_node` の実行はそれが返った後である。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, rhs, k)` の腕,
       CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, DEF 部分木,
       EXT 演算対象は式より先に評価される
  <2>4. `RcExpr::Retain(v, path, state, k)` の腕と `RcExpr::Release(v, path, state, k)` の腕は
        `self.rewrite_rc` を 1 回呼び、その値を返す。<1>3 よりその呼び出しは `self.rewrite(k)` を
        ちょうど 1 回、最初の文として行い、`rc_node` の実行はその後である。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Retain(v, path, state, k)` の腕,
       CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Release(v, path, state, k)` の腕,
       <1>3
  <2>5. `RcExpr::Destructure(container, fields, state, k)` の腕と `RcExpr::Eval(v, k)` の腕は
        `self.rewrite(k)` を 1 回呼び、`expr_node` で 1 節点を積む。この 2 種の節点の子は継続だけで
        あり (DEF 部分木)、腕が複製する `container`、`fields`、`v` は `RcVar` と `Vec<(usize, RcVar)>`
        なので木の位置を持ち込まない。EXT 演算対象は式より先に評価される より、どちらの腕も
        `expr_node` の引数として `self.rewrite(k)` を先に評価するので、`expr_node` の実行はそれが
        返った後である。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Destructure(container, fields, state, k)` の腕,
       CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Eval(v, k)` の腕,
       CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcVar, DEF 部分木,
       EXT 演算対象は式より先に評価される
  <2>6. `RcExpr::Ret(v)` の腕は `expr_node` で 1 節点を作って返し、`self.rewrite` を呼ばない。この節点に
        子は無い。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Ret(v)` の腕, DEF 部分木
  <2>7. QED
    `RcExpr` の変位は `Let`, `Retain`, `Release`, `Destructure`, `Eval`, `Ret` の 6 つで、`RcRhs` の
    変位は `Var`, `App`, `Closure`, `Llvm`, `Match` の 5 つである。<2>1 から <2>6 は
    `Let` (右辺が `App`)、`Let` (右辺が `Match`)、`Let` (右辺がそれ以外)、`Retain`、`Release`、
    `Destructure`、`Eval`、`Ret` の 8 つを尽くし、これは `rewrite_inner` の `match` の 8 つの腕である。
    <2>1 から <2>6 より、どの腕でも `self.rewrite` の呼び出しは順に行われ、`expr_node` の実行は
    そのすべてが返った後なので、EXT 呼び出しの入れ子 より実行区間は互いに交わらず、`expr_node` の実行は
    どの実行区間の中にも無い。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
       CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, EXT 呼び出しの入れ子
<1>5. QED
  `rewrite_inner` の本文は `node.expr.as_ref()` についての `match` であり、1 回の呼び出しはその 8 つの
  腕のうち 1 つを実行する。<1>4 はその 8 つを尽くし、どの腕についても 1、2、3 を与える。
  BY <1>4, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

### L4 (走査する本体の `Match` はアームを 1 つ以上持つ)

ASSUME  NEW `prog`: `RcProgram`,
        `prog` は `borrow_ify` の 1 回の呼び出しが返した値である
PROVE   `cancel(prog, type_env)` が走査する本体のすべての `Match` は、1 つ以上のアームを持つ。

**証明**

<1>1. `RewriteCtx::rewrite_inner(node)` が返す木の各 `Match` は、`node` の木のある `Match` から、
      アームの本体だけを差し替えて作られたものである。アームの個数は等しい。
  <2>1. `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕は、新しいアームの列を
        `arms.iter().map(|arm| arm.with_body(self.rewrite(&arm.body))).collect()` で作る。
        `Iterator::map` と `collect` は要素数を保ち、`MatchArm::with_body` は `body` だけを差し替えた
        `MatchArm` を返す。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
       CODE src/rc_ir/ast.rs: MatchArm::with_body, EXT Vec::iter と slice::iter,
       EXT Iterator::map と collect
  <2>2. ほかの腕は `RcRhs::Match` を作らない。`RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕が作る
        右辺は `RcRhs::App` であり、`RcExpr::Let(x, rhs, k)` の腕は `rhs.clone()` をそのまま運ぶ。
        `match` の腕はこの順に並んでいるので、この第 3 の腕に落ちる `rhs` は `RcRhs::Match` ではなく、
        `RcRhs` の残る 4 変位 (`Var`, `App`, `Closure`, `Llvm`) はアームを持たない。`rc_node`、
        `prepend_rc` が作る式は `Retain` と `Release` であり、`expr_node` はほかの腕で `Let` (右辺は
        `App` または複製した `rhs`)、`Destructure`、`Eval`、`Ret` を作る。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/borrow.rs: rc_node,
       CODE src/rc_ir/borrow.rs: prepend_rc, CODE src/rc_ir/ast.rs: RcRhs
  <2>3. QED
    L3a の 1 より `rewrite_inner` は `self.rewrite` を `node` の各子についてちょうど 1 回ずつ呼び、L1 より
    その呼び出しは `rewrite_inner` を 1 回呼ぶ。L3a の 2 より、返る木の位置は、それらの呼び出しが返した
    木の位置と、この呼び出しの中で作られた節点の全体である。木 `N(node)` の構造についての帰納法 (L0a の 4
    より整礎) で示す --- 各子について返る木の `Match` は帰納法の仮定が扱い、この呼び出しの中で作られる
    節点のうち `Match` を右辺に持つのは <2>1 の腕が作る 1 つだけで (<2>2)、そのアームの個数は `node` の
    `Match` のものに等しい。よって返る木の `Match` は入力の木の `Match` と 1 対 1 に対応し、アームの個数が
    等しい。
    BY <2>1, <2>2, L0a, L1, L3a, DEF 部分木
<1>2. 本補題の仮定の `borrow_ify` の呼び出しが `rewrite` に渡す本体は、その入力プログラムの関数の本体
      `f_own.body`、その複製 `clone.body`、または入力プログラムのグローバル初期化子 `g.init` である。
      `f_own` は `func.clone()` であり、`clone` は `clone_func` の値である。
  BY 本補題の仮定, CODE src/rc_ir/borrow.rs: borrow_ify
<1>3. `clone_func` が作る本体の `Match` は、入力の本体の `Match` と 1 対 1 に対応し、対応する 2 つの
      `Match` のアームの個数は等しい。
  <2>1. `clone_func` が作る本体は `fresh_rename_function` が返す第 3 の値であり、それは
        `rename_expr(body, &renaming)` の値である。ここで `body` は入力の関数の本体である。
    BY CODE src/rc_ir/borrow.rs: clone_func, CODE src/rc_ir/rename.rs: fresh_rename_function
  <2>2. `rename_expr(node, renaming)` の本文は `grow_stack(|| rename_expr_inner(node, renaming))` で
        ある。A15 より、この呼び出しは `rename_expr_inner(node, renaming)` をちょうど 1 回呼んでその値を
        返す。
    BY CODE src/rc_ir/rename.rs: rename_expr, A15
  <2>3. `rename_expr_inner(node, renaming)` が返す節点の式は、`node` の式と同じ `RcExpr` の変位である。
        `RcExpr::Let(x, rhs, k)` の腕は右辺を `rename_rhs(rhs, renaming)`、継続を
        `rename_expr(k, renaming)` にする。`Retain`、`Release`、`Destructure`、`Eval` の 4 つの腕は
        右辺を持たず、継続を `rename_expr(k, renaming)` にする。`Ret` の腕は継続を持たない。
    BY CODE src/rc_ir/rename.rs: rename_expr_inner, CODE src/rc_ir/ast.rs: RcExpr
  <2>4. `rename_rhs(rhs, renaming)` が返す右辺は、`rhs` と同じ `RcRhs` の変位である。
        `RcRhs::Match(scrut, arms)` の腕は、アームの列を
        `arms.iter().map(|arm| MatchArm { ..., body: rename_expr(&arm.body, renaming) }).collect()` で
        作る。EXT Iterator::map と collect よりこの列の要素数は `arms` の要素数に等しく、第 `i` アームの
        本体は `arms[i].body` について `rename_expr` が返した木である。残る 4 変位 (`Var`、`App`、
        `Closure`、`Llvm`) は `Match` を作らず、アームを持たない。
    BY CODE src/rc_ir/rename.rs: rename_rhs, CODE src/rc_ir/ast.rs: RcRhs,
       EXT Vec::iter と slice::iter, EXT Iterator::map と collect
  <2>5. QED
    木 `N(body)` の構造についての帰納法で示す。L0a の 4 よりこの帰納法は整礎である。<2>2 より
    `rename_expr(n, renaming)` は `rename_expr_inner(n, renaming)` の値である。<2>3 と
    <2>4 より、その値の節点は `n` と同じ式の変位を持ち、その子 (`Match` のアーム本体と継続) は、`n` の
    対応する子について `rename_expr` が返した木である。よって帰納法の仮定と合わせて、返る木の位置は `n` の
    木の位置と 1 対 1 に対応し、対応する位置の式は同じ変位であり、<2>4 より `Match` のアームの個数は
    等しい。<2>1 でこれを `body` に適用する。
    BY <2>1, <2>2, <2>3, <2>4, L0a, DEF 部分木
<1>4. 本補題の仮定の `borrow_ify` の呼び出しの入力プログラムのすべての `Match` は 1 つ以上のアームを持つ。
  BY A9, 本補題の仮定
<1>5. QED
  本補題の仮定は L3 の仮定であり、L3 より `cancel(prog, type_env)` が走査する本体は `rewrite` の出力で
  ある。<1>2 よりその入力は `borrow_ify` の入力プログラムの本体か、その複製である。<1>3 より複製はアームの
  個数を保ち、<1>1 より `rewrite` も保つ。<1>4 より元の個数は 1 以上である。
  BY L3, <1>1, <1>2, <1>3, <1>4, 本補題の仮定

## 3. P15 (節点と `NodeId`)

**言明** --- `cancel` の入力すなわち `borrow_ify` の出力の各本体について、相異なる位置は相異なる `NodeId`
を持つ。また `CancelAnalysis::walk` の 1 回の呼び出し `walk(n, ・, ・)` は、`n` の部分木の各位置をちょうど
1 回訪れ、その外の位置を訪れない。本体の根についてこれを読めば、走査は本体の各位置をちょうど 1 回訪れる。

前半は、`cancel` が走査する本体、すなわち `cancel` の入力プログラムの各関数の `body` と各グローバル
初期化子の `init` について示す。D2 が述べるとおり `RcExprNode` は式を `Arc` で共有できるので、これは
`RcExprNode` 一般の性質ではなく、`cancel` に渡される木の性質である。

### 3.1 前半 (相異なる位置は相異なる `NodeId` を持つ)

<1>1. `node_id(node)` は、`node.expr` の割り当ての占める番地の 1 つである。`node_id` の本文は
      `node.expr.as_ref() as *const RcExpr as NodeId` であり、`node.expr` の型は `Arc<RcExpr>` である。
      EXT Arc の契約 の最後の行より、`as_ref` が返す参照の番地は、そのブロックの占める番地の 1 つで
      ある。この行の仮説
      「`T` のサイズが 0 でない」は満たされる --- `RcExpr` の変位 `Ret(RcVar)` は `RcVar` の値を保持し、
      `RcVar` は `skip_null_check: bool` のフィールドを持つ。EXT bool のサイズ より `bool` のサイズは
      1 であり、EXT 型のサイズ より `RcVar` のサイズは 1 以上、`RcExpr` のサイズも 1 以上である。
  BY CODE src/rc_ir/borrow.rs: node_id, CODE src/rc_ir/ast.rs: RcExprNode,
     CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcVar, EXT Arc の契約, DEF 割り当て,
     EXT 型のサイズ, EXT bool のサイズ
<1>2. 同時に生存している相異なる 2 つの割り当てについて、一方の占める番地はどれも他方の占める番地と
      相異なる。割り当てが生存しているとは、そのハンドルが 1 つ以上存在することであり (DEF 割り当て)、
      EXT Arc の契約 より、ハンドルが 1 つでも在るあいだそのブロックはアロケータへ返らない。よって
      同時に生存している 2 つの割り当てのブロックはどちらもアロケータへ返っておらず、
      EXT アロケータの契約 よりその 2 つは記憶域を共有しない。すなわち一方の占める番地の集合と
      他方の占める番地の集合は交わらない。
  BY DEF 割り当て, EXT Arc の契約, EXT アロケータの契約
<1>2a. 2 つの節点 `p`、`q` の `p.expr` と `q.expr` の割り当てが相異なり、かつ同時に生存しているならば、
       `node_id(p) ≠ node_id(q)` である。<1>1 より `node_id(p)` は `p.expr` の割り当ての占める番地であり、
       `node_id(q)` は `q.expr` の割り当ての占める番地であって、<1>2 よりこの 2 つは相異なる。
  BY <1>1, <1>2
<1>3. `RewriteCtx::rewrite(node)` の 1 回の呼び出しが返す木の各位置は、その呼び出しの間に `expr_node` が
      実行した `Arc::new` の呼び出しが取った割り当てを持ち、**相異なる位置の割り当ては相異なる `Arc::new`
      の呼び出しが取ったもの、すなわち相異なる割り当てである**。さらに、それらの割り当てはその呼び出しが
      返るまですべて生存している。
  木 `N(node)` の構造についての帰納法で示す。L0a の 4 よりこの帰納法は整礎である。
  <2>0a. 帰納法の仮定: `node` の各子 `c` について、`rewrite(c)` の 1 回の呼び出しが返す木の各位置は、その
         呼び出しの間に `expr_node` が実行した `Arc::new` の呼び出しが取った割り当てを持ち、相異なる位置の
         割り当ては相異なり、それらはその呼び出しが返るまですべて生存している。
    BY 帰納法の仮定
  <2>5. `rewrite` の 1 回の呼び出しについて、次の 2 つが成り立つ。**(a)** この呼び出しの中で `expr_node`
        が作った各節点と、この呼び出しが `self.rewrite` を呼んで受け取った各木の各位置は、この呼び出しが
        返す木の位置である。**(b)** それらの割り当ては、それが作られた時点からこの呼び出しが返るまで
        生存している。とくに、この呼び出しとその下の呼び出しが `expr_node` で作った割り当ては 1 つも
        解放されない。
    <3>0. 木の根の `RcExprNode` の値が 1 つ在るあいだ、その木のすべての位置の `Arc` のハンドルが 1 つ以上
          存在し、その割り当ては生存している。
      BY L2a
    <3>1. (a) が成り立つ。L3a の 2 より、`rewrite_inner(node)` が返す木の位置は、`self.rewrite` の各
          呼び出しが返した木の位置と、この呼び出しの中で `expr_node` が作った節点の全体である。L1 より
          `rewrite(node)` はその値をそのまま返す。
      BY L3a, L1
    <3>1a. この呼び出しの中で `expr_node` が作った各節点は作られた時点から、この呼び出しが
           `self.rewrite` を呼んで受け取った各木は受け取った時点から、どちらもこの呼び出しが返るまで、
           その割り当てが生存している。`rewrite_inner` の各腕は、`expr_node` が作った節点と
           `self.rewrite` が返した木を、局所変数に束縛して保持し、その上に積む節点の子として、または
           戻り値として据える --- `rewrite_rc` は `self.rewrite(k)` の値を `k` に束縛し、`kept` を
           計算してから `fold` の初期値に据えるので、そのあいだ `k` がハンドルを保持する。どの腕も、
           作った節点も受け取った木も捨てない。よってそれらはその時点からこの呼び出しが返るまで、
           組み立て中の木の部分木の根であり続け、<3>0 よりその部分木のすべての位置の割り当てが生存して
           いる。
      BY L3a, <3>0, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, CODE src/rc_ir/borrow.rs: prepend_rc,
         DEF 割り当て
    <3>2. `arm.with_body(body)` の中の `self.clone()` は入力側のアームの本体の `Arc` のハンドルを 1 つ作り、
          そのハンドルは `MatchArm` の構築で `body` に置き換えられて drop される。drop されるのは入力の木の
          節点のハンドルであり、`expr_node` が作った割り当てのものではない。
      BY CODE src/rc_ir/ast.rs: MatchArm::with_body,
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
         L3a, DEF 割り当て
    <3>3. QED
      <3>1 が (a) を与える。(b) のうち、この呼び出しの中で `expr_node` が作った節点については <3>1a が
      そのまま与える。その下の呼び出しが `expr_node` で作った割り当てについては、<2>0a よりそれはその子の
      呼び出しが返した木の位置の割り当てであって、その子の呼び出しが返るまで生存し、<3>1a よりそれを
      受け取った時点からこの呼び出しが返るまで生存する。2 つの区間は隙間なく繋がるので、作られた時点から
      この呼び出しが返るまで生存している。<3>2 は、`Match` の腕が唯一 drop するハンドルが入力の木のもので
      あることを言う。
      BY <2>0a, <3>0, <3>1, <3>1a, <3>2
  <2>6. QED
    <2>0a の帰納法の仮定より、各子について `rewrite` が返す木の相異なる位置の割り当ては、相異なる
    `Arc::new` の呼び出しが取ったものである。L3a の 2 より `rewrite_inner(node)` が返す木の位置は、各子に
    ついて `rewrite` が返した木の位置と、この呼び出しの中で `expr_node` が新しく作った節点の全体である。
    **相異なる 2 つの子 `c`、`c'` について、この呼び出しが行う `rewrite(c)` と `rewrite(c')` の実行区間は
    交わらない** (L3a の 3、L1、EXT 呼び出しの入れ子)。よって一方の実行区間の中で実行された `Arc::new` は
    他方の実行区間の中では実行されておらず、<2>0a より 2 つの子の木の割り当ては相異なる。**この呼び出しの
    中で `expr_node` が実行する `Arc::new` は、どの子の `rewrite` の実行区間の中にも無い** (L3a の 3)
    ので、子の木の割り当てとも、互いとも相異なる。L1 より `rewrite(node)` は `rewrite_inner(node)` の値を
    そのまま返す。生存については <2>0a と <2>5 の (b) が与える。
    BY <2>0a, <2>5, L1, L3a, DEF 割り当て, EXT 呼び出しの入れ子
<1>4. `cancel_body(vars, body)` の実行中、`body` の木の割り当てはすべて生存している。
  <2>1. `cancel` の `funcs` を作る閉包は、`prog.funcs.values()` の各 `f` について
        `clone.body = cancel_body(&vars, &f.body)` を実行する。`cancel_body` の実行中、`f` は `prog` から
        借用されているので `f.body` の `RcExprNode` の値が在り、L2a よりその木のすべての位置の `Arc` の
        ハンドルが 1 つ以上存在して、その割り当ては生存している。
    BY CODE src/rc_ir/borrow.rs: cancel, L2a
  <2>2. `cancel` の `globals` を作る閉包は、`prog.globals` の各 `g` について
        `cancel_body(&vars, &g.init)` を実行する。`cancel_body` の実行中、`g` は `prog` から借用されて
        いるので `g.init` の `RcExprNode` の値が在り、L2a よりその木のすべての位置の `Arc` のハンドルが
        1 つ以上存在して、その割り当ては生存している。
    BY CODE src/rc_ir/borrow.rs: cancel, L2a
  <2>3. QED
    BY <2>1, <2>2
<1>5. QED
  P15 の言明は `cancel` の入力を `borrow_ify` の出力に限るので、L3 の仮定が満たされる。L3 より、`cancel`
  が走査する各本体は `RewriteCtx::rewrite` の 1 回の呼び出しが返した木である。<1>3 よりその木の相異なる
  位置は相異なる割り当てを持ち、<1>4 よりそれらは走査の間ずっと生存している。よって <1>2a より、相異なる
  位置の `NodeId` は相異なる。
  BY L3, <1>2a, <1>3, <1>4

### 3.2 後半 (`walk` は部分木の各位置をちょうど 1 回訪れる)

<1>1. `CancelAnalysis::walk` と `CancelAnalysis::walk_inner` はどちらも `CancelAnalysis` の非公開の
      メソッドであり、`CancelAnalysis` は `borrow.rs` の非公開の型である。`borrow.rs` は `mod` 宣言を
      1 つも持たないので、この 2 つが見える子モジュールも無い。よって `walk` と `walk_inner` の呼び出しは
      `borrow.rs` の中にしか書けない。**この文書が「訪問」と呼ぶのは `walk_inner` の 1 回の呼び出しで
      ある (DEF 訪問) ので、閉じるべきなのは `walk` の呼び出し元だけではなく `walk_inner` の呼び出し元でも
      ある。**
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, DEF 訪問, DEF 本文,
     EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>1a. `borrow.rs` の中で `CancelAnalysis::walk` を呼ぶのは、`cancel` の中の
       `analysis.walk(body, PendingRetains::default(), true)` と、`walk_inner` の中の 7 か所だけである。
       その 7 か所は、`Retain` の腕、`Release` の腕、`Match` の腕の 2 か所 (アームと継続)、
       右辺が `Match` でない `Let` の腕、`Destructure` の腕、`Eval` の腕である。`Match` の腕のアームの側の
       1 か所は `.map(|arm| self.walk(&arm.body, pending.clone(), false))` のクロージャの中に書かれて
       いるが、DEF 本文 よりそれは `walk_inner` の本文の一部である。また `borrow.rs` の中で
       `CancelAnalysis::walk_inner` を呼ぶのは、`walk` の本文の `grow_stack` に渡す閉包の 1 か所だけで
       ある。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, CODE src/rc_ir/borrow.rs: CancelAnalysis,
     DEF 本文
<1>2. `walk_inner` の 1 回の呼び出しの中で `walk` と `walk_inner` の呼び出しが起きるのは、`walk_inner` の
      本文に書かれた (DEF 本文) `self.walk(...)` の 7 か所を通ってだけである。すなわち、`walk_inner` が呼ぶ
      `walk` 以外の関数は、それが直接呼ぶものも、その先で呼ばれるものも、`walk` も `walk_inner` も
      呼ばない。
  <2>1. `walk_inner` の本文が呼ぶ関数のうち、`self.walk` 以外で `borrow.rs` で定義されているのは次の
        9 つである。`node_id`、`CancelAnalysis::acted_references`、`CancelAnalysis::other_objects`、
        `CancelAnalysis::consume_objects`、`CancelAnalysis::consume`、`CancelAnalysis::consume_rhs`、
        `CancelAnalysis::merge`、`un_bump`、および `PendingRetain` が derive する `Clone::clone`
        (`Match` の腕の `pending.clone()` が `Vec` の要素ごとに呼ぶ)。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, CODE src/rc_ir/borrow.rs: PendingRetain
  <2>1a. `walk_inner` の本文に現れる呼び出しのうち、`self.walk` と <2>1 の 9 つ以外はすべて `borrow.rs` の
         外で定義されている。本文に現れる呼び出しは次で尽きる。DEF 本文 より、`.map(|arm| ...)` に渡す
         クロージャの中に書かれた呼び出しもこの数え上げに入っている。

         | 式 | 解決する項目 | 定義の場所 |
         |---|---|---|
         | `node.expr.as_ref()` | `<Arc<RcExpr> as AsRef<RcExpr>>::as_ref` | 標準ライブラリ |
         | `node_id(node)` (2 か所) | `node_id` | `borrow.rs` (<2>1) |
         | `self.all_retains.push`、`pending.push`、`...or_default().push` | `Vec::push` | 標準ライブラリ |
         | `self.un_bump_releases.entry(...)` (2 か所) | `Map::entry` | 標準ライブラリ |
         | `.or_default()` (2 か所) | `Entry::or_default` | 標準ライブラリ |
         | `self.acted_references(v, path)` (2 か所) | `CancelAnalysis::acted_references` | `borrow.rs` (<2>1) |
         | `self.walk(...)` (7 か所) | `CancelAnalysis::walk` | `borrow.rs` (本ステップの対象外) |
         | `self.other_objects(v, path)` | `CancelAnalysis::other_objects` | `borrow.rs` (<2>1) |
         | `self.consume_objects(...)` (2 か所) | `CancelAnalysis::consume_objects` | `borrow.rs` (<2>1) |
         | `un_bump(&mut pending, &un_bumped)` | `un_bump` | `borrow.rs` (<2>1) |
         | `un_bumped.objects()` | `References::objects` | `ownership.rs` |
         | `arms.iter()` | `<[MatchArm]>::iter` | 標準ライブラリ |
         | `.map(|arm| ...)` | `Iterator::map` | 標準ライブラリ |
         | `pending.clone()` | `<Vec<PendingRetain> as Clone>::clone` (要素ごとに <2>1 の `PendingRetain` の `clone` を呼ぶ) | 標準ライブラリ |
         | `.collect()` | `Iterator::collect` | 標準ライブラリ |
         | `self.merge(&pending, &arm_exits)` | `CancelAnalysis::merge` | `borrow.rs` (<2>1) |
         | `self.consume_rhs(&mut pending, rhs, &x.ty)` | `CancelAnalysis::consume_rhs` | `borrow.rs` (<2>1) |
         | `destructure_consumes(container, fields, self.type_env)` | `destructure_consumes` | `ownership.rs` |
         | `for leaf in destructure_consumes(...)` | `<Vec<FieldPath> as IntoIterator>::into_iter` と `Iterator::next` | 標準ライブラリ |
         | `self.consume(&mut pending, &container.name, &leaf)` | `CancelAnalysis::consume` | `borrow.rs` (<2>1) |
         | `for retain in &pending` | `<&Vec<PendingRetain> as IntoIterator>::into_iter` と `Iterator::next` | 標準ライブラリ |
         | `self.needed_retains.insert(retain.node)` | `Set::insert` | 標準ライブラリ |

         `Map` と `Set` は `FxHashMap` と `FxHashSet` の別名である。`for` の脱糖が `IntoIterator::into_iter`
         と `Iterator::next` を呼ぶことは EXT IntoIterator と for が述べる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/ownership.rs: References::objects,
       CODE src/rc_ir/ownership.rs: destructure_consumes, CODE src/misc.rs: Map, CODE src/misc.rs: Set,
       EXT IntoIterator と for, EXT Vec::iter と slice::iter, DEF 本文
  <2>2. <2>1 の 9 つの本文は、`self.walk(...)` も `self.walk_inner(...)` も持たない。さらに、**この 9 つの
        本文に書かれた呼び出しのうち `borrow.rs` の項目に解決するのは、`consume` が呼ぶ
        `consume_objects` と `consume_rhs` が呼ぶ `consume` の 2 つだけであり、残りはすべて `borrow.rs` の
        外で定義された項目に解決する。** DEF 本文 より、この 9 つが引数として渡すクロージャの本文は
        それを書いた関数の本文の一部なので、その中に書かれた呼び出しもこの数え上げに入っている ---
        渡す先が標準ライブラリであっても `borrow.rs` の外の項目であっても同じである。
        `PendingRetain` が derive する `Clone::clone` の本文はフィールドごとの `clone` であり、その
        フィールドの型は `usize` と `References` で、どちらの `Clone` の実装も `borrow.rs` の外にある。
    BY <1>1a, DEF 本文, CODE src/rc_ir/borrow.rs: node_id,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, CODE src/rc_ir/borrow.rs: un_bump,
       CODE src/rc_ir/borrow.rs: PendingRetain, CODE src/rc_ir/ownership.rs: References, EXT Clone
  <2>2a. `borrow.rs` の外で定義された関数の本文は `borrow.rs` の中に無いので、<1>1 より `walk` も
         `walk_inner` も呼べない。DEF 本文 より、`borrow.rs` の中に書かれたクロージャを引数に取る
         標準ライブラリの関数についても同じである --- そのクロージャの本文は、それを書いた `borrow.rs` の
         関数の本文の一部であって、受け取った側の本文の一部ではない。そうしたクロージャを持つ本文は
         `walk_inner` の本文 (<2>1a) と <2>1 の 9 つ (<2>2) で尽きており、どちらの数え上げもそれを
         含んでいる。
    BY <1>1, <2>1, <2>1a, <2>2, DEF 本文
  <2>3. QED
    `walk_inner` の 1 回の呼び出しから始まる呼び出しの閉包は、`walk_inner` の本文、<2>1 の 9 つの本文、
    および `borrow.rs` の外で定義された関数の本文だけからなる (<2>1、<2>1a、<2>2、DEF 本文)。<2>2 より
    その 9 つのどれも `walk` も `walk_inner` も呼ばず、<2>2a より `borrow.rs` の外の関数はどちらも
    呼べない。よって `walk` と `walk_inner` の呼び出しは、`walk_inner` の本文に書かれた
    `self.walk(...)` の 7 か所 (<1>1a) を通ってだけ起きる。
    BY <1>1, <1>1a, <2>1, <2>1a, <2>2, <2>2a, DEF 本文
<1>3. 任意の節点 `n`、任意の `pending`、任意の `returns_from_func` について、
      `walk(n, pending, returns_from_func)` の 1 回の呼び出しの実行中、`N(n)` の各節点はちょうど 1 回
      訪問され、`N(n)` の外の節点は訪問されない。
  木 `N(n)` の構造についての帰納法で示す。L0a の 4 よりこの帰納法は整礎である。
  <2>1. 帰納法の仮定: `n` の各子 `c` と任意の引数について、`walk(c, ・, ・)` の 1 回の呼び出しの実行中、
        `N(c)` の各節点はちょうど 1 回訪問され、`N(c)` の外の節点は訪問されない。
    BY 帰納法の仮定
  <2>2. `walk(n, pending, returns_from_func)` は `walk_inner` を、`node` 引数を `n` としてちょうど
        1 回呼ぶ。すなわちこの呼び出しは `n` をちょうど 1 回訪問する。
    BY L1
  <2>2a. `walk(n, pending, returns_from_func)` の 1 回の呼び出しの中で起きる訪問 (DEF 訪問、すなわち
         `walk_inner` の 1 回の呼び出し) は、<2>2 のこの呼び出し自身の 1 回と、この呼び出しの中で起きる
         `walk` の呼び出しの中で起きる訪問だけである。<1>2 より、`walk_inner` の本文から `walk` と
         `walk_inner` の呼び出しが起きるのは本文に書かれた (DEF 本文) `self.walk(...)` を通ってだけであり、
         L1 と <1>1a より `walk` はその本文の 1 か所で `walk_inner` を 1 回呼ぶほかは `walk` も
         `walk_inner` も呼ばない。
    BY <1>1a, <1>2, L1, DEF 訪問, DEF 本文
  <2>3. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。
    <3>1. この腕は `self.walk(k, pending, returns_from_func)` を 1 回呼び、ほかに `walk` も `walk_inner` も呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。L0a の 4 より `n` は
          `N(k)` に入らない。
      BY L0a, DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <2>2a, <3>1, <3>2
  <2>4. CASE `n` の式が `RcExpr::Release(v, path, _, k)` である。
    <3>1. この腕は `self.walk(k, pending, returns_from_func)` を 1 回呼び、ほかに `walk` も `walk_inner` も呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。L0a の 4 より `n` は
          `N(k)` に入らない。
      BY L0a, DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <2>2a, <3>1, <3>2
  <2>5. CASE `n` の式が `RcExpr::Let(_, RcRhs::Match(_, arms), k)` である。
    <3>1. この腕は `arms.iter().map(|arm| self.walk(&arm.body, pending.clone(), false)).collect()` で
          各 `arm` について `self.walk(&arm.body, ・, ・)` を呼び、その後
          `self.walk(k, merged, returns_from_func)` を 1 回呼ぶ。ほかに `walk` も `walk_inner` も呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
         <1>2
    <3>2. `Iterator::map` の閉包は `collect` によって各要素についてちょうど 1 回、先頭から順に呼ばれる。
          よって <3>1 の `self.walk(&arm.body, ・, ・)` は `arms` の各要素についてちょうど 1 回である。
      BY EXT Vec::iter と slice::iter, EXT Iterator::map と collect
    <3>3. `n` の子は `arms` の各 `arm.body` と `k` であり、`N(n)` は `{n}` とそれらの部分木の非交和で
          ある。L0a の 3 より相異なる子の部分木は交わらず、L0a の 4 より `n` はどの子の部分木にも
          入らない。
      BY L0a, DEF 部分木
    <3>4. QED
      BY <2>1, <2>2, <2>2a, <3>1, <3>2, <3>3
  <2>6. CASE `n` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。
    <3>1. この腕は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼び、その後
          `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。ほかに `walk` も `walk_inner` も呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。L0a の 4 より `n` は
          `N(k)` に入らない。
      BY L0a, DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <2>2a, <3>1, <3>2
  <2>7. CASE `n` の式が `RcExpr::Destructure(container, fields, _state, k)` である。
    <3>1. この腕は `destructure_consumes(container, fields, self.type_env)` の各要素について
          `self.consume` を呼び、その後 `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。ほかに
          `walk` も `walk_inner` も呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
         <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。L0a の 4 より `n` は
          `N(k)` に入らない。
      BY L0a, DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <2>2a, <3>1, <3>2
  <2>8. CASE `n` の式が `RcExpr::Eval(_, k)` である。
    <3>1. この腕は `self.walk(k, pending, returns_from_func)` を 1 回呼び、ほかに `walk` も `walk_inner` も呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Eval(_, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。L0a の 4 より `n` は
          `N(k)` に入らない。
      BY L0a, DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <2>2a, <3>1, <3>2
  <2>9. CASE `n` の式が `RcExpr::Ret(_)` である。
    <3>1. この腕は `walk` も `walk_inner` も呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕, <1>2
    <3>2. `n` に子は無く、`N(n)` は `{n}` である。
      BY DEF 部分木
    <3>3. QED
      BY <2>2, <2>2a, <3>1, <3>2
  <2>10. QED
    `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を、<2>3 から <2>9 が尽くす。
    `walk_inner` の `match` の腕もこの 7 つであり、`Let` の 2 つの腕はこの順に並んでいるので、右辺が
    `Match` の `Let` は第 1 の腕に、それ以外の `Let` は第 2 の腕に入る。
    BY <2>3, <2>4, <2>5, <2>6, <2>7, <2>8, <2>9, CODE src/rc_ir/ast.rs: RcExpr,
       CODE src/rc_ir/ast.rs: RcRhs, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
<1>4. QED
  <1>3 を `n` を本体の根として適用する。`cancel_body` は `analysis.walk(body, ・, ・)` を 1 回だけ呼ぶ。
  BY <1>3, CODE src/rc_ir/borrow.rs: cancel

## 4. 基本操作の補題

### L5 (`un_bump` の作用)

要素 `e` が `References` の値 `R` と**位置を共有する**とは、`e.outstanding.shares_an_object(R)`
が真であることをいう。走査が扱う `References` の値については、これは `e.outstanding` と `R` の双方が
参照を持つ位置 (`VarPath`) が在ることと同値である (L2 の 3)。

`un_bump(pending, un_bumped)` の 1 回の呼び出しについて、次の 3 つが成り立ち、この 3 つは場合を尽くす。

1. `un_bumped` と位置を共有する要素が `pending` に無いとき、返り値は `NoBracket` であり、
   `pending` は変わらない。
2. あるとき、そのような要素の添字のうち最大のものを `i` とする。`pending[i].outstanding.covers(un_bumped)`
   が偽のとき、返り値は `OutsideBracket` であり、`pending` は変わらない。
3. 真のとき、返り値は `InBracket(pending[i].node)` である。`pending[i].outstanding` は
   `pending[i].outstanding - un_bumped` になり、それが空なら `pending[i]` が取り除かれる。ほかの要素の
   `node` と `outstanding` は変わらず、要素の相対順序も変わらない。

**証明**

<1>0. `un_bump` の呼び出しはどれも走査の中のものであり、`pending` の各要素の `outstanding` と
      `un_bumped` はどれも走査が扱う `References` の値である。よって L2 の 1 から 4 がこの 2 つに
      当たり、上の「位置を共有する」は L2 の 3 により、双方が参照を持つ位置が在ることと同値である。
      `un_bump` は `borrow.rs` の非公開の自由関数であり、`borrow.rs` は `mod` 宣言を 1 つも持たないので、
      その呼び出しは `borrow.rs` の中にしか書けない。`borrow.rs` の中で `un_bump` を呼ぶのは
      `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕 1 か所だけであり、その第 1 引数はその時点の
      走査の状態、第 2 引数は `self.acted_references(v, path)` の値である。
  BY CODE src/rc_ir/borrow.rs: un_bump,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     L2, DEF 本文, EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>1. `pending.iter().rposition(|retain| retain.outstanding.shares_an_object(un_bumped))` は、述語を
      満たす要素の添字のうち最大のものを `Some` で返し、そのような要素が無ければ `None` を返す。
      `let Some(index) = ... else { return UnBump::NoBracket; };` により、`None` のとき `NoBracket` を
      返す。この文は本文の最初の文であり、EXT 文は書かれた順に実行される よりこの文までに `pending` を
      変える操作は無い。
  BY CODE src/rc_ir/borrow.rs: un_bump, EXT Vec::iter と slice::iter, EXT Iterator::rposition,
     EXT 文は書かれた順に実行される
<1>2. `let innermost = &mut pending[index];` は添字 `index` の要素への可変参照である。
      `if !innermost.outstanding.covers(un_bumped) { return UnBump::OutsideBracket; }` により、`covers`
      が偽のとき `OutsideBracket` を返す。この 2 文は <1>1 の文の直後に書かれており、
      EXT 文は書かれた順に実行される よりこの文までに `pending` を変える操作は無い。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>1, EXT 文は書かれた順に実行される
<1>3. `covers` が真のとき、`innermost.outstanding.subtract(un_bumped)` は panic せず、
      `innermost.outstanding` を `pending[index].outstanding - un_bumped` に書き換える。
  BY CODE src/rc_ir/borrow.rs: un_bump, L2, <1>0
<1>4. `let retain = innermost.node;` は <1>3 の文の後に書かれており、
      EXT 文は書かれた順に実行される よりその後に実行される。`subtract` は `outstanding` しか変え
      ないので、`retain` は書き換え前後で同じ `pending[index].node` である。
  BY CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/ownership.rs: References::subtract,
     CODE src/rc_ir/borrow.rs: PendingRetain, <1>3, EXT 文は書かれた順に実行される
<1>5. `if innermost.outstanding.is_empty() { pending.remove(index); }` は、L2 の 1 より <1>3 の差が空の
      ときちょうど添字 `index` の要素を取り除き、空でないとき何もしない。`Vec::remove` は後続の要素を
      1 つずつ前へ詰めるだけなので、残る要素の値と相対順序は変わらない。
  BY CODE src/rc_ir/borrow.rs: un_bump, L2, <1>0, EXT Vec::remove
<1>6. `UnBump::InBracket(retain)` を返す。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>4
<1>7. <1>1 から <1>6 の間に `pending` に触れるのは <1>3 と <1>5 だけであり、どちらも添字 `index` の
      要素にしか触れない。本文の文はこの順に書かれており、EXT 文は書かれた順に実行される より
      ほかの文がそのあいだに実行されることはない。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>3, <1>5, EXT 文は書かれた順に実行される
<1>8. QED
  場合分けは「共有する要素が無い」「あって `covers` が偽」「あって `covers` が真」であり、尽くしている。
  BY <1>0, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7

### L6 (消費の作用)

`CancelAnalysis::consume_objects(pending, objects)` が `pending` に対して行うのは次のことである。
`objects` のいずれかについて `outstanding.names` が真である要素をすべて取り除き、取り除いた各要素の
`node` を `self.needed_retains` に入れる。残る要素の値と並びは変わらない。

さらに、`CancelAnalysis::consume`、`CancelAnalysis::consume_rhs`、および `walk_inner` の
`RcExpr::Destructure(container, fields, _state, k)` の腕が `pending` に対して行うのは、
`consume_objects` の呼び出しだけである。

**証明**

<1>1. `consume_objects` の本文は `pending.retain(|retain| { if objects.iter().any(|object|
      retain.outstanding.names(object)) { self.needed_retains.insert(retain.node); return false; } true })`
      である。`Vec::retain` は閉包が偽を返した要素を取り除き、残る要素の値と相対順序を保つ。閉包が
      `self.needed_retains` に入れるのは偽を返す枝でだけであり、その枝を通るのは
      `objects.iter().any(...)` が真の要素である。この関数はほかに `pending` にも `needed_retains` にも
      触れない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects, EXT Vec::retain,
     EXT Vec::iter と slice::iter, EXT Iterator::all と any
<1>2. `consume(pending, var, path)` は `origin(self.vars, self.type_env, var, path).acted_on()` から
      `objects` を作り、`self.consume_objects(pending, &objects)` を 1 回呼ぶ。ほかに `pending` に
      触れない。`origin` と `Origin::acted_on` は `pending` を引数に取らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, CODE src/rc_ir/ownership.rs: origin,
     CODE src/rc_ir/ownership.rs: Origin::acted_on
<1>3. `consume_rhs(pending, rhs, result_ty)` は `rhs_consumes` を呼んで `consumed` を集め、その各要素
      `(var, leaf)` について `self.consume(pending, &var, &leaf)` を呼ぶ。ほかに `pending` に触れない。
      `rhs_consumes` は `pending` を引数に取らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, CODE src/rc_ir/ownership.rs: rhs_consumes
<1>4. `walk_inner` の `RcExpr::Destructure(container, fields, _state, k)` の腕は、
      `destructure_consumes(container, fields, self.type_env)` の各 `leaf` について
      `self.consume(&mut pending, &container.name, &leaf)` を呼ぶ。ほかに `pending` に触れない。
      `destructure_consumes` は `pending` を引数に取らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
     CODE src/rc_ir/ownership.rs: destructure_consumes
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L7 (`merge` の作用)

`self.merge(pending_in, arm_exits)` の 1 回の呼び出しについて、各 `arm_exits[j]` の相異なる要素は相異なる
`node` を持つとする。このとき次が成り立つ。

1. `arm_states[j]` は `arm_exits[j]` の各要素 `e` を `e.node` から `&e.outstanding` へ写す表であり、
   その鍵の集合は `arm_exits[j]` の要素の `node` の集合に等しい。
2. `entered_with` は `pending_in` の要素の `node` の集合である。
3. `NodeId` の値 `x` について、二重ループの中で `x` を `retain` とする反復の `is_uniform` の値は、その
   反復がどの `j` のものであっても等しい。その値が真であることは、次の条件 `U(x)` と同値である。
   **`x` が `entered_with` の要素であり、すべての `j'` について `arm_states[j']` が `x` を鍵に持ち、
   それらの値が互いに等しい。**
4. 呼び出しの終わりに `uniform` が `x` を鍵に持つことは、「ある `j` について `arm_states[j]` が `x` を鍵に
   持ち、かつ `U(x)`」と同値である。このとき `uniform[x]` は、その共通の値と等しい `References` である。
5. `NodeId` の値 `x` について、この呼び出しが `x` を `self.needed_retains` に入れることは、「ある `j` に
   ついて `arm_states[j]` が `x` を鍵に持ち、かつ `U(x)` が成り立たない」ことと同値である。
6. 返り値は、`pending_in` の要素のうち `node` を `uniform` が鍵に持つものを、その並びのまま、`node` は
   そのまま、`outstanding` を `uniform[node]` と等しい値に差し替えて並べた `Vec` である。ほかの要素を
   持たない。

**証明**

<1>1. `arm_states` は
      `arm_exits.iter().map(|exit| exit.iter().map(|retain| (retain.node, &retain.outstanding)).collect()).collect()`
      である。`exit.iter()` は `arm_exits[j]` のすべての要素を渡す。内側の `collect` の行き先は
      `Map<NodeId, &References>` なので、EXT collect into Map と Set より、`arm_states[j]` の鍵の集合は
      `arm_exits[j]` の要素の `node` の集合に等しく、鍵を共有する要素が複数あればその鍵の値は最後の要素の
      値になる。仮定より `arm_exits[j]` の相異なる要素は相異なる `node` を持つので、2 つの要素が 1 つの鍵に
      落ちることはなく、各鍵の値はその鍵を `node` とする唯一の要素の `outstanding` への共有参照である。
      よって 1 が成り立つ。外側の `collect` の行き先は `Vec` なので、EXT Iterator::map と collect より
      `arm_states` は `arm_exits` と同じ長さで、第 `j` 要素は `arm_exits[j]` から作られる。
      **`arm_states` の各値は走査が扱う `References` の値である。** `merge` は非公開の型
      `CancelAnalysis` の非公開のメソッドであり、`borrow.rs` は `mod` 宣言を 1 つも持たないので、その
      呼び出しは `borrow.rs` の中にしか書けず、そこでそれを呼ぶのは `walk_inner` の
      `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕 1 か所だけである。よって L2 の 1 から 6 が
      `arm_states` の値と `pending_in` の各要素の `outstanding` に当たる。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, CODE src/rc_ir/borrow.rs: CancelAnalysis,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     本補題の仮定, L2, EXT collect into Map と Set,
     EXT Vec::iter と slice::iter, EXT Iterator::map と collect, DEF 本文,
     EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>2. `entered_with` は `pending_in.iter().map(|retain| retain.node).collect()` である。行き先は
      `Set<NodeId>` なので、EXT collect into Map と Set より `entered_with` の要素の集合は
      `pending_in` の各要素の `node` の集合に等しい。よって 2 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, EXT collect into Map と Set,
     EXT Vec::iter と slice::iter, EXT Iterator::map と collect
<1>3. 3 が成り立つ。
  <2>1. 反復は `for states in &arm_states { for (&retain, &outstanding) in states { ... } }` であり、
        `outstanding` は `states` すなわちある `arm_states[j]` の `retain` の値である。`is_uniform` は
        `entered_with.contains(&retain)` と
        `arm_states.iter().all(|other| other.get(&retain) == Some(&outstanding))` の連言である。
        この等式は `References` の `PartialEq` による値の比較である (DEF 参照の多重集合)。`other.get`
        は、`other` がその鍵を持つときその値への共有参照を `Some` で、持たないとき `None` を返す
        (EXT Map と Set)。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, DEF 参照の多重集合, EXT Map と Set,
       EXT Vec::iter と slice::iter, EXT Iterator::all と any, EXT IntoIterator と for
  <2>2. 第 1 の連言肢 `entered_with.contains(&retain)` は `retain` だけで決まる。
    BY <2>1
  <2>3. CASE すべての `j'` について `arm_states[j']` が `retain` を鍵に持ち、その値が互いに等しい。
        このとき、`retain` を鍵に持つどの `arm_states[j]` の反復から見ても、`outstanding` はその共通の
        値と等しいので、第 2 の連言肢は真である。
    BY <2>1
  <2>4. CASE ある `j'` について `arm_states[j']` が `retain` を鍵に持たない。このとき
        `other.get(&retain)` は `None` であり (EXT Map と Set)、`Some(&outstanding)` と等しくないので、
        `retain` を鍵に持つどの `arm_states[j]` の反復から見ても第 2 の連言肢は偽である。
    BY <2>1, EXT Map と Set
  <2>5. CASE すべての `j'` が `retain` を鍵に持つが、2 つの `j'` の値が互いに等しくない。このとき、
        `retain` を鍵に持つどの `arm_states[j]` の反復についても、その `outstanding` と等しくない値を
        持つ `j'` が在る。すべての `j'` の値が `arm_states[j]` の値と等しいとすると、L2 の 6 より
        どの `j'` の値も各位置について `arm_states[j]` の値と同じ個数を持つので、2 つの `j'` の値は
        各位置について同じ個数を持ち、L2 の 6 よりそれらは互いに等しい --- 本場合の仮定に反する。
        よって `all` は偽であり、第 2 の連言肢は偽である。<1>1 より `arm_states` の値は走査が扱う
        `References` の値なので、L2 が当たる。
    BY <1>1, <2>1, L2, DEF 参照の多重集合
  <2>6. QED
    <2>3、<2>4、<2>5 は、`retain` を鍵に持つ `arm_states[j']` の有無と値の一致について場合を尽くす。
    3 つのどの場合でも第 2 の連言肢の値は `j` によらず、それが真であるのは <2>3 の場合に限る。<2>2 と
    合わせて、`is_uniform` の値は `j` によらず、真であることは `U(retain)` と同値である。
    BY <2>1, <2>2, <2>3, <2>4, <2>5
<1>4. 二重ループの `retain` は、ある `j` について `arm_states[j]` が鍵に持つ `NodeId` の全体を、重複を
      除いてちょうど `arm_states` の中でその鍵を持つ表の個数だけ渡る。外側のループは `&arm_states` を、
      内側のループは `states`、すなわち `&Map<NodeId, &References>` を反復し、EXT Map と Set より
      `&Map` の反復は各 (鍵, 値) の対をちょうど 1 度ずつ渡すからである。`uniform` は `insert` でだけ変わり、
      EXT Map と Set より `insert` は鍵を失わない。`is_uniform` が真の反復では
      `uniform.insert(retain, outstanding.clone())` が、偽の反復では `self.needed_retains.insert(retain)`
      が実行される。この二重ループの外で `uniform` と `self.needed_retains` は変えられない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, EXT Map と Set, EXT IntoIterator と for
<1>5. 4 が成り立つ。<1>4 より `uniform` が `x` を鍵に持つのは、`x` を `retain` とする反復があってその
      `is_uniform` が真のとき、かつそのときに限る。<1>4 より `x` を `retain` とする反復があるのは、ある
      `j` について `arm_states[j]` が `x` を鍵に持つときに限り、<1>3 よりその反復の `is_uniform` が真で
      あることは `U(x)` と同値である。真の反復で `uniform` に入るのは `outstanding.clone()` であり、
      `uniform` の値の型は `References` なので、EXT Clone よりこれは `outstanding` と等しい `References`
      の値である。<1>3 の `U(x)` よりそれは各 `arm_states[j']` の共通の値であり、`x` を `retain` とする
      真の反復が複数あってもどれも同じ値を入れる。EXT Map と Set より `insert` は既にある鍵の値を置き換え
      るだけなので、上書きの後も `uniform[x]` はその共通の値と等しい。
  BY <1>3, <1>4, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, EXT Clone, EXT Map と Set
<1>6. 5 が成り立つ。<1>4 より `self.needed_retains` に入るのは `is_uniform` が偽の反復の `retain` で
      あり、`retain` はある `j` について `arm_states[j]` が鍵に持つ `NodeId` の全体を渡る。<1>3 より
      `is_uniform` が偽であることは `U(retain)` が成り立たないことと同値である。
  BY <1>3, <1>4
<1>7. 6 が成り立つ。返り値は
      `pending_in.iter().filter_map(|retain| uniform.get(&retain.node).map(|outstanding| PendingRetain { node: retain.node, outstanding: outstanding.clone() })).collect()`
      である。`Iterator::filter_map` は要素の順序を保ち、`uniform.get` が `Some` を返す要素だけを残す。
      EXT Map と Set より `uniform.get(&k)` が `Some` を返すことと `uniform` が鍵 `k` を持つことは同値で
      あり、そのときの値は `uniform` のその鍵の値である。作られる要素は `node` が元の要素の `node`、
      `outstanding` が `uniform[node]` と等しい `References` である (EXT Clone)。`merge` はほかに返り値を
      作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, EXT Vec::iter と slice::iter,
     EXT Iterator::filter_map, EXT Iterator::map と collect, EXT Map と Set, EXT Clone
<1>8. `arm_states` の各表と `pending_in` の反復について、`arm_states` の表は `Map` (ハッシュ表) なので
      反復の順序は定まらないが、<1>3 により `is_uniform` は反復の順序によらず、<1>4 により `uniform` は
      要素を失わないので、`uniform` の最終的な内容は反復の順序によらない。返り値は `pending_in` の順序で
      作られる。
  BY <1>3, <1>4, <1>7, CODE src/misc.rs: Map, EXT Map と Set
<1>9. QED
  BY <1>1, <1>2, <1>3, <1>5, <1>6, <1>7, <1>8

### L8 (出口状態は入口状態から基本操作で得られる)

ASSUME  NEW `prog`: `RcProgram`,
        `prog` は `borrow_ify` の 1 回の呼び出しが返した値である
PROVE   `cancel(prog, type_env)` が走査する本体の各節点 `n` について、`pending_out(n)` は `pending(n)`
        から有限個の基本操作 (追加・消費・引き・併合) の列で得られる。この列を `n` の**状態の鎖**と
        呼ぶ。「複製」はこの列に現れない。また、走査が `PendingRetains` の値を作るのは DEF 基本操作 の
        6 種だけである。

**証明** 木 `N(n)` の構造についての帰納法で示す。L0a の 4 よりこの帰納法は整礎である。

<1>0. 本補題の仮定は P15 の言明の仮説である。P15 より、走査はこの本体の各位置をちょうど 1 回訪問するので、
      各節点の「訪問」(DEF 訪問) は 1 つに定まり、`pending(n)` と `pending_out(n)` はその 1 つの訪問に
      ついての値として定まる。
  BY P15, 本補題の仮定, DEF 訪問
<1>1. 帰納法の仮定: `n` の各子 `c` について、`pending_out(c)` は `pending(c)` から有限個の基本操作の列で
      得られ、その走査が `PendingRetains` の値を作るのは DEF 基本操作 の 6 種だけである。
  BY 帰納法の仮定
<1>1a. 任意の節点 `m` について、`self.walk(m, q, ・)` の 1 回の呼び出しに渡る `q` は `pending(m)` であり、
       その呼び出しの戻り値は `pending_out(m)` である。L1 よりこの呼び出しは
       `walk_inner(m, q, ・)` をちょうど 1 回呼んでその値を返し、DEF 訪問 より `walk_inner` のその
       呼び出しが `m` の訪問である。<1>0 より `m` の訪問は 1 つしかないので、その `pending` 引数が
       `pending(m)`、その戻り値が `pending_out(m)` である。
  BY L1, <1>0, DEF 訪問
<1>2. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。この腕は `pending` に「追加」を 1 回行い、
      `self.walk(k, pending, returns_from_func)` の値を返す。よって `pending(k)` は `pending(n)` に
      「追加」を 1 回行ったものであり、`pending_out(n) = pending_out(k)` である。この腕はほかに
      `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕, <1>1, <1>1a
<1>3. CASE `n` の式が `RcExpr::Release(v, path, _, k)` である。この腕は `self.other_objects` の値を
      `others` として `self.consume_objects(&mut pending, &others)` を 1 回呼び (「消費」)、
      `un_bump(&mut pending, &un_bumped)` を 1 回呼び (「引き」)、その返り値が `OutsideBracket` のとき
      さらに `self.consume_objects(&mut pending, &objects)` を 1 回呼ぶ (「消費」)。その後
      `self.walk(k, pending, returns_from_func)` の値を返す。`un_bump` の返り値についての場合分けは
      `UnBump` の 3 変位を尽くしており、`InBracket` の枝が触れるのは `self.un_bump_releases` だけ、
      `NoBracket` の枝 (`UnBump::NoBracket => {}`) は何もしない。EXT 文は書かれた順に実行される より
      この 4 つはこの順に実行される。よって `pending(k)` は `pending(n)` に
      有限個の基本操作を行ったものであり、`pending_out(n) = pending_out(k)` である。この腕はほかに
      `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: UnBump, L6, <1>1, <1>1a, EXT 文は書かれた順に実行される
<1>4. CASE `n` の式が `RcExpr::Let(_, RcRhs::Match(_, arms), k)` である。この腕は各アームについて
      `pending.clone()` (「複製」) を渡して `walk` を呼び、`pending` 自身は変えない。その後
      `self.merge(&pending, &arm_exits)` (「併合」) で `merged` を作り、
      `self.walk(k, merged, returns_from_func)` の値を返す。よって `pending(k)` は `pending(n)` に
      「併合」を 1 回行ったものであり、`pending_out(n) = pending_out(k)` である。アーム本体の入口状態は
      「複製」で作られた別の値であり、この列には現れない。この腕はほかに `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕, <1>1, <1>1a
<1>5. CASE `n` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。この腕は
      `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼び (L6 より 0 個以上の「消費」)、
      `self.walk(k, pending, returns_from_func)` の値を返す。よって `pending(k)` は `pending(n)` に
      0 個以上の「消費」を行ったものであり、`pending_out(n) = pending_out(k)` である。ほかに
      `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, L6, <1>1, <1>1a
<1>6. CASE `n` の式が `RcExpr::Destructure(container, fields, _state, k)` である。この腕は
      `self.consume` を 0 回以上呼び (L6 より「消費」)、`self.walk(k, pending, returns_from_func)` の
      値を返す。よって `pending(k)` は `pending(n)` に 0 個以上の「消費」を行ったものであり、
      `pending_out(n) = pending_out(k)` である。ほかに `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
     L6, <1>1, <1>1a
<1>7. CASE `n` の式が `RcExpr::Eval(_, k)` である。この腕は `pending` を変えずに
      `self.walk(k, pending, returns_from_func)` の値を返す。よって `pending(k) = pending(n)` であり、
      `pending_out(n) = pending_out(k)` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Eval(_, k)` の腕, <1>1, <1>1a
<1>8. CASE `n` の式が `RcExpr::Ret(_)` である。この腕は `returns_from_func` が真のとき `pending` の各
      要素の `node` を `self.needed_retains` に入れるが、`pending` を変えずに `pending` を返す。よって
      `pending_out(n) = pending(n)` であり、鎖は空である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕
<1>9. QED
  `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を <1>2 から <1>8 が尽くす。走査が
  `PendingRetains` の値を作るのは、`cancel` の「初期」と、これら 7 つの腕が行う操作だけである。7 つの腕が
  呼ぶ関数のうち `PendingRetains` に触れるのは `consume_rhs`、`consume`、`consume_objects` (L6 より
  「消費」)、`un_bump` (L5 より「引き」)、`merge` (L7 より「併合」) であり、腕自身が行うのは「追加」
  (<1>2) と「複製」(<1>4) である。
  BY <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
     CODE src/rc_ir/borrow.rs: cancel, L5, L6, L7

### L8a (状態は有限個であり、生成順序は整礎である)

ASSUME  NEW `prog`: `RcProgram`,
        `prog` は `borrow_ify` の 1 回の呼び出しが返した値である
PROVE   `cancel(prog, type_env)` の中の `cancel_body` の 1 回の実行について、次の 3 つが成り立つ。

1. その実行が作る状態 (DEF 基本操作) は有限個である。
2. 初期状態を除く各状態は、それより前に作られた 1 つ以上の状態から基本操作 1 つで作られる。
3. その実行が作る状態の上の生成順序 (DEF 基本操作) は整礎である。すなわち無限の下降列を持たない。

**証明**

<1>1. 1 が成り立つ。
  <2>1. 本補題の仮定より P15 が当たるので、走査は 1 つの本体の各位置をちょうど 1 回訪問し、本体は有限の
        木である (D2)。よって 1 回の `cancel_body` の実行における訪問は有限回である。
    BY P15, D2, 本補題の仮定
  <2>2. 1 回の訪問が実行する基本操作は有限個である。「初期」は訪問の中では実行されない。「追加」は
        `Retain` の腕で 1 回、「引き」は `Release` の腕で 1 回、「複製」は `Match` の腕でアームごとに
        1 回、「併合」は `Match` の腕で 1 回である。「消費」は `consume_objects` の呼び出しごとに 1 回で、
        その呼び出しは `Release` の腕で高々 2 回、`Let` (右辺が `Match` でない) の腕で `consume_rhs` が
        `rhs_consumes` に積ませた `Vec` の要素ごとに 1 回、`Destructure` の腕で `destructure_consumes` が
        返す `Vec` の要素ごとに 1 回である (L6)。`Vec` の長さは有限であり、`arms` も `Vec` である。
    BY L6, DEF 基本操作, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume
  <2>3. QED
    L8 より、走査が `PendingRetains` の値を作るのは DEF 基本操作 の 6 種だけである。「初期」は
    `cancel_body` の実行につき 1 回であり (`cancel` の `analysis.walk(body, PendingRetains::default(), true)`)、
    残る 5 種は <2>2 より 1 回の訪問につき有限個で、<2>1 より訪問は有限回である。よって状態は有限個である。
    BY <2>1, <2>2, L8, 本補題の仮定, DEF 基本操作, CODE src/rc_ir/borrow.rs: cancel
<1>2. 2 が成り立つ。L8 より、走査が `PendingRetains` の値を作るのは DEF 基本操作 の 6 種だけである。
      「初期」以外の 5 種は、DEF 基本操作 が定めるとおり既に在る状態を入力に取る --- 「複製」「追加」
      「消費」「引き」は 1 つ、「併合」は `pending_in` と各 `arm_exits[j]` である。入力はその操作が走る
      時点で既に在るので、生成順序でその操作が作る状態より前にある。L8 の仮定は本補題の仮定である。
  BY L8, 本補題の仮定, DEF 基本操作
<1>3. QED
  3 が成り立つ。生成順序は時点の前後関係なので狭義の半順序であり (DEF 基本操作)、<1>1 よりこの実行が
  作る状態は有限個である。有限集合上の狭義半順序に無限の下降列は無い。
  BY <1>1, <1>2, DEF 基本操作

## 5. 状態の不変条件

### DEF INV

状態 `P` (DEF 基本操作) について、次の 4 つの連言を `INV(P)` と書く。「`P` の時点」とは、`P` を作った
基本操作が実行された時点をいう。

- **(i)** `P` の各要素 `e` について、`P` の時点までに訪問された `Retain` 節点 `t` であって
  `e.node = node_id(t)` であるものが**ちょうど 1 つ**ある。この `t` を `e` の**由来**と呼び、`orig(e)` と
  書く。(ii) と (iv) はこの記法を使うので、(i) が成り立つ状態についての条件である。
- **(ii)** `P` の各要素 `e` について、`e.outstanding` は空でなく、`e.outstanding ⊆ ActRefs(orig(e))` で
  ある。
- **(iii)** `P` の相異なる 2 つの要素は相異なる `node` を持つ。
- **(iv)** 添字 `i < j` について、`P[i]` の由来は `P[j]` の由来より前に訪問された。

### L9 (走査が作る状態は `INV` を満たす)

ASSUME  NEW `prog`: `RcProgram`,
        `prog` は `borrow_ify` の 1 回の呼び出しが返した値である
PROVE   `cancel(prog, type_env)` の中の `cancel_body` の 1 回の実行の中で走査が作るすべての状態 `P` に
        ついて、`INV(P)` が成り立つ。

**証明** 1 回の `cancel_body` の実行を固定し、状態の生成順序 (DEF 基本操作) についての帰納法で示す。
この帰納法が整礎であることは <1>0c が与える。

<1>0a. 1 回の `cancel_body` の実行が作る状態は有限個である。本補題の仮定は L8a の仮定である。
  BY L8a, 本補題の仮定
<1>0b. 初期状態を除く各状態は、それより前に作られた 1 つ以上の状態から基本操作 1 つで作られる。
  BY L8a, 本補題の仮定
<1>0c. 生成順序についての帰納法は整礎である。L8a の 3 より、この実行が作る状態の上の生成順序は整礎で
       ある。<1>0b より、初期状態でない各状態について、その状態を作った基本操作の入力はすべて生成順序で
       それより前にあるので、帰納法の仮定はその入力すべてに使える。初期状態には入力が無いので、その場合は
       帰納法の仮定を使わずに示す。
  BY L8a, <1>0a, <1>0b, DEF 基本操作
<1>0d. `NodeId` の値 `x` について、`node_id(t) = x` である `Retain` 節点 `t` はこの本体に高々 1 つで
       ある。P15 の前半より、この本体の相異なる位置は相異なる `NodeId` を持つ。よって状態 `P` の要素 `e`
       について「`P` の時点までに訪問された `Retain` 節点 `t` であって `e.node = node_id(t)` である
       ものが在る」ことを示せば、そのような `t` はちょうど 1 つであり、DEF INV の (i) が成り立つ。
  BY P15, 本補題の仮定, DEF INV
<1>1. 帰納法の仮定: この状態より前に作られたすべての状態について `INV` が成り立つ。
  BY 帰納法の仮定
<1>2. CASE 状態が「初期」で作られた。`PendingRetains` は `Vec<PendingRetain>` の別名なので、
      `PendingRetains::default()` は EXT Vec::default より要素を 1 つも持たない `Vec` である。
      (i) から (iv) はどれも `P` の要素についての全称なので、空虚に成り立つ。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: PendingRetains, EXT Vec::default,
     DEF INV
<1>3. CASE 状態が「複製」で作られた。EXT Clone より `pending.clone()` は元の状態と同じ長さで、第 `i`
      要素の `node` は元の第 `i` 要素の `node` と等しく、第 `i` 要素の `outstanding` は元の
      `outstanding` と等しい。(i) は「`P` の時点までに訪問された」を含むが、訪問された節点の集合は時が
      進んでも要素を失わないので、元の状態で成り立てば複製の時点でも成り立つ。<1>0d より由来は `node` で
      一意に定まるので、`node` が等しい複製の要素の由来は元の要素の由来と同じである。(ii) と (iii)
      は要素の `node` と `outstanding` の値だけで決まるので、由来が同じことと合わせて <1>1 から遺伝する。
      (iv) は訪問の時刻を読むが、それは走査全体についての事実であり、複製は要素の `node` も並びも保つので、
      元の状態について <1>1 が与える「`P[i]` の由来は `P[j]` の由来より前に訪問された」がそのまま複製の
      同じ添字の対に当たる。
  BY <1>0d, <1>1, P15, DEF INV, DEF 訪問, EXT Clone
<1>4. CASE 状態が「追加」で作られた。すなわち `Retain` 節点 `t = Retain(v, path, _, k)` の訪問が
      `pending.push(PendingRetain { node: retain, outstanding })` を実行した。ここで
      `retain = node_id(node)` であり `node` は `t` の節点、`outstanding = self.acted_references(v, path)`
      である。L0c の 2 と 3 よりこの値は走査のどの時点で読んでも同じなので、DEF 節点の量 の
      `ActRefs(t)` である。
      書き換え前の状態を `Q0` とする。
  <2>1. この操作は `Vec` の末尾に要素を 1 つ加えるだけであり、既存の要素の値と並びを変えない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       EXT Vec::push
  <2>2. `Q0` は `t` の訪問が始まる前に作られた状態である。この腕には `pending.push` より前に `pending`
        に触れる文が書かれておらず、EXT 文は書かれた順に実行される よりほかの文がそのあいだに実行される
        ことはないので、`Q0` は `walk_inner` に引数として渡された値 `pending(t)` であり、それは呼び出しの
        前に作られている。よって <1>1 の (i) より、`Q0` の各要素の由来は `t` の訪問が始まる前に訪問されて
        おり、P15 の後半 (各位置はちょうど 1 回訪問される) よりそれは `t` ではない。
    BY <1>1, P15, DEF 訪問, DEF 基本操作, EXT 文は書かれた順に実行される,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
  <2>3. (i) が成り立つ。新しい要素の `node` は `node_id(t)` であり、`t` の訪問はこの時点で始まっている
        ので、<1>0d よりその由来は `t` にちょうど 1 つ定まる。既存の要素は <1>1 の (i) のままで、由来も
        変わらない。
    BY <1>0d, <1>1, <2>1, DEF 訪問
  <2>4. (ii) が成り立つ。新しい要素の由来は <2>3 より `t` であり、その `outstanding` は
        `self.acted_references(v, path)` が返した値である。この要素が `pending` に積まれているので
        その呼び出しは値を返しており、L2 の 5 よりその値は空でない。L0c の 2 と 3 より、その値は
        DEF 節点の量 の `ActRefs(t)` である。`ActRefs(t) ⊆ ActRefs(t)` である (DEF 参照の多重集合)。
        既存の要素は <1>1 の (ii) のままである。
    BY <1>1, <2>1, <2>3, L0c, L2, DEF 節点の量, DEF 参照の多重集合
  <2>5. (iii) が成り立つ。
    <3>1. `Q0` のどの要素も `node` が `node_id(t)` と等しくない。
      <4>1. `Q0` のある要素 `e` が `e.node = node_id(t)` を満たすと仮定する。
        BY 背理法の仮定
      <4>2. <1>1 の (i) より、`orig(e)` は `Q0` の時点までに訪問された `Retain` 節点であり、
            `node_id(orig(e)) = e.node = node_id(t)` である。
        BY <1>1, <4>1
      <4>3. P15 の前半より、この本体の相異なる位置は相異なる `NodeId` を持つ。よって `orig(e)` と `t` は
            同じ位置である。
        BY P15, <4>2
      <4>4. QED (矛盾)
        <2>2 より `Q0` の各要素の由来は `t` ではない。<4>3 はそれに反する。
        BY <2>2, <4>3
    <3>2. QED
      BY <1>1, <2>1, <3>1
  <2>6. (iv) が成り立つ。新しい要素は `Vec` の末尾に置かれる。<2>2 より既存の要素の由来はどれも `t` の
        訪問が始まる前に訪問されており、新しい要素の由来は `t` である。既存の要素どうしの並びは <1>1 の
        (iv) のままである。
    BY <1>1, <2>1, <2>2, <2>3
  <2>7. QED
    BY <2>3, <2>4, <2>5, <2>6
<1>5. CASE 状態が「消費」で作られた。L6 より、これは要素を取り除くだけであり、残る要素の値と並びを変え
      ない。よって (i)、(ii)、(iii)、(iv) はいずれも <1>1 の対応する節から遺伝する ((i) の「`P` の時点
      までに訪問された」は時が進んでも保たれ、<1>0d より由来は `node` で一意に定まるので変わらない)。
  BY <1>0d, <1>1, L6, DEF INV, DEF 訪問
<1>6. CASE 状態が「引き」で作られた。すなわち `Release` 節点の訪問が `un_bump(&mut pending, &un_bumped)`
      を実行した。
  <2>1. CASE L5 の 1 または 2。`pending` は変わらないので、<1>1 の各節がそのまま成り立つ ((i) の
        「`P` の時点までに訪問された」は時が進んでも保たれる)。
    BY <1>0d, <1>1, L5, DEF 訪問
  <2>2. CASE L5 の 3。添字 `i` の要素の `outstanding` が `outstanding - un_bumped` になり、それが空なら
        その要素が取り除かれる。ほかの要素の値と相対順序は変わらない。
    <3>1. (i) が成り立つ。要素の `node` は変わらず、要素は減るだけなので、<1>1 の (i) がそのまま残る
          要素に当たる。
      BY <1>0d, <1>1, L5, DEF 訪問
    <3>2. (ii) が成り立つ。差が空になった要素はそのとき取り除かれるので、残る要素の `outstanding` は
          空でない。L2 の 4 より `outstanding - un_bumped ⊆ outstanding` である。`⊆` は各位置についての
          個数の不等式なので (DEF 参照の多重集合)、整数の `≤` の推移律より推移的であり、<1>1 の (ii) から
          `outstanding - un_bumped ⊆ ActRefs(orig(e))` が従う。
      BY <1>1, L5, L2, DEF 参照の多重集合
    <3>3. (iii) が成り立つ。要素は減るだけで `node` は変わらない。
      BY <1>1, L5
    <3>4. (iv) が成り立つ。要素の相対順序と由来は変わらない。
      BY <1>1, L5
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4
  <2>3. QED
    L5 の 3 つの場合は尽くしている。
    BY L5, <2>1, <2>2
<1>7. CASE 状態が「併合」で作られた。すなわち `Match` 節点の訪問が `self.merge(&pending, &arm_exits)` を
      実行し、その返り値 `merged` が新しい状態である。`pending_in` を `&pending` の指す状態とする。
  <2>1. 各 `arm_exits[j]` は `self.walk(&arms[j].body, ・, false)` の戻り値であり、L1 と DEF 訪問 より
        それは `pending_out(arms[j].body)`、すなわち `merged` より前に作られた状態である。よって <1>1 より
        `INV(arm_exits[j])` が成り立ち、その (iii) より `arm_exits[j]` の相異なる要素は相異なる `node` を
        持つ。すなわち L7 の仮定が満たされる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       <1>1, L1, DEF 訪問, DEF 基本操作
  <2>2. `pending_in` は `merged` より前に作られた状態なので、<1>1 より `INV(pending_in)` が成り立つ。
    BY <1>1
  <2>3. (i) が成り立つ。L7 の 6 より、`merged` の各要素の `node` は `pending_in` のある要素の `node` で
        ある。<2>2 の (i) より、それは `pending_in` の時点までに訪問された `Retain` 節点の `node_id` で
        あり、`merged` の時点はそれより後である。<1>0d より由来はちょうど 1 つ定まり、それは
        `pending_in` のその要素の由来と同じである。
    BY <1>0d, L7, <2>1, <2>2, DEF 訪問
  <2>4. (ii) が成り立つ。
    <3>1. `merged` の各要素 `e` の `outstanding` は、L7 の 6 と 4 より、ある `j` について
          `arm_states[j]` が `e.node` に与える値、すなわち `arm_exits[j]` の要素 `e'` で
          `e'.node = e.node` であるものの `outstanding` と等しい。
      BY L7, <2>1
    <3>2. `orig(e') = orig(e)` である。`e'.node = e.node` であり、<1>0d より由来は `node` で一意に
          定まる。
      BY <1>0d, DEF INV, <3>1, <2>1, <2>3
    <3>3. QED
      <2>1 の `INV(arm_exits[j])` の (ii) より `e'.outstanding` は空でなく
      `e'.outstanding ⊆ ActRefs(orig(e'))` である。<3>1 と <3>2 より `e.outstanding` はそれと等しい値で
      あり、その由来も同じである。L2 の 6 より、等しい 2 つの値は各位置についての個数が一致するので、
      `e.outstanding` も空でなく `e.outstanding ⊆ ActRefs(orig(e))` である。
      BY <2>1, <3>1, <3>2, L2, DEF 参照の多重集合
  <2>5. (iii) が成り立つ。L7 の 6 より、`merged` の要素は `pending_in` の要素から `filter_map` で作られ、
        1 つの入力要素からは高々 1 つの出力要素ができ、`node` は変わらない。<2>2 の (iii) より
        `pending_in` の相異なる要素は相異なる `node` を持つ。
    BY L7, <2>1, <2>2
  <2>6. (iv) が成り立つ。L7 の 6 より、`merged` は `pending_in` の部分列であり、順序は保たれる。要素の
        `node` は変わらないので <1>0d より由来も変わらない。<2>2 の (iv) よりその並びは由来の訪問順で
        ある。
    BY <1>0d, L7, <2>1, <2>2, <2>3
  <2>7. QED
    BY <2>3, <2>4, <2>5, <2>6
<1>8. QED
  DEF 基本操作 と L8 より、状態の作られ方は「初期」「複製」「追加」「消費」「引き」「併合」の 6 種で
  尽きるので、<1>2 から <1>7 が場合を尽くす。<1>0c より、この場合分けを生成順序についての帰納法の
  1 段として使ってよい。
  BY <1>0a, <1>0b, <1>0c, <1>0d, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, DEF 基本操作, L8

## 6. P16 (`pending` の不変条件)

**言明** --- 走査中の各位置において、`pending` は次を満たす。(a) 各要素の `node` は、その位置までに
訪れた `Retain` 節点である。(b) 各要素の `outstanding` は空でなく、その `Retain` の `ActRefs` に含まれる。
(c) 1 つの `Retain` 節点は `pending` に高々 1 回現れる。(d) `pending` の並びは、訪れた順である (後ろほど
新しい)。(e) `pending` から取り除かれた `Retain` は、次の 3 つのいずれかである。(e1) `outstanding` が
空になった。(e2) `needed_retains` に入った。(e3) その除去は `merge` によるものであり、各アームへ渡った
複製の側に同じ `Retain` の除去事象があって、それらがすべて (e1)(e2)(e3) のいずれかである。

(a) から (d) は、走査が作る各状態についての主張として読む。(b) の「その `Retain` の `ActRefs` に含まれる」
は、DEF 参照の多重集合 の `⊆` で読む。(e) は次の形にして示す。**各除去事象 (DEF 除去事象) について、
次の 3 つのいずれかが成り立つ。(e1) その事象は「引き」であり、取り除かれた要素の `outstanding` はその
事象の中で空になった。(e2) その事象は、取り除かれた `node` を `self.needed_retains` に入れる。
(e3) その事象は「併合」であり、各アームの状態の鎖 (L8) の中に、同じ `node` の除去事象がある。そして
(e3) の展開は有限で終わり、その葉は (e1) か (e2) である。**

**証明**

<1>0. **この証明が引く L4、L8、L8a、L9 の仮定 --- `cancel` の入力 `prog` が `borrow_ify` の 1 回の
      呼び出しが返した値であること --- は、`cancel` のすべての呼び出しについて満たされる。** よって
      この命題は README の P16 --- 仮説を持たない言明 --- として立つ。また、言明の (e) が主語にする
      「`pending` から取り除かれた `Retain`」は、`NodeId` の値ごとに読む。
      すなわち、`node` が `x` である要素が `pending` から取り除かれることと、`x` の除去事象
      (DEF 除去事象) が在ることは同値である。よって各除去事象について上の 3 つを示せば、言明の (e) が
      出る。
  <2>0. 第 1 文が成り立つ。L2b が `cancel(prog, type_env)` のすべての呼び出しについて `prog` が
        `borrow_ify` の 1 回の呼び出しが返した値そのものであることを述べる。
    BY L2b
  <2>1. `NodeId` の値 `x` を 1 つ取る。走査が `PendingRetains` の値を作るのは DEF 基本操作 の 6 種だけで
        あり (L8)、各状態はそれを作る基本操作が実行される時点で生じる (DEF 基本操作)。よって `node` が
        `x` である要素が `pending` から取り除かれるとは、その要素を持つ状態を入力に取り、それを持たない
        状態を作る基本操作が在ることであり、これは DEF 除去事象 の言う `x` の除去事象そのものである。
        **1 つの除去事象が `node` の相異なる 2 つの値の要素を同時に取り除くことも、1 つの `x` について
        除去事象が複数在ることもありうる。** この段が言うのは、`x` を固定したときの同値だけである。
    BY <2>0, L8, DEF 基本操作, DEF 除去事象
  <2>2. QED
    上の 3 つの場合は、言明の (e1)(e2)(e3) をそれぞれ含む。(e1) は「取り除かれた要素の `outstanding` は
    その事象の中で空になった」であり、言明の「`outstanding` が空になった」である。(e2) は「その事象は、
    取り除かれた `node` を `self.needed_retains` に入れる」であり、その `node` が `needed_retains` に
    入ったことを含む。(e3) は言明と同じ文である。
    BY <2>0, <2>1
<1>1. (a) が成り立つ。
  BY <1>0, L9 の (i), DEF INV
<1>2. (b) が成り立つ。
  BY <1>0, L9 の (ii), DEF INV, DEF 参照の多重集合
<1>3. (c) が成り立つ。
  BY <1>0, L9 の (iii), DEF INV
<1>4. (d) が成り立つ。
  BY <1>0, L9 の (iv), DEF INV
<1>5. 除去事象を起こしうる基本操作は「消費」「引き」「併合」の 3 つだけである。
  <2>1. 「初期」は入力の状態を持たないので、除去事象ではない。
    BY DEF 除去事象, DEF 基本操作
  <2>2. 「複製」で作られた状態は入力の状態と等しいので、要素を失わない。EXT Clone より
        `pending.clone()` は元と同じ長さで、第 `i` 要素の `node` は元の第 `i` 要素の `node` と等しい。
    BY DEF 基本操作, EXT Clone
  <2>3. 「追加」は `Vec` の末尾に要素を 1 つ加えるだけで、要素を取り除かない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       EXT Vec::push
  <2>4. QED
    L8 より基本操作は 6 種で尽きる。
    BY <1>0, <2>1, <2>2, <2>3, DEF 基本操作, L8
<1>6. CASE 除去事象が「消費」である。L6 より、`consume_objects` は取り除いた各要素の `node` を
      `self.needed_retains` に入れる。よって (e2) が成り立つ。
  BY L6
<1>7. CASE 除去事象が「引き」である。
  <2>1. L5 の 1 と 2 では `pending` が変わらないので、除去事象は L5 の 3 の場合に限る。
    BY L5
  <2>2. L5 の 3 で要素が取り除かれるのは、`pending[i].outstanding` が `un_bumped` を引いた結果が空に
        なったとき、かつそのときに限る。
    BY L5
  <2>3. QED (e1) が成り立つ。
    BY <2>1, <2>2
<1>8. CASE 除去事象が「併合」である。取り除かれた `node` を `x` とする。すなわち、`pending_in` と各
      `arm_exits[j]` のいずれかに `node` が `x` の要素があり、`merged` にはそれが無い。
  <2>1. DEF 基本操作 より「併合」は `walk_inner` の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の
        `self.merge(&pending, &arm_exits)` であり、各 `arm_exits[j]` は `self.walk(&arms[j].body, ・, false)`
        の戻り値である。L1 と DEF 訪問 よりそれは `pending_out(arms[j].body)`、すなわちこの走査が作った
        状態である。よって L9 の (iii) がそれに当たり、L7 の仮定が満たされる。
    BY <1>0, L9, L1, DEF 基本操作, DEF 訪問
  <2>2. CASE ある `j` について `arm_states[j]` が `x` を鍵に持つ。
    <3>1. L7 の 3 の条件 `U(x)` は成り立たない。
      <4>1. `U(x)` が成り立つと仮定する。
        BY 背理法の仮定
      <4>2. L7 の 4 より、呼び出しの終わりに `uniform` は `x` を鍵に持つ。
        BY L7, <2>1, <2>2, <4>1
      <4>3. `U(x)` の第 1 の連言肢より `x` は `entered_with` の要素であり、L7 の 2 より `pending_in` に
            `node` が `x` の要素がある。
        BY L7, <2>1, <4>1
      <4>4. QED (矛盾)
        <4>2 と <4>3 と L7 の 6 より、`merged` に `node` が `x` の要素がある。これは本場合の仮定
        (`merged` にそれが無い) に反する。
        BY L7, <2>1, <4>2, <4>3
    <3>2. QED (e2) が成り立つ。
      L7 の 5 より、この呼び出しは `x` を `self.needed_retains` に入れる。
      BY L7, <2>1, <2>2, <3>1
  <2>3. CASE どの `j` についても `arm_states[j]` が `x` を鍵に持たない。
    <3>1. `pending_in` に `node` が `x` の要素がある。L7 の 1 より、どの `arm_exits[j]` にも `node` が
          `x` の要素は無いので、本場合の仮定 (入力のいずれかにその要素がある) を満たすのは `pending_in`
          だけである。
      BY L7, <2>1, <2>3, DEF 除去事象
    <3>2. 各アーム `j` の入口状態 `pending(arm_j.body)` は `pending_in` の「複製」なので、`node` が `x`
          の要素を持つ。EXT Clone より `pending.clone()` は元と同じ長さで、第 `i` 要素の `node` は元の
          第 `i` 要素の `node` と等しいからである。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
         DEF 基本操作, EXT Clone, <3>1
    <3>3. L8 より `arm_exits[j] = pending_out(arm_j.body)` は入口状態から基本操作の有限列 (状態の鎖) で
          得られる。<3>2 でその鎖の最初の状態は `node` が `x` の要素を持ち、L7 の 1 と本場合の仮定より
          その鎖の最後の状態は持たない。よってその鎖の中に、`node` が `x` の要素を持つ状態を入力とし、
          持たない状態を作る操作、すなわち `x` の除去事象がある。
      BY <1>0, L8, L7, <2>1, <2>3, <3>2, DEF 除去事象
    <3>4. QED (e3) が成り立つ。
      BY <3>3
  <2>4. QED
    <2>2 と <2>3 は場合を尽くす。
    BY <2>2, <2>3
<1>9. (e3) の展開は有限で終わり、その葉は (e1) か (e2) である。ここで**展開**とは、次の木をいう。根は
      いま考えている除去事象である。(e3) が成り立つ除去事象の子は、各アームについてその (e3) が名指す
      除去事象を 1 つずつ選んだものである。(e1) または (e2) が成り立つ除去事象に子は付けない。
  <2>1. (e3) が指す除去事象は、`Match` 節点 `n` のアームの走査の中で起きる。アームの列を作る文は
        `self.merge` を呼ぶ文より前に書かれており、EXT 文は書かれた順に実行される よりその走査は
        `self.merge` の呼び出しより前に完了しているので、そこで作られる状態は `merged` より前に作られて
        いる。よって (e3) の各子は、生成順序について親より真に前にある。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       DEF 基本操作, <1>8, EXT 文は書かれた順に実行される
  <2>2. (e3) の子はアームごとに 1 つずつ選べるので有限個であり、L4 より 1 つ以上ある。
    BY <1>0, L4, <1>8
  <2>2a. 除去事象 `E` にその事象が作る状態 `P'(E)` を対応させると、<2>1 より (e3) の各子 `E'` について
         `P'(E')` は `P'(E)` より生成順序で真に前にある。L8a の 3 よりこの実行が作る状態の上の生成順序は
         整礎なので、この対応を通した整礎帰納法が展開の節点の上で使える。
    BY <1>0, L8a, <2>1
  <2>2b. 各除去事象 `E` について、`E` を根とする展開は有限の木である。<2>2a の整礎帰納法による。`E` に
         子が付かないとき、展開は 1 節点の木である。子が付くとき、<2>2 よりその個数は有限であり、<2>1 と
         <2>2a よりどの子も `E` より生成順序で真に前にあるので、帰納法の仮定より各子を根とする展開は
         有限の木である。有限個の有限の木の上に根を 1 つ置いたものは有限の木である。
    BY <2>1, <2>2, <2>2a
  <2>3. QED
    <1>5 より除去事象を起こしうる基本操作は「消費」「引き」「併合」の 3 つで尽き、<1>6、<1>7、<1>8 が
    その 3 つについてそれぞれ (e2)、(e1)、(e2) または (e3) を与えるので、各除去事象は (e1)、(e2)、(e3) の
    いずれかである。<2>2b より展開は有限の木であり、<2>2 より (e3) の節点は子を 1 つ以上持つので、葉は
    (e1) か (e2) である。
    BY <2>1, <2>2, <2>2a, <2>2b, <1>5, <1>6, <1>7, <1>8
<1>10. QED
  <1>5 より除去事象は「消費」「引き」「併合」のいずれかであり、<1>6、<1>7、<1>8 がそれぞれ (e2)、(e1)、
  (e2) または (e3) を与える。<1>9 が (e3) の展開について述べる。<1>0 より、除去事象について示したこの 3 つが
  言明の (e) を与える。
  BY <1>0, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9

## 7. P17 (`un_bump` の正しさ)

**言明** --- `un_bump(pending, R)` の返り値は次で決まる。`R` と**位置 (`VarPath`) を共有する**要素が
`pending` に無ければ `NoBracket` で、`pending` は変わらない。あって、そのうち最も後ろの要素 (最内) の
`outstanding` が `R` を `covers` しなければ `OutsideBracket` で、`pending` は変わらない。covers すれば
`InBracket(t)` で、`t` はその要素の `node` であり、その要素の `outstanding` から `R` が引かれ、空に
なればその要素が取り除かれる。他の要素は変わらない。

ここで「`R` と位置を共有する要素」とは、`e.outstanding.shares_an_object(R)` が真である要素である
(L5 の冒頭)。判定に使われるのはその要素の現在の `outstanding` であって、由来の `Retain` が作った
`ActRefs` ではない。README の P17 が「コードの `References::shares_an_object` と `References::objects`
は `VarPath` の鍵を扱う」と述べ、その鍵が D6 の位置であって D25 のオブジェクトではないことと、2 つが
写り合うことを P5 (a) が述べることを添える。この文書では L2 の 3 がその鍵を扱う。

**証明**

<1>0a. `un_bump` は `borrow.rs` の非公開の自由関数であり、`borrow.rs` は `mod` 宣言を 1 つも持たないので、
       その呼び出しは `borrow.rs` の中にしか書けない。
  BY CODE src/rc_ir/borrow.rs: un_bump, EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>1. `borrow.rs` の中で `un_bump` を呼ぶのは `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕
      1 か所だけであり、その第 1 引数はその時点の走査の状態、第 2 引数は
      `un_bumped = self.acted_references(v, path)` である。
  BY <1>0a, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: un_bump
<1>2. L5 の 1 は、言明の第 1 の場合と条件も結論も一致する。
  BY L5
<1>3. L5 の 2 と 3 の添字 `i` の要素は、言明の「最も後ろの要素 (最内)」である。L5 の `i` は、`R` と
      位置を共有する要素の添字のうち最大のものだからである。
  BY L5
<1>4. L5 の 2 は言明の第 2 の場合と、L5 の 3 は言明の第 3 の場合と、条件も結論も一致する。L5 の 3 の
      「`pending[i].outstanding` は `pending[i].outstanding - un_bumped` になる」は言明の
      「`outstanding` から `R` が引かれ」であり (DEF 参照の多重集合)、L5 の 3 の最後の文が言明の
      「他の要素は変わらない」である。
  BY L5, <1>3, DEF 参照の多重集合
<1>5. QED
  L5 の 3 つの場合は場合を尽くし、言明の 3 つの場合と一致する。
  BY L5, <1>1, <1>2, <1>3, <1>4

「最内」の名は P16 の (d) が支える。走査の状態では、`pending` の並びは由来の訪問順なので、`R` と
位置を共有する要素のうち添字が最大のものは、その中で由来が最も後に訪問された要素である。

## 8. P18 (`merge` の後に残るもの)

**言明** --- `merge` の返す `pending` に残る `Retain` は、`pending_in` に在り、いずれかのアームの出口に
現れ、かつすべてのアームの出口に同じ `outstanding` で現れるものだけである。いずれかのアームの出口に
現れてこの条件を満たさない `Retain` は `needed_retains` に入る。どのアームの出口にも現れない `Retain` は、
この呼び出しでは `needed_retains` にも返り値にも入らない (走査の他の位置が `needed_retains` に入れる
ことは妨げない)。

`NodeId` の値 `x` について、言明の条件を `C(x)` と書く。すなわち `C(x)` とは、`pending_in` に `node` が
`x` の要素があり、ある `j` について `arm_exits[j]` に `node` が `x` の要素があり、すべての `j'` に
ついて `arm_exits[j']` に `node` が `x` の要素があってそれらの `outstanding` が互いに等しいことである。
「`Retain` が `pending` に在る/現れる」は「その `NodeId` を `node` とする要素がある」と読む。

**証明**

<1>0. **この証明が引く L8、L8a、L9 の仮定 --- `cancel` の入力 `prog` が `borrow_ify` の 1 回の呼び出しが
      返した値であること --- は、`cancel` のすべての呼び出しについて満たされる (L2b)。** よってこの
      命題は README の P18 --- 仮説を持たない言明 --- として立つ。
      言明の `merge` は、`walk_inner` の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の
      `self.merge(&pending, &arm_exits)` の呼び出しである。`merge` は非公開の型 `CancelAnalysis` の
      非公開のメソッドであり、`borrow.rs` は `mod` 宣言を 1 つも持たないので、その呼び出しは `borrow.rs`
      の中にしか書けず、`borrow.rs` の中でそれを呼ぶのはこの 1 か所だけである。この呼び出しにおいて
      `pending_in` はその時点の走査の状態であり、`arm_exits[j]` は同じ腕の
      `arms.iter().map(|arm| self.walk(&arm.body, pending.clone(), false)).collect()` が作った `Vec` の
      第 `j` 要素、すなわち `self.walk(&arms[j].body, ・, false)` の返り値 `pending_out(arms[j].body)`
      である (L1、DEF 訪問、EXT Iterator::map と collect)。よって `pending_in` と各 `arm_exits[j]` は、この
      `cancel_body` の 1 回の実行の中で走査が作った状態である (DEF 基本操作)。
  BY L2b, CODE src/rc_ir/borrow.rs: CancelAnalysis, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     L1, DEF 訪問, DEF 基本操作, EXT Vec::iter と slice::iter, EXT Iterator::map と collect,
     EXT 可視性と私有性, EXT モジュールは `mod` が導入する
<1>1. L9 の (iii) を各 `arm_exits[j]` に適用すると、L7 の仮定が満たされる。<1>0 より各 `arm_exits[j]` は
      走査が作った状態であり、L9 の仮定も満たされるので、L9 が当たる。以下 L7 の 1 から 6 を使う。
  BY <1>0, L9, L7
<1>1a. `arm_states[j]` が `x` を鍵に持つことと、`arm_exits[j]` に `node` が `x` の要素があることは
       同値であり、そのときの値はその要素の `outstanding` である。
  BY L7 の 1, <1>1
<1>1b. `x` が `entered_with` の要素であることと、`pending_in` に `node` が `x` の要素があることは同値で
       ある。
  BY L7 の 2, <1>1
<1>2. `C(x)` は「ある `j` について `arm_states[j]` が `x` を鍵に持つ」と `U(x)` (L7 の 3) の連言と同値で
      ある。`U(x)` は「`x` が `entered_with` の要素であり、すべての `j'` について `arm_states[j']` が
      `x` を鍵に持ち、それらの値が互いに等しい」である。<1>1a と <1>1b でこれを言い換えると、`C(x)` の
      第 1 と第 3 の連言肢になる。`C(x)` の第 2 の連言肢は「ある `j` について `arm_states[j]` が `x` を
      鍵に持つ」である。
  BY L7, <1>1, <1>1a, <1>1b
<1>3. 呼び出しの終わりに `uniform` が `x` を鍵に持つことと `C(x)` は同値である。
  BY L7 の 4, <1>1, <1>2
<1>4. 第 1 の主張が成り立つ。L7 の 6 より、返り値の要素の `node` は `pending_in` の要素の `node` のうち
      `uniform` が鍵に持つものだけである。<1>3 よりそれは `C(x)` を満たすものだけである。
  BY L7, <1>1, <1>3
<1>5. 逆に、`C(x)` を満たす `x` は返り値に残り、その要素の `outstanding` は各アームの出口での共通の値と
      等しい。`C(x)` より `pending_in` に `node` が `x` の要素があり、<1>3 より `uniform` は `x` を鍵に
      持つので、L7 の 6 よりその要素は返り値に残る。その `outstanding` は `uniform[x]` と等しく、L7 の 4
      よりそれは各 `arm_states[j']` の共通の値と等しい。
  BY L7, <1>1, <1>3
<1>6. 第 2 の主張が成り立つ。`x` がいずれかの `arm_exits[j]` の要素の `node` であり、`C(x)` を満たさない
      とする。<1>1a より `x` はその `arm_states[j]` の鍵であり、<1>2 より `U(x)` は成り立たない。
      よって L7 の 5 より、この呼び出しは `x` を `self.needed_retains` に入れる。
  BY L7, <1>1, <1>1a, <1>2
<1>7. 第 3 の主張が成り立つ。`x` がどの `arm_exits[j]` の要素の `node` でもないとする。<1>1a より
      `x` はどの `arm_states[j]` の鍵でもない。よって L7 の 5 より、この呼び出しは `x` を
      `self.needed_retains` に入れない。また <1>2 より `C(x)` は成り立たないので、<1>3 より `uniform` は
      `x` を鍵に持たず、L7 の 6 より返り値に `node` が `x` の要素は無い。
  BY L7, <1>1, <1>1a, <1>2, <1>3
<1>8. QED
  BY <1>4, <1>5, <1>6, <1>7

## 9. 層 4 へ渡す補題

次の 4 つは P15 - P18 の証明には使わないが、`cancel` の走査の性質なのでここで示す。

### L10 (記録は増えるだけ)

走査の実行中、`self.needed_retains` は要素を失わず、`self.all_retains` は要素を失わず、
`self.un_bump_releases` は鍵を失わず、その各値の `Vec` も要素を失わない。また、`Retain` 節点 `t` の訪問の
後、走査が終わるまで、`node_id(t)` は `self.all_retains` の要素であり、`self.un_bump_releases` は
`node_id(t)` を鍵に持つ。

**証明** 以下、`CancelAnalysis` を構築するときの初期化 (`Set::default()`、`Map::default()`、`vec![]`) と、
`cancelled()` の読み出しは走査の実行の外なので、数えない
(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled`)。

<1>1. 走査の実行中に `self.needed_retains` に触れるのは、`walk_inner` の `RcExpr::Ret(_)` の腕、
      `consume_objects`、`merge` の 3 か所であり、どれも `insert` だけである。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>2. 走査の実行中に `self.all_retains` に触れるのは、`walk_inner` の `RcExpr::Retain(v, path, _, k)` の腕の
      `self.all_retains.push(retain)` だけである。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
<1>3. 走査の実行中に `self.un_bump_releases` に触れるのは、`walk_inner` の
      `RcExpr::Retain(v, path, _, k)` の腕の
      `self.un_bump_releases.entry(retain).or_default()` と、`RcExpr::Release(v, path, _, k)` の腕の
      `self.un_bump_releases.entry(retain).or_default().push(node_id(node))` の 2 か所だけである。
      どちらも鍵を取り除かず、値の `Vec` から要素を取り除かない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     EXT Map と Set, EXT Vec::push
<1>4. `Retain` 節点 `t` の訪問は `self.all_retains.push(node_id(t))` と
      `self.un_bump_releases.entry(node_id(t)).or_default()` を実行する。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: node_id
<1>5. QED
  <1>1、<1>2、<1>3 が前半を与える。後半は <1>4 から出る --- EXT Vec::push より
  `self.all_retains.push(node_id(t))` は `node_id(t)` を `self.all_retains` の要素に加え、
  EXT Map と Set より `self.un_bump_releases.entry(node_id(t)).or_default()` はその鍵が無ければ加え、
  いずれにせよ `self.un_bump_releases` がその鍵を持つ状態にする。<1>2 と <1>3 よりその後どちらも
  失われない。
  BY <1>1, <1>2, <1>3, <1>4, EXT Vec::push, EXT Map と Set

### L11 (訪問順序は実行路の順序を含む)

ASSUME  NEW `prog`: `RcProgram`,
        `prog` は `borrow_ify` の 1 回の呼び出しが返した値である
PROVE   `cancel(prog, type_env)` が走査する各本体について、その本体の実行路 `p` (D3) の上で節点 `m` が
        節点 `n` より真に前にあるならば、走査は `m` を `n` より前に訪問する。

**証明**

<1>0. 本補題の仮定は P15 の言明の仮説である。P15 より、走査はこの本体の各位置をちょうど 1 回訪問するので、
      各節点の「訪問」(DEF 訪問) は 1 つに定まり、「`m` を `n` より前に訪問する」はその 1 つの訪問の
      開始時刻の比較として読める。時刻の前後関係は推移的なので、この関係も推移的である。
  BY P15, 本補題の仮定, DEF 訪問
<1>1. 「`p` の上で真に前」は「`p` の上で直後」の推移閉包である。<1>0 より結論側の関係は推移的なので、
      `p` の上で `m'` が `m` の直後にあるすべての対について「走査は `m` を `m'` より前に訪問する」を
      示せば足りる。
  BY D3 (実行路は節点の有限列である), <1>0
<1>2. CASE `m` の式が `RcExpr::Let(_, RcRhs::Match(_, arms), k)` である。
  <2>1. D3 より、`p` の上の `m` の直後の節点 `m'` は、`p` が選んだアーム `arm_i` の本体 `arm_i.body` で
        ある。
    BY D3
  <2>2. `m` の訪問は `self.walk(&arm_i.body, pending.clone(), false)` を呼び、その呼び出しの中で
        `arm_i.body` が訪問される。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       L1
  <2>3. QED
    BY <2>1, <2>2
<1>3. CASE `m` の式が `RcExpr::Retain(..)`、`RcExpr::Release(..)`、`RcExpr::Destructure(..)`、
      `RcExpr::Eval(..)`、または `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でないもの、の
      いずれかである。
  <2>1. D3 より、`p` の上の `m` の直後の節点は `m` の継続 `k` である。
    BY D3
  <2>2. `m` の訪問は `self.walk(k, ・, ・)` を呼び、その呼び出しの中で `k` が訪問される。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Eval(_, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, L1
  <2>3. QED
    BY <2>1, <2>2
<1>4. CASE `m` の式が `RcExpr::Ret(_)` であり、`p` の上に `m` の直後の節点 `m'` がある。
  <2>1. D3 より、`Ret` の後に実行路が続くのは、`m` が、その実行路が入ったアームの本体の実行路を終える
        `Ret` であるときに限る。そのアームを `arm_i`、その `Match` 節点を `M` とすると、
        `m = ret(arm_i.body)` であり、`m'` は `M` の継続 `k_M` である。ここで `ret(・)` は D3 の継続終端で
        ある。
    BY D3
  <2>2. `M` の訪問は、まず `arms` の各アームについて `self.walk(&arm.body, pending.clone(), false)` を
        呼び、それらが返った後で `self.merge` を呼び、その後で
        `self.walk(k_M, merged, returns_from_func)` を呼ぶ。この 3 つはこの順に書かれた 3 つの文に在り、
        EXT 文は書かれた順に実行される よりこの順に実行される。アームの列を作る文が
        `arms.iter().map(...).collect()` で各アームの呼び出しを済ませることは
        EXT Iterator::map と collect が与え、`self.merge` と `self.walk(k_M, ・, ・)` の引数が
        その呼び出しより先に評価されることは EXT 演算対象は式より先に評価される が与える。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       EXT 文は書かれた順に実行される, EXT 演算対象は式より先に評価される,
       EXT Vec::iter と slice::iter, EXT Iterator::map と collect
  <2>2a. 任意の節点 `n` について `ret(n) ∈ N(n)` である。D3 より `ret(n)` は `n` から継続だけを辿って
         着く `Ret` 節点であり、DEF 部分木 より、`Ret` 以外の 5 種の節点の継続 `k` はその節点の子である
         (`Let(_, Match(_, arms), k)` の子はアーム本体と `k`、残る 4 種の子は `k`)。
         `N` は子について閉じている --- `N(n)` は `n` と各子 `c` の `N(c)` の合併である。よって `n` から
         `ret(n)` までの継続の鎖の長さについての帰納法で従う。鎖の長さが 0 のとき `ret(n) = n ∈ N(n)`
         であり、長さが 1 以上のとき `ret(n) = ret(k)` で、帰納法の仮定より `ret(k) ∈ N(k) ⊆ N(n)` で
         ある。
    BY D3, DEF 部分木
  <2>3. `m` の訪問は `k_M` の訪問より前に始まる。
    <3>1. 本補題の仮定は P15 の言明の仮説である。P15 の後半より、`walk(n, ・, ・)` の 1 回の呼び出しは
          `N(n)` の各位置をちょうど 1 回訪れ、`N(n)` の外の位置を訪れない。
      BY P15, 本補題の仮定
    <3>2. `m = ret(arm_i.body)` は <2>2a より `N(arm_i.body)` の要素なので、<3>1 より <2>2 の
          `self.walk(&arm_i.body, ・, ・)` の呼び出しの中で訪問される。
      BY <2>1, <2>2, <2>2a, <3>1
    <3>3. `k_M ∈ N(k_M)` なので、<3>1 より `k_M` は <2>2 の `self.walk(k_M, ・, ・)` の呼び出しの中で
          訪問される。
      BY <2>2, <3>1, DEF 部分木
    <3>4. `k_M` は `N(arm_i.body)` の要素ではない。L0a の 3 より `M` の相異なる子の部分木は交わらず、
          `arm_i.body` と `k_M` は `M` の相異なる子である。`k_M ∈ N(k_M)` なので `k_M ∉ N(arm_i.body)`
          である。よって <3>1 より、<2>2 の `self.walk(&arm_i.body, ・, ・)` の呼び出しの中で `k_M` は
          訪問されない。
      BY <2>2, <3>1, L0a, DEF 部分木
    <3>5. QED
      <2>2 より `self.walk(&arm_i.body, ・, ・)` の呼び出しは `self.walk(k_M, ・, ・)` の呼び出しより
      前に返る。<3>2 より `m` の訪問はその前者の呼び出しの中で始まり、<3>3 と <3>4 より `k_M` の訪問は
      その後者の呼び出しの中で始まる。<1>0 よりどちらの節点の訪問も 1 つに定まるので、`m` の訪問は
      `k_M` の訪問より前に始まる (DEF 訪問)。
      BY <1>0, <2>2, <3>2, <3>3, <3>4, DEF 訪問
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>5. QED
  <1>2、<1>3、<1>4 は、`p` の上に直後の節点がある場合を尽くす。`RcExpr` の 6 変位のうち `Ret` 以外の
  5 つを <1>2 と <1>3 が尽くし、`Ret` を <1>4 が扱う。
  BY <1>0, <1>1, <1>2, <1>3, <1>4, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L12 (`OutsideBracket` の後始末)

`un_bump` が `OutsideBracket` を返したとき、`walk_inner` の `RcExpr::Release(v, path, _, k)` の腕は、
`un_bumped` と位置を共有する `pending` の要素をすべて取り除き、その `node` を
`self.needed_retains` に入れる。取り除かれるのは `un_bump` が調べた最内の要素だけではない。

**証明**

<1>1. この腕の `match un_bump(...)` の `UnBump::OutsideBracket` の枝は、`let objects = un_bumped.objects();`
      の後に `self.consume_objects(&mut pending, &objects)` を実行する。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕
<1>2. L2 の 3 より `un_bumped.objects()` は `un_bumped` が参照を持つ位置を 1 度ずつ並べた列で
      ある。L6 より `consume_objects` はそのいずれかについて `outstanding.names` が真である要素をすべて
      取り除き、その `node` を `needed_retains` に入れる。L2 の 3 より、その条件は
      `outstanding.shares_an_object(un_bumped)` が真であること、すなわち L5 の意味で `un_bumped` と
      位置を共有することと同値である。
  BY L2, L5, L6, <1>1
<1>3. L5 の 2 より、`un_bump` が `OutsideBracket` を返すとき `pending` は変わらないので、<1>2 が見る
      `pending` は `un_bump` が見たものと同じである。`un_bump` が `covers` を検査したのはそのうち最内の
      要素だけである。
  BY L5
<1>4. QED
  BY <1>1, <1>2, <1>3

### L13 (`merge` を越えて残る要素の `outstanding`)

ASSUME  NEW `prog`: `RcProgram`,
        `prog` は `borrow_ify` の 1 回の呼び出しが返した値である
PROVE   `cancel(prog, type_env)` の走査が行う `merge` の呼び出しについて、その返り値の各要素 `e` の
        `outstanding` は、各アームの出口 `arm_exits[j]` にある `node` が `e.node` である要素の
        `outstanding` と等しく、その値はどの `j` についても等しい。

**証明**

<1>1. この `merge` の呼び出しは `walk_inner` の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の
      `self.merge(&pending, &arm_exits)` であり、各 `arm_exits[j]` は
      `self.walk(&arms[j].body, ・, false)` の返り値 `pending_out(arms[j].body)`、すなわちこの走査が
      作った状態である。よって L9 が当たり、その (iii) より `arm_exits[j]` の相異なる要素は相異なる
      `node` を持つので、L7 の仮定が満たされる。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     L1, L9, DEF 訪問, DEF 基本操作, 本補題の仮定
<1>2. 返り値の要素 `e` について、`uniform` は `e.node` を鍵に持ち、`e.outstanding` は `uniform[e.node]` と
      等しい。L7 の 6 より、返り値の要素は `pending_in` の要素のうち `node` を `uniform` が鍵に持つもの
      から作られ、その `outstanding` は `uniform[node]` と等しい `References` である。
  BY L7, <1>1
<1>3. QED
  <1>2 より `uniform` は `e.node` を鍵に持つので、L7 の 4 より `U(e.node)` が成り立ち、`uniform[e.node]`
  は各 `arm_states[j]` が `e.node` に与える共通の値と等しい。L7 の 1 より `arm_states[j]` が `e.node` に
  与える値は、`arm_exits[j]` の `node` が `e.node` である唯一の要素の `outstanding` である。
  BY L7, <1>1, <1>2

## 10. 言明についての注記

**注記 1 (P16 の (e) に第 3 の場合が要ること)**。(e) を「取り除かれた要素の `outstanding` がその時点で
空である」か「取り除かれた要素の `node` がその時点で `needed_retains` に入っている」かの二択にすると、
次の形で偽になる。`Retain` 節点 `t` が `pending` に入り、その後の `Match` のすべてのアームが `t` を完全に
un-bump する (各アームの中の `Release` が L5 の 3 の場合で `t` の `outstanding` を空にする) 場合である。
このとき `arm_exits` のどれも `t` の要素を持たないので、L7 の 1 よりどの `arm_states[j]` も `t` の
`NodeId` を鍵に持たず、L7 の 5 より `merge` は `t` を `needed_retains` に入れず、L7 の 4 と 6 より
返り値にも入れない。この「併合」は `pending_in` の要素の除去事象だが、その要素の `outstanding` は
L9 の (ii) より空でなく (減ったのは各アームに渡った複製の側である)、`t` は `needed_retains` にも
入っていない。(e3) は、この除去がどこで解消されたか --- 各アームの状態の鎖の中の除去事象 --- を名指す。
この展開が有限で終わり、その葉が (e1) か (e2) であることは P16 の証明が示す。

**注記 2 (P17 の 2 つの限定)**。第 1 に、`un_bump` が検査するのは `innermost.outstanding.covers(un_bumped)`
であって、最内の要素の由来である `Retain` が作った参照 (`ActRefs`) と `un_bumped` の関係ではない。この
2 つは食い違いうる。ある `Release` が P17 の第 3 の場合で最内の要素の `outstanding` を減らした後、
`ActRefs` は覆うが `outstanding` は覆わない `un_bumped` を持つ `Release` が来ると、`covers` は偽になる。
最内の要素を選ぶ `shares_an_object` の判定も同じく現在の `outstanding` で行われるので、部分的に un-bump
された要素は、残った `outstanding` が名指す位置についてしか後続の `Release` と共有しない。

第 2 に、`un_bump` は位置を共有する要素のうち最内のものしか調べないので、より外側の要素の
`outstanding` が `un_bumped` を覆っていても `OutsideBracket` を返す。この場合の後始末は L12 が述べる ---
`un_bumped` と位置を共有する要素は 1 つ残らず `needed_retains` に入る。`InBracket` の場合は、
外側の共有する要素は触られずに `pending` に残る。

**注記 3 (`consume_objects` は列の途中からも取り除く)**。`consume_objects` は `Vec::retain` で、消費された
位置を名指す要素を列のどこからも取り除く (L6)。取り除かれた要素の `node` は
`needed_retains` に入るので、それらは打ち消しの対象から外れる
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled` は `needed_retains` の要素を飛ばす)。残る要素の相対順序は変わらないので、
P16 の (d) は保たれ、P17 の「最内」はその後も由来の訪問順で決まる。

**注記 4 (P16 の (c) を支えるもの)**。1 つの `Retain` 節点が `pending` に高々 1 回しか現れないことは、
2 つの事実から出る。「追加」の場面では P15 (相異なる位置は相異なる `NodeId` を持ち、各位置はちょうど
1 回訪問される) が、すでに `pending` に在る要素の `node` が今の `Retain` の `node_id` と異なることを
与える (L9 の「追加」の場合)。「併合」の場面では、`merge` が返り値を `pending_in` から `filter_map` で
作ること (L7 の 6) が、`pending_in` の (c) をそのまま返り値へ運ぶ。`merge` は `arm_exits` の側から
要素を作らない。

**注記 5 (P15 の前半が何の性質か)**。D2 が述べるとおり `RcExprNode` は式を `Arc` で共有できるので、1 つの
木の相異なる位置が同じ `NodeId` を持つ木は作れる。P15 の前半が成り立つのは、`cancel` に渡される木が
`RewriteCtx::rewrite` の出力だからである (L3 と、P15 の前半の証明)。`rewrite` は入力の節点を出力にそのまま置かず、
出力の各位置に `expr_node` で新しい割り当てを作る。したがって P15 の前半は `borrow_ify` が保つべき性質で
あり、`borrow_ify` の実装を変えるときはこの性質を壊さないことを確かめる必要がある。

**注記 6 (アームが 0 個の `Match`)**。P18 の第 1 の主張の「いずれかのアームの出口に現れ」は、`arms` が
空のときに偽になり、そのとき返り値は空である。走査する本体にアームが 0 個の `Match` が無いことは L4 が
述べ、その根拠は A9 である。P16 の (e3) の展開が (e1) か (e2) で終わることも A9 に依る。

**注記 7 (層 4 が読むもの)**。P19 は実行路で量化した言明であり、P16 は実行路を量化
しない。この文書が層 4 へ渡すのは次の 3 つである。

- **L11** --- 実行路の上で `m` が `n` より真に前にあるならば、走査は `m` を `n` より前に訪問する。走査が
  状態を作る順序と実行路の順序を繋ぐのはこの補題である。
- **P16 の (e3)** --- その除去は `merge` によるものであり、各アームへ渡った複製の側に同じ `Retain` の
  除去事象がある。1 つの実行路は各 `Match` でアームを 1 つ選ぶので (D3)、その選択に沿った 1 つが、その
  実行路の上での除去事象を名指す。展開が有限で終わり、その葉が (e1) か (e2) であることも P16 が述べる。
- **L13** --- `merge` を越えて残る要素の `outstanding` は、各アームの出口での共通の値と等しい。P16 の
  (b) と合わせると、それは由来の `ActRefs` に含まれ、空でない。
