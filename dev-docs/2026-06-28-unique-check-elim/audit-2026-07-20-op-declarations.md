# InlineLLVM op の RC 宣言の監査（2026-07-20）

- 対象: `src/fixstd/builtin.rs` の `impl LLVMGen` 全 74 件
- 見る宣言: `result_prov`（結果の由来）・`borrows_operand`（オペランドの所有）・`unique_check_operand`
  （実行時 uniqueness チェックを持つことの申告）
- 方法: 3 分割してサブエージェントに全件通読させ（各 op の `generate` 本体・`object.rs` のヘルパまで確認）、
  併せて `make_array_unique` / `make_struct_union_unique` の全呼び出し箇所から
  「チェックを持つのに申告していない op」を洗った。

## 対応したもの

| op | 内容 |
|---|---|
| `InlineLLVMStructSetBody` / `InlineLLVMStructPunchBody` / `InlineLLVMStructPlugInBody` | unbox struct の更新経路が leaf ごとの `Arg` を宣言していなかった（boxed 側だけ `Fresh`）。unbox struct はレジスタ上で分解・再構成するだけで新しい参照を作らないので、`MakeStructBody` と同形の passthrough が正確。**ベンチ `struct_field_mod` で -29.5%**（3,406,169,155 -> 2,401,769,183 Ir） |
| `InlineLLVMMarkThreadedFunctionBody` | 既定の `Dyn` は**意図的**（plan の「プリミティブ宣言（`result_prov`）」・[5]・[#F4] の `is_unique ⟹ LOCAL` 補題の土台）なのに、その旨がコードに無かった。明示 override + 理由コメントにした。passthrough にすると呼び出し側が threaded object への `Fresh` handle を保持でき、他スレッドから見える値を素で in-place 破壊する |
| `InlineLLVMArrayUnsafeSetBoundsUniquenessUncheckedUnreleased` / `InlineLLVMArrayUnsafeSetSizeBody` | passthrough にできるが `is_unique` と同じ罠のため `Dyn` のまま。理由と失うもの（この配列への後続 `set` がチェックを保つ）をコメントに残した |
| `InlineLLVMUnsafeMutateBoxedInternalFunctionBody` / `InlineLLVMUnsafeMutateBoxedIOSInternalBody` | force-unique した値を返すのに `result_prov` 未宣言、かつ `unique_check_operand` も未宣言で自分のチェックも畳めなかった。返る値の leaf を `Fresh` にし、`force_unique` フラグ + `unique_check_operand` / `assuming_unique` を足した。**ベンチ `mutate_boxed_loop` で -8.4%**（598,306,544 -> 548,266,485 Ir） |

## フレームワーク側の非対称性（fail-loud 化した）

`result_prov` の leaf は `BaseSource` の**集合**だが、読み手 2 つの解釈が食い違っていた:

- `borrow.rs::root_inner` -> `single_arg`: 集合が `{Arg}` ちょうど 1 個のときだけ別名
- `borrow.rs::collect_arg_leaves`: 集合の中に `Arg` が 1 つでもあれば「その引数 leaf は消費されない」

よって `{Fresh | Arg(0)}` のような join を宣言すると、**消費だけ消えて別名リンクは張られず二重 release**。
`collect_arg_leaves` で `unreachable!` にし、不変条件（`Arg` はその leaf の唯一のソース）を
`LLVMGen::result_prov` の doc に明記した。`InlineLLVMUnionModBody` は意味的にこの join が正確な唯一の op で、
つまり現状の枠組みでは精密化できない（実装するなら枠組み側の変更が要る）。

## 残っている最大の妨げ: unbox getter が sibling を op 内で release する

`ObjectFieldType::get_struct_fields` は unbox 容器のとき、取り出さないフィールドを**その場で release** する
（`object.rs`）。したがって `@field` は容器の全 boxed leaf を消費し、容器を後でも使うと retain が入って
provenance が落ちる。実測（unbox struct をループ状態にし、スカラのフィールドを読んでから配列フィールドを
更新する形）では、punch/plug を passthrough にしても**この retain で鎖が切れ**、`Array::set` のチェックが残る。
容器がその getter で最後の使用になる形（`mod_x` だけでループを回す等）では鎖が通り、上記のベンチの利得になる。

plan の [#F4] / [#R12-3] は「unbox getter は純射影 op（op 内で retain/release しない）・落とす sibling は
明示 `Release` が引き受ける」を要求しており、これは未解決項目のまま。解消すれば
「フィールドを読んでから同じ構造体を更新する」という最も普通の形でもチェックが落ちる。

## 削除予定のため見送ったもの

いずれも force-unique するのに `unique_check_operand` を申告しておらず、チェックが除去対象にならない。
Array/Storage 再設計（`dev-docs/2026-07-18-array-buffer-representation/design.md`）で op ごと消えるため、
今回は触っていない。

- `InlineLLVMArrayForceUniqueBody`（`push_back` / `sort` / `reserve` が使う）
- `InlineLLVMArrayPopBackNonemptyBody`
- `InlineLLVMArrayUnsafeGetLinearBoundsUncheckedUnretained` の force-unique 版（`result_prov` も未宣言）

## borrows_operand

取りこぼしはゼロ。ボックス値を取る op はいずれもオペランドを実際に消費しているか、既に
`borrows_operand` を宣言している（3 エージェントの結論が一致）。
