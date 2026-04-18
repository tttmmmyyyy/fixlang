# 計画: 型スキームに "fixed variables" 条件を課す (§5.1)

## 背景

"Associated Type Synonyms" (Chakravarty, Keller, Peyton Jones, ICFP '05)
の §5.1 は、すべての型スキーム `e :: ∀ᾱ.ρ` に対し次の well-formed 条件を
課している:

```
ᾱ ∩ Fv(ρ) ⊆ Fixv(ρ)
```

ここで `Fixv`（fixed variables） は次で定義される:

```
Fixv T          = {}              -- 型コンストラクタ
Fixv α          = {α}             -- 型変数
Fixv (τ₁ τ₂)    = Fixv τ₁ ∪ Fixv τ₂
Fixv η          = {}              -- η = 連想型適用（fix しない）
Fixv ((η=τ)⇒ρ)  = Fixv τ ∪ Fixv ρ -- 等式制約: RHS のみ寄与
Fixv (D τ ⇒ ρ)  = Fixv ρ          -- クラス制約: 寄与しない
Fixv (∀α.σ)     = Fixv σ \ {α}
```

直観: 「型変数が fixed」とは、スキーム本体を具体型と単一化したときにその
変数が一意に決まる位置に現れる、ということ。連想型適用 (`Elem c`) の引数は
連想型が一般に非単射なので、決まらない。

論文はクラス宣言内のメソッドシグネチャに対して **別条件** として
「`D β` のクラス宣言中のメソッドシグネチャ `σ ≡ (∀α.π̄ ⇒ τ)` は `β ∈ Fixv σ`」
も課している。論文の形式では `β` はメソッドの `∀` の外側で量化されており、
条件 3 の ᾱ には含まれないため、これは独立した条件。

ただし Fix の実装では、trait メソッドを global value としてスキーム化する
際に [src/ast/traits.rs:381-396](../../src/ast/traits.rs#L381-L396) の
`member_scheme` が述語 `D β` を追加し、`Scheme::generalize` が `β` を
`gen_vars` に取り込む。結果 `β ∈ gen_vars` かつ `β ∈ Fv(body)`（述語経由）
となるので、**Fix の scheme 表現上は条件 3 一本で β に関する条件 2 も検出
される**（β がどこにも fix 位置で現れなければ `β ∉ Fixv(body)` で fire）。

したがって実装上は条件 3 の Fixv check 1 つで、以下をすべてカバーできる:
- β が fix 位置にも非 fix 位置にも一切現れない（述語だけ）
- β が連想型適用の引数としてしか現れない
- 通常のグローバル値で一般化変数が連想型適用の引数にしかない

論文 §3.4 末尾の reject される例:
```
class C a where
  type S a
  op :: S a -> Int    -- `a` は `S a` の中にしか現れない → ambiguous
```

Fix における対応: `trait a : C { type S a; op : S a -> I64 }` は reject
されるべき。

## 現状

- `Scheme::validate_constraints` ([src/ast/types.rs:1805](../../src/ast/types.rs#L1805))
  が制約検証の中心。呼び出し元は [src/ast/program.rs:1585-1600](../../src/ast/program.rs#L1585-L1600)
  で、以下に対して走る:
  - すべての global value の scheme（trait メソッドは
    `create_trait_member_symbols` 経由で global value 化されているので
    この経路で同様に通る）
  - 各 `TraitMemberImpl` の `scm`（ユーザー指定 impl シグネチャ）と
    `scm_via_defn`（trait 定義から導出されたスキーム）
- `validate_constraints` 内の既存チェックは論文 §3 の構造的制約をカバー
  している（述語は `tyvar : Trait` 形／等式 LHS の形状／opaque tyvar 等式
  の引数独立性／同一 LHS の等式重複禁止）。**Fixv 条件のチェックは無い**。
- 一方 [src/ast/traits.rs:897-919](../../src/ast/traits.rs#L897-L919) に
  §5.1 の **弱い版** が既に存在する:
  ```rust
  if !member.qual_ty.ty.contains_tyvar(&trait_defn.type_var) { ... }
  ```
  `contains_tyvar` は `AssocTy` 内部も走査するので、`op : S a -> I64` の
  ような曖昧メソッドは現状通ってしまう（`op : I64` だけ reject される）。
  コメント（907-909 行）自身が "This constraint is weaker than the condition
  mentioned in section 5.1 ... Strengthening this constraint is also an option"
  と明言している。これが今回 Fixv 化・統合する対象。
- [src/ast/traits.rs:921-937](../../src/ast/traits.rs#L921-L937) にはもう 1 つの
  §5.1 制約 `β ∉ Fv π`（trait 型変数はメソッド制約に現れてはならない）が
  ある。**これは Fixv とは独立**なので今回は触らず残す。
- `TypeNode::free_vars` 系は `AssocTy` の引数にも再帰するため、Fixv にその
  まま流用はできない。別ヘルパを用意する必要がある。
- `Scheme::generalize` ([src/ast/types.rs:2129](../../src/ast/types.rs#L2129))
  は opaque tyvar と opaque 等式の仮引数 (`args[1..]`) を `gen_vars` から
  除外済み。よって Fixv チェックを `gen_vars` に対して行っても opaque 関連の
  誤検知は起きない。

### 呼び出し順序（[src/elaboration/mod.rs:56-62](../../src/elaboration/mod.rs#L56-L62)）

```
validate_trait_env()                         ← 現在の弱い unrelated check はここ
create_trait_member_symbols()                ← trait member を global value として登録
validate_global_value_type_constraints()     ← validate_constraints を呼ぶ
```

unrelated check を後者に移して問題ないか調査済み。中間の
`create_trait_member_symbols` は `trait_.member_scheme` を機械的にスキーム化
するだけで依存なし。`validate_trait_env` 内の後続処理（`validate_trait_impl`・
overlapping instance check）も unrelated に依存しない。副作用として、
現在は unrelated エラーがあると `errors.to_result()?` で以後を skip する動作
になっているが、移動後は同一 run 中にもっと多くのエラーを一度に集められる
（診断上むしろ望ましい）。

## 作業ステップ

### Step 1 — `Fixv` ヘルパ追加

[src/ast/types.rs](../../src/ast/types.rs) に以下を追加:

- `TypeNode::fixed_vars_to_set(&self, out: &mut Set<Name>)`
  - `TyVar(α)` → `α` を挿入
  - `TyCon(_)` → 何もしない
  - `TyApp(f, a)` → 両方に再帰
  - `AssocTy(_, _)` → **打ち切り**（引数に再帰しない）

- `Predicate::fixed_vars_to_set(&self, _out)` → 何もしない
  （クラス制約は引数を fix しない）。

- `Equality::fixed_vars_to_set(&self, out)` → RHS (`eq.value`) のみ再帰。
  LHS は連想型適用なので寄与しない。

- `Scheme::fixed_vars(&self) -> Set<Name>` — `self.ty` と各 `Equality` の
  寄与をまとめる便利メソッド。述語は寄与ゼロなので省略。

補足: 本プロジェクトの scheme は既に qualified type 形（トップレベルに 1 つ
だけ `∀`）なので、論文の `Fixv (∀α.σ)` 則をヘルパ内で再現する必要はない。
本体の Fixv を求めて、外側で `gen_vars` と交差を取れば十分。

### Step 2 — `validate_constraints` に曖昧性チェックを追加

[src/ast/types.rs:1805](../../src/ast/types.rs#L1805) の `Scheme::validate_constraints`
末尾（既存チェックの後）に次を追加:

1. `fixed = self.fixed_vars()` を計算。
2. 述語・等式・本体型に free で現れる変数のうち、`gen_vars` に属するものを
   `appearing` として集める（既存 `free_vars_to_vec` の走査と同様だが
   `gen_vars` で絞り込む）。
3. 各 `α ∈ gen_vars` について、`α ∈ appearing` かつ `α ∉ fixed` ならエラー。
   span はその変数が現れる箇所を指す（既存の `free_vars_to_vec_with_span`
   [src/ast/types.rs:1693](../../src/ast/types.rs#L1693) を再利用）。

エラー文言の案（validate_constraints 既存メッセージの「主文 + NOTE」
スタイルに揃える。単一の check で「β が全く現れない」ケースと
「β が連想型下にしか現れない」ケースの両方を fire させるため、
どちらにも適合する汎用文面とする）:

> Type variable `{α}` is not fixed by this type signature, which makes it
> ambiguous.
> NOTE: `{α}` must appear outside of any associated type application.

span は `free_vars_to_vec_with_span` で引いた「`α` の出現箇所のうちの1つ」
を使う（これにより、述語にしか現れないケースでは述語位置、連想型下に
しか現れないケースでは連想型適用位置を具体的に指す）。既存の
`traits.rs:917` の span 精度（`member.qual_ty.ty.get_source()`）と同等以上。

### Step 3 — 旧 unrelated check の削除

[src/ast/traits.rs:897-919](../../src/ast/traits.rs#L897-L919) の
`contains_tyvar` ベースの弱い check を削除する。

- 同ファイルの隣接する `β ∉ Fv π` check
  ([traits.rs:921-937](../../src/ast/traits.rs#L921-L937)) はそのまま残す。
- 対応するテスト `test_unrelated_trait_method` /
  `test_unrelated_trait_method_via_type_alias`
  ([src/tests/test_basic.rs:5216](../../src/tests/test_basic.rs#L5216) 付近)
  は、新しい Fixv エラーメッセージに合わせて期待文言を更新する（どちらの
  ケースも Fixv 条件により引き続き reject されるため、削除ではなく更新）。

### Step 4 — opaque 等式との整合性確認

- `Scheme::generalize` が opaque tyvar と opaque 等式の `args[1..]` を
  `gen_vars` から除外済みなので、チェック対象に入らない。
- opaque 型を使うテストで回帰が出ないことを確認。

### Step 5 — テスト追加

既存のテスト枠組み（言語機能は [src/tests/test_basic.rs](../../src/tests/test_basic.rs)
等）に従って追加。

コンパイル失敗するべき（negative）:
- trait 型変数が連想型の引数にしか現れないメソッド:
  ```
  trait a : C {
    type S a;
    op : S a -> I64;
  }
  ```
- 通常のグローバル値で、一般化変数が連想型の引数にしか現れない例:
  ```
  foo : [c : Collects] Elem c -> I64;
  foo = |_| 0;
  ```
- `scm_via_defn` 経由で曖昧になる最小例（構築可能なら）。

コンパイル通り続けるべき（positive, 回帰防止）:
- [src/fix/std.fix](../../src/fix/std.fix) 内の全 trait 宣言。
- `op : (S a, a) -> I64`（`a` が単独で出現 → fixed）。
- 等式制約で変数を fix する例: `op : [S a = b] S a -> b -> I64` など。

### Step 6 — 副作用の処理

新チェック導入で [src/fix/std.fix](../../src/fix/std.fix) や周辺コードに違反
が出る可能性あり。各ケースで:

- 本当に曖昧なシグネチャなら、ソースを修正する（fix 位置の追加、等式制約
  の追加 等）。§5.1 は健全性・決定可能性の要件なので、チェックを緩めるより
  ソースを直す方針。
- チェック側の過剰検出であれば、Fixv 実装を見直す。

## 実装順序・コミット粒度

1. Step 1 + Step 2 + Step 3 をまとめて 1 commit:
   "Replace weak unrelated-member check with Fixv well-formedness check in
   Scheme::validate_constraints"
   （traits.rs の旧 check 削除と types.rs への統合は同一 commit、既存
   `test_unrelated_trait_method*` の期待文言更新もここで）
2. Step 5 を 1 commit で新規テスト追加。
3. Step 6 は影響ファイル単位で個別 commit（必要時のみ）。

## 未確定事項・リスク

- 既存の Fix コード（特に [src/fix/std.fix](../../src/fix/std.fix)）に
  ambiguous なシグネチャが残っているかは実行するまで分からない。Step 5 の
  規模は未知。まずブランチで入れて回してみる。
- エラー span の質: 初版は「最初に現れる箇所」にするが、将来的には問題を
  起こしている連想型適用の位置を指すよう改善の余地あり。
