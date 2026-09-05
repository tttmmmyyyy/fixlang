# P29・P30 と P27 -- 静的に決めた呼び出し先と、実行の合成

この文書は `README.md` の P29、P30、P27 を証明する。対象コミットは
`b6c51fb892746e493e155d9d59ea05d02d7357db` である。

**実行のモデルは `README.md` が持つ。**環境 (D22)、活性化 (D23)、実行 (D24)、参照の持ち手 (D25)、
実行の終わり方 (D31)、実行の読み (D32) は `README.md` の「プログラムの実行」の節が、計数下の
オブジェクトとグローバル状態のオブジェクト (D26) は「所有、参照、義務」の節が定める。**段の素動作と
段内の点も D24 が定める。**解放について閉じていること (D11a) も同じく `README.md` が定める。この文書は
それを引くだけで、実行のモデルを自分では置かない。
この文書が自分で置くのは、局所定義「本体の変数表」「`cancel` の所有述語」「段の素動作と段内の点」
「段内の点で閉じている」と、命題 L0d、L0b、L0c、L0、L0a、L1、L2、L2b、L3、L4、L5 と、第 4 節末尾の
「系 (環境が何も持たずに終わる実行)」、および外部の結果 `EXT LLVM モジュールの記号名` だけである。

P29 は層 0 の命題であり、その証明は README のどの命題も引かない。P30 は層 2 の命題であり、その証明は
P1、P9、P12、P24 の**言明**を引く。P27 の証明が引く README の命題は P28 の**言明**だけである --
参照の持ち手がちょうど 1 つであることを、L4 (`H` の分解) が読む。P27 が使うのは D11 と D12 という述語
そのものであって、それを `borrow_ify` と `cancel` が保存することではない。命題 L0 は P30 の言明に
`cancel` が読む所有についての主張を足したものであり、L0b と L0c と L0d を引く。

**コード生成は単位ごとに走るので、`L0d` の主語は分割前のプログラムと単位の切片の 2 つである。**D24 は
「プログラム `P` の実行は、`P` を分割して生成した単位を結合したものの実行である」と述べ、`P.funcs` の
各項目がちょうど 1 つの単位へ入ること、単位が定義しない名前についてその単位が持つのは宣言だけで
あること、リンクがその宣言をその名前を持つ単位が実装した本体へ解決することを定める。**`L0b`・`L0`・
`P29`・`P30` の `Q` は分割前のプログラム (`P`、`P'`) である。**`L0d` はその `Q` を主語に取り、1 つの
単位のモジュールの中の勘定と、単位を跨ぐ解決とを分けて述べる。

**`L3` は D11a を実行の側で果たす命題である。**D11a は、参照カウントが 1 以上である計数下オブジェクトが
どれも解放されていない点を解放について閉じていると呼び、「**プログラムのすべての本体が D11 を満たすとき、
その実行 (D24) の各時点と各段内の点で閉じている**」と述べる。D11 の (S-c) はその接頭を条件に取る。
この 2 種の点でこれが成り立つことを `L3` が示し、`L2b` が段内の点を 1 つずつ運ぶ。

## 1. 結論

| 主張 | 結果 |
|---|---|
| P29 静的に決めた呼び出し先は実行時の呼び出し先である | 証明した (L0b を入力に当てる) |
| P30 `borrow_ify` の出力についても静的に決めた呼び出し先は実行時の呼び出し先である | 証明した (L0b を出力に当てる) |
| L0 (b) `cancel` が読む所有は実行時の呼び出し先の所有である | 証明した |
| L3 実行の各時点と各段内の点は解放について閉じている (D11a) | 証明した |
| L4 `H(o)` は 3 種の持ち手が持つ `o` への参照の個数の和である | 証明した (P28 (a) の上で) |
| (R1) 解放されたオブジェクトの読みが起きない | 証明した |
| (R2) どのオブジェクトも高々 1 回しか解放されない | 証明した |
| (R3) 正常終了する実行で解放されずに残る計数下のオブジェクトは、環境が持つ参照が指すオブジェクトから到達できるものに限る | 証明した |

**(R2) は L3 と L2b の上で閉じ、L3 が使うもののほかに仮定を足さない。**D11 の (S-c) が `Retain(v, π)` と
`Release(v, π)` の**触れる**先を扱うので、L2b の `Retain` の場合が (S-c) から直接に出る。

**関数の値に番地を書き込む段の数え上げは A21 が持つ。**この文書はそれを引く。`llvmgen-function-values.md`
が A21 の果たす者の数え上げである。

**EXT LLVM モジュールの記号名**。1 つの LLVM モジュールにおいて、記号名は関数を高々 1 つ決める。
`Module::get_function(s)` は記号名が `s` の関数が在ればそれを返し、無ければ `None` を返す。
`Module::add_function(s, ..)` は、記号名 `s` の関数が既に在るとき、`s` とは別の記号名を持つ新しい関数を
作る -- LLVM がモジュールの記号表で名前を付け替える。`declare_lambda_function` の doc がこの振る舞いを
「LLVM resolves a collision between two such names by renaming one of them」と述べる
(`CODE src/generator.rs: Generator::declare_lambda_function`)。

**P27 は P28 の 4 つの前提を仮定に持つ。**P28 は D12 と、借用する unit を持つ本体の活性化を作る段が
(E3) に限られることと、A20 と、そのプログラムが `insert_rc` の入力から `cancel` の出力までのどこかに
現れることとを前提に取る。P27 が量化するのは D12 を満たす任意の `RcProgram` なので、残る 3 つは P27 の
仮定に置く。`borrow_ify` の出力について範囲の制限と (E3) への限定を果たすのは P14b であり、その範囲は
`cancel` の出力を含む。A20 は仮定として置く。

## 2. 静的に決めた呼び出し先

### L0d (名前で得た LLVM 関数が実装する `RcFunc`) <!--#3eee4d8-->

**L0d.** ASSUME  NEW `Q`: RcProgram -- コード生成が単位へ分割するプログラム (D24)、 <!--#25f7f06-->
                 **(N3)** `Q` の `funcs` の各鍵の名前は局所名 (`FullName::is_local` が真) ではなく、
                 コード生成が読む `global_types` はその名前を持たないか、funptr 型で持つ、
                 A22
         PROVE   **(a)** 1 つの単位の切片 `U` を渡した `implement_rc_program` が作る表 `func_vals` は、
                 `U.funcs` の各鍵 `fref` に対して、`Q.funcs[fref]` の本体を実装した LLVM 関数を与える。
                 **(b)** `Q.funcs` の鍵の名前 `n` について、コード生成が `n` の値として返すのは
                 `declare_lambda_function` が funptr 型で登録した LLVM 関数そのもの (アクセサの呼び出し
                 ではない) であり、**実行時にその値が指す関数は `Q.funcs[FuncRef{n}]` の本体を実装した
                 ものである**。

**主語が 2 つあるのは、コード生成が単位ごとに走るからである。**D24 は「プログラム `P` の実行は、`P` を
分割して生成した単位を結合したものの実行である」と述べ、`divide_among_units` が `P.funcs` の各項目を
その名前が決めるちょうど 1 つの単位へ入れること、コード生成が単位ごとに `implement_rc_program` をその
単位の切片について 1 回走らせることを定める。(a) は 1 つのモジュールの中の勘定であり、(b) は名前が
実行時に何を指すかという単位を跨ぐ言明である。(N3) の `global_types` は
`global_types_including_synthesized` が `Q` から 1 度だけ作って全単位が共有する表なので
(`CODE src/build/build_object_files.rs: build_object_files`)、単位ごとに別ではなく、(N3) は分割前の
`Q` について読める。

<1>0. 各単位の切片 `U` について、`U.funcs` の鍵は `Q.funcs` の鍵の部分集合であり、各鍵 `fref` の値は
      `Q.funcs[fref]` である。
  **在りかは述語で決める** -- `RcProgram` の `funcs` へ項目を入れる式は、その写像に `insert` を掛ける
  全出現であり、`src/` に 9 か所ある (改行を跨ぐものを含む)。**一覧で書くとパスが 1 つ増えるたびに
  古くなる。**そのうち `unit_programs` の要素を書き換えるのは 2 か所である。`divide_among_units` は
  `Q.funcs` の各項目 `(fref, func)` を `unit_of[&fref.name]` の単位へそのまま入れ、
  `import_what_each_unit_reaches` は `copyable_funcs` -- `Q.funcs` の項目を名前で引ける形に写したもの --
  の値を鍵 `FuncRef { name }` で入れる。残る 7 か所は別の写像を組み立てる -- `lower_program` と
  `Lowerer::lower_lam` が `Lowerer` の `funcs` へ、`borrow_ify` が自分の出力の `funcs` へ (2 か所)、
  `insert_rc` が `new_funcs` へ、`src/rc_ir/locality.rs` と `src/rc_ir/unique_check_elim.rs` が
  それぞれの `output_funcs` へ入れる。項目を落とす側は、`publish_and_prune` が呼ぶ
  `eliminate_unreachable` の `prog.funcs.retain(..)` だけであり、鍵に対する値を替えない。
  BY CODE src/build/divide_program.rs: divide_among_units, import_what_each_unit_reaches,
     copyable_funcs, publish_and_prune,
     CODE src/rc_ir/dead_code_elim.rs: eliminate_unreachable,
     CODE src/rc_ir/lower.rs: lower_program, Lowerer::lower_lam,
     CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/rc_insert.rs: insert_rc

<1>1. (a) が成り立つ。
  `implement_rc_program` は `U.funcs` の各 `(fref, func)` について LLVM 関数を 1 つ決めて `func_vals` に
  入れ、続く走査で `implement_rc_function(func, func_vals[fref], ..)` を呼ぶ。`implement_rc_function` は
  その LLVM 関数に `entry` ブロックを付けて `func.body` を出す。`<1>0` よりその `func` は
  `Q.funcs[fref]` である。
  BY <1>0, CODE src/rc_ir/codegen.rs: Generator::implement_rc_program,
     CODE src/rc_ir/codegen.rs: Generator::implement_rc_function

<1>2. コード生成が `n` の値として返すのは、`declared_globals` に `n` で登録された `ValueAccessor` が
      答える値である。
  **コード生成が変数の名前から値を得る道は `get_scoped_value` の 1 か所である** -- `src/` でその関数を
  呼ぶのは `get_scoped_obj` と `get_scoped_obj_noretain` の 2 つだけであり、どちらもその答えの
  `accessor` に `get` を掛ける。`get_scoped_value` は `var.is_local()` のときだけスコープを引き、
  そうでなければ `get_or_declare_global` を通る。(N3) より `n` は局所名ではないので後者である。
  `get_or_declare_global` は `declared_globals` に在ればそれを返し、無ければ
  `declare_program_global` を呼んで登録させ、それが `None` を返せば panic する。
  **名前から値を読む場所は述語で決める** -- `get_scoped_obj(`・`get_scoped_obj_noretain(`・
  `get_scoped_obj_field(` (3 つ目は `get_scoped_obj` を呼ぶ) の全出現であり、`src/` に定義を除いて
  143 か所ある。**一覧で書くと op が 1 つ増えるたびに古くなる。**その 143 か所は 3 群に分かれ、
  どれも上の道を通る。**`RcExpr` の節点について呼ぶもの、`src/rc_ir/codegen.rs` に 12 か所** --
  `eval_rc_expr_inner` の `Ret`/`Retain`/`Release`/`Eval`/`Destructure`/`App` の腕、`eval_rc_rhs` の
  `RcRhs::Var` の腕、`eval_rc_match` の scrutinee と payload の読み、`build_rc_closure` の capture の
  読みである。**`Llvm` 節点のオペランドについて呼ぶもの、`src/fixstd/builtin.rs` に 127 か所と
  `src/generator.rs` に 2 か所** -- `Let(x, Llvm(gen, args), k)` の腕は `llvm_gen.generate_tail` を
  呼び、各 op の `generate` / `generate_tail` が自分のオペランドを名前から読む
  (`get_scoped_obj_field` と `build_capture_project` がその 2 か所である)。**環境がグローバルを読む
  もの、2 か所** -- `ExportStatement::implement` と `build_main_function` である (D22)。
  BY (N3), <ref id=243ae2c/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_rhs, Generator::eval_rc_expr_inner,
     Generator::build_rc_closure, Generator::eval_rc_match,
     CODE src/ast/inline_llvm.rs: LLVMGen::generate, LLVMGen::generate_tail,
     CODE src/generator.rs: Generator::get_scoped_obj, Generator::get_scoped_obj_noretain,
     Generator::get_scoped_obj_field, Generator::build_capture_project,
     Generator::get_scoped_value, Generator::get_or_declare_global,
     CODE src/ast/export_statement.rs: ExportStatement::implement,
     CODE src/build/build_object_files.rs: build_main_function

<1>3. `n` について `declared_globals` に登録が在るならば、その登録は `declare_lambda_function` が
      `fn_ty.is_funptr()` の枝で行ったものであり、登録された値はその呼び出しが作った LLVM 関数である。
      登録は高々 1 つである。
  `declared_globals` に名前を登録するのは `add_global_object` だけであり、`src/` にその呼び出しは 2 つ
  ある -- `declare_program_global` が `global_types` の型が funptr でないときそのアクセサ関数を登録する
  行と、`declare_lambda_function` が `fn_ty.is_funptr()` のときいま作った LLVM 関数自身を登録する行で
  ある。(N3) より `n` は `global_types` に無いか funptr 型で在るので、`declare_program_global(n)` は
  `global_types.get(n)` が `None` の腕で `None` を返すか、`is_funptr()` の枝で
  `declare_lambda_function(&ty, n)` を呼ぶかのどちらかであり、アクセサを登録する行には着かない。よって
  `n` の登録を行うのは `declare_lambda_function` の funptr の枝だけであり、そこが登録するのは同じ呼び出しの
  `module.add_function` が作った関数である。同じ名前で 2 度登録すると `add_global_object` はコンパイルを
  止めるので、登録が 2 つ在るプログラムは走らず、この場合の段は存在しない。
  BY (N3), CODE src/generator.rs: Generator::add_global_object, Generator::declare_program_global,
     Generator::declare_lambda_function

<1>3a. 相異なる 2 つの鍵の名前は相異なる記号名を持ち、鍵の名前の記号名は `::` を含む。
  `FullName::to_string` は名前空間の各成分を `::` で継いだ列に `::` と `name` を継いだ文字列である。
  名前空間の成分も `name` も `::` を含まない。**名前空間の成分の在りかは述語で決める** -- 成分は
  module 宣言と namespace 宣言が与える `namespace_item` (`capital_name` を `.` で継いだもの) か、
  `FullName::to_namespace` が名前を名前空間の末尾へ移したものかのどちらかであり、後者の在りかは
  `to_namespace()` の全出現である。`src/` に 48 か所あり、うち 2 か所
  (`CODE src/ast/import.rs: ImportStatement::import_to_use_with_spans`,
  `ImportStatement::add_import`) は import 文の項目の列を取り出す読みであって新しい名前を作らない。
  残る 46 か所が末尾へ移す名前は 3 族に分かれる --
  **トレイト名** (`CODE src/ast/program.rs: Program::trait_member_symbols`)、**型名**
  (`CODE src/ast/program.rs: Program::add_methods` のゲッタ・セッタ)、**値の名前**である。値の名前を
  末尾へ移すのは 3 か所で、`fresh_closure_ref` が持ち上げる lambda の名前、
  `CODE src/elaboration/desugar_opaque.rs: Program::desugar_opaque_types` が作る `#wrap_opaque`、
  そして `CODE src/elaboration/desugar_opaque.rs: collect_opaque_infos` が不透明型の tycon に付ける
  名前である。トレイト名と型名は `capital_name`、値の名前は `name` であり、どちらも Fix の識別子にコンパイラが
  `#<タグ><10 進数字>` の形の接尾辞を足しうる形である (A13)。Fix の識別子を作る文字は英数字と `_`
  (と値の名前の頭の `@`) だけなので、どの成分も `:` を持たない。**コンパイラが自分で作る名前も同じで
  ある** -- `#wrap_opaque` と不透明型の tycon の名前 (`?` に続く型変数の名前) はどちらも `:` を持たない。
  よって `to_string` は相異なる名前に相異なる文字列を与える。
  `object_file_symbol_name` はその文字列の `SYMBOL_VERSION_SEPARATOR` を
  `SYMBOL_VERSION_SEPARATOR_SUBSTITUTE` に置き替えるだけであり、置き替える前の文字列が substitute を
  含まないことをその関数の表明が検査する -- 含むプログラムはコンパイルされず、この場合の段は存在しない --
  ので、置換も相異なる文字列を相異なる文字列へ移す。(N3) より鍵の名前は局所名ではなく、`is_local` は
  名前空間が空であることなので、鍵の名前空間は空でなく、その記号名は `::` を含む。
  BY (N3), <ref id=cb35ab1/>, CODE src/ast/name.rs: FullName::to_string, FullName::is_local, NameSpace::to_string,
     FullName::to_namespace,
     CODE src/generator.rs: object_file_symbol_name,
     CODE src/rc_ir/lower.rs: Lowerer::fresh_closure_ref,
     CODE src/parse/grammer.pest: name_char, namespace_item, capital_name, module_defn,
     global_defns_in_namespace

<1>3b. この単位のモジュールに記号名 `object_file_symbol_name(n)` の関数を作る `module.add_function`
       の呼び出しは、`declare_lambda_function` の 1 か所だけである。
  `src/` の `module.add_function(` は 17 か所であり、`object_file_symbol_name` の値をそのまま名前に
  するのは `declare_lambda_function` の 1 か所である。残る 16 か所を、渡す名前の形で分ける。
  **鍵の名前の記号名は、その名前が住む module の名前で始まる。**記号名は最も外側の名前空間の成分から
  始まり、その成分はその module の名前である (`CODE src/ast/name.rs: FullName::module` の doc --
  「The module the name lies in, which is the first name of its namespace.」)。module 宣言が与える
  `namespace_item` は `capital_name` を `.` で継いだもの、`capital_name` は `ASCII_ALPHA_UPPER` で
  始まる語なので、**記号名の先頭の成分は英大文字で始まり `#` を持たない**。`<1>3a` の 3 族はどれも
  `FullName::to_namespace` が名前を名前空間の**末尾**へ移す形なので、最も外側の成分を替えない。
  `object_file_symbol_name` が置き替えるのは `SYMBOL_VERSION_SEPARATOR`、すなわち `@` であって、
  先頭の成分の文字ではない。
  **英小文字で始まる名前**を渡すのは 10 か所である -- 走時の記号名 `fixruntime_...`・`sprintf`・
  `pthread_once`・`malloc`・`realloc` を渡す `runtime.rs` の 7 か所、`<接頭辞>_<型のハッシュ>` を渡す
  `emit_rc_helper_call` の 1 か所、そして走査関数の名前 `trav_<work><状態>_<型のハッシュ>` と
  `fixruntime_empty_traverser`・`fixruntime_empty_traverser_dynamic` を渡す `object.rs` の
  `create_traverser` と `get_traverser_ptr` である。**先頭の文字は名前を組む関数が決める。**走時の
  記号名は `runtime.rs` の `RUNTIME_*` の定数の値である。`emit_rc_helper_call` が組むのは
  `format!("{}_{}", prefix, obj.ty.hash())` であり、`prefix` はこの関数を呼ぶ 4 か所が渡す
  `retain<接尾辞>`・`release<接尾辞>`・`mark_global`・`mark_threaded` のいずれかである
  (`RcState::name_suffix` が返す接尾辞は先頭に付かない)。`create_traverser` が組むのは
  `TypeNode::traverser_name` の返り値 `format!("{}{}_{}", work_name, state.name_suffix(),
  self.hash_with_capture(capture))` であり、`work_name` は `trav_dyn`・`trav_release`・
  `trav_mark_global`・`trav_mark_threaded` の 4 つである。よってこれらは
  `object_file_symbol_name(n)` ではない。
  **最初の `::` より前に `#` を持つ名前**を渡すのは 5 か所である -- `global_accessor_name` の `Get#`
  (`declare_program_global`)、`InitValue#` と `InitOnce#` (`implement_rc_global` の 2 か所)、
  `release#` と `retain#` (`builtin.rs` の 2 か所) である。上に見たとおり記号名の先頭の成分は `#` を
  持たないので、これらも `object_file_symbol_name(n)` ではない。
  **残る 1 か所は `FFI_CALL` が呼ぶ C の関数の名前である** (`ffi.rs` の
  `CSignature::get_or_declare_in_module`)。**これは形では分けられない** -- 文法の `ffi_c_fun_char` は
  `(` 以外の任意の文字を許すので、その名前は鍵の記号名と同じ綴りでありうる。A26a が「`FFI_CALL` が
  名指す C 関数の名前は、このプログラムのどのグローバルの `object_file_symbol_name` とも異なる」を
  与えるので、これも `object_file_symbol_name(n)` ではない。
  BY <ref id=cb35ab1/>, <ref id=29a890a/>, <1>3a, CODE src/generator.rs: object_file_symbol_name, global_accessor_name,
     Generator::declare_lambda_function, Generator::declare_program_global,
     Generator::emit_rc_helper_call, Generator::retain, Generator::release, Generator::mark_global,
     Generator::mark_threaded,
     CODE src/ast/types.rs: TypeNode::hash_with_capture, TypeNode::traverser_name,
     CODE src/rc_ir/ast.rs: RcState::name_suffix,
     CODE src/ast/name.rs: FullName::module,
     CODE src/parse/grammer.pest: capital_name,
     CODE src/constants.rs: SYMBOL_VERSION_SEPARATOR,
     CODE src/rc_ir/codegen.rs: Generator::implement_rc_global,
     CODE src/object.rs: create_traverser, get_traverser_ptr,
     CODE src/fixstd/builtin.rs: InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody,
     InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody,
     CODE src/fixstd/runtime.rs: build_runtime, CODE src/ffi.rs: CSignature::get_or_declare_in_module,
     CODE src/rc_ir/lower.rs: Lowerer::fresh_closure_ref,
     CODE src/parse/grammer.pest: ffi_c_fun_char, namespace_item

<1>4. CASE この単位の切片 `U` が `n` を定義する。すなわち `FuncRef{n}` が `U.funcs` の鍵である。
      このとき、コード生成が `n` の値を読む時点で `declared_globals` は `n` の登録を持ち、そこに
      登録された LLVM 関数は `func_vals[FuncRef{n}]` と同じものである。
  <2>1. `n` について `declare_lambda_function` を呼ぶ道は 2 つであり、どちらも
        `module.add_function(&object_file_symbol_name(n), ..)` で LLVM 関数を作り、
        `fn_ty.is_funptr()` のときだけ `add_global_object` でその関数を登録して、その関数を返す。
    `declare_program_global(n)` は `global_types.get(n)` の型が funptr のとき
    `declare_lambda_function(&ty, n)` を呼んでその返り値を `Some` で返し、`implement_rc_program` は
    第 3 枝で `declare_lambda_function(&func.fn_ty, &func.name.name)` を呼ぶ。
    BY CODE src/generator.rs: Generator::declare_lambda_function, Generator::declare_program_global,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_program
  <2>2. 第 1 ループが鍵 `FuncRef{n}` の項目に着く時点で、モジュールは記号名 `object_file_symbol_name(n)`
        の関数を持たない。
    <3>1. 第 1 ループが鍵 `FuncRef{n}` の項目に着くまでに走った `module.add_function` の呼び出しの
          うち、`object_file_symbol_name(n)` を名前とするものは無い。
      `<1>3b` より、その名前を作りうるのは `declare_lambda_function` の 1 か所である。
      **第 1 ループの前に走るのは 4 つである** -- `build_object_files` は単位ごとのスレッドの中で、
      `Generator::create_module`、`Generator::new`、`config.debug_info` が真のときの
      `gc.create_debug_info()`、`build_runtime` の `Declare` の呼び出しを、この順に
      `gc.implement_rc_program(&unit_program)` の前に置く。**この 4 つは
      `declare_lambda_function` へ届かない。**`src/` で `declare_lambda_function` を呼ぶのは
      `declare_program_global` と `implement_rc_program` の 2 か所、`declare_program_global` を呼ぶのは
      `get_or_declare_global`・`implement_rc_program`・`implement_rc_global` の 3 か所、
      `get_or_declare_global` を呼ぶのは `get_scoped_value` の 1 か所であり、`implement_rc_program` を
      呼ぶのはこの `build_object_files` の 1 か所、`implement_rc_global` を呼ぶのは
      `implement_rc_program` の第 2 ループの 1 か所である。よって第 1 ループの前にこの道へ入るには
      `get_scoped_value` を通る要があるが、`<1>2` よりそこへ入る 143 か所は、`RcExpr` の節点の生成と
      `Llvm` 節点のオペランドの読み -- どちらも `implement_rc_program` の第 2 ループが呼ぶ
      `implement_rc_function` の中で走る -- と、`build_object_files` が `implement_rc_program` の**後**に
      置く `ExportStatement::implement` と `build_main_function` である。`create_module` はモジュールを
      作り、`Generator::new` は欄を埋め、`create_debug_info` は `add_basic_value_flag` と
      `create_debug_info_builder` を呼び、`build_runtime` の `Declare` は走時関数の宣言を出す。
      第 1 ループの
      3 枝が呼ぶのは `module.get_function`、`declare_program_global`、`declare_lambda_function` だけで
      あり、`<1>3` より (N3) の下で `declare_program_global` はアクセサを登録する行に着かないので、
      より前の項目が `declare_lambda_function` に渡す名前はその項目の鍵の名前 `n'` である。鍵は
      相異なるので `n' ≠ n` であり、`<1>3a` よりその記号名は `object_file_symbol_name(n)` と異なる。
      BY (N3), <1>2, <1>3, <1>3a, <1>3b, CODE src/build/build_object_files.rs: build_object_files,
         CODE src/rc_ir/codegen.rs: Generator::implement_rc_program, Generator::implement_rc_function,
         Generator::implement_rc_global,
         CODE src/generator.rs: Generator::declare_program_global, Generator::create_module,
         Generator::new, Generator::create_debug_info, Generator::get_scoped_value,
         Generator::get_or_declare_global,
         CODE src/fixstd/runtime.rs: build_runtime
    <3>2. QED
      モジュールが記号名 `s` の関数を持つのは、`module.add_function` がその名前で関数を作った後だけで
      ある (`EXT LLVM モジュールの記号名` -- `Module::get_function(s)` は記号名が `s` の関数が在れば
      それを返し、無ければ `None` を返す)。`<3>1` より、この時点までにその名前で関数を作った呼び出しは
      無い。
      BY EXT LLVM モジュールの記号名, <3>1
  <2>3. `n` について `declare_lambda_function` が走るのは、第 1 ループの鍵 `FuncRef{n}` の項目の第 2 枝
        か第 3 枝の、ちょうど 1 度である。その項目は第 1 枝を取らない。さらに、コード生成が `n` の値を
        読むとき、その時点で `declared_globals` は `n` の登録を持つ。
    <3>1. その項目は第 2 枝か第 3 枝を取り、そのどちらも `declare_lambda_function` を `n` について
          1 度だけ呼ぶ。
      第 1 枝に入るのは `module.get_function(&object_file_symbol_name(n))` が `Some` を返したときで
      あり、`<2>2` よりそれは起きない。第 2 枝は `declare_program_global(n)` が `Some` を返した場合で
      あり、(N3) の下でそれは `is_funptr()` の枝、すなわち `declare_lambda_function(&ty, n)` を呼んだ
      場合である (`<2>1`)。第 3 枝は `declare_program_global(n)` が `None` を返した場合、すなわち
      `global_types` が `n` を持たない場合であり、`declare_lambda_function(&func.fn_ty, n)` を呼ぶ。
      3 つの枝は排他であり、鍵は 1 つの項目にしか現れない。
      BY (N3), <2>1, <2>2, CODE src/rc_ir/codegen.rs: Generator::implement_rc_program,
         CODE src/generator.rs: Generator::declare_program_global
    <3>2. 第 1 ループの後、`n` の値を読むコード生成の時点で `declared_globals` は `n` の登録を持つ。
          よってその読みは `declare_lambda_function` を `n` について呼ばない。
      `<1>2` より、その読みは `get_or_declare_global(n)` を通る。`declared_globals` が `n` を持てば
      それを返して終わり、`declare_program_global` を呼ばない。持たない場合を `<3>1` の 2 つの場合で
      分ける。第 2 枝を取った場合は `declare_lambda_function` が funptr 型の `ty` で呼ばれているので
      `<2>1` より `n` は登録済みであり、持たない場合に当たらない。第 3 枝を取った場合は `global_types`
      が `n` を持たないので `declare_program_global(n)` は `None` を返し、`get_or_declare_global` の
      `unwrap_or_else` が panic する -- そのときプログラムはコンパイルされず、この場合の段は存在しない。
      よって読みが在る実行では、その時点で登録が在る。
      BY <1>2, <2>1, <3>1, CODE src/generator.rs: Generator::get_or_declare_global,
         Generator::declare_program_global
    <3>3. QED
      BY <3>1, <3>2
  <2>4. `func_vals[FuncRef{n}]` は `<2>3` のその 1 度が作った LLVM 関数である。
    `implement_rc_program` は `U.funcs` の各項目 `(fref, func)` について `func.name.name` を取り、まず
    `module.get_function(&object_file_symbol_name(func.name.name))` を引き、在ればそれを使い、無ければ
    `declare_program_global` を、それも `None` なら `declare_lambda_function` を呼び、得た関数を
    `func_vals[fref]` に入れる。A22 より `func.name` は鍵 `fref` に等しいので、鍵 `FuncRef{n}` の項目に
    ついて取る名前は `n` である。`<2>3` よりその項目は第 2 枝か第 3 枝を取る。第 2 枝が入れるのは
    `declare_program_global(n)` の返り値であり、`<2>1` より (N3) の下でそれは `declare_lambda_function`
    の返り値である。第 3 枝が入れるのは `declare_lambda_function` の返り値そのものである。どちらも
    `<2>3` のその 1 度の返り値である。
    BY (N3), <ref id=8d3e4af/>, <2>1, <2>3, CODE src/rc_ir/codegen.rs: Generator::implement_rc_program,
       CODE src/generator.rs: Generator::declare_program_global
  <2>5. QED
    `<2>3` より、`n` の値を読むコード生成の時点で `declared_globals` は `n` の登録を持つ。`<1>3` より
    その登録を行うのは `declare_lambda_function` の funptr の枝であり、登録されるのはその呼び出しが
    作った LLVM 関数である。`<2>3` よりその呼び出しは 1 度だけであり、`<2>4` より
    `func_vals[FuncRef{n}]` はその 1 度が作った関数である。よって 2 つは同じものである。
    BY <1>3, <2>3, <2>4

<1>4a. CASE この単位の切片 `U` が `n` を定義しない。すなわち `FuncRef{n}` は `U.funcs` の鍵ではない。
       このとき、コード生成が `n` の値を読む時点で `declared_globals` は `n` の登録を持ち、その登録が
       持つ LLVM 関数の記号名は `object_file_symbol_name(n)` であって、この単位はその名前の本体を
       出さない。すなわちこの単位が `n` について持つのは宣言だけである。
  <2>1. `n` の値を読む時点で `declared_globals` は `n` の登録を持ち、それを行ったのは
        `declare_lambda_function` の funptr の枝である。
    `<1>2` より、その読みは `get_or_declare_global(n)` を通る。`declared_globals` が `n` を持てば
    それを返して終わる。持たなければ `declare_program_global(n)` を呼び、(N3) より `global_types` は
    `n` を funptr 型で持つか、`n` を持たないかのどちらかである。funptr 型で持つならば
    `declare_lambda_function(&ty, n)` が走り、その `fn_ty.is_funptr()` の枝が `add_global_object` で
    登録する。持たないならば `declare_program_global(n)` は `global_types.get(n)` が `None` の腕で
    `None` を返し、`get_or_declare_global` の `unwrap_or_else` が panic する -- そのときプログラムは
    コンパイルされず、この場合の段は存在しない。どちらにせよ登録を行うのは `<1>3` より
    `declare_lambda_function` の funptr の枝である。
    BY (N3), <1>2, <1>3, CODE src/generator.rs: Generator::get_or_declare_global,
       Generator::declare_program_global, Generator::declare_lambda_function,
       Generator::add_global_object
  <2>1a. `<2>1` の `declare_lambda_function` が作る LLVM 関数の記号名は
         `object_file_symbol_name(n)` である。この単位のモジュールで、その記号名を持つ関数はそれ 1 つ
         だけである。
    `declare_lambda_function` は `module.add_function(&object_file_symbol_name(n), ..)` を呼ぶ。
    `EXT LLVM モジュールの記号名` より、`add_function` が渡された名前と別の記号名を付けるのは、その
    名前の関数がモジュールに既に在るときだけである。`<1>3b` より、`object_file_symbol_name(n)` を
    名前として関数を作りうる呼び出しは `declare_lambda_function` の 1 か所であり、**その 1 か所を
    `FFI_CALL` の C 関数名から分けるのは A26a である** -- この単位はこの名前の本体を出す段を持たない
    ので、`<1>4` の場合が使う「第 1 ループの項目の順序」による分離は当たらない。`declare_lambda_function`
    が `n` について走るのはこの 1 度である -- `<1>3` より `declared_globals` の `n` の登録は高々 1 つ
    であり、`get_or_declare_global` は登録が在れば `declare_program_global` を呼ばず、`FuncRef{n}` は
    `U.funcs` の鍵ではないので第 1 ループもこの名前について呼ばない。他の名前 `n'` について走る呼び出し
    が作るのは `<1>3a` より別の記号名である。よって衝突は起きず、改名も起きない。
    BY <ref id=29a890a/>, EXT LLVM モジュールの記号名, <1>3, <1>3a, <1>3b, <2>1,
       CODE src/generator.rs: Generator::declare_lambda_function, Generator::get_or_declare_global,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_program
  <2>2. この単位は `n` の本体を出さない。
    `<1>1` よりこの単位が本体を出す LLVM 関数は `func_vals` の値であり、その鍵は `U.funcs` の鍵で
    ある。この場合 `FuncRef{n}` はその鍵ではない。`declare_lambda_function` は `module.add_function` で
    作った関数をそのまま返し、ブロックを付けない。
    BY <1>1, CODE src/rc_ir/codegen.rs: Generator::implement_rc_program,
       CODE src/generator.rs: Generator::declare_lambda_function
  <2>3. QED
    D24 が「**単位が定義しない名前について、その単位が持つのは宣言だけである。** リンクがその宣言を、
    その名前を持つ単位が実装した本体へ解決する」と述べる。`<2>1` と `<2>2` が、この単位が `n` について
    持つのが本体の無い登録であることを与え、`<2>1a` が、その宣言の記号名が
    `object_file_symbol_name(n)` であって、この単位でそれが別のものへ解決されないことを与える。
    **後者を与えるのは A26a である** -- A26a はこの命題を読む者として名指されており、その仮定が無いと、
    同じ綴りの C 関数名を `FFI_CALL` が持つプログラムで `CSignature::get_or_declare_in_module` が
    この宣言を返し、その名前が記号でなく C 関数へ解決される。
    BY <ref id=29a890a/>, <ref id=e3436e8/>, <2>1, <2>1a, <2>2

<1>5. (b) が成り立つ。
  `<1>2` より、`n` の値を読むコード生成が返すのは `declared_globals` の `n` の登録の `ValueAccessor` が
  答える値である。`<1>3` よりその登録は `declare_lambda_function` の funptr の枝が置いた
  `ValueAccessor::Global(fun, ty)` であり、`ty.is_funptr()` なので `ValueAccessor::get` は
  `fun.as_global_value().as_basic_value_enum()` を返す -- アクセサを呼ばず、関数そのものを値とする。
  `FuncRef{n}` が `U.funcs` の鍵であるかどうかで 2 つの場合に尽きる。鍵であるならば、`<1>4` より
  その関数は `func_vals[FuncRef{n}]` であり、`<1>1` よりそれは `Q.funcs[FuncRef{n}]` の本体を実装した
  LLVM 関数である。**鍵である場合は 2 通りある** -- D24 は、`divide_among_units` が `Q.funcs` の各項目を
  1 つの単位へ入れ、「**そのうえで、`import_what_each_unit_reaches` が写しを配る**-- その名前に届く各単位へ
  `funcs` の項目を複製し、写しはその単位の内部の定義になる」と述べ、「**定義を持つ単位は分割が決めた
  1 つだが、その本体を持つ単位は 1 つとは限らない。**」と続ける。どちらでも `<1>0` より
  `U.funcs[FuncRef{n}]` は `Q.funcs[FuncRef{n}]` なので、出る本体は同じである。
  鍵でないならば、`<1>4a` よりこの単位が持つのは記号名 `object_file_symbol_name(n)` の宣言だけで
  ある。
  いずれの場合も、D24 が「**したがって、実行時に名前 `n` が指す関数は `P.funcs[FuncRef{n}]` の本体を
  実装したものである**-- どの単位がそれを実装したかに依らない」と述べる。その `P` がこの命題の `Q` で
  ある。よってその値が実行時に指す関数は `Q.funcs[FuncRef{n}]` の本体を実装したものである。
  **`<1>4a` の場合にその解決を与えるのは A26a である** -- `<1>4a` が結論する、宣言の記号名が
  `object_file_symbol_name(n)` であるという事実がその仮定の上に立つ。
  BY <ref id=29a890a/>, <ref id=e3436e8/>, <1>0, <1>1, <1>2, <1>3, <1>4, <1>4a, CODE src/generator.rs: ValueAccessor::get

<1>6. QED
  BY <1>1, <1>5

### L0b (名前が決める呼び出し先) <!--#35510ec-->

**DEF 本体の変数表**
`vars` は、その節点を含む本体の**変数表** `VarTable` である。関数の本体については
`VarTable::of(func)` が、グローバル初期化子の `init` については `VarTable::body_only(init)` が作る
(`CODE src/rc_ir/ownership.rs: VarTable`, `VarTable::of`, `VarTable::body_only`)。以下で
`vars.closure_targets` と `vars.bindings` と言うのはこの表の欄である。

**L0b.** ASSUME  NEW `Q`: RcProgram、 <!--#890138b-->
                 **(N1)** `Q` のすべての束縛変数の名前は相異なり、`Q` の `funcs` のどの鍵の名前とも
                 異なる、
                 **(N2)** `Q` の変数の使用は、その位置でスコープに入っている束縛に解決する、
                 **(N3)** `Q` の `funcs` の各鍵の名前は局所名 (`FullName::is_local` が真) ではなく、
                 コード生成が読む `global_types` はその名前を持たないか、funptr 型で持つ、
                 A22、
                 NEW `Let(x, App(callee, args), k)`: `Q` のある本体の節点、
                 NEW `vars`: その本体の変数表 (DEF 本体の変数表)、
                 `resolve_callee_params(callee, vars, Q) = Some(params)`
         PROVE   `Q` から生成したコードの実行のその節点の段について、`params` を持つ `Q.funcs` の関数は
                 その段の実行時の呼び出し先 (D23) と同じ `RcFunc` である。したがって `params` も
                 その `borrowed_units` も、実行時の呼び出し先のものである。

**「`Q` から生成したコード」は D24 の意味である** -- `Q` を単位へ分割して単位ごとに生成し、結合した
ものである。この節点を含む本体を出す単位の切片を `U` と書く。

<1>1. `resolve_callee_params(callee, vars, Q)` が `Some(params)` を返すのは 2 つの場合であり、
      `None` を返すのはそれ以外である。
  1 つは `vars.closure_targets` が `callee.name` を持つ場合、もう 1 つは `Q.funcs` が
  `FuncRef { name: callee.name }` を持つ場合である。どちらでもなければ `?` が `None` を返す。どちらの
  場合も返るのは `Q.funcs` のその `FuncRef` の関数の `params` である。
  BY CODE src/rc_ir/ownership.rs: resolve_callee_params

<1>2. `vars.closure_targets` が `callee.name` を持つのは、この本体に `Let(callee, Closure(fref, caps), k)`
      が在るとき、かつそのときに限る。そのとき返るのは `Q.funcs[fref]` のパラメータである。
  `VarTable::of` は関数の各パラメータと capture に `Binding::Param` を置いてから `collect_bindings` を
  本体に掛け、`VarTable::body_only` は `collect_bindings` だけを掛ける (DEF 本体の変数表)。
  `collect_bindings` が `closure_targets` に挿入するのは `RcRhs::Closure(fref, _)` の腕の 1 か所だけで
  あり、挿入する鍵はその `Let` の束縛変数の名前である。
  BY <1>1, DEF 本体の変数表, CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only,
     collect_bindings, resolve_callee_params

<1>3. `<1>2` の場合、実行時の呼び出し先は `Q.funcs[fref]` である。
  <2>1. `callee` の値は、その `Let` が束縛した値である。
    (N1) より `callee.name` を束縛する節点はこの `Let` の 1 つだけであり、(N2) より `callee` の使用は
    その束縛に解決する。コード生成はその束縛を橋渡しする -- `Let` の腕は右辺の値を `bind_and_continue`
    に渡し、それが `scope_push(&x.name, &obj)` でスコープに積み、継続を出してから `scope_pop` する。
    `App` の腕は `get_scoped_obj(&callee.name)` でスコープを引く。よって `callee` の位置で読まれるのは、
    その `Let` が積んだ `obj` である。
    BY (N1), (N2), <1>2, CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
       Generator::bind_and_continue, CODE src/generator.rs: Generator::scope_push,
       Generator::get_scoped_obj
  <2>2. その値は funptr の欄に `func_vals[fref]` を持つクロージャである。ここで `fref` はこの節点を
        含む本体を出している単位の切片 `U` の `funcs` の鍵である。
    `Let` の右辺が `Closure(fref, caps)` のとき、`eval_rc_rhs` は `build_rc_closure` を呼び、
    `build_rc_closure` は `CLOSURE_FUNPTR_IDX` の欄に `func_vals[func]` を入れる。`func_vals` は
    `U.funcs` の鍵で索かれるので、`fref` がその鍵でなければコード生成はそこで止まり、そのプログラムは
    コンパイルされず、この場合の段は存在しない。
    BY <2>1, CODE src/rc_ir/codegen.rs: Generator::eval_rc_rhs, Generator::build_rc_closure,
       Generator::implement_rc_program
  <2>3. QED
    D23 より、`callee` の値がクロージャのとき実行時の呼び出し先はその funptr の指す関数である
    (`apply_lambda` は `get_lambda_func_ptr` が返す関数ポインタを `build_indirect_call` で呼び、
    `get_lambda_func_ptr` は `is_closure()` のとき `CLOSURE_FUNPTR_IDX` の欄を読む)。`<2>2` より
    `fref` は `U.funcs` の鍵なので L0d (a) が当たり、`func_vals[fref]` は `Q.funcs[fref]` の本体を
    実装した LLVM 関数である。**L0d の ASSUME は `Q` と (N3) と A22 であり、その 3 つはこの命題の
    ASSUME に在る。**
    BY (N3), <ref id=8d3e4af/>, <ref id=ff5985d/>, <ref id=3eee4d8/>, <2>2, CODE src/generator.rs: Generator::apply_lambda,
       CODE src/generator.rs: Generator::get_lambda_func_ptr

<1>4. `Q.funcs` が `FuncRef { name: callee.name }` を持つ場合、実行時の呼び出し先はその関数である。
  <2>1. コード生成が `callee.name` の値として返すのは funptr であり、実行時にそれが指す関数は
        `Q.funcs[FuncRef{callee.name}]` の本体を実装したものである。
    L0d (b) を `n = callee.name` に当てる。**L0d の ASSUME は `Q` と (N3) と A22 であり、その 3 つは
    この命題の ASSUME に在る。**`App` の腕はその名前を `get_scoped_obj` で引く。
    BY (N3), <ref id=8d3e4af/>, <ref id=3eee4d8/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
       CODE src/generator.rs: Generator::get_scoped_obj
  <2>2. QED
    D23 より、`callee` の値が funptr のとき実行時の呼び出し先はそれ自身である
    (`get_lambda_func_ptr` は `is_funptr()` のとき値そのものを関数ポインタとする)。
    BY <ref id=ff5985d/>, <2>1, CODE src/generator.rs: Generator::get_lambda_func_ptr

<1>5. QED
  `<1>1` より `Some(params)` が返るのは 2 つの場合であり、どちらでも `params` は `Q.funcs` のある
  `FuncRef` の関数 `g` のパラメータの列である。`<1>3` と `<1>4` が、その 2 つの場合のそれぞれについて、
  `g` が実行時の呼び出し先であることを与える。`params` も `g.borrowed_units` も `g` の欄なので、
  実行時の呼び出し先のものである。
  BY <1>1, <1>2, <1>3, <1>4

### L0c (入力の関数の名前が (N3) を満たす) <!--#b2d588b-->

**言明**。A13 を満たす `borrow_ify` の入力プログラム `P` を取る。`P.funcs` の各鍵の名前は局所名では
ない。さらに **(N0)** `P.funcs` のどの鍵の名前も `P.globals` のどの要素の `symbol` とも異なるならば、
コード生成が読む `global_types` は `P.funcs` のどの鍵の名前も持たないか、funptr 型で持つ。

**(N0) を果たすのは `divide_into_units` である。**その関数は `funcs` の鍵の名前と `globals` の `symbol`
を 1 つの列に集め、同じ名前が 2 度現れれば `panic!` で止まる (`<2>3` の `<3>2`)。破れたプログラムは
コードを生成しないので、その実行の段も存在しない -- L0b の結論は「`Q` から生成したコードの実行のその
節点の段について」の形なので、段が在る場合には (N0) が満たされる。

<1>1. `P.funcs` の鍵と `P.globals` の `symbol` の集合は、lowering の出力のものである。
  `borrow_ify` の入力は、lowering の出力に `simplify`、`insert_rc`、`split_rc_units` をこの順で掛けた
  ものである (`CODE src/build/build_object_files.rs: lower_and_insert_rc`, `optimize_rc_program`)。
  `simplify` と `split_rc_units` は `funcs` の各値の `body` と `globals` の各要素の `init` を写すだけで
  あり、`insert_rc` は各項目を同じ鍵 `fref` で入れ直し、各グローバル初期化子の `init` だけを差し替える。
  どれも鍵も `symbol` も足さず、消さず、替えない。
  BY CODE src/build/build_object_files.rs: lower_and_insert_rc, optimize_rc_program,
     CODE src/rc_ir/simplify.rs: simplify, CODE src/rc_ir/rc_insert.rs: insert_rc,
     CODE src/rc_ir/borrow.rs: split_rc_units

<1>2. `P.funcs` の鍵の名前は局所名ではない。
  A13 が「**最上位の記号の名前は局所名ではない。** `FullName::is_local` が偽であり、`prog.funcs` の鍵と
  `global_types` の鍵はどちらもそのような名前である」を与える。
  BY <ref id=cb35ab1/>

<1>3. コード生成が読む `global_types` は、`P.funcs` の鍵の名前を持たないか、funptr 型で持つ。
  <2>1. `declare_program_global` が引く `global_types` は、`global_types_including_synthesized` の
        返り値である。
    `divide_among_units` はその返り値を `DividedProgram::global_types` に入れ、`build_object_files` は
    その欄を取り出して単位ごとに `Generator::new` の `global_types` 引数へ渡す。`Generator` はそれを
    同名の欄に持ち、`declare_program_global` は `self.global_types.get(name)` を引く。
    BY CODE src/build/divide_program.rs: divide_among_units, global_types_including_synthesized,
       DividedProgram, CODE src/build/build_object_files.rs: build_object_files,
       CODE src/generator.rs: Generator::new, Generator::declare_program_global
  <2>2. `global_types_including_synthesized(P, ・)` は 3 系統の項目からなる。引数の `global_types` --
        `Program::global_types`、すなわち最上位の記号の名前からその型への写像 -- の全項目、`fn_ty` が
        funptr である `P.funcs` の各項目をその `fn_ty` で上書き挿入したもの、そして `P.globals` の
        各要素をその `symbol` と `ty` で上書き挿入したものである。
    BY CODE src/build/divide_program.rs: global_types_including_synthesized,
       CODE src/ast/program.rs: Program::global_types
  <2>3. 持ち上げた lambda に `fresh_closure_ref` が付ける名前は、最上位の記号 -- `Program::symbols` の
        鍵、すなわち `Program::global_types` の鍵 -- の名前ではない。
    <3>0. `Program::symbols` の各項目の鍵は、その項目の `Symbol` の `name` に等しい。
      **在りかは述語で決める** -- `Program` の `symbols` の欄へ項目を入れる式であり、`src/` でその欄と、
      その欄へ代入される写像とに `insert`・`extend` を掛ける全出現である。**一覧で書くとパスが 1 つ
      増えるたびに古くなる。**鍵を新しく作るのは 7 か所で、どれも同じ名前をその `Symbol` の `name` に
      置く -- `Program::instantiate_symbols` は `sym.name` を鍵に取り、`uncurry::run` の funptr 版と
      `closure_specialization` の特殊化版は `Symbol { name: name.clone(), .. }` を `name` で入れ、
      `split_struct_args::split_one_argument` の twin、`defunctionalize_fix` の持ち上げた関数
      (`into_symbol` が `func_name` を `name` に置く)、`closure_specialization::register_lifted_lambdas`
      が持ち上げた lambda はどれもその `Symbol` の `name` を鍵に取り、`simplify_symbol_names::run` は
      `sym.name = new_name` の後で `new_name` を鍵にする。残る式は、`Program::symbols` から取り出した項目をその鍵のまま入れ直すか、
      項目を落とすだけである。**`Symbol` の `name` を書き換える式は `simplify_symbol_names::run` の
      1 か所だけである。**
      BY CODE src/ast/program.rs: Program::instantiate_symbols, Symbol,
         CODE src/optimization/simplify_symbol_names.rs: run
    <3>0a. `lower_program` に渡る記号の名前の集合は `Program::global_types` の鍵の集合であり、
           `lower_symbol` は funptr の記号を鍵がその記号の名前である `funcs` の項目に、funptr でない
           記号を `symbol` がその記号の名前である `globals` の要素にする。持ち上げた lambda を `funcs`
           に入れるのは `Lowerer::lower_lam` である。
      `build_object_files` は `program.global_types()` の返り値と `program.symbols.values()` を並べて
      取り、後者を `lower_and_insert_rc` を経て `lower_program` の `symbols` に渡す。
      `Program::global_types` は `self.symbols` の各鍵をそのまま鍵とし、`lower_symbol` が読むのは
      `sym.name` なので、2 つの集合が同じであることは `<3>0` が与える。
      BY <3>0, CODE src/build/build_object_files.rs: build_object_files, lower_and_insert_rc,
         CODE src/ast/program.rs: Program::global_types,
         CODE src/rc_ir/lower.rs: lower_program, Lowerer::lower_symbol, Lowerer::lower_lam
    <3>1. 持ち上げた lambda の名前は、funptr の最上位の記号の名前と等しくない。
      `<3>0a` より、等しければその名前で `Lowerer` の `funcs` へ 2 度挿入が起き、2 度目の
      `assert!(previous.is_none(), "two RC IR functions are named ...")` が発火する。この表明は
      `develop_mode` の門を持たないので、そのプログラムはコンパイルされず、lowering の出力から作られる
      `borrow_ify` の入力 `P` も存在しない。
      BY <3>0a, CODE src/rc_ir/lower.rs: lower_program, Lowerer::lower_lam
    <3>2. 持ち上げた lambda の名前は、funptr でない最上位の記号の名前とも等しくない。
      `<3>0a` より、等しければその名前は `P.funcs` の鍵であり、かつ `P.globals` のある要素の `symbol` で
      ある。(N0) がそれを禁じる。**(N0) を果たすのは `divide_into_units` である** -- その関数は
      `funcs` の鍵の名前と `globals` の `symbol` を 1 つの列に集めて整列し、隣り合う 2 つが等しければ
      `panic!("the program defines `{}` twice, ...")` で止まる。この panic は `develop_mode` の門を
      持たず、`build_object_files` は `divide_into_units` を `divide_among_units` の前に呼ぶので、
      (N0) が破れたプログラムはコードを生成せず、その実行も段も存在しない。**D24 が「プログラム `P` の
      実行は、`P` を分割して生成した単位を結合したものの実行である」と定めるので、`P` から生成した
      コードについて語るとき `divide_into_units` は `P` に掛かる。**
      BY (N0), <3>0a, <ref id=e3436e8/>, CODE src/build/divide_program.rs: divide_into_units,
         CODE src/build/build_object_files.rs: build_object_files
    <3>3. QED
      `<3>0a` より `lower_symbol` は最上位の記号を funptr かそうでないかで振り分けるので、`<3>1` と
      `<3>2` が 2 つの場合を尽くす。
      BY <3>0a, <3>1, <3>2
  <2>4. `P.globals` の `symbol` は、最上位の記号のうち型が funptr でないものの名前であり、`P.funcs` の
        どの鍵の名前とも異なる。
    `<1>1` より `symbol` と鍵は lowering の出力のものであり、`lower_symbol` は最上位の記号を、その型が
    funptr なら `funcs` の鍵 (鍵の名前はその記号の名前そのもの) に、そうでなければグローバル初期化子の
    `symbol` にする。相異なる記号は相異なる名前を持つので、非 funptr の記号の名前は funptr の記号の名前
    と異なる。`funcs` の残る鍵は持ち上げた lambda のものであり、`<2>3` よりそれは最上位の記号の名前では
    ないので `symbol` とも異なる。
    BY <1>1, <2>3, CODE src/rc_ir/lower.rs: Lowerer::lower_symbol
  <2>5. QED
    `<2>1` よりコード生成が読むのは `global_types_including_synthesized` の返り値である。funptr の関数の
    名前については、`<2>2` の第 2 系統がそれを `fn_ty` で入れ、`<2>4` より第 3 系統はその鍵を上書き
    しないので、funptr 型で在る。funptr でない関数の名前については、第 2 系統は `fn_ty.is_funptr()` の
    項目しか入れないので当たらない。`<1>1` よりその鍵は lowering の出力のものであり、`lower_symbol` が
    `funcs` の鍵にする最上位の記号は funptr のものだけなので、それは持ち上げた lambda の名前である。
    `<2>3` よりそれは最上位の記号の名前ではないので第 1 系統に項目が無く、`<2>4` より `P.globals` の
    `symbol` でもないので第 3 系統にも項目が無い。
    BY <1>1, <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/lower.rs: Lowerer::lower_symbol

<1>4. QED
  BY <1>2, <1>3

### P29 の証明

**P29.** ASSUME  NEW `P`: `borrow_ify` の入力プログラム、
                 A6、A11、A13、A22、
                 NEW `Let(x, App(callee, args), k)`: `P` のある本体の節点、
                 NEW `vars`: その本体の変数表 (DEF 本体の変数表)、
                 `resolve_callee_params(callee, vars, P) = Some(params)`
        PROVE   `params` を持つ `P.funcs` の関数はその段の実行時の呼び出し先 (D23) と同じ `RcFunc` で
                ある。したがって `params` もその `borrowed_units` も、実行時の呼び出し先のものである。

<1>1. `P` は L0b の (N1) を満たす。
  A6 が「`borrow_ify` の入力のすべての束縛変数の名前は相異なり、**どの関数の名前とも異なる**」を与え、
  A22 が各 `RcFunc` の `name` を `P.funcs` のその項目の鍵と同一視する。
  BY <ref id=33c54dc/>, <ref id=8d3e4af/>

<1>2. `P` は L0b の (N2) を満たす。
  A11 が「変数の使用は、その位置でスコープに入っている束縛に解決する」を与える。
  BY <ref id=3905b4e/>

<1>3. `P` は L0b の (N3) を満たす。
  L0c がまさにこれを述べる。**L0c の後半は (N0) の下での言明である。**この命題の結論は `P` から生成した
  コードの実行の段についてのものであり、L0c が述べるとおり (N0) が破れたプログラムは
  `divide_into_units` の `panic!` で止まってコードを生成しないので、段が在る場合には (N0) が満たされる。
  BY <ref id=cb35ab1/>, <ref id=b2d588b/>

<1>4. QED
  L0b を `Q = P` に当てる。A22 は L0b の仮定でもあり、この命題の仮定に在る。
  BY <ref id=8d3e4af/>, <ref id=35510ec/>, <1>1, <1>2, <1>3

**注**。`resolve_callee_params` が `None` を返す場合について P29 は何も言わない。そのとき
`rhs_consumes` は全位置を所有として扱う (`CODE src/rc_ir/ownership.rs: rhs_consumes` -- `callee_params`
が `None` のとき `is_owning_position` は `true`)。A7 が置く近似がこれである。

### L0 (`cancel` が読む所有と実行時の呼び出し先) <!--#2e3dd41-->

**DEF `cancel` の所有述語**
**`owns_cancel`** とは、`cancel(prog, type_env)` の走査の `consume_rhs` が `rhs_consumes` へ渡す述語
`owns` に付けた名前である。`owns_cancel(p, λ)` は
`owned_units.contains((p.name, truncate_to_unit(ty(p), λ)))` であり、`owned_units` は
`all_owned_units(prog, type_env)` である
(`CODE src/rc_ir/borrow.rs: cancel`, `CancelAnalysis::consume_rhs`)。

**言明**。A6、A10、A11、A13、A21、A22、A24 の下で、`borrow_ify` の入力プログラム `P` と、`borrow_ify` の
出力プログラム `P'` を取る。`P'` のどの
`Let(x, App(callee, args), k)` と、その節点を含む本体の変数表 `vars` (DEF 本体の変数表) についても、
次の 2 つが成り立つ。

- **(a)** `resolve_callee_params(callee, vars, P')` が `Some(params)` を返すならば、`params` を持つ
  `P'.funcs` の関数はその段の実行時の呼び出し先 (D23) と同じ `RcFunc` である。`None` を返すならば、
  その段の実行時の呼び出し先の `borrowed_units` は空である。
- **(b)** `Some(params)` の場合、実行時の呼び出し先を `g` とすると、各 `i` と `ty(params[i])` の各 boxed
  leaf `λ` について、`owns_cancel(params[i], λ)` が真であることと、`g` が
  `truncate_to_unit(ty(params[i]), λ)` を D14 の意味で所有することは同値である。

**(a) の前半が README の P30 である。** その証明を下の「P30 の証明」が L0 (a) から 1 段で書く。
(a) の後半 -- `None` を返す場合に実行時の呼び出し先の `borrowed_units` が空であること -- は P30 が
何も言わない部分であり、この文書が局所命題として足す強化である。`cancel` の走査は `None` のとき全位置を
所有として扱うので、その扱いが実行時の呼び出し先の所有と食い違わないことを言うのがこの後半である。

**(b) が読まれる場所。** `rhs_consumes` は `App(callee, args)` の第 `i` 引数の leaf `λ` を、
`resolve_callee_params` が返した `params` について `owns_cancel(params[i], λ)` が真であるときに消費と
して報告する (`CODE src/rc_ir/ownership.rs: rhs_consumes`)。(b) は、その報告が D9 の `App` の行 --
実行時の呼び出し先の所有 -- と一致することである。

<1>1. `P'` は L0b の (N1) を満たす。
  <2>1. `P'` の束縛変数の名前は、`P` の束縛変数の名前か、`clone_func` が作る複製の名前である。
    `borrow_ify` は出力の各版を、原本 `func.clone()` の本体を `RewriteCtx::rewrite` で書き換えたものか、
    `clone_func` の複製の本体を同じく書き換えたものとして作る。グローバル初期化子も同じ `rewrite` を
    通る。P24 の第 5 項より、書き換えが本体について変えるのは `Retain`/`Release` の節点と `App` の
    callee の名前だけであり、`Let` の束縛変数は元の本体のものに等しい。足される `Retain`/`Release` は
    束縛を持たない (D2)。
    BY <ref id=b3dfa37/>, <ref id=746e87a/>, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func
  <2>2. `P` の束縛変数の名前は相異なり、`P` のどの関数の名前とも異なる。
    BY <ref id=33c54dc/>
  <2>3. 複製が導入する名前は、`P` のどの束縛名とも異なる。
    BY <ref id=63eadd9/>
  <2>4. 相異なる 2 つの複製が導入する名前は互いに異なる。
    `borrow_ify` は 1 つの `rename_counter` をすべての `clone_func` の呼び出しに渡し、
    `assign_fresh_name` は呼ばれるたびにそれを 1 増やしてから `<元の名前>#b<counter>` を作る。
    **この形の分解は一意である** -- 複製名の `#` で区切った最後の断片は `b` の後に 10 進数字だけが
    続く形であり、A13 より入力のどの名前もその形の断片を最後に持たないので、複製名の最後の `#` は
    `assign_fresh_name` が入れたものであって元の名前の中の `#` ではない。よって
    `<元の名前>` と `<counter>` は複製名から一意に読み取れ、2 つの複製名が等しければ `counter` が
    等しく、同じ呼び出しである。
    BY <ref id=cb35ab1/>, CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/rename.rs: fresh_rename_function,
       assign_fresh_name
  <2>5. 複製が導入する名前は、`P'` のどの関数の名前とも異なる。
    `borrow_ify` が出力の `funcs` に入れるのは、原本を `f_own.name` で入れる項目と、複製を
    `borrow_version` で入れる項目だけである。A22 より前者の名前は `P.funcs` の鍵であり、後者は
    `borrow_funcref` が作る `<元の名前>#borrow` である。A13 より `P` に現れるどの名前も `#` で区切った
    最後の断片が `b<10 進数字>` の形ではないので、`P` の関数の名前は複製名と異なる。`#borrow` で終わる
    名前も、最後の断片が `b<10 進数字>` の形ではないので複製名とは異なる。
    BY <ref id=cb35ab1/>, <ref id=8d3e4af/>, CODE src/rc_ir/borrow.rs: borrow_ify, borrow_funcref
  <2>6. `P` の束縛変数の名前は、`P'` のどの関数の名前とも異なる。
    `<2>2` が `P` の関数の名前について与える。`#borrow` で終わる名前については A13 が、`P` に現れる
    どの名前も最後の断片が `borrow` ではないと言う。
    BY <ref id=cb35ab1/>, <2>2
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>2. `P'` は L0b の (N2) を満たす。
  `borrow_ify` は出力の各本体を、原本 `func.clone()` の本体か `clone_func` の複製の本体を
  `RewriteCtx::rewrite` で写して作る。複製は束縛変数の一斉の付け替えであり、それ以外の違いを持たない
  (P9) ので、束縛と使用の対応を保つ。P24 の第 5 項より、書き換えが本体について変えるのは
  `Retain`/`Release` の節点と `App` の callee の名前だけであり、節点の種類・その順序・`Let` の束縛変数・
  `Match` のアームの構成は元の本体のものに等しい。落とす節点も足す節点も束縛を持たない (D2)。よって
  A11 が `P` について与えるスコープの規律は `P'` でも成り立つ。
  BY <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=63eadd9/>, <ref id=746e87a/>, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>3. `P'` は L0b の (N3) を満たす。
  <2>1. `P'.funcs` の鍵の名前は、`P.funcs` の鍵の名前か、`borrow_funcref` が作る `<元の名前>#borrow`
        である。また `P'.globals` の `symbol` の集合は `P.globals` のものに等しい。
    `borrow_ify` は出力の `funcs` に、原本を `f_own.name` で、複製を `borrow_version` で入れる。A22 より
    前者は `P.funcs` の鍵であり、後者は `borrow_funcref` が作る名前である。グローバル初期化子については
    P24 が「出力のグローバル初期化子の列は入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力の
    第 `i` 要素のものに等しい」と述べる。
    BY <ref id=8d3e4af/>, <ref id=746e87a/>, CODE src/rc_ir/borrow.rs: borrow_ify, borrow_funcref
  <2>1a. `global_types_including_synthesized(P', ・)` は、`global_types_including_synthesized(P, ・)` に
         比べて、`<元の名前>#borrow` を鍵とする項目だけを余分に持つ。
    `global_types_including_synthesized(prog, global_types)` は、引数の `global_types` (最上位の記号の
    名前からその型への写像) を写した上に、`prog.funcs` のうち `fn_ty.is_funptr()` である項目をその
    `fn_ty` で、`prog.globals` の各要素をその `symbol` と `ty` で上書き挿入する。**引数の
    `global_types` は `P` と `P'` で同じ値である** -- `build_object_files` はそれを
    `let global_types = Arc::new(program.global_types());` で `Program` から 1 度だけ作り、その 1 つを
    `optimize_rc_program` にも `divide_among_units` にも渡す。`optimize_rc_program` の中で
    `borrow_ify` が走るので、その前後でこの表は動かない。`Program::global_types` が読むのは
    `Program::symbols` であって `RcProgram` ではない。
    `<2>1` より `P'.funcs` の鍵は `P.funcs` の鍵と `borrow_funcref` が作る `<元の名前>#borrow` で尽き、
    `P'.globals` の `symbol` の集合は `P.globals` のものに等しい。
    BY <2>1, CODE src/build/divide_program.rs: global_types_including_synthesized,
       CODE src/build/build_object_files.rs: build_object_files, optimize_rc_program,
       CODE src/ast/program.rs: Program::global_types
  <2>2. `P.funcs` の鍵の名前について (N3) が成り立つ。
    局所名でないことは A13 が与える。A13 より `P` に現れるどの名前も `#` で区切った最後の断片が
    `borrow` ではないので、`P.funcs` の鍵は `<元の名前>#borrow` の形の名前と異なる。よって `<2>1a` の
    余分な項目はこの鍵に当たらず、`P'` について読む `global_types` はこの名前について
    `global_types_including_synthesized(P, ・)` と同じ答えを返す。L0c がその答えについて (N3) を与える。
    **L0c の (N0) はこの場合に満たされる** -- `<2>1` より `P'.funcs` の鍵は `P.funcs` の鍵と
    `<元の名前>#borrow` で尽き、`P'.globals` の `symbol` の集合は `P.globals` のものに等しいので、
    `P` で (N0) が破れれば `P'` でも同じ 2 つの名前が並ぶ。L0c が述べるとおりそのとき
    `divide_into_units` の `panic!` が `P'` について止まってコードを生成せず、この命題の結論が語る
    実行の段も存在しない。
    BY <ref id=cb35ab1/>, <ref id=b2d588b/>, <2>1, <2>1a, CODE src/rc_ir/borrow.rs: borrow_funcref
  <2>3. `<元の名前>#borrow` について (N3) が成り立つ。
    `borrow_funcref` は元の名前の `name` の欄に `#borrow` を足すだけなので名前空間は変わらず、A13 より
    局所名ではない。借用版の `fn_ty` は原本の `fn_ty` と等しい (`clone_func`) ので、funptr の借用版の
    名前は `global_types_including_synthesized` が funptr 型で挿入する。funptr でない借用版の名前に
    ついては、その 3 系統のどれもこの鍵の項目を作らないことを言えばよい。funptr の項目を入れる系統は
    `fn_ty.is_funptr()` の項目しか入れないので当たらない。残る 2 系統の鍵 -- 引数の `global_types` の鍵
    (最上位の記号の名前) と `P'.globals` の `symbol` -- は、`<2>1a` よりどちらも `borrow_ify` の入力の側の
    名前であり、A13 が `borrow_ify` の入力に現れるすべての名前について、`#` で区切った最後の断片が
    `borrow` でないと述べる。
    BY <ref id=cb35ab1/>, <2>1a, CODE src/rc_ir/borrow.rs: borrow_funcref, clone_func,
       CODE src/build/divide_program.rs: global_types_including_synthesized
  <2>4. QED
    BY <2>1, <2>1a, <2>2, <2>3

<1>4. (a) の前半が成り立つ。
  L0b を `Q = P'` に当てる。A22 は `P'` についても成り立つ -- `borrow_ify` は原本を `f_own.name` で、
  複製を `borrow_version` で `funcs` に入れ、`clone_func` は複製の `name` をその `borrow_version` に
  する。
  BY <ref id=8d3e4af/>, <ref id=35510ec/>, <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>5. (a) の後半が成り立つ。
  <2>1. `borrow_ify` が `borrow_versions` に入れるのは、`funcs_observing_uniqueness` が挙げず、
        `func.capture.is_none()` を満たし、`func_has_borrowable_param` が真である関数だけである。
        とくに capture を持つ関数には借用版が作られない。
    BY CODE src/rc_ir/borrow.rs: borrow_ify
  <2>2. 出力の関数のうち `borrowed_units` が空でないのは、`clones` が作った借用版だけである。
    `borrow_ify` は各 `func` について `param_capture_units(func, type_env)` を丸ごと `owned_units` に
    入れ、`borrowed_units` を `param_capture_units` から `owned_units` を引いたものとして書く。原本
    `f_own` のパラメータ・capture の unit はすべて `owned_units` に在るので、`f_own.borrowed_units` は
    空である。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units
  <2>3. 借用版の名前が `App` の `callee` に現れるのは、`route` がそれを置いたときだけである。
    `rewrite_inner` が `App` の `callee` を書き換えるのは `route` の呼び出し 1 か所だけであり、
    `RcRhs::Closure` は `RcExpr::Let(x, rhs, k)` の一般の腕で `rhs.clone()` としてそのまま運ばれるので、
    クロージャの目標が借用版に付け替わることは無い。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::route
  <2>4. `route` が借用版を置くのは、`callee.name` を名前とする `FuncRef` が `borrow_versions` の鍵で
        あるときだけである。`borrow_versions` の鍵は `func.name` であり、A22 よりそれは `P.funcs` の鍵で
        ある。`borrow_ify` は原本を `f_own.name` で出力に入れ、A22 よりそれも同じ鍵なので、そのとき
        `P'.funcs` はその鍵を持ち、`resolve_callee_params` は `prog.funcs.contains_key` の枝で `Some` を
        返す。P12 が「`route` が返す呼び出し先は、元の呼び出し先と同じ関数の版である (元の版そのものか、
        その `borrow_versions` の像)」と述べるのがこの形であり、呼び出し先が入力の関数を名指すとき返る
        名前は出力の `funcs` の鍵である。
    BY <ref id=8d3e4af/>, <ref id=843e506/>, CODE src/rc_ir/borrow.rs: RewriteCtx::route, borrow_ify,
       CODE src/rc_ir/ownership.rs: resolve_callee_params
  <2>4a. 借用版の名前が `RcVar` として現れるのは、`App` の `callee` の位置だけである。
    `borrow_ify` が出力の本体に借用版の名前を書くのは `route` の返り値だけであり、それは
    `RcRhs::App(callee, args)` の `callee` に置かれる。`clone_func` が導入するのは束縛変数の名前だけで
    あって関数の名前ではない (P9 の言明)。A13 より、`P` に現れるどの名前も `#borrow` で終わらないので、
    入力から運ばれた名前がたまたま借用版の名前と一致することも無い。
    BY <ref id=cb35ab1/>, <ref id=63eadd9/>, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::route, borrow_ify
  <2>4b. 実行時の関数の値が持つ LLVM 関数の番地は、次の 3 つのいずれかが置いたものである。
    A21 が、Fix の関数型の値に LLVM 関数の番地を書き込むのは次の 3 か所だけであり、ほかのどの構文も op も
    既にある関数の値を写すだけであると述べる。

    - **(M1)** クロージャを作る段 (`CODE src/rc_ir/codegen.rs: Generator::build_rc_closure` --
      `CLOSURE_FUNPTR_IDX` の欄に `func_vals[func]` を入れる)。
    - **(M2)** funptr のグローバルを読む段 (`CODE src/generator.rs: ValueAccessor::get` の
      `ty.is_funptr()` の枝が `fun.as_global_value().as_basic_value_enum()` を返す)。
    - **(M3)** `InlineLLVMFixBody` (`CODE src/fixstd/builtin.rs: InlineLLVMFixBody` --
      `generate_tail` が `gc.current_function().as_global_value().as_pointer_value()` を
      `CLOSURE_FUNPTR_IDX` の欄に入れて `fix(f)` のクロージャを組み立てる)。

    BY <ref id=ebec376/>, CODE src/rc_ir/codegen.rs: Generator::build_rc_closure,
       CODE src/generator.rs: ValueAccessor::get, CODE src/fixstd/builtin.rs: InlineLLVMFixBody
  <2>4c. (M3) が置く `gc.current_function()` は、`P'` の関数のうち `capture` が `Some` であるものの
         本体を実装した LLVM 関数である。
    <3>1. `InlineLLVMFixBody` の `Llvm` 節点は `P'` の関数の本体にしか在らず、その関数の `capture` は
          `Some` である。
      A24 は `borrow_ify` の入力 `P` の各関数について「その本体に `InlineLLVMFixBody` の `Llvm` 節点が
      在るならば、その関数の `capture` は `Some` である」を述べる。`P'` の各版は、原本 `func.clone()` の
      本体か `clone_func` の複製の本体を `RewriteCtx::rewrite` で写して作られ、P24 の第 5 項より
      書き換えが本体について変えるのは `Retain`/`Release` の節点と `App` の callee の名前だけなので、
      `Llvm` の op は元の本体のものに等しい。P9 より複製は束縛変数の一斉の付け替えであり、`clone_func`
      は原本の `capture` を `fresh_rename_function` が付け替えた形で持つので、`Some` であることは
      保たれる。グローバル初期化子の `init` については、`fix_body` が op の `cap_name` を局所名
      `#CAP` (`CAP_NAME`) に置き、`lower_llvm` が各自由変数の名前を `resolve` で引く。`resolve` が値を
      返す枝ではその名前がその位置で束縛されている変数であり、`#CAP` を束縛するのは
      `lower_lambda_as_function` の `lam_ty.is_closure()` の枝だけで、その枝が束縛するのはその関数の
      capture 変数である。よってこの op のオペランドは、その本体を持つ関数の capture 変数であり、それが
      グローバル初期化子の `init` に在れば `init` の自由な局所名になる。A11 は「グローバル初期化子の
      `init` は自由な局所名を持たない」と述べる。`resolve` が `None` を返す枝では、`lower_llvm` は
      `global_types` をその名前で引き、無ければ素の `panic!` で止まる -- `#CAP` は `FullName::local` が
      作る局所名であり (`CAP_NAME`)、A13 より `global_types` の鍵は局所名ではないので、この枝を通る
      プログラムはコンパイルされず、その本体の活性化は存在しない。
      BY <ref id=3905b4e/>, <ref id=cb35ab1/>, <ref id=675b350/>, <ref id=63eadd9/>, <ref id=746e87a/>, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func,
         CODE src/rc_ir/rename.rs: fresh_rename_function, CODE src/fixstd/builtin.rs: fix_body,
         CODE src/constants.rs: CAP_NAME, CODE src/rc_ir/lower.rs: Lowerer::lower_llvm,
         Lowerer::lower_lambda_as_function
    <3>2. `implement_rc_function` が `func` の本体を出している間、`gc.current_function()` はその本体を
          実装している LLVM 関数である。
      `implement_rc_function` は `fn_val` に `entry` ブロックを足して builder をそこへ据え、その下で
      `eval_rc_expr` を呼ぶ。`current_function` は builder の insert block の parent を返し、本体を出す
      間 builder が据わるのは `fn_val` に足したブロックなので、返るのは `fn_val` である。
      BY CODE src/rc_ir/codegen.rs: Generator::implement_rc_function,
         CODE src/generator.rs: Generator::current_function
    <3>3. QED
      (M3) が `CLOSURE_FUNPTR_IDX` に入れるのは `gc.current_function()` であり、その op のコードが
      生成されるのは、それを含む本体を `implement_rc_function` が出している間である。`<3>1` より
      その本体は `capture` が `Some` である `P'` の関数のものであり、`<3>2` より `gc.current_function()`
      はその関数の LLVM 関数である。
      BY <3>1, <3>2, CODE src/fixstd/builtin.rs: InlineLLVMFixBody
  <2>4d. `App` の `callee` の位置で読まれた値は、その節点の呼び出し先としてだけ使われる。
    `eval_rc_expr_inner` の `App` の腕は `get_scoped_obj(&callee.name)` の返り値を `callee_obj` に取り、
    それを `apply_lambda` の第 1 引数に渡す。ほかのどこにも渡さず、`scope_push` もしない。よってその値が
    別の節点の呼び出し先になることは無い。
    BY CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
       CODE src/generator.rs: Generator::apply_lambda, Generator::get_scoped_obj
  <2>5. QED
    `resolve_callee_params` が `None` を返すなら、`callee.name` は `P'.funcs` の鍵でも
    `closure_targets` の鍵でもない (`CODE src/rc_ir/ownership.rs: resolve_callee_params` -- 2 つの枝が
    どちらも当たらないときだけ `?` が `None` を返す)。実行時の呼び出し先は、`callee` の値がクロージャ
    ならその funptr が指す関数、funptr ならそれ自身であり、D23 はその関数がプログラムの `funcs` の
    関数であると述べる。その番地は `<2>4b` の 3 つのいずれかが置いたものである。
    (M1) が置くのは `func_vals[fref]` である。`build_rc_closure` は `func_vals` をその単位の切片 `U` の
    `funcs` の鍵で索くので、`fref` はその鍵であり、L0d (a) よりその関数は `P'.funcs[fref]` の本体を
    実装した LLVM 関数である。`<2>3` よりクロージャの目標 `fref` は入力から運ばれた名前、すなわち原本の
    名前である。
    (M2) が置くのは、funptr のグローバル `m` について `ValueAccessor::get` が返す LLVM 関数である。
    D23 より実行時の呼び出し先はプログラムの `funcs` の関数なので、`m` は `P'.funcs` の鍵である -- 鍵で
    なければ、L0d (a) と (b) が本体を実装したものと言うのは `funcs` の項目についてだけなので、その
    LLVM 関数はどの `RcFunc` の本体も実装していない。`<1>3` より `P'` は L0b の (N3) を満たすので、
    L0d (b) より実行時にその LLVM 関数が指すのは `P'.funcs[FuncRef{m}]` の本体を実装したもので
    ある。`m` は借用版の名前ではない --
    `<2>4a` より借用版の名前が `RcVar` として現れるのは `App` の `callee` の位置だけであり、`<2>4d` より
    そこで読まれた値はその節点の呼び出し先としてだけ使われるので、`m` が借用版の名前ならばこの節点の
    `callee.name` が `m` である。ところが `<2>4` より、`route` が借用版を置いた節点では
    `resolve_callee_params` が `Some` を返すので、`None` を返すこの節点はそれではない。よって `m` は
    原本の名前である。
    (M3) の関数は `<2>4c` より `capture` を持つ `P'` の関数の LLVM 関数であり、`<2>1` より capture を
    持つ関数には借用版が作られないので原本である。
    `<2>2` より原本の `borrowed_units` は空である。
    **L0d を `Q = P'` に当てる 2 か所の ASSUME は `Q` と (N3) と A22 である。**(N3) は `<1>3` が、
    A22 はこの言明の仮定が与える -- `borrow_ify` は原本を `f_own.name` で、複製を `borrow_version` で
    `funcs` に入れ、`clone_func` は複製の `name` をその `borrow_version` にするので、A22 は `P'` に
    ついても成り立つ (`<1>4` と同じ理由)。
    BY <ref id=8d3e4af/>, <ref id=ff5985d/>, <ref id=3eee4d8/>, <1>3, <1>4, <2>1, <2>2, <2>3, <2>4, <2>4a, <2>4b, <2>4c, <2>4d,
       CODE src/rc_ir/ownership.rs: resolve_callee_params,
       CODE src/rc_ir/codegen.rs: Generator::build_rc_closure,
       CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>6. (b) が成り立つ。
  <2>1. `params` は `g` のパラメータの列である。
    BY <1>4
  <2>2. `all_owned_units(P', type_env)` は、`P'` の各関数 `f` の各パラメータ・capture `p` と `ty(p)` の
        各 unit `u` について、`u` が `f.borrowed_units` に入らないとき、かつそのときに限り `(p.name, u)`
        を含む。
    BY CODE src/rc_ir/ownership.rs: all_owned_units
  <2>3. `(g.params[i].name, u)` を `all_owned_units` に入れうる関数は `g` だけである。
    `<2>2` より `(g.params[i].name, u)` を入れるのは、`g.params[i].name` を名前とするパラメータか
    capture を持つ関数である。パラメータと capture は束縛変数であり、`<1>1` より `P'` の束縛変数の名前は
    相異なるので、その名前を持つパラメータ・capture は `g.params[i]` 1 つだけである。
    BY <1>1, <2>2
  <2>4. QED
    `owns_cancel(params[i], λ)` は `owned_units.contains((params[i].name, truncate_to_unit(ty(params[i]),
    λ)))` である (DEF `cancel` の所有述語)。`λ` は `ty(params[i])` の boxed leaf なので、A10 と P1 より
    `truncate_to_unit(ty(params[i]), λ)` は値を返し、それは `rc_units(ty(params[i]))` の元 -- すなわち
    `<2>2` の `u` が渡る集合の元 -- である。`<2>1` より `params[i]` は `g.params[i]` であり、`<2>3` より
    その名前で `owned_units` に入る項目は `g` が入れたものだけである。よって `<2>2` より、その項目が
    在ることと `truncate_to_unit(ty(params[i]), λ)` が `g.borrowed_units` に入らないことは同値であり、
    D14 より後者は `g` がその unit を所有することである。
    BY <ref id=8412761/>, <ref id=ef8efc4/>, <ref id=3597669/>, <2>1, <2>2, <2>3, DEF `cancel` の所有述語

<1>7. QED
  BY <1>4, <1>5, <1>6

**注**。この文書が D9 と D10 を読むときの「呼び出し先」は D23 が定める実行時の関数である。L0 は、その読みの
下での義務集合が、`cancel` が静的に計算する消費と食い違わないことを述べる。食い違えば、D11 を保存すると
称する P14 と P23 が別の述語を保存していることになる。

### P30 の証明

**P30.** ASSUME  NEW `P`: `borrow_ify` の入力プログラム、
                 A6、A10、A11、A13、A21、A22、A24、
                 NEW `P'`: `borrow_ify` の出力プログラム、
                 NEW `Let(x, App(callee, args), k)`: `P'` のある本体の節点、
                 NEW `vars`: その本体の変数表 (DEF 本体の変数表)、
                 `resolve_callee_params(callee, vars, P') = Some(params)`
        PROVE   `params` を持つ `P'.funcs` の関数はその段の実行時の呼び出し先 (D23) と同じ `RcFunc` で
                ある。したがって `params` もその `borrowed_units` も、実行時の呼び出し先のものである。

<1>1. QED
  L0 (a) の前半がこの言明である。L0 の仮定は A6、A10、A11、A13、A21、A22、A24 であり、この命題の仮定に
  在る。`params` も `borrowed_units` も実行時の呼び出し先の欄なので、後半の文が従う。
  BY <ref id=33c54dc/>, <ref id=8412761/>, <ref id=3905b4e/>, <ref id=cb35ab1/>, <ref id=ebec376/>, <ref id=8d3e4af/>, <ref id=675b350/>, <ref id=2e3dd41/>

**注**。`resolve_callee_params` が `None` を返す場合について P30 は何も言わない。そのとき `rhs_consumes` は
全位置を所有として扱う (A7)。L0 (a) の後半が、その扱いが実行時の呼び出し先の所有と食い違わないことを
述べる。

## 3. 実行の命題

### L0a (オペランドを適用する `Llvm` の段が作る活性化) <!--#1b34a16-->

**言明**。`LLVMGen::applies_a_function_operand` が真を宣言する op を持つ節点
`Let(x, Llvm(gen, args), k)` の段 (以下**適用する `Llvm` の段**と呼ぶ) について、次が成り立つ。

- **(a)** その段はちょうど 1 つの活性化を作り、その段を持つ活性化 `a` は、その活性化が終わるまで中断中で
  ある。1 つの節点が活性化を 2 つ作るときは、この種の段がその節点について 2 つ在る。
- **(b)** その段が作った活性化 `b` が終わる (E4) の段で `Obl(b)` を離れる参照は `Obl(a)` に入り、その段で
  `H` は変わらない。

<1>1. (a) が成り立つ。
  D24 の (E2) の `Llvm` の段の段落が「その op の生成コードがオペランドを関数として適用するとき
  (`LLVMGen::applies_a_function_operand` が真を宣言する op)、適用された関数の本体の活性化が作られる」と
  述べ、続けて「**この段は活性化を 1 つ作るごとに区切られる。** `a` はその活性化が終わるまで中断中で
  あり、(E4) の後、同じ位置で次の段を実行する。1 つの節点が活性化を 2 つ作るときは、この種の段がその節点に
  ついて 2 つ在る」と述べる。
  宣言と生成コードが一致することは A3 が言う -- `applies_a_function_operand` は A3 が名指す 3 つの宣言の
  1 つである。
  BY <ref id=e11772a/>, <ref id=e3436e8/> (E2 の `Llvm` の段の段落),
     CODE src/ast/inline_llvm.rs: LLVMGen::applies_a_function_operand

<1>2. (b) が成り立つ。
  D24 の (E4) が「`b` を作ったのが (E2) のうちオペランドを適用する `Llvm` の段であれば、それらの参照は
  その段を実行した活性化の `Obl` に入り、その活性化は同じ位置で続きを実行する。**このとき
  (E2) の生成の表の `Llvm` の行はその leaf について読まない** -- `App` の行と同じ理由で、その参照は
  呼び出し先の中で作られてここへ渡ってくるものであり、`H` は動かない」と述べる。
  BY <ref id=e3436e8/> (E4)

<1>3. QED
  BY <1>1, <1>2

**注 (`Obl(a)` を離れる参照の行き先)**。L0a は (E3) の受け渡しの形を主張しない。D24 の (E2) の
`Llvm` の段の段落は「(E3) と違うのは 2 点である。呼び出し先を決めるのが `callee` の値ではなく op の
生成コードであること、**呼び出し先に渡る値がオペランドとは限らない**ことである」と述べ、続けて
「`Obl(a)` からオペランドの参照が離れる動きは D9 の `Llvm` の行が、結果の leaf に参照が生じる動きは
A3 の宣言が決めるので、上の表の行はそのままである」と述べる。`InlineLLVMFixBody` がその形を示す --
`fix(f)` のクロージャをその場で組み立てて `f` に渡し、返った関数に改めてオペランド `x` を渡すので、
1 回目の活性化の第 1 引数はどのオペランドでもなく、この段が組み立てた値である。D24 が「`InlineLLVMFixBody`
の 1 回目の適用に渡る `fix(f)` はこの段が組み立てた値であってオペランドではない」と書くのがこれである。
**この段はオブジェクトを 1 つも割り当てない。** `create_obj` に渡す `fixf_ty` は `f.ty.get_lambda_dst()`
すなわちクロージャ型であり、クロージャの値は unbox なので (`CODE src/object.rs: ty_to_object_ty` --
`is_closure()` の枝が `is_unbox` に `true` を置く)、`create_obj` は unbox の枝で `struct_type.get_undef()`
を返すだけで `build_malloc` を呼ばない (`CODE src/object.rs: create_obj`)。組み立ては `insert_field` で
funptr と capture の欄を埋め、`apply_lambda` を 2 回呼ぶ
(`CODE src/fixstd/builtin.rs: InlineLLVMFixBody`)。capture の欄に入るのはオペランド `cap` の参照であり、
D24 は「その capture の欄が持つのはオペランド `cap` の参照であり、読みが retain を伴わない
(`CODE src/generator.rs: Scope::push_local` -- `retain_on_read` が偽) ので、この組み立てで参照は
作られない」と述べる。よって `Obl(a)` を離れる参照の多重集合と作られた活性化の初期 `Obl` は一致しない。

### L1 (呼び出しと返りの受け渡しが釣り合う) <!--#f3dcc8f-->

**言明**。(a) と (b) はどちらも A12 の下で述べ、(a) はさらに A14 の下で述べる。A12 は各 `RcFunc` について
「`params` の型の列は `fn_ty` の lambda src の列に等しく、`capture` が `Some` であることと
`fn_ty.is_closure()` が真であることは同値であり、コード生成がその関数に与える署名は
`lambda_function_type(fn_ty)` である」と述べ、`App(callee, args)` について「`ty(callee)` は実行時の
呼び出し先の `fn_ty` である」「引数とパラメータのほかに、結果の型も一致する」と述べる。(b) が読むのは
最後の行である。

- **(a)** (E3) の段で `Obl(a)` を離れる参照の多重集合は、`Obl(b)` の初期値に一致する。
- **(b)** (E4) の段で `Obl(b)` を離れる参照の多重集合は、その段の行き先 -- `b` を (E3) が作ったならその
  親の `Obl(a)`、適用する `Llvm` の段 (L0a) が作ったならその段を実行している活性化の `Obl`、(F) の解放が
  作ったならその解放を含む段を実行している活性化の `Obl`、(E1) か (E7) が作ったなら `E` -- が得る参照の
  多重集合に一致する。

**(a) は、D24 の (E3) と D10 の初期値が同じ多重集合を指すことの検算である。**D24 の (E3) は `Obl(a)` を
離れた参照がそのまま `Obl(b)` の初期値になると定め、D10 は `Obl(b)` の初期値を所有するパラメータ・
capture の leaf から独立に定める。L3 が (E3) の段について読むのは D24 の (E3) の定めだけなので、P27 は
(a) を読まない。L2b の `Llvm` の場合が読むのは (b) である。

<1>1. (E3) の段で `Obl(a)` を離れるのは、D9 の `App` の行が挙げる leaf の参照である。すなわち `callee` の
      inhabited な全 boxed leaf と、呼び出し先がその位置の unit を所有する (D14) 引数の inhabited な leaf で
      ある。
  BY <ref id=9d74736/>, <ref id=e3436e8/> (E3)

<1>2. `Obl(b)` の初期値は、`b` の本体の関数が所有する (D14) パラメータ・capture の unit の下の inhabited な
      各 leaf につき 1 つである。
  BY <ref id=f06144e/> (初期値), <ref id=e3436e8/> (E3)

<1>2a. 呼び出しが使う署名と、実行時の呼び出し先 `g` の LLVM 関数の署名は、どちらも
       `lambda_function_type(g.fn_ty)` である。
  `apply_lambda` は `func_ty = lambda_function_type(&fun.ty, self)` を作って `build_indirect_call` に
  渡し、`fun.ty` は `ty(callee)` である。A12 より `ty(callee)` は `g.fn_ty` に等しく、コード生成が
  `g` に与える LLVM 関数の署名も `lambda_function_type(g.fn_ty)` である。
  BY <ref id=83d98e9/>, CODE src/generator.rs: Generator::apply_lambda, CODE src/object.rs: lambda_function_type

<1>2b. `ty(callee)` がクロージャ型であることと、`g.capture` が `Some` であることは同値である。
  A12 より `ty(callee)` は `g.fn_ty` に等しく、`g.capture` が `Some` であることと
  `g.fn_ty.is_closure()` が真であることは同値である。
  BY <ref id=83d98e9/>

<1>2c. `apply_lambda` が置く LLVM の実引数の列と、`implement_rc_function` が LLVM パラメータを読む列は、
       位置ごとに一致する。すなわち、結果が out-pointer で返るときの先頭の pointer、続く第 `i` 引数の
       parts、そして `ty(callee)` がクロージャ型のときの最後の CAP は、それぞれ
       `implement_rc_function` が飛ばす第 0 パラメータ、`g.params[i]` に束縛する parts、`g.capture` に
       束縛する 1 つの LLVM パラメータに当たる。
  `apply_lambda` は、`returns_through_out_pointer` が真のとき out-pointer を先頭に置き、続けて各引数の
  parts を順に置き、`fun.ty.is_closure()` のとき最後に `CLOSURE_CAPTURE_IDX` の欄を置く。
  `implement_rc_function` は、`returns_through_out_pointer` が真のとき第 0 パラメータを飛ばし、
  `g.params` の各 `param` について `param.ty` の parts の個数だけ LLVM パラメータを順に取り、
  `g.capture` が `Some` のとき次の 1 つを取る。どちらの `returns_through_out_pointer` も
  `lambda_return_part_types` を掛けた結果を読み、A12 より `ty(callee)` は `g.fn_ty` に等しいので
  真偽は一致する。A12 より `g.params` の型の列は `g.fn_ty` の lambda src の列、すなわち `ty(callee)` の
  lambda src の列に等しいので、parts の列も一致する。CAP の有無は `<1>2b` より一致する。
  BY <ref id=83d98e9/>, <1>2a, <1>2b, CODE src/generator.rs: Generator::apply_lambda,
     Generator::returns_through_out_pointer, CODE src/object.rs: lambda_function_type,
     lambda_return_part_types, CODE src/rc_ir/codegen.rs: Generator::implement_rc_function

<1>3. `callee` の inhabited な boxed leaf は、`b` の capture パラメータの inhabited な boxed leaf と
      1 対 1 に対応し、その unit は所有される。
  <2>1. `callee` の型がクロージャのとき、`boxed_leaf_paths` はその capture の位置 1 つだけを leaf とする。
    BY <ref id=0594f24/> (規則 2)
  <2>2. `callee` の型がクロージャのとき、実行時の呼び出し先 `g` は `capture` を持ち、`b` の入力の束縛が
        その capture に与える値は `callee` の値の capture の欄そのものである。
    `<1>2b` より `g.capture` は `Some` である。`<1>2c` より、`apply_lambda` が最後に置く
    `CLOSURE_CAPTURE_IDX` の欄が、`implement_rc_function` が `g.capture` の名前に束縛する LLVM
    パラメータである。retain も release も挟まない。
    BY <1>2b, <1>2c, CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>3. capture パラメータの型は boxed であり、その `boxed_leaf_paths` も `rc_units` も 1 元である。
    lowering は capture 変数の型を `make_dynamic_object_ty()` とする。それは `Std::#DynamicObject` の
    tycon であり、その `TyConInfo` は `is_unbox: false` を持つので boxed である。よって D4 の規則 3 と
    D5 の `unit_step` の `is_box` の腕がどちらも自分自身 1 つを返す。
    BY <ref id=0594f24/>, <ref id=9cba81c/>, CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function,
       CODE src/fixstd/builtin.rs: make_dynamic_object_ty, bulitin_tycons
  <2>4. capture の unit は所有される。
    D14 が「capture の unit は必ず所有される」を与える。
    BY <ref id=ef8efc4/>
  <2>5. `callee` の型が funptr のとき、`callee` は boxed leaf を持たず、実行時の呼び出し先は `capture` を
        持たない。
    funptr の型は `is_fully_unboxed` が真であり (`is_funptr()` の枝が `true` を返す)、D4 の規則 1 で
    leaf を持たない。`<1>2b` より、`ty(callee)` がクロージャ型でないとき `g.capture` は `None` である。
    BY <ref id=0594f24/>, <1>2b, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>6. QED
    D23 は `App(callee, args)` の実行時の呼び出し先を、`callee` の値がクロージャの場合はその funptr の
    指す関数、funptr の場合はそれ自身、の 2 つで定める。`<2>1`-`<2>4` が前者を、`<2>5` が後者を扱う。
    BY <ref id=ff5985d/>, <2>1, <2>2, <2>3, <2>4, <2>5

<1>4. 呼び出し先が所有する位置の引数の inhabited な leaf は、`b` の対応するパラメータの inhabited な leaf と
      1 対 1 に対応する。
  <2>1. `apply_lambda` は各引数の値をそのまま呼び出し先へ渡し、`implement_rc_function` は第 `i` 引数の
        値を `g.params[i]` の名前に束縛する。retain も release も挟まない。
    `<1>2c` より 2 つの列は位置ごとに一致し、`implement_rc_function` は `g.params[i]` に取った parts から
    `Object::from_parts` で値を組み直して `scope_push` する。
    BY <1>2c, CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>1a. `args` の個数は実行時の呼び出し先 `g` の `params` の個数に等しい。
    `apply_lambda` は `assert_eq!(args.len(), src_tys.len())` を持ち、`src_tys` は `fun.ty` すなわち
    `ty(callee)` の lambda src の列である。この表明が発火するプログラムはコンパイルされず、その場合の
    段は存在しない。A12 より `ty(callee)` は `g.fn_ty` に等しく、`g.params` の型の列は `g.fn_ty` の
    lambda src の列なので、`args` の個数は `g.params` の個数に等しい。A14 が「呼び出し先」を D23 の
    実行時の関数についても読むと述べるのがこの一致である。
    BY <ref id=83d98e9/>, <ref id=f8ae607/>, CODE src/generator.rs: Generator::apply_lambda
  <2>2. 第 `i` 引数の型は `g.params[i]` の型に等しいので、両者の `boxed_leaf_paths` は同じ列であり、
        `<2>1` より同じ値の同じ leaf を指す。よって inhabited (D16) であることも一致する。
    A12 の「`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型」の行がこれを与える。
    A14 と同じく、ここでの「呼び出し先」は D23 の実行時の関数である。
    BY <ref id=83d98e9/>, <ref id=f8ae607/>, <ref id=0594f24/>, <ref id=66c9670/>, <ref id=ff5985d/>, <2>1, <2>1a
  <2>3. QED
    D9 の `App` の行と D10 の初期値は、どちらも同じ所有の割り当て (D14) を同じ関数 -- D23 が定める実行時の
    呼び出し先 -- について読む。`<2>1a` より引数とパラメータは 1 対 1 であり、余るパラメータは無い。よって
    消費される引数の leaf と、初期 `Obl` に入るパラメータの leaf は、`<2>2` の対応の下で同じ集合である。
    BY <ref id=9d74736/>, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=ff5985d/>, <2>1, <2>1a, <2>2

<1>5. (a) が成り立つ。
  BY <ref id=83d98e9/>, <1>1, <1>2, <1>3, <1>4

<1>6. (b) が成り立つ。
  <2>1. (E4) で `Obl(b)` を離れるのは `x` の inhabited な全 boxed leaf の参照である。
    BY <ref id=9d74736/> (終端の `Ret` の行), <ref id=e3436e8/> (E4)
  <2>2. CASE `b` を (E3) が作った場合。
    `Obl(a)` が得るのは `App` の結果の各 boxed leaf につき 1 つである (D10 の生成の `App` の行、
    D24 の (E2) の生成の表の `App` の行)。`apply_lambda` は呼び出し先が返した値をそのまま結果とするので、
    結果の値は `x` の値である。A12 の「`App` については引数とパラメータのほかに、結果の型も一致する」の
    行が `ty(x)` を呼び出し先の返り値の型とし、`RcFunc` の `ret_ty` の欄が終端の `Ret` が返す値の型を
    同じものとするので、`boxed_leaf_paths` の列も inhabited であることも一致する。よって `<2>1` の
    多重集合と等しい。
    BY <ref id=83d98e9/>, <ref id=f06144e/>, <ref id=66c9670/>, <ref id=e3436e8/> (E2), <ref id=e3436e8/> (E4), <2>1, CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/ast.rs: RcFunc
  <2>3. CASE `b` を (E1) か (E7) が作った場合。
    D24 の (E4) と (E7) より、`Obl(b)` を離れる参照はそのまま `E` に入る。行き先が 1 つで、参照は処分も
    生成もされないので、`E` が得る多重集合は `<2>1` のそれと等しい。
    BY <ref id=e3436e8/> (E4), <ref id=e3436e8/> (E7), <2>1
  <2>3a. CASE `b` を適用する `Llvm` の段が作った場合。
    L0a (b) より、`Obl(b)` を離れる参照はそのままその段を実行している活性化の `Obl` に入る。行き先が
    1 つで、参照は処分も生成もされないので、その `Obl` が得る多重集合は `<2>1` のそれと等しい。
    BY <ref id=1b34a16/>, <2>1
  <2>3b. CASE `b` を (F) の解放が作った場合。
    D24 の (E4) は、この場合の `b` が 2 つあることを書き分けたうえで「どちらの返りでも、参照はその解放を
    含む段を実行した活性化の `Obl` に入る」と述べる。1 つ目は `_dtor` の欄の関数を `_value` に適用した
    ものでその返り値は 2 つ目の入力になり、2 つ目は返った `IO` の runner の適用でその返り値は `o` の
    `_value` の欄へ書き込まれる。どちらについても行き先は 1 つであり、参照は処分も生成もされないので、
    その `Obl` が得る多重集合は `<2>1` のそれと等しい。
    BY <ref id=e3436e8/> (E4), <ref id=e3436e8/> (活性化の林), <2>1,
       CODE src/generator.rs: Generator::build_run_destructor
  <2>4. QED
    D24 の活性化の林の段落は「(E1) が作る活性化を**根**、(E3) と (E7)、(E2) のうちオペランドを適用する
    `Llvm` の段、および (F) の解放が `Destructor` について作る段が作る活性化を、それを作った活性化の
    **子**と呼ぶ」と述べ、続けて「**活性化を作る段はこの 5 種で尽きる。**」と述べる。`<2>2`、`<2>3`、
    `<2>3a`、`<2>3b` がその 5 種を尽くす。
    BY <ref id=e3436e8/> (活性化の林), <2>2, <2>3, <2>3a, <2>3b

<1>7. QED
  BY <1>5, <1>6

**DEF 段の素動作と段内の点**
**素動作**と**段内の点**は D24 が定める。D24 は、1 つの段を不可分な素動作の有限列 -- 参照の受け渡し、
生成、割り当て、処分、解放、グローバル化の 6 種 -- へ分解し、その列のどの切れ目が段内の点であるか、
段と段のあいだの時点がその列のどこに当たるか、(F) の解放が段の中で始めた活性化の木の節点が行う動作も
その列の元であること、束ねてよい切れ目が処分とそれが起こす解放のあいだ・素動作とそれに付随する
書き込みのあいだの 2 つに限られること、そして**読みの直前の点では勘定が直前の段内の点のものであること** --
「**その点と直前の段内の点のあいだに素動作は 1 つも無いので、`H` も `Obl` も `held` (D34) も動かず、
解放も起きない。**」-- を定める。**点集合を定めるのは D24 だけであり、この文書はそれを
引く。**以下の「素動作」「段内の点」はすべて D24 のこれである。

**DEF 段内の点で閉じている**
D11a は、時点 `τ` が**解放について閉じている**ことを
「`τ` において `H(O) ≥ 1` である各計数下オブジェクト (D26) `O` が、`τ` において解放されていない
(D24 の (F)) こと」と定め、**`τ` まで閉じている**ことを「`τ` 以前の各時点が解放について閉じていること」と
定める。この文書はその 2 つの語を段内の点についても読む -- 段内の点 `p` が**解放について閉じている**とは、
`p` において `H(O) ≥ 1` である各計数下オブジェクト `O` が `p` において解放されていないことをいい、
**`p` まで閉じている**とは、`p` 以前の各段内の点が解放について閉じていることをいう。
**この読みが要るのは、D11 の (S-c) が「その読み・その触れる動作の直前の点で」の粒度で条件を課すから
である。**

### L2 (到達できるオブジェクトは解放されていない) <!--#881a063-->

**言明**。解放について閉じている (D11a) 段内の点について、次の 2 つが成り立つ。

- **(a)** 次の 3 つはいずれもその点で生きている (D25)。
  - **(a-1)** `H(o) ≥ 1` である計数下オブジェクト (D26) `o`。
  - **(a-2)** その点で**記憶域が在って** (D7) 解放されていないオブジェクト。
  - **(a-3)** 計数下オブジェクト `o` であって、その点で `o` への未処分の参照を保持している値の
    inhabited な boxed leaf が `o` を指すもの (A5)。
- **(b)** オブジェクト `o'` がその点で生きており、`o'` から `o''` へ到達できる (D25) ならば、`o''` も
  解放されていない。`o'` と `o''` は計数下でもグローバル状態でもよい。

**(a-1) と (a-3) を別に置くのは、読み手が持つ仮定が 2 通りだからである。**D7 は記憶域が在ることを
「`o` を割り当てた素動作 (D24) より後、`o` の記憶域を返す素動作 ((F)) より前」と定め、D8 は `H(o)` を
未処分参照の総数と定めるだけなので、**参照の存在から割り当て済みであることを出す節はそのどちらにも
無い。**`<1>0` がその渡りを引き受ける。**(a-3) が「その点で参照を保持している値」に限るのは、A5 が
値の保持する参照について述べるものだからである** -- `Release(v, π)` の後の `v` の値の leaf は同じ
オブジェクトを指したままだが、その参照はもう保持されていない。(a-3) を読む段は、その節点がまだその値を
手放していないことを言う要がある (A26 の第 1 節 -- 記憶域から読む動作はその節点が行うどの参照の手放し
よりも前に起きる)。

**(b) の仮定を「生きている」に取るのは、D25 の 2 つ目の持ち手がその形だからである。**その持ち手は生きて
いるオブジェクトであり、D25 はそれを、割り当てた素動作より後で解放する素動作より前であることと定める。
**実行の最初の時点に環境が持ち込むオブジェクトもこれに入る** -- D25 は「**実行の最初の時点に環境が
持ち込むオブジェクトは、その時点から生きている**」と述べる。(a) は、参照だけを手に持つ読み手にその仮定を
渡す段である。

<1>0. 未処分の参照が計数下オブジェクト `o` を指す点では、`o` を割り当てた素動作は走っているか、`o` は
      環境が実行の最初の時点に持ち込んだオブジェクトである。
  **この言明は段内の点についてのものなので、参照を作る動作は段の境界の網羅では尽きない。**D24 は
  「**参照を作る動作を数え上げる段は、この 2 つの形に (E9) を足して 3 つを見る。**」と述べる。3 つとは
  段の中で相殺する retain、相殺しない retain、そして (E9) の retain であり、これに段の境界についての
  網羅 -- D24 の「**実行の最初の時点に在る参照は、環境が持ってきたものだけである。**参照を作るのは
  D10 の生成の表、(F) の retain、そして `InlineLLVMBoxedFromRetainedPtrIOS` が環境から受け取る行の
  3 か所だけであり、どれも段の中で起きる」-- を合わせたものを、最初の時点とともに順に見る。
  **D10 の生成の表**の各行は、この段が割り当てたオブジェクトへの参照を作るか、**既に在る**オブジェクトへの
  参照を作るかのどちらかである -- D24 の (E2) の `H` の表が、`Closure` の結果と単一の `Fresh` を宣言する
  `Llvm` の結果 leaf を「この段が新しく割り当てるオブジェクト」、単一の `Unknown` を「既存のオブジェクト」、
  boxed 容器の `Destructure` の名前付きフィールドと boxed union の変位アームの payload を読み出し、
  `App` の結果を呼び出し先から渡る参照、`Retain` をその変数が既に指すオブジェクトへの +1 と書き分ける。
  前者はその割り当ての素動作の後であり、後者のオブジェクトは既に在る。
  **(F) の retain** の対象は、解放されるオブジェクトが `_dtor` の欄に持つ関数のオブジェクトであり、その欄に
  在るので既に在る。
  **`InlineLLVMBoxedFromRetainedPtrIOS` の行**が渡すのは環境が持っていた参照であり、A17 の (i-d) より
  環境が持つ boxed な値はこのプログラムが作って番地を渡したものである。
  **段の中で相殺する retain**が名指すのはオペランドの値である -- D24 は
  「`InlineLLVMWithRetainedFunctionBody` はオペランドを retain し、適用の後に release する」と述べる。
  オペランドの値が指すオブジェクトは既に在る。**相殺しない retain**が名指すのは原本の側の値である --
  D24 は「複製を作る腕が複製の欄へ retain する形である」と述べる。原本のオブジェクトも既に在る。
  **(E9) の retain** については A17 (ii-c) が「**環境がその番地を呼ぶのは、その番地が指すオブジェクトへの
  参照を自分が持っている点でだけである。**」と述べ、(i-d) より環境が持つ boxed な値はこのプログラムが
  作って番地を渡したものである。
  **最初の時点に在る参照**については A17 の (i-c) が、環境が持つか環境が持ち込んだオブジェクトの leaf が
  持つと述べ、D25 が「**実行の最初の時点に環境が持ち込むオブジェクトは、その時点から生きている**」と
  述べる。
  BY <ref id=c9e4cca/>, <ref id=f06144e/>, <ref id=e3436e8/>, <ref id=e3436e8/> (E2), <ref id=e3436e8/> (E9), <ref id=e3436e8/> (F), <ref id=0b850c9/>, DEF 段の素動作と段内の点

<1>1. (a) が成り立つ。
  <2>1. (a-1) が成り立つ。
    D8 より `o` を指す未処分の参照が在るので、`<1>0` より `o` を割り当てた素動作は走っているか、`o` は
    最初の時点から生きている。この点は解放について閉じている (D11a) ので、`H(o) ≥ 1` である `o` は
    解放されていない -- すなわち `o` を解放する素動作は走っていない。D25 は生きていることを、割り当てた
    素動作より後・解放する素動作より前と定めるので、`o` はこの点で生きている。
    BY <ref id=ec8d1a0/>, <ref id=859cf84/>, <ref id=0b850c9/>, <ref id=88a06de/>, <1>0
  <2>2. (a-2) が成り立つ。
    D7 は記憶域が在ることを「`o` を割り当てた素動作 (D24) より後、`o` の記憶域を返す素動作 ((F)) より
    前」と定め、D25 は生きていることを「それを割り当てた**素動作** (D24) より後、それを解放する素動作
    ((F)) より前」と定める。第 1 の条件は同じであり、第 2 の条件はこの言明の仮定が与える。
    BY <ref id=56c2068/>, <ref id=0b850c9/>
  <2>3. (a-3) が成り立つ。
    A5 より、値が保持する参照はその型の `boxed_leaf_paths` が列挙する leaf のうち、inhabited であって
    計数下のオブジェクトを指すものにちょうど 1 つずつある。よってその leaf が保持する `o` への参照は
    1 つであり、この場合の仮定よりそれは未処分である。D8 より `H(o) ≥ 1` であり、`<2>1` が当たる。
    BY <ref id=4f63121/>, <ref id=ec8d1a0/>, <ref id=88a06de/>, <2>1
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>1a. `o'` がその点で生きており、`o'` が `o` への参照を持つならば、`o` もその点で生きている。とくに
       `o` は解放されていない。
  `o` がグローバル状態ならば、D26 より `o` は割り当てられた後に `mark_global` が印を付けたものであり、
  A8 より解放されない。よって D25 の意味でこの点で生きている。`o` が計数下ならば次のとおりである。
  `o'` は生きているオブジェクトなので D25 の 2 つ目の持ち手であり (D26 は D8 の参照を計数下のオブジェクト
  への参照に限るが、持ち手の側には制限を置かないので `o'` の状態を問わない)、A5 よりその持ち手の単位の
  うち `o` を指すものが `o` への参照をちょうど 1 つ持つ -- 単位は、`o'` が `#ArrayStorage` のオブジェクト
  でなければ `o'` が保持する値の inhabited な boxed leaf であり、`o'` が `#ArrayStorage` のオブジェクトで
  あるときは、A5 のとおりその記憶域の各スロットであって、生成の素動作がそこへ書いた時点から要素の型の
  inhabited な boxed leaf を 1 組ずつ持つ (`size` はこの数え上げを切らない)。**この各スロットが持つ leaf も
  (a-3) の主語である「値の inhabited な boxed leaf」に当たる** -- A5 の配列の記憶域の例外の節は、その
  leaf をそのスロットに格納された要素の値の leaf として数える。D25 が挙げるのは未処分の参照の持ち手なので、
  その参照は未処分である。よって (a-3) の場合の仮定が揃い、`<1>1` より `o` はその点で生きており、とくに
  解放されていない。
  BY <ref id=4f63121/>, <ref id=b6673ca/>, <ref id=ec8d1a0/>, <ref id=0b850c9/>, <ref id=88a06de/>, <1>1

<1>2. (b) が成り立つ。
  到達の道の長さについての帰納。長さ 0 のとき `o'' = o'` であり、生きているものは解放されていない
  (D25)。長さ `n+1` の道は、`o'` が持つ参照が指すオブジェクト `o1` への 1 歩と、`o1` から `o''` への
  長さ `n` の道に分かれる。`<1>1a` より `o1` は生きているので、帰納法の仮定より `o''` も解放されて
  いない。
  BY <ref id=0b850c9/>, <1>1a

<1>3. QED
  BY <1>0, <1>1, <1>2

### L2b (段の中でも、解放されたオブジェクトへ参照は作られない) <!--#ba46997-->

**言明**。**(H1)** 時点 `t` **まで**閉じている (D11a、DEF 段内の点で閉じている) とし、`t` の直後の段を
`S` とする。**(H2)** `S` を持つ活性化の本体と、`S` の中で (F) の解放が始めた活性化の木 (D24 の (F)) の
各活性化の本体は D11 を満たすとする。このとき次の 2 つが成り立つ。

- **(a)** `S` のどの段内の点も解放について閉じている。すなわち `S` の最後の点まで閉じている。
- **(b)** `S` が作る各参照について、その参照が指すオブジェクトは、その参照を作る動作の直前の点で解放されて
  いない。したがって `t` においても解放されていない -- D25 は「生きている」を、割り当てた素動作より後、
  解放する素動作より前と定めるので、解放されていることは解放の素動作より後のすべての点で成り立ち、`t` で
  解放されていたオブジェクトはその後のどの段内の点でも解放されている。

**(b) が「その参照を作る動作の直前の点で」と言うのは 2 つの理由による。**1 つは、`S` の中で (F) の解放が
始めた活性化の木の節点も参照を作ることである -- その節点は `t` の直後ではなく段の途中で走るので、`t` を
主語にした言明では D11 の (S-c) が与えるものと噛み合わない。もう 1 つは、(S-c) の結論そのものが「その読み・
その触れる動作の直前の点で」の形だからである。

**接頭の条件を仮定に置くのは、(S-c) がそれを条件に取るからである。**(S-c) は、その活性化がその時点まで
閉じているときにだけ解放後の読みを禁じる。`t` までの点の閉じていることは L3 の帰納が供給し、`S` の中の
点については `<1>9` が 1 点ずつ延ばす。

<1>0. `S` の最初の段内の点は `t` であり、`t` まで閉じている。
  D24 より、段と段のあいだの時点はその段の最初の段内の点である。(H1) が `t` まで閉じていることを与える。
  BY (H1), <ref id=e3436e8/>, DEF 段の素動作と段内の点

<1>1. `S` の中で参照を作る素動作は、次の 3 種で尽きる。**(K-i)** D24 の (E2) の `H` の表が挙げる
      生成の 7 行と、参照を処分する段の中で起きる (F) の解放が `Destructor` のオブジェクトについて
      行う retain。**(K-ii)** 表に行を持たない、段の中の retain であって、
      `InlineLLVMWithRetainedFunctionBody` が出すものか、複製・割り当てたオブジェクトの欄へ書く
      4 か所 (`clone_struct`、`clone_union`、`clone_array_range`、`append_value_into_array_buf`) が
      出すもの。**(K-iii)** 環境が (E9) で行う retain -- `get_funptr_retain` が渡した番地を環境が
      呼ぶことで作られる参照である (A17 (ii-c))。
      `S` の中で (F) の解放が始めた活性化の木 (D24 の (F)) の節点も、その本体について (K-i) と (K-ii) の
      2 種によって参照を作る -- (K-iii) は環境自身の段であって活性化の節点ではないので、この木の節点は
      これを持たない。
  **(K-i) の側。**D24 の (E2) の `H` の表が 7 行を挙げる -- `Retain` の行、`Llvm` の行、
  `InlineLLVMBoxedFromRetainedPtrIOS` の行、boxed 容器の `Destructure` の名前付きフィールドの行、
  boxed union の変位アームの payload の行、`Closure` の行、`App` の行である。
  (E1)、(E3)、(E4) は持ち手を移すだけ、(E5) は印を付けるだけ、(E7) は `Obl` が空の活性化を作るだけで
  ある。(E6) は「この段は参照を作らず、渡さず、処分しない」うえその後に段は無く、(E8) も「この段は参照を
  作らず、渡さず、処分しない」。**(E9) のうち release は環境の持ち分から参照を 1 つ処分するだけであり
  (A17 (ii-c))、retain が作る参照が (K-iii) である。**段の中で起きる (F) の解放については、D24 が
  「**この段は参照も作る。**
  `_dtor` の欄の関数に適用の分の参照を与える retain がそれである (`CODE src/generator.rs:
  Generator::build_run_destructor` -- `build_retain(dtor, one, ...)` が `apply_lambda` の前に立つ)。
  よって **新しい参照を作るのは (E2) だけではない**」と述べる。同じ (F) が `Destructor` について作る
  2 つの活性化と、D24 の (F) より同じ段の一部であるその子孫の活性化 -- 合わせてその解放が始めた
  活性化の木 -- の節点も、その本体についての (E2) の生成の 7 行によって参照を作る。
  **割り当ての素動作が作る参照もこの 7 行に入る** -- 表の `Closure` の行は「この段が新しく割り当てる
  capture object、`H` = 1」、`Llvm` の行の単一の `Fresh` は「この段が新しく割り当てるオブジェクトで
  `H` = 1」と述べる。

  **(K-ii) の側。**D24 は上の網羅に続けて「**この網羅は段の境界についてである。** 1 つの段の生成コードは、
  この表に行を持たない素動作を段の中で出しうる。**素動作の粒度で勘定する段は、その op の `generate` が
  出す retain と release を読む。**」と述べ、その在りかについて「**在りかは述語で決める** --
  `Generator::retain`・`Generator::build_retain`・`Generator::release` の呼び出しを出す生成コードの
  全体であり、一覧で書くと op が 1 つ増えるたびに古くなる。」と述べ、その綴りについて「**述語は名前の
  綴りでなく、呼ばれる項目で書く。**」と述べる。**この段は段内の点で勘定するので、その述語で走査する。**
  `src/` で `Generator::retain`・`Generator::build_retain`・`Generator::retain_nonnull_boxed` を呼ぶ式は
  21 か所ある (受け手の綴りが `gc` のものも `self` のものも数え、`Vec`・`Map` の `retain` を除く)。
  後ろの 1 つは D24 の挙げる 3 つに無いが、`build_retain` の boxed の枝がそれを呼ぶので、含めた方が
  広い。次の 7 群がその 21 か所を尽くす。

  - **第 1 群 (表の 7 行を出すもの)。7 か所。**`RcExpr::Retain` の 2 か所 (`eval_rc_expr_inner` の
    `skip_null_check` の 2 枝)、boxed union の変位アームの payload の retain (`eval_rc_match`)、
    boxed 容器の `Destructure` (`get_struct_fields` の boxed の枝)、boxed union の payload の読み出し
    (`get_union_value` の boxed の枝)、capture の射影 (`build_capture_project`)、配列の要素の読み出し
    (`read_from_array_buf`) である。後ろの 3 つは `Llvm` の行であり、A3 の宣言が単一の `Unknown` を置く
    読み出しがこれである。**これは (K-i) である。**
  - **第 2 群 (retain の素動作の実装)。6 か所。**`Generator::retain` が `build_retain` を呼び、
    `build_retain` が boxed の枝で `retain_nonnull_boxed` を、unbox の枝で各フィールドへ降りて
    `retain` / `build_retain` を呼び、unbox union については `retain_release_mark_union` が活性変位へ
    降りる (`generator.rs` に 4 か所、`object.rs` に 2 か所)。**これは他の群の retain を leaf ごとの
    増加として実装するものであって、それ自身が別の参照を作るのではない** -- A4 が、`Retain(v, π)` を
    `π` の下の inhabited な各 boxed leaf の参照カウントの +1 として実装すると述べるのがこの形である。
    **`clone_union` が `retain_union` を通ってここへ着く道だけは別である** -- そこが出す retain は
    第 6 群と同じ形であり、(K-ii) である。
  - **第 3 群 (グローバルの読み)。1 か所。**`get_scoped_obj` の `retain_on_read` の行である。
    `add_global_object` はその旗を `!ty.is_box(..)` のときだけ立てるので、これが走るのは unbox な
    グローバルを読むときである。A8 より、グローバル値が到達するオブジェクトへの `Retain` は参照カウントを
    変えず、D26 よりそれらを指す leaf は D8 の意味の参照を持たない。**よってこの群は参照を作らない。**
  - **第 4 群 ((F) の解放が `Destructor` について行う retain)。1 か所。**`build_run_destructor` の
    `build_retain(dtor, one, ..)` である。**これは (K-i) である。**
  - **第 5 群 (段の中で相殺するもの)。1 か所。**`InlineLLVMWithRetainedFunctionBody::generate` である。
    D24 が「**段の中で相殺するもの。** `InlineLLVMWithRetainedFunctionBody` はオペランドを retain し、
    適用の後に release する。段の出口では相殺するので表には現れないが、段内の点では見える。」と述べる
    形である。**これは (K-ii) である。**
  - **第 6 群 (相殺しないもの)。4 か所。**D24 が「**相殺しないもの。** 複製を作る腕が複製の欄へ retain
    する形である (`make_struct_union_unique` の共有の腕、`clone_array_buf`)。**原本が共有で生き残れば
    相殺しない**ので、段の境界でも `H` を上げる。**その参照の持ち手は、その生成コードが書き込む
    オブジェクトの持ち手の単位である** (D25 の 2 番目)。」と述べる形であり、`clone_struct`、
    `clone_array_range` (`clone_array_buf` が呼ぶ)、`initialize_array_buf_by_value`、
    `append_value_into_array_buf` の 4 か所である。**`clone_union` はこの 4 か所に数えない** -- それは
    `retain_union` を通って第 2 群の `retain_release_mark_union` へ着くので `gc.retain(` の出現を持たないが、
    同じ形であり (K-ii) である。**4 か所のうち `initialize_array_buf_by_value` は `src/` に呼び出し元を
    持たない**ので、どの段もその retain を出さない。残る 3 か所と `clone_union` が (K-ii) である。
  - **第 7 群 ((E9) の retain が実行するもの)。1 か所。**
    `InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody` の `generate` が定義する補助関数
    `retain#<型>` の本体である。この op 自身の段が出すのは
    その関数の**定義**とその番地であって、retain ではない。**この本体が走るのは、環境が (E9) の段で
    その番地を呼ぶときである** -- A17 (ii-c) が「**その呼び出しは D24 の段であり、その番地が指す
    オブジェクトへの参照を、retain なら 1 つ作って環境の持ち分に足し、release なら環境の持ち分から
    1 つ処分する。**」と述べる。**これは (K-iii) である。**
  BY <ref id=e11772a/>, <ref id=3f1bb47/>, <ref id=b6673ca/>, <ref id=c9e4cca/>, <ref id=ec8d1a0/>, <ref id=e3436e8/>, <ref id=e3436e8/> (E2), <ref id=e3436e8/> (E9), <ref id=e3436e8/> (F), <ref id=88a06de/>,
     CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner, Generator::eval_rc_match,
     CODE src/object.rs: ObjectFieldType::get_struct_fields, ObjectFieldType::get_union_value,
     ObjectFieldType::read_from_array_buf, ObjectFieldType::retain_release_mark_union,
     ObjectFieldType::retain_union, ObjectFieldType::clone_struct, ObjectFieldType::clone_union,
     ObjectFieldType::clone_array_buf, ObjectFieldType::initialize_array_buf_by_value,
     ObjectFieldType::append_value_into_array_buf,
     CODE src/generator.rs: Generator::build_capture_project, Generator::retain,
     Generator::build_retain, Generator::retain_nonnull_boxed, Generator::get_scoped_obj,
     Generator::add_global_object, Generator::build_run_destructor,
     CODE src/fixstd/builtin.rs: InlineLLVMWithRetainedFunctionBody,
     InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody

<1>1a. 参照を作る素動作の直前の点を `p` とし、`p` まで閉じているとする。その動作を行う節点が D7 の読む
       構文であり、その動作が作る参照が指すオブジェクト `o` が、その節点が**記憶域から読んだ**オブジェクト
       であるか、そこから到達できる (D25) オブジェクトであるとき、`o` は `p` で解放されていない。
  その節点はある活性化 `a'` の本体 `B(a')` の節点である。`<1>1` より `a'` は `S` を持つ活性化か、`S` の
  中で (F) の解放が始めた活性化の木の元であり、(H2) よりどちらの本体も D11 を満たす。`a'` が辿った節点の列は
  `B(a')` の実行路 (D3) である (D21, D23)。**その活性化に D11 を当ててよいことは
  D21 が言う** -- 「**実行 (D24) が作る活性化がこの制限を満たすことは P28 (b) が示す**」、そして
  「**D11 と D12 は、この意味のすべての活性化について条件を課す。**」`a'` は `p` まで閉じている --
  この段の仮定の「`p` まで閉じている」は `p` 以前の各段内の点についての条件であり
  (DEF 段内の点で閉じている)、`a'` の時点も段内の点であって (D24) `p` 以前に在るので、D11a の
  「`τ` まで閉じている」がそこから出る。
  節点が記憶域から読むオブジェクトは、D7 の読む構文がその位置で読みうるオブジェクト -- 名指された値の
  inhabited な各 boxed leaf が指すオブジェクト -- のうちに在る (D7、D32 の (読み-1))。よって (S-c) が
  当たり、読んだ各オブジェクトは**その読みの直前の点**で解放されていない。
  **読みの直前の点は `p` と同じ点ではない。** `p` は参照を作る素動作の直前の点であり、その 2 つのあいだに
  この節点の手放しが挟まりうる。A26 の第 2 節が両者を結ぶ -- この節点が作る参照の指すオブジェクトが、
  この節点が記憶域から読んだオブジェクトかそこから到達できる (D25) オブジェクトであるとき、読みの瞬間から
  参照を作る瞬間まで解放されない。`p` はその区間の中の点である。
  **読みの直前の点は段内の点とは限らない** -- 読みは 6 種の素動作のどれでもないからである。D24 が
  「**読みの直前の点では、勘定は直前の段内の点のものである。**」と述べ、「**その点と直前の段内の点の
  あいだに素動作は 1 つも無いので、`H` も `Obl` も `held` (D34) も動かず、解放も起きない。**」と続けるので、
  その直前の段内の点を `q` とすると、読んだ各オブジェクトは `q` でも解放されていない。`q` は `p` 以前なので
  解放について閉じている (この段の仮定)。
  **読んだオブジェクトが `q` で生きていることは L2 (a-3) が与える** -- D7 の読む構文が読みうるのは
  名指された値の inhabited な各 boxed leaf が指すオブジェクトなので、読んだオブジェクトはその形で
  指されており、A26 の第 1 節が「D7 の読む構文がオブジェクトの**記憶域から読む**動作は、その節点が行う
  どの参照の**手放し**よりも前に起きる」と述べるので、その読みの点でこの節点はまだその値を手放して
  おらず、A5 よりその leaf が保持する参照は未処分である。`q` と読みの直前の点のあいだに素動作は無いので
  (D24)、その条件は `q` でも成り立つ。計数下でないオブジェクトは A8 より解放されない。
  L2 (b) がそこから到達できるオブジェクトへ広げる。`q` から読みの
  直前の点までのあいだに解放は起きないので、その結論はそのまま読みの直前の点へ移る。
  **この節が言うのは、この節点が作る参照が指す先についてだけである。**この節点が読みうる他のオブジェクトに
  ついては何も言わない。A26 の第 2 節が主語を読んで得たオブジェクトに限るのと同じ理由であり、payload を
  retain し union を release した**後で** `create_obj` を呼ぶ `union_mod` の生成コードがその形である
  (A26) -- union はこの節点の読みうるオブジェクトでありながら、`p` で解放されていることがある。
  BY (H2), <ref id=4f63121/>, <ref id=b6673ca/>, <ref id=fd95f12/>, <ref id=ca36627/>, <ref id=56c2068/>, <ref id=95427eb/> (S-c), <ref id=859cf84/>, <ref id=c232680/>, <ref id=ff5985d/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=1b00a9e/>, <ref id=881a063/>,
     DEF 段の素動作と段内の点, DEF 段内の点で閉じている, <1>1

<1>1b. 参照を作る素動作の直前の点を `p` とし、`p` まで閉じているとする。その動作を行う節点が
       `Retain(v, π, s, k)` であるとき、`π` の下の inhabited な各 leaf が指すオブジェクトは `p` で
       解放されていない。
  D7 より `Retain(v, π)` が**触れる**のはちょうどそれらのオブジェクトである。`<1>1` よりその節点は `S` を
  持つ活性化 `a'` か (F) の解放が始めた活性化の木の元の本体の節点であり、(H2) よりその本体は D11 を
  満たす。その活性化が辿った節点の列はその本体の実行路 (D3) である (D21, D23)。**その活性化に D11 を当ててよいことは
  D21 が言う** -- 「**実行 (D24) が作る活性化がこの制限を満たすことは P28 (b) が示す**」、そして
  「**D11 と D12 は、この意味のすべての活性化について条件を課す。**」**その活性化が `p` まで
  閉じていることは、この段の仮定から出る** -- 仮定の「`p` まで閉じている」は `p` 以前の各段内の点に
  ついての条件であり (DEF 段内の点で閉じている)、`a'` の時点も段内の点であって (D24) `p` 以前に在るので、
  D11a の「`τ` まで閉じている」がそこから出る。よって (S-c) が当たり、(S-c) は、読む構文が読みうる
  オブジェクトに加えて `Retain(v, π)` と `Release(v, π)`
  が触れる各オブジェクトについても、**その触れる動作の直前の点**で解放されていないことを言う。
  **`π` の下に inhabited な leaf が 2 つ以上あるとき、触れる動作は leaf ごとに在り、その直前の点は
  leaf ごとに別である。**この節点が触れる leaf を触れる順に `λ_1, …, λ_k` とし、`λ_i` に触れる直前の点を
  `p_{i-1}` と書く。`p` は 1 つの leaf の生成の直前の点なので、ある `j` について `p = p_{j-1}` である。
  **この節点が行う動作は、D10 の `Retain` の行と D24 の (E2) の `H` の表の `Retain` の行が挙げる leaf ごとの
  生成だけである。**生成は `H` を上げるので、D24 の (F) が解放を起こす条件 -- ある段が参照を処分して `H` が
  0 になること -- をどのオブジェクトについても満たさない。よって `p_0` から `p_k` までのあいだに解放は
  1 つも起きない。
  `i` についての帰納で、各 `i` について「`p_i` まで閉じている」と「`obj(v, λ_i)` は `p_{i-1}` で解放されて
  いない」を示す。この段の仮定より `p = p_{j-1}` まで閉じており、`p_0` は `p` 以前なので `p_0` まで
  閉じている。
  `p_{i-1}` まで閉じているとすると、(S-c) の接頭条件が揃うので `obj(v, λ_i)` は `p_{i-1}` で解放されて
  いない。`p_{i-1}` から `p_i` までのあいだの素動作は `λ_i` の生成 1 つだけであり、解放は起きない。よって
  `p_i` で `H(O) ≥ 1` である計数下オブジェクト `O` は、`p_{i-1}` で `H(O) ≥ 1` であったもの --
  `p_{i-1}` で解放されておらず、区間に解放が無いので `p_i` でも解放されていない -- か、`obj(v, λ_i)` で
  ある。後者もいま示したとおり解放されていない。したがって `p_i` は解放について閉じており、`p_i` まで
  閉じている。
  区間 `[p_0, p_k]` に解放が 1 つも無いので、各 `obj(v, λ_i)` は `p` でも解放されていない。
  BY (H2), <ref id=ca36627/>, <ref id=56c2068/>, <ref id=f06144e/>, <ref id=95427eb/> (S-c), <ref id=859cf84/>, <ref id=c232680/>, <ref id=ff5985d/>, <ref id=e3436e8/>, <ref id=e3436e8/> (E2), <ref id=e3436e8/> (F), DEF 段の素動作と段内の点,
     DEF 段内の点で閉じている, <1>1

<1>1c. 割り当ての素動作が作るオブジェクトは、その動作の直前の点でも直後の点でも解放されていない。
  D25 より、オブジェクトが生きているのはそれを割り当てた**素動作**より後、それを解放する素動作より前で
  ある。よってその動作の直前の点にそのオブジェクトは在らない。**直後の点でまだ解放が起きていないのは、
  D24 の (F) が解放を起こす条件が「ある段が参照を処分して計数下のオブジェクト (D26) `o` の `H(o)` が
  0 になったとき」だからである** -- 割り当ての素動作は参照の処分ではないので、その動作は (F) を
  発火させない。割り当てられた直後のオブジェクトの `H` が 0 であること自体は (F) の条件ではない。
  BY <ref id=e3436e8/> (F), <ref id=0b850c9/>, DEF 段の素動作と段内の点

<1>1e. 参照を作る素動作の直前の点を `p` とし、`p` まで閉じているとする。その動作を行う節点 `n` が
       `Let(y, Llvm(gen, args), k)` であり、その動作が作る参照が指すオブジェクト `o` を `n` の第 `i`
       オペランドの inhabited な boxed leaf `λ` が指し、D9 の `Llvm` の行がその leaf を消費とし、かつ
       `n` の生成コードが `p` より前にその leaf の参照を手放す素動作を出さないならば、`o` は `p` で
       解放されていない。
  `o` がグローバル状態 (D26) ならば A8 より解放されない。計数下ならば次のとおりである。
  `n` を実行する活性化を `a'` とすると、`<1>1` より `a'` は `S` を持つ活性化か、`S` の中で (F) の解放が
  始めた活性化の木の元であり、(H2) よりその本体は D11 を満たす。`a'` が辿った節点の列はその本体の
  実行路 (D3) である (D21, D23)。**その活性化に D11 を当ててよいことは
  D21 が言う** -- 「**実行 (D24) が作る活性化がこの制限を満たすことは P28 (b) が示す**」、そして
  「**D11 と D12 は、この意味のすべての活性化について条件を課す。**」
  D10 の消費の行より、D9 の消費は `Obl(a')` から参照を 1 つ取り除く。D11 の (S-a) は「`Obl` から参照を
  取り除くすべての操作について、取り除かれる参照はその時点の `Obl` に入っている」と述べるので、その
  取り除きの時点で `obj(args[i], λ)` への参照は `Obl(a')` に在る。**その取り除きは `p` より後である** --
  この段の仮定より、`n` の生成コードは `p` より前にその leaf の参照を手放さない。**`p` からその
  取り除きまでのあいだにその参照が `Obl(a')` を離れることも無い** -- 同じ仮定がそれを言う。よって `p` で
  その参照は `Obl(a')` に在る。D25 が挙げるのは未処分の参照の持ち手なので、その参照は未処分であり、
  D8 より `H(o) ≥ 1` である。`p` は解放について閉じている (この段の仮定) ので、`o` は `p` で解放されて
  いない。
  BY (H2), <ref id=b6673ca/>, <ref id=ca36627/>, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=95427eb/> (S-a), <ref id=859cf84/>, <ref id=c232680/>, <ref id=ff5985d/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=88a06de/>, <1>1

<1>1d. DEFINE `p` == 参照を作る素動作の直前の段内の点。以下の各場合は、`p` まで閉じていることを
       仮定して、その素動作が作る参照が指すオブジェクトが `p` で解放されていないことを示す。
  D24 が素動作の直前・直後の点を段内の点と定めるので、参照を作る各素動作の直前には段内の点が在り、
  `p` は定まる。
  BY <ref id=e3436e8/>, DEF 段の素動作と段内の点

<1>2. CASE `Retain(v, π, s, k)` の行。
  D10 の `Retain` の行より、この動作が作る参照が指すのは `π` の下の inhabited な各 leaf `λ` の
  `obj(v, λ)` である。`<1>1b` がそれらについて言明を与える。
  BY <ref id=f06144e/>, <ref id=e3436e8/> (E2), <1>1b

<1>3. CASE `Closure(f, caps)` の結果の行。
  この動作が作る参照が指すのは、この段が新しく割り当てる capture object である (D24 (E2) の `H` の表)。
  `<1>1c` がそれについて言明を与える。`caps` が空のときは capture ポインタが null であり、オブジェクトも
  参照も無いので、言明は空虚に成り立つ。
  BY <ref id=e3436e8/> (E2), <1>1c

<1>4. CASE `App(callee, args)` の結果の行。
  この行は参照を作らない (D24 (E2) の `H` の表の `App` の行)。よって言明は空虚に成り立つ。
  BY <ref id=e3436e8/> (E2)

<1>5. CASE `Llvm` の行、および `InlineLLVMBoxedFromRetainedPtrIOS` の行。
  <2>0. op が `applies_a_function_operand` を宣言し、その適用が作った活性化 `b` が返した参照が結果の
        leaf に置かれるとき、この段はその leaf について新しい参照を作らない。
    L0a (b) より、その参照は `b` の中で作られて (E4) でこの段を実行している活性化の `Obl` に入るもの
    であり、この段が新しく作るのではない。D24 の (E4) も「このとき (E2) の生成の表の `Llvm` の行は
    その leaf について読まない」と述べる。
    BY <ref id=e3436e8/> (E4), <ref id=1b34a16/>
  <2>1. CASE 宣言が単一の `Fresh`。
    <3>1. その leaf に置かれる参照が指すオブジェクトは、この動作が新しく割り当てたオブジェクトか、この op の
          オペランドが指すオブジェクトである。
      A3 の `Fresh` の行は「新しく割り当てたオブジェクトへの新しい参照」を述べ、続けて
      「**実行時に参照カウントで分岐する op の `Fresh` の行は、オブジェクトの同一性については字義どおりでは
      ない。** そうした op の一意の腕はオペランドのオブジェクトをそのまま返す
      (`CODE src/fixstd/builtin.rs: make_struct_union_unique`, `force_unique_or_assert_with_hole`)」と
      例外を述べる。宣言が言うのは
      参照が新しいことだけであり、その参照が指すオブジェクトはこの 2 つのどちらかである。
      BY <ref id=e11772a/>
    <3>1a. **在りかを数え上げる。**実行時に参照カウントで分岐する op はどれも
           `Generator::build_branch_by_is_unique` を呼ぶ生成コードを持つ。`src/` にその呼び出しは
           6 か所ある。2 か所は `make_struct_union_unique` と `make_array_unique_with_hole` 自身の
           定義であり、A3 の名指す 2 つの補助関数はこの 2 か所を経由する道である。2 か所
           (`InlineLLVMIsUniqueFunctionBody`、`InlineLLVMArrayIsStorageUniqueBody`) は一意性の観測点
           (D18) であって `Bool` を返すので `Fresh` を宣言しない。**残る 2 か所は
           `InlineLLVMArrayAppendCapacityUnchecked` と `InlineLLVMArraySetCapacityBoundsUnchecked` の
           呼び出しである。**前者は `src` の消費について分岐するもの
           (D30 の (X2) が名指す形) であり、その `Fresh` 宣言の結果 (`dst`) は
           `array_tail_destination` を経て `force_unique_or_assert` が `make_array_unique_with_hole` へ
           渡す経路の上に在る。**`InlineLLVMArraySetCapacityBoundsUnchecked` の呼び出しだけが、この
           2 つの補助関数を経ない自分自身の一意腕を持つ。**その一意腕は `realloc_array` を呼ぶだけで
           `gc.release` を出さず、共有の腕は `release_replaced_array` で古い記憶域を処分する。
      BY <ref id=c422d87/>, <ref id=081e39f/>, CODE src/generator.rs: Generator::build_branch_by_is_unique,
         CODE src/fixstd/builtin.rs: InlineLLVMArraySetCapacityBoundsUnchecked,
         InlineLLVMArrayAppendCapacityUnchecked, InlineLLVMIsUniqueFunctionBody,
         InlineLLVMArrayIsStorageUniqueBody, make_struct_union_unique, make_array_unique_with_hole,
         array_tail_destination, force_unique_or_assert
    <3>2. QED
      `<3>1` の 2 つの場合を分ける。
      **新しく割り当てたオブジェクトの場合**は `<1>1c` が扱う。
      **オペランドが指すオブジェクトの場合**、どちらの読みを取るかは A3 の「**宣言が決める。**」の節が
      決める -- 「結果の leaf の宣言が単一の `Arg(j, σ)` であるとき、その leaf は移った参照を持つ --
      (E2) の行き先の段落が「消費されたオペランドの参照はそのまま結果の leaf の参照になる」と書くのは
      この場合である。宣言がそれ以外のとき、その leaf は生成の行が言う新しい参照を持つ。**参照カウントで
      分岐する op のように、実行路によって同じ leaf が両方の形を取りうるときも、宣言が単一の `Arg` か
      どうかがどちらの読みかを決める** -- 宣言は実行路に依らないからである。」この場合の宣言は単一の
      `Fresh` なので、その leaf は生成の行が言う**新しい参照**を持つ。すなわちこの場合は言明の量化に
      入り、その参照が指すのは `<3>1` よりオペランドのオブジェクトである。
      **そのオペランドの leaf を D9 の `Llvm` の行は消費とする。**宣言が単一の `Arg` でないので素通しでは
      なく、`borrows_operand(i)` も偽である -- `<3>1a` の 2 か所の補助関数を経由する道では、共有の腕が
      `gc.release(obj, state)` でそのオペランドの参照を処分する (`make_struct_union_unique`。配列に
      ついては `make_array_unique_with_hole` が `release_replaced_array` を呼ぶ)、`<3>1a` の残る 2 か所の
      うち `InlineLLVMArraySetCapacityBoundsUnchecked` では共有の腕が `release_replaced_array` で古い
      記憶域を処分し、`InlineLLVMArrayAppendCapacityUnchecked` の `Fresh` の結果は `<3>1a` より
      `make_array_unique_with_hole` を経由する道の上に在る。**一意の腕はそのオペランドの参照を
      手放さない** -- 補助関数を経由する道では `make_struct_union_unique` が `create_obj` と
      `clone_struct`/`clone_union` と `gc.release` を出すのは共有の腕だけであり、`InlineLLVMArraySetCapacityBoundsUnchecked` の一意腕は `realloc_array`
      を呼ぶだけで `gc.release` を出さない。いずれの道でも 2 つの腕は排他である。A3 は
      「`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの参照を処分しない」と述べるので、
      いずれの道でも `borrows_operand(i)` が偽であることがこの消費を支える。
      よって `<1>1e` が当たる。
      BY <ref id=e11772a/>, <ref id=9d74736/>, <ref id=e3436e8/> (E2), <1>1c, <1>1e, <3>1, <3>1a,
         CODE src/fixstd/builtin.rs: make_struct_union_unique, force_unique_or_assert_with_hole,
         make_array_unique_with_hole, release_replaced_array, InlineLLVMArraySetCapacityBoundsUnchecked,
         CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand
  <2>2. CASE 宣言が単一の `Unknown`。
    <3>0. CASE op が `InlineLLVMBoxedFromRetainedPtrIOS` である。
      A3 の `Unknown` の行は、この op について「オペランドは `Std::Ptr` で boxed leaf を持たないので、
      到達できる元が無い -- そのオブジェクトは C の側から渡された番地が指すものである」と述べ、その leaf に
      ついて行を読まないよう指示する。生成コードは、結果の組を `create_obj` で作り、その第 0 欄に
      オペランド `ios` を、第 1 欄に第 1 オペランド `ptr` の第 0 欄の番地を入れる。結果の型は
      `(IOState, a)` の unbox の組なので `create_obj` は割り当てを行わない
      (`CODE src/fixstd/builtin.rs: InlineLLVMBoxedFromRetainedPtrIOS`, `CODE src/object.rs: create_obj`)。
      **どの番地が渡されるかは A17 (i-b) が決める** -- 「`boxed_to_retained_ptr` が渡した番地について、
      環境はその参照を持ち、`boxed_from_retained_ptr` で Fix の側へ返すまで処分しない」、そして
      「**それ以外の番地を `boxed_from_retained_ptr` に渡す実行は、このモデルの外にある**」。
      その番地が指すオブジェクトを `o` とする。`o` がグローバル状態
      (D26) ならば A8 より解放されない。`o` が計数下ならば、環境が `o` への未処分の参照を 1 つ持つので
      (D25 の 3 つ目の持ち手)、D8 より `H(o) ≥ 1` であり、`p` は解放について閉じているので `o` は `p` で
      解放されていない。
      **この行は D24 の `H` の表の `InlineLLVMBoxedFromRetainedPtrIOS` の行でもある** --
      そこでは `H` は変わらず、環境が持っていた参照が `E` から `Obl(a)` へ渡るので、この段は新しい参照を
      作らない。
      BY <ref id=e11772a/>, <ref id=b6673ca/>, <ref id=c9e4cca/>, <ref id=ec8d1a0/>, <ref id=859cf84/>, <ref id=243ae2c/>, <ref id=e3436e8/> (E2 の `H` の表), <ref id=0b850c9/>, <ref id=88a06de/>,
         CODE src/fixstd/builtin.rs: InlineLLVMBoxedFromRetainedPtrIOS, CODE src/object.rs: create_obj
    <3>0a. CASE op が `applies_a_function_operand` を宣言する。
      A3 の `Unknown` の行は、参照の作られる先を「この op のオペランドの leaf が指すオブジェクトから
      到達できるか、グローバル値が到達する (`CODE src/rc_ir/provenance.rs: LeafOrigin`)」と限定した
      うえで「**この限定が成り立たない op が 2 種ある。**
      オペランドを適用する op (`LLVMGen::applies_a_function_operand`) では、適用した関数の中で新しく
      割り当てられたオブジェクトが結果に出る」と述べる。よってこの op の結果の leaf が指すオブジェクトは、
      オペランドの leaf が指すオブジェクトから到達できるもの、グローバル値が到達するもの、そして
      **適用した関数の中で新しく割り当てられたもの**の 3 つのいずれかである。
      第 1 の場合。A3 の `Unknown` の行はその参照を「既存のオブジェクトへの新しい参照 (retain を伴う
      読み出し)」と述べる。読み出しはオペランドのオブジェクトの記憶域を読む動作であり、
      `Let(x, Llvm(gen, args), k)` は D7 の読む構文で、読まれる値は各オペランドである。参照が指すのが
      オペランドの leaf が指すオブジェクトそのものであるときは、それがこの読み出しが記憶域を読んだ
      オブジェクトであり、A26 の第 2 節の第 1 の主語に当たる。それより深いオブジェクトは第 2 の主語 --
      そこから到達できる (D25) オブジェクト -- に当たる。いずれにせよ `<1>1a` がそれを扱う。
      第 2 の場合。グローバル値が到達するオブジェクトは A8 より解放されない。
      第 3 の場合。その割り当てを行ったのは、この段が作った活性化 `b` か、`b` の子孫の活性化である。
      D24 の活性化の林の段落は、(E1) が作る活性化を根とし、(E3) と (E7)、(E2) のうちオペランドを適用
      する `Llvm` の段、および (F) の解放が `Destructor` について作る段が作る活性化を子と呼んだうえで、
      「**活性化を作る段はこの 5 種で尽きる。**」と述べる。よって子を作る段は (E1) を除く 4 種であり、
      子孫はその辺で `b` から辿れるものに限る。**(F) の解放が作る段をこの 4 種に数えるのは、
      デストラクタの本体が Fix の関数であって `App` を持ちうるからである** (D24 の (F))。
      `b` が終わった後にこの段がそれらのオブジェクトへ届く道は、`b` の
      終端の `Ret` が渡した参照だけである。**この段自身の生成コードが `b` の呼び出しから得る Rust の値は
      その 1 つの返り値だけである** -- `apply_lambda` は `Option<Object<'c>>` を返すだけであり
      (`CODE src/generator.rs: Generator::apply_lambda`)、呼び出し元の生成コードはその返り値以外に
      `b` の内側のオブジェクトを指すハンドルを持たない。よってこの段が `b` の内側で割り当てられた
      オブジェクトへ届く道は、その返り値の leaf を経る他に無い。**`Obl` の側もこれと同じ形を述べる** --
      D24 の (E4) は「`b` を作ったのが (E2) のうちオペランドを適用する `Llvm` の段であれば、それらの
      参照はその段を実行した活性化の `Obl` に入り、その活性化は同じ位置で続きを実行する」と述べ、D24 は
      「**段の記述は `Obl` について網羅である。**」「**すなわち、ここに挙がっていない動きは起きない。**」
      と述べるので、`b` の内側で割り当てられたオブジェクトのうちこの段の活性化が `Obl` に持つ参照も、
      その `Ret` が渡した leaf が指すものに限る。A3 が参照の作られ方を
      「既存のオブジェクトへの新しい参照 (retain を伴う読み出し)」と述べるので、`o` はその leaf が指す
      オブジェクトそのものか、そこから記憶域を辿って着くオブジェクトである。
      その leaf が指す各計数下オブジェクト `o'` について、L1 (b) より `b` の終端の `Ret` が消費した参照は
      この段を実行している活性化の `Obl` に入る。**その参照は `p` でまだ処分されていない。**段の境界に
      ついての網羅ではこれを言えない -- D24 は「**この網羅は段の境界についてである。** 1 つの段の生成
      コードは、この表に行を持たない素動作を段の中で出しうる。」と述べ、`p` は段内の点だからである。
      **段内の点で言うには、その op の `generate` が出す release を読む。**D24 が「**在りかは述語で
      決める** -- `Generator::retain`・`Generator::build_retain`・`Generator::release` の呼び出しを
      出す生成コードの全体であり」と述べるとおりに走査すると、`src/` の `gc.release(`・`self.release(`・
      `release_nonnull_boxed(` は 15 か所であり、**そのどれも、適用が返した値の leaf の参照を処分しない。**
      内訳は次のとおりである。
      `RcExpr::Release` の 2 か所 (`eval_rc_expr_inner` の `skip_null_check` の 2 枝)。boxed 容器と
      unbox 容器の `Destructure` の 2 か所 (`get_struct_fields`)。boxed union の payload の読み出しが
      union を処分する 1 か所 (`get_union_value`)。**書き換える前の古い値**を処分する 2 か所
      (`write_to_array_buf` の古い要素、`InlineLLVMStructSetBody` の古い欄)。**この節点のオペランド**を
      処分する 7 か所 (`InlineLLVMArrayAppendCapacityUnchecked` の 2 か所、`make_struct_union_unique` の
      共有の腕、`InlineLLVMUnionModBody` の不一致の腕の `modifier`、
      `InlineLLVMWithRetainedFunctionBody` の `x`、`initialize_array_buf_by_value` と
      `append_value_into_array_buf` の `value`)。そして `get_funptr_release` が定義する補助関数の本体
      1 か所 (この本体が走るのは環境が (E9) の段でその番地を呼ぶときであり、`<1>1` の第 7 群 ((E9) の
      retain の側) と対をなす)。適用が返した値を処分するものはこの中に無いので、その参照は `p` で
      `Obl` に在る。D25 が挙げるのは未処分の参照の持ち手なので、D8 より
      `H(o') ≥ 1` であり、`p` は解放について閉じているので L2 (a-1) より `o'` は `p` で生きている。
      L2 (b) がそこから到達できるオブジェクトへ広げる。グローバル状態のオブジェクトは A8 より解放されない。
      `b` が返した参照をそのまま持つ leaf は `<2>0` が除く。
      BY <ref id=e11772a/>, <ref id=b6673ca/>, <ref id=c9e4cca/>, <ref id=fd95f12/>, <ref id=56c2068/>, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=e3436e8/> (E4), <ref id=e3436e8/> (E9), <ref id=e3436e8/> (F), <ref id=e3436e8/> (活性化の林), <ref id=0b850c9/>,
         <ref id=88a06de/>, <ref id=f3dcc8f/>, <ref id=881a063/>, <1>1, <1>1a, <2>0,
         CODE src/generator.rs: Generator::apply_lambda,
         CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
         CODE src/object.rs: ObjectFieldType::get_struct_fields, ObjectFieldType::get_union_value,
         ObjectFieldType::write_to_array_buf, ObjectFieldType::initialize_array_buf_by_value,
         ObjectFieldType::append_value_into_array_buf,
         CODE src/fixstd/builtin.rs: InlineLLVMArrayAppendCapacityUnchecked, make_struct_union_unique,
         InlineLLVMStructSetBody, InlineLLVMUnionModBody, InlineLLVMWithRetainedFunctionBody,
         InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody
    <3>1. CASE op が `InlineLLVMBoxedFromRetainedPtrIOS` でも `applies_a_function_operand` を宣言する
          op でもない。
      A3 の `Unknown` の行の限定が当たる -- 参照が作られるオブジェクト `o` は、この op のオペランドの
      inhabited な boxed leaf が指すオブジェクトから到達できるか、グローバル値が到達するオブジェクトで
      ある。グローバル値が到達するオブジェクトは A8 より解放されない。前者について、同じ行はその参照を
      「既存のオブジェクトへの新しい参照 (retain を伴う読み出し)」と述べる。読み出しはこの節点が
      オペランドの記憶域を読む動作であり、`Let(x, Llvm(gen, args), k)` は D7 の読む構文で、読まれる値は
      各オペランドである (D7、D32 の (読み-1))。**`o` がオペランドの leaf が指すオブジェクトそのもので
      あっても A26 の第 2 節の主語に入る** -- そのとき `o` はこの読み出しが記憶域を読んだオブジェクト
      そのものであり、A26 の第 2 節が第 1 の主語として挙げる「その節点が**記憶域から読んだ**
      オブジェクト」に当たるからである。それより深いオブジェクトは第 2 の主語 -- そこから到達できる
      (D25) オブジェクト -- に当たる。いずれにせよ `<1>1a` がそれを扱う。
      BY <ref id=e11772a/>, <ref id=b6673ca/>, <ref id=fd95f12/>, <ref id=56c2068/>, <ref id=0b850c9/>, <ref id=1b00a9e/>, <1>1a, CODE src/rc_ir/provenance.rs: LeafOrigin
    <3>2. QED
      A3 の `Unknown` の行が限定の成り立たない op として挙げるのは 2 種 --
      `InlineLLVMBoxedFromRetainedPtrIOS` とオペランドを適用する op -- であり、`<3>0`、`<3>0a`、`<3>1` が
      その 2 種とそれ以外を尽くす。
      BY <ref id=e11772a/>, <3>0, <3>0a, <3>1
  <2>3. CASE 宣言が空集合。
    A3 より、生成コードはその leaf に何も置かず、その leaf は inhabited にならない。D10 の生成は
    inhabited な leaf についてだけ参照を作るので、参照は生じない。よって言明は空虚に成り立つ。
    BY <ref id=e11772a/>, <ref id=f06144e/>
  <2>4. CASE 宣言が複数の元を持つ。
    A3 の数え上げより、複数の元を宣言する op はこのコミットのプログラムには存在しない。よってこの場合は
    起きない。
    BY <ref id=e11772a/>
  <2>5. QED
    A3 の表は宣言を、空集合、単一の `Arg(j, σ)`、単一の `Fresh`、単一の `Unknown`、複数の元の 5 つに
    尽くす。単一の `Arg(j, σ)` の leaf は D10 の生成の表から外れており (その行の条件は
    「`Llvm(gen, args)` の結果の leaf のうち、`result_prov` の宣言が単一の `Arg(j, σ)` **でない**もの」
    である)、参照を作らない。残る 4 つを `<2>1`-`<2>4` が扱い、そのうち適用が返した参照を
    受け取る leaf は `<2>0` が除く。
    BY <ref id=e11772a/>, <ref id=f06144e/>, <2>0, <2>1, <2>2, <2>3, <2>4

<1>6. CASE boxed 容器の `Destructure` の名前付きフィールドの行、および boxed union の変位アームの
      payload の行。
  <2>1. boxed 容器の `Destructure` について、この節点は容器 `c` のオブジェクトの記憶域を読み、参照が
        作られるオブジェクトはそこから到達できる (D25)。
    `get_struct_fields` の boxed の枝は、各フィールドを容器のオブジェクトの記憶域から取り出して retain し、
    それから容器を release する。取り出されるフィールドは容器のオブジェクトが保持する値の boxed leaf で
    あり、D25 よりそのオブジェクトは容器のオブジェクトが持つ参照が指す先である。
    BY <ref id=0b850c9/>, CODE src/object.rs: ObjectFieldType::get_struct_fields
  <2>2. boxed union の変位アームについて、この節点は scrutinee のオブジェクトの記憶域を読み、参照が
        作られるオブジェクトはそこから到達できる (D25)。
    <3>1. この段が選んだ変位アームの `tag` は `Some(t)` であり、`t` はその時点の scrutinee の値の実行時の
          タグに等しい。
      D21 より、活性化が選ぶのは `tag` が実行時のタグに等しいアームであり、そのようなアームが無いときだけ
      コード生成の振る舞いに従う。コード生成は最後のアームのブロックを switch の default とし、最後で
      ないアームについては `arm.tag` を `expect` で取り出す (`CODE src/rc_ir/codegen.rs:
      Generator::eval_rc_match` -- `else_bb` は `arm_bbs.last()`)。最後でないアームの `tag` が `None` で
      あればそこで panic する -- そのときプログラムは走らず、この場合の段は存在しない。よって段が在る
      実行では catch-all アームは最後のアームであり、default に落ちる選択はその catch-all アームである。
      A16 より `arms` は catch-all アームを持つか実行時のタグに等しい `tag` を持つアームを持つので、
      変位アームが選ばれたのは、その `tag` が実行時のタグに等しいときである。
      BY <ref id=f769887/>, <ref id=c232680/>, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
    <3>2. QED
      D10 の生成の表より、この段が参照を作るのは payload の各 boxed leaf についてである。`<3>1` より
      その payload は変位 `t` のものであり、`t` は実行時のタグなので、その leaf は scrutinee の値の
      inhabited (D16) な boxed leaf である。scrutinee は boxed なので payload はその記憶域に在り、
      D24 の (E2) の `H` の表の boxed union の行はその読み出しに参照 1 つを作る。D25 より、それが指す
      オブジェクトは scrutinee のオブジェクトが持つ参照の指す先である。
      BY <ref id=f06144e/>, <ref id=66c9670/>, <ref id=e3436e8/> (E2), <ref id=0b850c9/>, <3>1
  <2>3. QED
    `Destructure(c, fs, s, k)` と `Let(x, Match(v, arms), k)` はどちらも D7 の読む構文であり、読まれる値は
    それぞれ容器 `c` と scrutinee `v` である。`<2>1` と `<2>2` より、この段が作る参照が指すオブジェクトは、
    この節点が記憶域から読んだオブジェクト -- 容器または scrutinee のオブジェクト -- から到達できる。
    `<1>1a` がそれを扱う。
    BY <ref id=56c2068/>, <1>1a, <2>1, <2>2

<1>7. CASE (F) の解放が `Destructor` のオブジェクト `o` について行う `_dtor` への retain。
  D24 の (F) より、この retain の対象は `o` が `_dtor` の欄に持つ関数である
  (`CODE src/generator.rs: Generator::build_traverser_work_nonnull_boxed_with` --
  `obj.is_destructor_object()` の枝が `build_run_destructor` を `traverse_refs` の前に置く。
  `CODE src/generator.rs: Generator::build_run_destructor` -- `build_retain(dtor, one, ..)` が
  `apply_lambda` の前に立つ)。対象のオブジェクトがグローバル状態 (D26) ならば A8 より解放されない。
  計数下ならば次のとおりである。D24 の (F) より、この解放が `o` の持つ参照を処分するのは `_dtor` の適用と
  `IO` の動作の往復の**後**なので、この retain の点で `o` はまだその参照を持っている。`o` は割り当てられて
  いて記憶域を返す前であり、D24 が「解放の中では `o` はまだ解放されておらず、走査は `o` を読む」と述べる
  ので、`o` は D25 の 2 つ目の持ち手である。A5 よりその leaf は参照を 1 つ持ち、D8 より対象のオブジェクトの
  `H` は 1 以上である。`p` は解放について閉じているので、対象は `p` で解放されていない。
  BY <ref id=4f63121/>, <ref id=b6673ca/>, <ref id=ec8d1a0/>, <ref id=859cf84/>, <ref id=e3436e8/> (F), <ref id=0b850c9/>, <ref id=88a06de/>,
     CODE src/generator.rs: Generator::build_run_destructor,
     Generator::build_traverser_work_nonnull_boxed_with

<1>7a. CASE 段の中で相殺する retain (`<1>1` の第 5 群)。
  `InlineLLVMWithRetainedFunctionBody::generate` は、オペランド `f` と `x` をスコープから読み、
  `gc.retain(x, ..)` を出し、`gc.apply_lambda(f, vec![x], false)` を出し、その後に `gc.release(x, ..)` を
  出す。この retain が作る参照が指すのは `x` の値の inhabited な各 boxed leaf のオブジェクトである。
  この op は `borrows_operand` も `result_prov` も override しないので、`borrows_operand(i)` は既定の偽で
  あり、宣言は既定の単一の `Unknown` である。よって D9 の `Llvm` の行はこの op の各オペランドの boxed
  leaf を消費とする。
  **生成コードがこの retain より前に `x` の leaf の参照を手放す素動作を出さないことは、上の順序が
  与える** -- `apply_lambda` も `release` も retain の後に立つ。よって `<1>1e` が当たり、対象は `p` で
  解放されていない。
  BY <ref id=e11772a/>, <ref id=9d74736/>, <1>1, <1>1e,
     CODE src/fixstd/builtin.rs: InlineLLVMWithRetainedFunctionBody,
     CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand, LLVMGen::result_prov

<1>7b. CASE 相殺しない retain (`<1>1` の (K-ii) のうち、複製・割り当てたオブジェクトの欄へ書く形)。
  4 か所を 2 つに分ける。
  **複製が原本の記憶域から読んだ値を retain する形** -- `clone_struct` は各フィールドを
  `move_out_struct_field` で原本 `src` の記憶域から取り出してから retain し、`clone_array_range`
  (`clone_array_buf` が呼ぶ) は `src_buffer` の各スロットを `build_load` で読んでから retain する。
  `clone_union` は原本 `src` の payload バッファを `extract_field` で取り出して複製 `dst` へ入れ、
  `retain_union` で活性変位の payload を retain する -- boxed な union についてその取り出しは原本の
  記憶域の読みである。`clone_struct` と `clone_union` を呼ぶのは `make_struct_union_unique` の共有の腕で
  あり、その `obj` はこの op のオペランドである。`clone_array_buf` を呼ぶ 4 か所も、原本の配列の記憶域を
  読む。読み出されたオブジェクトは、この節点が**記憶域から読んだ**オブジェクト -- 原本の struct・union・
  `#ArrayStorage` -- から到達できる (D25) ので、`<1>1a` が当たる。
  **オペランドの値を割り当てた記憶域のスロットへ書く形** -- `append_value_into_array_buf` は
  `build_retain(value, count)` を出してから `count` 個のスロットへ格納し、最後に `value` を release
  する。それを呼ぶのは `InlineLLVMArrayAppendValueCapacityUnchecked` であり、その `value` はこの op の
  オペランドである。この op は `borrows_operand` を override しないので既定の偽である。
  **`InlineLLVMArrayAppendValueCapacityUnchecked::result_prov` は `Provenance::uniform(.., Fresh)` を
  返すので、素通し (単一の `Arg`) を宣言する結果 leaf は 1 つも無い** -- よって D9 の `Llvm` の行は
  `value` の leaf を消費とする。**生成コードがこの retain より前に `value` の leaf の参照を手放す素動作を
  出さないことは、上の順序が与える。**よって `<1>1e` が当たる。
  BY <ref id=9d74736/>, <ref id=0b850c9/>, <1>1, <1>1a, <1>1e,
     CODE src/object.rs: ObjectFieldType::clone_struct, ObjectFieldType::clone_union,
     ObjectFieldType::clone_array_buf, ObjectFieldType::clone_array_range,
     ObjectFieldType::append_value_into_array_buf, ObjectFieldType::move_out_struct_field,
     CODE src/fixstd/builtin.rs: make_struct_union_unique,
     InlineLLVMArrayAppendValueCapacityUnchecked,
     InlineLLVMArrayAppendValueCapacityUnchecked::result_prov,
     CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand

<1>7c. CASE (E9) の retain (`<1>1` の (K-iii))。
  A17 (ii-c) より、この段は環境が `get_funptr_retain` の渡した番地を呼ぶ段であり、その番地が指す
  オブジェクトへの参照を 1 つ作って環境の持ち分に足す。**環境がその番地を呼ぶのは、その番地が指す
  オブジェクトへの参照を自分が持っている点でだけである** (A17 (ii-c))。よってこの retain の直前の点で、
  環境はそのオブジェクト `o` への参照を 1 つ持っており、D25 の 3 つ目の持ち手より `o` は未処分の参照を
  持つ。D8 より `H(o) ≥ 1` である。`p` は解放について閉じているので、L2 (a-1) より `o` は `p` で
  解放されていない。
  BY <ref id=c9e4cca/>, <ref id=ec8d1a0/>, <ref id=0b850c9/>, <ref id=881a063/>

<1>8. `p` まで閉じている段内の点 `p` の直後の素動作が `S` の中で参照を作るとき、その参照が指す
      オブジェクトは `p` で解放されていない。
  **参照を作る素動作の直前には段内の点が在る。** D24 が束ねる切れ目は、処分とそれが起こす解放のあいだと、
  素動作とそれに付随する書き込みのあいだの 2 つだけであり、その右側に立つのは解放と書き込みである。
  書き込みは 6 種のどれでもない (D24) ので、解放を除く 5 種の素動作の直前には段内の点が在り、参照を作る
  素動作 -- (E2) の生成と割り当て、(F) の retain、(E9) の retain、および段の中の retain (`<1>1` の
  (K-ii)) -- はどれも解放ではない。
  `<1>1` が参照を作る動作を数え上げ、それを (K-i)・(K-ii)・(K-iii) の 3 種に分ける。**(K-i)** の (E2) の
  生成の 7 行を `<1>2`-`<1>6` が尽くす -- `<1>5` が `Llvm` の行と `InlineLLVMBoxedFromRetainedPtrIOS` の
  行の 2 つを扱う。(F) の retain を `<1>7` が扱う。**(K-ii)** の段の中の retain を `<1>7a` (相殺するもの)
  と `<1>7b` (相殺しないもの) が扱う。**(K-iii)** の (E9) の retain を `<1>7c` が扱う。(F) の解放が
  始めた活性化の木の節点が作る参照も、その本体についての (K-i) と (K-ii) によるので (`<1>1`)、同じ場合が
  扱う -- `<1>1a`・`<1>1b`・`<1>1e` はどれも「参照を作る素動作の直前の点」を主語にしており、`<1>1` が
  挙げる (K-i)・(K-ii) の活性化の節点のどちらについても、その本体が D11 を満たすこと (H2) と `p` まで
  閉じていることから (S-c) と (S-a) を引く。`<1>1c` が扱う割り当ての場合は、そのオブジェクトが `p` に
  在らないので `p` で解放されていない。**(K-iii) は環境が実行する段であって活性化の節点ではないので、
  `<1>1a`・`<1>1b`・`<1>1e` のどれも当たらず、`<1>7c` が独立に扱う。**
  BY (H2), <1>1, <1>1a, <1>1b, <1>1c, <1>1d, <1>1e, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>7a, <1>7b,
     <1>7c

<1>9. `p` まで閉じている段内の点 `p` の次の段内の点も、解放について閉じている。したがってその点まで
      閉じている。
  `p` と次の点のあいだに走るのは、1 つの素動作か、処分とそれが起こす解放の対である -- D24 が束ねる
  切れ目はその 2 つのあいだと、素動作とそれに付随する書き込みのあいだだけであり、書き込みは 6 種のどれでも
  ない (D24)。6 種の素動作を順に見る。**受け渡し**は `H` を変えず、解放も割り当ても起こさない。**処分**は
  参照を 1 つ処分してその先の `H` を 1 下げるので、`H(o) ≥ 1` である計数下のオブジェクトの集合は
  増えない。**1 つの素動作が複数のオブジェクトの参照を処分する形も同じである** -- A5 の `#ArrayStorage`
  の節がそれであり、記憶域のカウントを 0 にした素動作に `size` 個の要素の参照の処分が付く。そのどれも
  `H` を下げる向きなので、`H(o) ≥ 1` である計数下のオブジェクトの集合は増えない。**割り当て**は新しい
  オブジェクトを作り、`<1>1c` よりそれは動作の後の点で解放されていない。**解放**は、D24 の (F) より
  `H(o) = 0` になった `o` について起きるので、新たに解放される `o` の `H(o)` は 0 である。
  **グローバル化**は印を付けたオブジェクトを計数下の集合から外すので (D26)、条件が量化する集合が減る
  だけである。**生成**は `H` を 1 上げるが、`<1>8` よりその先は `p` で解放されていないので、解放されて
  いるオブジェクトの `H` は動かない。区間が処分とそれが起こす解放の対であるときは、この 2 つの場合が
  続けて当たる -- 処分が `H(o)` を 0 にし、解放はその `o` について起きるので、次の点で `o` は
  `H(o) = 0` であり条件の量化する集合の外にある。よって次の点でも「`H(O) ≥ 1` ならば解放されていない」が
  成り立ち、`p` まで閉じていることと併せてその点まで閉じている。
  BY <ref id=4f63121/>, <ref id=56c2068/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=e3436e8/> (F), <ref id=88a06de/>, DEF 段の素動作と段内の点, DEF 段内の点で閉じている, <1>1c, <1>8

<1>10. QED
  段内の点の列は有限である (DEF 段の素動作と段内の点)。`<1>0` が最初の点 `t` まで閉じていることを与え、
  `<1>9` が 1 点ずつそれを運ぶので、帰納により `S` のどの段内の点も解放について閉じており、`S` の最後の
  点まで閉じている -- これが (a) である。そのうえで `<1>8` が、各点についてその直後の動作が作る参照に
  ついて (b) の前半を与える。D25 が「生きている」を、割り当てた素動作より後、解放する素動作より前と
  定めるので、`t` で解放されていたオブジェクトはその後のどの点でも解放されており、その対偶が (b) の
  後半である。
  BY <ref id=e3436e8/>, <ref id=0b850c9/>, DEF 段の素動作と段内の点, <1>0, <1>8, <1>9

### L3 (実行のどの時点も解放について閉じている) <!--#58667a9-->

**言明**。**(H3)** プログラム `P` のすべての本体が D11 を満たすとする。`P` の実行 `ρ` の各時点と
各段内の点について、次の 2 つが成り立つ。

- **(a)** その点は解放について閉じている (D11a)。すなわち `H(O) ≥ 1` である各計数下オブジェクト `O` は
  その点で解放されていない。したがって `ρ` はその点まで閉じている。
- **(b)** `H(o) = 0` である各計数下オブジェクト `o` は、その点が D24 の時点であれば解放されている。

**(a) が D11a が名指す性質である。** D11a は「**プログラムのすべての本体が D11 を満たすとき、その実行
(D24) の各時点と各段内の点で閉じている** -- `p51-runs.md` の `L3` がそれを示す」と書き、D11 の (S-c) の
接頭条件はこの性質である。

**(b) を D24 の時点について言うのは、D24 の (F) がその粒度でそれを述べるからである。** (F) は「ある段が
参照を処分して計数下のオブジェクト (D26) `o` の `H(o)` が 0 になったとき、`o` は**その同じ段の中で解放
される**」と述べ、続けて「段の終わりには `H = 0` の計数下のオブジェクトはすべて解放されている」と述べる。
(b) を読むのは `<1>2` の `<2>3` と P27 の `<1>0` であり、どちらもこの粒度で読む。

<1>1. `ρ` の最初の時点について (a) と (b) が成り立つ。
  <2>1. 最初の時点で解放されているオブジェクトは無い。
    オブジェクトが解放されるのは (F) の解放においてであり、(F) は段の中で起きる。最初の時点までに段は
    1 つも実行されていない。
    BY <ref id=e3436e8/> (F), <ref id=0b850c9/>
  <2>2. 最初の時点の各計数下オブジェクト `o` について `H(o) ≥ 1` である。
    D24 は「実行の最初の時点に在る参照は、環境が持ってきたものだけである」と述べ、`FFI_EXPORT` の
    エントリ点が boxed な引数を取る実行ではその参照が在ると述べる。A17 (i-c) は「実行の最初の時点に
    在る参照は、環境が持つか、環境が持ち込んだオブジェクトの leaf が持つ」と述べ、各計数下オブジェクト
    `o` の `H(o)` を、環境が持つ `o` への参照の個数と、生きているオブジェクトの leaf が持つ `o` への
    参照の個数の和に等しいとする。この時点に在る計数下オブジェクトは環境が持ち込んだものであり、
    (i-c) の第 1 文がその持ち込みを参照の側から数える -- 環境が参照を持つオブジェクトについては
    `E` の参照が 1 つ、そこから到達できる (D25) オブジェクトについては自分を指す leaf を持つ生きている
    オブジェクトの分が 1 つ、それぞれ数えられる (A5)。
    **`E` から到達できないオブジェクトを環境が持ち込む形は無い** -- A17 の (i-c) が
    「**その時点に生きている各計数下オブジェクトは、少なくとも 1 つの持ち手を持つ** -- 誰も指さないまま
    解放されずに在るオブジェクトを、環境は持ち込まない」と述べる。持ち手は D25 の 3 種であり、この時点に
    生きている活性化は無いので (D24 -- 段が 1 つも実行されていない)、持ち手は生きているオブジェクトか
    環境である。よって `H(o) ≥ 1` である。
    BY <ref id=4f63121/>, <ref id=c9e4cca/>, <ref id=ec8d1a0/>, <ref id=e3436e8/>, <ref id=0b850c9/>
  <2>3. QED
    `<2>1` より「解放されている」計数下オブジェクトは無いので (a) が成り立ち、`<2>2` より「`H(o) = 0`」の
    計数下オブジェクトも無いので (b) が空虚に成り立つ。最初の時点は最初の段の最初の段内の点でもある
    (D24)。
    BY <ref id=859cf84/>, <ref id=e3436e8/>, DEF 段の素動作と段内の点, <2>1, <2>2

<1>2. ASSUME  **(I1)** `ρ` は時点 `t` まで閉じている、
             **(I2)** `t` について (b) が成り立つ
      PROVE   `t` の直後の 1 段 `S` のどの段内の点についても (a) が成り立ち、`S` の後の時点について
              (b) が成り立つ。
  <2>1. `S` を持つ活性化の本体と、`S` の中で (F) の解放が始めた活性化の木の各活性化の本体は D11 を
        満たす。
    D23 より活性化の本体は `P` のある関数の `body` かあるグローバル初期化子の `init` であり、(H3) は
    `P` のすべての本体について D11 を述べる。
    BY (H3), <ref id=ff5985d/>
  <2>2. `S` のどの段内の点も解放について閉じている。すなわち `S` の各点について (a) が成り立つ。
    L2b (a) を `t` と `S` に当てる。(I1) が L2b の (H1) を、`<2>1` が L2b の (H2) を満たす。
    BY (I1), <ref id=ba46997/>, <2>1
  <2>3. `S` の後の時点で `H(o) = 0` である計数下のオブジェクト `o` は解放されている。
    `t` において `H(o) = 0` であれば、(I2) より `o` は `t` で解放されており、D25 が「生きている」を
    割り当てた素動作より後・解放する素動作より前と定めるので `S` の後でも解放されている。`t` において
    `H(o) ≥ 1` であれば、`S` の中で `H(o)` が 0 に**なった**ということである。**`H` を下げる素動作は
    参照の処分だけである** -- D24 は「**段の記述は `Obl` について網羅である。**」の下で
    「(E1)-(E9) と (F) は、各段について `Obl` を離れる参照の行き先と、作られる参照の持ち手と、`H` の
    動きを全部書いている。」と述べ、そこで `H` を下げるのは
    「**処分**された参照は以後存在せず、`H` が 1 下がる」の行だけである。受け渡しは `H` を変えず
    (「行き先のある参照は存在し続け、`H` は変わらない」)、生成と割り当ては上げ、グローバル化はその
    オブジェクトを計数下の集合から外し (D26)、解放はそれ自身が参照の処分としてこの数え上げに入る。よって
    D24 の (F) が「ある段が参照を処分して計数下のオブジェクト (D26) `o` の
    `H(o)` が 0 になったとき」に `o` は「**その同じ段の中で解放される**」と述べ、続けて「段の終わりには
    `H = 0` の計数下のオブジェクトはすべて解放されている」と述べるとおり、`o` は `S` の中で解放されて
    いる。
    BY (I2), <ref id=e3436e8/>, <ref id=e3436e8/> (F), <ref id=0b850c9/>, <ref id=88a06de/>, DEF 段の素動作と段内の点
  <2>4. QED
    `<2>2` が (a) を `S` の各段内の点について、`<2>3` が (b) を `S` の後の時点について与える。
    BY <2>2, <2>3

<1>3. QED
  時点までの段の数についての帰納。`<1>1` が最初の時点について (a) と (b) を与え、`<1>2` が 1 段ずつ運ぶ。
  各段について (a) がその段のすべての段内の点で成り立つので、`ρ` はその段の最後の点まで閉じており、
  (b) はその段の後の時点で成り立つので、次の段に `<1>2` を当てる 2 つの条件が揃う。
  BY <1>1, <1>2

**注 (段が参照を受け渡すこと)**。L3 は `Obl` の推移を読まない -- 解放について閉じていることは `H` と解放
だけの条件だからである。段が参照をどこへ渡すかは L0a と L1 が扱い、L2b の `Llvm` の場合がそれを読む。

### L4 (`H` の分解) <!--#61b8f53-->

**L4.** ASSUME  NEW `P`: RcProgram、 <!--#7ea4752-->
                **(G0)** `P` は `insert_rc` の入力から `cancel` の出力までのどこかに現れる、
                **(G1)** `P` は D12 の意味で RC 規律を満たす、
                NEW `ρ`: `P` の実行 (D24)、
                **(G2)** `ρ` において借用する unit を持つ本体の活性化を作る段は (E3) に限られる、
                A20、
                NEW `p`: `ρ` の時点または段内の点 (D24)
        PROVE   `p` における各計数下オブジェクト (D26) `o` について

                `H(o) = Σ_{生きている活性化 a} Obl(a)[o] + Σ_{生きているオブジェクト o'} R(o')[o] + E[o]`

                である。ここで `R(o')[o]` は、`o'` が D25 の 2 つ目の持ち手として持つ `o` への参照の
                個数である。

**(G0)・(G1)・(G2)・A20 は P28 の 4 つの前提である。**P28 はその下で、実行の各時点と各段内の点について
(a) を述べる。**この命題を引く段は、その 4 つが揃っている場合にだけ在る。**

**段内の点まで量化するのは、それを要る読み手が在るからである。** 第 4 節の P27 の `<1>1` の
`<2>3` `<3>2` は、(E5) の段の**中**で走る `mark_global` の起点にこの等式を当てており、その点は
段内の点である。

<1>1. `H(o)` は `o` への処分されていない参照の総数である。
  D8 が `H(o)` をそう定め、D26 が D8 の参照を計数下のオブジェクトへの参照に限る。
  BY <ref id=ec8d1a0/>, <ref id=88a06de/>

<1>2. `p` において、処分されていない各参照は、D25 が挙げる 3 種の持ち手 -- 生きている活性化、生きている
      オブジェクト、環境 -- のちょうど 1 つに属する。
  P28 (a) がこれを述べる。P28 の 4 つの前提は (G0)、(G1)、(G2)、A20 であり、この命題の仮定に在る。P28 は
  その実行の**各時点と各段内の点**についてそれを述べるので、`p` に当たる。
  BY (G0), (G1), (G2), <ref id=680aaa9/>, <ref id=0b850c9/>, <ref id=0d151d9/>

<1>3. 3 種の持ち手が `p` で持つ `o` への参照の個数は、それぞれ `Σ_a Obl(a)[o]`、`Σ_{o'} R(o')[o]`、
      `E[o]` である。
  D25 より、生きている活性化 `a` が持つのは `Obl(a)` の元であり、その `o` への個数が `Obl(a)[o]` で
  ある。生きているオブジェクト `o'` が持つのは、A5 と D25 のとおり、`o'` が `#ArrayStorage` のオブジェクト
  でなければ `o'` が保持する値の inhabited な boxed leaf のうち計数下のオブジェクトを指すものが 1 つずつ
  であり、`o'` が `#ArrayStorage` のオブジェクトであるときは、A5 のとおりその記憶域の各スロットのうち、
  生成の素動作がそこへ書いた時点から要素の型の inhabited な boxed leaf を 1 組ずつ持つもの (`size` は
  この数え上げを切らない) である。その `o` への個数が `R(o')[o]` である。環境が持つのは `E` の元で
  あり、その `o` への個数が `E[o]` である。**割り当てられた直後のオブジェクトの埋まっていない単位は、
  この和に何も足さない** -- D24 が、その単位は参照を 1 つも持たず、単位が参照を持つのは受け渡しか
  生成がそれを書いた瞬間からであると定める。
  BY <ref id=4f63121/>, <ref id=e3436e8/>, <ref id=0b850c9/>

<1>4. QED
  `<1>1` の総和を持ち手で分けると、`<1>2` より各参照はちょうど 1 つの項に数えられ、どの項にも属さない
  参照は無い。`<1>3` が各項の値を与える。
  BY <1>1, <1>2, <1>3

### L5 (正常終了する実行について) <!--#252ad5a-->

**言明**。**(J0)** `P` は `insert_rc` の入力から `cancel` の出力までのどこかに現れるとし、
**(J1)** プログラム `P` のすべての本体が D11 を満たすとし、**(J2)** `ρ` を `P` の正常終了する
実行 (D31) とする。さらに P28 の残る 2 つの前提 -- **(J3)** `ρ` において借用する unit を持つ本体の
活性化を作る段が (E3) に限られること、および A20 -- を仮定する。このとき次が成り立つ。

- **(a)** `ρ` のすべての活性化が終わっている。
- **(b)** 各活性化 `a` が終わった時点で `Obl(a)` は空である。
- **(c)** `ρ` の最後の時点で、処分されていないどの参照の持ち手も、生きているオブジェクトか環境である。
- **(d)** `ρ` に現れるオブジェクトは有限個である。

<1>1. (a) が成り立つ。
  (J2) と D31 より、正常終了する実行の最後の時点で生きている活性化は無い。D23 より、生きている活性化とは
  始まって終わっていない活性化である。よって始まった活性化はすべて終わっている。
  BY (J2), <ref id=ff5985d/>, <ref id=0de0033/>

<1>2. (b) が成り立つ。
  活性化 `a` が終わるのは `B(a)` の終端の `Ret` に着いてその消費を行うときである (D23)。`a` が辿った
  節点の列は `B(a)` の実行路 (D3) であり (D21, D23)、(J1) より `B(a)` は D11 を満たすので、(S-b) が
  「実行路の終端の `Ret(v)` において、その `Ret` の消費を行った後の `Obl` は空である」を与える。
  BY (J1), <ref id=ca36627/>, <ref id=95427eb/> (S-b), <ref id=c232680/>, <ref id=ff5985d/>

<1>3. (c) が成り立つ。
  `<1>1` より最後の時点で生きている活性化は無いので、D25 が挙げる 3 つの持ち手のうち活性化は残らない。
  `<1>2` より、終わった各活性化はその終わりの時点で `Obl` を空にしているので、終わった活性化の側に
  参照が残ることも無い。P28 (a) より、処分されていない各参照はちょうど 1 つの持ち手を持つ -- P28 の
  4 つの前提は、`P` が `insert_rc` の入力から `cancel` の出力までのどこかに現れることが (J0)、
  D12 (すべての本体が D11 を満たすこと) が (J1) であり、残る 2 つは (J3) と A20 である。
  BY (J0), (J1), (J3), <ref id=680aaa9/>, <ref id=3d96eb8/>, <ref id=0b850c9/>, <ref id=0d151d9/>, <1>1, <1>2

<1>4. (d) が成り立つ。
  <2>1. `ρ` の段の列は有限である。
    (J2) より `ρ` は正常終了する実行であり、D31 はそれを「段の列が有限で、(E6) の段を含まず、最後の段の
    後に生きている活性化が 1 つも無い」ことと定める。
    BY (J2), <ref id=0de0033/>
  <2>2. 1 つの段の素動作の列は有限であり、その中の割り当ての素動作も有限個である。
    D24 は 1 つの段を不可分な素動作の**有限列**へ分解し、割り当てはその 6 種の 1 つである
    (DEF 段の素動作と段内の点)。
    BY <ref id=e3436e8/> (F), DEF 段の素動作と段内の点
  <2>3. `ρ` の中でオブジェクトを割り当てる動作は、どれかの段の割り当ての素動作である。
    D24 は実行を段の列と定め、A17 の (iii) は「環境の動作も D24 の段としてこの実行の 1 つの列に並ぶ。
    段は不可分なので、環境が動くのは段と段のあいだである」と述べるので、実行の動作はどれもいずれかの段の
    中で起きる。D24 は 1 つの段を 6 種の素動作の有限列へ分解し、オブジェクトを作るのはそのうち割り当てだけ
    である (DEF 段の素動作と段内の点)。(F) の解放が段の中で始めた活性化の木の節点が行う動作もその列の元で
    あると D24 が定めるので、そこで起きる割り当てもその段の素動作の列の元である。
    BY <ref id=c9e4cca/>, <ref id=e3436e8/>, <ref id=e3436e8/> (F), DEF 段の素動作と段内の点
  <2>3a. `ρ` の最初の時点に環境が持ち込むオブジェクトは有限個である。
    A17 の (i-c) が「**環境が持ち込むオブジェクトは有限個である。**」と述べる。この節が要るのは、それらが
    割り当てた段をこの実行に持たないからであり (D25、D30)、`<2>3` の数え上げはその分を覆わない。
    BY <ref id=c9e4cca/>, <ref id=0b850c9/>, <ref id=081e39f/>
  <2>4. QED
    `ρ` に現れるオブジェクトは、`ρ` の中で割り当てられたものか、環境が最初の時点に持ち込んだものである --
    D24 が「実行の最初の時点に在る参照は、環境が持ってきたものだけである」と述べ、A17 の (i-c) が最初の
    時点に在る参照を環境が持つものと環境が持ち込んだオブジェクトの leaf が持つものに限り、それ以後に
    現れるオブジェクトは `<2>3` より段の割り当ての素動作が作るからである。**環境が実行の途中で新しい
    オブジェクトを持ち込む形は無い** -- A17 の (i-d) が「**最初の時点より後の (E1) が環境から受け取る
    boxed な値も同じである**-- 環境が Fix の boxed な値の番地を得る道はその対しかないので、」と述べ、
    続けて「**環境は実行の途中で新しいオブジェクトを持ち込まない。**」と述べるので、この 2 つで尽きる。
    前者は `<2>1` より段が有限個で `<2>2` より 1 つの段の割り当てが有限個なので有限個であり、後者は
    `<2>3a` より有限個である。
    BY <ref id=c9e4cca/>, <ref id=e3436e8/>, <2>1, <2>2, <2>3, <2>3a

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

## 4. P27 の証明

**P27.** ASSUME  NEW `P`: RcProgram、
                 **(K0)** `P` は `insert_rc` の入力から `cancel` の出力までのどこかに現れる、
                 **(K1)** `P` は D12 の意味で RC 規律を満たす、
                 NEW `ρ`: `P` の実行 (D24)、
                 **(K2)** `ρ` において借用する unit を持つ本体の活性化を作る段は (E3) に限られる、
                 A17、A18、A20
        PROVE   **(R1)** `ρ` のどの読み (D32) も、解放されたオブジェクトを読まない。
                **(R2)** `ρ` において、どのオブジェクトも高々 1 回しか解放されない。
                **(R3)** `ρ` が正常終了する実行 (D31) ならば、`ρ` の最後の時点で解放されていない計数下の
                オブジェクト (D26) は、環境が最後に持つ参照が指すオブジェクトから到達できるものに限る。

**(R3) の「環境が最後に持つ参照」は、README の (R3) の「環境が持つ参照」を時点で固定した形である。**
README は「正常終了する実行では、最後まで解放されずに残る計数下のオブジェクト (D26) は、環境が持つ参照が
指すオブジェクトから到達できるものに限る」と書き、どの時点の `E` かを書かない。ここで最後の時点に取るのは、
実行の途中で環境が持っていて最後の時点までに手放した参照を除いた形が強いからである。強い方を示せば
README の (R3) は従う。

**(K0)・(K2)・A20 は P28 の前提である。**P28 は D12 とこの 3 つを前提に取り、この証明は P28 (a) を L4
(`H` の分解) を通じて引く。`borrow_ify` の出力について (K0) と (K2) を果たすのは P14b であり、`cancel` の
出力もその範囲に入る。README の P27 の言明はこの 3 つを同じ形で持つ。

<1>0. `ρ` の各時点と各段内の点は解放について閉じており (D11a)、各時点で `H(o) = 0` である計数下の
      オブジェクトは解放されている。
  L3 を `P` と `ρ` に当てる。(K1) と D12 より `P` のすべての本体が D11 を満たすので、L3 の (H3) が
  揃う。
  BY (K1), <ref id=3d96eb8/>, <ref id=58667a9/>

<1>1. (R1) が成り立つ。
  <2>1. CASE (読み-1) D7 の読む構文の実行。
    その構文はある活性化 `a` の本体 `B(a)` の節点であり、(K1) と D12 より `B(a)` は D11 を満たす。`a` が辿った
    節点の列は `B(a)` の実行路 (D3) である (D21, D23)。**読みの直前の点は段内の点とは限らない** -- 読みは
    6 種の素動作のどれでもないからである。D24 が「**読みの直前の点では、勘定は直前の段内の点のものである。**」
    と述べ、「**その点と直前の段内の点のあいだに素動作は 1 つも無いので、`H` も `Obl` も `held` (D34) も
    動かず、解放も起きない。**」と続けるので、`<1>0` がその直前の段内の点まで与える閉じていることが、
    読みの直前の点までの接頭へそのまま移る。よって (S-c) の接頭条件が揃う。(S-c) は「D7 の読む構文が
    その位置で読みうる各オブジェクト、および `Retain(v, π)` と `Release(v, π)` が触れる (D7) 各
    オブジェクトは、**その読み・その触れる動作の直前の点で**解放されていない」を言い、その第 1 の半分の
    主語が、D32 の (読み-1) が読むオブジェクトである。
    BY (K1), <ref id=ca36627/>, <ref id=56c2068/>, <ref id=95427eb/> (S-c), <ref id=859cf84/>, <ref id=3d96eb8/>, <ref id=c232680/>, <ref id=ff5985d/>, <ref id=e3436e8/>, <ref id=1b00a9e/>, DEF 段の素動作と段内の点, <1>0
  <2>2. CASE (読み-2) (F) の解放の走査。
    D32 の (読み-2) より、この走査が読むのは解放されるオブジェクト `o` そのものである。D24 の (F) より、
    走査が走る間 `o` はまだ解放されていない。
    BY <ref id=e3436e8/> (F), <ref id=1b00a9e/>
  <2>3. CASE (読み-2) (E5) の `mark_global` の走査。
    <3>1. 走査の起点は、グローバル初期化子の活性化 `b` が終端の `Ret` で返す値の inhabited な各 boxed
          leaf が指すオブジェクトであり、その時点でその参照は `Obl(b)` に在る。
      D24 の (E5) は「グローバル初期化子の活性化が終端の `Ret` に着き、返す前に、環境が `mark_global`
      でその値が到達するオブジェクトのグラフ全体に印を付ける」と述べ、(E7) は「`b` が終端の `Ret` に着き、
      返す前に (E5) の段が走る。続いて、(E4) と同じく `b` の終端の `Ret` が消費する参照が `Obl(b)` を
      離れるが、行き先は呼び出し元ではなく `E` である」と述べる。この段が要るのは「返す前に」の語である --
      2 つの節はどちらも `mark_global` を終端の `Ret` の消費より前に置くので、(E5) の段の中で `b` は
      まだ終わっておらず (D23)、その `Ret` が渡す参照はまだ `Obl(b)` に在る。コードも
      `implement_rc_global` が `eval_rc_expr`、`mark_global`、`build_return` の順に出す。
      BY <ref id=ff5985d/>, <ref id=e3436e8/> (E5), <ref id=e3436e8/> (E7), CODE src/rc_ir/codegen.rs: Generator::implement_rc_global
    <3>2. 起点のオブジェクトは、その点で生きている。
      `<3>1` の参照は生きている活性化 `b` の `Obl(b)` に在るので、起点のオブジェクトを `o` とすると
      L4 より `H(o) ≥ 1` である -- L4 の 4 つの仮定は (K0)、(K1)、(K2)、A20 であり、この命題の仮定に
      在る。この点は (E5) の段の中の段内の点であり、`<1>0` より解放について閉じているので、L2 (a-1) より
      `o` はその点で生きている。グローバル状態のオブジェクトなら A8 より解放されない。
      BY (K0), (K1), (K2), <ref id=680aaa9/>, <ref id=b6673ca/>, <ref id=ec8d1a0/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=881a063/>, <ref id=61b8f53/>, DEF 段の素動作と段内の点, <1>0, <3>1
    <3>3. QED
      D32 の (読み-2) よりこの走査が読むのは起点と、そこから到達できるオブジェクトである。`<1>0` より
      その点は解放について閉じているので、L2 (b) がそこから到達できるオブジェクトへ `<3>2` を広げる。
      BY <ref id=859cf84/>, <ref id=1b00a9e/>, <ref id=881a063/>, <1>0, <3>2
  <2>4. CASE (読み-2) `Std::mark_threaded` の走査。
    <3>1. `Std::mark_threaded` は `Llvm` の op であり、走査の起点はそのオペランドの inhabited な boxed
          leaf が指すオブジェクトである。
      BY <ref id=1b00a9e/>, CODE src/generator.rs: Generator::mark_threaded,
         CODE src/fixstd/builtin.rs: InlineLLVMMarkThreadedFunctionBody
    <3>2. 起点のオブジェクトは解放されていない。
      `Let(x, Llvm(gen, args), k)` は D7 の読む構文であり、読まれる値は各オペランドである。この節点は
      ある活性化 `a` の本体 `B(a)` の節点であり、(K1) と D12 より `B(a)` は D11 を満たす。`a` が辿った節点の列は
      `B(a)` の実行路 (D3) である (D21, D23)。**読みの直前の点は段内の点とは限らない** -- 読みは 6 種の
      素動作のどれでもないからである。D24 が「**読みの直前の点では、勘定は直前の段内の点のものである。**」
      と述べ、「**その点と直前の段内の点のあいだに素動作は 1 つも無いので、`H` も `Obl` も `held` (D34) も
      動かず、解放も起きない。**」と続けるので、`<1>0` がその直前の段内の点まで与える閉じていることが、
      読みの直前の点までの接頭へそのまま移る。よって (S-c) の接頭条件が揃い、(S-c) より、この位置で
      読みうる各オブジェクト -- 各オペランドの inhabited な boxed leaf が指すオブジェクト -- は、その
      読みの直前の点で解放されていない。
      BY (K1), <ref id=ca36627/>, <ref id=56c2068/>, <ref id=95427eb/> (S-c), <ref id=859cf84/>, <ref id=3d96eb8/>, <ref id=c232680/>, <ref id=ff5985d/>, <ref id=e3436e8/>, DEF 段の素動作と段内の点, <1>0, <3>1
    <3>3. QED
      D32 の (読み-2) よりこの走査が読むのは起点と、そこから到達できるオブジェクトである。読みの直前の
      点の直前の段内の点を `q` とすると、D24 の「**読みの直前の点では、勘定は直前の段内の点のもので
      ある。**」とその続きより、2 つの点のあいだに解放は起きないので、`<3>2` の結論は `q` でも成り立つ。
      **起点が `q` で生きていることは L2 (a-3) が与える** -- `<3>1` より起点はこの op のオペランドの
      inhabited な boxed leaf が指すオブジェクトであり、A26 の第 1 節が「D7 の読む構文がオブジェクトの
      **記憶域から読む**動作は、その節点が行うどの参照の**手放し**よりも前に起きる」と述べるので、
      その読みの点でこの節点はまだその値を手放しておらず、A5 よりその leaf が保持する参照は未処分である。
      `q` と読みの直前の点のあいだに素動作は無いので (D24)、その条件は `q` でも成り立つ。`<1>0` より `q` は
      解放について閉じているので、L2 (a-3) より起点は `q` で生きている。L2 (b) がそこから到達できる
      オブジェクトへ広げ、その結論は解放が起きないので読みの直前の点へそのまま移る。計数下でない
      オブジェクトは A8 より解放されない。
      BY <ref id=4f63121/>, <ref id=b6673ca/>, <ref id=fd95f12/>, <ref id=56c2068/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=1b00a9e/>, <ref id=881a063/>, DEF 段の素動作と段内の点, <1>0, <3>2
  <2>5. CASE (読み-3) 環境の読み。
    A17 (ii) より、環境が読むのはその時点で `E` が持つ参照が指すオブジェクトか、そこから到達できる
    オブジェクトである。A17 (iii) より環境が動くのは段と段のあいだなので、この点は D24 の時点であり、
    したがって段内の点である (DEF 段の素動作と段内の点)。前者を `o` とすると、L4 より `H(o) ≥ 1` で
    あり -- L4 の 4 つの仮定は (K0)、(K1)、(K2)、A20 であり、この命題の仮定に在る --、`<1>0` よりその
    時点は解放について閉じているので L2 (a-1) より `o` は生きている (グローバル状態なら A8 より解放
    されない)。後者へは L2 (b) が広げる。
    BY (K0), (K1), (K2), <ref id=680aaa9/>, <ref id=b6673ca/>, <ref id=c9e4cca/>, <ref id=ec8d1a0/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=881a063/>, <ref id=61b8f53/>, DEF 段の素動作と段内の点, <1>0
  <2>6. QED
    D32 は `ρ` の読みを (読み-1)、(読み-2)、(読み-3) の 3 つに尽くし、(読み-2) は 3 つの走査に尽きる。
    BY <ref id=1b00a9e/>, <2>1, <2>2, <2>3, <2>4, <2>5

<1>2. (R2) が成り立つ。
  <2>1. グローバル状態のオブジェクトは 1 度も解放されない。
    BY <ref id=b6673ca/>, <ref id=88a06de/>
  <2>2. 計数下のオブジェクト `o` が解放されるのは、`H(o)` が 0 になったときだけであり、それは `H(o)` を 0 に
        した段の中で起きる。
    BY <ref id=e3436e8/> (F)
  <2>3. `o` が解放された後、`o` への参照は 1 つも作られない。
    `o` を解放した段を `S_0` とする。`<1>0` より、`S_0` とそれ以後のどの段についても、その直前の時点まで
    `ρ` は閉じており、(K1) と D12 より本体はどれも D11 を満たすので、L2b (b) の (H1) と (H2) が揃う。
    **`S_0` の中で `o` が解放された後の段内の点**については、L2b (b) が、その段が作る参照の指す
    オブジェクトはその参照を作る動作の直前の点で解放されていないと言い、`o` はその点で解放されているので
    `o` ではない。**`S_0` より後の段**については、その段の直前の時点で `o` は解放されており (D25 が
    「生きている」を割り当てた素動作より後・解放する素動作より前と定めるので、解放は後の点まで続く)、
    同じく L2b (b) より、作られる参照が指すオブジェクトはその点で解放されていないので `o` ではない。
    BY (K1), <ref id=3d96eb8/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=ba46997/>, DEF 段の素動作と段内の点, <1>0, <2>2
  <2>4. `o` が解放された後、`H(o)` は 0 のままである。
    D8 より `H(o)` は `o` への処分されていない参照の総数である。解放の点でそれは 0 であり (`<2>2` と
    D24 の (F))、`<2>3` より新しい参照は作られず、無い参照は処分できないので減りもしない。
    BY <ref id=ec8d1a0/>, <ref id=e3436e8/> (F), <2>2, <2>3
  <2>5. QED
    `<2>4` より `H(o)` が 0 に**なる**ことは解放の後には無いので、`<2>2` より 2 度目の解放は無い。
    グローバル状態のオブジェクトについては `<2>1` が言明より強いことを言う。
    BY <2>1, <2>2, <2>4

<1>3. ASSUME  `ρ` は正常終了する実行 (D31) である
      PROVE   `ρ` の最後の時点で解放されていない計数下のオブジェクト (D26) は、環境が最後に持つ参照が
              指すオブジェクトから到達できるものに限る。すなわち (R3) が成り立つ。
  <2>1. DEFINE `T` == `ρ` の最後の時点で解放されていない計数下のオブジェクトのうち、その時点で `E` が持つ
        参照が指すオブジェクトから到達できる (D25) もの。指されているオブジェクト自身も `T` に入れる。
        DEFINE `S` == `ρ` の最後の時点で解放されていない計数下のオブジェクト全体から `T` を除いたもの。
    D25 が到達できることを定め、D26 が計数下のオブジェクトを定めるので、`T` と `S` はどちらも定まる。
    BY <ref id=0b850c9/>, <ref id=88a06de/>
  <2>2. `S` の各元 `o` について、`o` への参照を持つ生きているオブジェクト `o'` が在り、`o'` は `S` の
        元である。
    <3>1. `o` は解放されていないので、`<1>0` の第 2 の節の対偶より `H(o) ≥ 1` であり、L4 より
          `o` への処分されていない参照が在って、P28 (a) よりそれはちょうど 1 つの持ち手を持つ。
          L4 の 4 つの仮定は (K0)、(K1)、(K2)、A20 であり、この命題の仮定に在る。
      BY (K0), (K1), (K2), <ref id=680aaa9/>, <ref id=ec8d1a0/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=61b8f53/>, <ref id=0d151d9/>, <1>0, <2>1
    <3>2. その持ち手は生きている活性化ではない。
      L5 (c) より、正常終了する実行の最後の時点で持ち手は生きているオブジェクトか環境だけである。L5 の
      (J0) は (K0) が、(J1) は (K1) と D12 が、(J2) はこの段の仮定が、(J3) は (K2) が、A20 はこの命題の
      仮定が与える。
      BY (K0), (K1), (K2), <ref id=680aaa9/>, <ref id=3d96eb8/>, <ref id=252ad5a/>, <2>1
    <3>3. その持ち手は環境ではない。
      環境が `o` への参照を持てば、`<2>1` の `T` の定義より `o ∈ T` であり、同じ定義より `o ∈ S` と
      両立しない。
      BY <2>1, <3>1
    <3>4. よって持ち手は生きているオブジェクト `o'` である。
      BY <ref id=0b850c9/>, <3>1, <3>2, <3>3
    <3>5. `o'` は計数下のオブジェクトである。
      `o'` がグローバル状態なら、A18 (b) より `o'` は計数下のオブジェクトへの参照を持たない。`o` は
      計数下のオブジェクトである。
      BY <ref id=5f74a79/>, <2>1, <3>4
    <3>6. `o'` は `T` の元ではない。
      `o'` が `T` の元なら、`E` が持つ参照が指すオブジェクトから `o'` へ到達でき、`o'` は `o` への参照を
      持つので `o` へも到達でき、`o ∈ T` となる。`<2>1` より `o ∈ S` と両立しない。
      BY <ref id=0b850c9/>, <2>1, <3>4
    <3>7. QED
      D25 より `o'` は生きているオブジェクト、すなわち割り当てられていて解放されていないオブジェクトで
      ある (`<3>4`)。計数下のオブジェクトであり (`<3>5`)、`T` の元ではない (`<3>6`)。よって `<2>1` より
      `o' ∈ S` である。
      BY <ref id=0b850c9/>, <2>1, <3>4, <3>5, <3>6
  <2>3. `S` は空である。
    `S` が空でないとする。L5 (d) より `ρ` に現れるオブジェクトは有限個なので、`S` は有限集合である。
    `<2>2` より `S` の各元は `S` の中に「自分を指す元」を持つので、`S` の元 `o_0` から `o_1`(`o_0` を
    指す)、`o_2`(`o_1` を指す)、… と `S` の中を限りなく遡れる。`S` が有限なのでこの列には同じ元が 2 度
    現れ、その間が閉路になる。A18 (a) がそれを禁じるので、`S` は空である。L5 の (J0) は (K0) が、
    (J1) は (K1) と D12 が、(J2) はこの段の仮定が、(J3) は (K2) が、A20 はこの命題の仮定が与える。
    BY (K0), (K1), (K2), <ref id=680aaa9/>, <ref id=5f74a79/>, <ref id=3d96eb8/>, <ref id=252ad5a/>, <2>2
  <2>4. QED
    `<2>3` より、最後の時点で解放されていない計数下のオブジェクトはすべて `T` の元である。`<2>1` の `T` の
    定義がそのまま言明である。
    BY <2>1, <2>3

<1>4. QED
  BY <1>1, <1>2, <1>3

### 系 (環境が何も持たずに終わる実行)

**言明**。P27 の前提の下で、`ρ` が正常終了し、かつ `ρ` の最後の時点で環境が参照を 1 つも持たないならば、
グローバル状態 (D26) になったことのないオブジェクトはすべてちょうど 1 回解放される。

<1>1. 最後の時点で解放されていない計数下のオブジェクトは無い。
  環境が参照を 1 つも持たなければ、P27 の (R3) が言う「環境が最後に持つ参照が指すオブジェクトから到達
  できるもの」は空である。
  BY <ref id=0b850c9/>, <ref id=1e3699d/> (R3)

<1>2. QED
  グローバル状態になったことのないオブジェクトは最後の時点でも計数下である (D26 より逆向きの遷移は無い)
  ので、`<1>1` よりどれも解放されている。P27 の (R2) よりどのオブジェクトも高々 1 回しか解放されないので、
  それらはちょうど 1 回ずつ解放されている。
  BY <ref id=88a06de/>, <ref id=1e3699d/> (R2), <1>1

## 5. どの仮定が何を支えているか

| 主張 | 直に引く項目 | ほかに引く命題 |
|---|---|---|
| L0d | A13、A22、A26a、D22、D24、`EXT LLVM モジュールの記号名` | -- |
| L0b | A22、D23 | L0d。(N1)-(N3) は仮定として持つ |
| L0c | A13、D24。(N0) は仮定として持つ | -- |
| P29 | A6、A11、A13、A22 (L0b の (N1)-(N3) を果たす) | L0b、L0c |
| L0 | A6、A10、A11、A13、A21、A22、A24、D2、D14、D23 と、P1・P9・P12・P24 の言明 | L0b、L0c、L0d |
| P30 | -- | L0 が引くもののすべて |
| L0a | A3、D24 | -- |
| L1 (a) | A12、A14、D4、D5、D9、D10、D14、D16、D23、D24 | -- |
| L1 (b) | A12、D9、D10、D16、D24 | L0a |
| L2 | A5、A8、A17、D7、D8、D10、D11a、D24、D25、D26 | -- |
| L2b | A3、A4、A5、A8、A16、A17、A26、D3、D7、D8、D9、D10、D11 の (S-a) と (S-c)、D11a、D16、D21、D22、D23、D24、D25、D26、D32 | L0a、L1 (b)、L2 |
| L3 | A5、A17、D8、D11a、D23、D24、D25、D26 | L2b |
| L4 | A5、A20、D8、D24、D25、D26 と、P28 (a) の言明 | -- |
| L5 | A17、A20、D3、D11 の (S-b)、D12、D21、D23、D24、D25、D30、D31 と、P28 (a) の言明 | -- |
| (R1) | A5、A8、A17、A20、A26、D3、D7、D8、D11 の (S-c)、D11a、D12、D21、D23、D24、D25、D32 | L2、L3 (`<1>0` 経由)、L4 |
| (R2) | A8、D8、D12、D24、D25、D26 | L2b、L3 (`<1>0` 経由) |
| (R3) | A18、A20、D8、D12、D24、D25 と、P28 (a) の言明 | L3 (`<1>0` 経由)、L4、L5 |
| 系 | D25、D26 | P27 |

**この表は `BY` の行から作る。**行ごとに、その命題の証明の `BY` が挙げる `A<n>`・`D<n>`・`P<n>` を
集めたものが第 2 欄、`L<n>` を集めたものが第 3 欄である。**手で書き足すと落ちる** -- 段を 1 つ足すたびに、
その `BY` が挙げる項目を全部この表に当て直す要があり、当て直しは目では尽きない。

**第 3 欄の命題が引くものは第 2 欄に写していない。**たとえば L3 は L2b を通じて A3・A4・A16・A26 を
読み、(R2) は L3 と L2b を通じて同じものを読む。行を読むときは第 3 欄を辿る。

**(R2) は L3 が使うもののほかに仮定を足さない。**D11 の (S-c) が `Retain(v, π)` と `Release(v, π)` の
触れる先を扱うので、L2b の `<1>1b` がそれを (S-c) から直接に出す。`report.md` の第 6 節の
「(S-c) を強めた記録」が、その節が弾く本体を述べる。(R2) の証明は P28 を引かない -- 引くのは L3 と L2b だけである。

**L4 を引くのは P27 の 3 段である。**`<1>1` の `<2>3` `<3>2` と `<2>5`、`<1>3` の `<2>2` `<3>1` で <!--#5739b3f-->
あり、P27 の言明はそこで要る 4 つの前提 -- (K0)、(K1)、(K2)、A20 -- を持つ。

L0、L0c、L0d、L1 (a) は P27 の証明のどのステップからも引かれない。L0 が支えているのは D23 の読み -- D9 と
D10 の「呼び出し先」を実行時の関数と読むこと -- が、`cancel` が静的に計算するものと食い違わない、という
ことである。食い違えば、D11 を保存すると称する P14 と P23 が P27 の使う述語とは別の述語を保存している
ことになる。P29 は同じことを `borrow_ify` の入力について述べ、層 1 の P7 の `App` の場合がそれを読む。
L0c は P29 と L0 の両方が使う名前の性質を、L0d は両方が使うコード生成の性質を、それぞれ括り出したもので
ある。L1 (a) は D24 の (E3) と D10 の初期値が同じ多重集合を指すことの検算である。

**L1 (b) は P27 の側に届く。**L2b の `Llvm` の場合がそれを読み、L3 と (R2) はその道で A12 を読む。 <!--#8259a3c-->
L1 (a) が読む A14 はその道に入らない -- L2b が引くのは (b) だけである。

L0a は P27 の証明から L2b を通じて引かれる -- 直に `<1>5` の `<2>0` が、L1 (b) の `<2>3a` を経て
`<2>2` の `<3>0a` が引く。支えているのは、活性化を作る段が (E1)、(E3)、(E7) の 3 つに尽きないこと --
オペランドを適用する `Llvm` の段も作ること -- と、その段が作った活性化が終わるときの参照の行き先である。

**P28 の 4 つの前提のうち 3 つは P27 の仮定に置いた。**D12 は P27 自身の仮定であり、残る 3 つ -- `P` が
`insert_rc` の入力から `cancel` の出力までのどこかに現れること、借用する unit を持つ本体の活性化を作る段が
(E3) に限られること、A20 -- は仮定として置く。`borrow_ify` の出力について前二者を果たすのは P14b であり、
その範囲は `cancel` の出力を含む。P27 の言明を D12 だけの形にするには、P28 (a) を残る 3 つに依らない形で
示す道が要る。
