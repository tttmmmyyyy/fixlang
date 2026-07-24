# 引き継ぎ手順書 (LSP rename symbol 実装、レビュー中)

最終更新: 2026-04-26

## 1 行サマリー

LSP `textDocument/rename` の実装は機能完成 + 全テスト緑。**ユーザーがレビュー中**で、9 個の commit を `git reset HEAD~9` で巻き戻して全変更を作業ツリー差分にした状態。レビューが進むと `git add` していき、最終的に 1 個の commit にまとめる方針。

## 現状 (前セッション最終時点)

### Git 状態

- ブランチ: `lsp-rename`
- HEAD: `c5793528` (= "Include import-statement leaves in find-references" = Phase A1)
- `origin/lsp-rename` は HEAD より 9 commit 進んでいる (= reset 前の状態)。push しないこと、pull しないこと
- 元の 9 commit は reflog (`git reflog`) と origin から復元可能。コミット hash は §3 参照

### Staged / Unstaged 内訳

ユーザーがレビュー進捗に合わせて段階的に `git add` している。`git status` で最新を確認。

### 元の 9 commit (新→古)

```
5481b403 Trim parenthetical from external-symbol rename rejection message
b48ad18d Fix two rename bugs found during smoke testing
5fd7da4a Rename struct/union types with auto-namespace coupling (Phase D)
045ecaa9 Gate rename: stale-buffer / external / auto-method rejection
b3e3d976 Extend rename to types, traits, fields, and union variants
65ee5bf8 Implement rename for local variables and global values
28164122 Track field/variant name spans in MakeStruct and patterns
052993c6 Include auto-method call sites in field/variant references
700faac6 Detect cursor on struct fields and union variants
```

参照したいときは `git show <hash>` (オブジェクト DB に残っているので reflog 期限が来るまで参照可能)。

## 2. 設計ドキュメント

[plan.md](./plan.md) に Phase A〜D の全体設計が残っている。レビュー中に「なぜこうしたか」を聞かれたら基本的にここを参照すれば書いてある。

主要セクション:
- §3.1〜3.6: 設計判断の核 (import を refs に含める / 非ユーザー拒否 / 自動実装メソッド連動 / 型 rename と auto-namespace 連動 など)
- §4.x: 実装詳細
- §5: フェーズ分割
- §6: テスト計画

## 3. レビューで進行中の修正方針

ユーザーは差分を読みながら気づいた点を指摘してくる。これまでの傾向:

1. **コメントの追加要求** (例: validate_variant_name が値を返す理由)
2. **コードの clean-up** (例: clone + rebuild → clone + iter_mut パターン)
3. **データ構造の整理** (例: user_source_files と source_contents の統合)
4. **エラーメッセージの調整** (例: 不要な parenthetical の削除)

修正は「指摘 → 直す → ビルド + テスト確認 → ユーザーが add」という流れ。**勝手に commit しない** こと。指示があってから commit する。

### 直近行ったレビュー対応 (作業ツリーに既に反映済み)

- `validate_variant_name` の docstring 拡充 (なぜ Result<Arc<PatternNode>, Errors> を返すかを説明)
- `pattern.rs` の 3 箇所で `Vec<...>::iter().map().collect()` パターンを `clone() + iter_mut()` に統一 (`resolve_namespace`, `resolve_type_aliases`, `global_to_absolute` の Pattern::Struct アーム)
- `Program` の `user_source_files` + `source_contents` を `user_source_contents: Map<PathBuf, String>` に統合 ([server.rs:1050-1063](../../src/commands/lsp/server.rs)、[rename.rs](../../src/commands/lsp/rename.rs)、テスト docstring 更新)
- prepareRename の reject ケース (ステイル / 外部 / auto-method) を null から `ResponseError` に変更 (VS Code が generic な "this element can't be renamed" ではなく実メッセージを表示するように)
- 外部シンボル拒否メッセージから `(e.g. Std or a dependency)` parenthetical を削除

### スモークテストで見つかったバグ (作業ツリーに既に反映済み、回帰テスト追加済み)

- `walk_expr_for_inline_qualified` で `compiler_defined_method` チェックを忘れていたバグ。`namespace MinCostFlowGraph { create : ... }` のような user 定義 helper への qualified 参照が型 rename 時に書き換えられていた。修正済み + `test_rename_struct_type_skips_user_helper_qualified_call` で再現テスト

## 4. 完了条件

ユーザーがレビューを完了したら:

```bash
# 残ってる未 stage を全部 add (untracked 含む)
git add src/

# 全テストが通ることを確認
cargo test --release          # 全 700+ テスト

# 単一の commit にまとめる
git commit -m "Add LSP textDocument/rename and supporting infrastructure"
```

commit message 本文には Phase A〜D の構成を箇条書きで残すと良い (元の 9 commit のメッセージから引いてくる)。

push する/しないはユーザーに確認。

## 5. テスト実行のクセ

- LSP 統合テストは `fix` バイナリを spawn するので、コード変更後は `cargo install --path . --force --offline` で `~/.cargo/bin/fix` を更新してから `cargo test --release` する必要がある
- `install_fix()` は process-once なので、テスト前に手動 install を忘れないこと
- 全 LSP テスト (97-103 件): `cargo test --release test_lsp` で約 2-3 分
- 全テスト: 約 7-8 分

## 6. 既知の制限 (ドキュメント済み)

`plan.md` の §3.6 に書いてある通りの制限が rename 機能にある:

1. **mixed import の rebuild でフォーマット/コメントが失われる** (`stringify()` 経由のため)。テスト `test_rename_struct_type_mixed_import_split` で挙動確認
2. **インライン qualified `Point::@x` は最後の namespace_item だけ rewrite** する。多層的に同名の namespace path が出るエキゾチックなケース (`Foo::Point::sub::Point` のような) では完璧でない可能性。実用上ほぼ問題なし

## 7. ファイル変更マップ (レビューの参考)

| ファイル | 役割 |
|---|---|
| `src/ast/program.rs` | `EndNode::Field`/`Variant` 追加、`user_source_contents` 追加、`GlobalValue::find_node_at` で `syn_scm` 優先 |
| `src/ast/typedecl.rs` | `Field.name_src` 追加、`TypeDefn::find_node_at` で field/variant 名を検出 |
| `src/ast/expr.rs` | `Expr::MakeStruct` のフィールド triple 化 (`Name, Option<Span>, ExprNode`)、`expr_make_struct_with_spans` 追加 |
| `src/ast/pattern.rs` | `Pattern::Struct/Union` の triple 化、`make_struct_with_spans`/`make_union_with_span` 追加、find_node_at 拡張 |
| `src/ast/traverse.rs` | (Phase B3 の AST shape 変更追従、`name_src` を新フィールドに通す) |
| `src/ast/collect_annotation_tyvars.rs` | (B3 追従) |
| `src/parse/parser.rs` | `validate_token_str`, `parse_namespace_items_in_fullname`, `TokenCategory` 追加。`type_field`/`pattern_struct`/`pattern_union`/`expr_make_struct` で名前 span 保存 |
| `src/commands/lsp/mod.rs` | `pub mod rename;` 追加 |
| `src/commands/lsp/server.rs` | rename capability、dispatch、`run_diagnostics` で `user_source_contents` populate、`LatestContent.path` を pub 化 |
| `src/commands/lsp/references.rs` | import 文中の参照を refs に含める、Field/Variant 用 collector (`find_member_occurrences` 系)、syn_scm 優先 |
| `src/commands/lsp/util.rs` | `find_member_def_src` 追加、hover で Field/Variant 対応 |
| `src/commands/lsp/goto_definition.rs` | Field/Variant 配線 |
| `src/commands/lsp/completion.rs` | Field/Variant の import 補助無し対応 |
| `src/commands/lsp/rename.rs` | **新規** ファイル。本実装の核。約 970 行 |
| `src/elaboration/desugar_opaque.rs` | (Pattern triple 化追従) |
| `src/elaboration/typecheck.rs` | (Pattern/MakeStruct triple 化追従) |
| `src/generator.rs` | (Pattern triple 化追従) |
| `src/optimization/*.rs` | (Pattern/MakeStruct triple 化追従、let_elimination/remove_hktvs/uncurry/unwrap_newtype) |
| `src/tests/test_lsp/test_references.rs` | imports refs / Field/Variant refs テスト追加 |
| `src/tests/test_lsp/test_rename.rs` | **新規** ファイル。36 ケース |
| `src/tests/test_lsp/mod.rs` | `pub mod test_rename;` 追加 |
| `src/tests/test_lsp/cases/*` | 6 個の新 fixture (`refs_field_variant`, `rename_basic`, `rename_types`, `rename_struct_type`, `rename_mixed_import`, `rename_user_helper_qualified`) |

## 8. 急いで再確認したい時のコマンド

```bash
# 現状把握
git status
git diff --stat
git log --oneline -5
git reflog | head -15  # 元の 9 commit が見える

# テスト
cargo install --path . --force --offline
cargo test --release test_rename       # rename テスト 36 件
cargo test --release test_lsp          # LSP 全体 ~103 件
cargo test --release                   # 全部 ~700 件

# 元の 9 commit 復元 (作業ツリーが消えるので要注意)
git reset --hard ORIG_HEAD             # ここまでの編集が消える
# または特定 commit だけ取り戻したいとき
git cherry-pick <hash>
```

## 9. 引き継ぎ完了後にやること

新セッションでこの handoff を読んだら、まず:

1. `git status` で現状確認
2. ユーザーに「レビューどこまで進んでますか」「次の指摘どうぞ」と聞く
3. 指摘が来たら修正 → ビルド/テスト確認
4. ユーザーが「add しといて」と言ったら add (commit はしない)
5. 「commit して」と明示的に言われたら commit

ユーザーは `/home/maruyama/fixlang/lsp-rename` を `cp-library` の隣で開いていて、実機 (VS Code 等) でも rename を試している。スモークテストで新しいバグを見つけたら、まず最小再現 fixture を作って回帰テストにしてから直す方針 (cp-library のバグの時にこの方針で進めた)。
