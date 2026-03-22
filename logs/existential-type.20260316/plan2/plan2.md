# Plan 2：Opaque Type の脱糖（Desugaring）方式

## 方針概要

plan1では、型チェッカー内部でopaque typeを直接扱う設計だったが、Requireにおけるfixed_tyvarsの制約に根本的な問題があった。

本計画では、**型チェック・型推論の前に opaque type を脱糖（desugar）する**方針を取る。
opaque typeは型チェックの間だけ使い、instantiationの最中あるいはその前に除去する。

### 脱糖の要素

1. **opaque type ごとに型コンストラクタを生成する**
   - generic type variables がその型コンストラクタの型引数になる。
   - opaque type は struct ではない。具体的な内部型は型チェック後に判明する。
   - trait member で opaque type が使われる場合も型コンストラクタを作成する（associated type に変換するだけでは不十分）。

2. **opaque type に関する制約を QualPredScheme / EqualityScheme に直接変換する**
   - trait impl の合成は**行わない**。
   - `?it : Iterator` → `QualPredScheme`（assumed_predsに追加する形式と同等）に変換。
   - `Item ?it = a` → `EqualityScheme`（assumed_eqsに追加する形式と同等）に変換。
   - これは、ユーザが書いた trait impl が TypeCheckContext の assumed_preds / assumed_eqs に変換されるのと同じ仕組み。

3. **式に wrap 関数を挿入する**
   - opaque type `?it` に対して `?it_wrap` を生成する。
   - `?it_wrap` の型は opaque type の制約から導出される。例えば `[?it : Iterator, Item ?it = a]` なら、`?it_wrap : [x : Iterator, Item x = a] x -> ?it a`。`x` は opaque type と同じ制約を持つ新しい型変数。
   - `?it_wrap` に実装は与えない。型チェック専用の構成要素。
   - 型チェック後、`?it_wrap` の domain（`x`）が何に推論されたかを調べれば、opaque type の具体型が分かる。
   - 使用側では unwrap 不要。opaque type のまま扱い、QualPredScheme / EqualityScheme の仮定を通じて trait のメンバを呼べる。
   - ※ 実装時は名前衝突を避けるため、完全修飾名を使う（例：型コンストラクタは `Std::Iterator::repeat::?it`、wrap は `Std::Iterator::repeat::?it_wrap` 等）。

4. **型チェックの実装方針に集中する**
   - opaque type の具体型は考えない。opaque type は型チェック後に消去されるため、具体型の決定は不要。
   - 型チェックが通ることをまず実現する。

---

## use_cases.md の各ユースケースに対する脱糖の検証

### ユースケース1：`repeat`（イテレータ戻り値型の簡略化）

**元のコード**：
```fix
repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
repeat = |x, n| range(0, n).map(|_| x);
```

**脱糖後**：

```fix
// (1) 型コンストラクタ生成
//     ?it a  ← gen_vars の a が型引数

// (2) ?it_wrap の生成
//     ?it_wrap : [x : Iterator, Item x = a] x -> ?it a
//     （実装は与えない。型チェック専用。）

// (3) 制約を QualPredScheme / EqualityScheme に変換
//     QualPredScheme: ?it a : Iterator
//     EqualityScheme: Item (?it a) = a

// (4) 型シグネチャの書き換え
repeat : a -> I64 -> ?it a;

// (5) ?it_wrap を式に挿入（定義側）
repeat = |x, n| ?it_wrap(range(0, n).map(|_| x));

// 使用側では ?it a 型のまま扱う。QualPredScheme / EqualityScheme の仮定により Iterator として使える。
// 例: let iter = repeat("hello", 3);  // iter : ?it String
//      iter.fold(...)                   // Iterator として使える
```

型チェック後、`?it_wrap` の domain `x` が `MapIterator RangeIterator I64 a` に推論される。これが `?it a` の具体型。

**結果**: ✅ 脱糖可能。

### ユースケース2：trait member での使用（ToIter パターン）

**元のコード**：
```fix
trait c : ToIter {
    type Elem c;
    to_iter : [?it : Iterator, Item ?it = ToIter::Elem c] c -> ?it;
}

impl Array a : ToIter {
    type Elem (Array a) = a;
    to_iter = |arr| ArrayIterator { _idx : 0, _arr : arr };
}
```

**脱糖後**：

trait member にopaque typeがある場合も型コンストラクタを作成する。

```fix
// (1) 型コンストラクタ生成
//     ?it c  ← to_iter 自身に専用の gen_vars がないので、trait の型変数 c だけが型引数になる

// (2) ?it_wrap の生成
//     ?it_wrap : [c : ToIter, x : Iterator, Item x = ToIter::Elem c] x -> ?it c
//     （実装は与えない。型チェック専用。）

// (3) 制約を QualPredScheme / EqualityScheme に変換
//     QualPredScheme: ?it c : Iterator
//     EqualityScheme: Item (?it c) = ToIter::Elem c

// (4) trait 定義の書き換え（型シグネチャ）
trait c : ToIter {
    type Elem c;
    to_iter : c -> ?it c;
}

// (5) 各 impl で ?it_wrap を式に挿入（定義側）
impl Array a : ToIter {
    type Elem (Array a) = a;
    to_iter = |arr| ?it_wrap(ArrayIterator { _idx : 0, _arr : arr });
}

// 使用側では ?it c 型のまま扱う。QualPredScheme / EqualityScheme の仮定により Iterator として使える。
// 例: let iter = [1, 2, 3].to_iter;  // iter : ?it (Array I64)
//      iter.fold(...)                  // Iterator として使える
```

型チェック後、`?it_wrap` の domain `x` が `ArrayIterator a` に推論される。これが `?it (Array a)` の具体型。

**検討**：
- 型コンストラクタ `?it c` は `c` に依存する。`Array a` に対しては `?it (Array a)`、他の型に対しては別の内部型を持つ。
- **制約はどの段階で検証されるか？** 各 impl で `?it_wrap` の型推論が通るとき、`to_iter` の戻り値型が `Iterator` を実装し `Item` が `Elem c` と一致することが（QualPredScheme / EqualityScheme を通じて）自動的に検証される。
- `to_iter` の返す値を使う側は `?it c` 型を受け取る。この型に対して `Iterator` の QualPredScheme と `Item` の EqualityScheme が仮定されているので、イテレータとして使える。

**結果**: ✅ 脱糖可能。

### ユースケース3：higher-kinded opaque type

**元のコード**：
```fix
safe_div : [?m : * -> *, ?m : Monad] I64 -> I64 -> ?m I64;
safe_div = |x, y| if y == 0 { none() } else { some(x / y) };
```

**脱糖後**：
```fix
// 型コンストラクタ ?m（カインド * -> *）
// QualPredScheme: ?m : Monad

safe_div : I64 -> I64 -> ?m I64;
safe_div = |x, y| ?m_wrap(if y == 0 { none() } else { some(x / y) });
// ?m_wrap : [x : * -> *, x : Monad] x I64 -> ?m I64
// （実装は与えない。型チェック専用。）
```

型チェック後、`?m_wrap` の domain `x` が `Option` に推論される。これが `?m` の具体型。

**結果**: ✅ 脱糖可能。

### ユースケース4：複数opaque type

**元のコード**：
```fix
zip_with_index : [it_in : Iterator, Item it_in = a, ?it_out : Iterator, Item ?it_out = (I64, a)] it_in -> ?it_out;
zip_with_index = |iter| iter.enumerate;
```

**脱糖後**：
```fix
// 型コンストラクタ ?it_out a
// QualPredScheme: ?it_out a : Iterator
// EqualityScheme: Item (?it_out a) = (I64, a)

zip_with_index : [it_in : Iterator, Item it_in = a] it_in -> ?it_out a;
zip_with_index = |iter| ?it_out_wrap(iter.enumerate);
```

**結果**: ✅ 脱糖可能。

### ユースケース5：実行時条件分岐（非対応）

plan1と同様、❌ 非対応。opaque typeは1つの具体型に解決される必要がある。

---

## 追加ユースケースの検証

### ユースケース6：引数位置のopaque type

```fix
consume_iter : [?it : Iterator, Item ?it = I64] ?it -> I64;
consume_iter = |iter| iter.fold(0, |x, acc| acc + x);
```

**脱糖後**：
```fix
// 型コンストラクタ ?it
// QualPredScheme: ?it : Iterator
// EqualityScheme: Item ?it = I64

consume_iter : ?it -> I64;
consume_iter = |iter| iter.fold(0, |x, acc| acc + x);
```

引数位置の opaque は実質的に通常の trait bound 型変数と同等。?it 型のまま Iterator として使えるので、opaque にする意味が薄い。

**結果**: 技術的には対応可能だが、実用上は通常の型変数で十分。

### ユースケース7：同じ関数を複数回呼び出す（plan1の問題ケース）

```fix
let x = repeat("hello", 3);  // ?it String 型
let y = repeat(42, 3);       // ?it I64 型
```

`?it String` と `?it I64` は異なる型。✅

```fix
let xs = [repeat("a", 3), repeat("b", 3)];  // Array (?it String) 型
```

2つの `repeat` の戻り値は同じ `?it String` 型。✅

型コンストラクタ方式により、型引数が同じなら同じ型、異なれば異なる型。plan1の根本問題が解消。

**結果**: ✅

### ユースケース8：opaque type同士の相互参照

```fix
transform : [?it_out : Iterator, ?coll : Collects,
             Elem ?coll = I64, Iterator ?coll = ?it_out, Item ?it_out = I64]
            I64 -> ?coll;
```

2つのopaque typeが相互に参照。実用上ほぼ不要。

**結果**: ❌ 非対応（切り捨て可能）。

### ユースケース9：opaque typeにequalityなし（predicateのみ）

```fix
to_string_opaque : [?s : ToString] a -> ?s;
to_string_opaque = |x| x.to_string;  // ?s = String
```

**脱糖後**：
```fix
// 型コンストラクタ ?s a
// QualPredScheme: ?s a : ToString

to_string_opaque : a -> ?s a;
to_string_opaque = |x| ?s_wrap(x.to_string);
```

**結果**: ✅ 脱糖可能。

---

## Equality 制約の自由度について

trait impl に変換する方式（旧plan2）では equality の形が制限されたが、
QualPredScheme / EqualityScheme に直接変換する方式では、**plan1と同等の自由度を持つ**。

具体的には、以下の形がすべて許容される：
- `Item ?it = a` — 標準的な形
- `Item ?it = (I64, a)` — 複合型
- `MyAssocTy ?t <t1> ... <tn> = <type>` — 多引数 associated type

ただし、EqualityScheme として妥当な形でなければならない（既存の validate_constraints のチェックに準じる）。

---

## 脱糖処理の詳細

### Phase 0：パース

- `?` で始まる型変数を opaque type としてパースする。
- Scheme に `opaque_tys` フィールドとして保持する。

### Phase 1：型コンストラクタの生成

各グローバル値（または trait member）の Scheme に含まれる opaque type に対して：

- **型コンストラクタを生成する**

  例：`repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it` の場合：

  ```
  型コンストラクタ ?it（カインド * → *）
  型引数：a（元のSchemeのgen_vars）
  ```

  - 名前：`{関数のフルネーム}::?{opaque名}` 例：`Std::Iterator::repeat::?it`
  - wrap 関数：`{関数のフルネーム}::?{opaque名}_wrap` 例：`Std::Iterator::repeat::?it_wrap`
  - 型引数：元の Scheme の gen_vars
  - opaque type は struct ではない。内部表現は型チェック後に決まる。

- **trait member の場合も型コンストラクタを生成する**

  ```
  trait c : ToIter の to_iter に ?it がある場合
  → 型コンストラクタ ToIter::to_iter::?it を生成（型引数：c）
  ```

### Phase 2：制約の変換

opaque type に関する制約を QualPredScheme / EqualityScheme に変換する：

1. **predicate `?it : Iterator`** → QualPredScheme として保持：

   ```
   QualPredScheme {
       gen_vars: [a],  // Scheme の gen_vars
       qual_pred: QualPred {
           predicate: ?it a : Iterator,
           ...
       }
   }
   ```

2. **equality `Item ?it = a`** → EqualityScheme として保持：

   ```
   EqualityScheme {
       gen_vars: [],  // equality のローカル型変数（plan1の「a1...an」に相当する場合あり）
       equality: Item (?it a) = a
   }
   ```

これらはユーザが書いた trait impl が assumed_preds / assumed_eqs に変換されるのと同じ形式。
使用側で instantiate_scheme する際に、これらを仮定として追加する。

### Phase 3：型シグネチャと式の書き換え

1. **型シグネチャ**：opaque type を型コンストラクタに置き換え、opaque制約を除去。

   ```
   // before
   repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it
   // after
   repeat : a -> I64 -> ?it a
   ```

2. **式（定義側）**：戻り値に `?it_wrap` を挿入。

   ```
   // before
   repeat = |x, n| range(0, n).map(|_| x)
   // after
   repeat = |x, n| ?it_wrap(range(0, n).map(|_| x))
   ```

   `?it_wrap` は `[x : Iterator, Item x = a] x -> ?it a` という型を持つ。実装は与えない。
   型チェック後、domain `x` が何に推論されたかで具体型が判明する。

   使用側では unwrap 不要。opaque type のまま扱い、QualPredScheme / EqualityScheme の仮定により trait のメンバを呼べる。

### Phase 4：型チェック

既存の型チェックがそのまま動く。型コンストラクタは通常の型として扱われる。
QualPredScheme / EqualityScheme は使用側で仮定として追加されるので、
使用側で `?it a` を `Iterator` として使える。

`?it_wrap` の domain が何に推論されたかを調べれば opaque type の具体型が判明する。

---

## 未解決事項（TODO）

- **?it_wrap の内部表現**：ExprNode にどう表現するか。新しい variant を追加するか、特殊な関数呼び出しとして表現するか。
- **型コンストラクタの内部表現**：TypeEnv にどう追加するか。struct でも union でもない新しい種類の型。
- **QualPredScheme / EqualityScheme の保持場所**：Scheme に保持するか、グローバルなテーブルに保持するか。使用側で instantiate_scheme する際にどうアクセスするか。
- **trait member の場合の制約伝播**：`?it c : Iterator` の制約は、`to_iter` を使用する全ての箇所で仮定として利用可能でなければならない。impl ごとに異なる具体型になることの整合性。
- **higher-kinded opaque type**：`?m : * -> *` の型コンストラクタのカインド推論。
- **エラーメッセージ**：生成された型コンストラクタ名がエラーに出る場合の可読性。
