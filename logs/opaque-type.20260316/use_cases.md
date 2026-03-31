# Opaque Type ユースケース検証（TODO 7）

### ユースケース1：イテレータコンビネータの戻り値型の簡略化（最も典型的）

**問題**：イテレータコンビネータを組み合わせると、戻り値型が内部実装に依存した非常に冗長な型になる。

現状（opaque typeなし）：
```fix
repeat : a -> I64 -> MapIterator RangeIterator I64 a;
repeat = |x, n| range(0, n).map(|_| x);
```

提案構文：
```fix
repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
repeat = |x, n| range(0, n).map(|_| x);
```

**検証**：型チェック時に`?it`は`MapIterator RangeIterator I64 a`に解決される。`MapIterator RangeIterator I64 a`は`Iterator`を実装し、`Item (MapIterator RangeIterator I64 a) = a`なので、制約を満たす。✅

### ユースケース2：複数コンビネータの連鎖（型の爆発）

現状（opaque typeなし）：
```fix
doubled_evens : I64 -> MapIterator (FilterIterator RangeIterator I64) I64 I64;
doubled_evens = |n| range(0, n).filter(|x| x % 2 == 0).map(|x| x * 2);
```

提案構文：
```fix
doubled_evens : [?it : Iterator, Item ?it = I64] I64 -> ?it;
doubled_evens = |n| range(0, n).filter(|x| x % 2 == 0).map(|x| x * 2);
```

**検証**：`MapIterator (FilterIterator RangeIterator I64) I64 I64`は`Iterator`を実装し、`Item`は`I64`。制約を満たす。実装を変更（例：`filter`と`map`の順序変更）しても、型シグネチャは変わらない。✅

### ユースケース3：トレイトメソッドでの使用（ToIter パターン）

plan.mdの「他の応用例」で既に示されている。異なるコレクション型に対して、`to_iter`メソッドが異なるイテレータ型を返すケース。

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

**検証**：各impl内で`?it`は具体的なイテレータ型に解決される。`Array a`では`ArrayIterator a`、別のコレクション型では別のイテレータ型になる。これはRustの`fn to_iter(&self) -> impl Iterator<Item = Self::Elem>`に相当。✅

### ユースケース4：higher-kinded opaque type（Monad/Functorの抽象化）

特定のモナドを返すが、呼び出し側にはモナドインターフェースだけ見せたい場合。

```fix
safe_div : [?m : * -> *, ?m : Monad] I64 -> I64 -> ?m I64;
safe_div = |x, y| (
    if y == 0 { none() }
    else { some(x / y) }
);
```

ここで`?m`は`Option`に解決される。

**検証**：`?m`はカインド`* -> *`の不透明型。`Option`は`Monad`を実装しているので制約を満たす。提案設計はカインドシグネチャ（`?m : * -> *`）をサポートしている。✅

### ユースケース5：複数のopaque typeを持つシグネチャ

入力と出力が異なるイテレータ型の場合：

```fix
zip_with_index : [it_in : Iterator, Item it_in = a, ?it_out : Iterator, Item ?it_out = (I64, a)] it_in -> ?it_out;
zip_with_index = |iter| iter.enumerate;
```

**検証**：`?it_out`は`EnumerateIterator it_in`に解決される。入力側の`it_in`は呼び出し側から決まるので通常の型変数、戻り値側のみがopaqueになる。✅

### ユースケース6（非対応ケース）：実行時の条件分岐による型の選択

```fix
// これはopaque typeでは書けない
choose_iter : [?it : Iterator, Item ?it = I64] Bool -> ?it;
choose_iter = |flag| (
    if flag { range(0, 10) }   // RangeIterator
    else { count_up(0).take(10) }  // TakeIterator CountUpIterator
);
```

**検証**：`?it`はコンパイル時に**1つの**具体型に解決される必要がある。`if`の分岐で`RangeIterator`と`TakeIterator CountUpIterator`という異なる型を返すことはできない。これはRustの`impl Trait`でも同様の制限であり、動的ディスパッチ（trait object相当）が必要になるケース。❌（設計上正しい制限）

---

### まとめ

| # | ユースケース | 対応 | 備考 |
|---|---|---|---|
| 1 | イテレータ戻り値型の簡略化 | ✅ | 最も典型的。plan.mdの`repeat`例 |
| 2 | コンビネータ連鎖の型隠蔽 | ✅ | 実装変更に対して型シグネチャが安定する |
| 3 | トレイトメソッド（ToIter） | ✅ | impl毎に異なる具体型に解決 |
| 4 | higher-kinded opaque type | ✅ | カインドシグネチャが必要 |
| 5 | 複数opaque type | ✅ | 入力位置は通常の型変数が適切 |
| 6 | 実行時条件分岐（非対応） | ❌ | 設計上正しい制限。動的ディスパッチが必要 |

提案設計はopaque typeの主要ユースケースをすべてカバーしている。カインドシグネチャ（`?m : * -> *`）をサポートすることでhigher-kindedケースにも対応でき、Rust/Swiftの`impl Trait`/`some`と同等以上の表現力がある。
