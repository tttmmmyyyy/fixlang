# Handoff: 数値キャストトレイト関連の構造リファクタ (Option A1 改良案 + old→new 委譲)

## 背景

既存の auto-generated トレイト impl（`Std::To<X>` 群）は、ソース文字列を組み立てて `parse_and_save_to_temporary_file` でパースする方式で構築されている。

現在 (commit `24060ad5` を取り消した未コミット状態) は、ソーステキスト上はまだ `impl I32 : ToF64 { f64 = to_F64; }` のように deprecated 関数 `to_F64` を呼ぶ形で書かれており、パース直後の AST を Rust 側で書き換えて、trait impl の本体を実際の LLVM キャスト式に差し替えている。

```rust
// stdlib.rs::make_numeric_cast_traits_mod (現状)
let mut prog = parse_and_save_to_temporary_file(&src, "std_numeric_cast_traits", config)?;

// (a) (from_name, to_name) -> 完成済みキャストラムダ のマップを構築
let mut cast_exprs: Map<(String, String), Arc<ExprNode>> = Map::default();
// ... 8 ループでマップを埋める ...

// (b) パース後 AST のトレイト impl を走査し、本体を上記キャスト式で上書き
for (trait_id, impls) in prog.trait_env.impls.iter_mut() {
    let to_name = trait_id.name.name.strip_prefix("To").unwrap();
    let method_name = upper_camel_to_lower_snake(&to_name);
    for impl_ in impls.iter_mut() {
        let from_name = impl_.qual_pred.predicate.ty
            .toplevel_tycon().unwrap().name.name.clone();
        if let Some(expr) = cast_exprs.get(&(from_name, to_name.to_string())) {
            impl_.members.insert(method_name.clone(), expr.clone());
        }
    }
}
```

これは動作するが、**ソースファイルとメモリ上 AST に乖離がある**（ソースには `to_F64`、AST には実体のキャスト式）という不純さを残す。

## このリファクタのゴール

二段階の構造改善を行う：

### Step 1: トレイト impl の programmatic 構築 (Option A1 改良案)

`make_numeric_cast_traits_mod` を以下のように作り直す：

1. **ソース文字列にはトレイト宣言のみを書く。impl は一切書かない。**
   ```fix
   trait a : ToF64 {
       f64 : a -> F64;
   }
   trait a : ToI64 {
       i64 : a -> I64;
   }
   // ... 全 To<X> トレイトの宣言だけ ...
   ```
2. パース後、Rust 側で `TraitImpl` 構造体を直接構築し、`prog.trait_env.add_instance(...)` で追加する。
3. 各 `TraitImpl.members[<method>]` には、`cast_between_integral_function` 等が返す完成済みキャストラムダ式をそのまま入れる。

これで「ソースの嘘」が消え、deprecation 警告が `std_numeric_cast_traits.<hash>.fix` の特定行を指すこともなくなる（impl がそもそもソースに無いため）。

### Step 2: `to_<To>` を「トレイトメソッド呼び出し」へ書き換え (真の old→new 委譲)

現状 (Step 1 完了後) は以下の二重登録：
- `Std::<From>::to_<To>` の本体 = LLVM キャスト式（deprecated）
- `Std::To<To>::<method>` の I32-impl 本体 = 同じ LLVM キャスト式

**同一実装の二重持ち**になっており、構造的にも実質的にも `to_<To>` がトレイトメソッドを呼ぶ形（一般的な慣例）になっていない。

Step 2 では `make_std_mod` の 8 つのキャスト登録ループを書き換え、`Std::<From>::to_<To>` の本体を **トレイトメソッドへの薄い委譲**にする：

```rust
// 現状 (の make_std_mod 内)
errors.eat_err(fix_module.add_global_value(
    target.clone(),
    cast_between_integral_function(from.clone(), to.clone(), None),  // ← LLVM キャスト式
    None, None,
    Some(format!("Casts a value of `{}` into a value of `{}`.", ...)),
));
```

を以下のように変える：

```rust
let trait_method = FullName::from_strs(
    &[STD_NAME, &format!("To{}", to_name)],
    &upper_camel_to_lower_snake(&to_name),
);
let body = expr_var(trait_method, None);  // ← 単純な Var 参照
let scm = Scheme::generalize(
    Default::default(),
    vec![],
    vec![],
    type_fun(from.clone(), to.clone()),
);
errors.eat_err(fix_module.add_global_value(
    target.clone(),
    (body, scm),
    None, None,
    Some(format!("Casts a value of `{}` into a value of `{}`.", ...)),
));
```

その後 `add_deprecation` は変えない。

これで「`to_F64` (deprecated) → トレイト method `ToF64::f64` (canonical) → LLVM キャスト式」というコンセプトが実現される。

**point-free スタイルの注意**：`to_F64 = ToF64::f64;` は型推論で `[a : ToF64] a -> F64` を `I64 -> F64` に特殊化する。Fix のオーバーロード解決で動作する（同パターンが std.fix の `_unsafe_from_c_str_ptr = unsafe_from_c_str_ptr;` 等で既に使われている）。

**FullName の絶対性**：`expr_var` で渡す FullName は **絶対パス** (`Std::ToF64::f64`) にする。理由は Step 1 の「名前解決のタイミング」と同じく、programmatic 登録は name resolution を経由しないため。

**循環の心配**：`Std::I64::to_F64`'s body が `ToF64::f64` を呼び、`ToF64::f64` の I64 用 impl 本体は LLVM キャスト式。typecheck 後に I64 impl が選択されるので循環しない。

## 主要参照ポイント

### 関連する既存コード

- `src/fixstd/stdlib.rs::make_numeric_cast_traits_mod` — このリファクタのターゲット関数
- `src/fixstd/stdlib.rs::make_std_mod` 内の 8 つのキャスト登録ループ（ライン 204-396 付近）— 同じキャスト式構築ロジックがある（重複）
- `src/fixstd/builtin.rs::cast_between_integral_function` / `cast_between_float_function` / `cast_int_to_float_function` / `cast_float_to_int_function` — `(Arc<ExprNode>, Arc<Scheme>)` を返す。式は外部シンボル参照のない閉じたラムダ。

### 構築すべきデータ構造

`src/ast/traits.rs::TraitImpl`:

```rust
pub struct TraitImpl {
    pub qual_pred: QualPred,
    pub members: Map<Name, Arc<ExprNode>>,
    pub member_lhs_srcs: Map<Name, Vec<Span>>,
    pub member_sigs: Map<Name, QualType>,
    pub assoc_types: Map<Name, AssocTypeImpl>,
    pub define_module: Name,
    pub source: Option<Span>,
    pub is_user_defined: bool,
}
```

各フィールドの設定指針：

| フィールド | 値 |
|-----------|-----|
| `qual_pred` | `QualPred { pred_constraints: vec![], eq_constraints: vec![], kind_constraints: vec![], predicate: Predicate { trait_id, ty: from_type, info: PredicateInfo::default() (or src: None) } }` |
| `members` | `{ method_name -> cast_expr }` 単一エントリ |
| `member_lhs_srcs` | `Default::default()` (空) — ソースに対応する span が無いため |
| `member_sigs` | `Default::default()` (空) — 型シグネチャはトレイト宣言から推論 |
| `assoc_types` | `Default::default()` |
| `define_module` | `STD_NAME.to_string()` |
| `source` | `None` |
| `is_user_defined` | `false` (コンパイラ生成) |

`src/ast/predicate.rs::Predicate` のフィールドを確認すること。`info: PredicateInfo` も含む可能性。

### `TraitId` の構築

```rust
let trait_id = TraitId::from_fullname(
    FullName::from_strs(&[STD_NAME], &format!("To{}", to_name))
);
```

### `TraitEnv::add_instance` の使い方

`src/ast/traits.rs:1304` 付近：

```rust
pub fn add_instance(&mut self, inst: TraitImpl) -> Result<(), Errors> {
    let trait_id = inst.trait_id();
    if !self.impls.contains_key(&trait_id) {
        self.impls.insert(trait_id.clone(), vec![]);
    }
    self.impls.get_mut(&trait_id).unwrap().push(inst);
    Ok(())
}
```

呼び出し側：
```rust
prog.trait_env.add_instance(trait_impl)?;
```

## 実装手順（推奨）

### Step 1: `make_numeric_cast_traits_mod` を作り直す

- ソース生成部から impl 部分を削除。トレイト宣言のみ生成。
- パース後、現実装と同じく `cast_exprs` マップを構築（8 ループ）。
- 各 `(from_name, to_name)` ペアに対して `TraitImpl` を構築し、`prog.trait_env.add_instance` で追加。

ここで一旦動作確認：cp-library `fix test` が完走、`grep std_numeric_cast` 0 件、生成ソースに impl が無いこと。

### Step 2: `make_std_mod` の `to_<To>` 登録をトレイトメソッド委譲に置き換え

- 8 つのループそれぞれで、現状の `cast_*_function(...)` を本体に登録している部分を、`expr_var` でトレイトメソッドの絶対 FullName を参照する body に置き換える。
- `Scheme` は `from -> to` の関数型を generalize したもの（既存と同じ）。
- `add_deprecation` 呼び出しは現状のまま維持。

ここで動作確認：cp-library `fix test` が完走、warning が「ユーザコードで `to_<To>` を呼んでいる箇所」だけになり、それぞれの`to_<To>` を呼ぶラムダの中で警告がカスケードしないこと。

### Step 3 (任意): コードの共通化

- Step 1 の `cast_exprs` 構築と Step 2 のループは「8 つの `(from, to)` パターンを舐める」点で共通。
- 共通ヘルパに切り出せる：
  ```rust
  fn for_each_numeric_cast<F: FnMut(&str, &Arc<TypeNode>, &str, Arc<ExprNode>, Arc<Scheme>)>(
      config: &Configuration,
      mut f: F,
  ) {
      // 8 ケースを内部で展開して f を呼ぶ
  }
  ```
- 共通化は読みやすさ向上が目的なので、優先度は低い。

### 動作確認

- 既存テスト：`cargo test --release --bin fix test_deprecation` で 9/9 通過すること。
- cp-library ベンチ（手動）：
  ```bash
  cd ~/fixlang-projs/cp-library
  rm -rf .fixlang/run/ .fixlang/intermediate/ .fixlang/cache/
  time fix test
  ```
  53 秒前後で完走、`grep -c std_numeric_cast` で 0 件であること。
- 生成されたソースファイルの確認：
  ```
  ~/.fixlang/tmp/src/std_numeric_cast_traits.<hash>.fix
  ```
  impl 行が **存在しないこと**。トレイト宣言だけになっていること。
- Step 2 後の追加確認：
  - ユーザコードで `let _ : F64 = (3 : I64).to_F64;` するとちゃんと deprecation 警告が出ること。
  - ユーザコードで `let _ : F64 = (3 : I64).f64;` の場合は警告ゼロであること。
  - LLVM の最適化が効いて、`to_F64` 経由でも 1 回のキャストにインライン化されていること（生成された `.ll` を確認）。

## 注意点・落とし穴

### `Predicate.info` などの src フィールド

`Predicate` や `QualType` は `info: PredicateInfo` のようなソース span 情報を持つフィールドが含まれる可能性。`Default::default()` か `None` で埋めれば問題ないはず。コンパイル時は src 情報なしでも動く。

### 名前解決のタイミング

programmatic に作った TraitImpl の `predicate.trait_id` の FullName は **絶対パス**（`Std::ToF64`）にしておくと安全。
理由：パース由来の trait_id は後段の `resolve_namespace_*` で解決されるが、programmatic 構築だと解決ステップを飛ばすため、最初から正しいフルネームを与える必要がある。

確認用：パーサが生成した既存の他の TraitImpl の `predicate.trait_id` の中身（`is_absolute` を含む）を `dbg!` で覗いて同形式に揃える。

### `impl_type_as_written` のような派生情報

`AssocTypeImpl` には `impl_type_as_written` があるが、`TraitImpl` 自体には基本ない。impl_type は `qual_pred.predicate.ty` から取得するのみ。

### kind 設定

programmatic 構築した TraitImpl の `qual_pred.predicate.ty` に kind 情報が入っていないと、後段の `set_kinds_in_qual_pred_and_member_sigs` で問題になる可能性。`integral_types()` / `floating_types()` が返す `Arc<TypeNode>` には既に kind=`*` が設定されているはず。確認：
```rust
let from_ty = ...; // from integral_types()
dbg!(&from_ty.kind);
```

### `define_module` の値

`STD_NAME = "Std"`。ソース由来の TraitImpl は `define_module = "Std"` になっている（生成ソースが `module Std;` で始まるため）。programmatic 構築でも同じ値で揃える。

## 完了の定義

- 既存 9 件の deprecation テストが通る
- cp-library `fix test` が 60 秒以内に exit 0 で完走（プレ deprecation の 45 秒に近い水準）
- `std_numeric_cast_traits.<hash>.fix` ソースファイルに impl が含まれない（Step 1 の効果）
- 生成された警告は全てユーザコード由来（`std_numeric_cast` 由来 0 件）
- `Std::<From>::to_<To>` の本体が単純な `Var(<TraitMethod>)` であること（Step 2 の効果。確認方法：`fix docs --with-private` 等で実装を覗くか、コンパイラから `gv.expr` を dbg!）
- バイナリレベルでは LLVM 最適化により `to_<To>` 経由でも単一キャスト命令になっていること（`fix build --emit-llvm` で確認）

## 参考: 現状の関連コミット

- `2e803da7` — Render `**Deprecated**` callout in generated documentation
- `1e1d3d68` — Surface `DEPRECATED` warnings in `fix check`
- `b1370583` — Slim down deprecation warning output
- `6c6e5f4f` — Migrate stdlib deprecations to the `DEPRECATED[...]` pragma
- `6225fbaf` — Add `DEPRECATED[...]` pragma and extend `FFI_EXPORT` to qualified paths

未コミットの作業ディレクトリ変更（`src/fixstd/stdlib.rs`）は本リファクタの「中間状態（パース後書き換え版）」になっている。最初に `git diff` で現状を確認し、これを下地として A1 改良案へ書き換えるか、いったん退避してフレッシュに作り直すかを判断すること。
