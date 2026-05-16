# Plan: error_tolerant typecheck の本質改修

## Context

Fix の LSP は dot completion のために、ユーザの live buffer を `error_tolerant = true` で再エラボレート ([commands/lsp/completion/mod.rs:362-394](src/commands/lsp/completion/mod.rs#L362-L394) の `run_completion_elaborate`)。狙いは、コードに型エラーがあっても**カーソル位置周辺の型情報**を引き出して dot ranking に使うこと。

しかし現状の `error_tolerant` fallback ([typecheck.rs:721-740](src/elaboration/typecheck.rs#L721-L740)) は:

```rust
Err(errs) if self.error_tolerant => {
    let _ = errs;
    Ok(ei.set_type(ty_for_fallback))  // ← 元の AST に外側の型だけ
}
```

戻り値の子ノードは未型付け（`set_match_cond` / `set_app_args` 等の差し替えが走る前に `?` で巻き戻されたため）。結果として:

1. **`fix_types` (typecheck.rs:1921) の `expr.type_.as_ref().unwrap()` で panic** — LSP プロセス死、補完リクエストが返らずクラッシュに見える。これがユーザ報告の症状。
2. **典型的な「サブ式は型付けできるはず」のケースで型情報が失われる**。`match it { some(a) => (a, a).<cursor> }` のような非網羅 match の中で `(a, a)` のタプル型すら抽出できず、dot completion が `Tuple2::@0/@1` を Tier 0 に上げられない。

LSP 側の応急処置（コミット `36f0d983`）で別の silent-drop バグは閉じたが、本 panic は残っている。`error_tolerant` 経路の本質を直す。

**指針**: error_tolerant typecheck は **completion 専用** の「ベストエフォート型情報抽出」モードと割り切る。

- 全サブ式に型を付ける（fixed である必要はない）
- 構造的に推論可能な情報（`[1,2,3]` → `Array I64`、`if cond ...` の cond は `Bool`、`(a, b)` → `(_, _)`）は積極的に保つ
- 制約矛盾・未解決 tyvar・predicate 未充足・equality 未充足はすべて許容
- 出力されるダイアグノスティクスは error_tolerant モードでは原則無視（receiver 型さえ得られれば十分）

## 改修方針

### 1. per-case で「validation/unify 失敗で子を捨てない」化

[typecheck.rs:742-1244](src/elaboration/typecheck.rs#L742-L1244) の `unify_type_of_expr_inner` を case ごとに修正。各ケースで `error_tolerant=false` の場合は従来通り早期 `Err`、`true` の場合は「失敗を吞んで構造的に分かる範囲で子を型付け」する。

| Case | 行 | 失敗点 | error_tolerant での扱い |
|---|---|---|---|
| `Var` | 749-911 | overload 候補なし or 全候補 fail | 期待型 `ty` をそのまま node に貼って返す（子なし） |
| `Lam` | 935-957 | 948行: `unify(ty, fun_ty)` 失敗 | body を fresh tyvar 期待型で型付け、`set_lam_body(body)` |
| `Let` | 958-971 | 959行 `validate_pattern?` / 960行 `pat.get_typed?` 失敗 | pat に fresh tyvar 型 + 空 var_ty、val/body も fresh tyvar 期待で型付け |
| `Match` | 973-1067 | 1058行 `validate_match_cases_exhaustiveness?` 失敗 | typed 木を組み立てた**後**に check を移動、失敗を吞む |
| `Match` の arm 内部 | 996-1040 | 各 validation/unify 失敗 | pat に fresh tyvar、val は fresh tyvar 期待で続行 |
| `TyAnno` | 1077-1085 | 1079行 unify 失敗 | child expr を fresh tyvar 期待で型付け |
| `MakeStruct` | 1086-1183 | 1089-1161行 の structural validation / unify | 各 field expr を fresh tyvar 期待で型付け |
| `ArrayLit` | 1184-1206 | 1196行 unify 失敗 | 各 elem を fresh tyvar 期待で型付け |
| `FFICall` | 1207-1237 | 1214行 unify 失敗 | 各 arg を fresh tyvar 期待で型付け |
| `App` `If` `Eval` | 919-933, 1068-1076, 1238-1244 | なし（既に全子を `unify_type_of_expr` 経由で型付け） | 無変更 |

**`Var` の方針**: leaf なので fresh tyvar は与えず、期待型 `ty` をそのまま貼る。

**ヘルパー追加**: 「failure を吞んで fresh tyvar に置換」用のプライベートヘルパーを 1–2 個追加。例:

```rust
fn fresh_ty_with_src(&mut self, src: &Option<Span>) -> Arc<TypeNode> {
    let tv = self.new_tyvar_star();
    self.add_tyvar_source(tv.name.clone(), src.clone());
    type_from_tyvar(tv)
}
```

### 2. `Pattern::get_typed` を error_tolerant 化

[ast/pattern.rs:110-200](src/ast/pattern.rs#L110-L200) の `get_typed` を修正。失敗パスは:
- Struct: sub-pattern と field 型の unify 失敗 (148-162行)
- Union: sub-pattern と variant 型の unify 失敗 (180-194行)
- Union: `validate_variant_name` 失敗 (関連メソッド)

これらの unify は**既に sub-pattern を typed にした後**に走る。`typechcker.error_tolerant` を見て、失敗時に sub-pattern の typed 木はそのまま保ち、自身の type は強制せず（あるいは fresh tyvar）、`var_to_ty` も sub-pattern の分は保持して `Ok` で返す。

Pattern 側を直接直すと Match/Let の call site は無変更で済み、call site での hack（fake typed pat を作る）が不要になる。これは「逃げ」ではなく **責務の正しい配置**。

### 3. `fix_types` を error_tolerant 化（panic 根絶の核）

[typecheck.rs:1921-1983](src/elaboration/typecheck.rs#L1921-L1983) の `fix_types` と [typecheck.rs:1859-1883](src/elaboration/typecheck.rs#L1859-L1883) の `fix_types_for_pattern` が 2 種類のエラーで死にうる:

- `expr.type_.as_ref().unwrap()` (現在: panic) — Section 1 で全 node が型付けされる前提が立つので、追加ガード不要。`debug_assert!` のみ
- `self.substitute_and_reduce_type(ty)?` (現在: `?` で hard error) — associated-type reduction の失敗。`error_tolerant` なら**un-substitute/un-reduced の `ty` をそのまま使って続行**

### 4. `check_type` の検証 Layer を `error_tolerant` で全スキップ

[typecheck.rs:1430-1545](src/elaboration/typecheck.rs#L1430-L1545) の `check_type` の cascade を error_tolerant で全部スキップ:

- **Layer 1 (holes, 1500-1504行)**: スキップ（hole エラーも tolerant では報告不要、receiver 型抽出が目的）
- **Layer 2 (`check_types_are_fixed`, 1506-1509行)**: 代わりに新規 `check_all_typed` (全 node に `type_.is_some()` を assert) を呼ぶ。これが Section 1 の不変条件の常時テストになる
- **Layer 3 (`reduce_predicates`, 1515-1519行 + `self.predicates.len() > 0`)**: スキップ
- **Layer 4 (equalities, 1521行以降)**: スキップ

```rust
let hole_errors = check_holes::collect_hole_errors(&expr, self);
if !self.error_tolerant && hole_errors.has_diagnostics() {
    return Ok((expr, hole_errors));
}
if self.error_tolerant {
    // 不変条件: error_tolerant 経路は全 node に型が付くことを保証
    self.check_all_typed(&expr)?;
    return Ok((expr, Errors::empty()));
}
// 以下、従来の Layer 2-4 ...
```

**`check_all_typed`** は新規関数。`check_types_are_fixed` ([typecheck.rs:2008](src/elaboration/typecheck.rs#L2008)) と同じ AST walk 構造で、各 node に対して `expr.type_.is_some()`、各 pattern に対して `pat.info.type_.is_some()` だけを assert（`check_is_type_fixed` の代わり）。失敗は単純な internal error として `Errors` で返す（panic ではなく Err、bug 検出用）。

これにより:
- **既存の全 completion テストが Section 1 の不変条件 (全 node 型付け) のテストを兼ねる** → 専用 unit test 不要
- error_tolerant モードでは無関係なダイアグノスティクスが出ない（completion の品質に直結）

### 5. リグレッションテスト

#### (a) LSP 統合テスト: 最小化したフィクスチャ

`src/tests/test_lsp/cases/completion-dot-after-tuple/main.fix`:

```fix
module Main;

main : IO () = ( pure() );

f : Std::Option I64 -> (I64, I64);
f = |opt| match opt {
    some(a) => (a, a).
};
```

問題の核（非網羅 match + dot at tuple receiver）だけを残し、`Iterator` トレイト制約・`generate`・ネスト match などユーザ snippet の周辺ノイズはすべて省略。

`src/tests/test_lsp/test_completion.rs` に追加するテスト:

1. **`test_completion_dot_after_tuple_no_crash`**: cursor が `(a, a).` 直後の position で completion request に LSP が応答する。
2. **`test_completion_dot_after_tuple_infers_tuple_receiver`**: 同じ位置で:
   - `Std::Tuple2::@0` (タプル系) が Tier 0 (`sortText.starts_with("0")`)
   - `Std::Iterator::fold` (非タプル系) が Tier 0 で**ない**（fresh tyvar で誤魔化された場合との区別）

#### (b) 他の case の dot completion 確認（任意、Phase 1 各 case の効果を個別検証）

優先度低。(a) で本筋のバグは閉じる。余裕があれば:
- `MakeStruct`: `Point { x: foo().<cursor>, y: 0 }` で `foo` 未定義でも cursor の receiver type が推論される
- `ArrayLit`: 型不一致な配列の中での dot
- `Lam` 型不一致: `(|x: Bool| x.<cursor>)(42)` で body 内 cursor が `Bool` 情報を得る

これらは Section 1 の各 case 修正の効果を個別に保証する。

## Critical Files

- `/home/maruyama/fixlang/completion/src/elaboration/typecheck.rs` — Section 1, 3, 4 のメイン
- `/home/maruyama/fixlang/completion/src/ast/pattern.rs` — Section 2
- `/home/maruyama/fixlang/completion/src/tests/test_lsp/test_completion.rs` — Section 5(a)
- `/home/maruyama/fixlang/completion/src/tests/test_lsp/cases/completion-dot-after-tuple/` — Section 5(a) 新規

## 既存資産の再利用

- `new_tyvar_star()` / `add_tyvar_source()` ([typecheck.rs:467-507](src/elaboration/typecheck.rs#L467-L507)) — fresh tyvar 生成
- `type_from_tyvar()` — TyVar → TypeNode
- 各 `set_*` setter ([ast/expr.rs:101 ほか](src/ast/expr.rs#L101)) — typed 木の組み立て
- `UnifOrOtherErr::extract_others` — unify エラーの recovery 可能部分の分離（既存パターン）
- `check_types_are_fixed` の AST walk 構造 ([typecheck.rs:2008-2084](src/elaboration/typecheck.rs#L2008-L2084)) — `check_all_typed` のテンプレート
- `LspCompletionCtx::setup` / `complete` / `find_sort_text` ([test_lsp/test_completion.rs:43-133](src/tests/test_lsp/test_completion.rs#L43-L133)) — LSP テスト infrastructure

## 検証

### ビルド & テスト
1. `cargo build --release` でビルド通過
2. `cp target/release/fix ~/.cargo/bin/fix` で `fix` バイナリ更新
3. `cargo test --release test_lsp` で全 131 件 + 新規分が通過。Section 4 の `check_all_typed` が機能していれば、もし Section 1 で漏れがあれば既存テスト経由で検出される
4. 特に既存の `test_completion_dot_sort_*` 系（特に `_error_tolerant_inside_if_body`）に回帰がないこと

### 手動再現確認
- ユーザ報告の元 snippet（フィクスチャ最小化前の original）で LSP がハングせず completion response が返る
- VSCode 上で `(a, a).` の補完候補上位にタプル系メソッド（`@0`, `@1` 等）が来る

### コミット粒度（推奨）
1. Section 1 (`unify_type_of_expr_inner` 各 case 修正) + Section 2 (`Pattern::get_typed` 修正)
2. Section 3 (`fix_types` の `substitute_and_reduce_type` フォールバック)
3. Section 4 (`check_type` の Layer 全スキップ + `check_all_typed` 新規)
4. Section 5(a) (regression test: 最小化フィクスチャ + 2 つのテスト)
5. (任意) Section 5(b)

コミット 1〜3 はそれぞれビルド + 既存 LSP テスト通過を確認しながら積む。最終 4 が落ちたら逆順で原因を切り分けられる。
