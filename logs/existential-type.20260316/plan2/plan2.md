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

2. **opaque type に関する制約を QualPredScheme / EqualityScheme に直接変換し、グローバルな仮定として追加する**
   - trait impl の合成は**行わない**。
   - `?it : Iterator` → `QualPredScheme`（assumed_predsに追加する形式と同等）に変換。
   - `Item ?it = a` → `EqualityScheme`（assumed_eqsに追加する形式と同等）に変換。
   - これは、ユーザが書いた trait impl が TypeCheckContext の assumed_preds / assumed_eqs に変換されるのと同じ仕組み。
   - これらは**グローバルな仮定**として追加する。`?it a : Iterator` は型コンストラクタ `?it` についての普遍的な事実（すべての `a` に対して成立）であり、trait impl がグローバルであるのと同じ構造。`?it` は新規生成された型コンストラクタなので既存の impl と衝突しない。

3. **式に wrap 関数を挿入する**
   - 各関数（または trait member の各 impl）に対して1つの `#wrap` を生成する。
   - `#wrap` の型は **元の関数の実装型全体を domain、opaque化された関数型全体を codomain** とする。
     例：`repeat : a -> I64 -> ?it` なら `#wrap : [x : Iterator, Item x = a] (a -> I64 -> x) -> (a -> I64 -> ?it a)`。
   - 挿入位置は常に **定義の最外側**。実装全体を `#wrap` に渡す。
     例：`repeat = #wrap(|x, n| range(0, n).map(|_| x))`。
   - opaque type が複数ある場合も同様。return type 中の各 opaque type に対応する型変数を domain 側に持つ。
     例：`#wrap : [x : Iterator, Item x = a, y : Iterator, Item y = a] ((a -> Bool) -> Array a -> (x, y)) -> ((a -> Bool) -> Array a -> (?evens a, ?odds a))`。
   - `#wrap` に実装は与えない。型チェック専用の構成要素。
   - 型チェック後、`#wrap` の domain 内の型変数が何に推論されたかを調べれば、opaque type の具体型が分かる。
   - 使用側では unwrap 不要。opaque type のまま扱い、QualPredScheme / EqualityScheme の仮定を通じて trait のメンバを呼べる。
   - ※ 実装時は名前衝突を避けるため、完全修飾名を使う（例：型コンストラクタは `Std::Iterator::repeat::?it`、wrap は `Std::Iterator::repeat::#wrap` 等）。trait member の場合、wrap は impl ごとに生成されるため、impl 型を含む名前を使う（例：`ToIter::to_iter[Array a]::#wrap`）。

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

// (2) #wrap の生成
//     #wrap : [x : Iterator, Item x = a] (a -> I64 -> x) -> (a -> I64 -> ?it a)
//     （実装は与えない。型チェック専用。）

// (3) 制約を QualPredScheme / EqualityScheme に変換（グローバルな仮定として追加）
//     QualPredScheme: ?it a : Iterator
//     EqualityScheme: Item (?it a) = a

// (4) 型シグネチャの書き換え
repeat : a -> I64 -> ?it a;

// (5) #wrap を式に挿入（実装全体を渡す）
repeat = #wrap(|x, n| range(0, n).map(|_| x));

// 使用側では ?it a 型のまま扱う。QualPredScheme / EqualityScheme の仮定により Iterator として使える。
// 例: let iter = repeat("hello", 3);  // iter : ?it String
//      iter.fold(...)                   // Iterator として使える
```

型チェック後、`#wrap` の domain `(a -> I64 -> x)` の `x` が `MapIterator RangeIterator I64 a` に推論される。これが `?it a` の具体型。

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

// (2) #wrap の生成（impl ごとに1つ。impl 型を代入した型を持つ。）
//     impl Array a の場合:
//     ToIter::to_iter[Array a]::#wrap : [x : Iterator, Item x = a] (Array a -> x) -> (Array a -> ?it (Array a))
//     （実装は与えない。型チェック専用。）

// (3) 制約を QualPredScheme / EqualityScheme に変換（グローバルな仮定として追加）
//     QualPredScheme: [c : ToIter] ?it c : Iterator
//     EqualityScheme: Item (?it c) = ToIter::Elem c
//     （EqualityScheme は Qual を持たない。条件 c : ToIter は QualPredScheme 側で扱われる。）

// (4) trait 定義の書き換え（型シグネチャ）
trait c : ToIter {
    type Elem c;
    to_iter : c -> ?it c;
}

// (5) 各 impl で #wrap を式に挿入（実装全体を渡す）
impl Array a : ToIter {
    type Elem (Array a) = a;
    to_iter = #wrap(|arr| ArrayIterator { _idx : 0, _arr : arr });
}

// 使用側では ?it c 型のまま扱う。QualPredScheme / EqualityScheme の仮定により Iterator として使える。
// 例: let iter = [1, 2, 3].to_iter;  // iter : ?it (Array I64)
//      iter.fold(...)                  // Iterator として使える
```

型チェック後、`#wrap` の domain `(Array a -> x)` の `x` が `ArrayIterator a` に推論される。これが `?it (Array a)` の具体型。

**検討**：
- 型コンストラクタ `?it c` は `c` に依存する。`Array a` に対しては `?it (Array a)`、他の型に対しては別の内部型を持つ。
- **制約はどの段階で検証されるか？** 各 impl で `#wrap` の型推論が通るとき、`to_iter` の戻り値型が `Iterator` を実装し `Item` が `Elem c` と一致することが（QualPredScheme / EqualityScheme を通じて）自動的に検証される。
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
// (1) 型コンストラクタ生成
//     ?m  ← カインド * -> *、型引数なし（gen_vars が空）

// (2) #wrap の生成
//     #wrap : [x : * -> *, x : Monad] (I64 -> I64 -> x I64) -> (I64 -> I64 -> ?m I64)
//     （実装は与えない。型チェック専用。）

// (3) 制約を QualPredScheme / EqualityScheme に変換（グローバルな仮定として追加）
//     QualPredScheme: ?m : Monad

// (4) 型シグネチャの書き換え
safe_div : I64 -> I64 -> ?m I64;

// (5) #wrap を式に挿入（実装全体を渡す）
safe_div = #wrap(|x, y| if y == 0 { none() } else { some(x / y) });

// 使用側では ?m I64 型のまま扱う。QualPredScheme の仮定により Monad として使える。
```

型チェック後、`#wrap` の domain `(I64 -> I64 -> x I64)` の `x` が `Option` に推論される。これが `?m` の具体型。

**結果**: ✅ 脱糖可能。

### ユースケース4：実行時条件分岐（非対応）

plan1と同様、❌ 非対応。opaque typeは1つの具体型に解決される必要がある。

---

## 追加ユースケースの検証

### ユースケース5：同じ関数を複数回呼び出す（plan1の問題ケース）

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

### ユースケース6：複数 opaque type（partition）

**元のコード**：
```fix
partition : [?evens : Iterator, Item ?evens = a, 
             ?odds : Iterator, Item ?odds = a]
            (a -> Bool) -> Array a -> (?evens, ?odds);
partition = |pred, arr| 
    (arr.to_iter.filter(pred), arr.to_iter.filter(|x| pred(x).not));
```

**脱糖後**：
```fix
// (1) 型コンストラクタ生成
//     ?evens a, ?odds a  ← gen_vars の a が型引数

// (2) wrap 関数の生成（実装型全体 → opaque化された型全体）
//     #wrap : [x : Iterator, Item x = a, y : Iterator, Item y = a]
//            ((a -> Bool) -> Array a -> (x, y)) -> ((a -> Bool) -> Array a -> (?evens a, ?odds a))
//     （実装は与えない。型チェック専用。）

// (3) 制約を QualPredScheme / EqualityScheme に変換（グローバルな仮定として追加）
//     QualPredScheme: ?evens a : Iterator
//     EqualityScheme: Item (?evens a) = a
//     QualPredScheme: ?odds a : Iterator
//     EqualityScheme: Item (?odds a) = a

// (4) 型シグネチャの書き換え
partition : (a -> Bool) -> Array a -> (?evens a, ?odds a);

// (5) #wrap を式に挿入（実装全体を渡す）
partition = #wrap(|pred, arr|
    (arr.to_iter.filter(pred), 
     arr.to_iter.filter(|x| pred(x).not))
);
```

**結果**: ✅ 脱糖可能。

### ユースケース7：opaque typeにequalityなし（predicateのみ）

```fix
to_string_opaque : [?s : ToString] a -> ?s;
to_string_opaque = |x| x.to_string;  // ?s = String
```

**脱糖後**：
```fix
// 型コンストラクタ ?s a
// QualPredScheme: ?s a : ToString

to_string_opaque : a -> ?s a;
to_string_opaque = #wrap(|x| x.to_string);
```

**結果**: ✅ 脱糖可能。

### ユースケース8：higher-arity associated type（Rebuildable パターン）

**元のコード**：
```fix
trait c : Rebuildable {
    type Elem c;
    type Rebuild c a;  // higher-arity: c に加えて追加の型引数 a を取る
    rebuild : (Elem c -> a) -> c -> Rebuild c a;
}

impl Array a : Rebuildable {
    type Elem (Array a) = a;
    type Rebuild (Array a) b = Array b;
    rebuild = |f, arr| arr.map(f);
}

from_array : [?c : Rebuildable, Elem ?c = a, Rebuild ?c b = Array b]
             Array a -> ?c;
from_array = |arr| arr;
```

`Rebuild ?c b = Array b` の `b` は formal parameter（Scheme の型シグネチャには出現せず、この equality 内でのみ使われる型変数）。

**脱糖後**：
```fix
// (1) 型コンストラクタ生成
//     ?c a  ← gen_vars の a が型引数

// (2) #wrap の生成（関数ごとに1つ）
//     #wrap : [x : Rebuildable, Elem x = a, Rebuild x b = Array b] (Array a -> x) -> (Array a -> ?c a)
//     （実装は与えない。型チェック専用。）

// (3) 制約を QualPredScheme / EqualityScheme に変換（グローバルな仮定として追加）
//     QualPredScheme: ?c a : Rebuildable
//     EqualityScheme: Elem (?c a) = a                    （gen_vars: [a]）
//     EqualityScheme: Rebuild (?c a) b = Array b         （gen_vars: [a, b]）  ← higher-arity

// (4) 型シグネチャの書き換え
from_array : Array a -> ?c a;

// (5) #wrap を式に挿入（実装全体を渡す）
from_array = #wrap(|arr| arr);
```

**使用側**：
```fix
let c = from_array([1, 2, 3]);            // c : ?c I64
let result = c.rebuild(|x| x.to_string);  // Rebuild (?c I64) String
                                           // → EqualityScheme により Array String に簡約
```

**ポイント**：
- `Rebuild (?c a) b = Array b` の EqualityScheme は gen_vars に `b` を持つ。これが higher-arity associated type の特徴。
- `b` は equality 内でのみ量化される formal parameter であり、Scheme の gen_vars（`a`）とは別。
- 型推論時に `Rebuild (?c I64) String` が出現すると、`b = String` として EqualityScheme がマッチし、`Array String` に簡約される。

**結果**: ✅ 脱糖可能。

---

## Equality 制約の自由度について

trait impl に変換する方式（旧plan2）では equality の形が制限されたが、
QualPredScheme / EqualityScheme に直接変換する方式では、**plan1と同等の自由度を持つ**。

具体的には、以下の形がすべて許容される：
- `Item ?it = a` — 標準的な形
- `Item ?it = (I64, a)` — 複合型
- `MyAssocTy ?t <t1> ... <tn> = <type>` — 多引数 associated type

ただし、`<t1> ... <tn>` は**仮引数**でなければならない。すなわち、型変数であり、かつ RHS の `<type>` 以外には出現しないものである必要がある。これは型推論中に equality を使って型を簡約する処理（`reduce_type_by_equality`）で合流性を保つための制約である。本質的な制限ではないかもしれないが、現状の Fix の型推論に存在する制約。

また、EqualityScheme として妥当な形でなければならない（既存の validate_constraints のチェックに準じる）。

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
  - wrap 関数（通常関数）：`{関数のフルネーム}::#wrap` 例：`Std::Iterator::repeat::#wrap`（関数ごとに1つ）
  - wrap 関数（trait member）：`{trait}::{member}[{impl_type}]::#wrap` 例：`ToIter::to_iter[Array a]::#wrap`（impl ごとに1つ）
  - 型引数：元の Scheme の gen_vars
  - opaque type は struct ではない。内部表現は型チェック後に決まる。

- **trait member の場合も型コンストラクタを生成する**

  例：`to_iter : [?it : Iterator, Item ?it = ToIter::Elem c] c -> ?it`（trait `c : ToIter` のメンバ）の場合：

  ```
  型コンストラクタ ?it（カインド * → *）
  型引数：c（trait の型変数。to_iter 自身に専用の gen_vars がないため）
  ```

  - 名前：`{trait}::{member}::?{opaque名}` 例：`ToIter::to_iter::?it`
  - wrap 関数：`{trait}::{member}[{impl_type}]::#wrap` 例：`ToIter::to_iter[Array a]::#wrap`（impl ごとに1つ）
  - 型引数：trait の型変数（ここでは `c`）
  - wrap の型は impl 型を代入した具体的な型になる。例：`ToIter::to_iter[Array a]::#wrap : [x : Iterator, Item x = a] (Array a -> x) -> (Array a -> ?it (Array a))`

### Phase 2：制約の変換

opaque type に関する制約を QualPredScheme / EqualityScheme に変換する：

**グローバル値の場合**（例：`repeat`）：

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
       gen_vars: [a],  // Scheme の gen_vars
       equality: Item (?it a) = a
   }
   ```

**trait member の場合**（例：`to_iter`）：

`to_iter : [?it : Iterator, Item ?it = ToIter::Elem c] c -> ?it`（trait `c : ToIter` のメンバ）の場合：

1. **predicate `?it : Iterator`** → 条件付き QualPredScheme として保持：

   ```
   QualPredScheme {
       gen_vars: [c],  // trait の型変数
       qual_pred: QualPred {
           conditions: [c : ToIter],  // trait 制約が条件になる
           predicate: ?it c : Iterator,
           ...
       }
   }
   ```

2. **equality `Item ?it = ToIter::Elem c`** → EqualityScheme として保持：

   ```
   EqualityScheme {
       gen_vars: [c],  // trait の型変数
       equality: Item (?it c) = ToIter::Elem c
   }
   ```

   ※ EqualityScheme は条件（Qual）を持たない。条件 `c : ToIter` は QualPredScheme 側で扱われる。

**higher-arity の場合**（例：`Rebuild ?c b = Array b`）：

   ```
   EqualityScheme {
       gen_vars: [a, b],  // a は Scheme の gen_vars、b は equality の仮引数
       equality: Rebuild (?c a) b = Array b
   }
   ```

   **実装上の注意**：EqualityScheme の gen_vars には Scheme の gen_vars（`a`）と equality の仮引数（`b`）の両方が入る。ただし、equality の仮引数 `b` は関数の Scheme の gen_vars には含まれない。型コンストラクタ `?c` の型引数も `a` のみで `b` は含まない。`b` は equality 簡約時に instantiate される変数であり、関数の型シグネチャには出現しない。

これらはユーザが書いた trait impl が assumed_preds / assumed_eqs に変換されるのと同じ形式。
グローバルな仮定として追加するので、どのスコープからでも利用可能。

### Phase 3：型シグネチャと式の書き換え

1. **型シグネチャ**：opaque type を型コンストラクタに置き換え、opaque制約を除去。

   グローバル値の場合（`repeat`）：
   ```
   // before
   repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it
   // after
   repeat : a -> I64 -> ?it a
   ```

   trait member の場合（`to_iter`）：
   ```
   // 内部的には member_scheme により c : ToIter が制約に含まれる
   // before
   to_iter : [c : ToIter, ?it : Iterator, Item ?it = ToIter::Elem c] c -> ?it
   // after
   to_iter : [c : ToIter] c -> ?it c
   ```

2. **式（定義側）**：実装全体を wrap に渡す。

   グローバル値の場合（`repeat`）：
   ```
   // #wrap : [x : Iterator, Item x = a] (a -> I64 -> x) -> (a -> I64 -> ?it a)
   // before
   repeat = |x, n| range(0, n).map(|_| x)
   // after
   repeat = #wrap(|x, n| range(0, n).map(|_| x))
   ```

   trait member の場合（`to_iter`、impl ごとに wrap を挿入）：
   ```
   // ToIter::to_iter[Array a]::#wrap : [x : Iterator, Item x = a] (Array a -> x) -> (Array a -> ?it (Array a))
   // before
   impl Array a : ToIter { to_iter = |arr| ArrayIterator { _idx : 0, _arr : arr }; }
   // after
   impl Array a : ToIter { to_iter = #wrap(|arr| ArrayIterator { _idx : 0, _arr : arr }); }
   ```

   opaque type が複数の場合も同様。各 opaque type に対応する型変数を domain 側の実装型全体に持つ単一の wrap 関数を生成：
   ```
   // #wrap : [x : Iterator, Item x = a, y : Iterator, Item y = a]
   //        ((a -> Bool) -> Array a -> (x, y)) -> ((a -> Bool) -> Array a -> (?evens a, ?odds a))
   // before
   partition = |pred, arr| (e1, e2)
   // after
   partition = #wrap(|pred, arr| (e1, e2))
   ```

   wrap 関数の型は「実装全体の型 → opaque化された型」。挿入位置は常に定義の最外側で、式の構造を分析する必要はない。
   wrap 関数に実装は与えない。型チェック後、domain 内の型変数が何に推論されたかで具体型が判明する。

   使用側では unwrap 不要。opaque type のまま扱い、QualPredScheme / EqualityScheme の仮定により trait のメンバを呼べる。

### Phase 4：型チェック

既存の型チェックがそのまま動く。型コンストラクタは通常の型として扱われる。
QualPredScheme / EqualityScheme は使用側で仮定として追加されるので、
使用側で `?it a` を `Iterator` として使える。

### Phase 5：opaque type の具体型の決定と保存

型チェック後、`#wrap` の domain 内の型変数が何に推論されたかを調べることで、opaque type の具体型が判明する。

具体的には：`check_type` 内で `#wrap` の Scheme が `instantiate_scheme`（Require モード）により instantiate されると、`#wrap` の型変数 `x` が新しい型変数に置き換えられる。型チェック（unify）の結果、substitution を通じてその型変数の具体型が決まる。

**グローバル値の場合**（`repeat`）：

`#wrap` の instantiate 後の substitution から、`x` に対応する型変数が `MapIterator (RangeIterator I64) a` 等に解決される。これにより：
```
?it a = MapIterator (RangeIterator I64) a
```
右辺は gen_vars（`a`）と型コンストラクタ・associated type の組み合わせで構成される。

**trait member の場合**（`to_iter`）：

trait member では impl ごとに別の `#wrap` が生成されるため、具体型も impl ごとに異なる（type family 的な構造）：
```
?it (Array a) = ArrayIterator a           ← impl Array a : ToIter から
?it (HashMap k v) = HashMapIterator k v   ← impl HashMap k v : ToIter から
```
グローバル値のように `?it c = ...` と一つの式で書くことはできず、impl ごとに個別の等式が得られる。

#### 具体型の保存

opaque type の具体型情報（「opaque type の型コンストラクタが具体的に何か」）の保持方法は要検討。候補：
- `TypeCheckContext` のメンバとして保持し、`check_type` の呼び出しで更新する（`assert_freshness` で空を検証可能）。
- `check_type` の戻り値として返す。

いずれにせよ、型チェック結果は `save_cache`（`typecheckcache.rs`）によりキャッシュファイルに保存されるので、opaque type の具体型情報もこのキャッシュに含めるべきである。

### Phase 6：インスタンス化処理における opaque type constructor の解消

`instantiate_symbol` の中で `fix_types` が呼ばれる際に、Phase 5 で求めた具体型情報に基づき、opaque type constructor を具体的な型に置き換える。

具体的には、`instantiate_symbol` では：
1. `sym.ty`（要求された型）と型チェック済みの式の型を `unify` する
2. equality 制約も `unify` で解消する
3. `fix_types` で substitution を適用し、すべての型変数を解決する

この `fix_types` の中（またはその前後）で：
- **opaque type constructor の置換**：`?it a` → `MapIterator (RangeIterator I64) a` のように、Phase 5 の情報に基づき opaque type constructor を具体型に置き換える。置換後の型は型変数を含みうるが、opaque type constructor や associated type は含まない。
- **`#wrap` の除去**：`#wrap` は型チェック専用の構成要素であり、実装を持たない。インスタンス化時に `#wrap(expr)` → `expr` のように除去する。`#wrap` は恒等関数として振る舞う（domain と codomain が同じ型に解決されるため）。

---

## 未解決事項（TODO）

- **エラーメッセージ**：生成された型コンストラクタ名がエラーに出る場合の可読性。実装がある程度完了した段階で、opaque type の性質を破るコードをコンパイルし、エラーメッセージを確認・改善する。
- **LSP 対応**：opaque type にホバーしたとき、解決後の具体型が表示される必要がある（例：`?f` にホバーすると `F64` と表示）。

### 解決済み

- **~~#wrap の内部表現~~**：`#wrap` は実際には呼ばれないため、特別な内部表現は不要。仮の実装が必要なら `undefined("")` でよい。
- **~~型コンストラクタの内部表現~~**：opaque type の型コンストラクタは実際にはメモリ上に置かれないため、特別な内部表現は不要。
