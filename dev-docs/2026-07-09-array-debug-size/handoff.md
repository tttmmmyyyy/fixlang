# Handoff: デバッガに配列の実行時サイズを伝える（固定 100 → 動的 DW_AT_count）

## タスク
デバッガ（gdb / lldb）で **`Array a` / `String` の要素を、実行時の正しいサイズで表示できるようにする**。
現状はデバッグ情報が配列の要素数を**固定 100** に埋め込んでいるため、正しく表示できていない。これを、配列構造体が実行時に持つ `size` フィールドを参照する **動的な配列境界（DWARF の `DW_AT_count` / `DISubrange` の動的 count）** に置き換える。

このブランチ `array-debug-size` は `unique-check-elim` から生やしている（デバッガの E2E テスト基盤 `src/tests/test_debug_info.rs` が最初に入ったブランチ）。その上に配列表示の改善とテストを載せる。

## 現状の限界（ユーザが未解決だった問題）
`Document-ja.md` の 3241-3242 行:
- （3241）Fix はローカル変数をスコープ末でなく **最終使用時に解放**する。so 最終使用より後でブレークすると解放済みの無効値が見える。
- （3242）**「実行時に決定される配列サイズをデバッガに伝えられないので、デバッグ情報では配列サイズを常に 100 に設定している。100 超のインデックスは表示できず、配列が 100 より短いと無効値が表示される」**。

観測（この worktree の親 `unique-check-elim` で確認済み）:
- **lldb**: 配列要素は見える（固定 100 のため）。ただし >100 は出ない・実サイズ<100 だと余分がゴミ表示（上記 doc の制約）。
- **gdb**: `print *arr` が `<array elements> = <error reading variable: access outside bounds of object>` とエラー。固定 100 要素を読もうとして実 object（例 3 要素）を超えるため gdb が範囲外と判定。
- **データはメモリに在る**: `x/8dg arr` で配列構造体が `[refcnt=1, size=3, capacity=3, 10, 20, 30, …]` と並び、要素 `10,20,30` はそのまま読める。so 欠けているのは「デバッガに実サイズを伝える DWARF 属性」だけ。

## 原因コード
`src/object.rs` の `ObjectFieldType::Array(elem_ty)` の**デバッグ型生成分岐**（おおよそ L227-300）。核心は要素配列型の生成:

```rust
// src/object.rs（L272-280 付近）
let element_array_ty = gc
    .get_di_builder()
    .create_array_type(
        element_debug_ty,
        element_size_in_bits,
        element_align_in_bits,
        &[0..100],            // ← 固定 100。ここを「size フィールド参照の動的 count」に変える
    )
    .as_type();
```

配列構造体のデバッグ表現は `{ <control block>, <array size> : I64, <array buffer> { <array capacity> : I64, <array elements> : [elem; 100] } }`（同 L227-310）。
- `<array size>`（`ARRAY_...` の size メンバ）が**実行時の有効要素数**を持つ。これを `<array elements>` の subrange の count に参照させたい。

## 解法（DWARF）
DWARF では `DW_TAG_array_type` の子 `DW_TAG_subrange_type` の要素数を、定数でなく:
- 別の DIE（変数/メンバ）への参照、または
- ロケーション式（例: `DW_OP_push_object_address` で被記述オブジェクトの先頭を得て、`size` メンバのオフセットを足して deref）

で表せる。**gdb も lldb もこの動的境界を解釈**するので、`size` を参照させれば両方で正しい要素数表示になる（C の VLA・Fortran/Ada の動的配列と同じ機構）。狙いは「`<array elements>` の count ＝ 同じ配列構造体の `<array size>` の実行時値」。

## 実装上の壁（inkwell）
本コードは LLVM の Rust バインディング **inkwell** を使う。`DIBuilder::create_array_type(inner, size, align, subscripts: &[Range<i64>])` は **定数レンジしか受け取らない**（だから今 `&[0..100]`）。動的 count を出すには:
- LLVM の `DISubrange` を **count = `DIVariable` または `DIExpression`** で作る必要がある（LLVM 本体は対応済み: `DIBuilder::getOrCreateArrayType` ＋ `DISubrange::get(..., count, ...)`）。
- **inkwell の高レベル API がこれを露出しているか要調査**。無ければ (a) inkwell の該当メソッド追加/PR、(b) `llvm-sys` の生 API（`LLVMDIBuilderGetOrCreateSubrange` 相当・または `LLVMDIBuilderCreateArrayType` に DISubrange を渡す形）を直接呼ぶ、のいずれか。
- **`size` はメンバ**（ローカル変数でない）なので、count は「被記述オブジェクト先頭からの size メンバのオフセットを読むロケーション式」で表すのが素直（`DW_OP_push_object_address, DW_OP_plus_uconst <size_off>, DW_OP_deref`）。inkwell/llvm-sys で `DIExpression` を組んで `DISubrange` に渡せるか確認する。

（もし動的 count が難しければ、次善策として固定値を 100 でなく大きめにしてもゴミ表示問題は残るので、**本筋は動的 count**。）

## 検証
1. **手動（gdb と lldb 両方）**: サイズ 3 と 150（>100）の `Array I64` を持つ小プログラムを、その配列が**生存する地点**（Fix は最終使用で解放するので、配列をブレーク行より後で使う）でブレークし、要素が正しく（3 要素 / 150 要素とも・ゴミ無し）表示されることを確認。`x/` で答え合わせ。
2. **自動テスト（`src/tests/test_debug_info.rs` を拡張）**: 既存の gdb 駆動テストに倣い、`fix build -g`（`-g` は自動で `-O none`）→ `gdb -batch` で配列要素を `print` し、`[10, 20, 30]` 等が出ることを assert。**注意: 親ブランチ `unique-check-elim` には P0.5 で `test_debug_info_variable_values`（int/bool 値・array は型と `<array size>=3` まで）が入る予定**。それが landing 後にこのブランチを rebase し、その配列 assert を「要素本体まで」強化する形が綺麗。lldb でも回すなら lldb -batch 版も足す。
3. **回帰**: 既存の debug ベースライン（`test_debug_info_baseline`）が壊れないこと。`cargo test --release`（CLAUDE.md 参照）。

## 仕上げ
- `Document-ja.md` 3242 と `Document.md` の対応箇所（「配列サイズを常に 100」）を、修正後の挙動に**更新**（制約が消える旨）。
- 変数が最終使用で解放される件（3241）は別問題なので触らない。
- CHANGELOG は user-visible な改善（デバッガで配列/文字列の中身が正しく見える）なので `### Fixed` か `### Changed` に一行。ただし public API でなくツール挙動なので `#### Tool` 想定。

## 参考
- 原因: `src/object.rs` `ObjectFieldType::Array` デバッグ分岐（`create_array_type(..., &[0..100])`）。
- 現状記述: `Document-ja.md` 3241-3242。
- デバッグ情報の他の生成: 同 `src/object.rs` の `ty_to_debug_struct_ty` / `to_debug_type` 系、`create_member_type` / `create_struct_type` の使い方。
- `String = struct { _data : Array U8 }` なので、配列表示が直れば文字列バイト列表示も改善する（別途 U8→文字列の pretty 表示までやるかは任意）。
