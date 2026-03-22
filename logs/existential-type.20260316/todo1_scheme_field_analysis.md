# TODO 1：Scheme フィールドの全使用箇所の精査と opaque 分類

## 設計方針（改訂版）

当初は `opaque_predicates`, `opaque_equalities` を別フィールドとして分離する方針だったが、
使用箇所の分析の結果、大半（32箇所）が「both＝opaque/non-opaqueの両方に同じ操作」であるため、
**predicates / equalities は分離せず統一して保持する**方針に変更する。

### 新しい Scheme 構造

```rust
pub struct Scheme {
    pub gen_vars: Vec<Arc<TyVar>>,   // 全称量化された型変数（opaque を含まない、事前計算）
    pub opaque_tys: Vec<Arc<TyVar>>, // 不透明型変数（`?it` 等）
    pub predicates: Vec<Predicate>,  // opaque + non-opaque の制約を混在して保持
    pub equalities: Vec<Equality>,   // opaque + non-opaque の等式を混在して保持
    pub ty: Arc<TypeNode>,
}
```

### ヘルパーメソッド

```rust
impl Scheme {
    /// gen_vars + opaque_tys を返す
    fn all_tyvars(&self) -> impl Iterator<Item = &Arc<TyVar>>;

    /// opaque_tys に関する predicate のみ返す（pred.ty が opaque_tys に含まれるもの）
    fn opaque_predicates(&self) -> Vec<&Predicate>;
    /// opaque_tys に関しない predicate のみ返す
    fn non_opaque_predicates(&self) -> Vec<&Predicate>;

    /// opaque_tys に関する equality のみ返す（equality の第一引数が opaque_tys に含まれるもの）
    fn opaque_equalities(&self) -> Vec<&Equality>;
    /// opaque_tys に関しない equality のみ返す
    fn non_opaque_equalities(&self) -> Vec<&Equality>;
}
```

### 利点

- **predicates / equalities の使用箇所（32箇所の both）が変更不要になる**。resolve_namespace, resolve_type_aliases, set_kinds, find_node_at, global_to_absolute, LSP 関連等すべてそのまま動く。
- opaque / non-opaque の分離が必要な箇所（instantiate_scheme の dual 4箇所、validate_constraints の non-opaque 4箇所）でのみフィルタヘルパーを使う。
- `opaque_tys` はフィールドとして持つ（instantiate_scheme で固定/発行が必要、predicates/equalities から毎回計算するのは冗長）。

## 分類凡例

各使用箇所を以下のいずれかに分類する：

| ラベル | 意味 |
|--------|------|
| **変更不要** | 既存コードがそのまま動く |
| **all_tyvars** | `gen_vars` → `all_tyvars()` ヘルパーに変更 |
| **dual** | opaque と non-opaque で逆の処理をする（Assume/Require の双対性） |
| **special** | 個別の対応が必要 |

---

## gen_vars の使用箇所

### 1. types.rs L1762 — フィールド定義
- **分類**: N/A — `opaque_tys` フィールドを追加する。

### 2. types.rs L1925 — `to_string_normalize`
```rust
for tyvar in &self.gen_vars {
    // gen_vars を a, b, c, ... にリネーム
}
```
- **分類**: **all_tyvars** — opaque_tys も正規化名を与えるべき。ただし opaque_tys は `?a`, `?b`, ... のように `?` プレフィクスをつける必要がある。

### 3. types.rs L1954 — `free_vars_to_vec`（フィルタリング）
```rust
for tv in &free_vars {
    if !self.gen_vars.iter().any(|tv0| tv0.name == tv.name) {
        buf.push(tv.clone());
    }
}
```
- **呼び出し箇所**: typecheck.rs L720, L754 — エラーメッセージ生成時、`create_tyvar_location_messages` に渡す型変数の位置情報を収集するために使用。`create_tyvar_location_messages` は `tyvar_expr` を引くが、opaque 型は `tyvar_expr` に登録しない想定のため、opaque 関連を収集しても参照されない。
- **分類**: **変更不要** — ただし opaque_tys もフィルタ除外する必要がある（`all_tyvars()` で判定）。

### 4. types.rs L1964 — `set_kinds`（kind scope 初期化）
```rust
for tv in &self.gen_vars {
    if tv.kind != kind_star() {
        kind_scope.add_tyvar(tv);
    }
}
```
- **分類**: **all_tyvars** — opaque_tys も kind scope に登録する。

### 5. types.rs L1986 — `set_kinds`（gen_vars の kind 更新）
```rust
for tv in &mut ret.gen_vars {
    *tv = kind_scope.set_tv(tv);
}
```
- **分類**: **all_tyvars** — opaque_tys も同様に更新する。`all_tyvars_mut()` が必要。

### 6. types.rs L2011 — `new_arc`（コンストラクタ）
- **分類**: **special** — コンストラクタに `opaque_tys` パラメータを追加する。

### 7. types.rs L2095 — `global_to_absolute`
```rust
gen_vars: self.gen_vars.clone(),
```
- **分類**: **special** — `opaque_tys` も clone する（Scheme構築部分）。

### 8. typecheck.rs L220 — `substitute_scheme`（不変条件チェック）
```rust
for v in &scm.gen_vars {
    assert!(!self.data.contains_key(&v.name));
}
```
- **分類**: **all_tyvars** — opaque_tys も substitution に含まれていないことを assert する。

### 9. typecheck.rs L232 — `substitute_scheme`（新 Scheme 構築）
```rust
Scheme::new_arc(scm.gen_vars.clone(), preds, eqs, ...)
```
- **分類**: **special** — opaque_tys も clone して引き渡す（コンストラクタ呼び出し）。

### 10. typecheck.rs L550 — `instantiate_scheme` (Require)
```rust
for tv in &scheme.gen_vars {
    let new_tv = self.new_tyvar_by(tv);
    sub.merge(&Substitution::single(&tv.name, type_from_tyvar(new_tv)));
}
```
- **分類**: **dual** — gen_vars は新しい型変数を発行して substitution を作る（現状通り）。
  - **opaque_tys**: `fixed_tyvars` に追加する。ただしリネーム（例：`?it` → `Std::Iterator::repeat::?it`）を施し、その substitution を predicates/equalities に適用する。
  - `opaque_predicates()` → `self.assumed_preds` に追加（仮定として使える）
  - `opaque_equalities()` → `self.assumed_eqs` に追加（仮定として使える）

### 11. typecheck.rs L572 — `instantiate_scheme` (Assume)
```rust
for tv in &scheme.gen_vars {
    self.fixed_tyvars.push(tv.clone());
}
```
- **分類**: **dual** — gen_vars は fixed_tyvars に追加（現状通り）。
  - **opaque_tys**: 新しい型変数を発行する。
  - `opaque_predicates()` → `self.predicates` に追加（証明が必要）
  - `opaque_equalities()` → `self.add_equality` で追加（証明が必要）

### 12. types.rs `generalize` 関数（L2019）
```rust
for pred in &preds { pred.free_vars_to_vec(&mut vars); }
for eq in &eqs { eq.free_vars_to_vec(&mut vars); }
ty.free_vars_to_vec(&mut vars);
Scheme::new_arc(vars, preds, eqs, ty)
```
- **分類**: **special** — generalize は Scheme を構築する関数。収集した free vars のうち `?` 付き変数は `opaque_tys` に、それ以外は `gen_vars` に分離する。

---

## predicates の使用箇所

predicates は opaque/non-opaque を混在して保持するため、大半は **変更不要**。

### 1. types.rs L1764 — フィールド定義
- **分類**: **変更不要**（フィールドはそのまま。opaque を含む predicate も同じフィールドに格納する）。

### 2. types.rs L1777 — `validate_constraints`（predicate が tyvar 上であることのチェック）
```rust
for pred in &self.predicates {
    if !pred.ty.is_tyvar() { return Err(...) }
}
```
- **分類**: **special** — `non_opaque_predicates()` に対してのみ既存バリデーションを適用。`opaque_predicates()` には plan.md で規定された別ルールを適用する。

### 3. types.rs L1792 — `validate_constraints`（trait alias 解決）
- **分類**: **変更不要** — 全 predicates に trait alias 解決を適用。

### 4. types.rs L1881 — `generalize` 内の predicates イテレーション
- **分類**: **変更不要** — generalize は predicates をそのまま受け取る（gen_vars #12 参照）。

### 5. types.rs L1944 — `free_vars_to_vec`
- **分類**: **変更不要** — 全 predicates の free vars を収集する。

### 6. types.rs L1971 — `set_kinds`（kind scope 拡張）
- **分類**: **変更不要** — 全 predicates を渡す。

### 7. types.rs L1973-1975 — `set_kinds`（エラー報告 span）
- **分類**: **変更不要**

### 8. types.rs L1979 — `set_kinds`（predicates の kind 設定）
- **分類**: **変更不要**

### 9. types.rs L1993 — `to_string_substituted`
- **分類**: **変更不要**

### 10. types.rs L2012 — `new_arc`（コンストラクタ）
- **分類**: **変更不要**（predicates パラメータはそのまま）

### 11. types.rs L2053 — `resolve_namespace`
- **分類**: **変更不要**

### 12. types.rs L2065 — `resolve_type_aliases`
- **分類**: **変更不要**

### 13. types.rs L2077 — `find_node_at`（LSP）
- **分類**: **変更不要**

### 14. types.rs L2097 — `global_to_absolute`
- **分類**: **変更不要**

### 15. references.rs L337 — LSP `collect_scheme_assoc_type_refs`
- **分類**: **変更不要**

### 16. references.rs L614 — LSP `collect_scheme_type_refs`
- **分類**: **変更不要**

### 17. references.rs L674 — LSP `collect_scheme_trait_refs`
- **分類**: **変更不要**

### 18. typecheck.rs L223 — `substitute_scheme`
- **分類**: **変更不要**

### 19. typecheck.rs L542 — `instantiate_scheme`（trait alias 解決 + Require/Assume）
```rust
for pred in &scheme.predicates {
    preds.append(&mut pred.resolve_trait_aliases(&self.trait_env.aliases)?);
}
```
- **分類**: **dual** — trait alias 解決は全 predicates に適用だが、解決後の振り分けで分離が必要。
  - non_opaque_predicates: Require → `self.predicates`、Assume → `self.assumed_preds`（現状通り）
  - opaque_predicates: Require → `self.assumed_preds`（仮定）、Assume → `self.predicates`（証明対象）

---

## equalities の使用箇所

equalities も opaque/non-opaque を混在して保持するため、大半は **変更不要**。

### 1. types.rs L1796 — `validate_constraints`（RHS が assoc-ty-free）
```rust
for eq in &self.equalities {
    if !eq.value.is_assoc_ty_free() { return Err(...) }
}
```
- **分類**: **special** — `non_opaque_equalities()` に対してのみ既存バリデーションを適用。`opaque_equalities()` には plan.md で規定された別ルールを適用する。

### 2. types.rs L1864-1870 — `validate_constraints`（重複 LHS チェック）
- **分類**: **変更不要** — 全 equalities に対する重複チェック。opaque / non-opaque 混在でも LHS が同じなら重複なのでそのまま適用可能。

### 3. types.rs L1890 — `new_arc`（コンストラクタ）
- **分類**: **変更不要**

### 4. types.rs L1947 — `free_vars_to_vec`
- **分類**: **変更不要**

### 5. types.rs L1971 — `set_kinds`（kind scope 拡張）
- **分類**: **変更不要**

### 6. types.rs L1982 — `resolve_namespace`
- **分類**: **変更不要**

### 7. types.rs L1996 — `check_kinds`
- **分類**: **変更不要**

### 8. types.rs L2013 — `new_arc`（コンストラクタ、#3 と同じ）
- **分類**: **変更不要**

### 9. types.rs L2056 — `resolve_type_aliases`
- **分類**: **変更不要**

### 10. types.rs L2083 — `find_node_at`（LSP）
- **分類**: **変更不要**

### 11. types.rs L2101-2102 — `global_to_absolute`
- **分類**: **変更不要**

### 12. references.rs L340 — LSP `collect_scheme_assoc_type_refs`
- **分類**: **変更不要**

### 13. references.rs L617 — LSP `collect_scheme_type_refs`
- **分類**: **変更不要**

### 14. typecheck.rs L227 — `substitute_scheme`
- **分類**: **変更不要**

### 15. typecheck.rs L545 — `instantiate_scheme`
```rust
let mut eqs = scheme.equalities.clone();
```
- **分類**: **dual** — 全 equalities をcloneして trait alias 解決後、振り分けで分離が必要。
  - non_opaque_equalities: Require → `self.add_equality`、Assume → `self.assumed_eqs`（現状通り）
  - opaque_equalities: Require → `self.assumed_eqs`（仮定）、Assume → `self.add_equality`（証明対象）

---

## 集計

| 分類 | gen_vars | predicates | equalities | 合計 |
|------|----------|------------|------------|------|
| 変更不要 | 1 | 16 | 13 | **30** |
| all_tyvars | 5 | — | — | **5** |
| dual | 2 | 1 | 1 | **4** |
| special | 4 | 2 | 1 | **7** |
| N/A (定義) | 1 | — | — | **1** |

## 主な所見

### 1. predicates / equalities の使用箇所は大半が変更不要
統一方式の最大の利点。resolve_namespace, resolve_type_aliases, set_kinds, find_node_at, global_to_absolute, LSP系, substitute_scheme, free_vars_to_vec 等すべてそのまま動く。

### 2. gen_vars の使用箇所は `all_tyvars()` ヘルパーで対応
set_kinds, substitute_scheme の assert, to_string_normalize 等で gen_vars + opaque_tys の両方を見る必要があるが、`all_tyvars()` / `all_tyvars_mut()` で対応。

### 3. "dual" は instantiate_scheme に集中（4箇所）
`instantiate_scheme` で `opaque_predicates()` / `opaque_equalities()` フィルタヘルパーを使い、opaque と non-opaque で逆の振り分けを行う。

### 4. validate_constraints は special（3箇所）
`non_opaque_predicates()` / `non_opaque_equalities()` に既存バリデーション、opaque 用に新バリデーションを追加。重複 LHS チェックは変更不要。

### 5. generalize 関数は special
free vars から `?` 付きを opaque_tys に、それ以外を gen_vars に分離する処理を追加。
