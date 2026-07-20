# plan と実装の突き合わせ監査（2026-07-20）

- 対象ブランチ: `unique-check-elim`、tip `2ea43905`
- 対象 plan: `dev-docs/2026-06-28-unique-check-elim/plan.md`
- 方法: plan §1-§10 を通読し、各具体項目を実コード（`src/rc_ir/*`, `src/fixstd/*`, `src/object.rs`, `src/ast/*`, テスト）と突き合わせ。「実装済み / 逸脱 / 未実装（plan 認容の延期か真のギャップか）」で分類。

## 結論

P0.5-P3 は概ね plan どおり実装済み。健全性に関わる真のバグは無い。残差は plan ドキュメントの陳腐化・良性の未記録逸脱・plan 認容の延期・テスト網の粗である。

## Tier 1: 健全性に関わると疑った項目 -> 検証の結果「健全（plan テキストの陳腐化）」

### `Retain` の root ベース降格が未実装（plan §3.2）だが、健全性は別機構で成立

- plan §3.2 は「`Retain(y)` は `root(y@π)` を共有する全生存変数の末端を Dyn 化（root ベース）」を要求し、誤コンパイル例を挙げる: `let y=t.@0; Retain(y)` を変数キーだけで降格すると `env[t]@[0]` が `Fresh` のまま残り、再取り出し `a=t.@0` が unique に化けて §4 が誤 elide -> 共有配列を in-place 破壊（unboxed aggregate の getter 限定。boxed 容器 getter は元々 Dyn）。
- 実コード（`provenance.rs::interp_inner` の `Retain` arm）は `p.demote(path)` で retained 変数自身の leaf のみ降格 = 変数キーのみ。module doc も root ベースは未達と自認。
- **検証（テストを書いて実行、修正なし）**: unboxed tuple / struct のフィールドを二重射影して alias を作り、片方を変異・他方を読む形の候補を 9 本作成し、`-O none`（参照）/`basic`/`max`/`experimental`（+valgrind）で出力比較。**全候補で全 opt 一致、miscompile 再現せず**。
- **機構（C1 の RC IR dump で確定）**: unboxed aggregate の leaf を複数回射影すると、RC 挿入は容器（= root）への明示 `retain struct#N.π` を出す。その変数キー降格が root の leaf `env[struct#N]@π` を降格し、以降の射影 `@π(struct#N)` は `[dyn]` と解決 -> `set` はチェック保持（`Array::set` のまま = 非 elide） -> clone -> 安全。plan が想定した「alias 側 `Retain(y)`」は実際には生じず、Retain は常に root に載る。よって「変数キー降格 = この場合の root ベース降格」。
- **既存テスト**: `test_basic.rs::test_unique_check_elim_reprojected_alias_shared` が正にこの reprojected-alias ケースを memcheck 付きでカバー済み（コメントが container-retain 機構を明記）。
- **結論**: 健全性ギャップではない。plan §3.2 のテキストが「root ベース降格が必要」と記すのに対し、コードは container-retain 機構で同じ健全性を達成しており、**plan テキストが陳腐化**している。対処するなら plan §3.2 に container-retain 機構の記述を足す（コード修正は不要）。

## Tier 2: 真のギャップ（plan が今フェーズで期待、未実装 / 未記録逸脱）

- **RC IR validator（[#14], §9.0/P1）が無い**: `validate(RcProgram)` 相当のデバッグ専用 well-formedness チェッカ（名前一意性・use-after-consume・root ごとの参照バランス）が `src/rc_ir/` に存在しない。plan は各 RC 変換後に走らせる「過剰 retain リークの網」と位置づけていた。
  - **更新（2026-07-20 同日）: 部分実装済み。** `src/rc_ir/validate.rs::validate` を追加し、`config.develop_mode` gate で `optimize_rc_program` の各パス直後に実行（通常ビルドでは走らない）。検査は **(i) 束縛名の関数内一意 + use-in-scope** まで。**(ii) use-after-consume・(iii) 参照収支・(iv) closure 捕捉順は未実装**（ownership/consume モデルを要し false-positive 検証が重いため follow-up）。全 opt suite で false-positive ゼロ + validator の unit テスト（malformation 検出）を確認済み。
- **uniqueness assert ビルドが無い（valgrind で代替）**: unique 判定値が実行時に共有だった時に abort するモードは未実装（`set_sanitize_memory` はコメントアウト）。`develop_mode` の valgrind MemCheck + 共有値テストで実質代替。plan の明示的な網は未構築で未記録。
- **`InlineLLVMArrayPunchBody` に `result_prov` が無く既定の全 `Dyn`（未記録逸脱）**: plan §3.3 は punch の内側 array 末端 = `Boxed({Fresh})` と宣言するが、実装は override 無しで既定 Dyn。std の全 flow（mod/act は非 fu plug = チェック無し、汎用 act の fu-plug は map 共有で必ずチェック残留）では elision に影響せず健全・実質無害。plan 表との乖離が未記録なだけ。
- **`pop_back` が force-unique 除去対象になっていない（軽微）**: `InlineLLVMArrayPopBackNonemptyBody` は `make_array_unique` 無条件、`unique_check_operand`/`assuming_unique` 無し。P0.7 の「各 atomic op に force_unique を付け §4 除去対象に揃える」目標から漏れ。

## Tier 3: plan ドキュメントの陳腐化（コードが正・plan 未更新）

**更新（2026-07-20 同日）: 下記 3 件はいずれも plan の該当箇所に `[実装メモ 2026-07-20]` を追記して解消済み**（§1.2 の enum 直後 / §3.1 / §4.1、および §8 の unbox getter パリティ項目）。

- **`RcExpr::Destructure` ノードが存在するが §1.2 は「getter 列に lower」と記す**（§1.2 enum・§1 の destructure 記述・impl-notes が stale。コードは専用ノードで解決済み。§8 の「unbox getter パリティ未解決・要解消」も解決済み）。
- **provenance 表現が leaf-map にリファクタ**（§3.1 は旧 enum `Boxed/UnboxedAgg/Unboxed` を図示、コードは `Provenance = Map<Path, LeafSource>` / `Uniqueness = BTreeMap<Path, CTRefCnt>`。挙動等価・単体テスト有り）。
- **α-merge（構造ハッシュ併合）が LLVM backend（MergeFunctions/ICF）へ委譲**され RC IR 上には無い（§4.1/§10/§7-P3 が RC IR 併合を chosen として記述 = stale）。

## Tier 4: 良性の未記録逸脱（挙動保存）

- `infer_ownership` が SCC condensation の bottom-up でなくフラットな大域不動点ループ（plan §2.1 は SCC を指示）。単調降格で最小不動点は同一 = 良性。
- `tail_of` 相当が `codegen.rs::is_tail_cont` と `borrow.rs::trivially_returns` に二重化（plan は単一共有を意図）。ロジック同一だが drift リスク。

## Tier 5: plan 認容の延期（未実装で正常）

- **P3.5**（`*_uniqueness_unchecked` 掃除）未着手 -> `_unsafe_set_bounds_uniqueness_unchecked_unreleased` は未初期化 fill 用途で残存（append/push_back/map/reserve/resize）。
- **`_unsafe_get_linear_bounds_unchecked_unretained` の物理削除**は deviation A どおり P3/P3.5 へ延期（外部 cp-library が使用）。両プリミティブとも存在を確認。
- **P4**（reuse / reorder / bounds-check-elim = BCE）は別ブランチ `bce`。§6 の将来 RC 最適化群（reuse / move-out / reordering / borrow-closure calling convention / state 推論 / case-of-known-constructor）も未着手で正常。

## Tier 6: 未追加の計画テスト（plan が §9 で列挙）

- **マルチスレッド [#F4]「is_unique => LOCAL」E2E テストが無い**（plan は必須と明記）。単一スレッド入力では誤 elide を捕捉できないが、branch にスレッド API が無く実装困難な可能性。
- P1 の tail-call 専用テスト、P0.5 の `not` branchless-IR アサート、§9.5/§9.6 の多数の名前付き adversarial テストが dedicated には無く、広域 valgrind soundness テスト群（`test_punched_array` / `test_union_match` / `test_struct_destructure` / `test_match_return_outer` / `test_shared_boxed_swap` 等、opt >= Max）で代替。

## 対処優先度（提案・未実施）

1. plan §3.2 に container-retain 機構の記述を追記（Tier 1、コード修正不要）。
2. RC IR validator の追加（Tier 2、デバッグ時の健全性網）。**-> (i)+use-in-scope は 2026-07-20 実装済み。残る (ii)/(iii)/(iv) が follow-up。**
3. punch の `result_prov` 明示化 or plan §3.3 表の更新（Tier 2、精度/整合）。
4. plan の陳腐化箇所（§1.2 Destructure / §3.1 leaf-map / §4.1 α-merge 委譲）を実装に合わせて更新（Tier 3）。**-> 2026-07-20 実装メモとして追記済み。**
