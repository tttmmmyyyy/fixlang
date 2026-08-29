# `LLVMGen` の 78 実装のうち、関数の値を作るもの

`p51-runs.md` の `L0` は「実行時に関数の値が生じるのは `Closure(fref, caps)` の評価か `prog.funcs` の鍵で
ある名前の読みだけである」を仮定として置いていた。この文書はその数え上げを行い、仮定を命題に変える材料を
与える。**`src/fixstd/builtin.rs` の `impl LLVMGen for` 78 個すべてと、それらが呼ぶヘルパを読んだ。**

読んだコードはコミット `fa6c9ca3` の版である。

## 分類

各実装が、Fix の関数型 (`Arrow a b` すなわちクロージャ、または `Std::#FunPtr{n}`) の値を実行時に作りうるか
で分ける。

| 類 | 意味 | 件数 |
|---|---|---|
| (A) | 関数の値を作らない | 70 |
| (B) | 受け取った関数の値を写すだけ (load / extractvalue / bit_cast / `apply_lambda` の戻り値 / オペランドそのもの) | 7 |
| (C) | 自ら関数ポインタを作って Fix の関数型の値へ書き込む | **1** |

(B) の 7 件: `InlineLLVMArrayUnsafeGetBoundsUnchecked`、`InlineLLVMStructGetBody`、
`InlineLLVMCaptureProjectBody`、`InlineLLVMUnionAsBody`、`InlineLLVMWithRetainedFunctionBody`、
`InlineLLVMArrayBorrowElementsBody`、`InlineLLVMMarkThreadedFunctionBody`。

結果の**成分**が関数型になりうる場合も (B) と読むと (A) 51 / (B) 26 / (C) 1 になる。(C) はどちらの読みでも
1 件である。

## (C) はただ 1 件 -- `InlineLLVMFixBody`

`Std::fix` の本体は、生成中の関数自身を指すクロージャを組み立てる
(`CODE src/fixstd/builtin.rs: InlineLLVMFixBody::generate_tail`)。

```rust
let fixf_ty = f.ty.get_lambda_dst();                 // f : (a -> b) -> (a -> b) なので a -> b
let fixf = create_obj(fixf_ty.clone(), ..);
let fixf_funptr = gc.current_function().as_global_value().as_pointer_value();
let fixf = fixf.insert_field(gc, CLOSURE_FUNPTR_IDX, fixf_funptr);
let cap_obj = gc.get_scoped_obj(&self.cap_name);     // "#CAP"
let fixf = fixf.insert_field(gc, CLOSURE_CAPTURE_IDX, cap_obj.value(gc));
```

**指す先は `RcProgram::funcs` のエントリである。** `gc.current_function()` は builder が今いる関数であり、
この op は必ず `implement_rc_function` の中で生成される。根拠は `#CAP` の束縛規則である --
`#CAP` を束縛するのはラムダの下降だけで、しかも `lam_ty.is_closure()` のときに限る
(`CODE src/rc_ir/lower.rs`)。グローバル初期化子の `InitValue#<symbol>` の直下では `#CAP` が束縛されず、
コード生成が名前解決に失敗する。

**本体が別の関数へ動かされることもない。** `ExprNode::free_vars` はラムダの自由変数から `CAP_NAME` を
落とし (`CODE src/ast/expr.rs`)、`closure_specialization` の `decapturable` は
`body.has_free_var(&FullName::local(CAP_NAME))` なラムダの持ち上げを拒否し、`let_elimination` の
`FreeOccurrenceProbe::new` は `CAP_NAME` を渡されると表明で落ちる。

## 関数ポインタを作るが Fix の関数型ではない値に入れる 2 件

`InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody` と
`InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody` は、`release#<型名>` / `retain#<型名>` という
ヘルパ関数を `gc.module.add_function(.., Linkage::Internal)` で新設し、その番地を取る。入れる先は
`Std::Ptr` であって Fix の関数型ではないので、`L0` の数え上げには入らない。この 2 つのヘルパは
`RcProgram::funcs` にも `InitValue#` にも属さず、コード生成器がその場で作るものである。

## `builtin.rs` の外にある 2 つの経路

`L0` の言明は `builtin.rs` の外の 2 つを数える必要がある。どちらも指す先は `RcProgram::funcs` である。

- **`Generator::build_rc_closure`** (`CODE src/rc_ir/codegen.rs: build_rc_closure`)。`Closure(fref, caps)`
  の評価がここに来る。`func_vals[func]` の番地を `CLOSURE_FUNPTR_IDX` へ書き込む。
- **funptr 型のグローバルの読み** (`CODE src/generator.rs: ValueAccessor::get`)。
  `ty.is_funptr()` の枝は `fun.as_global_value().as_basic_value_enum()`、すなわち**関数そのもの**を返す。
  クロージャ型のグローバルはアクセサ呼び出しで読むので、この枝を通らない。宣言は
  `declare_program_global` の `if ty.is_funptr() { return Some(self.declare_lambda_function(..)) }` で、
  実装は `implement_rc_program` が `implement_rc_function` で与える。

  **この経路は op の中に現れない。** どの op も、funptr 型のグローバルをオペランドに取れば、その読み出しの
  時点で関数の番地を得る。オペランドをそのまま返す `InlineLLVMMarkThreadedFunctionBody` や、容器へ入れる
  `InlineLLVMMakeStructBody` などがその値を結果に出す。

## `L0` の数え上げが除いてよいもの

- **FFI の戻り値**。`ret_tycon` は `is_c_scalar()` (`I8`..`U64`、`F32`、`F64`、`Ptr`) か `()` に限られる
  (`CODE src/fixstd/builtin.rs: c_boundary_tycon`)。`Arrow` は候補にならない。C の関数ポインタは `Ptr` と
  して受け取るしかなく、それは Fix の関数型ではない。
- **`[a : Boxed]` の付く型変数**。`Std::Boxed` は boxed struct / boxed union / `#DynamicObject` にしか
  実装されない (`CODE src/ast/program.rs: Program::add_boxed_impls`)。`Arrow` も `#FunPtr{n}` も
  `is_unbox: true` なので、`Boxed` 制約の付いた型変数は関数型にならない。これが `is_unique`、
  `boxed_to_retained_ptr`、`boxed_from_retained_ptr`、`get_boxed_ptr`、`mutate_boxed` 系の `a` を除く。
- **`undefined` と `hole`**。結果型は自由型変数なので関数型に具体化されうるが、生成コードは
  `RUNTIME_ABORT` の呼び出しか `build_unreachable()` の後に `Object::undef(ty)` を返す。実行時にこの値が
  存在することはない。
- **`#DynamicObject` の traverser 欄**。`create_obj` が `get_traverser_ptr` の作る関数を書き込むが、この欄は
  `#DynamicObject` にしか無く、78 件のどれも `#DynamicObject` を作らない (作るのは `build_rc_closure` だけ)。
  書き込み先はオブジェクトのヘッダであって Fix の関数型の値ではない。
