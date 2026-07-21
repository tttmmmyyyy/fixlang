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
| `InlineLLVMIsUniqueFunctionBody` | 返す値は引数と同じ共有状態だが、`result_prov` でそれを言うと消費まで消えてチェックが不正直になる（test95）。共有状態の側だけを `provenance.rs::is_unique_result` の局所ルールで伝えるようにした |
| `InlineLLVMUndefinedInternalBody` | abort する op なので値は存在しないのに、結果を `Dyn` と宣言していた。`if bad { undefined(msg) }; value` という Fix のガード節が合流で provenance を潰していたので、束の底（∅ = 合流の単位元）に変えた。`Debug::assert_unique` を挟むだけで後続のチェックが落ちなくなる（= 調べる行為が調べたい最適化を消す）状態が解消 |

## フレームワーク側の非対称性（fail-loud 化した）

`result_prov` の leaf は `BaseSource` の**集合**だが、読み手 2 つの解釈が食い違っていた:

- `borrow.rs::root_inner` -> `single_arg`: 集合が `{Arg}` ちょうど 1 個のときだけ別名
- `borrow.rs::collect_arg_leaves`: 集合の中に `Arg` が 1 つでもあれば「その引数 leaf は消費されない」

よって `{Fresh | Arg(0)}` のような join を宣言すると、**消費だけ消えて別名リンクは張られず二重 release**。
`collect_arg_leaves` で `unreachable!` にし、不変条件（`Arg` はその leaf の唯一のソース）を
`LLVMGen::result_prov` の doc に明記した。`InlineLLVMUnionModBody` は意味的にこの join が正確な唯一の op で、
つまり現状の枠組みでは精密化できない（実装するなら枠組み側の変更が要る）。

## 共有状態だけを伝える宣言（`SharingOf`）は保留

`Arg` は「引数と同じ共有状態」と「引数のエイリアスなので消費しない」を同時に主張する。前者だけ言いたい
op — force-unique せずに引数の格納域をそのまま返すもの — は `Dyn` に落とすしかなく、そのための言い訳
コメントがこの監査で 4 件付いた。分離するなら `BaseSource` に「共有状態は同じだが所有権は移る」変種を
足し、`compose`/`resolve` は `Arg` と同じ、`borrow.rs` は無視（= 借用挙動は `Dyn` と同一）にすればよい。

**保留の理由**: 最適化が生む非 force-unique 版は解析に見られない（`analyze_program` は elision の前に
1 回だけ走る）。したがって効くのは「ソースから直接到達できる非 force-unique op」だけで、非 fu の Fix 関数を
廃止する方針のもとでは punch/plug の非 fu 版・`_unsafe_set_bounds_uniqueness_unchecked_unreleased`・
`unsafe_set_size` はいずれも消える。残るのは `is_unique` だけなので、そこは変種を足さずに
`provenance.rs::is_unique_result` の局所ルールで埋めた。

変種を入れるべきになるのは、(a) elision の後にもう一度解析を回す（analyze -> elide -> analyze）に踏み切る
とき — そうすれば非 fu 版の宣言が意味を持ち、std の `act` を checking punch に替えた回避策も要らなくなる —
または (b) Array/Storage 再設計が force-unique しない storage プリミティブを複数残すとき。

## 残っている最大の妨げ: unbox getter が sibling を op 内で release する

`ObjectFieldType::get_struct_fields` は unbox 容器のとき、取り出さないフィールドを**その場で release** する
（`object.rs`）。したがって `@field` は容器の全 boxed leaf を消費し、容器を後でも使うと retain が入って
provenance が落ちる。実測（unbox struct をループ状態にし、スカラのフィールドを読んでから配列フィールドを
更新する形）では、punch/plug を passthrough にしても**この retain で鎖が切れ**、`Array::set` のチェックが残る。
容器がその getter で最後の使用になる形（`mod_x` だけでループを回す等）では鎖が通り、上記のベンチの利得になる。

plan の [#F4] / [#R12-3] は「unbox getter は純射影 op（op 内で retain/release しない）・落とす sibling は
明示 `Release` が引き受ける」を要求しており、これは未解決項目のまま。解消すれば
「フィールドを読んでから同じ構造体を更新する」という最も普通の形でもチェックが落ちる。

## 規則: 一意性を契約で要求する unsafe プリミティブは結果を一意所有と宣言する

`_unsafe_set_bounds_uniqueness_unchecked_unreleased` / `unsafe_set_size` は、名前が言っているのは
「一意性チェックが無い」であって「一意性が不要」ではない。in-place で書き込む以上、呼び手が一意な配列を
持っていることが契約であり、契約が満たされていれば結果は一意所有である。したがって `Fresh` が正しい宣言で、
これで `from_map` / `fill` / `push_back` で組んだ配列が以後 unique と分かる（それまでは fill ループを
抜けた時点で `Dyn` に落ちていた）。punch の非 force-unique 版も同じ理由で無条件 `Fresh` にした
（最適化が一意性を証明したか、呼び手が約束したかのどちらかでしか走らない）。std 側の呼び出しは全件が
新規確保直後か `_unsafe_force_unique` 直後で、契約を満たしている。

この 2 つは Array/Storage 再設計で消えるが、**規則は後継に引き継ぐこと**。再設計後に残る
「一意性を契約で要求する」プリミティブ（`_unsafe_initialize` / `unsafe_set_bounds_unchecked` など）は、
最初から結果を `Fresh` と宣言する。

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
