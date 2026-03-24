# Opaque Type 実装計画（詳細）

本書は plan2.md の設計に基づき、Fix コンパイラへの具体的なコード変更を記述する。

---

## 全体の流れ

```
Phase 0: パース            → grammer.pest, parser.rs
Phase 1: 型コンストラクタ生成  → desugar_opaque.rs (新規), program.rs (TypeEnv更新)
Phase 2: 制約変換           → desugar_opaque.rs, traits.rs (TraitEnv拡張)
Phase 3: 署名・式の書き換え   → desugar_opaque.rs, program.rs
Phase 4: 型チェック          → 変更なし（既存がそのまま動く）
Phase 5: 具体型の決定        → typecheck.rs, program.rs
Phase 6: インスタンス化時の解消 → program.rs (instantiate_symbol)
追加: エラーメッセージの検証   → Phase 6完了後
追加: LSP対応               → lsp/
```

elaborate() 内の挿入位置：

```rust
// src/elaboration/mod.rs の elaborate() 内

// ... 既存ステップ (validate_trait_env, create_trait_member_symbols,
//     validate_global_value_type_constraints, validate_import_statements,
//     set_kinds) はそのまま ...

program.set_kinds()?;                    // 既存 step 15

// If typechecking is not needed, return here.  // 既存 step 16
if !config.subcommand.typecheck() {
    assert!(!config.subcommand.build_binary());
    return Ok(program);
}

// ★ NEW: Opaque type desugaring (Phase 1-3)
program.desugar_opaque_types()?;

let typechecker = program.create_typechecker(config);  // 既存 step 17
```

---

## Phase 0: パース

### 0-1. 文法の変更

**ファイル**: `src/parse/grammer.pest`

```diff
- type_var = { !keywords ~ tyvar_head ~ tyvar_char* }
+ type_var = { !keywords ~ "?"? ~ tyvar_head ~ tyvar_char* }
```

これにより `?it` が `type_var` としてパースされる。`pair.as_str()` で `"?it"` が得られる（`?` を含む文字列）。

`type_var` は `type_nlr` の一部であり、`type_expr` → `type_fun` → `type_tyapp` → `type_nlr` → `type_var` と解決される。制約ブロック内の `predicate` (`type_expr : trait_fullname`) や `equality` (`type_expr = type_expr`) でも自然に使える。

**影響範囲**：`type_var` は型変数のパースに使われるすべての場所に影響するが、`?` は既存コードで使われていないため副作用なし。

### 0-2. パーサの変更

**ファイル**: `src/parse/parser.rs`

`parse_type_var()` (L2480付近) は `pair.as_str()` をそのまま TyVar の name として使う。`?it` は `name = "?it"` として格納される。変更不要。

パーサ自体には検証を追加しない。opaque 型変数の使用箇所制限は V-1 参照。

### 0-3. `Scheme::generalize` の変更

**ファイル**: `src/ast/types.rs`

`generalize` で free vars を収集した後、`gen_vars` に入れる前に以下を除外する：

1. **opaque 型変数**：名前が `?` で始まる TyVar を `gen_vars` から除外する。
2. **equality 仮引数**：opaque 型に関する equality 内でのみ出現し、型本体（`ty`）や非 opaque predicate には出現しない TyVar を除外する（例：`Rebuild ?c b = Array b` の `b`）。

これにより、`gen_vars` は generalize の時点から正しい状態になる。後続の `validate_type_defns`、`validate_trait_env`、`set_kinds` への悪影響がないことを確認済み：

- **`validate_type_defns`**: `Scheme.gen_vars` を使用しない（`TypeDefn.tyvars` のみ検証）。
- **`validate_trait_env`**: `Scheme.gen_vars` を直接参照しない（型と predicates を直接走査）。
- **`set_kinds`**: gen_vars だけでなく全 free type variable を扱うように修正する（Phase 0-4 参照）。これにより、opaque TyVar や equality 仮引数が gen_vars になくてもカインドの初期化・反映が正しく行われる。

```rust
pub fn generalize(
    kind_signs: &[KindSignature],
    preds: Vec<Predicate>,
    eqs: Vec<Equality>,
    ty: Arc<TypeNode>,
) -> Arc<Scheme> {
    let mut vars = vec![];
    for pred in &preds {
        pred.free_vars_to_vec(&mut vars);
    }
    for eq in &eqs {
        eq.free_vars_to_vec(&mut vars);
    }
    ty.free_vars_to_vec(&mut vars);
    for tv in &mut vars {
        for kind_sign in kind_signs {
            if tv.name == kind_sign.tyvar {
                *tv = tv.set_kind(kind_sign.kind.clone());
            }
        }
    }
    // ★ NEW: opaque 型変数と equality 仮引数を gen_vars から除外
    let opaque_eq_only_vars = collect_opaque_equality_formal_params(&preds, &eqs, &ty);
    vars.retain(|tv| !is_opaque_tyvar(&tv.name) && !opaque_eq_only_vars.contains(&tv.name));
    Scheme::new_arc(vars, preds, eqs, ty)
}
```

`collect_opaque_equality_formal_params` は、opaque 関連の equality に出現するが `ty` や非 opaque predicates には出現しない型変数名を返す。

### 0-4. `Scheme::set_kinds` の変更

**ファイル**: `src/ast/types.rs`

現在の `set_kinds` は `gen_vars` のみを対象にカインドの初期化（非`*`カインドの KindScope 登録）と反映（KindScope からのカインド更新）を行うが、opaque TyVar と equality 仮引数は `gen_vars` に含まれないため処理されない。

`set_kinds` を修正し、`gen_vars` に限らず predicates・equalities・ty 内の全 free type variable を扱うようにする：

1. **ステップ1（カインド初期化）**: `gen_vars` の代わりに、全 free vars から非`*`カインドを KindScope に追加
2. **ステップ4（カインド反映）**: `gen_vars` に加えて、predicates・equalities・ty 内の全 TyVar にもカインドを設定

```rust
pub fn set_kinds(&self, kind_env: &KindEnv) -> Result<Arc<Scheme>, Errors> {
    let mut ret = self.clone();
    let mut kind_scope = KindScope::new();
    // ★ CHANGED: gen_vars だけでなく全 free vars から明示カインドを取得
    let mut all_vars = vec![];
    for p in &self.predicates { p.free_vars_to_vec(&mut all_vars); }
    for eq in &self.equalities { eq.free_vars_to_vec(&mut all_vars); }
    self.ty.free_vars_to_vec(&mut all_vars);
    for tv in &self.gen_vars { /* gen_vars にしかないものも追加 */
        if !all_vars.iter().any(|v| v.name == tv.name) {
            all_vars.push(tv.clone());
        }
    }
    for tv in &all_vars {
        if tv.kind != kind_star() {
            kind_scope
                .insert(tv.name.clone(), tv.kind.clone())
                .map_err(|msg| Errors::from_msg_srcs(msg, &[&ret.ty.get_source()]))?;
        }
    }
    let res = kind_scope.extend(&ret.predicates, &ret.equalities, &vec![], kind_env);
    // ... (エラー処理は既存のまま) ...
    for p in &mut ret.predicates { p.set_kinds(&kind_scope); }
    for eq in &mut ret.equalities { eq.set_kinds(&kind_scope); }
    ret.ty = ret.ty.set_kinds(&kind_scope);
    for tv in &mut ret.gen_vars {
        *tv = kind_scope.set_tv(tv);
    }
    Ok(Arc::new(ret))
}
```

ステップ4 で `gen_vars` のみ更新するのは、predicates・equalities・ty 内の TyVar は既にステップ3（`p.set_kinds` / `eq.set_kinds` / `ty.set_kinds`）でカインド設定済みのため。

---

## Phase 1: 型コンストラクタの生成

**ファイル**: `src/elaboration/desugar_opaque.rs`（新規作成）、`src/ast/types.rs`、`src/ast/program.rs`

### 1-1. opaque 型変数の検出

各 GlobalValue の Scheme を走査し、`?` で始まる TyVar を opaque 型変数として検出する。

```rust
fn is_opaque_tyvar(name: &str) -> bool {
    name.starts_with('?')
}
```

対象：
- `program.global_values` 内の各 GlobalValue の `scm`

`create_trait_member_symbols` の後であるため、trait member の情報も GlobalValue（`SymbolExpr::Method`）に含まれている。TraitDefn を別途走査する必要はない。

### 1-2. TyConVariant::Opaque の追加

**ファイル**: `src/ast/types.rs`

```rust
pub enum TyConVariant {
    Primitive,
    Arrow,
    Array,
    Struct,
    Union,
    DynamicObject,
    Opaque,         // ★ NEW
}
```

### 1-3. TyCon の生成と TypeEnv への登録

各 opaque 型変数 `?it` に対して：

1. **FullName の生成**：
   - グローバル値の場合：`FullName::new(&fn_fullname.to_namespace(), "?it")`  
     例：`Std::Iterator::repeat::?it`
   - trait member の場合：`FullName::new(&FullName::new(&trait_id.name.to_namespace(), &member.name).to_namespace(), "?it")`  
     例：`ToIter::to_iter::?it`

2. **Kind の計算**：
   - opaque 型変数を除いた gen_vars の各カインドから決まる。gen_vars が `[v1 : k1, v2 : k2, ..., vn : kn]` の場合、opaque type constructor の kind は `k1 -> k2 -> ... -> kn -> *`。
   - 例 1（グローバル値）：`repeat` の `?it`、gen_vars = `[a : *]` → kind `* -> *`
   - 例 2（trait member）：`to_iter` の `?it`、gen_vars = `[c : *]`（trait の型変数） → kind `* -> *`
   - 例 3（higher-kinded gen_var）：仮に gen_vars = `[f : * -> *]` の場合 → kind `(* -> *) -> *`
   - グローバル値の場合も trait member の場合も同じ処理で統一できる（GlobalValue の Scheme の gen_vars を使う）。

3. **TyConInfo の作成と登録**：
   ```rust
   let ti = TyConInfo {
       kind: computed_kind,
       variant: TyConVariant::Opaque,
       is_unbox: false,
       tyvars: non_opaque_gen_vars.clone(),
       fields: vec![],
       source: decl_src,
       document: None,
   };
   program.type_env.add_tycons(Map::from([(tycon.clone(), ti)]));
   ```

### 1-4. 名前衝突の確認

生成される TyCon の FullName が既存の型と衝突しないことを確認する。`?` から始まるため通常は衝突しない。

---

## Phase 2: 制約の変換

**ファイル**: `src/elaboration/desugar_opaque.rs`、`src/ast/traits.rs`

### 2-1. TraitEnv の拡張

**ファイル**: `src/ast/traits.rs`

```rust
pub struct TraitEnv {
    pub traits: Map<TraitId, TraitDefn>,
    pub impls: Map<TraitId, Vec<TraitImpl>>,
    // ★ NEW: opaque type から生成された仮定
    pub opaque_preds: Map<TraitId, Vec<QualPredScheme>>,
    pub opaque_eqs: Map<AssocType, Vec<EqualityScheme>>,
}
```

### 2-2. qualified_predicates() / type_equalities() の拡張

`TraitEnv::qualified_predicates()` と `TraitEnv::type_equalities()` を拡張し、`opaque_preds` / `opaque_eqs` の内容もマージして返すようにする。これにより、`TypeCheckContext::new()` が自動的に opaque の仮定を取り込む。

### 2-3. 制約の変換ロジック

各 opaque 型変数 `?it` の制約を QualPredScheme / EqualityScheme に変換する。

**グローバル値の場合**（`repeat`）：

- Scheme の predicates から opaque 関連を抽出
  - `?it : Iterator` → QualPredScheme `{ gen_vars: [a], qual_pred: { predicate: ?it_tycon a : Iterator } }`
- Scheme の equalities から opaque 関連を抽出
  - `Item ?it = a` → EqualityScheme `{ gen_vars: [a], equality: Item (?it_tycon a) = a }`

ここで `?it_tycon a` は `TyApp(TyCon(repeat::?it), TyVar(a))`。

変換時の注意：
- 元の制約では `?it` は TyVar だが、変換後は TyCon + gen_vars の TyApp に置換する
- gen_vars は opaque 型変数を除いた元の Scheme の gen_vars

**trait member の場合**（`to_iter`）：

- `?it : Iterator` → 条件付き QualPredScheme  
  `{ gen_vars: [c], qual_pred: { conditions: [c : ToIter], predicate: ?it_tycon c : Iterator } }`
- `Item ?it = ToIter::Elem c` → EqualityScheme  
  `{ gen_vars: [c], equality: Item (?it_tycon c) = ToIter::Elem c }`

条件 `c : ToIter` は QualPred の `pred_constraints` に入る。

### 2-4. opaque_preds / opaque_eqs への追加

変換した QualPredScheme / EqualityScheme を `program.trait_env.opaque_preds` / `program.trait_env.opaque_eqs` に追加する。

---

## Phase 3: 型シグネチャと式の書き換え

**ファイル**: `src/elaboration/desugar_opaque.rs`

### 3-1. 型シグネチャの書き換え

各 Scheme に対して：

1. **TyVar → TyCon 置換**：型中の opaque TyVar `?it` を `TyApp(TyCon(repeat::?it), non_opaque_gen_vars...)` に置換する。
   - `ty` 内の TyVar("?it") → TyApp(TyCon(repeat::?it), TyVar(a))
   - predicates / equalities 内の TyVar("?it") も同様に置換（ただし opaque を除去するものについてはこの時点で除去）

2. **opaque 制約の除去**：Scheme.predicates から opaque 関連の predicate を除去。Scheme.equalities から opaque 関連の equality を除去。

※ `gen_vars` からの opaque TyVar / equality 仮引数の除外は Phase 0-3（`Scheme::generalize`）で済んでいるため、ここでは不要。

**before**:
```
gen_vars: [a], predicates: [?it : Iterator], equalities: [Item ?it = a]
ty: a -> I64 -> ?it
```
**after**:
```
gen_vars: [a], predicates: [], equalities: []
ty: a -> I64 -> TyApp(TyCon(repeat::?it), a)
```

trait member の場合、`create_trait_member_symbols()` が既に GlobalValue（`SymbolExpr::Method`）を作成済みなので、GlobalValue の `scm` と各 `TraitMemberImpl` の `scm` / `scm_via_defn` を書き換える。

### 3-2. #wrap GlobalValue の生成

各関数（またはtrait member の各 impl）に対して #wrap の GlobalValue を生成する。

**グローバル値の場合**（`repeat`）：

1. **名前**: `FullName::new(&fn_fullname.to_namespace(), "#wrap")`  
   例：`Std::Iterator::repeat::#wrap`

2. **Scheme の構築**: 
   ```
   gen_vars: [a, x]
   predicates: [x : Iterator]
   equalities: [Item x = a]
   ty: (a -> I64 -> x) -> (a -> I64 -> TyApp(TyCon(repeat::?it), a))
   ```
   - `x` は新しい TyVar（domain 側の実装型を表す）。`?it` が複数ある場合は各 opaque に対して `x`, `y`, ... を生成。
   - domain: 元の関数の型から opaque TyVar を `x` に置換したもの
   - codomain: 書き換え後の関数の型（opaque TyCon 版）

3. **式**: `SymbolExpr::Simple(TypedExpr::from_expr(expr_app(expr_var(std_undefined_name), [expr_string_literal("")], None)))`  
   body は `undefined("")`。型チェックは通る（`undefined` の戻り型は任意型 `a`）。

4. **GlobalValue として登録**:
   ```rust
   program.global_values.insert(wrap_name, GlobalValue {
       scm: wrap_scheme,
       syn_scm: None,
       expr: SymbolExpr::Simple(TypedExpr::from_expr(undefined_expr)),
       decl_src: original_decl_src,
       defn_src: None,
       document: None,
       compiler_defined_method: true,
   });
   ```

**trait member の場合**（`to_iter`、impl ごと）：

1. **名前**: impl 型を含む名前。例：`ToIter::to_iter[Array a]::#wrap`
   - FullName の構築には impl 型の文字列表現を使う
   - 例：`FullName::new(&FullName::new(&trait_ns, "to_iter[Array a]").to_namespace(), "#wrap")`
   - 名前の一意性が保証されれば具体的なフォーマットは自由

2. **Scheme**: impl 固有の型（`trait_impl.member_scheme_by_defn` から構築し、opaque TyVar を domain TyVar に対応付け）
   ```
   gen_vars: [a, x]
   predicates: [x : Iterator]
   equalities: [Item x = a]
   ty: (Array a -> x) -> (Array a -> TyApp(TyCon(to_iter::?it), TyApp(TyCon(Array), a)))
   ```

3. **式**: `undefined("")`（グローバル値と同様）

4. **GlobalValue として登録**

### 3-3. 式への #wrap 挿入

**グローバル値の場合**：

GlobalValue の式を `#wrap(original_expr)` に書き換える：
```rust
let wrapped = expr_app(
    expr_var(wrap_fullname, None),
    vec![original_expr],
    original_expr.source.clone(),
);
gv.expr = SymbolExpr::Simple(TypedExpr::from_expr(wrapped));
```

**trait member の場合**：

`create_trait_member_symbols()` が作成済みの GlobalValue（`SymbolExpr::Method`）内の各 `TraitMemberImpl.expr` を書き換える：
```rust
// program.global_values 内の SymbolExpr::Method を走査
for impl_ in member_impls {
    let original_expr = impl_.expr.expr.clone();
    impl_.expr = TypedExpr::from_expr(expr_app(
        expr_var(per_impl_wrap_fullname, None),
        vec![original_expr],
        None,
    ));
}
```

---

## Phase 4: 型チェック

変更なし。既存の型チェックがそのまま動く。

- QualPredScheme / EqualityScheme は `TypeCheckContext::new()` で assumed_preds / assumed_eqs に組み込まれる（Phase 2-2 の拡張による）。
- `#wrap` は通常の GlobalValue として型チェックされる（`undefined("")` の body は任意型に推論される）。
- 使用側では opaque TyCon のまま扱われ、QualPredScheme の仮定により trait メンバを呼べる。

---

## Phase 5: 具体型テンプレートの決定と保存

**ファイル**: `src/ast/program.rs`、`src/elaboration/typecheck.rs`

### 前提：fix_types の2段階呼び出し

`fix_types` はコンパイル中に2回呼ばれる：
1. **1回目：`check_type` 内**（本フェーズ）：各 ExprNode の `type_` は、gen_vars と associated types を含みうるが、fixed でない型変数は含まない状態に「fix」される。
2. **2回目：`instantiate_symbol` 内**（Phase 6）：各 ExprNode の `type_` は完全に具体的な型（型変数も associated types も opaque type constructor も残らない）に「fix」される。

本フェーズでは1回目の `fix_types` 後の状態から、opaque type の**具体型テンプレート**を抽出する。テンプレートは gen_vars や associated types を含みうる。これらは Phase 6 の2回目の `fix_types` で完全に解消される。

### 5-1. OpaqueConcreteType の定義

**ファイル**: `src/ast/program.rs`（または新規ファイル）

```rust
/// opaque type constructor の具体型テンプレート。
/// 1回目の fix_types 後に抽出される。gen_vars や associated types を含みうる。
/// 2回目の fix_types（instantiate_symbol 内）で完全に具体化される。
pub struct OpaqueConcreteType {
    /// opaque type constructor
    pub opaque_tycon: Arc<TyCon>,
    /// 型引数（TyCon の tyvars と同じ）
    pub gen_vars: Vec<Arc<TyVar>>,
    /// 具体型テンプレート（gen_vars や associated types を含みうる）
    pub concrete_ty: Arc<TypeNode>,
}
```

### 5-2. TypedExpr の拡張

**ファイル**: `src/ast/program.rs`

```rust
pub struct TypedExpr {
    pub expr: Arc<ExprNode>,
    pub equalities: Vec<Equality>,
    pub opaque_types: Vec<OpaqueConcreteType>,  // ★ NEW
}
```

- `opaque_types` は当該関数（または trait member impl）が持つ opaque type の具体型情報。
- serde_pickle による cache の serialize/deserialize にも含める必要がある。

### 5-3. 具体型の抽出

**ファイル**: `src/ast/program.rs`（`resolve_namespace_and_check_type_sub` 内）

`check_type` の後、返された TypedExpr の式から #wrap の呼び出しを探し、具体型を抽出する：

```rust
// check_type 完了後
let typed_expr = tc.check_type(expr, scm)?;

// #wrap 呼び出しを検出し、具体型を抽出
let opaque_types = extract_opaque_concrete_types(&typed_expr.expr);
typed_expr.opaque_types = opaque_types;
```

抽出ロジック：
1. 式のトップレベルが `App(Var(#wrap_name), [inner_expr])` の場合
2. `Var(#wrap_name).type_` は `D -> C` の形（D = domain 型、C = codomain 型）
3. D と C を構造的に走査し、C 内の opaque TyCon 出現位置に対応する D 内の型を取得
4. OpaqueConcreteType を構築

走査の具体例（`repeat`）：
- C = `a -> I64 -> ?it a`
- D = `a -> I64 -> MapIterator (RangeIterator I64) a`
- Arrow の引数を順に比較。第3要素: `?it a` vs `MapIterator (RangeIterator I64) a`
- `?it a` は `TyApp(TyCon(repeat::?it), TyVar(a))` → opaque TyCon 検出
- gen_vars = `[a]`、concrete_ty = `MapIterator (RangeIterator I64) a`

### 5-4. キャッシュへの保存

`save_cache` で TypedExpr をシリアライズする際、`opaque_types` も含める。`TypedExpr` の serde derive に `OpaqueConcreteType` を追加する。

---

## Phase 6: インスタンス化時の opaque type 解消

**ファイル**: `src/ast/program.rs`（`instantiate_symbol` 内）

### 前提：2回目の fix_types

`instantiate_symbol` 内の `fix_types`（2回目）は、1回目で残っていた gen_vars や associated types をすべて解消し、完全に具体的な型にする。fix_types 自体は opaque type constructor を通常の TyCon として扱い、変更不要。

2回目の `fix_types` 後には、すべての ExprNode の `type_` から型変数と associated types は除去されるが、opaque type constructor はまだ残っている。opaque type constructor の解消は fix_types の後に独立した処理として行う。

### 6-1. opaque TyCon の置換

`instantiate_symbol` 内で、2回目の `fix_types` の**後**に、独立した処理として opaque TyCon を具体型に置換する：

```rust
// instantiate_symbol 内
fix_types(...);  // 2回目の fix_types（既存のまま変更なし）

// ★ NEW: fix_types の後に独立した処理として実行
let expr = resolve_opaque_types(expr, &typed_expr.opaque_types);
let expr = remove_wraps(expr);
```

`resolve_opaque_types` のロジック：
1. 型中の `TyApp` を走査
2. toplevel の TyCon が opaque（`TyConVariant::Opaque`）の場合
3. TyCon の引数（TyApp を uncurry して取得）と OpaqueConcreteType の gen_vars を対応付け
4. concrete_ty に gen_vars → 引数 の substitution を適用
5. 置換後の型で TyApp を置き換え
6. 式のすべてのノードの `type_` フィールドに再帰的に適用

この時点では fix_types により gen_vars も associated types も解消済みなので、opaque TyCon の型引数は完全に具体的。OpaqueConcreteType の concrete_ty に gen_vars → 型引数 の substitution を適用すると、結果も完全に具体的な型になる。

resolve_opaque_types と remove_wraps の完了後、すべての ExprNode の `type_` が**完全に具体的な型**（型変数も associated types も opaque type constructor も残らない）になる。

### 6-2. #wrap の除去

`resolve_opaque_types` の直後に、#wrap の App を除去する（上記コード例の `remove_wraps`）：

1. 式が `App(Var(name), [inner])` で `name` が `#wrap` の名前パターンに合致する場合
2. `inner` に置換する（#wrap は恒等関数）

```rust
fn remove_wraps(expr: Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::App(func, args) if is_wrap_var(func) && args.len() == 1 => {
            remove_wraps(args[0].clone())
        }
        _ => // 再帰的にサブ式を処理
    }
}
```

### 6-3. コード生成への影響

Phase 6 完了後、式中に opaque TyCon は存在しない。すべて完全に具体的な型に置換済み。よってコード生成（`src/object.rs` の `ty_to_object_ty` 等）は変更不要。

---

## Validation（検証）の追加

### V-1. opaque 型変数の使用箇所制限

`validate_type_defns` / `validate_trait_env` で検証：
- `type_defn` の型パラメータに `?` 始まりの型変数が出現した場合はエラー（`validate_type_defns` で検証）。
- `trait_defn` の型パラメータに `?` 始まりの型変数が出現した場合はエラー（`validate_trait_env` で検証）。
- opaque 型変数は関数の型シグネチャ（制約ブロックおよび型本体）でのみ使用可能。

パーサーではなくこれらの関数で検証する理由：パーサーでエラーにすると、そのエラー1つだけが報告され他の検証（型定義やトレイトの整合性チェック等）が一切行われない。`validate_type_defns` / `validate_trait_env` で検証すれば、他のエラーもまとめて報告できる。

### V-2. opaque 型変数に対する最低1つの制約

opaque 型変数には最低1つの trait predicate が必要…とは限らない。制約なしでも文法上は許可する。ただし実用上は意味がないため、warning としてもよい（要検討）。

### V-3. Equality 制約の formal parameter チェック

equality `MyAssocTy t <t1> ... <tn> = <type>` において、`<t1> ... <tn>` は**仮引数**でなければならない（plan2.md「Equality制約の自由度」セクション参照）。具体的には：

1. **型変数であること**: `<t1> ... <tn>` の各々が TyVar であること。（例: `Rebuild c I64 = Array I64` は不可 — `I64` は型変数ではない）
2. **RHS 以外に出現しないこと**: `<t1> ... <tn>` は equality の RHS（`<type>`）にのみ出現し、関数の型本体（`ty`）や他の制約には出現しないこと。（例: `Rebuild c b = Array b` の `b` は OK — `b` は他に出現しない。しかし `Rebuild c a = Array a` で `a` が `ty` にも出現するなら NG）

このチェックは **`validate_constraints` の強化**として実装する。理由：
- この制約は opaque type 固有ではなく、すべての equality に適用される一般的な制約（`reduce_type_by_equality` の合流性のため）
- `validate_global_value_type_constraints`（→ `validate_constraints`）は `desugar_opaque_types` の前に実行されるため、opaque の equality もまだ `Scheme.equalities` 内にあり、チェック対象に含まれる
- 既存の equality チェックと一元管理できる

---

## エラーメッセージの検証と改善

Phase 6 までの基本実装が完了した後：

1. **opaque type の性質を破るコード**をコンパイルし、エラーメッセージを確認する。例：
   - opaque type を具体型として直接構築しようとするコード
   - opaque type が要求する trait を持たない操作を行うコード
   - 関連する型エラー全般

2. エラーメッセージに opaque TyCon の内部名（`Std::Iterator::repeat::?it`）が出る場合、可読性の高い表示に変換する。例：
   - `opaque type ?it in repeat` のような表示
   - 元の型シグネチャのどの部分に対応するかの情報

---

## LSP 対応

### LSP-1. ホバー時の opaque type 解決表示

opaque type にホバーしたとき、Phase 5 で求めた具体型を表示する。

例：`pi : ?f = 3.14;` の `?f` にホバー → `F64` と表示。

実装箇所：`src/commands/lsp/` 内のホバー処理

---

## テスト計画

別ドキュメント（todo.md のテスト計画セクション）参照。概要：

- `test_opaque_type` モジュールを新設
- use_cases.md のサンプルが動くことを確認
- trait member impl でのアノテーションに opaque type を使う例
- higher-kinded opaque type / higher-arity associated type の例
- 各 validation に fail するコードのエラーメッセージ確認
- LSP テスト（ホバーで解決型表示）

---

## 実装順序

推奨する実装順序（依存関係を考慮）：

1. **Phase 0**: 文法変更（grammer.pest）— 最も基礎的。他のすべてに先行。
2. **Phase 1-3**: desugar_opaque.rs の実装 — 一つの関数 `desugar_opaque_types` にまとめる。
   - 1-2 (TyConVariant::Opaque) → 1-3 (TyCon生成) → 2-1/2-2 (TraitEnv拡張) → 2-3/2-4 (制約変換) → 3-1 (署名書き換え) → 3-2/3-3 (#wrap生成・挿入)
3. **Phase 4**: 変更なし。Phase 1-3 完了後にテスト（型チェックが通ることを確認）。
4. **Phase 5**: 具体型抽出。TypedExpr 拡張、extract ロジック、キャッシュ対応。
5. **Phase 6**: instantiate_symbol 内の opaque 解消。#wrap 除去。
6. **テスト**: 各フェーズの単体テスト + 統合テスト。
7. **エラーメッセージ改善**: 基本実装完了後。
8. **LSP対応**: 基本実装完了後。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `src/parse/grammer.pest` | `type_var` ルールに `"?"?` 追加 |
| `src/parse/parser.rs` | 変更なし（`?it` が自然に `"?it"` としてパースされる） |
| `src/ast/types.rs` | `Scheme::generalize` で opaque 型変数と equality 仮引数を `gen_vars` から除外、`Scheme::set_kinds` で全 free vars を扱うよう変更、`TyConVariant::Opaque` 追加 |
| `src/elaboration/validate_type_defns.rs` 相当 | `type_defn` の型パラメータに opaque 型変数が含まれていないか検証 |
| `src/elaboration/validate_trait_env.rs` 相当 | `trait_defn` の型パラメータに opaque 型変数が含まれていないか検証 |
| `src/ast/traits.rs` | `TraitEnv` に `opaque_preds` / `opaque_eqs` フィールド追加、`qualified_predicates()` / `type_equalities()` のマージ拡張 |
| `src/ast/program.rs` | `OpaqueConcreteType` 定義、`TypedExpr::opaque_types` 追加、`instantiate_symbol` 内の opaque 解消ロジック |
| `src/elaboration/mod.rs` | `elaborate()` に `desugar_opaque_types()` 呼び出し追加 |
| `src/elaboration/desugar_opaque.rs` | ★ 新規: `desugar_opaque_types()` 実装（Phase 1-3 のすべて） |
| `src/elaboration/typecheck.rs` | Phase 5 の具体型抽出（`check_type` 後処理） |
| `src/elaboration/typecheckcache.rs` | `OpaqueConcreteType` の serialize/deserialize 対応 |
| `src/object.rs` | 変更なし（Phase 6 で opaque TyCon は除去済み） |
| `src/commands/lsp/` | ホバー時の opaque type 解決表示 |
| `src/tests/` | `test_opaque_type` モジュール新設 |

---

## 実行可能性の検証ポイント

### 課題1: desugar の前に実行されるステップとの互換性

`desugar_opaque_types()` は `set_kinds()` の後、typechecker の直前に実行される。それより前のステップとの互換性：

- **resolve_namespace_not_in_expr**: opaque TyVar はまだ TyVar のまま。TyVar の名前解決は不要なので問題なし。
- **validate_trait_env**: opaque 制約はまだ Scheme 内にあるが、trait 定義の検証には影響しない。
- **create_trait_member_symbols**: opaque 制約を含んだ Scheme で GlobalValue が作られるが、desugar で書き換える。
- **validate_global_value_type_constraints**: opaque 制約（`?it : Iterator`）は既存の trait を参照するので通過する。
- **set_kinds**: `Scheme::set_kinds` を修正し、gen_vars だけでなく全 free type variable からカインドを初期化するため、opaque TyVar や equality 仮引数が gen_vars に含まれなくても問題なし（Phase 0-4 参照）。

### 課題2: desugar 後に生成される TyCon の名前解決

desugar 後に生成される opaque TyCon の名前は完全修飾名なので、名前解決は不要。
`#wrap` GlobalValue も完全修飾名で登録され、式中の参照も完全修飾名なので問題なし。

### 課題4: cache の互換性

`TypedExpr` に `opaque_types` フィールドを追加すると、古い cache との互換性が崩れる。serde_pickle の default 値設定（`#[serde(default)]`）で対処可能。古い cache は opaque_types が空として読まれる。
