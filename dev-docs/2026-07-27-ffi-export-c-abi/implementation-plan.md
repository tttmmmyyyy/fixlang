# FFI_EXPORT の型を C ABI が保証できる範囲に限る: 実装計画

issue #112。`FFI_EXPORT` した関数の引数・戻り値が集約型（タプル・unbox struct・union を値渡し）のとき、C 側と値が食い違う。

## 問題

`ExportStatement::implement` は wrapper を

```rust
codom.get_embedded_type(gc, &vec![]).fn_type(&dom_llvm_tys, false)
```

の形、つまり **LLVM IR の集約型をそのまま値渡し・値返し**で作る。LLVM はこれを要素ごとにレジスタへ割り付けるが、C の ABI は構造体を「サイズと eightbyte ごとのクラス」（System V AMD64）あるいは「HFA かどうかと 16 バイト境界」（AAPCS64）で分類する。両者が一致するのは形が偶然そろったときだけである。

`llc -O2` (LLVM 17.0.6) と `gcc -O2` で戻り値を突き合わせた結果:

| 形 | サイズ | C (x86-64) | 現状の Fix | C (AArch64) | 現状の Fix |
| --- | --- | --- | --- | --- | --- |
| `{i64, i64}` | 16 | RAX, RDX | 同じ | X0, X1 | 同じ |
| `{double, double}` | 16 | XMM0, XMM1 | 同じ | D0, D1 (HFA) | 同じ |
| `{i64, double}` | 16 | RAX, XMM0 | 同じ | X0, X1 | **X0, D0** |
| `{float, float}` | 8 | XMM0 に 2 個 | **XMM0, XMM1** | S0, S1 (HFA) | 同じ |
| `{i32, i32}` | 8 | RAX に 2 個 | **EAX, EDX** | X0 に 2 個 | **W0, W1** |
| `{i8, i8}` | 2 | RAX に詰める | **AL, DL** | X0 に詰める | **W0, W1** |
| `{i64, i64, i64}` | 24 | 隠しポインタ | **RAX, RDX, RCX** | X8 経由 | **X0, X1, X2** |

引数側も同じ性質である（24 バイトの構造体を C はスタックに置き、現状の Fix は RDI/RSI/RDX に置く。2 個の `float` を C は XMM0 に詰め、現状の Fix は XMM0/XMM1 に置く）。

**16 バイト超だけの問題ではない。** 8 バイト未満のフィールドが同じ eightbyte を共有する形は、小さくても壊れている。そして**安全な形の集合はターゲットごとに違う**: `(I64, F64)` は x86-64 で一致し AArch64 で壊れ、`(F32, F32)` は AArch64 で一致し x86-64 で壊れる。

**スカラーは正しい。** 整数・浮動小数・ポインタは LLVM の既定の下ろし方が C ABI とそのまま一致する。8 バイト未満の整数の上位ビットも、C 側の callee が自分で拡張する（`movsbl %dil, %eax` / `sxtb w0, w0`）ので食い違わない。

`FFI_CALL` は影響を受けない。文法 `ffi_c_fun_ty = { ffi_c_ty | ffi_c_ty_unit | ffi_c_ty_ptr | number_lit_type }` が受け付けるのは C の型名（`CInt` 等、`c_type_sizes` を引いて `I32` 等に解決される）、Fix の固定幅型 `I8`..`U64` / `F32` / `F64`、`Ptr`、戻り値の `()` だけで、集約型は構文段階で弱かれる。

## 方針

**`FFI_EXPORT` を、ABI が保証できる型に限る。** 集約型は診断で拒否する。

集約型を渡せるようにするには、System V の eightbyte 分類と AAPCS64 の HFA 判定を Fix 側に実装し、coerce した IR 型（`i64` / `double` / `<2 x float>`）や `sret` / `byval` を出し分けることになる。引数側にはレジスタ枯渇の規則もある。ターゲットを増やすたびに保守が要る。それは機能追加であって、この issue が報告している「黙って壊れた値が渡る」ことの修正ではない。

修正としては、`FFI_CALL` が既に持っている制限に `FFI_EXPORT` を揃えるのが筋である。多値を C に渡したいときは、C 側で確保した領域のポインタを受け取り、`FFI_CALL` の `memcpy` で書き込む形が今も使える（Fix の unbox struct は非 packed の LLVM 構造体なので、同じフィールド順の C 構造体とレイアウトは一致する）。

### 既存コードへの影響

`FFI_EXPORT` の実例を調べた結果:

| 場所 | export した型 |
| --- | --- |
| minilib `_sandbox/ffi/dylib/mylib` | `I64 -> I64 -> I64`、`Ptr -> Ptr -> Ptr` × 2 |
| minilib `_sandbox/curl` | `Ptr -> CSizeT -> CSizeT -> Ptr -> CSizeT` |
| minilib `_sandbox/io/uv` | `Ptr -> ()` 系のコールバック |
| asynctask | `Box (IO Ptr) -> IO Ptr` |
| このリポジトリのテスト | `test_export`（`CInt`）、`test_export_unbox_struct_arg`（`(I64, I64)` の引数） |
| `Document.md` / `Document-ja.md` の例 | すべて `CInt` / `Ptr` |

集約型を使っているのはこのリポジトリのテスト 1 本だけである。**boxed 型（asynctask の `Box (IO Ptr)`）は使われている**ので、許可し続ける。

## 許可する型

| Fix の型 | C 側 | 理由 |
| --- | --- | --- |
| `I8`..`I64` / `U8`..`U64` | 対応する固定幅整数 | レジスタ 1 本、C と一致 |
| `F32` / `F64` | `float` / `double` | SIMD レジスタ 1 本、C と一致 |
| `Ptr` | `void *` | レジスタ 1 本 |
| boxed 型 | 不透明ポインタ | LLVM 型がポインタ 1 個 |
| `()` | `void` | 戻り値のみ |

`CInt` などの C 型名は上記の固定幅型の別名なので、そのまま通る。

拒否する型と、その理由:

- **unbox struct・タプル・union**（この issue の本体）
- **`Bool`** — System V と AAPCS64 は `_Bool` を 1 バイトと定めているが、C 側で `int` や独自 typedef の `BOOL` を書けば黙って食い違う。`U8` か `CInt` を明示してもらう
- **`Array` / `String` などの unbox でない標準型** — boxed なら不透明ポインタとして通る

## 実装

- `ExportedFunctionType::validate`: `doms` の各要素と `codom` を上記の集合で検査し、外れたら診断を返す。`()` は戻り値の位置でのみ許す（引数の `()` は現在 `build_ffi_call_core` が実行時に `panic_with_msg_src` するが、export 側は `validate` で弾く）。
- 診断の文面は、拒否した型と、`Ptr` + `FFI_CALL` の `memcpy` で受け渡す回避策を示す。
- `ExportStatement::implement` は変更しない。許可した型はすべて LLVM 型がスカラーかポインタ 1 個で、現状の値渡しがそのまま C ABI と一致する。

## テスト

- 拒否のテスト: タプル・unbox struct・`Bool` を export したときに診断が出ること（`test_source_fail` の形）。
- 通過のテスト: 現行の `test_export`（`CInt`）に加え、`I64` / `F64` / `Ptr` / boxed 型 / `()` 戻り値を C から呼ぶ形。
- `test_export_unbox_struct_arg` は削除する。16 バイトの引数がたまたま一致することを固定しているテストであり、この変更で拒否される形になる。
- 回避策のテスト: `Ptr` を受け取り `FFI_CALL` の `memcpy` で C 側の構造体に書き込む形が、C 側から見て正しい値になること。

## 参照カウント用の関数ポインタのテスト

`Std::FFI::get_funptr_retain` と `get_funptr_release` は、外部言語から `void (*)(void*)` として呼ぶための関数ポインタを返す。ドキュメントはこれで参照カウンタが増減すると約束しているが、既存のテストは**取得までしか見ていない**。

| 既存のテスト | 見ているもの |
| --- | --- |
| `test_unsafe_get_release_retain_function_of_boxed_value_decltype_technique_1` / `_2` | 呼べて型が付くこと |
| `test_get_funptr_retain_error` / `test_get_funptr_release_error` | boxed でない型に使うとコンパイルエラーになること |

いずれも取得した関数ポインタを呼ばずに `pure()` で終わり、テスト自身のコメントが「実際の使用は asynctask.fix でテストされている」と外部プロジェクトに委ねている。

`FFI_EXPORT` の型制限で FFI 周辺のテストを触るので、ここも埋める。`test_source_with_c` で C 側から関数ポインタを呼び、次を確かめる。

- retain を呼ぶと参照カウンタが増え、Fix 側が手放しても値が生き続けること。`boxed_from_retained_ptr` で戻した値が読めること
- release を必要回数呼ぶと解放されること。解放の観測には `Destructor` を使い、その dtor から
  `FFI_CALL` で C の関数を呼んでフラグを立て、テストの最後にそのフラグを読む（`IORef` は標準
  ライブラリに無い）
- ドキュメントの手順（責任の回数は retain N 回に対して N+1 回）どおりに使えば辻褄が合うこと
- valgrind で leak も double free も出ないこと

## ドキュメント

`Document.md` と `Document-ja.md` の FFI の節を直す。

- `FFI_EXPORT` できる型の一覧と、拒否される型（集約型・`Bool`）
- **多値を返す方法**: C 側で確保した領域のポインタを引数で受け取り、`FFI_CALL` の `memcpy` で書き込む形。Fix の unbox struct は非 packed の LLVM 構造体なので、同じフィールド順の C 構造体とレイアウトが一致することを述べる。コード例を両言語で置く
- `get_funptr_retain` / `get_funptr_release` の節は、上のテストで確かめた手順と一致しているかを読み直す

## 将来: 集約型を渡せるようにする場合

この計画では実装しないが、やるとすれば次の形になる。

- 新モジュール `src/c_abi.rs` に `classify_return(llvm_ty, triple)` と `classify_arguments(llvm_tys, triple)` を置き、`Direct(coerce した型の列)` か `Indirect`（`sret` / `byval`）を返す。
- System V: 16 バイト超は MEMORY。16 バイト以下は eightbyte ごとに INTEGER / SSE を判定し、マージ規則（INTEGER が勝つ）を適用して coerce する。引数は整数 6 本・SSE 8 本の枯渇を見る。
- AAPCS64: HFA / HVA（同じ浮動小数型のメンバ 1-4 個）は SIMD レジスタ、16 バイト以下は汎用レジスタに詰める、超えるものは間接。
- 表に無いターゲットでは集約型の export を拒否する。
- 両ターゲットの CI で、プラットフォームの C コンパイラと突き合わせる差分テストを形の行列で回す。
