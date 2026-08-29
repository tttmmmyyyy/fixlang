# P29 と P27 -- 静的に決めた呼び出し先と、実行の合成

この文書は `README.md` の P29 と P27 を証明する。対象コミットは
`ce6d32b1480c59eac76c6cdedf386b51a0edbde6` である。

**実行のモデルは `README.md` が持つ。**環境 (D22)、活性化 (D23)、実行 (D24)、参照の持ち手 (D25)、
計数下のオブジェクトとグローバル状態のオブジェクト (D26)、実行の終わり方 (D31)、実行の読み (D32) は、
`README.md` の第 3.6 節が定める。この文書はそれを引くだけで、実行のモデルを自分では置かない。この文書が
自分で置くのは、局所定義「勘定が合っている時点」と「`cancel` の所有述語」、および補題 L0b、L0、L0a、
L1-L5 だけである。

P29 は層 0 の命題であり、その証明はどの命題も引かない。P27 の証明が引く命題は P28 の**言明**だけである --
参照の持ち手がちょうど 1 つであることを、下の「`H` の分解」が読む。P27 が使うのは D11 と D12 という述語
そのものであって、それを `borrow_ify` と `cancel` が保存することではない。補題 L0 は `cancel` が読む所有に
ついての別の主張であり、P9、P10、P11、P12、P29 の**言明**を引く。

## 1. 結論

| 主張 | 結果 |
|---|---|
| P29 静的に決めた呼び出し先は実行時の呼び出し先である | 証明した (L0b を入力に当てる) |
| (R1) 解放されたオブジェクトの読みが起きない | 証明した |
| (R2) どのオブジェクトも高々 1 回しか解放されない | 証明した |
| (R3) 正常終了する実行で解放されずに残る計数下のオブジェクトは、環境が持つ参照が指すオブジェクトから到達できるものに限る | 証明した |

**(R2) は仮定を 1 つも足さずに閉じる。**この文書はかつて「`Retain` は解放されたオブジェクトに触れない」を
仮定として要求していた。その穴は D11 の (S-c) が `Retain(v, π)` と `Release(v, π)` の**触れる**先を扱う
ようになったことで閉じ、L2b の `Retain` の場合が (S-c) から直接に出る。

**関数の値に番地を書き込む段の数え上げは A21 が持つ。**この文書はそれを引く。`llvmgen-function-values.md`
が A21 の果たす者の数え上げである。

**残っているものの節が、README に足す文を 5 つ挙げる。**(E4) の行き先に (F) の解放が作った活性化の分が
無いこと、(F) の解放が作る活性化の内側に D24 の時点が無いこと、D22 の環境に `FFI_CALL` が届く C のコードが
無いこと、「最上位の記号の名前は局所名ではない」を述べる者が居ないこと、`proof_links.py` の走査が README を
読まないことである。どれもこの文書の段が読む箇所であり、どこがそれを読むかを同節が名指す。

## 2. 静的に決めた呼び出し先

### L0b (名前が決める呼び出し先)

**L0b.** ASSUME  NEW `Q`: RcProgram、
                 **(N1)** `Q` のすべての束縛変数の名前は相異なり、`Q` の `funcs` のどの鍵の名前とも
                 異なる、
                 **(N2)** `Q` の変数の使用は、その位置でスコープに入っている束縛に解決する、
                 **(N3)** `Q` の `funcs` の各鍵の名前は局所名 (`FullName::is_local` が真) ではなく、
                 コード生成が読む `global_types` はその名前を持たないか、funptr 型で持つ、
                 NEW `Let(x, App(callee, args), k)`: `Q` のある本体の節点、
                 `resolve_callee_params(callee, vars, Q) = Some(params)`
         PROVE   `Q` から生成したコードの実行のその節点の段について、`params` はその段の実行時の
                 呼び出し先 (D23) のパラメータの列である。

<1>1. `resolve_callee_params(callee, vars, Q)` が `Some(params)` を返すのは 2 つの場合であり、
      `None` を返すのはそれ以外である。
  1 つは `vars.closure_targets` が `callee.name` を持つ場合、もう 1 つは `Q.funcs` が
  `FuncRef { name: callee.name }` を持つ場合である。どちらでもなければ `?` が `None` を返す。どちらの
  場合も返るのは `Q.funcs` のその `FuncRef` の関数の `params` である。
  BY CODE src/rc_ir/ownership.rs: resolve_callee_params

<1>2. `vars.closure_targets` が `callee.name` を持つのは、この本体に `Let(callee, Closure(fref, caps), k)`
      が在るとき、かつそのときに限る。そのとき返るのは `Q.funcs[fref]` のパラメータである。
  `collect_bindings` が `closure_targets` に挿入するのは `RcRhs::Closure(fref, _)` の腕の 1 か所だけで
  ある。
  BY <1>1, CODE src/rc_ir/ownership.rs: collect_bindings, resolve_callee_params

<1>3. `<1>2` の場合、実行時の呼び出し先は `Q.funcs[fref]` である。
  <2>1. `callee` の値は、その `Let` が束縛した値である。
    (N1) より `callee.name` を束縛する節点はこの `Let` の 1 つだけであり、(N2) より `callee` の使用は
    その束縛に解決する。
    BY (N1), (N2), <1>2
  <2>2. その値は funptr の欄に `func_vals[fref]` を持つクロージャである。
    `build_rc_closure` は `CLOSURE_FUNPTR_IDX` の欄に `func_vals[func]` を入れる。
    BY <2>1, CODE src/rc_ir/codegen.rs: Generator::build_rc_closure
  <2>3. `func_vals[fref]` は `implement_rc_program` が `Q.funcs[fref]` の本体を実装した LLVM 関数で
        ある。
    `implement_rc_program` は `Q.funcs` の各 `(fref, func)` について LLVM 関数を 1 つ決めて
    `func_vals` に入れ、続く走査で `implement_rc_function(func, func_vals[fref], ..)` を呼ぶ。
    `implement_rc_function` はその LLVM 関数に `entry` ブロックを付けて `func.body` を出す。
    BY CODE src/rc_ir/codegen.rs: Generator::implement_rc_program,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>4. QED
    D23 より、`callee` の値がクロージャのとき実行時の呼び出し先はその funptr の指す関数である
    (`apply_lambda` は `get_lambda_func_ptr` が返す関数ポインタを `build_indirect_call` で呼び、
    `get_lambda_func_ptr` は `is_closure()` のとき `CLOSURE_FUNPTR_IDX` の欄を読む)。
    BY D23, <2>2, <2>3, CODE src/generator.rs: Generator::apply_lambda,
       CODE src/generator.rs: Generator::get_lambda_func_ptr

<1>4. `Q.funcs` が `FuncRef { name: callee.name }` を持つ場合、実行時の呼び出し先はその関数である。
  <2>1. コード生成が `callee.name` の値として返すのは、`declared_globals` に `callee.name` で登録された
        `ValueAccessor` が答える値である。
    `App` の腕は `get_scoped_obj(&callee.name)` を呼び、`get_scoped_obj` は `get_scoped_value` の答えの
    `accessor` に `get` を掛ける。`get_scoped_value` は `var.is_local()` のときだけスコープを引き、
    そうでなければ `get_or_declare_global` を通る。(N3) より `callee.name` は局所名ではないので後者で
    ある。`get_or_declare_global` は `declared_globals` に在ればそれを返し、無ければ
    `declare_program_global` を呼んで登録させ、それが `None` を返せば panic する。
    BY (N3), CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
       CODE src/generator.rs: Generator::get_scoped_obj, Generator::get_scoped_value,
       Generator::get_or_declare_global
  <2>2. `declared_globals` に名前を登録するのは `add_global_object` だけであり、それを呼ぶのは
        `declare_program_global` (`global_types` の型が funptr でないとき、そのアクセサ関数を登録する) と
        `declare_lambda_function` (`fn_ty.is_funptr()` のとき、いま作った LLVM 関数自身を登録する) の
        2 か所である。同じ名前で 2 度登録すると `add_global_object` は abort する。
    BY CODE src/generator.rs: Generator::add_global_object, Generator::declare_program_global,
       Generator::declare_lambda_function
  <2>3. `callee.name` について登録されるのは、`declare_lambda_function` が funptr 型で登録する場合だけで
        ある。
    `declare_program_global(name)` はまず `global_types.get(name)` を引き、`None` なら `None` を返す。
    (N3) より `callee.name` は `global_types` に無いか funptr 型で在る。無ければ `declare_program_global`
    は `None` を返し、`<2>1` より `get_or_declare_global` は panic する -- そのときプログラムは走らず、
    この場合の段は存在しない。funptr 型で在れば `declare_program_global` は `is_funptr()` の枝を取り、
    `declare_lambda_function(&ty, name)` を呼ぶ。`declare_lambda_function` は `fn_ty.is_funptr()` の
    ときだけ `add_global_object` を呼ぶので、アクセサとして登録される道は無い。
    BY (N3), <2>1, <2>2, CODE src/generator.rs: Generator::declare_program_global,
       Generator::declare_lambda_function
  <2>4. `<2>3` が登録する LLVM 関数は、`implement_rc_program` が `Q.funcs[FuncRef{callee.name}]` の
        本体を実装した LLVM 関数と同じものである。
    `declare_lambda_function(fn_ty, name)` が作る関数の記号名は `object_file_symbol_name(name)` である。
    `implement_rc_program` はその名前について、まず `module.get_function(object_file_symbol_name(..))` を
    引き、在ればそれを使い、無ければ `declare_program_global` を、それも `None` なら
    `declare_lambda_function` を呼ぶ。`<2>3` より `callee.name` について
    `declare_lambda_function` が呼ばれるのは funptr 型のときであり、`add_global_object` が同じ名前の
    2 度目の登録で abort するので、この名前について `declare_lambda_function` が呼ばれるのは高々 1 度で
    ある。よって `implement_rc_program` の 3 つの枝のどれを通っても、得られる LLVM 関数は
    `object_file_symbol_name(callee.name)` を記号名とするその 1 つである。`implement_rc_program` は
    続く走査でその関数に本体を実装する。
    BY <2>2, <2>3, CODE src/generator.rs: Generator::declare_lambda_function,
       Generator::add_global_object, CODE src/rc_ir/codegen.rs: Generator::implement_rc_program
  <2>5. QED
    `<2>3` の登録は `ValueAccessor::Global(fun, ty)` であり、`ty.is_funptr()` なので `ValueAccessor::get`
    は `fun.as_global_value().as_basic_value_enum()` を返す -- アクセサを呼ばず、関数そのものを値とする。
    D23 より、`callee` の値が funptr のとき実行時の呼び出し先はそれ自身である。`<2>4` よりその関数は
    `Q.funcs[FuncRef{callee.name}]` の本体を持つ。
    BY D23, <2>1, <2>3, <2>4, CODE src/generator.rs: ValueAccessor::get,
       CODE src/generator.rs: Generator::get_lambda_func_ptr

<1>5. QED
  `<1>1` より `Some(params)` が返るのは 2 つの場合であり、どちらでも `params` は `Q.funcs` のある
  `FuncRef` の関数のパラメータの列である。`<1>3` と `<1>4` が、その 2 つの場合のそれぞれについて、
  その関数が実行時の呼び出し先であることを与える。
  BY <1>1, <1>2, <1>3, <1>4

### P29 の証明

**P29.** ASSUME  NEW `P`: `borrow_ify` の入力プログラム、
                 A6、A11、A13、
                 NEW `Let(x, App(callee, args), k)`: `P` のある本体の節点、
                 `resolve_callee_params(callee, vars, P) = Some(params)`
        PROVE   `params` はその段の実行時の呼び出し先 (D23) のパラメータの列である。

<1>1. `P` は L0b の (N1) を満たす。
  A6 が「`borrow_ify` の入力のすべての束縛変数の名前は相異なり、どの関数の名前とも異なる」を与える。
  BY A6

<1>2. `P` は L0b の (N2) を満たす。
  A11 が「変数の使用は、その位置でスコープに入っている束縛に解決する」を与える。
  BY A11

<1>3. `P` は L0b の (N3) を満たす。
  <2>1. `P.funcs` の鍵の名前は局所名ではない。
    `P.funcs` の鍵は、lowering が最上位の記号から作る鍵 (`lower_symbol` が funptr 型の記号について
    `FuncRef { name: sym.name }` を作る) か、持ち上げた lambda に付ける鍵 (`fresh_closure_ref` が
    `current_symbol` の名前空間の下に `closure{N}` を作る) である。どちらも名前空間を持つ。
    BY CODE src/rc_ir/lower.rs: Lowerer::lower_symbol, Lowerer::fresh_closure_ref
  <2>2. コード生成が読む `global_types` は、`P.funcs` の鍵の名前を持たないか、funptr 型で持つ。
    コード生成に渡る `global_types` は `global_types_including_synthesized` が作る。それは
    `Program::global_types` -- 最上位の記号の名前からその型への写像 -- に、`fn_ty` が funptr である
    `funcs` の各項目をその `fn_ty` で上書き挿入したものである。よって funptr の関数の名前は funptr 型で
    在る。funptr でない関数の名前について残るのは `Program::global_types` の側の項目だけであり、その
    項目が在るのはその名前が最上位の記号の名前であるときである。`lower_symbol` は funptr 型の記号だけを
    `funcs` の鍵にし、それ以外の型の記号をグローバル初期化子にするので、funptr でない関数の名前が
    最上位の記号の名前であることは無い。
    BY CODE src/build/divide_program.rs: global_types_including_synthesized,
       CODE src/ast/program.rs: Program::global_types, CODE src/rc_ir/lower.rs: Lowerer::lower_symbol
  <2>3. QED
    BY <2>1, <2>2

<1>4. QED
  L0b を `Q = P` に当てる。
  BY L0b, <1>1, <1>2, <1>3

**注**。`resolve_callee_params` が `None` を返す場合について P29 は何も言わない。そのとき
`rhs_consumes` は全位置を所有として扱う (`CODE src/rc_ir/ownership.rs: rhs_consumes` -- `callee_params`
が `None` のとき `is_owning_position` は `true`)。A7 が置く近似がこれである。

### L0 (`cancel` が読む所有と実行時の呼び出し先)

**DEF `cancel` の所有述語**
`cancel(prog, type_env)` は `owned_units = all_owned_units(prog, type_env)` を作り、走査の `consume_rhs`
が `rhs_consumes` に渡す述語 `owns` を
`owns(p, λ) = owned_units.contains((p.name, truncate_to_unit(ty(p), λ)))` と定める
(`CODE src/rc_ir/borrow.rs: cancel`, `CancelAnalysis::consume_rhs`)。この述語を **`owns_cancel`** と書く。
`rhs_consumes` は `App(callee, args)` の第 `i` 引数の leaf `λ` を、`resolve_callee_params` が返した
`params` について `owns_cancel(params[i], λ)` が真であるときに消費として報告する
(`CODE src/rc_ir/ownership.rs: rhs_consumes`)。

**言明**。`borrow_ify` の出力プログラム `P'` のどの `Let(x, App(callee, args), k)` についても、次の 2 つが
成り立つ。

- **(a)** `resolve_callee_params(callee, vars, P')` が `Some(params)` を返すならば、`params` はその段の
  実行時の呼び出し先 (D23) のパラメータの列である。`None` を返すならば、その段の実行時の呼び出し先の
  `borrowed_units` は空である。
- **(b)** `Some(params)` の場合、実行時の呼び出し先を `g` とすると、各 `i` と各 leaf `λ` について、
  `owns_cancel(params[i], λ)` が真であることと、`g` が `truncate_to_unit(ty(params[i]), λ)` を D14 の
  意味で所有することは同値である。

<1>1. `P'` は L0b の (N1) を満たす。
  <2>1. `P'` の束縛変数の名前は、`P` の束縛変数の名前か、`clone_func` が作る複製の名前である。
    `borrow_ify` は出力の各版を、原本 `func.clone()` の本体を `RewriteCtx::rewrite` で書き換えたものか、
    `clone_func` の複製の本体を同じく書き換えたものとして作る。`rewrite` は節点を落とし、`Retain`/
    `Release` の節点を既にある変数について足し、`App` の `callee` を差し替えるだけで、束縛を新しく
    作らない (P10、P11、P12)。グローバル初期化子も同じ `rewrite` を通る。
    BY P10, P11, P12, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func
  <2>2. `P` の束縛変数の名前は相異なり、`P` のどの関数の名前とも異なる。
    BY A6
  <2>3. 複製が導入する名前は、`P` のどの束縛名とも異なる。
    BY P9
  <2>4. 相異なる 2 つの複製が導入する名前は互いに異なる。
    `borrow_ify` は 1 つの `rename_counter` をすべての `clone_func` の呼び出しに渡し、
    `assign_fresh_name` は呼ばれるたびにそれを 1 増やしてから `<元の名前>#b<counter>` を作る。よって
    2 つの複製名が等しければ `counter` が等しく、同じ呼び出しである。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/rename.rs: fresh_rename_function,
       assign_fresh_name
  <2>5. 複製が導入する名前は、`P'` のどの関数の名前とも異なる。
    `P'` の関数の名前は `P` の関数の名前か、`borrow_funcref` が作る `<元の名前>#borrow` である。A13 より
    `P` に現れるどの名前も `#` で区切った最後の断片が `b<10 進数字>` の形ではないので、`P` の関数の名前は
    複製名と異なる。`#borrow` で終わる名前も、最後の断片が `b<10 進数字>` の形ではないので複製名とは
    異なる。
    BY A13, CODE src/rc_ir/borrow.rs: borrow_funcref
  <2>6. `P` の束縛変数の名前は、`P'` のどの関数の名前とも異なる。
    `<2>2` が `P` の関数の名前について与える。`#borrow` で終わる名前については A13 が、`P` に現れる
    どの名前も最後の断片が `borrow` ではないと言う。
    BY A13, <2>2
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>2. `P'` は L0b の (N2) を満たす。
  `borrow_ify` は出力の各本体を、原本 `func.clone()` の本体か `clone_func` の複製の本体を
  `RewriteCtx::rewrite` で写して作る。複製は束縛変数の一斉の付け替えであり、それ以外の違いを持たない
  (P9) ので、束縛と使用の対応を保つ。`rewrite` は `Retain`/`Release` の節点を落とし、既にある変数を
  名指す `Retain`/`Release` を足し、`App` の `callee` の名前を替えるだけである (P10、P11、P12)。落とす
  節点も足す節点も束縛を持たない。よって A11 が `P` について与えるスコープの規律は `P'` でも成り立つ。
  BY A11, P9, P10, P11, P12, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>3. `P'` は L0b の (N3) を満たす。
  <2>1. `P'.funcs` の鍵の名前は、`P.funcs` の鍵の名前か、`borrow_funcref` が作る `<元の名前>#borrow`
        である。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, borrow_funcref
  <2>2. `P.funcs` の鍵の名前について (N3) が成り立つ。
    `P.funcs` の鍵は、lowering が最上位の記号から作る鍵 (`lower_symbol` が funptr 型の記号について
    `FuncRef { name: sym.name }` を作る) か、持ち上げた lambda に付ける鍵 (`fresh_closure_ref` が
    `current_symbol` の名前空間の下に `closure{N}` を作る) であり、どちらも名前空間を持つ。コード生成に
    渡る `global_types` は `global_types_including_synthesized` が作り、それは `Program::global_types` --
    最上位の記号の名前からその型への写像 -- に、`fn_ty` が funptr である `funcs` の各項目をその `fn_ty` で
    上書き挿入したものである。よって funptr の関数の名前は funptr 型で在る。funptr でない関数の名前に
    ついて残るのは `Program::global_types` の側の項目だけであり、その項目が在るのはその名前が最上位の
    記号の名前であるときである。`lower_symbol` は funptr 型の記号だけを `funcs` の鍵にし、それ以外の型の
    記号をグローバル初期化子にするので、funptr でない関数の名前が最上位の記号の名前であることは無い。
    BY CODE src/rc_ir/lower.rs: Lowerer::lower_symbol, Lowerer::fresh_closure_ref,
       CODE src/build/divide_program.rs: global_types_including_synthesized,
       CODE src/ast/program.rs: Program::global_types
  <2>3. `<元の名前>#borrow` について (N3) が成り立つ。
    `borrow_funcref` は元の名前の `name` の欄に文字列を足すだけなので名前空間は変わらず、`<2>2` より
    局所名ではない。借用版の `fn_ty` は原本の `fn_ty` と等しい (`clone_func`) ので、funptr の借用版の
    名前は `global_types_including_synthesized` が funptr 型で挿入する。funptr でない借用版の名前は
    最上位の記号の名前ではない -- A13 より `P` に現れるどの名前も `#` で区切った最後の断片が `borrow`
    ではなく、最上位の記号の名前は `P` に現れる名前だからである。
    BY A13, <2>2, CODE src/rc_ir/borrow.rs: borrow_funcref, clone_func,
       CODE src/build/divide_program.rs: global_types_including_synthesized
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>4. (a) の前半が成り立つ。
  L0b を `Q = P'` に当てる。
  BY L0b, <1>1, <1>2, <1>3

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
        あるときだけである。`borrow_versions` の鍵は `P.funcs` の鍵であり、`borrow_ify` は原本を同じ鍵で
        出力に入れるので、そのとき `P'.funcs` はその鍵を持ち、`resolve_callee_params` は
        `prog.funcs.contains_key` の枝で `Some` を返す。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::route, borrow_ify,
       CODE src/rc_ir/ownership.rs: resolve_callee_params
  <2>4a. 借用版の名前が `RcVar` として現れるのは、`App` の `callee` の位置だけである。
    `borrow_ify` が出力の本体に借用版の名前を書くのは `route` の返り値だけであり、それは
    `RcRhs::App(callee, args)` の `callee` に置かれる。`clone_func` が導入するのは束縛変数の名前だけで
    あって関数の名前ではない (P9 の言明)。A13 より、`P` に現れるどの名前も `#borrow` で終わらないので、
    入力から運ばれた名前がたまたま借用版の名前と一致することも無い。
    BY A13, P9, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, RewriteCtx::route, borrow_ify
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

    BY A21, CODE src/rc_ir/codegen.rs: Generator::build_rc_closure,
       CODE src/generator.rs: ValueAccessor::get, CODE src/fixstd/builtin.rs: InlineLLVMFixBody
  <2>4c. (M3) が置く `gc.current_function()` は、capture を持つ関数の LLVM 関数である。
    <3>1. `InlineLLVMFixBody` を持つ節点の第 3 オペランドは、lowering が局所名 `#CAP` について作った
          ものである。
      `fix_body` が `cap_name` を `FullName::local(CAP_NAME)` として `InlineLLVMFixBody` を組み立て、
      `CAP_NAME` は `"#CAP"` である。`free_vars_mut` はその欄を自由変数の 1 つとして挙げ、`lower_llvm`
      は各自由変数をオペランドに直す。
      BY CODE src/fixstd/builtin.rs: fix_body, InlineLLVMFixBody, CODE src/constants.rs: CAP_NAME,
         CODE src/rc_ir/lower.rs: Lowerer::lower_llvm
    <3>2. CASE `#CAP` が lowering の環境で解ける。
      `Lowerer` は関数ごとに新しい環境を作り (`lower_lambda_as_function` が `self.scope` を
      `mem::take` で置き換える)、その環境に `FullName::local(CAP_NAME)` を束縛するのは
      `lower_lambda_as_function` の `lam_ty.is_closure()` の枝 1 か所だけである。その枝は `capture` を
      `Some(capture_var)` とする `RcFunc` を作る。よってこの op を含む本体の関数は capture を持つ。
      BY CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function
    <3>3. CASE `#CAP` が lowering の環境で解けない。
      `lower_llvm` は `self.global_types.get(name)` を引き、`None` なら panic する。`global_types` の
      鍵は最上位の記号の名前であり (`Program::global_types`)、最上位の記号の名前は局所名ではない
      (`lower_var` は束縛の解けない名前について `assert!(!v.name.is_local())` を立てる)。`#CAP` は
      `FullName::local` が作る局所名なので鍵ではなく、この場合 lowering は panic する。プログラムは
      作られず、実行も段も無い。
      BY <3>1, CODE src/rc_ir/lower.rs: Lowerer::lower_llvm, Lowerer::lower_var,
         CODE src/ast/program.rs: Program::global_types
    <3>4. QED
      `<3>2` と `<3>3` は `#CAP` が解けるか解けないかで尽きる。`implement_rc_function` が本体を出す間
      `gc.current_function()` はその本体の関数である。
      BY <3>1, <3>2, <3>3, CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>5. QED
    `resolve_callee_params` が `None` を返すなら、`callee.name` は `P'.funcs` の鍵でも
    `closure_targets` の鍵でもない (`CODE src/rc_ir/ownership.rs: resolve_callee_params` -- 2 つの枝が
    どちらも当たらないときだけ `?` が `None` を返す)。`callee` の値の funptr は `<2>4b` の 3 つの
    いずれかである。(M1) の `fref` は `<2>3` より原本の名前であり、(M2) が読むグローバルの名前は
    `<2>4a` より借用版ではなく (借用版の名前は `App` の `callee` の位置にしか現れず、そこでは `<2>4` より
    `resolve_callee_params` が `Some` を返す) やはり原本であり、(M3) の関数は `<2>4c` より capture を
    持つので `<2>1` より借用版を持たない原本である。`<2>2` より原本の `borrowed_units` は空である。
    BY <2>1, <2>2, <2>3, <2>4, <2>4a, <2>4b, <2>4c,
       CODE src/rc_ir/ownership.rs: resolve_callee_params

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
    λ)))` である (DEF `cancel` の所有述語)。`<2>1` より `params[i]` は `g.params[i]` であり、`<2>3` より
    その名前で `owned_units` に入る項目は `g` が入れたものだけである。よって `<2>2` より、その項目が
    在ることと `truncate_to_unit(ty(params[i]), λ)` が `g.borrowed_units` に入らないことは同値であり、
    D14 より後者は `g` がその unit を所有することである。
    BY D14, <2>1, <2>2, <2>3, DEF `cancel` の所有述語

<1>7. QED
  BY <1>4, <1>5, <1>6

**注**。この文書が D9 と D10 を読むときの「呼び出し先」は D23 が定める実行時の関数である。L0 は、その読みの
下での義務集合が、`cancel` が静的に計算する消費と食い違わないことを述べる。食い違えば、D11 を保存すると
称する P14 と P23 が別の述語を保存していることになる。

## 3. 実行の補題

### L0a (オペランドを適用する `Llvm` の段が作る活性化)

**言明**。`LLVMGen::applies_a_function_operand` が真を宣言する op を持つ節点
`Let(x, Llvm(gen, args), k)` の段 (以下**適用する `Llvm` の段**と呼ぶ) について、次が成り立つ。

- **(a)** その段は 1 つ以上の活性化を作り、その段を持つ活性化 `a` は、作った活性化のそれぞれが終わるまで
  中断中である。
- **(b)** その段が作った活性化 `b` が終わる (E4) の段で `Obl(b)` を離れる参照は `Obl(a)` に入り、その段で
  `H` は変わらない。

<1>1. (a) が成り立つ。
  D24 の (E2) の `Llvm` の段の段落が「その op の生成コードがオペランドを関数として適用するとき
  (`LLVMGen::applies_a_function_operand` が真を宣言する op)、適用された関数の本体の活性化が作られ、
  `a` はそれが終わるまで中断中である」と述べ、続けて「**1 つの段が活性化を 2 つ以上作りうる**」と述べる。
  どの op がこれを宣言するかは op 自身が答え、宣言せずに関数を適用した op は開発モードで落ちる
  (`CODE src/ast/inline_llvm.rs: LLVMGen::applies_a_function_operand`,
  `CODE src/generator.rs: Generator::apply_lambda` -- 先頭の `assert!(declares, ..)`)。
  BY D24 (E2 の `Llvm` の段の段落), CODE src/ast/inline_llvm.rs: LLVMGen::applies_a_function_operand,
     CODE src/generator.rs: Generator::apply_lambda

<1>2. (b) が成り立つ。
  D24 の (E4) が「`b` を作ったのが (E2) のうちオペランドを適用する `Llvm` の段であれば、それらの参照は
  同じくその段を実行している活性化の `Obl` に入り、その活性化は同じ位置で続きを実行する。**このとき
  (E2) の生成の表の `Llvm` の行はその leaf について読まない** -- `App` の行と同じ理由で、その参照は
  呼び出し先の中で作られてここへ渡ってくるものであり、`H` は動かない」と述べる。
  BY D24 (E4)

<1>3. QED
  BY <1>1, <1>2

**注 (`Obl(a)` を離れる参照の行き先)**。L0a は (E3) の受け渡しの形を主張しない。D24 の (E2) の
`Llvm` の段の段落は「(E3) と違うのは 3 点である。呼び出し先を決めるのが `callee` の値ではなく op の
生成コードであること、**呼び出し先に渡る値がオペランドとは限らない**こと、そして **1 つの段が活性化を
2 つ以上作りうる**ことである」と述べ、続けて「`Obl(a)` の動きは D9 の `Llvm` の行と A3 の宣言が決めるので、
この表の行はそのままである」と述べる。`InlineLLVMFixBody` がその形を示す -- `fix(f)` のクロージャをその場で
組み立てて `f` に渡し、返った関数に改めてオペランド `x` を渡すので、1 回目の活性化の第 1 引数は
どのオペランドでもなく、その段が作った参照である
(`CODE src/fixstd/builtin.rs: InlineLLVMFixBody` -- `create_obj` で `fix(f)` を作り、`apply_lambda` を
2 回呼ぶ)。よって `Obl(a)` を離れる参照の多重集合と作られた活性化の初期 `Obl` は一致しない。

### L1 (呼び出しと返りの受け渡しが釣り合う)

**言明**。(a) (E3) の段で `Obl(a)` を離れる参照の多重集合は、`Obl(b)` の初期値に一致する。
(b) (E4) の段で `Obl(b)` を離れる参照の多重集合は、その段の行き先 -- `b` を (E3) が作ったならその親の
`Obl(a)`、適用する `Llvm` の段 (L0a) が作ったならその段を実行している活性化の `Obl`、(F) の解放が
作ったならその解放を含む段を実行している活性化の `Obl`、(E1) か (E7) が作ったなら `E` -- が得る参照の
多重集合に一致する。

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
    挟まない。`implement_rc_function` が capture の引数を読むのは `func.capture` が `Some` のときだけで
    あり、開発モードの表明が、消費したパラメータが LLVM 関数のパラメータを尽くすことを検査する。
    BY CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>3. capture パラメータの型は boxed であり、その `boxed_leaf_paths` も `rc_units` も 1 元である。
    lowering は capture 変数の型を `make_dynamic_object_ty()` とする。それは `Std::#DynamicObject` の
    tycon であり、その `TyConInfo` は `is_unbox: false` を持つので boxed である。よって D4 の規則 3 と
    D5 の `unit_step` の `is_box` の腕がどちらも自分自身 1 つを返す。
    BY D4, D5, CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function,
       CODE src/fixstd/builtin.rs: make_dynamic_object_ty, bulitin_tycons
  <2>4. capture の unit は所有される。
    `borrow_ify` は各 `func` について `param_capture_units(func, type_env)` を丸ごと `owned_units` に
    入れ、出力の各版の `borrowed_units` を `param_capture_units` から `owned_units` を引いたものとして
    書く。よって原本 `f_own` の `borrowed_units` は空である。capture を持つ関数には借用版が作られない
    (`func.capture.is_none()` が条件) ので、capture を持つ関数の版は原本だけである。D14 より
    `borrowed_units` に入らない unit は所有される。
    BY D14, CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units
  <2>5. `callee` の型が funptr のとき、`callee` は boxed leaf を持たず、実行時の呼び出し先は `capture` を
        持たない。
    funptr の型は `is_fully_unboxed` が真であり (`is_funptr()` の枝が `true` を返す)、D4 の規則 1 で
    leaf を持たない。`apply_lambda` は `fun.ty.is_closure()` が偽のとき capture の引数を渡さない。
    `RcFunc` の `capture` の欄は「`Some` for the closure ABI ... `None` for the funptr ABI, which has no
    captures」であり、funptr ABI の関数は `capture` を持たない。実行時の呼び出し先の `fn_ty` が funptr で
    あることは、`callee` の値が funptr であることと、`lambda_function_type` が署名を型から作ることによる。
    BY D4, CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/generator.rs:
       Generator::apply_lambda, CODE src/rc_ir/ast.rs: RcFunc,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>6. QED
    `callee` の型はクロージャか funptr のどちらかである (`apply_lambda` の先頭の表明
    `assert!(fun.ty.is_closure() || fun.ty.is_funptr())`)。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, CODE src/generator.rs: Generator::apply_lambda

<1>4. 呼び出し先が所有する位置の引数の inhabited な leaf は、`b` の対応するパラメータの inhabited な leaf と
      1 対 1 に対応する。
  <2>1. `apply_lambda` は各引数の値をそのまま呼び出し先へ渡し、`implement_rc_function` はそれを対応する
        パラメータの名前に束縛する。retain も release も挟まない。
    BY CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/codegen.rs: Generator::implement_rc_function
  <2>1a. `args` の個数は呼び出し先の `params` の個数に等しい。
    `apply_lambda` は `assert_eq!(args.len(), src_tys.len())` を立てる。ここで
    `src_tys = fun.ty.get_lambda_srcs()` であり、`fun.ty` は `callee` の値の型である。呼び出し先の側では
    `lower_lambda_as_function` が `assert_eq!(params.len(), src_tys.len())` を立てる。ここで `src_tys` は
    その関数の `fn_ty` の引数型の列であり、`clone_func` は `fn_ty` と `params` を対で運ぶ。`callee` の値の
    型が呼び出し先の `fn_ty` と等しいことは、直接呼び出しでは `lower_symbol` が記号の型をそのまま
    `fn_ty` にすること、クロージャでは `lower_lam` が束縛変数の型を lambda の型 -- 持ち上げた関数の
    `fn_ty` -- とすることによる。
    BY CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function, Lowerer::lower_symbol, Lowerer::lower_lam,
       CODE src/rc_ir/borrow.rs: clone_func
  <2>2. A12 より第 `i` 引数の型は呼び出し先の第 `i` パラメータの型に等しいので、両者の
        `boxed_leaf_paths` は同じ列であり、`<2>1` より同じ値の同じ leaf を指す。よって inhabited (D16) で
        あることも一致する。
    BY A12, A14, D4, D16, <2>1, <2>1a
  <2>3. QED
    D9 の `App` の行と D10 の初期値は、どちらも同じ所有の割り当て (D14) を同じ関数 -- D23 が定める実行時の
    呼び出し先 -- について読む。`<2>1a` より引数とパラメータは 1 対 1 であり、余るパラメータは無い。よって
    消費される引数の leaf と、初期 `Obl` に入るパラメータの leaf は、`<2>2` の対応の下で同じ集合である。
    BY D9, D10, D14, D23, <2>1, <2>1a, <2>2

<1>5. (a) が成り立つ。
  BY <1>1, <1>2, <1>3, <1>4

<1>6. (b) が成り立つ。
  <2>1. (E4) で `Obl(b)` を離れるのは `x` の inhabited な全 boxed leaf の参照である。
    BY D9 (終端の `Ret` の行), D24 (E4)
  <2>2. CASE `b` を (E3) が作った場合。
    `Obl(a)` が得るのは `App` の結果の各 boxed leaf につき 1 つである (D10 の生成の `App` の行、
    D24 の (E2) の生成の表の `App` の行)。`apply_lambda` は呼び出し先が返した値をそのまま結果とするので、
    結果の値は `x` の値である。A12 の「`App` については引数とパラメータのほかに、結果の型も一致する」の
    行が `ty(x)` を呼び出し先の返り値の型とし、`RcFunc` の `ret_ty` の欄が終端の `Ret` が返す値の型を
    同じものとするので、`boxed_leaf_paths` の列も inhabited であることも一致する。よって `<2>1` の
    多重集合と等しい。
    BY A12, D10, D16, D24 (E2), D24 (E4), <2>1, CODE src/generator.rs: Generator::apply_lambda,
       CODE src/rc_ir/ast.rs: RcFunc
  <2>3. CASE `b` を (E1) か (E7) が作った場合。
    D24 の (E4) と (E7) より、`Obl(b)` を離れる参照はそのまま `E` に入る。行き先が 1 つで、参照は処分も
    生成もされないので、`E` が得る多重集合は `<2>1` のそれと等しい。
    BY D24 (E4), D24 (E7), <2>1
  <2>3a. CASE `b` を適用する `Llvm` の段が作った場合。
    L0a (b) より、`Obl(b)` を離れる参照はそのままその段を実行している活性化の `Obl` に入る。行き先が
    1 つで、参照は処分も生成もされないので、その `Obl` が得る多重集合は `<2>1` のそれと等しい。
    BY L0a, <2>1
  <2>3b. CASE `b` を (F) の解放が作った場合。
    D24 の (F) より、解放される `Destructor` のオブジェクト `o` について、`_dtor` の欄の関数が `_value` の
    欄の値に適用され、返った `IO` の動作が走る。その動作を走らせた結果は `o` の `_value` の欄へ書き戻される
    (`CODE src/generator.rs: Generator::build_run_destructor` -- `apply_lambda` の結果を
    `run_io_or_ios_runner` に渡し、返った `res` を `move_into_struct_field` で `_value` の欄へ入れる)。
    書き戻しまでの間、その参照を持つのはこの解放を含む段を実行している活性化である -- D24 の (F) が、
    この段が作る参照 (`_dtor` への retain) の持ち手をそう定めているのと同じである。行き先が 1 つで、参照は
    処分も生成もされないので、その `Obl` が得る多重集合は `<2>1` のそれと等しい。
    BY D24 (F), <2>1, CODE src/generator.rs: Generator::build_run_destructor
  <2>4. QED
    D24 の活性化の林の段落は「(E1) が作る活性化を**根**、(E3) と (E7)、(E2) のうちオペランドを適用する
    `Llvm` の段、および (F) の解放が `Destructor` について作る段が作る活性化を、それを作った活性化の
    **子**と呼ぶ。**活性化を作る段はこの 5 種で尽きる。**」と述べる。`<2>2`、`<2>3`、`<2>3a`、`<2>3b` が
    その 5 種を尽くす。
    BY D24 (活性化の林), <2>2, <2>3, <2>3a, <2>3b

<1>7. QED
  BY <1>5, <1>6

**DEF 勘定が合っている時点**
実行 `ρ` の 1 つの時点 (段と段の間) が**勘定が合っている**とは、各計数下のオブジェクト (D26) `o` について、
`o` が解放されている (D7) のは `H(o) = 0` のとき、かつそのときに限ることをいう。

**この定義は 1 つの条件しか持たない。**参照の持ち手がちょうど 1 つであること (P28) と、`H(o)` が `o` への
未処分の参照の総数であること (D8。計数下のオブジェクトについて -- D26) は、どの時点でも成り立つものとして
引く。この 2 つと D25 の持ち手の 3 分から、各計数下オブジェクト `o` について

`H(o) = Σ_{生きている活性化 a} Obl(a)[o] + Σ_{生きているオブジェクト o'} R(o')[o] + E[o]`

が出る。ここで `R(o')[o]` は、`o'` が保持する値の inhabited な boxed leaf のうち `o` を指すものの個数で
ある。以下ではこの等式を **`H` の分解**と呼び、P28、D8、D25 を引いて使う。

**L3 が示すのは、この定義のただ 1 つの条件である。**すなわち「`H` が 0 になったオブジェクトは解放される」
(D7) と「解放されたオブジェクトのカウントは 0 のままである」の 2 つが、実行のどの時点でも保たれる、という
ことである。

### L2 (到達できるオブジェクトは解放されていない)

**言明**。勘定が合っている時点において、オブジェクト `o'` が解放されておらず、`o'` から `o''` へ
到達できる (D25) ならば、`o''` も解放されていない。`o'` と `o''` は計数下でもグローバル状態でもよい。

<1>1. `o'` が解放されておらず、`o'` が `o` への参照を持つならば、`o` は解放されていない。
  `o` がグローバル状態ならば A8 より解放されない。`o` が計数下ならば、`o'` は割り当てられていて解放
  されていない -- すなわち D25 の意味で**生きている**オブジェクトである -- ので D25 の持ち手であり
  (D26 は D8 の参照を計数下のオブジェクトへの参照に限るが、持ち手の側には制限を置かないので `o'` の状態を
  問わない)、`H` の分解より `H(o) ≥ 1` であり、勘定が合っているので `o` は解放されていない。
  BY A8, D25, D26, DEF 勘定が合っている時点

<1>2. QED
  到達の道の長さについての帰納。長さ 0 のとき `o'' = o'` であり仮定そのものである。長さ `n+1` の道は、
  `o'` が持つ参照が指すオブジェクト `o1` への 1 歩と、`o1` から `o''` への長さ `n` の道に分かれる。
  `<1>1` より `o1` は解放されておらず、帰納法の仮定より `o''` も解放されていない。
  BY D25, <1>1

### L2b (参照が作られるオブジェクトは解放されていない)

**言明**。時点 `t` が勘定が合っており、`t` の直後の段が新しい参照を作るならば、その参照が指すオブジェクトは
`t` において解放されていない。ただしその段を持つ活性化の本体と、その段の中で (F) の解放が作る活性化の本体は
D11 を満たすとする。

<1>1. 新しい参照を作るのは、(E2) の段の生成の 6 行と、参照を処分する段の中で起きる (F) の解放が
      `Destructor` のオブジェクトについて行う retain と、その解放が作る活性化の節点である。
  D24 の (E2) の生成の表が 6 行を挙げる -- `Retain` の行、`Llvm` の行、boxed 容器の `Destructure` の
  名前付きフィールドの行、boxed union の変位アームの payload の行、`Closure` の行、`App` の行である。
  (E1)、(E3)、(E4) は持ち手を移すだけ、(E5) は印を付けるだけ、(E7) は `Obl` が空の活性化を作るだけで
  あり、(E6) の後に段は無い。段の中で起きる (F) の解放については、D24 が「**この段は参照も作る。**
  `_dtor` の欄の関数に適用の分の参照を与える retain がそれである」「よって **新しい参照を作るのは (E2)
  だけではない**」と述べる。同じ (F) が `Destructor` について作る活性化の節点も、その本体の (E2) の
  生成の 6 行によって参照を作る。
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

<1>1c. 段の中でこの段が新しく割り当てたオブジェクトは、`t` において解放されていない。
  D25 より、オブジェクトが解放されるのはそれを解放する段 ((F)) においてである。`t` は割り当ての段より
  前の時点なので、`t` までのどの段もそのオブジェクトを解放していない。
  BY D24 (F), D25

<1>2. CASE `Retain(v, π, s, k)` の行。
  D10 の `Retain` の行より、この段が作る参照が指すのは `π` の下の inhabited な各 leaf `λ` の
  `obj(v, λ)` である。`<1>1b` がそれらについて言明を与える。
  BY D10, D24 (E2), <1>1b

<1>3. CASE `Closure(f, caps)` の結果の行。
  この段が新しく割り当てる capture object である (D24 (E2) の生成の表)。`<1>1c` がそれについて言明を
  与える。`caps` が空のときは参照が生じないので、言明は空虚に成り立つ。
  BY D24 (E2), <1>1c

<1>4. CASE `App(callee, args)` の結果の行。
  この行は参照を作らない (D24 (E2) の生成の表の最終行)。よって言明は空虚に成り立つ。
  BY D24 (E2)

<1>5. CASE `Llvm` の行。
  <2>0. CASE op が `applies_a_function_operand` を宣言する。
    <3>1. 適用した活性化 `b` が返した参照を持つ結果の leaf について、この段は新しい参照を作らない。
      L0a (b) より、その参照は `b` の中で作られて (E4) でこの段を実行している活性化の `Obl` に入るもの
      であり、この段が新しく作るのではない。D24 の (E4) も「このとき (E2) の生成の表の `Llvm` の行は
      その leaf について読まない」と述べる。
      BY D24 (E4), L0a
    <3>2. 残る結果の leaf に置かれる参照が指すオブジェクトは、この op のオペランドの inhabited な boxed
          leaf が指すオブジェクトから到達できるか、グローバル値が到達するか、`b` が返したか、この段が
          割り当てたかのいずれかである。
      A3 の `Unknown` の行は「**この限定が成り立たない op が 2 種ある。** オペランドを適用する op
      (`LLVMGen::applies_a_function_operand`) では、適用した関数の中で新しく割り当てられたオブジェクトが
      結果に出る」と述べ、その leaf について行を読まないよう指示する。行を読まずに残るのは A4 である --
      「ある段がオブジェクトの記憶域へ書き込む内容は、その段の節点と、オペランドの値と、D21 の 4 種の
      結果だけで決まる」。子の活性化を作る段の結果は D21 の 4 種の 1 つであり、`b` の結果がそれである。
      よってこの段が結果の leaf に置ける番地は、オペランドの値から到達できるもの (D25)、グローバル値が
      到達するもの、`b` が返したもの、この段自身が割り当てたものに限る。
      BY A3, A4, D21, D25
    <3>3. QED
      `Let(x, Llvm(gen, args), k)` は D7 の読む構文であり、読まれる値は各オペランドである。`<1>1a` が
      オペランドの leaf の指すオブジェクトとそこから到達できるオブジェクトについて言明を与え、
      グローバル値が到達するオブジェクトは A8 より解放されない。`b` が割り当てたものとこの段が割り当てた
      ものは `<1>1c` が扱う。`<3>1` が除いた leaf について、この段は新しい参照を作らない。
      BY A8, D7, <1>1a, <1>1c, <3>1, <3>2
  <2>1. CASE 宣言が単一の `Fresh`。
    A3 より、生成コードはその leaf に新しく割り当てたオブジェクトへの新しい参照を置く。`<1>1c` が
    それについて言明を与える。
    BY A3, <1>1c
  <2>2. CASE 宣言が単一の `Unknown` であり、op は `applies_a_function_operand` を宣言しない。
    <3>0. CASE op が `InlineLLVMBoxedFromRetainedPtrIOS` である。
      A3 の `Unknown` の行は、この op について「オペランドは `Std::Ptr` で boxed leaf を持たないので、
      到達できる元が無い -- そのオブジェクトは C の側から渡された番地が指すものである」と述べ、その leaf に
      ついて行を読まないよう指示する。生成コードは第 1 オペランド `ptr` の第 0 欄の番地を結果の第 1 欄へ
      入れるだけである (`CODE src/fixstd/builtin.rs: InlineLLVMBoxedFromRetainedPtrIOS`)。この op を包む
      公開関数 `Std::FFI::boxed_from_retained_ptr` の doc は「Creates a boxed value from a retained
      pointer obtained by `boxed_to_retained_ptr`」「It is the user's responsibility to ensure that the
      argument is actually a pointer to the type of the return value, and undefined behavior will occur
      if it is not」と述べ、`boxed_to_retained_ptr` の doc は「This function is used to share ownership
      of Fix's boxed values with foreign languages」と述べる。すなわち番地は、参照を 1 つ持ったまま
      RC IR プログラムの外へ渡されたものである。その参照の持ち手は RC IR プログラムの外側のコード、
      すなわち環境 (D22) であり、`E` に在る。よって `H` の分解より `H ≥ 1` であり、勘定が合っているので
      解放されていない。その形でない番地を渡す実行は、doc が言うとおりこの文書のモデルの外にある。
      BY A3, D22, D8, D25, P28, DEF 勘定が合っている時点,
         CODE src/fixstd/builtin.rs: InlineLLVMBoxedFromRetainedPtrIOS,
         CODE src/fixstd/std.fix: boxed_from_retained_ptr, boxed_to_retained_ptr
    <3>1. CASE op が `InlineLLVMBoxedFromRetainedPtrIOS` でない。
      A3 の `Unknown` の行の限定が当たる -- 参照が作られるオブジェクトは、この op のオペランドの
      inhabited な boxed leaf が指すオブジェクトから到達できるか、グローバル値が到達するオブジェクトで
      ある。`Let(x, Llvm(gen, args), k)` は D7 の読む構文であり、読まれる値は各オペランドである。
      `<1>1a` がオペランドの leaf の指すオブジェクトとそこから到達できるオブジェクトについて言明を与え、
      グローバル値が到達するオブジェクトは A8 より解放されない。
      BY A3, A8, D7, D25, <1>1a, CODE src/rc_ir/provenance.rs: LeafOrigin
    <3>2. QED
      A3 の `Unknown` の行が限定の成り立たない op として挙げるのは 2 種であり、この場合の仮定が
      オペランドを適用する op を除いているので、残るのは `InlineLLVMBoxedFromRetainedPtrIOS` だけで
      ある。`<3>0` と `<3>1` はその 2 つの場合を尽くす。
      BY A3, <3>0, <3>1
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
    **でない**もの」)、参照を作らない。残る 4 つのうち、`applies_a_function_operand` を宣言する op に
    ついては `<2>0` が結果の全 leaf を扱い、宣言しない op については `<2>1`-`<2>4` が扱う。
    BY A3, D10, <2>0, <2>1, <2>2, <2>3, <2>4

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

<1>6a. この段の中で (F) が解放するどのオブジェクト `o` も、`t` において解放されていない。
  D24 の (F) より `o` が解放されるのは、この段の処分で `H(o)` が 0 になったときである。`t` において
  `H(o) = 0` であれば、`H` の分解より `o` への未処分の参照は無く、処分できる参照も無いので、この段の
  処分で `H(o)` が 0 に**なる**ことはない。よって `t` において `H(o) ≥ 1` であり、勘定が合っているので
  `o` は解放されていない。
  BY D8, D24 (F), D25, P28, DEF 勘定が合っている時点

<1>7. CASE (F) の解放が `Destructor` のオブジェクト `o` について行う `_dtor` への retain。
  D24 の (F) より、この retain の対象は `o` が `_dtor` の欄に持つ関数である
  (`CODE src/generator.rs: Generator::build_traverser_work_nonnull_boxed_with` --
  `obj.is_destructor_object()` の枝が `build_run_destructor` を `traverse_refs` の前に置く。
  `CODE src/generator.rs: Generator::build_run_destructor` -- `build_retain(dtor, one, ..)` が
  `apply_lambda` の前に立つ)。それは `o` が保持する値の boxed leaf が指すオブジェクトであり、D25 より
  `o` から到達できる。`<1>6a` と L2 がそれについて言明を与える。
  BY D24 (F), D25, L2, <1>6a, CODE src/generator.rs: Generator::build_run_destructor,
     Generator::build_traverser_work_nonnull_boxed_with

<1>8. CASE (F) の解放が作る活性化 `b` の節点が作る参照。
  `b` の入力の束縛は `o` の `_dtor` の欄の値と `_value` の欄の値である
  (`CODE src/generator.rs: Generator::build_run_destructor` -- `move_out_struct_field` で 2 つの欄を
  取り出し、`apply_lambda(dtor, vec![value], false)` を呼ぶ)。`<1>6a` より `o` は `t` において
  解放されておらず、D25 よりこの 2 つの値の boxed leaf が指すオブジェクトは `o` から到達できるので、L2 より
  それらも `t` において解放されていない。仮定より `B(b)` は D11 を満たすので、`b` の各節点について
  `<1>2`-`<1>7` の場合分けがそのまま当たる -- そこで新しい参照が指すオブジェクトは、`b` が読む値から
  到達できるもの (`<1>1a` と L2)、グローバル値が到達するもの (A8)、この段の中で割り当てられたもの
  (`<1>1c`) のいずれかである。
  BY A8, D11, D24 (F), D25, L2, <1>1a, <1>1c, <1>2, <1>3, <1>4, <1>5, <1>6, <1>6a, <1>7

<1>9. QED
  `<1>2`、`<1>3`、`<1>4`、`<1>5`、`<1>6` が `<1>1` の (E2) の 6 行を尽くし、`<1>7` が (F) の retain を、
  `<1>8` が (F) が作る活性化の節点を扱う。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>6a, <1>7, <1>8

### L3 (実行のすべての時点は勘定が合っている)

**言明**。プログラム `P` のすべての本体が D11 を満たすとする。`P` の実行 `ρ` のすべての時点は、勘定が
合っている。

<1>1. `ρ` の最初の時点は勘定が合っている。
  最初の時点では活性化もオブジェクトも無い。計数下のオブジェクトが無いので、定義の条件は空虚に成り立つ。
  BY D25, DEF 勘定が合っている時点

<1>2. 勘定が合っている時点の直後の 1 段の後の時点も、勘定が合っている。
  <2>1. CASE (E1) 環境が活性化を作る段。
    D24 の (E1) より、この段は作られる活性化 `a` の初期 `Obl` の各参照を `E` から渡し、A17 (i) より環境は
    それ以後それを持たない。参照は処分も生成もされず `H` も動かず、オブジェクトの割り当ても解放も
    起きないので、定義の条件は保たれる。
    BY A17, D24 (E1), DEF 勘定が合っている時点
  <2>2. CASE (E2) 節点の段。
    <3>1. 移動 (D9) の節点では、`Obl(a)` も `H` も変わらない。
      移動は「参照の持ち手が活性化の中で変わるだけ」であって義務集合を変えない。
      BY D9, D10 (移動)
    <3>2. `Obl(a)` を離れて行き先のある参照については、参照は処分されず `H` も変わらない。
      BY D24 (E2 の表)
    <3>3. `Obl(a)` を離れて処分される参照については、その参照が 1 つ消え、`H` がちょうど 1 下がる。
      BY D24 (E2 の表)
    <3>4. `<3>3` の処分で `H` が 0 になった計数下のオブジェクト `o` については、この段の中で (F) の解放が
          起きる。段の終わりには `H = 0` の計数下のオブジェクトはすべて解放されている。
      D24 の (F) が「ある段が参照を処分して計数下のオブジェクト (D26) `o` の `H(o)` が 0 になったとき、
      `o` は**その同じ段の中で解放される**」「その処分で `H` が 0 になったオブジェクトも、同じ段の中で
      同じように解放される。この連鎖はオブジェクトの個数で抑えられるので有限で終わり、段の終わりには
      `H = 0` の計数下のオブジェクトはすべて解放されている」と述べる。
      BY D24 (F)
    <3>4a. `<3>4` の解放は、`o` が `Destructor` のオブジェクトであるとき、`_dtor` の欄への retain を
           1 つ行い、活性化 `b` を作る。`b` の節点は `Obl(b)` と `H` を、`b` の本体についての (E2) の
           規則のとおりに動かし、`b` が終わる (E4) の段で `Obl(b)` を離れる参照はこの段を実行している
           活性化の `Obl` に入る (L1 (b))。`b` はこの段の中で終わる。
      BY D24 (F), L1
    <3>4b. 節点が適用する `Llvm` の段 (L0a) であるとき、その段は活性化を作り、それらが終わる (E4) の段で
           `Obl` を離れる参照はこの段を実行している活性化の `Obl` に入る (L1 (b))。参照は処分も生成も
           されず `H` も変わらない。**その (E4) の段で入る参照が結果の leaf に置かれる分は、この (E2) の
           段が新しく作る参照ではない。**
      BY D24 (E4), L0a, L1
    <3>5. 生成の節点では、生じる参照 1 つごとに `H` がちょうど 1 上がるか、新しいオブジェクトが `H = 1`
          で作られる。`<3>4b` が除いた leaf はここに数えない。
      BY D24 (E2 の生成の表), <3>4b
    <3>6. `H` が上がる先はこの段の時点で解放されていない。
      この言明の仮定よりこの段を持つ活性化の本体は D11 を満たし、`<3>4a` が作る活性化の本体も D11 を
      満たす。よって L2b が、`<3>5` と `<3>4a` が作る各参照について、その対象がこの段の直前の時点で
      解放されていないことを与える。
      BY D11, L2b, <3>4a, <3>5
    <3>7. QED
      この段で `H` が下がるのは参照の処分によるときだけである -- `<3>3` の処分、`<3>4` の解放が行う処分、
      `<3>4a` の活性化の節点が行う処分の 3 つがそれである。どの処分で 0 になったものも `<3>4` の引く
      D24 の (F) によりこの段の中で解放され、段の終わりには `H = 0` の計数下のオブジェクトはすべて
      解放されている。`H` が上がるのは `<3>5` の生成と `<3>4a` の retain によるときだけであり、`<3>6`
      よりその対象は解放されていないので、`H` が上がることで「解放されているのに `H ≥ 1`」になることは
      無い。よって段の終わりにも「`H = 0`」と「解放されている」は一致する。
      BY D24 (F), L2b, <3>3, <3>4, <3>4a, <3>4b, <3>5, <3>6
  <2>3. CASE (E3) 呼び出しの段。
    L1 (a) より、`Obl(a)` を離れる参照はそのまま `Obl(b)` の初期値になる。参照は処分も生成もされず `H` も
    動かず、割り当ても解放も起きないので、条件は保たれる。
    BY L1, D24 (E3), DEF 勘定が合っている時点
  <2>4. CASE (E4) 返りの段。
    L1 (b) より、`Obl(b)` を離れる参照はそのまま行き先に入る。参照は処分も生成もされず `H` も動かず、
    割り当ても解放も起きないので、条件は保たれる。
    BY L1, D24 (E4), DEF 勘定が合っている時点
  <2>5. CASE (E5) グローバル化の段。
    この段は印を付けるだけで、参照も `H` も動かさない。印の付いたオブジェクトは D26 より以後グローバル
    状態であり、定義が量化する計数下のオブジェクトから外れる。残る計数下のオブジェクトについて `H` も
    解放も動かないので、条件は保たれる。
    BY D24 (E5), D26, DEF 勘定が合っている時点
  <2>6. CASE (E6) 中断の段。
    この段の後に時点は無い。
    BY D24 (E6), D31
  <2>6a. CASE (E7) グローバルの初期化の段。
    D24 の (E7) より、この段は `Obl` が空の活性化を作るだけである。参照は処分も生成もされず、`H` も
    動かず、割り当ても解放も起きないので、条件は保たれる。
    BY D24 (E7), DEF 勘定が合っている時点
  <2>7. QED
    D24 は段を (E1)-(E7) の 7 種に尽くす。活性化を作る段のうち適用する `Llvm` の段は (E2) の一部で
    あり、`<2>2` の `<3>4b` がそれを扱う。(F) の解放は段ではなく段の一部なので独立した場合を持たず、
    参照を処分する段の中で起きる。参照を処分する段は (E2) だけである -- `<2>1`、`<2>3`、`<2>4`、
    `<2>5`、`<2>6`、`<2>6a` の段はどれも参照を処分しない (D24 の (E1)、(E3)、(E4)、(E5)、(E6)、(E7))。
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
  参照が入る先 -- L1 (b) より、終わった活性化を (E3) か適用する `Llvm` の段か (F) の解放が作ったなら
  その親の `Obl`、(E1) か (E7) が作ったなら `E` -- である。
  BY D24, L1

<1>2a. その間、`a` 自身が終わる (E4) の段は無い。
  `a` 自身が終わる (E4) は `a` が `B(a)` の終端の `Ret` に着く段であり (D23)、それは `a` が段を持つときで
  ある。`<1>1` よりその間 `a` は段を持たない。
  BY D23, D24 (E4), <1>1

<1>3. その間に (E4) の段で終わる活性化のうち、参照を `Obl(a)` へ入れるものは無い。
  `<1>2` より、参照が `Obl(a)` へ入るのは、終わる活性化を `a` の (E3) の段、`a` の適用する `Llvm` の段、
  または `a` の段の中で起きた (F) の解放が作った場合だけである。`a` がその 3 種のどれかを行うのは `a` が
  段を持つときであり、`<1>1` よりその間 `a` は段を持たないので、その間に `a` が新しい子を作ることは
  無い。`a` が `b` より前に作った子は、D24 の (E3)、D24 の (E7)、D24 の (F)、L0a (a) より、それぞれの
  中断が終わった時点ですでに終わっている。よってその間に終わりうる `a` の子は `b` だけであり、`b` の
  (E4) はこの区間の終わりである。
  BY D24 (E3), D24 (E7), D24 (F), L0a, <1>1, <1>2

<1>4. QED
  BY <1>1, <1>2, <1>2a, <1>3

**注 (借用した unit のオブジェクト)**。L4 が言うのは、`b` の活性化の間ずっと `a` が持ち続ける参照がある、
ということである。呼び出し先が**借用する** unit のオブジェクトが `b` の実行中に解放されないのはこれによる --
D9 の `App` の行は借用位置の引数の leaf を消費しないので、その参照は `Obl(a)` に残り、`H` の分解と L3 より
対象は解放されない。`Ownership::Borrow` の doc が「どちらの側も参照カウント操作を行わない」と言うのはこの
規律である (`CODE src/rc_ir/ast.rs: Ownership`)。(R1) の証明はこの経路を通らない -- (S-c) が `b` の本体に
ついて直接に読みの安全を与えるからである。L4 が支えているのは、その (S-c) が `borrow_ify` の出力について
実際に成り立ちうるということであって、P27 の段ではない。

### L5 (正常終了する実行について)

**言明**。プログラム `P` のすべての本体が D11 を満たすとし、`ρ` を `P` の正常終了する実行 (D31) とする。
このとき次が成り立つ。

- **(a)** `ρ` のすべての活性化が終わっている。
- **(b)** 各活性化 `a` が終わった時点で `Obl(a)` は空である。
- **(c)** `ρ` の最後の時点で、処分されていないどの参照の持ち手も、生きているオブジェクトか環境である。
- **(d)** `ρ` の活性化の林 (D24) は有限であり、`ρ` に現れるオブジェクトは有限個である。

<1>1. (a) が成り立つ。
  D31 より、正常終了する実行の最後の時点で生きている活性化は無い。D23 より、生きている活性化とは始まって
  終わっていない活性化である。よって始まった活性化はすべて終わっている。
  BY D23, D31

<1>3. (b) が成り立つ。
  活性化 `a` が終わるのは `B(a)` の終端の `Ret` に着いてその消費を行うときである (D23)。`a` が辿った
  節点の列は `B(a)` の実行路 (D3) であり (D21, D23)、仮定より `B(a)` は D11 を満たすので、(S-b) が
  「終端の `Ret` の消費を行った後の `Obl` は空である」を与える。
  BY D3, D11 (S-b), D21, D23

<1>4. (c) が成り立つ。
  `<1>1` より最後の時点で生きている活性化は無いので、D25 が挙げる 3 つの持ち手のうち活性化は残らない。
  P28 より、処分されていない各参照はちょうど 1 つの持ち手を持つ。
  BY D25, P28, <1>1

<1>4a. (d) が成り立つ。
  <2>1. 林の根は有限個である。
    根は (E1) の段が作る活性化である (D24 の活性化の林)。D31 より正常終了する実行の段の列は有限なので、
    (E1) の段も有限個である。
    BY D24 (活性化の林), D31
  <2>2. 各活性化が作る活性化は有限個である。
    D31 の後の段落が「D2 より本体は有限の木で、D3 の実行路は有限なので、1 つの活性化が終わるまでに持つ段は
    有限であり、1 つの活性化が作る活性化も有限個である」と述べる。
    BY D31
  <2>3. 林に無限の道は無い。
    D31 の後の段落が「発散する実行では、活性化の入れ子が無限に深くなるか、ある段が終わらないかの
    どちらかが起きている」と述べる。林の無限の道は活性化の入れ子が無限に深くなることであり、`ρ` は
    D31 の意味で正常終了するので発散ではない。
    BY D31
  <2>4. 林は有限である。
    林が無限とする。`<2>1` より根は有限個なので、子孫を無限に持つ根が在る。子孫を無限に持つ活性化 `a` に
    ついて、`<2>2` より `a` の子は有限個なので、そのうち子孫を無限に持つものが在る。これを繰り返すと
    林の無限の道が得られ、`<2>3` に反する。
    BY <2>1, <2>2, <2>3
  <2>5. 各活性化が割り当てるオブジェクトは有限個である。
    活性化が訪れる節点の列はその本体の実行路 (D3) であり (D21)、D2 より本体は有限の木なので実行路は
    有限である。1 つの節点の実行が割り当てるのは、D24 の (E2) の生成の表で割り当てを行う 2 行 --
    `Closure` の行 (1 つ) と `Llvm` の行の単一の `Fresh` の場合 (結果の型の boxed leaf ごとに 1 つ) --
    に限り、A10 より `boxed_leaf_paths` は有限の列である。
    BY A10, D2, D3, D21, D24 (E2)
  <2>6. QED
    オブジェクトを割り当てるのは、D24 の (E2) の生成の表で割り当てを行う 2 行だけである -- (E1)、(E3)、
    (E4)、(E5)、(E7) はどれも割り当てを持たず、(E6) の後に段は無く、(F) の解放は参照の処分と記憶域の
    返却と `Destructor` の活性化からなり、その活性化の節点の実行もまた (E2) の生成の表に従う。よって
    どの割り当てもある活性化の節点の実行に属する。`<2>4` より活性化は有限個であり、`<2>5` より各活性化が
    割り当てるオブジェクトは有限個なので、`ρ` に現れるオブジェクトは有限個である。
    BY D24, <2>4, <2>5

<1>5. QED
  BY <1>1, <1>3, <1>4, <1>4a

## 4. P27 の証明

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
      `<3>1` の参照は `E` に在るので、`H` の分解より `H ≥ 1` であり、L3 より解放されていない。
      グローバル状態のオブジェクトなら A8 より解放されない。
      BY A8, D8, D25, L3, P28, <3>1
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
    オブジェクトである。前者は `H` の分解と L3 より解放されておらず (グローバル状態なら A8 より解放
    されない)、後者は L2 と L3 より解放されていない。
    BY A8, A17, D8, D25, L2, L3, P28
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
    作られるオブジェクトはその段の直前の時点で解放されていない。`o` は解放された後ずっと解放されたままで
    ある (D24 に解放を取り消す段は無い)。
    BY D12, D24, L2b, L3
  <2>4. `o` が解放された時点の後、`H(o)` は 0 のままである。
    `H` の分解より `H(o)` は `o` への処分されていない参照の総数である。解放の時点でそれは 0 であり
    (`<2>2` と L3)、`<2>3` より新しい参照は作られず、無い参照は処分できないので減りもしない。
    BY D8, D25, L3, P28, <2>2, <2>3
  <2>5. QED
    `<2>4` より `o` の `H` が 0 に**なる**ことは解放の後には無いので、`<2>2` より 2 度目の解放は無い。
    グローバル状態のオブジェクトについては `<2>1` が言明より強いことを言う。
    BY <2>1, <2>2, <2>4

<1>3. ASSUME  `ρ` は正常終了する実行 (D31) である
      PROVE   `ρ` の最後の時点で解放されていない計数下のオブジェクト (D26) は、環境が最後に持つ参照が
              指すオブジェクトから到達できるものに限る。すなわち (R3) が成り立つ。
  <2>1. DEFINE `T` == `ρ` の最後の時点で解放されていない計数下のオブジェクトのうち、その時点で `E` が持つ
        参照が指すオブジェクトから到達できる (D25) もの。指されているオブジェクト自身も `T` に入れる。
        DEFINE `S` == `ρ` の最後の時点で解放されていない計数下のオブジェクト全体から `T` を除いたもの。
  <2>2. `S` の各元 `o` について、`o` への参照を持つ生きているオブジェクト `o'` が在り、`o'` は `S` の
        元である。
    <3>1. `o` は解放されていないので、L3 より `H(o) ≥ 1` であり、`H` の分解より `o` への処分されていない
          参照が在って、P28 よりそれはちょうど 1 つの持ち手を持つ。
      BY D8, D25, L3, P28, <2>1
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
      D25 より `o'` は生きているオブジェクト、すなわち割り当てられていて解放されていないオブジェクトで
      ある (`<3>4`)。計数下のオブジェクトであり (`<3>5`)、`T` の元ではない (`<3>6`)。よって `<2>1` より
      `o' ∈ S` である。
      BY D25, <2>1, <3>4, <3>5, <3>6
  <2>3. `S` は空である。
    `S` が空でないとする。L5 (d) より `ρ` に現れるオブジェクトは有限個なので、`S` は有限集合である。
    `<2>2` より `S` の各元は `S` の中に「自分を指す元」を持つので、`S` の元 `o_0` から `o_1`(`o_0` を
    指す)、`o_2`(`o_1` を指す)、… と `S` の中を限りなく遡れる。`S` が有限なのでこの列には同じ元が 2 度
    現れ、その間が閉路になる。A18 (a) がそれを禁じるので、`S` は空である。
    BY A18, D12, L5, <2>2
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

## 5. どの仮定が何を支えているか

| 主張 | 使う仮定 |
|---|---|
| P29 | A6、A11、A13 (L0b の (N1)-(N3) を果たす) |
| L0 | A6、A11、A13 と、P9・P10・P11・P12・P29 の言明 |
| (R1) | D11 の (S-b) と (S-c)、A3 と A4 (D24 の生成の表と L2b を通じて)、A5 (D25 を通じて)、A8、A12、A14 (L1 を通じて)、A16 (アームがタグを尽くすこと。L2b の `<1>6` の boxed union の場合)、A17、および P28 の言明 |
| (R2) | (R1) が使うもののすべて |
| (R3) | (R2) が使うもののすべてと **A18 (a)(b)**、および A10 と D2 (オブジェクトが有限個であること。L5 (d) を通じて) |
| 系 | (R3) が使うもののすべて |

**(R2) はもう仮定を足さない。**かつてこの文書は「`Retain` は解放されたオブジェクトに触れない」を仮定として
要求していた。D11 の (S-c) が `Retain(v, π)` と `Release(v, π)` の触れる先を扱うようになった後は、L2b の
`<1>1b` がそれを (S-c) から直接に出す。README の第 6 節「(S-c) を強めた記録」が、この強化がどの本体を
弾くようになったかを述べる。

L0 は P27 の証明のどのステップからも引かれない。L0 が支えているのは D23 の読み -- D9 と D10 の「呼び出し先」を
実行時の関数と読むこと -- が、`cancel` が静的に計算するものと食い違わない、ということである。食い違えば、
D11 を保存すると称する P14 と P23 が P27 の使う述語とは別の述語を保存していることになる。P29 は同じことを
`borrow_ify` の入力について述べ、層 1 の P7 の `App` の場合がそれを読む。

L0a は P27 の証明から L1、L3、L4 を通じて引かれる。支えているのは、活性化を作る段が (E1)、(E3)、(E7) の
3 つに尽きないこと -- オペランドを適用する `Llvm` の段も作ること -- と、その段が作った活性化が終わるときの
参照の行き先である。

## 6. 残っているもの

**(1) (E4) の行き先に、(F) の解放が作った活性化の分が無い。** (E4) の本文は

> 活性化 `b` が終端の `Ret(x)` に着く。D9 の終端の `Ret` の行が消費する参照 -- `x` の inhabited な全
> boxed leaf の参照 -- が `Obl(b)` を離れ、`b` を作った (E3) の呼び出し元 `a` の `Obl(a)` に入る。`a` は
> `k` の位置で再開する。`b` を作ったのが環境 (E1) であれば、それらの参照は `E` に入る。

であり、続く段落がオペランドを適用する `Llvm` の段の場合を足す。(F) の解放が作った活性化については、
どの本文も行き先を述べない。L1 の `<1>6` の `<2>3b` はそれを (F) の本文と `build_run_destructor` の
コードから出す -- 適用の結果は `run_io_or_ios_runner` を経て `_value` の欄へ書き戻される。

足す文は 1 つである。(E4) に「`b` を作ったのが (F) の解放であれば、それらの参照はその解放を含む段を
実行している活性化の `Obl` に入り、その段は続けてそれを `o` の `_value` の欄へ書き込む」。

**(2) (F) の解放が作る活性化の内側に D24 の時点が無い。** (F) の本文は

> **(F) が作る活性化はその段の中で終わるので、その内側に D24 の時点は無い。** 解放が段ではなく段の一部で
> あるのは、そうしないと「`H` が 0 になったが解放されていない」時点ができてしまうからである。

と述べる。この 2 つ目の文が理由であり、`H(o) = 0` でありながら `o` がまだ解放されていない時点を作らない
ためにこう書かれている。ところがこの読みの下では、その活性化の節点が行う割り当て・retain・release に
ついて、勘定が合っていることを段の水準の帰納で追えない -- 追う先の時点が無いからである。L2b の `<1>8` と
L3 の `<3>4a` はそこを、活性化の本体が D11 を満たすことと、その活性化が読む値が `o` から到達できることの
2 つで埋めており、時点の列の上の帰納にはなっていない。

足す文の形は 2 つ考えられる。1 つは、(F) が作る活性化の各節点の実行も段とし、勘定の条件を「`H(o) = 0` で
ありながら解放されていない計数下のオブジェクトは、その時点でどれかの (F) が解放しつつあるものに限る」へ
広げること。もう 1 つは、(F) が作る活性化について、その節点が触れるオブジェクトが解放されていないことを
D24 の側で述べること。どちらを取るかは定義の判断なので、ここでは述べるだけにする。

**(3) 環境 (D22) の数え上げに、`FFI_CALL` が届く C のコードが無い。** D22 は「環境は次の 3 つからなる」と
して C のエントリ点、`FFI_EXPORT` のエントリ点、グローバルのアクセサを挙げる。L2b の `<1>5` の `<2>2` の
`<3>0` は、`Std::FFI::boxed_to_retained_ptr` が参照を持ったまま番地を渡した先の C のコードを、参照を持つ
環境として読む。この読みを D22 が支えるには、`FFI_CALL` が呼ぶ C の関数も環境に数える行が要る。A17 の
3 つの契約はその読みのままで当たる。

**(4) 「最上位の記号の名前は局所名ではない」を述べる者が居ない。** L0b の (N3) を果たす段 (P29 の証明の
`<1>3`、L0 の `<1>3`) と、L0 の `<2>4c` の `<3>3` がこれを読む。この文書はそれを
`Lowerer::lower_var` の表明 -- 束縛の解けない名前について `assert!(!v.name.is_local())` -- から取って
いるが、それは lowering が変数参照について立てる表明であって、`Program::global_types` の鍵の集合について
の言明ではない。A12 の「束縛を持たない `RcVar` の型が、その名前の記号の型であること」の隣に置くのが
自然である。

**(5) `proof_links.py` が README を読まない。**`citations_of` が `CODE` を集めるのは `p*.md` からだけで
あり、README の本文と `llvmgen-function-values.md` の `CODE` は `citations.tsv` にも `// PROOF:` の注釈にも
入らない (`dev-docs/proof/proof_links.py` の `citations_of`)。この文書から README へ移った定義が引くコード --
`Generator::mark_global`、`Generator::build_release_boxed_with`、`build_free_boxed`、
`Generator::build_traverser_work`、`build_main_function`、`run_ios_runner`、`ExportStatement::implement`、
`InlineLLVMUndefinedInternalBody::generate` -- のうち、他の `p*.md` が引かないものがそれである。定義が引く
コードにもリンクを張るなら、走査の対象に README と `llvmgen-function-values.md` を足すことになる。
