# 引数の構造体のフィールドを特殊化の入り口にする (#450)

## 直すもの

`Iterator` の組み合わせ子は、渡された関数を**自分の構造体のフィールド**に置く。

```fix
type MapIterator i a b = unbox struct { iter : i, f : a -> b };
map = |f, iter| MapIterator { iter : iter, f : f };
```

`closure_specialization` は関数の値を 2 通りの道で追う。引数として渡された関数と、持ち上げられた
ラムダが受け取るキャプチャリストのフィールドに入った関数である。構造体のフィールドはどちらでも
ないので、`f` はクロージャの値のまま残り、要素ごとに間接呼び出しになる。

`arr.to_iter.map(f).fold(0, op)` を最適化し切った後の本体では、同じ関数の中に 2 つの呼び出しが並ぶ。

```
destructure #v2 { .0 -> iter, .1 -> f : U8 -> I64 }
retain f.1 @deeplocal                       // 要素ごとに参照カウント
let struct#58 = struct_make(struct#48, f)   // 要素ごとに作り直し
let app#59 = f(v#45)                        // 間接。map の関数はフィールドに入っている
let app#67 = Main::...closure_lam1#funptr3(#v1, app#59, #v0)   // 直接。fold の演算子は引数
```

## 効き目 (実測)

変換が作る形を Fix で手書きして測った。300,000 要素の `Array U8`、`-O max`、1 要素あたりの命令数。

| 連鎖 | 今 | 変換後 | 手書きループ |
| --- | --- | --- | --- |
| `map` -> `fold` | 14.00 | **0.84** | - |
| `filter` -> `fold` | 20.02 | **12.02** | - |
| `map` -> `filter` -> `fold` | 29.02 | **13.02** | 1.44 |
| `map` -> `filter` -> `to_array` | 33.04 | **15.54** | 11.52 |

`map` は完全に融合して消える。`filter` に 12 命令が残るのは、`FilterIterator::advance` が
条件に合わない要素を飛ばすために**自分自身を呼ぶ**からで、これは関数の置き場とは別の問題である。

## 今の仕組みのうち、この変更が使うもの

**decapturing** は各ラムダをグローバル関数に持ち上げ、捕まえた環境を運ぶ非ボックスの構造体
(キャプチャリスト) を鋳造する。ラムダの値はそのキャプチャリストの値になり、クロージャが要る場所
では持ち上げた関数を部分適用して包み直す。

**特殊化**は、そういう値を受け取るグローバル関数を、受け取る値ごとに複製する。複製の中では
その値を名前で呼べる。どの入り口を複製する価値があるかは `find_specializable_slots` が
プログラム全体で 1 度解き、`Slot { arg, field }` の集合として持つ。

**絞り込み (narrowing)** は、キャプチャリストのフィールドがまた別のキャプチャリストを持つとき、
そのフィールドの型をクロージャからキャプチャリストへ変える。型が変わるので、それを消費する
ラムダも複製される。これがあるので、受け取ったクロージャをそのまま渡すだけのラムダを越えて
複製の鎖が続く。

## パスが見ている本体

`arr.to_iter.map(f).fold(0, op)` に対して、`closure_specialization` の入力は次の形をしている
(`--emit-symbols` の `6.inline_local`)。`advance` は既にインライン展開されている。

```
fold : I64 -> (I64 -> I64 -> I64) -> MapIterator (ArrayIterator U8) U8 I64 -> I64;
fold = |#param| |#v0| |#v1| (
    match (
        let MapIterator {iter: #v2, f: f} = #v1;           // 引数を分解する
        match ( ...ArrayIterator の advance... ) {
            none() => union_make_0(none),
            some((#v3, a)) =>
                let some = Tuple2{0 : MapIterator{iter : #v3, f : f}, 1 : f(a)};   // 組み立てと呼び出し
                union_make_1(some)
        }
    ) {
        none() => #param,
        some((iter, a)) => fold(#v0(a, #param), #v0, iter),                        // 再帰。引数 2
    }
);
```

`MapIterator` の値が本体の中で通る場所は、引数、分解のパターン、組み立て、`Tuple2` のフィールド、
`Option` の payload、`match` が束縛する名前、再帰呼び出しの引数 2 の 7 か所である。

## 設計

### `Slot` の意味を 1 つにする

今の `Slot { arg, field }` は、`field: Some(j)` を「**キャプチャリスト**の j 番目のフィールド」と
読み、`arg` は 0 に固定している。持ち上げられたラムダは第 0 引数でキャプチャリストを受け取るので、
これは「第 0 引数の構造体の j 番目のフィールド」と言っているのと同じである。

意味を**「第 `arg` 引数として届く非ボックス構造体の j 番目のフィールド」**に広げる。キャプチャ
リストはその 1 例になり、場合分けが 1 つ減る。

### 値の木にユーザ構造体の節を足す

`ClosureTree` は「どのラムダか + どのキャプチャフィールドが既知か」を表す。頭を 2 種類にする。

```rust
enum KnownHead {
    // 持ち上げられたラムダ。この値を通る呼び出しは名前で行える。
    Lambda(FullName),
    // ユーザが宣言した構造体の値。フィールドのいくつかの正体が分かっている。
    Struct(Arc<TyCon>),
}
```

複製の鍵、名前に入る要約、停止規則の鍵はすべてこの木から作られているので、頭を広げるだけで
そのまま働く。

### 絞った構造体型を鋳造する

`MapIterator (ArrayIterator U8) U8 I64` のうち `f : U8 -> I64` を `#CapList@lam_f` に替えた型を
`CaptureStruct::new` で鋳造する。プログラムはこの段階で単相なので、鋳造するのは型引数を取らない
構造体でよい。ボックス／非ボックスは元の宣言から引き継ぐ。

`f` が何も捕まえていなければ `#CapList@lam_f` は大きさ 0 なので、絞った構造体は `iter` だけを
持つ。要素ごとの `retain` と作り直しが消えるのはこのためである。

### 複製の本体の型を差し替える

`MapIterator` の値は `Tuple2` のフィールドと `Option` の payload を通る。そこの型も
`Tuple2 #N I64`、`Option (Tuple2 #N I64)` に変わる。`Tuple2` と `Option` は既にある型なので、
新しく鋳造するものは無く、**型注釈の差し替えだけ**で済む。

複製の本体を作るときに、まず本体全体へ次の置換をかける。

- 型が `MapIterator (ArrayIterator U8) U8 I64` に等しいノードを `#N` にする
- 型がそれだった `MakeStruct(MapIterator, ...)` の型構築子を `#N` にする
- 型がそれだった `Pattern::Struct(MapIterator, ...)` の型構築子を `#N` にし、絞ったフィールドの
  パターンの型を `#CapList@lam_f` にする

置換の後で今までどおり本体を歩けば、`f` はキャプチャリストの型を持つ名前として認識され、
`f(a)` は名前での呼び出しになり、`MakeStruct` のフィールドは絞られた値のまま置かれる。

### どこで断るか

上の置換が正しいのは、**その型の値が本体の中に絞られたもの以外に無い**ときに限る。
`find_specializable_slots` の段で次を全部満たすときだけ入り口として登録する。

- 引数の型が構造体で、絞るフィールドの型がクロージャである
- 本体の中でその引数を分解しているのが `Pattern::Struct` であり、束縛された名前が
  `reaches_a_direct_call` を満たす
- 本体の中のその型構築子の `MakeStruct` が、絞るフィールドに同じ値を置いている
- 複製の返り値の型がその型に触れない
- その型を型に持つグローバルが、同じ入り口で複製されるもの以外に無い

最後の条件が、`MapIterator::@f` のような取得関数、他所へ渡す経路、`Array` の要素や union の
payload を通る経路を落とす。`opaque_boundary` のテストが固定しているのはこれらの形で、
この変更の後も断られたままになる。

## 段階

1. `Slot` の意味を「第 `arg` 引数の構造体の j 番目のフィールド」に広げる。振る舞いは変わらない。
2. 値の木の頭を 2 種類にする。振る舞いは変わらない。
3. 絞った構造体型の鋳造と、複製の本体への型の置換を足す。
4. `find_specializable_slots` に構造体フィールドの入り口を足し、断る条件を実装する。
5. 予算と停止規則を、構造体の節が入った木で見直す。

## 測り方

コンパイラは worktree の `target/release/fix`。1 周の費用は
`(I(11) - I(1)) / 10` を `perf stat -e instructions:u` で取る。実装ごとに別のバイナリで測る。
比較の相手は `~/.cache/fixlang-iter-spec/bench-itertest/` の
`a_map_only` / `a_filter_only` / `a_map_filter_fold` / `a_map_filter_toarray2` と、
変換後の形を手書きした `n_map_only` / `n_filter_only` / `n_narrowed_struct` /
`n_map_filter_toarray`。
