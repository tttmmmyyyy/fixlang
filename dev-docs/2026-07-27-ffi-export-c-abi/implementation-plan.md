# FFI_EXPORT の C ABI: 実装計画

issue #112。`FFI_EXPORT` した関数の引数・戻り値が集約型のとき、C 側と値が食い違う。

## 問題

`ExportStatement::implement` は wrapper を

```rust
codom.get_embedded_type(gc, &vec![]).fn_type(&dom_llvm_tys, false)
```

の形、つまり **LLVM IR の集約型をそのまま値渡し・値返し**で作る。LLVM はこれを要素ごとに `CC_X86_64_C` / `RetCC_AArch64_AAPCS` のレジスタへ割り付けるが、C の ABI は構造体を「サイズと eightbyte ごとのクラス」で分類する。両者が一致するのは形が偶然そろったときだけである。

`llc -O2` (LLVM 17.0.6) と `gcc -O2` で戻り値を突き合わせた結果:

| 形 | サイズ | C (x86-64) | 現状の Fix | AArch64 の C | 現状の Fix |
| --- | --- | --- | --- | --- | --- |
| `{i64, i64}` | 16 | RAX, RDX | 同じ | X0, X1 | 同じ |
| `{double, double}` | 16 | XMM0, XMM1 | 同じ | D0, D1 (HFA) | 同じ |
| `{i64, double}` | 16 | RAX, XMM0 | 同じ | X0, X1 | **X0, D0** |
| `{float, float}` | 8 | XMM0 に 2 個 | **XMM0, XMM1** | S0, S1 (HFA) | 同じ |
| `{i32, i32}` | 8 | RAX に 2 個 | **EAX, EDX** | X0 に 2 個 | **W0, W1** |
| `{i8, i8}` | 2 | RAX に詰める | **AL, DL** | X0 に詰める | **W0, W1** |
| `{i64, i64, i64}` | 24 | 隠しポインタ | **RAX, RDX, RCX** | X8 経由 | **X0, X1, X2** |

引数側も同じ性質である (24 バイトの構造体を C はスタックに置き、現状の Fix は RDI/RSI/RDX に置く。2 個の `float` を C は XMM0 に詰め、現状の Fix は XMM0/XMM1 に置く)。

**16 バイト超だけの問題ではない。** 8 バイト未満のフィールドが同じ eightbyte を共有する形は、小さくても壊れている。そして**安全な形の集合はターゲットごとに違う**: `(I64, F64)` は x86-64 で一致し AArch64 で壊れ、`(F32, F32)` は AArch64 で一致し x86-64 で壊れる。

`FFI_CALL` は影響を受けない。パーサが型名をスカラーの C 型に限定しており、集約型は構文エラーになる。

レイアウトそのものは既に C と一致している。Fix の unbox struct は非 packed の LLVM 構造体 (`context.struct_type(&fields, false)`) なので、同じ順序・同じフィールド型の C 構造体と同じオフセットになる。**直すのは受け渡しの規約だけ**である。

## 何を約束するか

「C ABI に合わせる」と言うには、まず Fix の型が C 側でどの型に見えるかを決める必要がある。現状 `Document.md` はスカラーの例しか書いていない。

| Fix の型 | C 側 |
| --- | --- |
| `I8`..`I64` / `U8`..`U64` / `F32` / `F64` / `Ptr` / `CInt` 等 | 対応するスカラー |
| `Bool` | `bool` (1 バイト) |
| `()` | `void` (戻り値のみ) |
| unbox struct / タプル | 同じ順序・同じフィールド型の C 構造体 |
| boxed 型 | 不透明ポインタ |
| union | タグ + payload バッファ (下記の判断待ち) |
| 固定長配列を含む unbox struct | 同じ配列を含む C 構造体 |

## 分類器

新モジュール `src/c_abi.rs` に、LLVM 型と triple から受け渡し方を返す関数を置く。

```rust
enum CAbiClass {
    /// レジスタで渡す。coerce した IR 型の列 (`i64`, `double`, `<2 x float>` 等)。
    Direct(Vec<BasicTypeEnum>),
    /// メモリで渡す。戻り値は `sret`、引数は `byval`。
    Indirect,
}
```

### x86-64 System V

1. サイズが 16 バイトを超えるものは MEMORY。
2. それ以外は eightbyte ごとに INTEGER / SSE を判定し、マージ規則 (INTEGER が勝つ) を適用する。
3. eightbyte のクラスから coerce する IR 型を決める。INTEGER は `i64`、SSE は含まれるフィールドに応じて `double` または `<2 x float>`。
4. 引数はレジスタ枯渇を見る。整数 6 本・SSE 8 本を使い切った後の集約は MEMORY へ落ちる。

Fix には x87 の `long double` も `__int128` も可変長引数も無いので、その分類は実装しない。

### AArch64 AAPCS64

1. HFA / HVA (同じ浮動小数型のメンバ 1-4 個) は SIMD レジスタで渡す。
2. それ以外で 16 バイト以下は汎用レジスタに詰める (`i64` の列に coerce)。
3. 16 バイトを超えるものは間接。戻り値は X8 (= `sret`)、引数はコピーのポインタ (= `byval`)。
4. 引数はレジスタ枯渇を見る (X0-X7 / V0-V7)。

### 表に無いターゲット

export をコンパイルエラーにする。黙って壊れた ABI を出すより、そのターゲットで export が使えないことを告げる方がよい。

## 実装

- `src/c_abi.rs`: `classify_return(llvm_ty, triple)` と `classify_arguments(llvm_tys, triple)`。引数側は枯渇を見るため列全体を受け取る。
- `ExportStatement::implement`: 分類の結果から wrapper の署名を組み、marshalling を書く。Direct は alloca 経由で bit を読み替える (`store` した後に coerce 型で `load`)、Indirect は `sret` / `byval` 属性を付けたポインタ引数にする。
- `ExportedFunctionType::validate`: 対応できない型を診断で拒否する。

## テスト

`src/tests/test_basic.rs` の `test_source_with_c` を使い、C 側で値を検証する差分テストを形の行列で用意する。CI が x86-64 (ubuntu) と AArch64 (macOS) の両方を回すので、そこが実証になる。

行列: `(I64,I64)` / `(I64,I64,I64)` / `(F64,F64)` / `(F32,F32)` / `(I32,I32)` / `(U8,U8)` / `(I64,F64)` / 入れ子の struct / 24 バイト超 / `Bool` / 引数を多数並べてレジスタを枯渇させる形。各形を引数と戻り値の両方で通す。

## フェーズ

1. 型の対応を決め、`validate` を対応済みの型に限定する。拒否の診断とそのテスト。
2. x86-64 の分類器と wrapper の書き換え。差分テスト。
3. AArch64 の分類器。CI で実証する。
4. `Document.md` に型の対応を書く。

## 判断が要る点

- **union を export 可能にするか。** Fix の union は「タグ + payload バッファ」で、C の `union` とは違う形である。C 側で対応する型を書くことはできるが、レイアウトを公開の約束にするかどうかは別の判断である。拒否する方が安全である。
- **boxed 型を export 可能にするか。** ABI 上は単なるポインタなので分類器の問題は無い。参照カウントの扱いを C 側に要求することになる。
- **レジスタ枯渇を忠実に実装するか。** 引数が多い export は稀だが、外すと黙って壊れる形が残る。
- **対応外ターゲットで export を拒否してよいか。**
