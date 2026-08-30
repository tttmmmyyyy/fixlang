# `LLVMGen` の 78 実装のうち、関数の値を作るもの

README の A21 は「Fix の関数型の値に LLVM 関数の番地を書き込むのは、クロージャを作る段、funptr のグローバル
を読む段、そして `InlineLLVMFixBody` の 3 か所だけである」を置き、その果たす者を `builtin.rs` の op の集合と
する。この文書がその数え上げである。`p51-runs.md` の `L0` が A21 を引く。
**`src/fixstd/builtin.rs` の `impl LLVMGen for` 78 個すべてと、それらが呼ぶヘルパを読んだ。**

読んだコードはコミット `89ae3cf54a2f2e98ae879e0d6cc3b4f722368466` の版である。

## 分類

各実装が、Fix の関数型 (`Arrow a b` すなわちクロージャ、または `Std::#FunPtr{n}`) の値を実行時に作りうるか
で分ける。

| 類 | 意味 | 件数 |
|---|---|---|
| (A) | 関数の値を作らない | 70 |
| (B) | 受け取った関数の値を写すだけ (load / extractvalue / bit_cast / `apply_lambda` の戻り値 / オペランドそのもの) | 7 |
| (C) | 自ら関数ポインタを作って Fix の関数型の値へ書き込む | **1** |

この表は、**op の結果の値そのものが Fix の関数型である**ものを (B) に数えている。その 7 件:
`InlineLLVMArrayUnsafeGetBoundsUnchecked`、`InlineLLVMStructGetBody`、
`InlineLLVMCaptureProjectBody`、`InlineLLVMUnionAsBody`、`InlineLLVMWithRetainedFunctionBody`、
`InlineLLVMArrayBorrowElementsBody`、`InlineLLVMMarkThreadedFunctionBody`。

### 結果の成分まで数える読み

もう一つの読みは、結果の**成分**が関数型になりうるものまで (B) に数える。成分は次の規則で取る。

- 結果型に現れる型構築子の宣言に沿って降りる -- struct のフィールド、union の variant の payload、
  `Std::Array` の要素。`Std::FFI::Destructor` は `_dtor : a -> IOState -> (IOState, a)` を持つ struct なので、
  これを結果に含む op はこの規則で (B) に入る。
- **型変数は葉であり、その先の実体化には降りない。**葉の型変数は、それを関数型に実体化できるときだけ数える。
  `[a : Boxed]` の付く変数は、下の「`L0` の数え上げが除いてよいもの」の節が言うとおり関数型にならないので、
  葉のまま数えない。
- 同じ節が除く `undefined` と `hole` は、この読みでも (A) に置く。

この規則で **(A) 46 / (B) 31 / (C) 1** になる。(C) はどちらの読みでも 1 件である。

(B) の 31 件は、上の 7 件と次の 24 件である (`builtin.rs` に現れる順)。

`InlineLLVMArrayUnsafeEmpty`、`InlineLLVMArrayTruncateBoundsUnchecked`、
`InlineLLVMArrayAppendValueCapacityUnchecked`、`InlineLLVMArraySetCapacityBoundsUnchecked`、
`InlineLLVMArrayAppendCapacityUnchecked`、`InlineLLVMArrayCopyCapacityBoundsUnchecked`、
`InlineLLVMArrayGrowSizeBody`、`InlineLLVMArraySetBody`、`InlineLLVMArraySwapBody`、
`InlineLLVMArrayPunchBody`、`InlineLLVMPunchedArrayPlugBody`、`InlineLLVMMakeStructBody`、
`InlineLLVMArrayLitBody`、`InlineLLVMStructPunchBody`、`InlineLLVMStructPlugInBody`、
`InlineLLVMStructSetBody`、`InlineLLVMMakeUnionBody`、`InlineLLVMUnionModBody`、
`InlineLLVMArrayIsStorageUniqueBody`、`InlineLLVMUnsafeMutateBoxedInternalFunctionBody`、
`InlineLLVMUnsafeMutateBoxedIOSInternalBody`、`InlineLLVMArrayMutateElementsInternalBody`、
`InlineLLVMArrayMutateElementsIosInternalBody`、`InlineLLVMDestructorMake`。

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
(`CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function`)。グローバル初期化子の `InitValue#<symbol>` の
直下では `#CAP` が束縛されず、コード生成が名前解決に失敗する。

**本体が別の関数へ動かされることもない。** `ExprNode::calc_free_vars` はラムダの自由変数から `CAP_NAME` を
落とし (`CODE src/ast/expr.rs: ExprNode::calc_free_vars`)、`closure_specialization` の `decapturable` は
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

  **この経路は op の `generate` の中でも通る。** `Lowerer::lower_llvm` は、op の自由変数のうち束縛の解けない
  名前を、その記号の型を付けた `RcVar` としてオペランドに残す
  (`CODE src/rc_ir/lower.rs: Lowerer::lower_llvm`)。op が `gc.get_scoped_obj` でそのオペランドを読むと、
  この枝が関数の番地を返す。番地を作るのはその読み出しであって op ではないので、op の側は (B) のままである --
  オペランドをそのまま返す `InlineLLVMMarkThreadedFunctionBody` や、容器へ入れる
  `InlineLLVMMakeStructBody` などがその値を結果に出す。

## `L0` の数え上げが除いてよいもの

- **FFI の戻り値**。型検査は `FFI_CALL` の結果型を `type_tycon(ret_ty)` -- 引数を 1 つも取らない型構築子
  そのもの、`FFI_CALL_IO` と `FFI_CALL_IOS` ならそれを `(IOState, ret)` に包んだもの -- に定める
  (`CODE src/elaboration/typecheck.rs: TypeCheckContext::unify_type_of_expr_inner`)。その `ret_ty` は書かれた
  名前から作る単独の `TyCon` である (`CODE src/parse/parser.rs: parse_ffi_c_fun_ty`)。`Arrow` の種は
  `* -> * -> *`、`Std::#FunPtr{n}` の種は矢印 `n` 本なので、引数を取らないその位置に来られない。C の関数
  ポインタは `Ptr` として受け取るしかなく、それは Fix の関数型ではない。
- **`[a : Boxed]` の付く型変数**。`Std::Boxed` の instance は、boxed struct と boxed union に
  `Program::add_boxed_impls` が (`CODE src/ast/program.rs: Program::add_boxed_impls`)、`#DynamicObject` に
  `make_std_mod` が (`CODE src/fixstd/stdlib.rs: make_std_mod`) 与えるものだけであり、利用者が手で書いた
  instance は `TraitEnv::validate_trait_impl` が拒む (`CODE src/ast/traits.rs: TraitEnv::validate_trait_impl`)。
  `Arrow` も `#FunPtr{n}` も `is_unbox: true` なので、`Boxed` 制約の付いた型変数は関数型にならない。これが
  `is_unique`、`boxed_to_retained_ptr`、`boxed_from_retained_ptr`、`get_boxed_ptr`、`mutate_boxed` 系の `a`
  を除く。
- **`undefined` と `hole`**。結果型は自由型変数なので関数型に具体化されうるが、生成コードは
  `RUNTIME_ABORT` の呼び出しか `build_unreachable()` の後に `Object::undef(ty)` を返す。実行時にこの値が
  存在することはない。
- **`#DynamicObject` の traverser 欄**。`create_obj` が `get_traverser_ptr` の作る関数を書き込むが、この欄は
  `#DynamicObject` にしか無く、78 件のどれも `#DynamicObject` を作らない (作るのは `build_rc_closure` だけ)。
  書き込み先はオブジェクトのヘッダであって Fix の関数型の値ではない。

## この数え上げを検査する仕組みは無い

この数え上げを支えているのは `impl LLVMGen for` 78 個の通読だけである。(C) が 1 件であることを実行時に
確かめる表明も、ビルド時に確かめる走査も無い。**op が 1 つ足されるたびに数え直すのが、この数え上げを保つ
唯一の道である。**

`Generator::apply_lambda` が develop mode で検査するのは `LLVMGen::applies_a_function_operand` --
op が関数を**適用**するか -- であり、この文書が数える「関数の値を**作る**」については何も言わない。
関数の値を作って適用しない op が足されても、その表明は鳴らない。
