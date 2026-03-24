# Opaque Type テスト計画

本書は opaque type 機能の実装に対するテスト計画である。

---

## テストモジュールの構成

- **`src/tests/test_opaque_type.rs`**: opaque type の基本機能・バリデーションのユニットテスト
- **`src/tests/test_lsp/cases/`**: LSP hover テスト用の Fix プロジェクト
- **`src/tests/test_lsp/test_hover.rs`**（新規）: LSP hover の統合テスト

---

## 1. 基本機能テスト（test_opaque_type.rs）

### 1-1. use_cases.md のサンプルコード

use_cases.md に記載されている各ユースケースが正しくコンパイル・実行できることを確認する。

| テスト名 | 内容 | 対応する use_cases.md |
|---------|------|---------------------|
| `test_opaque_repeat` | `repeat` 関数：イテレータ戻り値型の簡略化 | ユースケース1 |
| `test_opaque_doubled_evens` | 複数コンビネータの連鎖（型の爆発回避） | ユースケース2 |
| `test_opaque_to_iter` | `ToIter` trait member での使用 | ユースケース3 |
| `test_opaque_to_iter_multiple_impls` | `ToIter` を複数の型（例：`Array a` と自作の `MyList a`）で実装し、main 内で両方の `to_iter` を呼び出して結果を使用する。異なる impl ごとに opaque type が異なる具体型に解決されることを確認 | ユースケース3 の拡張 |
| `test_opaque_higher_kinded` | higher-kinded opaque type（`?m : * -> *`） | ユースケース4 |
| `test_opaque_zip_with_index` | 複数の opaque type を持つシグネチャ | ユースケース5 |
| `test_opaque_partition` | 複数 opaque type（partition） | ユースケース6 |
| `test_opaque_predicate_only` | equality なし（predicate のみ） | ユースケース7 |
| `test_opaque_higher_arity_associated_type` | higher-arity associated type（Rebuildable パターン） | ユースケース8 |

### 1-2. trait impl のアノテーションにおける opaque type

trait メソッドの定義ではなく、**impl 側の型アノテーション**で opaque type を使用するケース。

| テスト名 | 内容 |
|---------|------|
| `test_opaque_in_impl_annotation` | impl メソッドの型アノテーションに opaque type を含むケース。例：impl 側で `\|x\| (x.to_iter : ?it)` のように型アノテーションで opaque type を参照し、型推論が正しく動作することを確認 |

### 1-3. higher-kinded opaque type の追加ケース

| テスト名 | 内容 |
|---------|------|
| `test_opaque_higher_kinded_functor` | `?f : * -> *` でかつ `Functor` 制約を持つ opaque type。`map` が使えることを確認 |

### 1-4. opaque type に対する associated type の使用

opaque type の導入により associated type の利用パターンが増える。以下のテストで associated type が opaque type と組み合わせて正しく動作することを確認する。

| テスト名 | 内容 |
|---------|------|
| `test_opaque_with_associated_type_basic` | `Item ?it = a` のような基本的な associated type equality |
| `test_opaque_with_higher_arity_assoc_type` | `Rebuild ?c b = Array b` のような higher-arity associated type |
| `test_opaque_with_higher_kinded_assoc_type` | `MyResult ?m = f` のような higher-kinded associated type（カインド `* -> *`） |
| `test_opaque_associated_type_reduction` | opaque type の associated type が使用側で正しく簡約されることを確認（例：`Item (?it String)` → `String`） |
| `test_opaque_multi_opaque_with_shared_assoc_type` | 複数の opaque type が同じ associated type 制約を共有するケース |

### 1-5. 同一関数の複数回呼び出し

| テスト名 | 内容 |
|---------|------|
| `test_opaque_multiple_calls_different_type_args` | 異なる型引数で同一 opaque 関数を複数回呼び出す。例：`repeat("hello", 3)` と `repeat(42, 3)` が異なる型として共存 |
| `test_opaque_multiple_calls_same_type_args` | 同じ型引数で同一 opaque 関数を複数回呼び出す。例：`[repeat("a", 3), repeat("b", 3)]` が `Array (?it String)` として統一される |

---

## 2. バリデーションテスト（エラーケース）

各 validation ルールに対して、違反するコードがコンパイルエラーとなり、適切なエラーメッセージが出力されることを確認する。

### 2-1. V-1: opaque 型変数の使用箇所制限

| テスト名 | 内容 | 期待するエラーメッセージ（部分一致） |
|---------|------|-------------------------------|
| `test_opaque_in_type_defn` | 型定義（struct/union）の型パラメータに `?` 始まりの型変数を使用 | "Opaque type variable" / 使用箇所制限に関するエラー |
| `test_opaque_in_trait_defn` | trait 定義の型パラメータに `?` 始まりの型変数を使用 | 同上 |
| `test_opaque_in_impl_type_param` | trait impl の型パラメータに `?` 始まりの型変数を使用 | 同上 |

### 2-2. V-3: equality 制約の formal parameter チェック

| テスト名 | 内容 | 期待するエラーメッセージ（部分一致） |
|---------|------|-------------------------------|
| `test_opaque_equality_non_tyvar_formal_param` | equality の仮引数位置に具体型（例：`Rebuild ?c I64 = Array I64`）を使用 | 仮引数が型変数でないことに関するエラー |
| `test_opaque_equality_formal_param_in_ty_body` | equality の仮引数が関数の型本体にも出現するケース（例：`[Rebuild ?c a = Array a] ... -> a`） | 仮引数が他の場所にも出現することに関するエラー |

### 2-3. opaque type の具体型決定失敗

| テスト名 | 内容 | 期待するエラーメッセージ（部分一致） |
|---------|------|-------------------------------|
| `test_opaque_unused_cannot_determine` | 使用されない opaque type（例：`pi : [?t : ToString] F64`）で `?t` が決定できない | 型が決定できないことに関するエラー |
| `test_opaque_branch_type_mismatch` | `if` 分岐で異なる具体型を返そうとするケース（ユースケース6の非対応ケース） | 型の不一致エラー |

### 2-4. opaque type に関する trait 制約の不満足

| テスト名 | 内容 | 期待するエラーメッセージ（部分一致） |
|---------|------|-------------------------------|
| `test_opaque_trait_not_satisfied_at_use_site` | opaque type が持つ trait 以外のメソッドを呼ぼうとするケース | trait が推論できないことに関するエラー |

---

## 3. LSP テスト

### 3-1. ホバーで opaque type の解決型が表示されること

**テストプロジェクト**: `src/tests/test_lsp/cases/opaque_hover/`

プロジェクト構成：
```
opaque_hover/
  fixproj.toml
  main.fix
```

`main.fix` の内容例：
```fix
module Main;

repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
repeat = |x, n| Iterator::range(0, n).map(|_| x);

main : IO ();
main = (
    let iter = repeat("hello", 3);
    pure()
);
```

テスト内容：
- `?it` にホバーしたとき、解決された具体型（例：`MapIterator (RangeIterator I64) String` 等）が表示されることを確認
- trait member の opaque type にホバーしたときも同様に具体型が表示されることを確認

テスト実装は `src/tests/test_lsp/test_hover.rs`（新規）に配置する。既存の `test_references.rs` の `LspTestCtx` パターンを参考にし、`textDocument/hover` リクエストを送信して応答を検証する。

---

## 4. テスト実装の優先順位

1. **Phase 4 完了後**（型チェックが通る段階）：1-1 のうちコンパイル成功のみ確認するサブセット
2. **Phase 6 完了後**（実行可能）：1-1 ～ 1-5 の全テスト、2-1 ～ 2-4 のバリデーションテスト
3. **エラーメッセージ改善後**：2-1 ～ 2-4 のエラーメッセージ文言の精査・修正
4. **LSP 対応後**：3-1 の LSP テスト
