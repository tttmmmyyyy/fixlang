# P29 と P27 -- 静的に決めた呼び出し先と、実行の合成

この文書は `README.md` の P29 と P27 を証明する。対象コミットは
`e8eda4718cdae4d0927dbbb60c15299dbcc23ad5` である。

**実行のモデルは `README.md` が持つ。**環境 (D22)、活性化 (D23)、実行 (D24)、参照の持ち手 (D25)、
計数下のオブジェクトとグローバル状態のオブジェクト (D26)、実行の終わり方 (D31)、実行の読み (D32) は、
`README.md` の第 3.6 節が定める。この文書はそれを引くだけで、実行のモデルを自分では置かない。この文書が
自分で置くのは、局所定義「本体の変数表」「`cancel` の所有述語」「勘定が合っている時点」「`H` の分解」
「段の素動作と段内の点」「解放が閉じている点」、および補題 L0d、L0b、L0c、L0、L0a、L1、L2、L2b、L3、L5
だけである。

P29 は層 0 の命題であり、その証明はどの命題も引かない。P27 の証明が引く命題は P28 の**言明**だけである --
参照の持ち手がちょうど 1 つであることを、下の「`H` の分解」が読む。P27 が使うのは D11 と D12 という述語
そのものであって、それを `borrow_ify` と `cancel` が保存することではない。補題 L0 は `cancel` が読む所有に
ついての別の主張であり、L0b と L0c と L0d、および P1、P9、P12、P24 の**言明**を引く。

## 1. 結論

| 主張 | 結果 |
|---|---|
| P29 静的に決めた呼び出し先は実行時の呼び出し先である | 証明した (L0b を入力に当てる) |
| L0 (README の P29 (b)) `cancel` が読む所有は実行時の呼び出し先の所有である | 証明した (L0b を出力に当てる) |
| (R1) 解放されたオブジェクトの読みが起きない | 証明した |
| (R2) どのオブジェクトも高々 1 回しか解放されない | 証明した |
| (R3) 正常終了する実行で解放されずに残る計数下のオブジェクトは、環境が持つ参照が指すオブジェクトから到達できるものに限る | 証明した |

**(R2) は仮定を 1 つも足さずに閉じる。**D11 の (S-c) が `Retain(v, π)` と `Release(v, π)` の**触れる**先を
扱うので、L2b の `Retain` の場合が (S-c) から直接に出る。

**関数の値に番地を書き込む段の数え上げは A21 が持つ。**この文書はそれを引く。`llvmgen-function-values.md`
が A21 の果たす者の数え上げである。

**残っているものの節が 3 つを挙げる。**L1 (a) が置く仮説 (ABI) を果たす節が枠に無いこと、D11 の (S-c) の
量化が D24 の段内の点へ届いていないこと、`proof_links.py` の走査が `llvmgen-function-values.md` を
読まないことである。

## 2. 静的に決めた呼び出し先

### L0d (名前で得た LLVM 関数が実装する `RcFunc`)

**L0d.** ASSUME  NEW `Q`: RcProgram、
                 **(N3)** `Q` の `funcs` の各鍵の名前は局所名 (`FullName::is_local` が真) ではなく、
                 コード生成が読む `global_types` はその名前を持たないか、funptr 型で持つ、
                 A22
         PROVE   **(a)** `implement_rc_program` が作る表 `func_vals` は、`Q.funcs` の各鍵 `fref` に
                 対して、`Q.funcs[fref]` の本体を実装した LLVM 関数を与える。
                 **(b)** `Q.funcs` の鍵の名前 `n` について、コード生成が `n` の値として返すのは
                 `declare_lambda_function` が funptr 型で登録した LLVM 関数そのもの (アクセサの呼び出し
                 ではない) であり、それは `func_vals[FuncRef{n}]`、すなわち `Q.funcs[FuncRef{n}]` の
                 本体を実装した LLVM 関数である。

<1>1. (a) が成り立つ。
  `implement_rc_program` は `Q.funcs` の各 `(fref, func)` について LLVM 関数を 1 つ決めて `func_vals` に
  入れ、続く走査で `implement_rc_function(func, func_vals[fref], ..)` を呼ぶ。`implement_rc_function` は
  その LLVM 関数に `entry` ブロックを付けて `func.body` を出す。
  BY CODE src/rc_ir/codegen.rs: Generator::implement_rc_program,
     CODE src/rc_ir/codegen.rs: Generator::implement_rc_function

<1>2. コード生成が `n` の値として返すのは、`declared_globals` に `n` で登録された `ValueAccessor` が
      答える値である。
  変数を読む腕は `get_scoped_obj(&n)` を呼び、`get_scoped_obj` は `get_scoped_value` の答えの
  `accessor` に `get` を掛ける。`get_scoped_value` は `var.is_local()` のときだけスコープを引き、
  そうでなければ `get_or_declare_global` を通る。(N3) より `n` は局所名ではないので後者である。
  `get_or_declare_global` は `declared_globals` に在ればそれを返し、無ければ `declare_program_global` を
  呼んで登録させ、それが `None` を返せば panic する。
  BY (N3), CODE src/generator.rs: Generator::get_scoped_obj, Generator::get_scoped_value,
     Generator::get_or_declare_global

<1>3. `n` について登録されるのは、`declare_lambda_function` が funptr 型で登録する場合だけである。
  `declared_globals` に名前を登録するのは `add_global_object` だけであり、それを呼ぶのは
  `declare_program_global` (`global_types` の型が funptr でないとき、そのアクセサ関数を登録する) と
  `declare_lambda_function` (`fn_ty.is_funptr()` のとき、いま作った LLVM 関数自身を登録する) の 2 か所で
  ある。同じ名前で 2 度登録すると `add_global_object` は abort する。
  `declare_program_global(n)` はまず `global_types.get(n)` を引き、`None` なら `None` を返す。(N3) より
  `n` は `global_types` に無いか funptr 型で在る。無ければ `declare_program_global` は `None` を返し、
  `<1>2` より `get_or_declare_global` は panic する -- そのときプログラムは走らず、この場合の段は存在
  しない。funptr 型で在れば `declare_program_global` は `is_funptr()` の枝を取り、
  `declare_lambda_function(&ty, n)` を呼ぶ。`declare_lambda_function` は `fn_ty.is_funptr()` のときだけ
  `add_global_object` を呼ぶので、アクセサとして登録される道は無い。
  BY (N3), <1>2, CODE src/generator.rs: Generator::add_global_object, Generator::declare_program_global,
     Generator::declare_lambda_function

<1>4. `<1>3` が登録する LLVM 関数は `func_vals[FuncRef{n}]` と同じものである。
  `declare_lambda_function(fn_ty, name)` が作る関数の記号名は `object_file_symbol_name(name)` である。
  `implement_rc_program` は `Q.funcs` の各項目 `(fref, func)` について `func.name.name` を取り、まず
  `module.get_function(object_file_symbol_name(func.name.name))` を引き、在ればそれを使い、無ければ
  `declare_program_global` を、それも `None` なら `declare_lambda_function` を呼ぶ。A22 より `func.name`
  は鍵 `fref` に等しいので、鍵 `FuncRef{n}` の項目について取る名前は `n` である。`<1>3` より `n` に
  ついて `declare_lambda_function` が呼ばれるのは funptr 型のときであり、`add_global_object` が同じ名前の
  2 度目の登録で abort するので、この名前について `declare_lambda_function` が呼ばれるのは高々 1 度で
  ある。よって `implement_rc_program` の 3 つの枝のどれを通っても、得られる LLVM 関数は
  `object_file_symbol_name(n)` を記号名とするその 1 つである。
  BY A22, <1>3, CODE src/generator.rs: Generator::declare_lambda_function, Generator::add_global_object,
     CODE src/rc_ir/codegen.rs: Generator::implement_rc_program

<1>5. (b) が成り立つ。
  `<1>3` の登録は `ValueAccessor::Global(fun, ty)` であり、`ty.is_funptr()` なので `ValueAccessor::get`
  は `fun.as_global_value().as_basic_value_enum()` を返す -- アクセサを呼ばず、関数そのものを値とする。
  `<1>4` よりその関数は `func_vals[FuncRef{n}]` であり、`<1>1` よりそれは `Q.funcs[FuncRef{n}]` の本体を
  実装した LLVM 関数である。
  BY <1>1, <1>2, <1>3, <1>4, CODE src/generator.rs: ValueAccessor::get

<1>6. QED
  BY <1>1, <1>5

### L0b (名前が決める呼び出し先)

**DEF 本体の変数表**
`vars` は、その節点を含む本体の**変数表** `VarTable` である。関数の本体については
`VarTable::of(func)` が、グローバル初期化子の `init` については `VarTable::body_only(init)` が作る
(`CODE src/rc_ir/ownership.rs: VarTable`)。`VarTable::of` はその関数の各パラメータと capture に
`Binding::Param` を置き、続けて `collect_bindings` を本体に掛ける。`VarTable::body_only` は
`collect_bindings` だけを掛ける。`collect_bindings` は、本体の各 `Let(x, rhs, k)` について `x` に
`rhs` の形が決める `Binding` を置き、`rhs` が `Closure(fref, _)` のときは `closure_targets` に
`(x.name, fref)` を挿入し、各 `Destructure` のフィールド変数と各 `Match` アームの payload 変数にも
`Binding` を置く (`CODE src/rc_ir/ownership.rs: collect_bindings`)。以下で `vars.closure_targets` と
`vars.bindings` と言うのはこの表の欄である。

**L0b.** ASSUME  NEW `Q`: RcProgram、
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

<1>1. `resolve_callee_params(callee, vars, Q)` が `Some(params)` を返すのは 2 つの場合であり、
      `None` を返すのはそれ以外である。
  1 つは `vars.closure_targets` が `callee.name` を持つ場合、もう 1 つは `Q.funcs` が
  `FuncRef { name: callee.name }` を持つ場合である。どちらでもなければ `?` が `None` を返す。どちらの
  場合も返るのは `Q.funcs` のその `FuncRef` の関数の `params` である。
  BY CODE src/rc_ir/ownership.rs: resolve_callee_params

<1>2. `vars.closure_targets` が `callee.name` を持つのは、この本体に `Let(callee, Closure(fref, caps), k)`
      が在るとき、かつそのときに限る。そのとき返るのは `Q.funcs[fref]` のパラメータである。
  `collect_bindings` が `closure_targets` に挿入するのは `RcRhs::Closure(fref, _)` の腕の 1 か所だけで
  あり、挿入する鍵はその `Let` の束縛変数の名前である (DEF 本体の変数表)。
  BY <1>1, DEF 本体の変数表, CODE src/rc_ir/ownership.rs: collect_bindings, resolve_callee_params

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
  <2>2. その値は funptr の欄に `func_vals[fref]` を持つクロージャである。
    `Let` の右辺が `Closure(fref, caps)` のとき、`eval_rc_rhs` は `build_rc_closure` を呼び、
    `build_rc_closure` は `CLOSURE_FUNPTR_IDX` の欄に `func_vals[func]` を入れる。
    BY <2>1, CODE src/rc_ir/codegen.rs: Generator::eval_rc_rhs, Generator::build_rc_closure
  <2>3. QED
    D23 より、`callee` の値がクロージャのとき実行時の呼び出し先はその funptr の指す関数である
    (`apply_lambda` は `get_lambda_func_ptr` が返す関数ポインタを `build_indirect_call` で呼び、
    `get_lambda_func_ptr` は `is_closure()` のとき `CLOSURE_FUNPTR_IDX` の欄を読む)。L0d (a) より
    `func_vals[fref]` は `Q.funcs[fref]` の本体を実装した LLVM 関数である。
    BY D23, L0d, <2>2, CODE src/generator.rs: Generator::apply_lambda,
       CODE src/generator.rs: Generator::get_lambda_func_ptr

<1>4. `Q.funcs` が `FuncRef { name: callee.name }` を持つ場合、実行時の呼び出し先はその関数である。
  <2>1. コード生成が `callee.name` の値として返すのは、`Q.funcs[FuncRef{callee.name}]` の本体を実装した
        LLVM 関数そのものであり、その値は funptr である。
    L0d (b) を `n = callee.name` に当てる。`App` の腕はその名前を `get_scoped_obj` で引く。
    BY L0d, CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner,
       CODE src/generator.rs: Generator::get_scoped_obj
  <2>2. QED
    D23 より、`callee` の値が funptr のとき実行時の呼び出し先はそれ自身である
    (`get_lambda_func_ptr` は `is_funptr()` のとき値そのものを関数ポインタとする)。
    BY D23, <2>1, CODE src/generator.rs: Generator::get_lambda_func_ptr

<1>5. QED
  `<1>1` より `Some(params)` が返るのは 2 つの場合であり、どちらでも `params` は `Q.funcs` のある
  `FuncRef` の関数 `g` のパラメータの列である。`<1>3` と `<1>4` が、その 2 つの場合のそれぞれについて、
  `g` が実行時の呼び出し先であることを与える。`params` も `g.borrowed_units` も `g` の欄なので、
  実行時の呼び出し先のものである。
  BY <1>1, <1>2, <1>3, <1>4

### L0c (入力の関数の名前が (N3) を満たす)

**言明**。A13 を満たす `borrow_ify` の入力プログラム `P` について、`P.funcs` の各鍵の名前は局所名では
なく、コード生成が読む `global_types` はその名前を持たないか、funptr 型で持つ。

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
  A13 が「最上位の記号の名前は局所名ではない。`FullName::is_local` が偽であり、`prog.funcs` の鍵と
  `global_types` の鍵はどちらもそのような名前である」を与える。
  BY A13

<1>3. コード生成が読む `global_types` は、`P.funcs` の鍵の名前を持たないか、funptr 型で持つ。
  コード生成に渡る `global_types` は `global_types_including_synthesized` が作る。それは 3 系統の項目から
  なる -- `Program::global_types` (最上位の記号の名前からその型への写像) の全項目、`fn_ty` が funptr で
  ある `funcs` の各項目をその `fn_ty` で上書き挿入したもの、そして `program.globals` の各要素をその
  `symbol` と `ty` で上書き挿入したものである。第 2 系統より、funptr の関数の名前は funptr 型で在る。
  funptr でない関数の名前について、第 2 系統は項目を作らないので、残る 2 系統を見ればよい。`<1>1` より
  鍵と `symbol` は lowering の出力のものであり、`lower_symbol` は funptr 型の記号を `funcs` の鍵にし、
  それ以外の型の記号をグローバル初期化子の `symbol` にする。よって funptr でない関数の名前は最上位の
  記号の名前でも初期化子の `symbol` でもなく、第 1 系統にも第 3 系統にも項目が無い -- 持ち上げた lambda に
  付ける鍵は `fresh_closure_ref` が `current_symbol` の名前空間の下に作る `closure#N` であり、その doc が
  「`<symbol>::closure#N` names no source-level value」と述べる。
  BY <1>1, CODE src/build/divide_program.rs: global_types_including_synthesized,
     CODE src/ast/program.rs: Program::global_types,
     CODE src/rc_ir/lower.rs: Lowerer::lower_symbol, Lowerer::fresh_closure_ref

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
  A6 が「`borrow_ify` の入力のすべての束縛変数の名前は相異なり、どの関数の名前とも異なる」を与え、
  A22 が各 `RcFunc` の `name` を `P.funcs` のその項目の鍵と同一視する。
  BY A6, A22

<1>2. `P` は L0b の (N2) を満たす。
  A11 が「変数の使用は、その位置でスコープに入っている束縛に解決する」を与える。
  BY A11

<1>3. `P` は L0b の (N3) を満たす。
  L0c がまさにこれを述べる。
  BY A13, L0c

<1>4. QED
  L0b を `Q = P` に当てる。A22 は L0b の仮定でもあり、この命題の仮定に在る。
  BY A22, L0b, <1>1, <1>2, <1>3

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

**言明**。A6、A11、A13、A22 を満たすプログラム `P` について、`borrow_ify` の出力プログラム `P'` のどの
`Let(x, App(callee, args), k)` と、その節点を含む本体の変数表 `vars` (DEF 本体の変数表) についても、
次の 2 つが成り立つ。

- **(a)** `resolve_callee_params(callee, vars, P')` が `Some(params)` を返すならば、`params` を持つ
  `P'.funcs` の関数はその段の実行時の呼び出し先 (D23) と同じ `RcFunc` である。`None` を返すならば、
  その段の実行時の呼び出し先の `borrowed_units` は空である。
- **(b)** `Some(params)` の場合、実行時の呼び出し先を `g` とすると、各 `i` と `ty(params[i])` の各 boxed
  leaf `λ` について、`owns_cancel(params[i], λ)` が真であることと、`g` が
  `truncate_to_unit(ty(params[i]), λ)` を D14 の意味で所有することは同値である。

**(a) の前半が README の P29 (b) である。** README は「同じことが `borrow_ify` の出力についても成り立つ」
とだけ書き、その内容をこの節に委ねている。(a) の後半 -- `None` を返す場合に実行時の呼び出し先の
`borrowed_units` が空であること -- は README の P29 が何も言わない部分であり、この文書が局所補題として
足す強化である。`cancel` の走査は `None` のとき全位置を所有として扱うので、その扱いが実行時の呼び出し先の
所有と食い違わないことを言うのがこの後半である。

<1>1. `P'` は L0b の (N1) を満たす。
  <2>1. `P'` の束縛変数の名前は、`P` の束縛変数の名前か、`clone_func` が作る複製の名前である。
    `borrow_ify` は出力の各版を、原本 `func.clone()` の本体を `RewriteCtx::rewrite` で書き換えたものか、
    `clone_func` の複製の本体を同じく書き換えたものとして作る。グローバル初期化子も同じ `rewrite` を
    通る。P24 の第 4 項より、書き換えが本体について変えるのは `Retain`/`Release` の節点と `App` の
    callee の名前だけであり、`Let` の束縛変数は元の本体のものに等しい。足される `Retain`/`Release` は
    束縛を持たない (D2)。
    BY D2, P24, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func
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
    `borrow_ify` が出力の `funcs` に入れるのは、原本を `f_own.name` で入れる項目と、複製を
    `borrow_version` で入れる項目だけである。A22 より前者の名前は `P.funcs` の鍵であり、後者は
    `borrow_funcref` が作る `<元の名前>#borrow` である。A13 より `P` に現れるどの名前も `#` で区切った
    最後の断片が `b<10 進数字>` の形ではないので、`P` の関数の名前は複製名と異なる。`#borrow` で終わる
    名前も、最後の断片が `b<10 進数字>` の形ではないので複製名とは異なる。
    BY A13, A22, CODE src/rc_ir/borrow.rs: borrow_ify, borrow_funcref
  <2>6. `P` の束縛変数の名前は、`P'` のどの関数の名前とも異なる。
    `<2>2` が `P` の関数の名前について与える。`#borrow` で終わる名前については A13 が、`P` に現れる
    どの名前も最後の断片が `borrow` ではないと言う。
    BY A13, <2>2
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6

<1>2. `P'` は L0b の (N2) を満たす。
  `borrow_ify` は出力の各本体を、原本 `func.clone()` の本体か `clone_func` の複製の本体を
  `RewriteCtx::rewrite` で写して作る。複製は束縛変数の一斉の付け替えであり、それ以外の違いを持たない
  (P9) ので、束縛と使用の対応を保つ。P24 の第 4 項より、書き換えが本体について変えるのは
  `Retain`/`Release` の節点と `App` の callee の名前だけであり、節点の種類・その順序・`Let` の束縛変数・
  `Match` のアームの構成は元の本体のものに等しい。落とす節点も足す節点も束縛を持たない (D2)。よって
  A11 が `P` について与えるスコープの規律は `P'` でも成り立つ。
  BY A11, D2, P9, P24, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

<1>3. `P'` は L0b の (N3) を満たす。
  <2>1. `P'.funcs` の鍵の名前は、`P.funcs` の鍵の名前か、`borrow_funcref` が作る `<元の名前>#borrow`
        である。また `P'.globals` の `symbol` の集合は `P.globals` のものに等しい。
    `borrow_ify` は出力の `funcs` に、原本を `f_own.name` で、複製を `borrow_version` で入れる。A22 より
    前者は `P.funcs` の鍵であり、後者は `borrow_funcref` が作る名前である。グローバル初期化子については
    P24 が「出力のグローバル初期化子の列は入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力の
    第 `i` 要素のものに等しい」と述べる。
    BY A22, P24, CODE src/rc_ir/borrow.rs: borrow_ify, borrow_funcref
  <2>1a. `global_types_including_synthesized(P', ・)` は、`global_types_including_synthesized(P, ・)` に
         比べて、`<元の名前>#borrow` を鍵とする項目だけを余分に持つ。
    `global_types_including_synthesized(prog, global_types)` は、引数の `global_types` (最上位の記号の
    名前からその型への写像) を写した上に、`prog.funcs` のうち `fn_ty.is_funptr()` である項目をその
    `fn_ty` で、`prog.globals` の各要素をその `symbol` と `ty` で上書き挿入する。引数の `global_types` は
    `borrow_ify` に渡される `RcProgram` から作られるものではないので `P` と `P'` で同じである。
    `<2>1` より `P'.funcs` の鍵は `P.funcs` の鍵と `borrow_funcref` が作る `<元の名前>#borrow` で尽き、
    `P'.globals` の `symbol` の集合は `P.globals` のものに等しい。
    BY <2>1, CODE src/build/divide_program.rs: global_types_including_synthesized
  <2>2. `P.funcs` の鍵の名前について (N3) が成り立つ。
    局所名でないことは A13 が与える。A13 より `P` に現れるどの名前も `#` で区切った最後の断片が
    `borrow` ではないので、`P.funcs` の鍵は `<元の名前>#borrow` の形の名前と異なる。よって `<2>1a` の
    余分な項目はこの鍵に当たらず、`P'` について読む `global_types` はこの名前について
    `global_types_including_synthesized(P, ・)` と同じ答えを返す。L0c がその答えについて (N3) を与える。
    BY A13, L0c, <2>1a, CODE src/rc_ir/borrow.rs: borrow_funcref
  <2>3. `<元の名前>#borrow` について (N3) が成り立つ。
    `borrow_funcref` は元の名前の `name` の欄に `#borrow` を足すだけなので名前空間は変わらず、A13 より
    局所名ではない。借用版の `fn_ty` は原本の `fn_ty` と等しい (`clone_func`) ので、funptr の借用版の
    名前は `global_types_including_synthesized` が funptr 型で挿入する。funptr でない借用版の名前に
    ついては、その 3 系統のどれもこの鍵の項目を作らないことを言えばよい。funptr の項目を入れる系統は
    `fn_ty.is_funptr()` の項目しか入れないので当たらない。残る 2 系統の鍵 -- 引数の `global_types` の鍵
    (最上位の記号の名前) と `P'.globals` の `symbol` -- は、`<2>1a` よりどちらも `borrow_ify` の入力の側の
    名前であり、A13 が `borrow_ify` の入力に現れるすべての名前について「`#` で区切った最後の断片は
    `borrow` ではない」と述べる。
    BY A13, <2>1a, CODE src/rc_ir/borrow.rs: borrow_funcref, clone_func,
       CODE src/build/divide_program.rs: global_types_including_synthesized
  <2>4. QED
    BY <2>1, <2>1a, <2>2, <2>3

<1>4. (a) の前半が成り立つ。
  L0b を `Q = P'` に当てる。A22 は `P'` についても成り立つ -- `borrow_ify` は原本を `f_own.name` で、
  複製を `borrow_version` で `funcs` に入れ、`clone_func` は複製の `name` をその `borrow_version` に
  する。
  BY A22, L0b, <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func

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
    BY A22, P12, CODE src/rc_ir/borrow.rs: RewriteCtx::route, borrow_ify,
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
  <2>4c. (M3) が置く `gc.current_function()` は、`P'` の関数のうち `capture` が `Some` であるものの
         本体を実装した LLVM 関数である。
    <3>1. `InlineLLVMFixBody` の `Llvm` 節点は `P'` の関数の本体にしか在らず、その関数の `capture` は
          `Some` である。
      A24 は `borrow_ify` の入力 `P` の各関数について「その本体に `InlineLLVMFixBody` の `Llvm` 節点が
      在るならば、その関数の `capture` は `Some` である」を述べる。`P'` の各版は、原本 `func.clone()` の
      本体か `clone_func` の複製の本体を `RewriteCtx::rewrite` で写して作られ、P24 の第 4 項より
      書き換えが本体について変えるのは `Retain`/`Release` の節点と `App` の callee の名前だけなので、
      `Llvm` の op は元の本体のものに等しい。P9 より複製は束縛変数の一斉の付け替えであり、`clone_func`
      は原本の `capture` を `fresh_rename_function` が付け替えた形で持つので、`Some` であることは
      保たれる。グローバル初期化子の `init` については、`fix_body` が op の `cap_name` を局所名
      `#CAP` (`CAP_NAME`) に置き、`lower_llvm` がその名前をその位置の環境で解いてオペランドに直す。
      `#CAP` を環境に束縛するのは `lower_lambda_as_function` の `lam_ty.is_closure()` の枝だけであり、
      その枝が束縛するのはその関数の capture 変数である。よってこの op のオペランドは、その本体を持つ
      関数の capture 変数であり、それがグローバル初期化子の `init` に在れば `init` の自由な局所名に
      なる。A11 は「グローバル初期化子の `init` は自由な局所名を持たない」と述べる。
      BY A11, A24, P9, P24, CODE src/rc_ir/borrow.rs: borrow_ify, clone_func,
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
  <2>5. QED
    `resolve_callee_params` が `None` を返すなら、`callee.name` は `P'.funcs` の鍵でも
    `closure_targets` の鍵でもない (`CODE src/rc_ir/ownership.rs: resolve_callee_params` -- 2 つの枝が
    どちらも当たらないときだけ `?` が `None` を返す)。実行時の呼び出し先は、`callee` の値がクロージャ
    ならその funptr が指す関数、funptr ならそれ自身であり、D23 はその関数がプログラムの `funcs` の
    関数であると述べる。その番地は `<2>4b` の 3 つのいずれかが置いたものである。
    (M1) が置くのは `func_vals[fref]` であり、L0d (a) よりそれは `P'.funcs[fref]` の本体を実装した
    LLVM 関数である。`<2>3` よりクロージャの目標 `fref` は入力から運ばれた名前、すなわち原本の名前で
    ある。
    (M2) が置くのは、funptr のグローバル `m` について `ValueAccessor::get` が返す LLVM 関数である。
    D23 より実行時の呼び出し先はプログラムの `funcs` の関数なので、`m` は `P'.funcs` の鍵である -- 鍵で
    なければ、L0d (a) が本体を実装するのは `funcs` の項目についてだけなので、その LLVM 関数はどの
    `RcFunc` の本体も実装していない。`<1>3` より `P'` は L0b の (N3) を満たすので、L0d (b) よりその
    LLVM 関数は `P'.funcs[FuncRef{m}]` の本体を実装したものである。`m` は `<2>4a` より借用版の名前では
    ない -- 借用版の名前は `App` の `callee` の位置にしか現れず、そこでは `<2>4` より
    `resolve_callee_params` が `Some` を返す -- のでやはり原本の名前である。
    (M3) の関数は `<2>4c` より `capture` を持つ `P'` の関数の LLVM 関数であり、`<2>1` より capture を
    持つ関数には借用版が作られないので原本である。
    `<2>2` より原本の `borrowed_units` は空である。
    BY D23, L0d, <1>3, <2>1, <2>2, <2>3, <2>4, <2>4a, <2>4b, <2>4c,
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
    λ)))` である (DEF `cancel` の所有述語)。`λ` は `ty(params[i])` の boxed leaf なので、A10 と P1 より
    `truncate_to_unit(ty(params[i]), λ)` は値を返し、それは `rc_units(ty(params[i]))` の元 -- すなわち
    `<2>2` の `u` が渡る集合の元 -- である。`<2>1` より `params[i]` は `g.params[i]` であり、`<2>3` より
    その名前で `owned_units` に入る項目は `g` が入れたものだけである。よって `<2>2` より、その項目が
    在ることと `truncate_to_unit(ty(params[i]), λ)` が `g.borrowed_units` に入らないことは同値であり、
    D14 より後者は `g` がその unit を所有することである。
    BY A10, D14, P1, <2>1, <2>2, <2>3, DEF `cancel` の所有述語

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
  宣言と生成コードが一致することは A3 が言う -- `applies_a_function_operand` は A3 が名指す 3 つの宣言の
  1 つである。
  BY A3, D24 (E2 の `Llvm` の段の段落),
     CODE src/ast/inline_llvm.rs: LLVMGen::applies_a_function_operand

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

**言明**。(a) は次の仮説の下で述べる。

> **(ABI)** 実行するプログラムの各関数 `f` について、`f.params` の型の列は `f.fn_ty` の lambda src の列に
> 等しく、`f.capture` が `Some` であることと `f.fn_ty.is_closure()` が真であることは同値であり、コード
> 生成が `f` に与える LLVM 関数の署名は `lambda_function_type(f.fn_ty)` である。さらに各
> `Let(x, App(callee, args), k)` の段について、`ty(callee)` はその段の実行時の呼び出し先 (D23) の
> `fn_ty` に等しい。

- **(a)** (ABI) の下で、(E3) の段で `Obl(a)` を離れる参照の多重集合は、`Obl(b)` の初期値に一致する。
- **(b)** (E4) の段で `Obl(b)` を離れる参照の多重集合は、その段の行き先 -- `b` を (E3) が作ったならその
  親の `Obl(a)`、適用する `Llvm` の段 (L0a) が作ったならその段を実行している活性化の `Obl`、(F) の解放が
  作ったならその解放を含む段を実行している活性化の `Obl`、(E1) か (E7) が作ったなら `E` -- が得る参照の
  多重集合に一致する。

**(a) は、D24 の (E3) と D10 の初期値が同じ多重集合を指すことの検算である。**D24 の (E3) は `Obl(a)` を
離れた参照がそのまま `Obl(b)` の初期値になると定め、D10 は `Obl(b)` の初期値を所有するパラメータ・
capture の leaf から独立に定める。L3 が (E3) の段について読むのは D24 の (E3) の定めだけなので、P27 は
(a) を読まない。**(ABI) を果たす節は枠に無い** -- 第 6 節がその形を述べる。(b) は (ABI) を要らない。

<1>1. (E3) の段で `Obl(a)` を離れるのは、D9 の `App` の行が挙げる leaf の参照である。すなわち `callee` の
      inhabited な全 boxed leaf と、呼び出し先がその位置の unit を所有する (D14) 引数の inhabited な leaf で
      ある。
  BY D9, D24 (E3)

<1>2. `Obl(b)` の初期値は、`b` の本体の関数が所有する (D14) パラメータ・capture の unit の下の inhabited な
      各 leaf につき 1 つである。
  BY D10 (初期値), D24 (E3)

<1>2a. 呼び出しが使う署名と、実行時の呼び出し先 `g` の LLVM 関数の署名は、どちらも
       `lambda_function_type(g.fn_ty)` である。
  `apply_lambda` は `func_ty = lambda_function_type(&fun.ty, self)` を作って `build_indirect_call` に
  渡し、`fun.ty` は `ty(callee)` である。(ABI) より `ty(callee)` は `g.fn_ty` に等しく、コード生成が
  `g` に与える LLVM 関数の署名も `lambda_function_type(g.fn_ty)` である。
  BY (ABI), CODE src/generator.rs: Generator::apply_lambda, CODE src/object.rs: lambda_function_type

<1>2b. `ty(callee)` がクロージャ型であることと、`g.capture` が `Some` であることは同値である。
  (ABI) より `ty(callee)` は `g.fn_ty` に等しく、`g.capture` が `Some` であることと
  `g.fn_ty.is_closure()` が真であることは同値である。
  BY (ABI)

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
  `lambda_return_part_types` を掛けた結果を読み、(ABI) より `ty(callee)` は `g.fn_ty` に等しいので
  真偽は一致する。(ABI) より `g.params` の型の列は `g.fn_ty` の lambda src の列、すなわち `ty(callee)` の
  lambda src の列に等しいので、parts の列も一致する。CAP の有無は `<1>2b` より一致する。
  BY (ABI), <1>2a, <1>2b, CODE src/generator.rs: Generator::apply_lambda,
     Generator::returns_through_out_pointer, CODE src/object.rs: lambda_function_type,
     lambda_return_part_types, CODE src/rc_ir/codegen.rs: Generator::implement_rc_function

<1>3. `callee` の inhabited な boxed leaf は、`b` の capture パラメータの inhabited な boxed leaf と
      1 対 1 に対応し、その unit は所有される。
  <2>1. `callee` の型がクロージャのとき、`boxed_leaf_paths` はその capture の位置 1 つだけを leaf とする。
    BY D4 (規則 2)
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
    BY D4, D5, CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function,
       CODE src/fixstd/builtin.rs: make_dynamic_object_ty, bulitin_tycons
  <2>4. capture の unit は所有される。
    D14 が「capture の unit は必ず所有される」を与える。
    BY D14
  <2>5. `callee` の型が funptr のとき、`callee` は boxed leaf を持たず、実行時の呼び出し先は `capture` を
        持たない。
    funptr の型は `is_fully_unboxed` が真であり (`is_funptr()` の枝が `true` を返す)、D4 の規則 1 で
    leaf を持たない。`<1>2b` より、`ty(callee)` がクロージャ型でないとき `g.capture` は `None` である。
    BY D4, <1>2b, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>6. QED
    D23 は `App(callee, args)` の実行時の呼び出し先を、`callee` の値がクロージャの場合はその funptr の
    指す関数、funptr の場合はそれ自身、の 2 つで定める。`<2>1`-`<2>4` が前者を、`<2>5` が後者を扱う。
    BY D23, <2>1, <2>2, <2>3, <2>4, <2>5

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
    段は存在しない。(ABI) より `ty(callee)` は `g.fn_ty` に等しく、`g.params` の型の列は `g.fn_ty` の
    lambda src の列なので、`args` の個数は `g.params` の個数に等しい。A14 が「呼び出し先」を D23 の
    実行時の関数についても読むと述べるのがこの一致である。
    BY (ABI), A14, CODE src/generator.rs: Generator::apply_lambda
  <2>2. 第 `i` 引数の型は `g.params[i]` の型に等しいので、両者の `boxed_leaf_paths` は同じ列であり、
        `<2>1` より同じ値の同じ leaf を指す。よって inhabited (D16) であることも一致する。
    A12 の「`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型」の行がこれを与える。
    A14 と同じく、ここでの「呼び出し先」は D23 の実行時の関数である。
    BY A12, A14, D4, D16, D23, <2>1, <2>1a
  <2>3. QED
    D9 の `App` の行と D10 の初期値は、どちらも同じ所有の割り当て (D14) を同じ関数 -- D23 が定める実行時の
    呼び出し先 -- について読む。`<2>1a` より引数とパラメータは 1 対 1 であり、余るパラメータは無い。よって
    消費される引数の leaf と、初期 `Obl` に入るパラメータの leaf は、`<2>2` の対応の下で同じ集合である。
    BY D9, D10, D14, D23, <2>1, <2>1a, <2>2

<1>5. (a) が成り立つ。
  BY (ABI), <1>1, <1>2, <1>3, <1>4

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
    D24 の (E4) が「`b` を作ったのが (F) の解放であれば、それらの参照はその解放を含む段を実行している
    活性化の `Obl` に入り、その段は続けてそれを `o` の `_value` の欄へ書き込む」と述べる。行き先が 1 つで、
    参照は処分も生成もされないので、その `Obl` が得る多重集合は `<2>1` のそれと等しい。この場合の `b` は
    2 つある -- `_dtor` の欄の関数を `_value` の欄の値へ適用するものと、それが返した `IO` の動作の runner を
    適用するものである (D24 の活性化の林) -- が、どちらの (E4) についても行き先の記述は同じである。
    BY D24 (E4), D24 (活性化の林), <2>1,
       CODE src/generator.rs: Generator::build_run_destructor
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

**この定義は 1 つの条件しか持たない。**`H(o)` が `o` への未処分の参照の総数であることは D8 が定める
(計数下のオブジェクトについて -- D26)。**参照の持ち手がちょうど 1 つであることは P28 が言い、P28 は
「D12 の意味で RC 規律を満たすプログラムの実行の各時点において」と量化する。**この 2 つと D25 の持ち手の
3 分から、各計数下オブジェクト `o` について

`H(o) = Σ_{生きている活性化 a} Obl(a)[o] + Σ_{生きているオブジェクト o'} R(o')[o] + E[o]`

が出る。ここで `R(o')[o]` は、`o'` が保持する値の inhabited な boxed leaf のうち `o` を指すものの個数で
ある。以下ではこの等式を **`H` の分解**と呼び、P28、D8、D25 を引いて使う。**`H` の分解を引く段は、その
プログラムが D12 を満たす場合にだけ在る** -- それを引く L3、L5、P27 はどれも D11 か D12 を言明の仮定に
持つ。

**L3 が示すのは、この定義のただ 1 つの条件である。**すなわち「`H` が 0 になったオブジェクトは解放される」
(D7) と「解放されたオブジェクトのカウントは 0 のままである」の 2 つが、実行のどの時点でも保たれる、という
ことである。

**DEF 段の素動作と段内の点**
D24 の (F) の段落は、1 つの段を**不可分な動作の有限列**へ分解する -- 参照の受け渡し、生成、割り当て、
処分、解放、グローバル化の 6 種である。この列の動作と動作のあいだの点、および列の最初と最後の点を、その
段の**段内の点**と呼ぶ。段と段のあいだの時点 (D24 の時点) は、その段の最初の点であり、直前の段の最後の
点である。**(F) の解放がその段の中で作る活性化 -- `_dtor` の適用と、それが返した `IO` の動作の runner の
適用 -- の節点が行う動作も、この列の元である。**D24 が「**(F) が作る活性化の中で起きる割り当て・
`Retain`・`Release`・入れ子の解放も、この段の一部である。**その活性化の節点は D24 の時点を持たないので、
そこでの勘定は時点の列の帰納では追えない。追うには段を不可分な動作の有限列へ分解する」と述べるのがこの
分解である。

**節点の位置についての条件は、その節点の段のどの段内の点にも当たるものとして読む。** D11 の (S-c) は
「その時点で解放されていない」を節点の位置について言い、D24 は 1 つの節点の段の素動作の列の中で読みと
処分がどう並ぶかを定めない。定めないまま (S-c) を段の入口だけで読むと、段の中で解放したオブジェクトを
同じ段の中で読む本体を (S-c) が縛らなくなり、(R1) が言う「解放されたオブジェクトの読みは起きない」も
その分だけ弱くなる。この文書は (S-c) を、その節点の段のどの段内の点についても読む。第 6 節がこの読みを
D11 の側に書く形を述べる。

**DEF 解放が閉じている点**
段内の点が**解放が閉じている**とは、その点で解放されている (D7) 各計数下のオブジェクト (D26) `o` に
ついて `H(o) = 0` であることをいう。同値な言い換えは「`H(o) ≥ 1` である計数下のオブジェクトはその点で
解放されていない」である。**勘定が合っている時点は解放が閉じている。**逆は言わない -- 段の途中には、
`H(o) = 0` でありながらまだ解放されていない `o` の在る点がある。

### L2 (到達できるオブジェクトは解放されていない)

**言明**。解放が閉じている段内の点において、オブジェクト `o'` が解放されておらず、`o'` から `o''` へ
到達できる (D25) ならば、`o''` も解放されていない。`o'` と `o''` は計数下でもグローバル状態でもよい。

<1>1. `o'` が解放されておらず、`o'` が `o` への参照を持つならば、`o` は解放されていない。
  `o` がグローバル状態ならば A8 より解放されない。`o` が計数下ならば、`o'` は割り当てられていて解放
  されていない -- すなわち D25 の意味で**生きている**オブジェクトである -- ので D25 の持ち手であり
  (D26 は D8 の参照を計数下のオブジェクトへの参照に限るが、持ち手の側には制限を置かないので `o'` の状態を
  問わない)、A5 よりその leaf は `o` への参照をちょうど 1 つ持つ。D25 が挙げるのは未処分の参照の持ち手
  なので、その参照は未処分であり、D8 より `H(o) ≥ 1` である。解放が閉じているので `o` は解放されて
  いない。
  BY A5, A8, D8, D25, D26, DEF 解放が閉じている点

<1>2. QED
  到達の道の長さについての帰納。長さ 0 のとき `o'' = o'` であり仮定そのものである。長さ `n+1` の道は、
  `o'` が持つ参照が指すオブジェクト `o1` への 1 歩と、`o1` から `o''` への長さ `n` の道に分かれる。
  `<1>1` より `o1` は解放されておらず、帰納法の仮定より `o''` も解放されていない。
  BY D25, <1>1

### L2b (段の中でも、解放されたオブジェクトへ参照は作られない)

**言明**。時点 `t` が勘定が合っているとし、`t` の直後の段を `S` とする。`S` を持つ活性化の本体と、`S` の
中で (F) の解放が作る活性化の本体は D11 を満たすとする。このとき次の 2 つが成り立つ。

- **(a)** `S` のどの段内の点も解放が閉じている。
- **(b)** `S` が作る各参照について、その参照が指すオブジェクトは、その参照が作られる点で解放されていない。
  したがって `t` においても解放されていない -- D24 の 6 種の素動作に解放を取り消すものは無いので、`t` で
  解放されていたオブジェクトは、その後のどの段内の点でも解放されている。

**(b) が「その参照が作られる点で」と言うのは、`S` の中で (F) の解放が作る活性化の節点も参照を作るから
である。**その節点は `t` の直後ではなく段の途中で走るので、`t` を主語にした言明では D11 の (S-c) が
与えるものと噛み合わない。

<1>0. `t` は解放が閉じている。
  BY DEF 勘定が合っている時点, DEF 解放が閉じている点

<1>1. `S` が新しい参照を作るのは、D24 の (E2) の `H` の表が挙げる生成の 7 行と、参照を処分する段の中で
      起きる (F) の解放が `Destructor` のオブジェクトについて行う retain と、その解放が作る活性化の節点が
      同じ 7 行によって作るものだけである。
  D24 の (E2) の `H` の表が 7 行を挙げる -- `Retain` の行、`Llvm` の行、
  `InlineLLVMBoxedFromRetainedPtrIOS` の行、boxed 容器の `Destructure` の名前付きフィールドの行、
  boxed union の変位アームの payload の行、`Closure` の行、`App` の行である。
  (E1)、(E3)、(E4) は持ち手を移すだけ、(E5) は印を付けるだけ、(E7) は `Obl` が空の活性化を作るだけで
  あり、(E6) の後に段は無い。段の中で起きる (F) の解放については、D24 が「**この段は参照も作る。**
  `_dtor` の欄の関数に適用の分の参照を与える retain がそれである」「よって **新しい参照を作るのは (E2)
  だけではない**」と述べる。同じ (F) が `Destructor` について作る活性化の節点も、その本体についての
  (E2) の生成の 7 行によって参照を作る。
  BY D24

<1>1a. 参照を作る動作を行う節点が D7 の読む構文であるとき、その構文が読みうるオブジェクト -- 名指された
       値の inhabited な各 boxed leaf が指すオブジェクト -- は、その動作の点で解放されていない。その点が
       解放が閉じているならば、そこから到達できる (D25) オブジェクトも解放されていない。
  その節点はある活性化 `a'` の本体 `B(a')` の節点である。`<1>1` より `a'` は `S` を持つ活性化か、`S` の
  中で (F) の解放が作る活性化であり、仮定よりどちらの本体も D11 を満たす。`a'` が辿った節点の列は
  `B(a')` の実行路 (D3) である (D21, D23)。(S-c) が、読みうる各オブジェクトについて解放されていないことを
  言う。その条件は節点の位置について述べられており、DEF 段の素動作と段内の点よりその節点の段のどの段内の
  点についても読む。到達できるオブジェクトへは L2 が広げる。グローバル状態のオブジェクトは A8 より
  解放されない。
  BY A8, D3, D7, D11 (S-c), D21, D23, D25, L2, DEF 段の素動作と段内の点, <1>1

<1>1b. 参照を作る動作を行う節点が `Retain(v, π, s, k)` であるとき、`π` の下の inhabited な各 leaf が指す
       オブジェクトは、その動作の点で解放されていない。
  D7 より `Retain(v, π)` が**触れる**のはちょうどそれらのオブジェクトである。`<1>1` よりその節点は `S` を
  持つ活性化か (F) の解放が作る活性化の本体の節点であり、仮定よりその本体は D11 を満たす。その活性化が
  辿った節点の列はその本体の実行路 (D3) である (D21, D23)。D11 の (S-c) は、読む構文が読みうる
  オブジェクトに加えて `Retain(v, π)` と `Release(v, π)` が触れる各オブジェクトについても、解放されて
  いないことを言う。その条件は節点の位置について述べられており、DEF 段の素動作と段内の点よりその節点の
  段のどの段内の点についても読む。
  BY D3, D7, D11 (S-c), D21, D23, DEF 段の素動作と段内の点, <1>1

<1>1c. 割り当ての素動作が作るオブジェクトは、その点で解放されていない。
  D25 より、オブジェクトが解放されるのはそれを割り当てた後である。割り当ての動作の点では、そのオブジェクト
  についてまだ解放は起きていない。
  BY D24 (F), D25

<1>2. CASE `Retain(v, π, s, k)` の行。
  D10 の `Retain` の行より、この動作が作る参照が指すのは `π` の下の inhabited な各 leaf `λ` の
  `obj(v, λ)` である。`<1>1b` がそれらについて言明を与える。
  BY D10, D24 (E2), <1>1b

<1>3. CASE `Closure(f, caps)` の結果の行。
  この動作が作るのは、この段が新しく割り当てる capture object である (D24 (E2) の `H` の表)。`<1>1c` が
  それについて言明を与える。`caps` が空のときは capture ポインタが null でありオブジェクトも参照も無い
  ので、言明は空虚に成り立つ。
  BY D24 (E2), <1>1c

<1>4. CASE `App(callee, args)` の結果の行。
  この行は参照を作らない (D24 (E2) の `H` の表の `App` の行)。よって言明は空虚に成り立つ。
  BY D24 (E2)

<1>5. CASE `Llvm` の行、および `InlineLLVMBoxedFromRetainedPtrIOS` の行。
  <2>0. op が `applies_a_function_operand` を宣言し、その適用が作った活性化 `b` が返した参照が結果の
        leaf に置かれるとき、この段はその leaf について新しい参照を作らない。
    L0a (b) より、その参照は `b` の中で作られて (E4) でこの段を実行している活性化の `Obl` に入るもの
    であり、この段が新しく作るのではない。D24 の (E4) も「このとき (E2) の生成の表の `Llvm` の行は
    その leaf について読まない」と述べる。
    BY D24 (E4), L0a
  <2>1. CASE 宣言が単一の `Fresh`。
    <3>1. その leaf に置かれる参照が指すオブジェクトは、この動作が新しく割り当てたオブジェクトか、この op の
          オペランドが指すオブジェクトである。
      A3 の `Fresh` の行は「新しく割り当てたオブジェクトへの新しい参照」を述べ、続けて
      「**`unique_check_operand` を宣言する op の `Fresh` の行は、オブジェクトの同一性については字義
      どおりではない。** そうした op は実行時に参照カウントで分岐し、一意の腕ではオペランドのオブジェクトを
      そのまま返す」と例外を述べる。宣言が言うのは参照が新しいことだけであり、その参照が指すオブジェクトは
      この 2 つのどちらかである。
      BY A3
    <3>2. QED
      前者は `<1>1c` が扱う。後者について、D24 の (E2) の行き先の段落は「その腕は `create_obj` を呼ばず
      オペランドのオブジェクトをそのまま返すので、消費されたオペランドの参照はそのまま結果の leaf の参照に
      なる -- 処分でも新しい割り当てでもない」と述べる。すなわちその leaf については参照は**移る**の
      であって作られないので、この場合は言明の量化の外にある。さらに `Let(x, Llvm(gen, args), k)` は
      D7 の読む構文であり読まれる値は各オペランドなので、そのオブジェクトについては `<1>1a` も言明を
      与える。
      BY A3, D7, D24 (E2), <1>1a, <1>1c, <3>1
  <2>2. CASE 宣言が単一の `Unknown`。
    <3>0. CASE op が `InlineLLVMBoxedFromRetainedPtrIOS` である。
      A3 の `Unknown` の行は、この op について「オペランドは `Std::Ptr` で boxed leaf を持たないので、
      到達できる元が無い -- そのオブジェクトは C の側から渡された番地が指すものである」と述べ、その leaf に
      ついて行を読まないよう指示する。生成コードは第 1 オペランド `ptr` の第 0 欄の番地を結果の第 1 欄へ
      入れるだけである (`CODE src/fixstd/builtin.rs: InlineLLVMBoxedFromRetainedPtrIOS`)。この op を包む
      公開関数 `Std::FFI::boxed_from_retained_ptr` の doc は「Creates a boxed value from a retained
      pointer obtained by `boxed_to_retained_ptr`」「It is the user's responsibility to ensure that the
      argument is actually a pointer to the type of the return value, and undefined behavior will occur
      if it is not」と述べる。すなわち番地は `boxed_to_retained_ptr` が渡したものであり、**A17 (i-b) が
      「`boxed_to_retained_ptr` が渡した番地について、環境はその参照を持ち、`boxed_from_retained_ptr` で
      Fix の側へ返すまで処分しない」と言う。**その番地が指すオブジェクトを `o` とすると、環境が `o` への
      未処分の参照を 1 つ持つので (D25 の 3 つ目の持ち手)、D8 より `H(o) ≥ 1` であり、この点は解放が
      閉じているので `o` は解放されていない。その形でない番地を渡す実行は、doc が言うとおりこの文書の
      モデルの外にある。**この行は D24 の `H` の表の `InlineLLVMBoxedFromRetainedPtrIOS` の行でもある** --
      そこでは `H` は変わらず、環境が持っていた参照が `E` から `Obl(a)` へ渡るので、この段は新しい参照を
      作らない。
      BY A3, A17, D8, D22, D24 (E2 の `H` の表), D25, DEF 解放が閉じている点,
         CODE src/fixstd/builtin.rs: InlineLLVMBoxedFromRetainedPtrIOS,
         CODE src/fixstd/std.fix: boxed_from_retained_ptr, boxed_to_retained_ptr
    <3>0a. CASE op が `applies_a_function_operand` を宣言する。
      A3 の `Unknown` の行は「**この限定が成り立たない op が 2 種ある。** オペランドを適用する op
      (`LLVMGen::applies_a_function_operand`) では、適用した関数の中で新しく割り当てられたオブジェクトが
      結果に出る」と述べる。すなわちこの場合の限定は、オペランドから到達できるオブジェクトとグローバル値が
      到達するオブジェクトに、**適用した関数が返した値から到達できるオブジェクト**を足したものである。
      `Let(x, Llvm(gen, args), k)` は D7 の読む構文であり、読まれる値は各オペランドなので、オペランドから
      到達できるオブジェクトは `<1>1a` が扱い、グローバル値が到達するオブジェクトは A8 より解放されない。
      適用が作った活性化 `b` が返した値については、L1 (b) より `b` の終端の `Ret` が消費した参照が
      この段を実行している活性化の `Obl` に入っているので、その leaf が指す各オブジェクト `o` について
      D8 より `H(o) ≥ 1` であり、この点は解放が閉じているので `o` は解放されていない。そこから到達できる
      オブジェクトへは L2 が広げる。`b` が返した参照をそのまま持つ leaf は `<2>0` が除く。
      BY A3, A8, D7, D8, D25, L1, L2, DEF 解放が閉じている点, <1>1a, <2>0
    <3>1. CASE op が `InlineLLVMBoxedFromRetainedPtrIOS` でも `applies_a_function_operand` を宣言する
          op でもない。
      A3 の `Unknown` の行の限定が当たる -- 参照が作られるオブジェクトは、この op のオペランドの
      inhabited な boxed leaf が指すオブジェクトから到達できるか、グローバル値が到達するオブジェクトで
      ある。`Let(x, Llvm(gen, args), k)` は D7 の読む構文であり、読まれる値は各オペランドである。
      `<1>1a` がオペランドの leaf の指すオブジェクトとそこから到達できるオブジェクトについて言明を与え、
      グローバル値が到達するオブジェクトは A8 より解放されない。
      BY A3, A8, D7, D25, <1>1a, CODE src/rc_ir/provenance.rs: LeafOrigin
    <3>2. QED
      A3 の `Unknown` の行が限定の成り立たない op として挙げるのは 2 種 --
      `InlineLLVMBoxedFromRetainedPtrIOS` とオペランドを適用する op -- であり、`<3>0`、`<3>0a`、`<3>1` が
      その 2 種とそれ以外を尽くす。
      BY A3, <3>0, <3>0a, <3>1
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
    **でない**もの」)、参照を作らない。残る 4 つを `<2>1`-`<2>4` が扱い、そのうち適用が返した参照を
    受け取る leaf は `<2>0` が除く。
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
      ないアームについては `arm.tag` を `expect` で取り出す (`CODE src/rc_ir/codegen.rs:
      Generator::eval_rc_match` -- `else_bb` は `arm_bbs.last()`)。最後でないアームの `tag` が `None` で
      あればそこで panic する -- そのときプログラムは走らず、この場合の段は存在しない。よって段が在る
      実行では catch-all アームは最後のアームであり、default に落ちる選択はその catch-all アームである。
      A16 より `arms` は catch-all アームを持つか実行時のタグに等しい `tag` を持つアームを持つので、
      変位アームが選ばれたのは、その `tag` が実行時のタグに等しいときである。
      BY A16, D21, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
    <3>2. QED
      D10 の生成の表より、この段が参照を作るのは payload の各 boxed leaf についてである。`<3>1` より
      その payload は変位 `t` のものであり、`t` は実行時のタグなので、その leaf は scrutinee の値の
      inhabited (D16) な boxed leaf である。D25 より、それが指すオブジェクトは scrutinee のオブジェクトが
      持つ参照の指す先である。
      BY D10, D16, D25, <3>1
  <2>3. QED
    `Destructure(c, fs, s, k)` と `Let(x, Match(v, arms), k)` はどちらも D7 の読む構文であり、読まれる値は
    それぞれ容器 `c` と scrutinee `v` である。`<1>1a` が、それらの指すオブジェクトとそこから到達できる
    オブジェクトについて言明を与える。
    BY D7, <1>1a, <2>1, <2>2

<1>7. CASE (F) の解放が `Destructor` のオブジェクト `o` について行う `_dtor` への retain。
  D24 の (F) より、この retain の対象は `o` が `_dtor` の欄に持つ関数である
  (`CODE src/generator.rs: Generator::build_traverser_work_nonnull_boxed_with` --
  `obj.is_destructor_object()` の枝が `build_run_destructor` を `traverse_refs` の前に置く。
  `CODE src/generator.rs: Generator::build_run_destructor` -- `build_retain(dtor, one, ..)` が
  `apply_lambda` の前に立つ)。D24 の (F) より、この解放が `o` の持つ参照を処分するのは `_dtor` の適用と
  `IO` の動作の往復の**後**なので、この retain の点で `o` はまだその参照を持っている。`o` は割り当てられて
  いて記憶域を返す前であり、D24 が「解放の中では `o` はまだ解放されておらず、走査は `o` を読む」と述べる
  ので、`o` は D25 の 2 つ目の持ち手である。A5 よりその leaf は参照を 1 つ持ち、D8 より対象のオブジェクトの
  `H` は 1 以上である。この点は解放が閉じているので、対象は解放されていない。
  BY A5, D8, D24 (F), D25, DEF 解放が閉じている点,
     CODE src/generator.rs: Generator::build_run_destructor,
     Generator::build_traverser_work_nonnull_boxed_with

<1>8. 解放が閉じている段内の点で `S` が参照を作るとき、その参照が指すオブジェクトはその点で解放されて
      いない。
  `<1>1` が参照を作る動作を数え上げる。(E2) の生成の 7 行を `<1>2`-`<1>6` が尽くす -- `<1>5` が `Llvm` の
  行と `InlineLLVMBoxedFromRetainedPtrIOS` の行の 2 つを扱う。(F) の retain を `<1>7` が扱う。(F) が作る
  活性化の節点が作る参照も、その本体についての (E2) の生成の 7 行によるので (`<1>1`)、同じ `<1>2`-`<1>6`
  が扱う -- `<1>1a` と `<1>1b` はどちらも「参照を作る動作を行う節点」を主語にしており、`<1>1` が挙げる
  2 種の活性化のどちらの節点についても、その本体が D11 を満たすことから (S-c) を引く。
  BY <1>1, <1>1a, <1>1b, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7

<1>9. 解放が閉じている段内の点の直後の素動作の後の点も、解放が閉じている。
  6 種の素動作を順に見る (DEF 段の素動作と段内の点)。**受け渡し**は `H` を変えず、解放も割り当ても
  起こさない。**処分**は `H` を 1 下げるので、`H(o) ≥ 1` である計数下のオブジェクトの集合は増えず、
  それ自身は解放を起こさない。**割り当て**は新しいオブジェクトを作り、`<1>1c` よりそれは解放されて
  いない。**解放**は、D24 の (F) より `H(o) = 0` になった `o` について起きるので、新たに解放される `o` の
  `H(o)` は 0 である。**グローバル化**は印を付けたオブジェクトを計数下の集合から外すので (D26)、条件が
  量化する集合が減るだけである。**生成**は `H` を 1 上げるが、`<1>8` よりその先はこの点で解放されて
  いないので、解放されているオブジェクトの `H` は動かない。
  BY D7, D24, D24 (F), D26, DEF 解放が閉じている点, DEF 段の素動作と段内の点, <1>1c, <1>8

<1>10. QED
  段内の点の列は有限である (DEF 段の素動作と段内の点)。`<1>0` が最初の点について (a) を与え、`<1>9` が
  1 動作ずつそれを運ぶので、帰納により `S` のどの段内の点も解放が閉じている。そのうえで `<1>8` が (b) の
  前半を与える。D24 の 6 種の素動作に解放を取り消すものは無いので、`t` で解放されていたオブジェクトは
  その後のどの点でも解放されており、その対偶が (b) の後半である。
  BY D24, DEF 段の素動作と段内の点, <1>0, <1>8, <1>9

### L3 (実行のすべての時点は勘定が合っている)

**言明**。プログラム `P` のすべての本体が D11 を満たすとする。`P` の実行 `ρ` のすべての時点は、勘定が
合っている。

<1>1. `ρ` の最初の時点は勘定が合っている。
  <2>1. 最初の時点で解放されているオブジェクトは無い。
    オブジェクトが解放されるのは (F) の解放においてであり、(F) は段の中で起きる。最初の時点までに段は
    1 つも実行されていない。
    BY D24 (F), D25
  <2>2. 最初の時点の各計数下オブジェクト `o` について `H(o) ≥ 1` である。
    D24 は「実行の最初の時点に在る参照は、環境が持ってきたものだけである」と述べ、`FFI_EXPORT` の
    エントリ点が boxed な引数を取る実行ではその参照が在ると述べる。A17 (i-c) は「実行の最初の時点に
    在る参照は、環境が持つか、環境が持ち込んだオブジェクトの leaf が持つ」と述べ、各計数下オブジェクト
    `o` の `H(o)` を、環境が持つ `o` への参照の個数と、生きているオブジェクトの leaf が持つ `o` への
    参照の個数の和に等しいとする。この時点に在る計数下オブジェクトは環境が持ち込んだものであり、
    (i-c) の第 1 文がその持ち込みを参照の側から数える -- 環境が参照を持つオブジェクトについては
    `E` の参照が 1 つ、そこから到達できる (D25) オブジェクトについては自分を指す leaf を持つ生きている
    オブジェクトの分が 1 つ、それぞれ数えられる (A5)。
    BY A5, A17, D24, D25
  <2>3. QED
    `<2>1` より「解放されている」計数下オブジェクトは無く、`<2>2` より「`H(o) = 0`」の計数下オブジェクトも
    無いので、2 つは一致する。
    BY DEF 勘定が合っている時点, <2>1, <2>2

<1>2. 勘定が合っている時点 `t` の直後の 1 段 `S` の後の時点も、勘定が合っている。
  <2>1. `S` を持つ活性化の本体と、`S` の中で (F) の解放が作る活性化の本体は D11 を満たす。
    D23 より活性化の本体は `P` のある関数の `body` かあるグローバル初期化子の `init` であり、この言明の
    仮定は `P` のすべての本体について D11 を述べる。
    BY D23
  <2>2. `S` のどの段内の点も解放が閉じている。とくに `S` の後の時点で、解放されている計数下の
        オブジェクトの `H` は 0 である。
    L2b (a) を `t` と `S` に当てる。`t` は勘定が合っており、`<2>1` が本体についての仮定を満たす。
    BY L2b, <2>1
  <2>3. `S` の後の時点で `H(o) = 0` である計数下のオブジェクト `o` は解放されている。
    `t` において `H(o) = 0` であれば、`t` は勘定が合っているので `o` はそこで解放されており、D24 の
    6 種の素動作に解放を取り消すものは無いので `S` の後でも解放されている。`t` において `H(o) ≥ 1` で
    あれば、`S` の中で `H(o)` が 0 に**なった**ということであり、`H` を下げる素動作は参照の処分だけなので
    (DEF 段の素動作と段内の点)、D24 の (F) が「ある段が参照を処分して計数下のオブジェクト `o` の `H(o)`
    が 0 になったとき、`o` はその同じ段の中で解放される」「段の終わりには `H = 0` の計数下のオブジェクトは
    すべて解放されている」と述べるとおり、`o` は `S` の中で解放されている。
    BY D24, D24 (F), DEF 勘定が合っている時点, DEF 段の素動作と段内の点
  <2>4. QED
    `<2>2` が「解放されている ⟹ `H = 0`」を、`<2>3` が「`H = 0` ⟹ 解放されている」を、`S` の後の時点に
    ついて与える。
    BY DEF 勘定が合っている時点, <2>2, <2>3

<1>3. QED
  時点までの段の数についての帰納。
  BY <1>1, <1>2

**注 (段が参照を受け渡すこと)**。L3 は `Obl` の推移を読まない -- 勘定が合っていることは `H` と解放だけの
条件だからである。段が参照をどこへ渡すかは L0a と L1 が扱い、L2b の `Llvm` の場合がそれを読む。

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

<1>2. (b) が成り立つ。
  活性化 `a` が終わるのは `B(a)` の終端の `Ret` に着いてその消費を行うときである (D23)。`a` が辿った
  節点の列は `B(a)` の実行路 (D3) であり (D21, D23)、仮定より `B(a)` は D11 を満たすので、(S-b) が
  「終端の `Ret` の消費を行った後の `Obl` は空である」を与える。
  BY D3, D11 (S-b), D21, D23

<1>3. (c) が成り立つ。
  `<1>1` より最後の時点で生きている活性化は無いので、D25 が挙げる 3 つの持ち手のうち活性化は残らない。
  `<1>2` より、終わった各活性化はその終わりの時点で `Obl` を空にしているので、終わった活性化の側に
  参照が残ることも無い。P28 より、処分されていない各参照はちょうど 1 つの持ち手を持つ。
  BY D25, P28, <1>1, <1>2

<1>4. (d) が成り立つ。
  <2>1. 林の根は有限個である。
    根は (E1) の段が作る活性化である (D24 の活性化の林)。D31 より正常終了する実行の段の列は有限なので、
    (E1) の段も有限個である。
    BY D24 (活性化の林), D31
  <2>1a. `ρ` の段の列は有限である。
    BY D31
  <2>2. 1 つの段が作る活性化は有限個である。
    (E1)、(E3)、(E7) はそれぞれ 1 つ作る。(E5) と (E6) は活性化を作らない。残るのは 2 つである。
    **オペランドを適用する `Llvm` の段 (L0a) が作る活性化**は、自分の段を持つ -- D23 が自分の段を持たないと
    するのは (F) の解放が作る活性化だけであり、`<1>1` よりこの活性化は終わるので、その終端の `Ret` を
    実行する (E4) の段を少なくとも 1 つ持つ。相異なる活性化の (E4) の段は相異なるので、`<2>1a` より
    その個数は有限である。
    **(F) の解放が作る活性化**は、解放されるオブジェクト 1 つにつき 2 つである (D24 の活性化の林)。
    D24 の (F) より解放はその段の処分で `H` が 0 になったオブジェクトについて起きるので、1 つの段の中の
    解放の個数はその段の処分の素動作の個数で抑えられ、段の素動作の列は有限である (DEF 段の素動作と
    段内の点)。
    BY D23, D24 (F), D24 (活性化の林), L0a, DEF 段の素動作と段内の点, <1>1, <2>1a
  <2>4. 林は有限である。
    D24 の活性化の林より、活性化を作る段は 5 種で尽きる。よってどの活性化にも、それを作った段が 1 つずつ
    対応する ((F) の解放が作る活性化については、その解放を含む段がそれである)。`<2>1a` より段は有限個で
    あり、`<2>1` より根を作る (E1) の段も有限個であり、`<2>2` より各段が作る活性化は有限個なので、
    `ρ` の活性化は有限個である。
    BY D24 (活性化の林), <2>1, <2>1a, <2>2
  <2>5. 各活性化が割り当てるオブジェクトは有限個である。
    活性化が訪れる節点の列はその本体の実行路 (D3) であり (D21)、D2 より本体は有限の木なので実行路は
    有限である。1 つの節点の実行が割り当てるのは、D24 の (E2) の `H` の表で割り当てを行う 2 行 --
    `Closure` の行 (1 つ) と `Llvm` の行の単一の `Fresh` の場合 (結果の型の boxed leaf ごとに 1 つ) --
    に限り、A10 より `boxed_leaf_paths` は有限の列である。
    BY A10, D2, D3, D21, D24 (E2)
  <2>6. QED
    オブジェクトを割り当てるのは、D24 の (E2) の `H` の表で割り当てを行う 2 行だけである -- (E1)、(E3)、
    (E4)、(E5)、(E7) はどれも割り当てを持たず、(E6) の後に段は無く、(F) の解放は参照の処分と記憶域の
    返却と `Destructor` の活性化からなり、その活性化の節点の実行もまた (E2) の `H` の表に従う。よって
    どの割り当てもある活性化の節点の実行に属する。`<2>4` より活性化は有限個であり、`<2>5` より各活性化が
    割り当てるオブジェクトは有限個なので、`ρ` に現れるオブジェクトは有限個である。
    BY D24, <2>4, <2>5

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

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
    各オブジェクトは、その時点で解放されていない」を言い、DEF 段の素動作と段内の点よりその条件をその
    節点の段のどの段内の点についても読む -- 読みは段の入口ではなく段の中で起きる。D32 の (読み-1) が
    読むのはちょうどそのオブジェクトである。
    BY D3, D7, D11 (S-c), D12, D21, D23, D32, DEF 段の素動作と段内の点
  <2>2. CASE (読み-2) (F) の解放の走査。
    D32 の (読み-2) より、この走査が読むのは解放されるオブジェクト `o` そのものである。D24 の (F) より、
    走査が走る間 `o` はまだ解放されていない。
    BY D24 (F), D32
  <2>3. CASE (読み-2) (E5) の `mark_global` の走査。
    <3>1. 走査の起点は、グローバル初期化子の活性化 `b` が終端の `Ret` で返す値の inhabited な各 boxed
          leaf が指すオブジェクトであり、その時点でその参照は `Obl(b)` に在る。
      D24 の (E5) は「グローバル初期化子の活性化が終端の `Ret` に着き、**返す前に**、環境が `mark_global`
      でその値が到達するオブジェクトのグラフ全体に印を付ける」と述べ、(E7) は「`b` が終端の `Ret` に着き、
      返す前に (E5) の段が走る。続いて、(E4) と同じく `b` の終端の `Ret` が消費する参照が `Obl(b)` を
      離れる」と述べる。すなわち (E5) は終端の `Ret` の消費より前であり、`b` はその時点でまだ終わって
      いない (D23)。コードも `implement_rc_global` が `eval_rc_expr`、`mark_global`、`build_return` の
      順に出す。
      BY D23, D24 (E5), D24 (E7), CODE src/rc_ir/codegen.rs: Generator::implement_rc_global
    <3>2. 起点のオブジェクトは解放されていない。
      `<3>1` の参照は生きている活性化 `b` の `Obl(b)` に在るので、起点のオブジェクトを `o` とすると
      `H` の分解より `H(o) ≥ 1` であり、L3 より `o` は解放されていない。グローバル状態のオブジェクト
      なら A8 より解放されない。
      BY A8, D8, D25, L3, P28, <3>1
    <3>3. QED
      D32 の (読み-2) よりこの走査が読むのは起点と、そこから到達できるオブジェクトである。L3 よりその
      時点は勘定が合っており、したがって解放が閉じているので、L2 がそこから到達できるオブジェクトへ
      `<3>2` を広げる。
      BY D32, L2, L3, DEF 解放が閉じている点, <3>2
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
      D32 の (読み-2) よりこの走査が読むのは起点と、そこから到達できるオブジェクトである。L3 よりその
      時点は勘定が合っており、したがって解放が閉じているので、L2 がそこから到達できるオブジェクトへ
      `<3>2` を広げる。
      BY D32, L2, L3, DEF 解放が閉じている点, <3>2
  <2>5. CASE (読み-3) 環境の読み。
    A17 (ii) より、環境が読むのはその時点で `E` が持つ参照が指すオブジェクトか、そこから到達できる
    オブジェクトである。前者を `o` とすると、`H` の分解より `H(o) ≥ 1` であり、L3 よりその時点は勘定が
    合っているので `o` は解放されていない (グローバル状態なら A8 より解放されない)。後者へは、勘定が
    合っている時点は解放が閉じているので L2 が広げる。
    BY A8, A17, D8, D25, L2, L3, P28, DEF 勘定が合っている時点, DEF 解放が閉じている点
  <2>6. QED
    D32 は `ρ` の読みを (読み-1)、(読み-2)、(読み-3) の 3 つに尽くし、(読み-2) は 3 つの走査に尽きる。
    BY D32, <2>1, <2>2, <2>3, <2>4, <2>5

<1>2. (R2) が成り立つ。
  <2>1. グローバル状態のオブジェクトは 1 度も解放されない。
    BY A8, D26
  <2>2. 計数下のオブジェクト `o` が解放されるのは、`H(o)` が 0 になったときだけであり、それは `H(o)` を 0 に
        した段の中で起きる。
    BY D24 (F)
  <2>3. `o` が解放された後、`o` への参照は 1 つも作られない。
    D12 よりどの本体も D11 を満たすので、L3 よりすべての時点は勘定が合っている。`o` を解放した段を `S_0`
    とする。**`S_0` の中で `o` が解放された後の段内の点**については、L2b (b) を `S_0` に当てる -- その段が
    作る参照が指すオブジェクトはその参照が作られる点で解放されておらず、`o` はその点で解放されているので
    `o` ではない。**`S_0` より後の段**については、その段の直前の時点で `o` は解放されており (D24 の 6 種の
    素動作に解放を取り消すものは無い)、L2b (b) をその段に当てると、作られる参照が指すオブジェクトはその点で
    解放されていないので `o` ではない。
    BY D12, D24, L2b, L3, DEF 段の素動作と段内の点, <2>2
  <2>4. `o` が解放された後、`H(o)` は 0 のままである。
    D8 より `H(o)` は `o` への処分されていない参照の総数である。解放の点でそれは 0 であり (`<2>2` と
    D24 の (F))、`<2>3` より新しい参照は作られず、無い参照は処分できないので減りもしない。
    BY D8, D24 (F), <2>2, <2>3
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
| L0d | A22 |
| L0c | A13 |
| P29 | A6、A11、A13、A22 (L0b の (N1)-(N3) を果たす)、および L0c と L0d |
| L0 | A6、A10、A11、A13、A21、A22、A24 と、P1・P9・P12・P24 の言明、および L0b と L0c と L0d |
| L1 (a) | 仮説 (ABI)、および A12、A14 |
| (R1) | D11 の (S-c)、A3 と A4 (D24 の生成の表と L2b を通じて)、A5 (D8 と D25 を参照の個数へ繋ぐ)、A8、A16 と A17 (i-b) (L2b を通じて)、A17 ((i-c)、(ii))、および P28 の言明 |
| (R2) | (R1) が使うもののすべて |
| (R3) | (R2) が使うもののすべてと **A18 (a)(b)**、D11 の (S-b) (L5 (b) を通じて)、および A10 と D2 (オブジェクトが有限個であること。L5 (d) を通じて) |
| 系 | (R3) が使うもののすべて |

**(R2) は仮定を足さない。**D11 の (S-c) が `Retain(v, π)` と `Release(v, π)` の触れる先を扱うので、
L2b の `<1>1b` がそれを (S-c) から直接に出す。README の第 6 節「(S-c) を強めた記録」が、その節が弾く
本体を述べる。

L0、L0c、L0d、L1 は P27 の証明のどのステップからも引かれない。L0 が支えているのは D23 の読み -- D9 と
D10 の「呼び出し先」を実行時の関数と読むこと -- が、`cancel` が静的に計算するものと食い違わない、という
ことである。食い違えば、D11 を保存すると称する P14 と P23 が P27 の使う述語とは別の述語を保存している
ことになる。P29 は同じことを `borrow_ify` の入力について述べ、層 1 の P7 の `App` の場合がそれを読む。
L0c は P29 と L0 の両方が使う名前の性質を、L0d は両方が使うコード生成の性質を、それぞれ括り出したもので
ある。L1 (a) は D24 の (E3) と D10 の初期値が同じ多重集合を指すことの検算であり、L1 (b) は L2b の
`Llvm` の場合が読む。

L0a は P27 の証明から L2b と L5 を通じて引かれる。支えているのは、活性化を作る段が (E1)、(E3)、(E7) の
3 つに尽きないこと -- オペランドを適用する `Llvm` の段も作ること -- と、その段が作った活性化が終わるときの
参照の行き先である。

## 6. 残っているもの

**(1) 仮説 (ABI) を果たす節が枠に無い。** L1 (a) は、実行するプログラムの各関数について
`params` の型の列が `fn_ty` の lambda src の列であること、`capture` が `Some` であることと
`fn_ty.is_closure()` が同値であること、コード生成がその関数に与える LLVM 関数の署名が
`lambda_function_type(fn_ty)` であること、そして `App` の `ty(callee)` がその段の実行時の呼び出し先の
`fn_ty` に等しいことを、仮説として置いている。A12 が並べるのは `RcVar` と節点の型の一致であって、
`RcFunc` の欄どうしの整合も `ty(callee)` と呼び出し先の `fn_ty` の一致も含まない。

**署名の一致から仮説を出すことはできない。**LLVM の署名はパラメータの型の平らな列であり、末尾の
ポインタが CAP かどうかを署名から読み取る手立ては無い。`Generator::type_parts` の doc が「a zero-sized
type is none」と述べるので、クロージャ型 `IOState -> I64` の署名 (引数の parts は 0 個、CAP の `ptr` が
1 つ) と funptr 型 `Ptr -> I64` の署名 (引数の parts は `ptr` 1 つ、CAP 無し) はどちらも `(ptr) -> i64`
であり、`is_closure` は逆である。

足す節の形は、`RcFunc` の欄が互いに整合していること (`RcFunc` の `fn_ty`・`params`・`capture` の doc が
述べる内容) と、`App(callee, args)` の `ty(callee)` が呼び出し先の `fn_ty` に等しいことを、A12 の節として
置くことである。果たす者は `Lowerer::lower_lambda_as_function` (`fn_ty` を `lam_ty` に取り、`params` を
その lambda src から作り、`capture` を `lam_ty.is_closure()` の枝でだけ作る) と、その 3 つを写すだけの
後段のパスである。

**(2) (S-c) の量化が D24 の段内の点へ届いていない。** D11 の (S-c) は節点の位置について
「その時点で解放されていない」を言い、D24 は 1 つの節点の段の素動作の列の中で読みと処分がどう並ぶかを
定めない。この文書は (S-c) をその節点の段のどの段内の点についても読む (DEF 段の素動作と段内の点)。
(R1) の (読み-1) の場合も同じ読みに立つ -- 読みは段の入口ではなく段の中で起きるからである。足す文の形は、
D11 の (S-c) の「その時点」を「その節点の段のどの点でも」と書くことである。

**(3) `proof_links.py` の走査が `llvmgen-function-values.md` を読まない。**`citations_of` は README の
`CODE` を `FRAME` として集めるが、`p*.md` の走査は `glob("p*.md")` なので
`llvmgen-function-values.md` に当たらない (`dev-docs/proof/proof_links.py` の `citations_of`)。A21 の
果たす者の数え上げがそこに在り、この文書の L0 の `<1>5` の `<2>4b` がそれを引くので、そのファイルが引く
コードにもリンクを張るなら、走査の対象にそのファイルを足すことになる。
