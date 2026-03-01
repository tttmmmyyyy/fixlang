# refs / calls テスト計画

## 1. 概要

LSPの「Find All References (refs)」と「Call Hierarchy (calls)」機能のテストケースを網羅的に列挙する。

**用語:**
- **refs** = 「textDocument/references」。あるシンボルのすべての参照箇所を返す。
- **calls** = 「callHierarchy/incomingCalls」「callHierarchy/outgoingCalls」。呼び出し関係を返す。callsはグローバル値・トレイトメンバーのみ対象。
- **カーソル位置（起点）** = ユーザーがカーソルを置く位置。ここからrefsやcallsが起動される。
- **参照場所** = 検索結果として見つかるべき場所。

**検索されるもの（シンボルの種類）:**
1. グローバル値（Global Value）
2. トレイトメンバー（Trait Member）
3. 型（Type: struct / union）
4. ~~型エイリアス（Type Alias）~~ — プログラムチェック処理の早期段階で解決されるため、refs非サポート
5. トレイト（Trait）
6. トレイトエイリアス（Trait Alias）— 型チェック時に内部的に解決されるがASTは編集されないため、refsサポート可能
7. 関連型（Associated Type）— 言語機能としては存在するが、refs / calls での検索は未対応

**参照が見つかる場所:**

*式木内（グローバル値の右辺 / トレイトメンバー実装の右辺）:*
- **変数参照**: 式中のグローバル値・トレイトメンバーの参照 (`helper(x)`, `x.describe`)
- **MakeStruct構文**: 構造体生成構文中の型名 (`Vec2 { x: 1.0, y: 2.0 }`)
- **構造体パターン**: パターンマッチ中の型名 (`let Vec2 { x: x, y: y } = ...`)
- **式の型アノテーション**: 式に付与された型注釈中の型名 (`expr : Vec2`)
- **パターン型アノテーション**: let束縛のパターンに付与された型注釈中の型名 (`let v : Vec2 = ...`)

*型シグネチャ・宣言:*
- **GV型sig**: グローバル値の型シグネチャ（Scheme: 型, トレイト制約, Equality制約）
- **TM定義型sig**: トレイトメンバー定義の型シグネチャ（QualType: 型, トレイト制約, Equality制約）
- **TM実装型sig**: トレイトメンバー実装の型シグネチャ（型, トレイト制約を含む場合がある）
- **impl宣言**: `impl [制約] Type : Trait`のQualPred（型, トレイト）

*定義の右辺:*
- **関連型実装右辺**: `type Elem (Array a) = a;`の右辺の型
- **型定義右辺**: struct/unionのフィールド型
- **型alias右辺**: `type Point = Vec2;`の右辺の型。型エイリアス自体は検索対象外だが、定義の右辺にある型名は検索の起点として使える。
- **trait alias右辺**: `trait Foo = Bar + Baz;`の右辺のトレイト

注: 「変数参照」〜「パターン型アノテーション」は、GV右辺（グローバル値の式木）とTM実装右辺（トレイトメンバー実装の式木）の両方に出現しうる。

---

## 2. 参照場所マトリックス

各シンボル種類がどの場所に出現しうるかの一覧。  
`✓` = 出現しうる（テストすべき）、`-` = 出現しない。

以下のマトリックスでは、式木内の場所を「変数参照」「MakeStruct」「構造体パターン」「式型アノテーション」「パターン型アノテーション」に分離して記載する。  
これらはGV右辺（グローバル値の式木）とTM実装右辺（トレイトメンバー実装の式木）の両方に存在するが、検索ロジックは同一なので統合して記載する。

| 検索されるもの＼検索場所 | 変数参照 | MakeStruct | 構造体パターン | 式Anno | パターン型Anno | GV型sig | TM定義型sig | TM実装型sig | impl宣言 | 関連型実装右辺 | 型定義右辺 | 型alias右辺 | trait alias右辺 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **グローバル値**     | ✓ | - | - | - | - | - | - | - | - | - | - | - | - |
| **トレイトメンバー** | ✓ | - | - | - | - | - | - | - | - | - | - | - | - |
| **型**             | - | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | - |
| ~~**型エイリアス**~~ | - | - | - | - | - | - | - | - | - | - | - | - | - |
| **トレイト**        | - | - | - | - | - | ✓ | ✓ | ✓※3 | ✓ | - | - | - | ✓ |
| **トレイトエイリアス** | - | - | - | - | - | ✓ | ✓ | ✓※3 | ✓ | - | - | - | - |
| **関連型**          | - | - | - | - | - | ✓※2 | ✓※2 | ✓※2 | - | ✓ | - | - | - |

- ※2: 関連型はEquality制約 (`[Item iter = a]`) として出現。refs未対応。
- ※3: TM実装型sigにトレイト制約を書ける（例: `hoge : [b : ToString] I64 -> b -> String;`）。ただし現在の `find_trait_references` 実装では `impl_.member_sigs` のトレイト制約を検索していない（実装の漏れ）。

備考:
- 「変数参照」にはユニオンパターンのコンストラクタ名（`some(v)` の `some` 等）も含む（グローバル値として扱われる）。
- 「MakeStruct」「構造体パターン」は型名（struct/union）のみ出現。
- 型エイリアスはプログラムチェック処理の早期段階でAST上から解決されるため、refsでは検索対象外。
- トレイトエイリアスは型チェック時に内部的に解決されるがAST自体は編集されないため、refsでの検索が可能。

### GV型sig・TM定義型sigのサブカテゴリ

型シグネチャの中でシンボルが出現する位置を細分化：

| サブカテゴリ | 型 | トレイト | トレイトエイリアス | 関連型 |
|---|:---:|:---:|:---:|:---:|
| 関数型中の型名 (`f : Vec2 -> I64`) | ✓ | - | - | - |
| トレイト制約 (`[a : Eq]`) | - | ✓ | ✓ | - |
| Equality制約の右辺 (`[Item c = Vec2]`) | ✓ | - | - | - |
| Equality制約の左辺（関連型名） (`[Item c = a]`) | - | - | - | ✓ |

---

## 3. カーソル位置（検索の起点）一覧

同じシンボルでも、カーソルを置く位置が異なれば、異なるEndNodeが生成される。  
すべてのカーソル位置から正しく検索できることを確認する。

### 3a. グローバル値のカーソル位置

| ID | カーソル位置 | EndNode種類 |
|---|---|---|
| GV-cur-1 | 宣言の左辺 (`helper : I64 -> I64;` の `helper`) | ValueDecl |
| GV-cur-2 | 定義の左辺 (`helper = \|x\| x + 1;` の `helper`) | ValueDecl |
| GV-cur-3 | 型宣言+定義一体の左辺 (`helper : I64 -> I64 = \|x\| x + 1;` の `helper`) | ValueDecl |
| GV-cur-4 | グローバル値の右辺での使用 (`double = helper(x);` の `helper`) | Expr(Var) |
| GV-cur-5 | トレイトメンバー実装の右辺での使用 | Expr(Var) |

### 3b. トレイトメンバーのカーソル位置

| ID | カーソル位置 | EndNode種類 |
|---|---|---|
| TM-cur-1 | トレイト定義内のメンバー宣言の左辺 (`describe : a -> String;` の `describe`) | ValueDecl |
| TM-cur-2 | impl内のメンバー定義の左辺 (`describe = \|v\| ...;` の `describe`) | ValueDecl |
| TM-cur-3 | グローバル値の右辺での呼び出し (`x.describe` の `describe`) | Expr(Var) |
| TM-cur-4 | トレイトメンバー実装の右辺での呼び出し | Expr(Var) |

### 3c. 型のカーソル位置

| ID | カーソル位置 | EndNode種類 |
|---|---|---|
| Ty-cur-1 | 型定義の左辺名 (`type Vec2 = struct { ... };` の `Vec2`) | Type |
| Ty-cur-2 | グローバル値の型シグネチャ中 (`f : Vec2 -> I64;` の `Vec2`) | Type / TypeOrTrait |
| Ty-cur-3 | 式の型アノテーション中 (`x : Vec2` の `Vec2`) | Type |
| Ty-cur-4 | パターンの型アノテーション中 (`let x : Vec2 = ...` の `Vec2`) | Type |
| Ty-cur-5 | MakeStruct構文中 (`Vec2 { x: 1.0, y: 2.0 }` の `Vec2`) | Type |
| Ty-cur-6 | 構造体パターン中 (`let Vec2 { x: x } = ...` の `Vec2`) | Type |
| Ty-cur-7 | 型定義の右辺中（フィールド型） | Type |
| Ty-cur-8 | 型エイリアス定義の右辺中 (`type Point = Vec2;` の `Vec2`) | Type / TypeOrTrait |
| Ty-cur-9 | impl宣言の型 (`impl Vec2 : Trait` の `Vec2`) | Type / TypeOrTrait |
| Ty-cur-10 | TM定義型sig中 | Type / TypeOrTrait |
| Ty-cur-11 | TM実装型sig中 | Type / TypeOrTrait |
| Ty-cur-12 | Equality制約の右辺中 (`[Elem c = Vec2]` の `Vec2`) | Type / TypeOrTrait |
| Ty-cur-13 | 関連型実装の右辺中 (`type Elem (Array a) = I64;` の `I64`) | Type / TypeOrTrait |

### 3d. ~~型エイリアスのカーソル位置~~ （refs非サポート — プログラムチェック早期に解決されるため対象外）

### 3e. トレイトのカーソル位置

| ID | カーソル位置 | EndNode種類 |
|---|---|---|
| Tr-cur-1 | トレイト定義名 (`trait a : Describable { ... }` の `Describable`) | Trait |
| Tr-cur-2 | GV型sigのトレイト制約中 (`[a : Eq]` の `Eq`) | Trait / TypeOrTrait |
| Tr-cur-3 | impl宣言のトレイト (`impl Vec2 : Describable` の `Describable`) | Trait / TypeOrTrait |
| Tr-cur-4 | impl宣言の制約トレイト (`impl [a : Eq] ... : ...` の `Eq`) | Trait / TypeOrTrait |
| Tr-cur-5 | TM定義型sigのトレイト制約中 | Trait / TypeOrTrait |
| Tr-cur-6 | trait alias右辺 (`trait Foo = Bar + Baz;` の `Bar`) | Trait / TypeOrTrait |

### 3f. トレイトエイリアスのカーソル位置

| ID | カーソル位置 | EndNode種類 |
|---|---|---|
| TrA-cur-1 | トレイトエイリアス定義の左辺名 (`trait Printable = ...;` の `Printable`) | Trait |
| TrA-cur-2 | GV型sigのトレイト制約中 (`[a : Printable]` の `Printable`) | Trait / TypeOrTrait |

### 3g. 関連型のカーソル位置（未実装）

| ID | カーソル位置 | EndNode種類 |
|---|---|---|
| AT-cur-1 | トレイト定義内の関連型宣言 (`type Item iter;` の `Item`) | ？ |
| AT-cur-2 | impl内の関連型実装の左辺 (`type Item (Array a) = a;` の `Item`) | ？ |
| AT-cur-3 | Equality制約中 (`[Item iter = a]` の `Item`) | ？ |
| AT-cur-4 | 型中での使用 (`Option (iter, Item iter)` の `Item`) | ？ |

---

## 4. 詳細テストケース一覧

### 4.1 グローバル値

#### GV-1: 宣言の左辺からのrefs / calls

```fix
helper : I64 -> I64;
       // ^ ここにカーソル → refsで helper のすべての参照を返す
helper = |x| x + 1;
double = |x| helper(helper(x));
```
- カーソル: `helper : I64 -> I64;` の `helper`
- 期待refs: 宣言行, 定義行, `double`内の2つの使用箇所
- 期待calls: incoming = `double`, outgoing = なし（宣言からは不明 / 定義に委譲）

#### GV-2: 定義の左辺からのrefs / calls

```fix
helper : I64 -> I64;
helper = |x| x + 1;
       // ^ ここにカーソル
double = |x| helper(helper(x));
```
- カーソル: `helper = |x| x + 1;` の `helper`
- 期待refs: GV-1と同じ
- 期待calls: GV-1と同じ

#### GV-3: 型宣言+値定義が一体のグローバル値の左辺からのrefs / calls

```fix
answer : I64 = 42;
         // ^ ここにカーソル
use_answer : I64;
use_answer = answer + 1;
```
- カーソル: `answer : I64 = 42;` の `answer`
- 期待refs: 定義行（宣言と定義が同一行なので1つ）, `use_answer`内の使用
- 期待calls: incoming = `use_answer`

#### GV-4: グローバル値の右辺での使用箇所からのrefs / calls

```fix
helper : I64 -> I64;
helper = |x| x + 1;
double = |x| helper(helper(x));
                     // ^ ここにカーソル（使用箇所）
```
- カーソル: `double`定義内の `helper` 呼び出し
- 期待refs: GV-1と同じ
- 期待calls: incoming = `double`; outgoing = helperから呼ぶものがあれば

#### GV-5: トレイトメンバー実装の右辺で使われているグローバル値からのrefs / calls

```fix
add_one : I64 -> I64;
add_one = |x| x + 1;

trait a : Process {
    process : a -> I64;
}

impl I64 : Process {
    process = |n| add_one(n);
                   // ^ ここにカーソル
}
```
- カーソル: impl内の `add_one` 呼び出し
- 期待refs: `add_one` の宣言, 定義, impl内の使用
- 期待calls: incoming = `Process::process`（impl内で呼ばれている）

---

### 4.2 トレイトメンバー

#### TM-1: トレイト定義内のメンバー宣言の左辺からのrefs / calls

```fix
trait a : Describable {
    describe : a -> String;
    // ^ ここにカーソル
}

impl I64 : Describable {
    describe = |n| n.to_string;
}

show : [a : Describable] a -> String;
show = |x| x.describe;
```
- カーソル: `describe : a -> String;` の `describe`
- 期待refs: トレイト宣言, impl内の定義, `show`内の使用
- 期待calls: incoming = `show`

#### TM-2: impl内のメンバー定義の左辺からのrefs / calls

```fix
trait a : Describable {
    describe : a -> String;
}

impl I64 : Describable {
    describe = |n| n.to_string;
    // ^ ここにカーソル
}

show : [a : Describable] a -> String;
show = |x| x.describe;
```
- カーソル: impl内の `describe = ...` の `describe`
- 期待refs: TM-1と同じ

#### TM-3: グローバル値の右辺からトレイトメンバー呼び出しのrefs / calls

```fix
trait a : Describable {
    describe : a -> String;
}

impl I64 : Describable {
    describe = |n| n.to_string;
}

show : [a : Describable] a -> String;
show = |x| x.describe;
                // ^ ここにカーソル
```
- カーソル: `show`内の `describe` 呼び出し
- 期待refs: TM-1と同じ
- 期待calls: incoming = `show`

#### TM-4: トレイトメンバー実装の右辺から別のトレイトメンバー呼び出しのrefs / calls

```fix
trait a : Describable {
    describe : a -> String;
}

type Vec2 = struct { x: F64, y: F64 };

impl Vec2 : Describable {
    describe = |v| v.@x.to_string + ", " + v.@y.to_string;
                          // ^ to_string にカーソル
}
```
- カーソル: impl内での `to_string` 呼び出し
- 期待refs: `to_string` のすべての参照（trait定義, 各impl, 使用箇所）
- 注: `to_string`は`ToString`トレイトメンバー

---

### 4.3 型（struct / union）

#### Ty-1: 型定義の左辺名からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };
     // ^ ここにカーソル

origin : Vec2;
origin = Vec2 { x: 0.0, y: 0.0 };

get_x : Vec2 -> F64;
get_x = |Vec2 { x: x, y: _ }| x;
```
- カーソル: `type Vec2 = struct { ... }` の `Vec2`
- 期待refs: 型定義, `origin`の型sig中, `origin`定義のMakeStruct, `get_x`の型sig中, `get_x`定義の構造体パターン

#### Ty-2: グローバル値の型シグネチャ中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

get_x : Vec2 -> F64;
         // ^ ここにカーソル
get_x = |Vec2 { x: x, y: _ }| x;
```
- カーソル: `get_x : Vec2 -> F64;` 中の `Vec2`
- 期待refs: Ty-1と同じ

#### Ty-3: MakeStruct構文中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

origin : Vec2;
origin = Vec2 { x: 0.0, y: 0.0 };
          // ^ ここにカーソル
```
- カーソル: `Vec2 { x: 0.0, y: 0.0 }` の `Vec2`
- 期待refs: Ty-1と同じ

#### Ty-4: 構造体パターン中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

get_x : Vec2 -> F64;
get_x = |Vec2 { x: x, y: _ }| x;
          // ^ ここにカーソル
```
- カーソル: パターン中の `Vec2`
- 期待refs: Ty-1と同じ

#### Ty-5: 式の型アノテーション中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

annotated : Vec2 -> Vec2;
annotated = |v| v : Vec2;
                     // ^ ここにカーソル
```
- カーソル: `v : Vec2` の `Vec2`
- 期待refs: `Vec2`のすべての参照

#### Ty-6: パターンの型アノテーション中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

get_x : Vec2 -> F64;
get_x = |v| (
    let p : Vec2 = v;
            // ^ ここにカーソル
    p.@x
);
```
- カーソル: `let p : Vec2 = v;` の `Vec2`
- 期待refs: `Vec2`のすべての参照

#### Ty-7: 型定義の右辺（フィールド型）中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };
type Line = struct { start: Vec2, end_: Vec2 };
                              // ^ ここにカーソル
```
- カーソル: `Line`の定義内の`Vec2`
- 期待refs: `Vec2`のすべての参照

#### Ty-8: 型エイリアス定義の右辺の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };
type Point = Vec2;
              // ^ ここにカーソル（Vec2の参照）
```
- カーソル: `type Point = Vec2;` の右辺 `Vec2`
- 期待refs: `Vec2`のすべての参照（`Point`の参照ではない）
- 注: 型エイリアス自体（`Point`）は検索対象外だが、右辺の`Vec2`は検索の起点として機能する

#### Ty-9: impl宣言の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

trait a : Describable {
    describe : a -> String;
}

impl Vec2 : Describable {
      // ^ ここにカーソル
    describe = |v| "vec2";
}
```
- カーソル: `impl Vec2 : Describable` の `Vec2`
- 期待refs: `Vec2`のすべての参照

#### Ty-10: トレイトメンバー定義の型シグネチャ中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

trait a : HasVec {
    get_vec : a -> Vec2;
                    // ^ ここにカーソル
}
```
- カーソル: トレイトメンバーの型sig中の `Vec2`
- 期待refs: `Vec2`のすべての参照

#### Ty-11: トレイトメンバー実装の型シグネチャ中の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

trait a : Describable {
    describe : a -> String;
}

impl Vec2 : Describable {
    describe : Vec2 -> String;
                // ^ ここにカーソル
    describe = |v| "vec2";
}
```
- カーソル: impl内メンバー型sig中の `Vec2`
- 期待refs: `Vec2`のすべての参照

#### Ty-12: Equality制約の右辺の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

trait c : Container {
    type Elem c;
    get_first : c -> Option (Elem c);
}

first_vec : [c : Container, Elem c = Vec2] c -> Option Vec2;
                                      // ^ ここにカーソル
first_vec = |c| c.get_first;
```
- カーソル: `Elem c = Vec2` の `Vec2`
- 期待refs: `Vec2`のすべての参照

#### Ty-13: 関連型実装の右辺の型からのrefs

```fix
type Vec2 = struct { x: F64, y: F64 };

trait c : Container {
    type Elem c;
}

type VecArray = struct { data: Array Vec2 };

impl VecArray : Container {
    type Elem VecArray = Vec2;
                          // ^ ここにカーソル
}
```
- カーソル: `type Elem VecArray = Vec2;` の右辺 `Vec2`
- 期待refs: `Vec2`のすべての参照

---

### ~~4.4 型エイリアス~~ （refs非サポート）

型エイリアスはプログラムチェック処理の早期段階でAST上から解決されるため、refsでの検索は対象外。

---

### 4.5 トレイト

#### Tr-1: トレイト定義名からのrefs

```fix
trait a : Describable {
              // ^ ここにカーソル
    describe : a -> String;
}

impl I64 : Describable {
    describe = |n| n.to_string;
}

show : [a : Describable] a -> String;
show = |x| x.describe;
```
- カーソル: `trait a : Describable` の `Describable`
- 期待refs: トレイト定義, impl宣言の`Describable`, `show`型sigの制約中の`Describable`

#### Tr-2: グローバル値の型シグネチャのトレイト制約からのrefs

```fix
trait a : Describable {
    describe : a -> String;
}

show : [a : Describable] a -> String;
                // ^ ここにカーソル
show = |x| x.describe;
```
- カーソル: `[a : Describable]` の `Describable`
- 期待refs: Tr-1と同じ

#### Tr-3: impl宣言のトレイト名からのrefs

```fix
trait a : Describable {
    describe : a -> String;
}

impl I64 : Describable {
                // ^ ここにカーソル
    describe = |n| n.to_string;
}
```
- カーソル: `impl I64 : Describable` の `Describable`
- 期待refs: Tr-1と同じ

#### Tr-4: impl宣言の制約トレイトからのrefs

```fix
trait a : Eq {
    eq : a -> a -> Bool;
}

type Pair a b = struct { fst: a, snd: b };

impl [a : Eq, b : Eq] Pair a b : Eq {
           // ^ ここにカーソル（制約の Eq）
    eq = |lhs, rhs| lhs.@fst == rhs.@fst && lhs.@snd == rhs.@snd;
}
```
- カーソル: impl制約 `[a : Eq, ...]` 中の `Eq`
- 期待refs: `Eq`のすべての参照

#### Tr-5: トレイトメンバー定義の型シグネチャ中のトレイト制約からのrefs

```fix
trait a : Eq {
    eq : a -> a -> Bool;
}

trait c : Container {
    type Elem c;
    has : [Elem c : Eq] Elem c -> c -> Bool;
                    // ^ ここにカーソル
}
```
- カーソル: trait memberの型sig中の `Eq`
- 期待refs: `Eq`のすべての参照

#### Tr-6: トレイトエイリアス定義の右辺からのrefs

```fix
trait a : Describable {
    describe : a -> String;
}

trait Printable = ToString + Describable;
                              // ^ ここにカーソル
```
- カーソル: `trait Printable = ... + Describable;` の `Describable`
- 期待refs: `Describable`のすべての参照

---

### 4.6 トレイトエイリアス

#### TrA-1: トレイトエイリアス定義の左辺からのrefs

```fix
trait a : Describable {
    describe : a -> String;
}

trait Printable = ToString + Describable;
       // ^ ここにカーソル（Printableの定義）

show : [a : Printable] a -> String;
show = |x| x.describe;
```
- カーソル: `trait Printable = ...;` の `Printable`
- 期待refs: 定義, `show`の型sig中の`Printable`
- 注: トレイトエイリアスは型チェック時に内部的に解決されるが、ASTは編集されないためrefsで検索可能。

#### TrA-2: グローバル値の型シグネチャ中のトレイトエイリアスからのrefs

```fix
trait Printable = ToString + Describable;

show : [a : Printable] a -> String;
                // ^ ここにカーソル
show = |x| x.to_string;
```
- カーソル: `[a : Printable]` の `Printable`
- 期待refs: TrA-1と同じ
- 注: 同上。ASTは保持されるのでrefsで検索可能。

---

### 4.7 関連型（未実装）

#### AT-1: トレイト定義内の関連型宣言からのrefs

```fix
trait iter : Iterator {
    type Item iter;
          // ^ ここにカーソル
    advance : iter -> Option (iter, Item iter);
}
```
- カーソル: `type Item iter;` の `Item`
- 期待refs: 宣言, 各impl内の`Item`実装, `advance`型sig中の`Item iter`, GV型sig中のEquality制約`Item iter = a`
- 状態: **未実装**

#### AT-2: impl内の関連型実装の左辺からのrefs

```fix
trait iter : Iterator {
    type Item iter;
    advance : iter -> Option (iter, Item iter);
}

type MyIter = struct { idx: I64 };

impl MyIter : Iterator {
    type Item MyIter = I64;
          // ^ ここにカーソル
    advance = |it| some((MyIter { idx: it.@idx + 1 }, it.@idx));
}
```
- カーソル: `type Item MyIter = I64;` の `Item`
- 期待refs: AT-1と同じ
- 状態: **未実装**

#### AT-3: Equality制約中の関連型名からのrefs

```fix
trait iter : Iterator {
    type Item iter;
    advance : iter -> Option (iter, Item iter);
}

sum_iter : [iter : Iterator, Item iter = I64] iter -> I64;
                              // ^ ここにカーソル（Itemの参照）
sum_iter = |it| ...;
```
- カーソル: `Item iter = I64` の `Item`
- 期待refs: AT-1と同じ
- 状態: **未実装**

#### AT-4: 型中での関連型の使用からのrefs

```fix
trait iter : Iterator {
    type Item iter;
    advance : iter -> Option (iter, Item iter);
                                     // ^ ここにカーソル
}
```
- カーソル: `Item iter` の `Item`（型シグネチャ中の使用）
- 期待refs: AT-1と同じ
- 状態: **未実装**

#### AT-5: 高アリティ関連型（arity 2以上）

```fix
trait n : Nat {
    type Add n m;
          // ^ ここにカーソル
    value : Value n;
}

impl Zero : Nat {
    type Add Zero m = m;
    value = Value { data: 0 };
}

impl [n : Nat] Succ n : Nat {
    type Add (Succ n) m = Succ (Add n m);
                                // ^ Add の参照
    value = ...;
}
```
- カーソル: `type Add n m;` の `Add`
- 期待refs: trait定義内宣言, impl内宣言, 式/型中での使用
- 状態: **未実装**

---

## 5. 追加考慮事項

### 5.1 callsとrefsの違い

- **refs**: すべてのシンボル種類で動作。宣言・定義・使用すべてを返す。
- **calls**: グローバル値とトレイトメンバーのみ。
  - **incoming calls**: `target`を呼び出しているグローバル値を返す。
  - **outgoing calls**: `source`が呼び出しているグローバル値を返す。
  - 注: `SymbolExpr::Method`（トレイトメンバー実装）のoutgoing callsは意図的に未対応。理由: すべての既知のimplを検索すると結果が多すぎるため。

### 5.2 includeDeclaration

refsリクエストには`includeDeclaration`パラメータがある。
- `true`: 宣言・定義の場所も結果に含む。
- `false`: 使用箇所のみ。

これも各テストケースで確認すべき。

### 5.3 複数ファイルにまたがる参照

テストプロジェクトでは、少なくとも2つのファイル（lib.fix + main.fix）を使用し、ファイルをまたいで参照が検索されることを確認する。

### 5.4 名前空間付き参照

Fix では `Lib::helper(x)` のように名前空間を明示する参照が可能。名前空間付き / なしの両方でrefsが動作することを確認する。

### 5.5 型宣言+値定義が一体のグローバル値

`truth : I64 = 42;` のように、型宣言と値定義が同じ行にある場合、`decl_src == defn_src` となるケースがある。refsが重複しないことを確認する。

---

## 6. 完全性チェックリスト

### カーソル位置の網羅性

| シンボル種類 | カーソル位置 | テストID |
|---|---|---|
| グローバル値 | 宣言の左辺 | GV-1 |
| グローバル値 | 定義の左辺 | GV-2 |
| グローバル値 | 型宣言+定義一体の左辺 | GV-3 |
| グローバル値 | GV右辺での使用 | GV-4 |
| グローバル値 | TM実装右辺での使用 | GV-5 |
| トレイトメンバー | trait定義内の宣言左辺 | TM-1 |
| トレイトメンバー | impl内の定義左辺 | TM-2 |
| トレイトメンバー | GV右辺での呼び出し | TM-3 |
| トレイトメンバー | TM実装右辺での呼び出し | TM-4 |
| 型 | 型定義の左辺名 | Ty-1 |
| 型 | GV型sig中 | Ty-2 |
| 型 | MakeStruct構文中 | Ty-3 |
| 型 | 構造体パターン中 | Ty-4 |
| 型 | 式の型アノテーション中 | Ty-5 |
| 型 | パターン型アノテーション中 | Ty-6 |
| 型 | 型定義の右辺中 | Ty-7 |
| 型 | 型alias右辺中 | Ty-8 |
| 型 | impl宣言の型 | Ty-9 |
| 型 | TM定義型sig中 | Ty-10 |
| 型 | TM実装型sig中 | Ty-11 |
| 型 | Equality制約の右辺 | Ty-12 |
| 型 | 関連型実装の右辺 | Ty-13 |
| ~~型エイリアス~~ | ~~（refs非サポート）~~ | — |
| トレイト | trait定義名 | Tr-1 |
| トレイト | GV型sig制約中 | Tr-2 |
| トレイト | impl宣言のトレイト | Tr-3 |
| トレイト | impl宣言の制約トレイト | Tr-4 |
| トレイト | TM定義型sig制約中 | Tr-5 |
| トレイト | trait alias右辺中 | Tr-6 |
| トレイトエイリアス | 定義の左辺名 | TrA-1 |
| トレイトエイリアス | GV型sig制約中 | TrA-2 |
| 関連型 | trait定義内の宣言 | AT-1 🚧 |
| 関連型 | impl内の実装左辺 | AT-2 🚧 |
| 関連型 | Equality制約中 | AT-3 🚧 |
| 関連型 | 型中での使用 | AT-4 🚧 |
| 関連型 | 高アリティ関連型 | AT-5 🚧 |

🚧 = 未実装

### 参照場所の網羅性（マトリックスの各セルが最低1つのテストケースでカバーされているか）

| 検索されるもの＼検索場所 | 変数参照 | MakeStruct | 構造体パターン | 式Anno | パターン型Anno | GV型sig | TM定義型sig | TM実装型sig | impl宣言 | 関連型実装右辺 | 型定義右辺 | 型alias右辺 | trait alias右辺 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| グローバル値     | GV-4,5 | - | - | - | - | - | - | - | - | - | - | - | - |
| トレイトメンバー | TM-3,4 | - | - | - | - | - | - | - | - | - | - | - | - |
| 型             | - | Ty-3 | Ty-4 | Ty-5 | Ty-6 | Ty-2 | Ty-10 | Ty-11 | Ty-9 | Ty-13🚧 | Ty-7 | Ty-8 | - |
| ~~型エイリアス~~ | - | - | - | - | - | - | - | - | - | - | - | - | - |
| トレイト        | - | - | - | - | - | Tr-2 | Tr-5 | ※J | Tr-3,4 | - | - | - | Tr-6 |
| トレイトエイリアス | - | - | - | - | - | TrA-2 | ※H | ※J | ※I | - | - | - | - |
| 関連型          | - | - | - | - | - | AT-3🚧 | AT-4🚧 | ※L🚧 | - | AT-2🚧 | - | - | - |

※H, ※I: トレイトエイリアスのテストケースで、上記テスト一覧に明示していない組み合わせ。
テスト実装時に、トレイトで確認済みのパターンと同等のテストをトレイトエイリアスでも実施すること。
※J: TM実装型sigにトレイト / トレイトエイリアスの制約が書ける場合のテスト。現在の実装では検索漏れあり（要修正）。
※L: TM実装型sigに関連型が出現する場合のテスト（例: `advance : MyIter -> Option (MyIter, Item MyIter);`）。※3と同様、`impl_.member_sigs`の検索漏れの影響を受ける。
