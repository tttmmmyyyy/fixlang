# P27 -- 実行の合成

この文書は `README.md` の P27 を証明する。対象コミットは
`10bb7e118ce88fbed8b66d20a7b8d8fea2e82e92` である。

**実行のモデルは `README.md` が持つ。**環境 (D22)、活性化 (D23)、実行 (D24)、参照の持ち手 (D25)、
計数下のオブジェクトとグローバル状態のオブジェクト (D26)、実行の終わり方 (D31)、実行の読み (D32) は、
`README.md` の第 3.6 節が定める。この文書はそれを引くだけで、実行のモデルを自分では置かない。この文書が
自分で置くのは、局所定義「勘定が合っている時点」と補題 L0、L0a、L1-L5 だけである。

立つのは `README.md` の定義 D1-D26、D29、D31、D32 と、下の「どの仮定が何を支えているか」の節が挙げる
仮定である。P1-P26 のうち引くのは L0 の中の P9 の**言明**だけであり、P27 の証明はどの命題も引かない --
P27 が使うのは D11 と D12 という述語そのものであって、それを `borrow_ify` と `cancel` が保存すること
ではないからである。

## 1. 結論

| 主張 | 結果 |
|---|---|
| (R1) 解放されたオブジェクトの読みが起きない | 証明した |
| (R2) どのオブジェクトも高々 1 回しか解放されない | 証明した |
| (R3) 正常終了する実行で解放されずに残る計数下のオブジェクトは、環境が持つ参照が指すオブジェクトから到達できるものに限る | 証明した |

**(R2) は仮定を 1 つも足さずに閉じる。**この文書はかつて「`Retain` は解放されたオブジェクトに触れない」を
仮定として要求していた。その穴は D11 の (S-c) が `Retain(v, π)` と `Release(v, π)` の**触れる**先を扱う
ようになったことで閉じ、L2b の `Retain` の場合が (S-c) から直接に出る。

**L0 の数え上げは済んだ。**「実行時の関数の値が持つ LLVM 関数の番地をどこが書くか」は
`llvmgen-function-values.md` が `impl LLVMGen for` の実装 78 個すべてとそれらが呼ぶヘルパを読んで
数え上げており、L0 の `<2>4b` はそれを引く。

**D24 について開いているものが 2 つある。**「残っているもの」の節が、README に足す文をどちらについても
述べる。

1 つは**引用の穴**である。(E4) は、オペランドを適用する `Llvm` の段が作った活性化が終わるとき参照が
どこへ入るかを述べない。L0a はそれを (E2) の `Llvm` の段の段落の「(E3) と同じ形」という文から出すが、
同じ (E2) の生成の表がその leaf について `H` を 1 上げると読める。

もう 1 つは**モデルとコードのずれ**である。(F) の解放は「`o` が持つ参照をすべて処分し、それから `o` の
記憶域を返すこと」と定められているが、`Std::FFI::Destructor` のオブジェクトの解放は、参照を処分する前に
デストラクタ関数を適用する -- すなわち活性化を作る。この文書の L3 の `<3>4` は D24 の (F) をそのまま
引くので、その活性化はこの証明のモデルの外に落ちている。

## 2. 補題

### L0 (静的に決めた呼び出し先と実行時の呼び出し先)

**言明**。`borrow_ify` の出力プログラムのどの `App(callee, args)` についても、`resolve_callee_params` が
`callee` に答えるパラメータの列は、その呼び出しの実行時の呼び出し先 (D23) のパラメータの列であり、その
呼び出し先の `borrowed_units` は `resolve_callee_params` の答えから読まれる所有と一致する。

<1>1. `resolve_callee_params(callee, vars, prog)` が `Some(params)` を返すのは 2 つの場合であり、
      `None` を返すのはそれ以外である。
  1 つは `vars.closure_targets` が `callee.name` を持つ場合、もう 1 つは `prog.funcs` が
  `FuncRef { name: callee.name }` を持つ場合である。どちらでもなければ `?` が `None` を返す。
  BY CODE src/rc_ir/ownership.rs: resolve_callee_params

<1>2. `vars.closure_targets` が `callee.name` を持つのは、この本体に `Let(callee, Closure(fref, caps), k)`
      が在るとき、かつそのときに限る。そのとき返るのは `fref` のパラメータである。
  `collect_bindings` が `closure_targets` に挿入するのは `RcRhs::Closure(fref, _)` の腕の 1 か所だけで
  ある。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Closure` の場合,
     resolve_callee_params

<1>3. `<1>2` の場合、実行時の呼び出し先は `fref` である。
  A6 と A11 より `callee` の値はその `Let` が束縛した値であり、`Let(callee, Closure(fref, caps), k)` の
  値は funptr の欄に `fref` の関数を持つクロージャである。D23 より実行時の呼び出し先はその funptr の指す
  関数である。
  BY A6, A11, D23, CODE src/rc_ir/codegen.rs: Generator::build_rc_closure,
     CODE src/generator.rs: Generator::apply_lambda

<1>4. `prog.funcs` が `FuncRef { name: callee.name }` を持つ場合、実行時の呼び出し先はその関数である。
  <2>1. コード生成が `callee.name` の値として返すのは、`declared_globals` に `callee.name` で登録された
        値である。局所名でない名前について `get_scoped_value` は `get_or_declare_global` を通り、そこに
        無ければ `declare_program_global` を呼び、それも `None` を返せば panic する。
    BY CODE src/generator.rs: Generator::get_scoped_value, Generator::get_or_declare_global,
       CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner の
       `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕
  <2>2. `declared_globals` に名前を登録するのは `add_global_object` だけであり、それを呼ぶのは
        `declare_program_global` (非 funptr のグローバルについてそのアクセサ関数を登録する) と
        `declare_lambda_function` (`fn_ty.is_funptr()` のときその関数自身を登録する) の 2 か所である。
    BY CODE src/generator.rs: Generator::add_global_object, Generator::declare_program_global,
       Generator::declare_lambda_function
  <2>3. `callee.name` が `prog.funcs` の鍵であるとき、`declare_program_global` がそれを非 funptr の
        グローバルとして登録することは無い。
    `global_types` はプログラムのグローバル記号の型の表である
    (`CODE src/generator.rs: Generator` の `global_types` フィールドの doc)。lowering は funptr 型の記号を
    `prog.funcs` の関数にし、それ以外の型の記号をグローバル値の初期化子にするので、`prog.funcs` の鍵で
    ある記号の型は funptr である。`prog.funcs` の鍵のうち記号でないもの -- lowering が持ち上げた lambda に
    付ける名前 -- は `global_types` に無いので、`declare_program_global` は `None` を返す。
    BY CODE src/rc_ir/lower.rs: Lowerer::lower_symbol,
       CODE src/generator.rs: Generator::declare_program_global, Generator
  <2>4. QED
    `<2>2` と `<2>3` より、`callee.name` が登録されるのは funptr 型の関数として登録される場合だけである。
    `implement_rc_program` は `prog.funcs` の各関数を実装する前にその名前の LLVM 関数を 1 つ作り、
    `declare_lambda_function` が登録するのはその関数である。登録されなければ `<2>1` より panic する。
    D23 より実行時の呼び出し先はその funptr が指す関数である。
    BY D23, <2>1, <2>2, <2>3, CODE src/rc_ir/codegen.rs: Generator::implement_rc_program

<1>5. `resolve_callee_params` が `None` を返す場合、実行時の呼び出し先の `borrowed_units` は空である。
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
        あるときだけである。`borrow_versions` の鍵は `prog.funcs` の鍵なので、そのとき
        `resolve_callee_params` は `<1>1` の 2 つ目の場合に当たり `Some` を返す。
    BY <1>1, CODE src/rc_ir/borrow.rs: RewriteCtx::route, borrow_ify
  <2>4a. 借用版の名前が `RcVar` として現れるのは、`App` の `callee` の位置だけである。
    `borrow_ify` が出力の本体に借用版の名前を書くのは `route` の返り値だけであり、それは
    `RcRhs::App(callee, args)` の `callee` に置かれる。`clone_func` が導入するのは束縛変数の名前だけで
    あって関数の名前ではない (P9 の言明)。
    BY P9, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::route, borrow_ify
  <2>4b. 実行時の関数の値が持つ LLVM 関数の番地は、次の 3 つのいずれかが置いたものである。
    Fix の関数型の値は、クロージャなら `CLOSURE_FUNPTR_IDX` の欄に、funptr ならその値そのものとして、
    LLVM 関数の番地をちょうど 1 つ持つ (`CODE src/generator.rs: Generator::get_lambda_func_ptr` --
    `is_closure()` なら `CLOSURE_FUNPTR_IDX` の欄を、`is_funptr()` なら値そのものをポインタとして
    読み、どちらでもなければ panic する)。その番地を置くのは次の 3 か所である。

    - **(M1)** `Let(x, Closure(fref, caps), k)` を実行する段が置く `func_vals[fref]`
      (`CODE src/rc_ir/codegen.rs: Generator::build_rc_closure` -- `closure.insert_field(self,
      CLOSURE_FUNPTR_IDX, fn_ptr)` の `fn_ptr` は `func_vals[func]` である)。
    - **(M2)** funptr 型のグローバルを読む段が返す関数そのもの
      (`CODE src/generator.rs: ValueAccessor::get` -- `ty.is_funptr()` の枝は
      `fun.as_global_value().as_basic_value_enum()` を返し、アクセサ関数を呼ばない)。`<1>4` より
      その `fun` は `implement_rc_program` がその関数のために作った LLVM 関数である。
    - **(M3)** `Std::fix` の `Llvm` の op が置く `gc.current_function()`
      (`CODE src/fixstd/builtin.rs: InlineLLVMFixBody` -- `generate_tail` が
      `gc.current_function().as_global_value().as_pointer_value()` を `CLOSURE_FUNPTR_IDX` の欄に
      入れて `fix(f)` のクロージャを組み立てる)。

    ほかの構文と op は、既にある関数の値を写すだけである -- load、`extractvalue`、bit_cast、
    `apply_lambda` の戻り値、オペランドそのもの。値が配列の要素、構造体のフィールド、union の payload、
    グローバルの記憶域、FFI の境界を通っても、番地の中身はこの 3 か所のどれかが書いたものである。

    **この数え上げを行うのは `llvmgen-function-values.md` である。**同文書は `impl LLVMGen for` の実装
    78 個すべてとそれらが呼ぶヘルパを読み、関数の値を作らないもの 70 件、受け取った関数の値を写すだけの
    もの 7 件、自ら関数ポインタを作って Fix の関数型の値へ書き込むもの **1 件** に分類する。1 件は
    `InlineLLVMFixBody` であり、それが (M3) である。関数ポインタを作る残る 2 件
    (`InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody` と
    `InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody`) が書き込む先は `Std::Ptr` であって Fix の
    関数型ではない。(M2) は同文書が別に数える経路であり、**どの op の本体にも現れない** -- funptr 型の
    グローバルをオペランドに取る op は、その読み出しの時点で番地を得るのであって、自分では番地を作らない。
    BY <1>4, llvmgen-function-values.md,
       CODE src/generator.rs: Generator::get_lambda_func_ptr,
       CODE src/generator.rs: ValueAccessor::get,
       CODE src/rc_ir/codegen.rs: Generator::build_rc_closure,
       CODE src/fixstd/builtin.rs: InlineLLVMFixBody
  <2>4c. (M3) が置く `gc.current_function()` は、capture を持つ関数の LLVM 関数である。
    `InlineLLVMFixBody` は `#CAP` を自由変数の 1 つとして持つ
    (`CODE src/fixstd/builtin.rs: InlineLLVMFixBody` の `free_vars_mut`)。lowering は `Llvm` の op の
    各自由変数をオペランドに直し、スコープに解決できない局所名に出会うと panic する
    (`CODE src/rc_ir/lower.rs: Lowerer::lower_llvm`)。`#CAP` を束縛するのは
    `lower_lambda_as_function` の `lam_ty.is_closure()` の枝だけであり、その枝は `capture` が `Some` の
    `RcFunc` を作る (`CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function`)。よってこの op を
    含む本体の関数は capture を持ち、`implement_rc_function` がその本体を出す間
    `gc.current_function()` はその関数である
    (`CODE src/rc_ir/codegen.rs: Generator::implement_rc_function`)。
    BY CODE src/fixstd/builtin.rs: InlineLLVMFixBody, CODE src/rc_ir/lower.rs: Lowerer::lower_llvm,
       CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>5. QED
    `resolve_callee_params` が `None` を返すなら、`callee.name` は `prog.funcs` の鍵でも
    `closure_targets` の鍵でもない (`<1>1`)。`callee` の値の funptr は `<2>4b` の 3 つのいずれかで
    ある。(M1) の `fref` は `<2>3` より原本の名前であり、(M2) が読むグローバルの名前は `<2>4a` より
    借用版ではないので (借用版の名前は `App` の `callee` の位置にしか現れず、そこでは `<2>4` より
    `resolve_callee_params` が `Some` を返す) やはり原本であり、(M3) の関数は `<2>4c` より capture を
    持つので `<2>1` より借用版を持たない原本である。`<2>2` より原本の `borrowed_units` は空である。
    BY <1>1, <2>1, <2>2, <2>3, <2>4a, <2>4b, <2>4c

<1>6. QED
  `<1>2`-`<1>4` が `Some` の 2 つの場合を尽くし、そこでは `resolve_callee_params` の答えが実行時の
  呼び出し先のパラメータそのものである。`None` の場合は `<1>5` より実行時の呼び出し先が何も借用せず、
  A7 が置く近似 (全パラメータの全 unit を所有) がそれと一致する。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, A7

**注**。この文書が D9 と D10 を読むときの「呼び出し先」は D23 が定める実行時の関数である。L0 は、その読みの
下での義務集合が、`cancel` と `borrow_ify` が静的に計算する消費と食い違わないことを述べる。食い違えば、
D11 を保存すると称する P14 と P23 が別の述語を保存していることになる。

### L0a (オペランドを適用する `Llvm` の段が作る活性化)

**言明**。`LLVMGen::applies_a_function_operand` が真を宣言する op を持つ節点
`Let(x, Llvm(gen, args), k)` の段 (以下**適用する `Llvm` の段**と呼ぶ) について、次が成り立つ。

- **(a)** その段は活性化 `b` を作り、その段を持つ活性化 `a` は `b` が終わるまで中断中である。
- **(b)** その段で `Obl(a)` を離れて `b` へ渡る参照の多重集合は `Obl(b)` の初期値に一致し、渡す間
  `H` は変わらない。
- **(c)** `b` が終わる (E4) の段で `Obl(b)` を離れる参照は `Obl(a)` に入り、その段で `H` は変わらない。

<1>1. (a) が成り立つ。
  D24 の (E2) の `Llvm` の段の段落が「その op の生成コードがオペランドを関数として適用するとき
  (`LLVMGen::applies_a_function_operand` が真を宣言する op)、適用された関数の本体の活性化が作られ、
  `a` はそれが終わるまで中断中である」と述べる。どの op がこれを宣言するかは op 自身が答え、宣言せずに
  関数を適用した op は開発モードで落ちる (`CODE src/ast/inline_llvm.rs:
  LLVMGen::applies_a_function_operand`, `CODE src/generator.rs: Generator::apply_lambda` --
  先頭の `assert!(declares, ...)`)。
  BY D24 (E2 の `Llvm` の段の段落), CODE src/ast/inline_llvm.rs: LLVMGen::applies_a_function_operand,
     CODE src/generator.rs: Generator::apply_lambda

<1>2. (b) と (c) が成り立つ。
  同じ段落が続けて「(E3) と同じ形であり、違うのは呼び出し先を決めるのが `callee` の値ではなく op の
  生成コードだということだけである」と述べる。よって (E3) と (E4) の規則がそのまま当たる。(E3) は
  「消費する各参照が `Obl(a)` を離れ、呼び出し先 (D23) の本体の新しい活性化 `b` が作られて、それらの
  参照が `Obl(b)` の初期値になる」と述べ、(E4) は「`Obl(b)` を離れ、`b` を作った (E3) の呼び出し元 `a`
  の `Obl(a)` に入る」と述べる。どちらの段でも参照は処分も生成もされないので、`H` は変わらない。
  BY D24 (E2 の `Llvm` の段の段落), D24 (E3), D24 (E4)

<1>3. QED
  BY <1>1, <1>2

**注 (D24 の 2 か所が食い違う 1 点)**。(c) が言う「`H` は変わらない」は、D24 の (E2) の生成の表の `App`
の行が「**変わらない**。(E4) が呼び出し先から受け取る参照である」と述べるのと同じことである。同じ表の
`Llvm` の行は結果の leaf の `H` を A3 の宣言から読み、単一の `Unknown` の宣言には `+1` を当てる。適用した
活性化が返した参照を持つ leaf について、その 2 つは食い違う。**どちらが正しいかは D24 自身が述べている**
-- `App` の行の直後の段落が「呼び出しの結果の leaf が持つ参照は、呼び出し先の中で作られて (E4) で渡って
くるものであり、返りの段で新しく作られるのではない。1 つの本体だけを見る証明ではこの差は現れないが、
実行の水準では `H` の収支が合わなくなるので、ここで正す」と述べ、この理由は適用する `Llvm` の段にその
まま当たる。第 5 節が、その行に同じ訂正を足すことを求める。

### L1 (呼び出しと返りの受け渡しが釣り合う)

**言明**。(a) (E3) の段で `Obl(a)` を離れる参照の多重集合は、`Obl(b)` の初期値に一致する。
(b) (E4) の段で `Obl(b)` を離れる参照の多重集合は、その段の行き先 -- `b` を (E3) か適用する `Llvm` の段
(L0a) が作ったならその親の `Obl(a)`、`b` を (E1) か (E7) が作ったなら `E` -- が得る参照の多重集合に
一致する。

<1>1. (E3) の段で `Obl(a)` を離れるのは、D9 の `App` の行が挙げる leaf の参照である。すなわち `callee` の
      inhabited な全 boxed leaf と、呼び出し先がその位置の unit を所有する (D14) 引数の inhabited な leaf で
      ある。
  BY D9, D24 (E3)

<1>2. `Obl(b)` の初期値は、`b` の本体の関数が所有する (D14) パラメータ・capture の unit の下の inhabited な
      各 leaf につき 1 つである。
  BY D10 (初期値), D24 (E3)

<1>3. `callee` の inhabited な boxed leaf は、`b` の capture パラメータの inhabited な boxed leaf と
      1 対 1 に対応し、その unit は所有される。
  <2>1. `callee` の型がクロージャのとき、`boxed_leaf_paths` はその capture の位置 1 つだけを leaf とする。
    BY D4 (規則 2)
  <2>2. `callee` の型がクロージャのとき、実行時の呼び出し先は `capture` を持ち、その値は `callee` の値の
        capture の欄そのものである。
    `apply_lambda` は `fun.ty.is_closure()` のとき `fun` の `CLOSURE_CAPTURE_IDX` の欄を最後の引数として
    渡し、`implement_rc_function` はその引数を `func.capture` の名前に束縛する。retain も release も
    挟まない。
    BY CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>3. capture パラメータの型は boxed であり、その `boxed_leaf_paths` も `rc_units` も 1 元である。
    lowering は capture 変数の型を `make_dynamic_object_ty()` とする。これは boxed なので D4 の規則 3 と
    D5 の `unit_step` の `is_box` の腕がどちらも自分自身 1 つを返す。
    BY D4, D5, CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function
  <2>4. capture の unit は所有される。
    `borrow_ify` は各 `func` について `param_capture_units(func, type_env)` を丸ごと `owned_units` に
    入れ、出力の各版の `borrowed_units` を `param_capture_units` から `owned_units` を引いたものとして
    書く。よって原本 `f_own` の `borrowed_units` は空である。capture を持つ関数には借用版が作られない
    (`func.capture.is_none()` が条件) ので、capture を持つ関数の版は原本だけである。D14 より
    `borrowed_units` に入らない unit は所有される。
    BY D14, CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units
  <2>5. `callee` の型が funptr のとき、`callee` は boxed leaf を持たず、実行時の呼び出し先は `capture` を
        持たない。
    funptr の型は `is_fully_unboxed` であって D4 の規則 1 で leaf を持たず、`apply_lambda` は
    `fun.ty.is_closure()` が偽のとき capture の引数を渡さない。D1 より `capture` が `Some` なのは
    クロージャ ABI の関数だけである。
    BY D1, D4, CODE src/generator.rs: Generator::apply_lambda, CODE src/rc_ir/ast.rs: RcFunc
  <2>6. QED
    `callee` の型はクロージャか funptr のどちらかである (`apply_lambda` の先頭の表明
    `fun.ty.is_closure() || fun.ty.is_funptr()`)。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, CODE src/generator.rs: Generator::apply_lambda

<1>4. 呼び出し先が所有する位置の引数の inhabited な leaf は、`b` の対応するパラメータの inhabited な leaf と
      1 対 1 に対応する。
  <2>1. `apply_lambda` は各引数の値をそのまま呼び出し先へ渡し、`implement_rc_function` はそれを対応する
        パラメータの名前に束縛する。retain も release も挟まない。
    BY CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>2. A12 より第 `i` 引数の型は呼び出し先の第 `i` パラメータの型に等しいので、両者の
        `boxed_leaf_paths` は同じ列であり、`<2>1` より同じ値の同じ leaf を指す。よって inhabited (D16) で
        あることも一致する。
    BY A12, A14, D4, D16, <2>1
  <2>3. QED
    D9 の `App` の行と D10 の初期値は、どちらも同じ所有の割り当て (D14) を同じ関数 -- D23 が定める実行時の
    呼び出し先 -- について読む。よって消費される引数の leaf と、初期 `Obl` に入るパラメータの leaf は、
    `<2>2` の対応の下で同じ集合である。
    BY D9, D10, D14, D23, <2>1, <2>2

<1>5. (a) が成り立つ。
  BY <1>1, <1>2, <1>3, <1>4

<1>6. (b) が成り立つ。
  <2>1. (E4) で `Obl(b)` を離れるのは `x` の inhabited な全 boxed leaf の参照である。
    BY D9 (終端の `Ret` の行), D24 (E4)
  <2>2. CASE `b` を (E3) が作った場合。
    `Obl(a)` が得るのは `App` の結果の各 boxed leaf につき 1 つである (D10 の生成の `App` の行、
    D24 の (E2) の生成の表の `App` の行)。`apply_lambda` は呼び出し先が返した値をそのまま結果とするので、
    結果の値は `x` の値であり、A12 より型が一致するので `boxed_leaf_paths` の列も inhabited であることも
    一致する。よって `<2>1` の多重集合と等しい。
    BY A12, D10, D16, D24 (E2), D24 (E4), <2>1, CODE src/generator.rs: Generator::apply_lambda
  <2>3. CASE `b` を (E1) か (E7) が作った場合。
    D24 の (E4) と (E7) より、`Obl(b)` を離れる参照はそのまま `E` に入る。行き先が 1 つで、参照は処分も
    生成もされないので、`E` が得る多重集合は `<2>1` のそれと等しい。
    BY D24 (E4), D24 (E7), <2>1
  <2>3a. CASE `b` を適用する `Llvm` の段が作った場合。
    L0a (c) より、`Obl(b)` を離れる参照はそのままその段を持つ活性化の `Obl(a)` に入る。行き先が 1 つで、
    参照は処分も生成もされないので、`Obl(a)` が得る多重集合は `<2>1` のそれと等しい。
    BY L0a, <2>1
  <2>4. QED
    D24 の活性化の林より、活性化を作るのは (E1)、(E3)、(E7)、および (E2) のうち適用する `Llvm` の段の
    4 つだけである。
    BY D24, <2>2, <2>3, <2>3a

<1>7. QED
  BY <1>5, <1>6

**DEF 勘定が合っている時点**
実行 `ρ` の 1 つの時点 (段と段の間) が**勘定が合っている**とは、次の 3 つが成り立つことをいう。

- **(i)** 処分されていない各参照はちょうど 1 つの持ち手 (D25) を持つ。
- **(ii)** 各計数下のオブジェクト `o` について、`H(o)` は `o` への処分されていない参照の総数に等しい。すなわち
  `H(o) = Σ_{生きている活性化 a} Obl(a)[o] + Σ_{生きているオブジェクト o'} R(o')[o] + E[o]` である。ここで
  `R(o')[o]` は、`o'` が保持する値の inhabited な boxed leaf のうち `o` を指すものの個数である。
- **(iii)** 計数下のオブジェクト `o` が解放されているのは `H(o) = 0` のとき、かつそのときに限る。

L3 が、`ρ` のすべての時点が勘定が合っていることを示す。L2 と L2b はその帰納の 1 段の中で使うので、
「勘定が合っている時点では」の形で述べる。

### L2 (到達できるオブジェクトは解放されていない)

**言明**。勘定が合っている時点において、オブジェクト `o'` が解放されておらず、`o'` から `o''` へ
到達できる (D25) ならば、`o''` も解放されていない。`o'` と `o''` は計数下でもグローバル状態でもよい。

<1>1. `o'` が解放されておらず、`o'` が `o` への参照を持つならば、`o` は解放されていない。
  `o` がグローバル状態ならば A8 より解放されない。`o` が計数下ならば、`o'` は生きているオブジェクトなので
  D25 の持ち手であり (D26 は D8 の参照を計数下のオブジェクトへの参照に限るが、持ち手の側には制限を置かない
  ので `o'` の状態を問わない)、(ii) より `H(o) ≥ 1` であり、(iii) より `o` は解放されていない。
  BY A8, D25, D26, DEF 勘定が合っている時点

<1>2. QED
  到達の道の長さについての帰納。長さ 0 のとき `o'' = o'` であり仮定そのものである。長さ `n+1` の道は、
  `o'` が持つ参照が指すオブジェクト `o1` への 1 歩と、`o1` から `o''` への長さ `n` の道に分かれる。
  `<1>1` より `o1` は解放されておらず、帰納法の仮定より `o''` も解放されていない。
  BY D25, <1>1

### L2b (参照が作られるオブジェクトは解放されていない)

**言明**。時点 `t` が勘定が合っており、`t` の直後の段が新しい参照を作るならば、その参照が指すオブジェクトは
`t` において解放されていない。ただしその段を持つ活性化の本体は D11 を満たすとする。

<1>1. 新しい参照を作る段は (E2) だけであり、作る構文は D24 の (E2) の生成の表の 6 行である。
  (E1)、(E3)、(E4) は持ち手を移すだけ、(E5) は印を付けるだけ、(E7) は `Obl` が空の活性化を作るだけで
  あり、(E6) の後に段は無い。段の中で起きる解放 (F) は参照を処分するだけで作らない。表の 6 行は、
  `Retain` の行、`Llvm` の行、boxed 容器の `Destructure` の名前付きフィールドの行、boxed union の変位
  アームの payload の行、`Closure` の行、`App` の行である。
  BY D24

<1>1a. `t` の直後の段の節点が D7 の読む構文であるとき、その構文が読みうるオブジェクト -- 名指された値の
       inhabited な各 boxed leaf が指すオブジェクト -- と、そこから到達できる (D25) オブジェクトは、
       `t` において解放されていない。
  段を持つ活性化を `a` とすると、仮定より `B(a)` は D11 を満たし、`a` が辿った節点の列は `B(a)` の
  実行路 (D3) である (D21, D23)。(S-c) が読みうる各オブジェクトについて言明を与え、L2 がそこから
  到達できるオブジェクトへ広げる。グローバル状態のオブジェクトは A8 より解放されない。
  BY A8, D3, D7, D11 (S-c), D21, D23, D25, L2

<1>1b. `t` の直後の段の節点が `Retain(v, π, s, k)` であるとき、`π` の下の inhabited な各 leaf が指す
       オブジェクトは、`t` において解放されていない。
  D7 より `Retain(v, π)` が**触れる**のはちょうどそれらのオブジェクトである。段を持つ活性化を `a` と
  すると、仮定より `B(a)` は D11 を満たし、`a` が辿った節点の列は `B(a)` の実行路 (D3) である
  (D21, D23)。D11 の (S-c) は、読む構文が読みうるオブジェクトに加えて `Retain(v, π)` と
  `Release(v, π)` が触れる各オブジェクトについても、その時点で解放されていないことを言う。
  BY D3, D7, D11 (S-c), D21, D23

<1>2. CASE `Retain(v, π, s, k)` の行。
  D10 の `Retain` の行より、この段が作る参照が指すのは `π` の下の inhabited な各 leaf `λ` の
  `obj(v, λ)` である。`<1>1b` がそれらについて言明を与える。
  BY D10, D24 (E2), <1>1b

<1>3. CASE `Closure(f, caps)` の結果の行。
  この段が新しく割り当てる capture object である (D24 (E2) の生成の表)。割り当てたばかりのオブジェクトは
  `H = 1` なので、(iii) より解放されていない。`caps` が空のときは参照が生じないので、言明は空虚に
  成り立つ。
  BY D24 (E2), DEF 勘定が合っている時点

<1>4. CASE `App(callee, args)` の結果の行。
  この行は参照を作らない (D24 (E2) の生成の表の最終行)。よって言明は空虚に成り立つ。
  BY D24 (E2)

<1>5. CASE `Llvm` の行。
  <2>1. CASE 宣言が単一の `Fresh`。
    A3 より、生成コードはその leaf に新しく割り当てたオブジェクトへの新しい参照を置く。割り当てたばかりの
    オブジェクトは `H = 1` なので、(iii) より解放されていない。
    BY A3, DEF 勘定が合っている時点
  <2>2. CASE 宣言が単一の `Unknown`。
    <3>1. 参照が作られるオブジェクトは、この op のオペランドの inhabited な boxed leaf が指すオブジェクト
          から到達できるか、グローバル値が到達するオブジェクトである。
      A3 の `Unknown` の行がそう述べており、コードの側の doc も同じことを言う
      (`CODE src/rc_ir/provenance.rs: LeafOrigin` -- `Unknown` は「boxed な容器から、グローバルから、
      または `Retain` が複製して」得た値の出どころである)。op が読める boxed な容器はそのオペランドの
      leaf が指すオブジェクトか、そこから到達できるオブジェクトである。
      BY A3, D25, CODE src/rc_ir/provenance.rs: LeafOrigin
    <3>1a. 適用する `Llvm` の段 (L0a) の結果の leaf のうち、適用した活性化 `b` が返した参照を持つ
           ものについて、この段は新しい参照を作らない。よってそれらの leaf はこの補題の言明の範囲外で
           ある。
      L0a (c) より、その参照は `b` の中で作られて (E4) で `Obl(a)` に入るものであり、この段が新しく
      作るのではない。
      BY L0a
    <3>2. QED
      `Let(x, Llvm(gen, args), k)` は D7 の読む構文であり、読まれる値は各オペランドである。`<1>1a` が
      オペランドの leaf の指すオブジェクトとそこから到達できるオブジェクトについて言明を与え、
      グローバル値が到達するオブジェクトは A8 より解放されない。`<3>1a` が除いた leaf について、この段は
      新しい参照を作らない。
      BY A8, D7, <1>1a, <3>1, <3>1a
  <2>3. CASE 宣言が空集合。
    A3 より、生成コードはその leaf に何も置かず、その leaf は inhabited にならない。D10 の生成は
    inhabited な leaf についてだけ参照を作るので、参照は生じない。よって言明は空虚に成り立つ。
    BY A3, D10
  <2>4. CASE 宣言が複数の元を持つ。
    A3 の数え上げより、複数の元を宣言する op はこのコミットのプログラムには存在しない。よってこの場合は
    起きない。
    BY A3
  <2>5. QED
    A3 の表は宣言を、空集合、単一の `Arg(j, σ)`、単一の `Fresh`、単一の `Unknown`、複数の元の 5 つに
    尽くす。単一の `Arg(j, σ)` の leaf は D10 の生成の表から外れており (「宣言が単一の `Arg(j, σ)`
    **でない**もの」)、参照を作らない。残る 4 つが `<2>1`-`<2>4` である。
    BY A3, D10, <2>1, <2>2, <2>3, <2>4

<1>6. CASE boxed 容器の `Destructure` の名前付きフィールドの行、および boxed union の変位アームの
      payload の行。
  <2>1. boxed 容器の `Destructure` について、参照が作られるオブジェクトは容器 `c` が指すオブジェクトから
        到達できる。
    `get_struct_fields` の boxed の枝は、各フィールドを retain してから容器を release する。retain される
    フィールドは容器のオブジェクトが保持する値の boxed leaf であり、D25 よりそのオブジェクトは容器の
    オブジェクトが持つ参照が指す先である。
    BY D25, CODE src/object.rs: ObjectFieldType::get_struct_fields
  <2>2. boxed union の変位アームについて、参照が作られるオブジェクトは scrutinee が指すオブジェクトから
        到達できる。
    <3>1. この段が選んだ変位アームの `tag` は `Some(t)` であり、`t` はその時点の scrutinee の値の実行時の
          タグに等しい。
      D21 より、活性化が選ぶのは `tag` が実行時のタグに等しいアームであり、そのようなアームが無いときだけ
      コード生成の振る舞いに従う。コード生成は最後のアームのブロックを switch の default とし、最後で
      ないアームの `tag` が `None` であればそこで panic する (`CODE src/rc_ir/codegen.rs:
      Generator::eval_rc_match` -- `else_bb` は `arm_bbs.last()`、非最終アームの tag は
      `expect("a non-final match arm must be a variant arm")` で取り出される)。よって catch-all アームが
      在ればそれが最後のアームであり、default に落ちる選択はその catch-all アームである。A16 より
      `arms` は catch-all アームを持つか実行時のタグに等しい `tag` を持つアームを持つので、変位アームが
      選ばれたのは、その `tag` が実行時のタグに等しいときである。
      BY A16, D21, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
    <3>2. QED
      D10 の生成の表より、この段が参照を作るのは payload の各 boxed leaf についてである。`<3>1` より
      その payload は変位 `t` のものであり、`t` は実行時のタグなので、その leaf は scrutinee の値の
      inhabited (D16) な boxed leaf である。D25 より、それが指すオブジェクトは scrutinee のオブジェクトが
      持つ参照の指す先である。
      BY D10, D16, D25, <3>1
  <2>3. QED
    `Destructure(c, fs, s, k)` と `Let(x, Match(v, arms), k)` はどちらも D7 の読む構文であり、読まれる値は
    それぞれ容器 `c` と scrutinee `v` である。`<1>1a` が言明を与える。
    BY D7, <1>1a, <2>1, <2>2

<1>7. QED
  `<1>2`、`<1>3`、`<1>4`、`<1>5`、`<1>6` は `<1>1` が挙げた 6 行を尽くす。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### L3 (実行のすべての時点は勘定が合っている)

**言明**。プログラム `P` のすべての本体が D11 を満たすとする。`P` の実行 `ρ` のすべての時点は、勘定が
合っている。

<1>1. `ρ` の最初の時点は勘定が合っている。
  最初の時点では活性化もオブジェクトも無い。オブジェクトが無いので、処分されていない参照も無く、(i) は
  空虚に成り立つ。計数下のオブジェクトが無いので (ii) と (iii) も空虚に成り立つ。
  BY D25, DEF 勘定が合っている時点

<1>2. 勘定が合っている時点の直後の 1 段の後の時点も、勘定が合っている。
  <2>1. CASE (E1) 環境が活性化を作る段。
    D24 の (E1) より、この段は作られる活性化 `a` の初期 `Obl` の各参照を `E` から渡し、A17 (i) より環境は
    それ以後それを持たない。持ち手が環境から `a` へ変わるだけなので (i) は保たれる。参照は処分も生成も
    されず `H` も動かないので、(ii) と (iii) も保たれる。
    BY A17, D24 (E1), D25, DEF 勘定が合っている時点
  <2>2. CASE (E2) 節点の段。
    <3>1. 移動 (D9) の節点では、`Obl(a)` も `H` も変わらず、参照の持ち手も変わらない。
      移動は「参照の持ち手が活性化の中で変わるだけ」であって義務集合を変えない。変わるのはどのスロットが
      その参照を指すかであり、D25 の持ち手 -- どの活性化か、どのオブジェクトか、環境か -- ではない。
      BY D9, D10 (移動), D25
    <3>2. `Obl(a)` を離れて行き先のある参照については、持ち手が `Obl(a)` からその行き先へ変わるだけで
          あり、参照は処分されず `H` も変わらない。
      BY D24 (E2 の表), D25
    <3>3. `Obl(a)` を離れて処分される参照については、その参照が 1 つ消え、`H` がちょうど 1 下がる。
      BY D24 (E2 の表)
    <3>4. `<3>3` の処分で `H` が 0 になった計数下のオブジェクト `o` については、この段の中で (F) の解放が
          起きる。解放は `o` が持つ参照をすべて処分し (各々について `H` がちょうど 1 下がる)、`o` を
          持ち手の集合から外し、その処分で `H` が 0 になったオブジェクトも同じように解放する。段の
          終わりには `H = 0` の計数下のオブジェクトはすべて解放されている。
      BY D24 (F)
    <3>4a. 節点が適用する `Llvm` の段 (L0a) であるとき、その段は活性化 `b` を作り、`Obl(a)` を離れて
           `b` へ渡る参照は `Obl(b)` の初期値になる。持ち手が `Obl(a)` から `Obl(b)` へ移るだけであり、
           参照は処分も生成もされず `H` も変わらない。`b` が終わる段は (E4) であり、この (E2) の段では
           ない。**その (E4) の段で `Obl(a)` に入る参照が結果の leaf に置かれる分は、この (E2) の段が
           新しく作る参照ではない。**
      BY L0a
    <3>5. 生成の節点では、生じる参照 1 つごとに、その持ち手は `Obl(a)` であり、`H` がちょうど 1 上がるか、
          新しいオブジェクトが `H = 1` で作られる。`<3>4a` が除いた leaf はここに数えない。
      BY D24 (E2 の生成の表), <3>4a
    <3>6. (i) が保たれる。
      `<3>1` は持ち手を変えず、`<3>2` と `<3>4a` は持ち手を 1 つから 1 つへ移し、`<3>3` と `<3>4` は参照を
      持ち手ごと消し、`<3>5` は持ち手がちょうど 1 つ (`Obl(a)`) の参照を足す。`<3>4` で持ち手の集合から
      外れる `o` が持っていた参照は、同じ `<3>4` ですべて処分される。
      BY <3>1, <3>2, <3>3, <3>4, <3>4a, <3>5
    <3>7. (ii) が保たれる。
      `<3>1` から `<3>5` のどの動きでも、処分されていない参照の個数の増減と `H` の増減が同じである。
      `<3>4` で持ち手の集合から外れる `o` について、`o` への処分されていない参照の個数は `H(o) = 0` に
      等しいので、`o` が外れても (ii) の等式は崩れない。
      BY <3>1, <3>2, <3>3, <3>4, <3>4a, <3>5
    <3>8. (iii) が保たれる。
      `H` が下がるのは `<3>3` と `<3>4` の場合だけであり、0 になったものは `<3>4` によりこの段の中で
      解放される。`H` が上がるのは `<3>5` の場合であり、この言明の仮定よりこの段を持つ活性化の本体は
      D11 を満たすので、L2b よりその対象は段の時点で解放されていない。よって段の終わりにも「`H = 0`」と
      「解放されている」は一致する。
      BY L2b, <3>3, <3>4, <3>4a, <3>5
    <3>9. QED
      BY <3>6, <3>7, <3>8
  <2>3. CASE (E3) 呼び出しの段。
    L1 (a) より、`Obl(a)` を離れる参照はそのまま `Obl(b)` の初期値になる。持ち手が `a` から `b` へ変わる
    だけで、参照は処分も生成もされず `H` も動かないので、3 つとも保たれる。
    BY L1, D24 (E3), D25
  <2>4. CASE (E4) 返りの段。
    <3>1. L1 (b) より、`Obl(b)` を離れる参照はそのまま行き先 -- `b` を (E3) か適用する `Llvm` の段
          (L0a) が作ったならその親の `Obl(a)`、`b` を (E1) か (E7) が作ったなら `E` -- に入る。
      BY L0a, L1, D24 (E4), D24 (E7)
    <3>2. この段で `b` は終わるが、その時点で `Obl(b)` に参照は残らない。
      仮定より `B(b)` は D11 を満たし、`b` が辿った節点の列は `B(b)` の実行路 (D3) なので (D21, D23)、
      (S-b) が「終端の `Ret` の消費を行った後の `Obl` は空である」を与える。
      BY D3, D11 (S-b), D21, D23
    <3>3. QED
      `<3>1` より持ち手が変わるだけであり、`<3>2` より持ち手を失う参照は無い。参照は処分も生成もされず
      `H` も動かないので、3 つとも保たれる。
      BY D25, <3>1, <3>2
  <2>5. CASE (E5) グローバル化の段。
    この段は印を付けるだけで、参照も `H` も動かさない。印の付いたオブジェクトは D26 より以後グローバル
    状態であり、(ii) と (iii) の勘定から外れる。外れる時点でそれらは解放されていないので、(iii) が偽に
    なることは無い。それらへの参照も D26 より勘定から外れるので、(i) と (ii) も保たれる。
    BY D24 (E5), D26
  <2>6. CASE (E6) 中断の段。
    この段の後に時点は無い。
    BY D24 (E6), D31
  <2>6a. CASE (E7) グローバルの初期化の段。
    D24 の (E7) より、この段は `Obl` が空の活性化を作るだけである。参照は処分も生成もされず、持ち手も
    `H` も動かないので、3 つとも保たれる。
    BY D24 (E7), D25, DEF 勘定が合っている時点
  <2>7. QED
    D24 は段を (E1)-(E7) の 7 つに尽くす。活性化を作る段のうち適用する `Llvm` の段は (E2) の一部で
    あり、`<2>2` の `<3>4a` がそれを扱う。(F) の解放は段ではないので独立した場合を持たず、`<2>2` が
    (E2) の中で扱う。参照を処分する段は (E2) だけだからである -- `<2>1`、`<2>3`、`<2>4`、`<2>5`、
    `<2>6`、`<2>6a` の段はどれも参照を処分しない。
    BY D24, <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>6a

<1>3. QED
  時点までの段の数についての帰納。
  BY <1>1, <1>2

### L4 (中断中の活性化の `Obl` は動かない)

**言明**。活性化 `a` が (E3)、(E7)、または適用する `Llvm` の段 (L0a) で活性化 `b` を作ってから、`b` が
終わって `a` が再開するまでの間、`Obl(a)` は変わらない。

<1>1. その間 `a` は段を持たない。
  D24 の (E3) と (E7) はどちらも「`a` は `b` が終わるまで中断中であり、その間 `a` は段を持たない」と
  述べ、適用する `Llvm` の段については L0a (a) が `a` は中断中であると述べる。D24 の活性化の林の段落が
  「子は親が中断中の間だけ段を持ち、親は子が終わってから再開する」と述べるので、中断中の `a` は段を
  持たない。
  BY D24 (E3), D24 (E7), D24 (活性化の林), L0a

<1>2. `Obl(a)` を変える段は 3 種である。`a` が段を持つ (E2) と (E3) の段、`a` 自身が終わる (E4) の段、
      そして `a` が作った活性化が終わる (E4) の段である。
  D24 の (E1) と (E7) は新しい活性化の `Obl` を作り、(E5) は印を付けるだけであり、(E6) の後に段は無い。
  (E2) と (E3) が動かすのは、その段を持つ活性化の `Obl` である。(E4) が動かすのは、終わる活性化の `Obl` と、
  参照が入る先 -- L1 (b) より、終わった活性化を (E3) か適用する `Llvm` の段が作ったならその親の `Obl`、
  (E1) か (E7) が作ったなら `E` -- である。段の中で起きる解放 (F) が処分するのは、オブジェクトが持つ
  参照だけである。
  BY D24, L1

<1>2a. その間、`a` 自身が終わる (E4) の段は無い。
  `a` 自身が終わる (E4) は `a` が `B(a)` の終端の `Ret` に着く段であり (D23)、それは `a` が段を持つときで
  ある。`<1>1` よりその間 `a` は段を持たない。
  BY D23, D24 (E4), <1>1

<1>3. その間に (E4) の段で終わる活性化のうち、参照を `Obl(a)` へ入れるものは無い。
  `<1>2` より、参照が `Obl(a)` へ入るのは、終わる活性化を `a` の (E3) の段か `a` の適用する `Llvm` の段が
  作った場合だけである。`a` がその 2 種の段を持つのは `a` が段を持つときであり、`<1>1` よりその間 `a` は
  段を持たないので、その間に `a` が新しい子を作ることは無い。`a` が `b` より前に作った子は、D24 の
  (E3)、D24 の (E7)、L0a (a) より、それぞれの中断が終わった時点ですでに終わっている。よってその間に
  終わりうる `a` の子は `b` だけであり、`b` の (E4) はこの区間の終わりである。
  BY D24 (E3), D24 (E7), L0a, <1>1, <1>2

<1>4. QED
  BY <1>1, <1>2, <1>2a, <1>3

**注 (借用した unit のオブジェクト)**。L4 が言うのは、`b` の活性化の間ずっと `a` が持ち続ける参照がある、
ということである。呼び出し先が**借用する** unit のオブジェクトが `b` の実行中に解放されないのはこれによる --
D9 の `App` の行は借用位置の引数の leaf を消費しないので、その参照は `Obl(a)` に残り、L3 の (ii)(iii) より
対象は解放されない。`Ownership::Borrow` の doc が「どちらの側も参照カウント操作を行わない」と言うのはこの
規律である (`CODE src/rc_ir/ast.rs: Ownership`)。(R1) の証明はこの経路を通らない -- (S-c) が `b` の本体に
ついて直接に読みの安全を与えるからである。L4 が支えているのは、その (S-c) が `borrow_ify` の出力について
実際に成り立ちうるということであって、P27 の段ではない。

### L5 (活性化の林についての帰納)

**言明**。プログラム `P` のすべての本体が D11 を満たすとし、`ρ` を `P` の正常終了する実行 (D31) とする。
このとき次が成り立つ。

- **(a)** `ρ` のすべての活性化が終わっている。
- **(b)** 各活性化 `a` が終わった時点で `Obl(a)` は空である。
- **(c)** `ρ` の最後の時点で、処分されていないどの参照の持ち手も、生きているオブジェクトか環境である。

**帰納法の仮定**。D24 の**活性化の林** -- (E1) が作る活性化を根、(E3) と (E7)、および (E2) のうち適用
する `Llvm` の段 (L0a) が作る活性化を、それを作った活性化の子とする -- の上の構造帰納であり、活性化 `a` に
ついての帰納法の仮定は「`a` が作ったすべての活性化が終わっている」である。D31 の後の注より 1 つの活性化が
作る活性化は有限個であり、正常終了する実行の段は有限個なので、この林は有限であって帰納は well-founded で
ある。

<1>1. (a) が成り立つ。
  D31 より、正常終了する実行の最後の時点で生きている活性化は無い。D23 より、生きている活性化とは始まって
  終わっていない活性化である。よって始まった活性化はすべて終わっている。
  BY D23, D31

<1>2. 活性化 `a` が終わるのは、`a` が作ったすべての活性化が終わった後である。
  `a` が (E3)、(E7)、または適用する `Llvm` の段で `b` を作ると、`a` は `b` が終わるまで中断中である
  (D24 (E3), D24 (E7), L0a (a))。D24 の活性化の林の段落が「子は親が中断中の間だけ段を持ち、親は子が
  終わってから再開する」と述べるので、その間 `a` は段を持たない。`a` が終わるのは `a` 自身の (E4) の段で
  あり、それは `a` が段を持つときである (D23)。
  BY D23, D24 (E3), D24 (E4), D24 (E7), D24 (活性化の林), L0a

<1>3. (b) が成り立つ。
  活性化 `a` が終わるのは `B(a)` の終端の `Ret` に着いてその消費を行うときである (D23)。`a` が辿った
  節点の列は `B(a)` の実行路 (D3) であり (D21, D23)、仮定より `B(a)` は D11 を満たすので、(S-b) が
  「終端の `Ret` の消費を行った後の `Obl` は空である」を与える。
  BY D3, D11 (S-b), D21, D23

<1>4. (c) が成り立つ。
  `<1>1` より最後の時点で生きている活性化は無いので、D25 が挙げる 3 つの持ち手のうち活性化は残らない。
  L3 の (i) より、処分されていない各参照はちょうど 1 つの持ち手を持つ。
  BY D25, L3, <1>1

<1>5. QED
  BY <1>1, <1>3, <1>4

**注**。`<1>2` は帰納が well-founded であることを支えるだけで、`<1>3` はそれを使わない -- (S-b) が各活性化に
ついて独立に成り立つからである。活性化の林が効いているのは `<1>1` と `<1>2`、すなわち「正常終了なら
すべての活性化が終わっている」の側である。

## 3. P27 の証明

**P27.** ASSUME  NEW `P`: RcProgram、
                 `P` は D12 の意味で RC 規律を満たす、
                 NEW `ρ`: `P` の実行 (D24)、
                 A17、A18
        PROVE   **(R1)** `ρ` のどの読み (D32) も、解放されたオブジェクトを読まない。
                **(R2)** `ρ` において、どのオブジェクトも高々 1 回しか解放されない。
                **(R3)** `ρ` が正常終了する実行 (D31) ならば、`ρ` の最後の時点で解放されていない計数下の
                オブジェクト (D26) は、環境が最後に持つ参照が指すオブジェクトから到達できるものに限る。

<1>1. (R1) が成り立つ。
  <2>1. CASE (読み-1) D7 の読む構文の実行。
    その構文はある活性化 `a` の本体 `B(a)` の節点であり、D12 より `B(a)` は D11 を満たす。`a` が辿った
    節点の列は `B(a)` の実行路 (D3) である (D21, D23)。(S-c) は「D7 の読む構文がその位置で読みうる
    各オブジェクトは、その時点で解放されていない」を言う。D32 の (読み-1) が読むのはちょうどその
    オブジェクトである。
    BY D3, D7, D11 (S-c), D12, D21, D23, D32
  <2>2. CASE (読み-2) (F) の解放の走査。
    D32 の (読み-2) より、この走査が読むのは解放されるオブジェクト `o` そのものである。D24 の (F) より、
    走査が走る間 `o` はまだ解放されていない。
    BY D24 (F), D32
  <2>3. CASE (読み-2) (E5) の `mark_global` の走査。
    <3>1. 走査の起点は、グローバル初期化子の活性化が返した値の inhabited な各 boxed leaf が指す
          オブジェクトである。
      (E5) の段は、グローバル初期化子の活性化が終わってその参照が `E` に入った直後に走る
      (D24 の (E4) と (E7))。
      BY D24 (E4), D24 (E5), D24 (E7),
         CODE src/rc_ir/codegen.rs: Generator::implement_rc_global
    <3>2. 起点のオブジェクトは解放されていない。
      `<3>1` の参照は `E` に在るので、L3 の (ii) より `H ≥ 1` であり、L3 の (iii) より解放されていない。
      グローバル状態のオブジェクトなら A8 より解放されない。
      BY A8, D25, L3, <3>1
    <3>3. QED
      D32 の (読み-2) よりこの走査が読むのは起点と、そこから到達できるオブジェクトである。L2 と L3 より
      それらも解放されていない。
      BY D32, L2, L3, <3>2
  <2>4. CASE (読み-2) `Std::mark_threaded` の走査。
    <3>1. `Std::mark_threaded` は `Llvm` の op であり、走査の起点はそのオペランドの inhabited な boxed
          leaf が指すオブジェクトである。
      BY D32, CODE src/generator.rs: Generator::mark_threaded,
         CODE src/fixstd/builtin.rs: InlineLLVMMarkThreadedFunctionBody
    <3>2. 起点のオブジェクトは解放されていない。
      `Let(x, Llvm(gen, args), k)` は D7 の読む構文であり、読まれる値は各オペランドである。この節点は
      ある活性化 `a` の本体 `B(a)` の節点であり、D12 より `B(a)` は D11 を満たす。`a` が辿った節点の列は
      `B(a)` の実行路 (D3) である (D21, D23)。(S-c) より、この位置で読みうる各オブジェクト --
      各オペランドの inhabited な boxed leaf が指すオブジェクト -- は、その時点で解放されていない。
      BY D3, D7, D11 (S-c), D12, D21, D23, <3>1
    <3>3. QED
      D32 の (読み-2) よりこの走査が読むのは起点と、そこから到達できるオブジェクトである。L2 と L3 より
      それらも解放されていない。
      BY D32, L2, L3, <3>2
  <2>5. CASE (読み-3) 環境の読み。
    A17 より、環境が読むのはその時点で `E` が持つ参照が指すオブジェクトか、そこから到達できる
    オブジェクトである。前者は L3 の (ii)(iii) より解放されておらず (グローバル状態なら A8 より解放
    されない)、後者は L2 と L3 より解放されていない。
    BY A8, A17, D25, L2, L3
  <2>6. QED
    D32 は `ρ` の読みを (読み-1)、(読み-2)、(読み-3) の 3 つに尽くし、(読み-2) は 3 つの走査に尽きる。
    BY D32, <2>1, <2>2, <2>3, <2>4, <2>5

<1>2. (R2) が成り立つ。
  <2>1. グローバル状態のオブジェクトは 1 度も解放されない。
    BY A8, D26
  <2>2. 計数下のオブジェクト `o` が解放されるのは、`H(o)` が 0 になったときだけであり、それは `H(o)` を 0 に
        した段の中で起きる。
    BY D24 (F)
  <2>3. `o` が解放された時点の後、`o` への参照は 1 つも作られない。
    D12 よりどの本体も D11 を満たすので、L3 よりすべての時点は勘定が合っており、L2b より新しい参照が
    作られるオブジェクトはその段の時点で解放されていない。`o` は解放された後ずっと解放されたままである
    (D24 に解放を取り消す段は無い)。
    BY D12, D24, L2b, L3
  <2>4. `o` が解放された時点の後、`H(o)` は 0 のままである。
    L3 の (ii) より `H(o)` は `o` への処分されていない参照の総数である。解放の時点でそれは 0 であり
    (`<2>2` と L3 の (iii))、`<2>3` より新しい参照は作られず、無い参照は処分できないので減りもしない。
    BY L3, <2>2, <2>3
  <2>5. QED
    `<2>4` より `o` の `H` が 0 に**なる**ことは解放の後には無いので、`<2>2` より 2 度目の解放は無い。
    グローバル状態のオブジェクトについては `<2>1` が言明より強いことを言う。
    BY <2>1, <2>2, <2>4

<1>3. (R3) が成り立つ。
  <2>1. DEFINE `T` == `ρ` の最後の時点で解放されていない計数下のオブジェクトのうち、その時点で `E` が持つ
        参照が指すオブジェクトから到達できる (D25) もの。指されているオブジェクト自身も `T` に入れる。
        DEFINE `S` == `ρ` の最後の時点で解放されていない計数下のオブジェクト全体から `T` を除いたもの。
  <2>2. `S` の各元 `o` について、`o` への参照を持つ生きているオブジェクト `o'` が在り、`o'` は `S` の
        元である。
    <3>1. `o` は解放されていないので、L3 の (iii) より `H(o) ≥ 1` であり、L3 の (i)(ii) より `o` への
          処分されていない参照が在って、それはちょうど 1 つの持ち手を持つ。
      BY L3, <2>1
    <3>2. その持ち手は生きている活性化ではない。
      L5 (c) より、正常終了する実行の最後の時点で持ち手は生きているオブジェクトか環境だけである。
      BY D12, L5, <2>1
    <3>3. その持ち手は環境ではない。
      環境が `o` への参照を持てば、`<2>1` の `T` の定義より `o ∈ T` であり、同じ定義より `o ∈ S` と
      両立しない。
      BY <2>1, <3>1
    <3>4. よって持ち手は生きているオブジェクト `o'` である。
      BY D25, <3>1, <3>2, <3>3
    <3>5. `o'` は計数下のオブジェクトである。
      `o'` がグローバル状態なら、A18 (b) より `o'` は計数下のオブジェクトへの参照を持たない。`o` は
      計数下のオブジェクトである。
      BY A18, <2>1, <3>4
    <3>6. `o'` は `T` の元ではない。
      `o'` が `T` の元なら、`E` が持つ参照が指すオブジェクトから `o'` へ到達でき、`o'` は `o` への参照を
      持つので `o` へも到達でき、`o ∈ T` となる。`<2>1` より `o ∈ S` と両立しない。
      BY D25, <2>1, <3>4
    <3>7. QED
      `o'` は解放されておらず (生きているオブジェクトだから)、計数下のオブジェクトであり (`<3>5`)、`T` の元では
      ない (`<3>6`)。よって `<2>1` より `o' ∈ S` である。
      BY <2>1, <3>4, <3>5, <3>6
  <2>3. `S` は空である。
    `S` が空でないとする。D31 より `ρ` が正常終了するなら `ρ` の段は有限個である。1 つの段が割り当てる
    オブジェクトは有限個である -- D24 の (E2) の生成の表で割り当てを行うのは `Closure` の行と `Llvm` の
    行の `Fresh` の場合だけであり、後者が割り当てるのは結果の型の boxed leaf ごとに 1 つで、A10 より
    `boxed_leaf_paths` は有限の列である。よって `ρ` に現れるオブジェクトは有限個であり、`S` は有限集合で
    ある。`<2>2` より `S` の各元は `S` の中に「自分を指す元」を持つので、`S` の元 `o_0` から
    `o_1`(`o_0` を指す)、`o_2`(`o_1` を指す)、… と `S` の中を限りなく遡れる。`S` が有限なのでこの列には
    同じ元が 2 度現れ、その間が閉路になる。A18 (a) がそれを禁じるので、`S` は空である。
    BY A10, A18, D24 (E2), D31, <2>2
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
  BY D25, P27 (R3)

<1>2. QED
  グローバル状態になったことのないオブジェクトは最後の時点でも計数下である (D26 より逆向きの遷移は無い)
  ので、`<1>1` よりどれも解放されている。P27 の (R2) よりどのオブジェクトも高々 1 回しか解放されないので、
  それらはちょうど 1 回ずつ解放されている。
  BY D26, P27 (R2), <1>1

## 4. どの仮定が何を支えているか

| 主張 | 使う仮定 |
|---|---|
| (R1) | D11 の (S-b) と (S-c)、A3 (D24 の生成の表と L2b を通じて)、A5 (D25 を通じて)、A8、A12、A14 (L1 を通じて)、A16 (アームがタグを尽くすこと。L2b の `<1>6` の boxed union の場合)、A17 |
| (R2) | (R1) が使うもののすべて |
| (R3) | (R2) が使うもののすべてと **A18 (a)(b)**、および A10 (オブジェクトが有限個であること) |
| 系 | (R3) が使うもののすべて |

**(R2) はもう仮定を足さない。**かつてこの文書は「`Retain` は解放されたオブジェクトに触れない」を仮定として
要求していた。D11 の (S-c) が `Retain(v, π)` と `Release(v, π)` の触れる先を扱うようになった後は、L2b の
`<1>1b` がそれを (S-c) から直接に出す。README の第 6 節「(S-c) を強めた記録」が、この強化がどの本体を
弾くようになったかを述べる。

L0 は P27 の証明のどのステップからも引かれない。L0 が支えているのは D23 の読み -- D9 と D10 の「呼び出し先」を
実行時の関数と読むこと -- が、`borrow_ify` と `cancel` が静的に計算するものと食い違わない、ということである。
食い違えば、D11 を保存すると称する P14 と P23 が P27 の使う述語とは別の述語を保存していることになる。

L0a は P27 の証明から L1、L3、L4、L5 を通じて引かれる。支えているのは、活性化を作る段が (E1)、(E3)、(E7)
の 3 つに尽きないこと -- オペランドを適用する `Llvm` の段も 1 つ作ること -- であり、その段の受け渡しが
(E3) と (E4) の受け渡しと同じ形であることである。

## 5. 残っているもの

D24 が活性化を作る段を 2 つ数え落としている。どちらも README に足す文の提案であり、1 つ目は L0a と L3 の
`<3>4a` が読む。

**(1) (E4) の行き先に、適用する `Llvm` の段が作った活性化の分が無い。** (E4) の本文は

> 活性化 `b` が終端の `Ret(x)` に着く。D9 の終端の `Ret` の行が消費する参照 -- `x` の inhabited な全
> boxed leaf の参照 -- が `Obl(b)` を離れ、`b` を作った (E3) の呼び出し元 `a` の `Obl(a)` に入る。`a` は
> `k` の位置で再開する。`b` を作ったのが環境 (E1) であれば、それらの参照は `E` に入る。

であり、(E7) の本文がその 2 つ目 (`E`) を自分の場合について繰り返す。適用する `Llvm` の段が作った活性化に
ついては、どちらの本文も行き先を述べない。L0a はそれを (E2) の `Llvm` の段の段落の「(E3) と同じ形であり、
違うのは呼び出し先を決めるのが `callee` の値ではなく op の生成コードだということだけである」から出す。

同じ (E2) の生成の表の `Llvm` の行は、結果の leaf の `H` を A3 の宣言から読み、単一の `Unknown` の宣言に
`+1` を当てる。適用した活性化が返した参照を持つ leaf について、その 2 つは食い違う -- 参照は `b` の中で
作られており、この段で新しく作られるのではない。`App` の行がすでに受けているのと同じ訂正である。

足す文は 2 つである。(E4) に「`b` を作ったのが適用する `Llvm` の段であれば、それらの参照はその op が結果の
leaf に置く参照として `Obl(a)` に入る」。(E2) の生成の表の `Llvm` の行に「ただしオペランドを適用する op の
結果の leaf のうち、適用した活性化が返した参照を持つものについては `H` は**変わらない** -- `App` の行と
同じ理由である」。

**(2) (F) の解放が `Std::FFI::Destructor` のデストラクタを走らせることを (F) が述べない。** (F) の本文は

> 解放とは、`o` が持つ参照 (D25) をすべて処分し、それから `o` の記憶域を返すことである

であり、参照の処分と記憶域の返却の 2 つしか挙げない。コードは `Destructor` のオブジェクトについて、
参照を処分する前にデストラクタ関数を適用し、返った `IO` を走らせる (`CODE src/generator.rs:
Generator::build_traverser_work_nonnull_boxed_with` -- `obj.is_destructor_object()` の枝が
`build_run_destructor` を `traverse_refs` の前に置く、`CODE src/generator.rs:
Generator::build_run_destructor` -- `apply_lambda(dtor, vec![value], false)`)。すなわち解放は活性化を
作る。(1) と同じ形の追記が要る -- 解放の中で `_dtor` の欄の参照が retain され、適用がそれと `_value` の
欄の値を消費し、返った値が `_value` の欄に戻る。

**D24 の書き出しが段を「6 種」と数える。**「段は次の 6 種である」の後に (E1)-(E7) の 7 つが並ぶ。

**D22 のグローバルのアクセサの行。** 現在の本文は

> **グローバルのアクセサ**。初期化済みの旗を見て、まだならグローバル初期化子の本体を持つ関数
> `InitValue#<symbol>` を呼び、返った値に `mark_global` で印を付けてから記憶域へ格納する。

であるが、`mark_global` を呼ぶのは `InitValue#<symbol>` の側であって、アクセサではない。
`implement_rc_global` は初期化子の本体を `eval_rc_expr` で評価した直後、`build_return` の前に
`self.mark_global(obj.clone())` を呼び、アクセサ (`acc_fn`) は `store_init_value` でその関数を呼んで
返った値を記憶域へ格納するだけである (`CODE src/rc_ir/codegen.rs: Generator::implement_rc_global`)。
D24 が要る順序 -- 活性化が終わってから (E5) が走る -- はこのコードでも成り立つ。印を付けるのは本体の
評価が終わった後だからである。

**`proof_links.py` が README とこの走査の文書を読まない。**`citations_of` が `CODE` を集めるのは
`p*.md` からだけであり、README の本文と `llvmgen-function-values.md` の `CODE` は `citations.tsv` にも
`// PROOF:` の注釈にも入らない (`dev-docs/proof/proof_links.py` の `citations_of`)。この文書から README へ
移った定義が引くコード -- `Generator::mark_global`、`Generator::build_release_boxed_with`、
`build_free_boxed`、`Generator::build_traverser_work`、`build_main_function`、`run_ios_runner`、
`ExportStatement::implement`、`Generator::build_rc_closure`、`InlineLLVMUndefinedInternalBody::generate`、
`Lowerer::lower_var`、`Generator::add_global_object` -- のうち、他の `p*.md` が引かないものがそれである。
定義が引くコードにもリンクを張るなら、走査の対象に README とこの文書を足すことになる。
