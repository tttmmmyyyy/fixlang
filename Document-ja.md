# 目次

- [目次](#目次)
- [チュートリアル](#チュートリアル)
    - [ツールのセットアップ](#ツールのセットアップ)
        - [Fixコンパイラ](#fixコンパイラ)
            - [ビルド済みバイナリを使用](#ビルド済みバイナリを使用)
            - [ソースからビルド](#ソースからビルド)
            - [Dockerイメージを使用](#dockerイメージを使用)
        - [（オプション）VScode拡張機能](#オプションvscode拡張機能)
    - [最初のFixプログラムを実行する](#最初のfixプログラムを実行する)
    - [モジュール](#モジュール)
    - [グローバルな値](#グローバルな値)
    - [名前空間](#名前空間)
    - [型](#型)
    - [式](#式)
    - [let式](#let式)
    - [if式](#if式)
    - [関数適用](#関数適用)
    - [関数定義](#関数定義)
    - [演算子`.`と`$`](#演算子と)
    - [パターン](#パターン)
    - [`loop`、`continue`、`break`関数](#loopcontinuebreak関数)
    - [ユニオン](#ユニオン)
    - [構造体](#構造体)
    - [イテレータ](#イテレータ)
    - [Fixにおけるミュータビリティと参照カウンタ](#fixにおけるミュータビリティと参照カウンタ)
    - [IO（またはモナド）について少し](#ioまたはモナドについて少し)
- [言語と標準ライブラリの詳細](#言語と標準ライブラリの詳細)
    - [ブール値とリテラル](#ブール値とリテラル)
    - [数値とリテラル](#数値とリテラル)
    - [文字列とリテラル](#文字列とリテラル)
    - [配列とリテラル](#配列とリテラル)
    - [ユニットとタプル](#ユニットとタプル)
    - [構造体](#構造体-1)
        - [`@f : S -> F`](#f--s---f)
        - [`set_f : F -> S -> S`](#set_f--f---s---s)
        - [`mod_f : (F -> F) -> S -> S`](#mod_f--f---f---s---s)
        - [`act_f : [f : Functor] (F -> f F) -> S -> f S`](#act_f--f--functor-f---f-f---s---f-s)
    - [ユニオン](#ユニオン-1)
        - [`v : V -> U`](#v--v---u)
        - [`is_v : U -> Bool`](#is_v--u---bool)
        - [`as_v : U -> V`](#as_v--u---v)
        - [`mod_v : (V -> V) -> U -> U`](#mod_v--v---v---u---u)
    - [モジュールとインポート文](#モジュールとインポート文)
    - [名前空間とオーバーロード](#名前空間とオーバーロード)
    - [インポート文の詳細: エンティティのフィルタリング](#インポート文の詳細-エンティティのフィルタリング)
    - [モジュールをインポートするか、必要なエンティティをインポートするか](#モジュールをインポートするか必要なエンティティをインポートするか)
    - [再帰](#再帰)
    - [型注釈](#型注釈)
    - [パターンマッチング](#パターンマッチング)
    - [トレイト](#トレイト)
    - [関連型](#関連型)
    - [トレイトエイリアス](#トレイトエイリアス)
    - [型エイリアス](#型エイリアス)
    - [動的イテレータ](#動的イテレータ)
    - [モナド](#モナド)
        - [モナドとは](#モナドとは)
            - [状態系モナド](#状態系モナド)
            - [失敗系モナド](#失敗系モナド)
            - [シーケンス系モナド](#シーケンス系モナド)
        - [`do`ブロックとモナドのバインド演算子`*`](#doブロックとモナドのバインド演算子)
        - [明示的な`do`ブロックが必要な場合](#明示的なdoブロックが必要な場合)
        - [モナドアクションを`;;`構文で連鎖させる](#モナドアクションを構文で連鎖させる)
        - [Fixのイテレータはモナドではない](#fixのイテレータはモナドではない)
    - [ボックス型とアンボックス型](#ボックス型とアンボックス型)
        - [関数](#関数)
        - [タプルとユニット](#タプルとユニット)
        - [配列](#配列)
        - [構造体](#構造体-2)
        - [ユニオン](#ユニオン-2)
    - [外部関数インターフェース (FFI)](#外部関数インターフェース-ffi)
        - [Fixで外部関数を呼び出す](#fixで外部関数を呼び出す)
        - [Fixの値や関数を外部言語にエクスポートする](#fixの値や関数を外部言語にエクスポートする)
        - [Fixで外部リソースを管理する](#fixで外部リソースを管理する)
        - [外部言語でFixのボックス値の所有権を管理する](#外部言語でfixのボックス値の所有権を管理する)
        - [CからFixの構造体値のフィールドにアクセスする](#cからfixの構造体値のフィールドにアクセスする)
    - [`eval`構文](#eval構文)
    - [演算子](#演算子)
- [コンパイラの機能](#コンパイラの機能)
    - [プロジェクトファイル](#プロジェクトファイル)
    - [依存関係の管理](#依存関係の管理)
    - [設定ファイル](#設定ファイル)
    - [ドキュメントの生成](#ドキュメントの生成)
    - [Language Server Protocol](#language-server-protocol)
        - [ドキュメントコメントでパラメータリストを指定して言語サーバーにヒントを与える](#ドキュメントコメントでパラメータリストを指定して言語サーバーにヒントを与える)
    - [Fixプログラムのデバッグ](#fixプログラムのデバッグ)
- [その他のドキュメント](#その他のドキュメント)


# チュートリアル

## ツールのセットアップ

### Fixコンパイラ

現在、FixコンパイラはmacOS / Linux / Windows（WSL経由）でサポートされています。以下のいずれかの方法でコンパイラを用意できます。

#### ビルド済みバイナリを使用

[Releases](https://github.com/tttmmmyyyy/fixlang/releases/)からビルド済みのコンパイラバイナリをダウンロードできます。
これをダウンロードし、`fix`という名前にして、"/usr/local/bin" などに配置します。

#### ソースからビルド

FixコンパイラはRustで書かれています。Cargoを使うことでソースからコンパイラをビルドすることができます。

1. [Rust](https://www.rust-lang.org/tools/install)をインストールします。
2. LLVM 17.0.xをインストールします。
   - Linux / WSLでは、[LLVM Download Page](https://releases.llvm.org/download.html)からLLVMのビルド済みバイナリをダウンロードできます。
   - macOSでは、`brew install llvm@17`でLLVMを入手できます。
3. LLVMがインストールされているディレクトリを`LLVM_SYS_170_PREFIX`変数に設定します。
   - `brew`でLLVMをインストールした場合、`export LLVM_SYS_170_PREFIX=$(brew --prefix llvm@17)`で設定できます。
4. `git clone https://github.com/tttmmmyyyy/fixlang.git && cd fixlang`。
5. `cargo install --locked --path .`。これにより`fix`コマンドが`~/.cargo/bin`にインストールされます。

#### Dockerイメージを使用

[pt9999](https://github.com/pt9999)のおかげで、[Dockerイメージ](https://hub.docker.com/r/pt9999/fixlang)が利用可能です！

### （オプション）VScode拡張機能

VScodeを使用している場合、以下の拡張機能をインストールすることをお勧めします。

- [構文ハイライト](https://marketplace.visualstudio.com/items?itemName=tttmmmyyyy.fixlangsyntax)
- [言語クライアント](https://marketplace.visualstudio.com/items?itemName=tttmmmyyyy.fixlang-language-client)

## 最初のFixプログラムを実行する

以下は、フィボナッチ数列の最初の30個の数値を計算するFixプログラムです。

```fix
module Main;

calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_size {
            break $ arr
        } else {
            let x = arr.@(idx-1);
            let y = arr.@(idx-2);
            let arr = arr.set(idx, x+y);
            continue $ (idx+1, arr)
        }
    );
    arr
);

main : IO ();
main = (
    let fib = calc_fib(30);
    println("The first 30 numbers of Fibonacci sequence are: ");;
    println $ fib.to_iter.map(to_string).join(", ")
);
```

このプログラムを実行するには、まず、Fixプロジェクト用の作業ディレクトリを作成し、そのディレクトリで`fix init`を実行して、Fixのプロジェクトのテンプレート（"fixproj.toml", "main.fix", "test.fix"）を作成します。
次に、上記のソースコードを"main.fix"にコピーして下さい。

プロジェクトファイル"fixproj.toml"は、Fixコンパイラにプロジェクトの構成やビルド方法を伝えるものです。
`fix init`で作成されるデフォルトのプロジェクトファイルには、以下のように書かれているため、ソースファイル"main.fix"がビルド対象として認識されます。

```toml
[build]
files = ["main.fix"]
```

作業ディレクトリで`fix run`を実行すると、プログラムがコンパイルされ、実行されます。
以下の出力が標準出力に表示されるはずです。

```
The first 30 numbers of Fibonacci sequence are: 
1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040
```

別の方法として、`fix build`を実行すると、コンパイラは実行可能バイナリ（"a.out"）を生成します。`./a.out`で実行できます。

これがFixコンパイラの基本的な使い方です。コンパイラの機能についての詳細は[コンパイラの機能](#コンパイラの機能)を参照してください。

以下では、上記のサンプルプログラムの構文と意味について説明します。

## モジュール

"main.fix"の最初の行はモジュール定義です。

```
module Main;
```

Fixでは、あるソースファイルで定義された値、関数、型、トレイトは一つの「モジュール」にまとめられます。
各ソースファイルは、`module {モジュール名};`によって、そのソースファイルで定義するモジュールの名前を指定する必要があります。

Fixプログラムが実行されると、`Main`モジュールで定義された`main`関数が呼び出されるようになっています。

モジュール名は大文字で始める必要があります。
さらに、そのような文字列をピリオドで連結したもの（例: `Main.Model.Impl`）をモジュール名として使用できます。
これは、モジュールの階層構造を表現するのに便利です。

## グローバルな値

以下の部分は、2つのグローバルな値`calc_fib`と`main`の定義です。

```fix
calc_fib : I64 -> Array I64;
calc_fib = {式A};

main : IO ();
main = {式B};
```

これらの行は次のことを意味します。

- グローバル値`calc_fib`は型`I64 -> Array I64`を持ち、その値は`{式A}`で定義されます。
- グローバル値`main`は型`IO ()`を持ち、その値は`{式B}`で定義されます。

Fixでは、グローバルな値の型を明示的に指定する必要があります。

注: Fixのバージョン1.1.0以降では、次のようにより簡潔に書くことができます。

```fix
calc_fib : I64 -> Array I64 = {式A};
```

## 名前空間

`Array::fill`の`Array`は名前空間です。
名前空間とは名前の住所のようなものであり、同じ名前を持つ2つの値（または型やトレイト、グローバルに定義されるものすべて）を区別するために使用されます。

多くのケースでは、名前空間は省略できます。
実際、標準ライブラリの現在のバージョンでは、`Array::fill(n, 0)`の代わりに単に`fill(n, 0)`と書くこともできます。
これは、コンパイラが文脈から、`fill`と書かれた値は`Array::fill`のことであると推測できるためです。

実際、`fill`の「完全な名前」は（`Array::fill`ではなく）`Std::Array::fill`です。
`Std`は標準ライブラリ用のモジュールです。
モジュールはトップレベルの名前空間として使われます。
`Std::Array::fill`は、モジュール`Std`の中の名前空間`Array`で定義された関数`fill`であるということです。

単に`fill(n, 0)`と書くことができるにも関わらず、サンプルプログラムで`Array::fill(n, 0)`と書いた理由は次の通りです。

- `Array::fill(n, 0)`は、この`fill`関数が`Array`型を作成することを表現しているため、単に`fill(n, 0)`よりも読みやすいと考えられます。
- 将来的に、`Array`以外の名前空間に`fill`という名前の関数が追加される可能性があります。この場合、`fill`という名前が曖昧になり、サンプルプログラムのコンパイルが失敗する可能性があります。

同様に、`calc_fib`関数の「完全な名前」は`Main::calc_fib`です。

## 型

Fixの各値には型があります。数学の言葉を使えば、型は集合であり、Fixの値はその型の要素であると考えることができます。

以下は型の例です。

- `I64`: 64ビット符号付き整数の型。
- `Bool`: 真偽値（`true`と`false`）の型。
- `Array a`: 要素が型`a`を持つ配列の型。`Array`は型コンストラクタと呼ばれ、型に適用されると`Array I64`や`Array Bool`といった型を生成します。`a`は型パラメータと呼ばれます。
- `String`: 文字列の型。
- `I64 -> Array I64`: 整数を受け取り、整数の配列を返す関数の型。
- `()`: ユニット型。この型には`()`と書かれる単一の値があります。
- `(a, b)`: `a`と`b`の値のペアの型。ここで`a`と`b`は型パラメータです。
- `IO a`: 値が文字列の出力やファイルの内容の読み取りなどのI/Oアクションに対応する型。型変数`a`はI/Oアクションによって返される値の型を表します。たとえば、I/Oアクションが標準入力を`String`として読み取る場合（失敗しないと仮定すると）、その型は`IO String`となります。
- `IO ()`: 値を返さないI/Oアクションの型。Fixプログラムの`main`関数の型です。
- `I64 -> Bool -> Array Bool`: これは`I64 -> (Bool -> Array Bool)`と同等で、整数を受け取り、ブール値をブール配列に変換する関数を返す関数の型です。たとえば、長さと初期値からブール配列を生成する関数はこの型を持ちます。Fixには「2変数関数」を扱う文法はありませんが、`a -> b -> c`という型で、`a`と`b`の2つの引数を受け取り、`c`を返す関数を表現できます（実際には「`a`を受け取り`b -> c`型の値を返す関数」ですが）。

Fixでは、特定の型（例: `I64`や`Bool`）や型コンストラクタ（例: `Array`）の名前は大文字で始める必要があります。
小文字で始まる型は型パラメータとして解釈されます。
各型パラメータは、プログラムがコンパイルされるときに特定の型にインスタンス化されます。

## 式

式とは値を記述する文字列です。以下は式の例です。

- `42`: 符号付き64ビット整数として表される数値42を意味するリテラル式。
- `false`, `true`: ブール値を意味するリテラル式（内部的には8ビット整数`0`と`1`として表されます）。
- `[1, 2, 3]`: 要素が`1`、`2`、`3`の整数配列を意味するリテラル式。
- `"Hello World!"`: 文字列リテラル。
- `()`: ユニットリテラル。その型も`()`と書かれ、「ユニット型」と呼ばれます。
- `(1, true)`: タプルリテラル。型`(I64, Bool)`の値を生成します。
- `3 + 5`: `3`と`5`を加算して得られる整数を意味する式。
- `let x = 3 + 5 in x * x`: `3 + 5`を計算し、その結果を`x`と呼びます。その後、`x * x`を計算します。
- `if c { x + y } else { x - y }`: ブール値`c`が`true`の場合、この式の値は`x + y`です。それ以外の場合、この式の値は`x - y`です。
- `f(x)`: 関数`f`を値`x`に適用して得られる値を意味する式。
- `|x| x + 3`: `x`を`x + 3`に変換する関数を意味する式。

## let式

値によってローカル名を定義するには、`let`式を使用します。構文は`let {名前} = {式0} in {式1}`または`let {名前} = {式0}; {式1}`です。

この`;`と`in`は同義です。お好みの方を使用してください。

`{式0}`と`{式1}`を別の行に配置したい場合は、セミコロンを使用する方が適しています。

```
let x = 3;
let y = 5;
x + y
```

`{式0}`が複数行にわたる場合、`{式0}`を括弧でインデントすることが推奨されます。たとえば、

```
let sixty_four = (
    let n = 3 + 5;
    n * n
);
sixty_four + sixty_four
```

（`128`に評価される）は、次のようにも書けます。

```
let sixty_four = 
let n = 3 + 5;
n * n;
sixty_four + sixty_four
```

しかし、後者は可読性が低いため推奨されません。

Fixの`let`式は再帰的な定義を許可しません。たとえば、次のプログラム:

```
use_rec_defn : I64;
use_rec_defn = let x = x + 3 in x * x;
```

はコンパイルできません。一方、次のプログラム:

```
use_rec_defn : I64;
use_rec_defn = (
    let x = 5;
    let x = x + 3;
    x * x
);
```

はコンパイルされますが、`let x = x + 3`の右辺にある名前`x`は、新しいものではなく、前の行で定義された名前`x`（つまり値は`5`）として扱われます。

これは、`let`式を使用してローカルな再帰関数を定義できないことを意味します。
ローカルな再帰関数を定義するには、`fix`組み込み関数を使用してください。

## if式

`if`の構文は次のとおりです: `if {条件} { {式0} } (else|;) { {式1} }`。
ここで、`{式1}`を囲む中括弧は省略可能です。
`{条件}`の型は`Bool`である必要があり、`{式0}`と`{式1}`の型は一致している必要があります。

通常は、`if {条件} { {式0} } else { {式1} }`を使用します。

```
if cond { 
    "cond is true!"
} else {
    "cond is false!"
}
```

「早期リターン」パターンを書く際は、`{式1}`を囲む中括弧を省略するのが便利です。

```
if cache_is_available { "the cached value" };
"a long program which calculates a value, store it into cache, and returns the value."
```

## 関数適用

関数`f`を値`x`に適用するには、`f(x)`と書きます。

```
neg(3) // -3 -- `neg`は`I64`値を受け取り、その負の値を返す組み込み関数です。
```

前述のように、Fixには「2変数関数」や「3変数関数」という型はありません。
その代わり、型`a -> b -> c`（これは`a -> (b -> c)`と等価）を「`a`の値と`b`の値を受け取る2変数関数」のように扱います。

2つの整数を掛け算する「2変数関数」`multiply : I64 -> I64 -> I64`を考えてみましょう。
この場合、`multiply(3) : I64 -> I64`は、与えられた整数に3を掛ける関数です。
したがって、`multiply(3)(5)`は15になります。
この式は`multiply(3, 5)`と書くこともできます。
これは、`f(x, y)`が`f(x)(y)`と等価であるという糖衣構文があるためです。

フィボナッチ数列のプログラムでは、`Array::fill(n, 0)`という式が、2つの値`n`と`0`に対して2変数関数`Array::fill`を呼び出す例です。

特別な文法として、`f()`と書くことは`f(())`、つまり関数`f`をユニット値`()`に適用することを意味します。

## 関数定義

`|{引数}| {本体}`を使用して関数値（他の言語で「ラムダ」や「クロージャ」と呼ばれるものに似ています）を作成できます。
「2変数関数」を定義する際は、`|{引数0}, {引数1}| {本体}`と書くことができます。
これは`|{引数0}| |{引数1}| {本体}`と書くのと等価です。

Fixの関数は、関数定義の外で定義された値を「キャプチャ」することができます。次のプログラムを考えてみましょう。

```
fifteen : I64;
fifteen = (
    let x = 3;
    let add_x = |n| n + x;
    add_x(4) + add_x(5) // (4 + 3) + (5 + 3) = 15
);
```

式`|n| n + x`では、`n`は関数の引数であり、`x`は前の行で定義された名前を参照しています。
関数`add_x`は値`3`を記憶しており、呼び出されたときにそれを使用します。

Fixのすべての値（関数を含む）は不変であるため、関数`add_x`の動作は変更されることはありません。たとえば、

```
fifteen : I64;
fifteen = (
    let x = 3;
    let add_x = |n| n + x;
    let x = 0;
    add_x(4) + add_x(5) // (4 + 3) + (5 + 3) = 15
);
```

この式は依然として15に評価されます。なぜなら、`add_x`は名前`x`が参照する値の変更の影響を受けないからです。

関数の`{本体}`部分が複数行にわたる場合、`{本体}`を括弧で囲い、インデントすることが推奨されます。
例えば、

```
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_size {
            break $ arr
        } else {
            let x = arr.@(idx-1);
            let y = arr.@(idx-2);
            let arr = arr.set(idx, x+y);
            continue $ (idx+1, arr)
        }
    );
    arr
);
```

と書くことが推奨されます。
これを

```
calc_fib = |n| 
let arr = Array::fill(n, 0);
let arr = arr.set(0, 1);
let arr = arr.set(1, 1);
let arr = loop((2, arr), |(idx, arr)|
    if idx == arr.get_size {
        break $ arr
    } else {
        let x = arr.@(idx-1);
        let y = arr.@(idx-2);
        let arr = arr.set(idx, x+y);
        continue $ (idx+1, arr)
    }
);
arr;
```

と書くこともできますが、読みやすさの観点から推奨されません。

## 演算子`.`と`$`

演算子`.`は、値に関数を適用する別の方法です。これは`x.f == f(x)`として定義されます。

演算子`.`の優先順位は、括弧による関数適用よりも低く設定されています。
そのため、関数`method`が型`Param -> Obj -> Result`を持つ場合、`obj.method(arg)`は`obj.(method(arg)) == method(arg)(obj) == method(arg, obj)`と解釈され、`(obj.method)(arg)`とはなりません。

フィボナッチ数列のプログラムでは、以下が演算子`.`の使用例です。

- `arr.get_size`: `get_size`は型`Array a -> I64`の関数で、配列の長さを返します。他の言語のように`arr.get_size()`と書いてはいけません。単に`arr.get_size`と書くだけで、`get_size(arr)`と同じ意味になります。
- `arr.set(0, 1)`: `set`は型`I64 -> a -> Array a -> Array a`の関数で、配列の要素を指定された値に更新します。
- `arr.@(idx-1)`: `@`は型`I64 -> Array a -> a`の関数で、指定されたインデックスの要素を返します。

型`Param0 -> ... -> ParamN -> Obj -> Result`の関数を、N+1個のパラメータを持ち、型`Result`の値を返す型`Obj`の「メソッド」と呼ぶことがあります。
メソッドは、OOP言語のように`obj.method(arg0, ..., argN)`と書いて呼び出すことができます。

関数適用のもう一つの方法は演算子`$`です: `f $ x = f(x)`。
この演算子は右結合です: `f $ g $ x = f(g(x))`。

演算子`$`は括弧を減らすのに便利です。フィボナッチ数列のプログラムでは、以下が演算子`$`の使用例です。

- `continue $ (idx+1, arr)`: タプル値`(idx+1, arr)`に対する`continue`関数の適用。Fixでは、`continue`と`break`は通常の関数であり、構文ではありません。そのため、この式は`continue((idx+1, arr))`または`(idx+1, arr).continue`と書くこともできます。`continue`と`break`関数の詳細な説明は後述します。
- `println $ fib.to_iter.map(to_string).join(", ")`: 文字列`fib.to_iter.map(to_string).join(", ")`に対する`println`関数の適用。`println`関数は型`String -> IO ()`を持つため、文字列に適用すると`IO ()`型の値が生成されます。この式は`println(fib.to_iter.map(to_string).join(", "))`と書くこともできますが、演算子`$`を使用することで長い文字列式の周りの括弧を減らすことができます。

3つの関数適用方法の優先順位は`f(x)` > `x.f` > `f $ x`です。
このため、`obj.method $ arg`と書くことはできません。
これは`method(obj) $ arg == method(obj, arg)`と等価であり、間違った順序で2つの引数に対して`method`を呼び出そうとしています。
一方、`method(arg) $ obj`と書くことは可能であり、これは「`method`を`arg`に適用して型`Obj -> Result`の関数を取得し、それを`obj`に適用する」と読めます。

## パターン

Fixでは、`let`式と`match`式、そして関数式で、構造体やタプルのパターンマッチングを行うことができます。

たとえば、タプル型`(I64, Bool)`の値を受け取り、`(Bool, I64)`の値を返す関数`swap`を定義してみましょう。
パターンを使わない場合、タプルから要素を抽出するための組み込み関数`@0 : (a, b) -> a`および`@1 : (a, b) -> b`を使用して、次のように書けます。

```
swap : (I64, Bool) -> (Bool, I64);
swap = |tuple| (
    let fst = tuple.@0;
    let snd = tuple.@1;
    (snd, fst)
);
```

`let`式でのパターンを使用すると、このプログラムは次のように書けます。

```
swap : (I64, Bool) -> (Bool, I64);
swap = |tuple| (
    let (fst, snd) = tuple;
    (snd, fst)
);
```

あるいは、関数式でのパターンを使用して、次のように書くこともできます。

```
swap : (I64, Bool) -> (Bool, I64);
swap = |(fst, snd)| (snd, fst);
```

注意：
`|(x, y)| ...`と`|x, y| ...`を混同しないでください。
前者はタプルを受け取る関数を定義し、後者は2変数関数を定義します。

## `loop`、`continue`、`break`関数

組み込み関数`loop`は、Fixでループを実装するために使用されます。
ループの継続や中断には、`continue`および`break`関数を使用します。

`loop`, `continue`, `break`の型は次のとおりです。

- `loop : s -> (s -> LoopState s b) -> b`
- `continue : s -> LoopState s b`
- `break : b -> LoopState s b`

`loop`関数は2つの引数を取ります。
ループの初期状態`s0`とループ本体関数`body`です。
`loop`関数は、まず`s0`に対して`body`を呼び出します。
`body`が値`break(r)`を返すと、`loop`関数は終了し、結果として`r`を返します。
`body`が値`continue(s)`を返した場合は、`loop`関数は再び`s`に対して`body`を呼び出します。

フィボナッチ数列のプログラムでは、`loop`関数は次の式で使用されています。

```
loop((2, arr), |(idx, arr)|
    if idx == arr.get_size {
        break $ arr
    } else {
        let x = arr.@(idx-1);
        let y = arr.@(idx-2);
        let arr = arr.set(idx, x+y);
        continue $ (idx+1, arr)
    }
);
```

このループの初期状態は`(2, arr)`です。
ループ本体はタプル型の状態`(idx, arr)`を受け取ります。
ここで、`idx`は次に更新すべき配列のインデックスであり、`arr`はインデックス`0`から`idx-1`までが計算済みになっているフィボナッチ数列の配列です。

`idx`が`arr.get_size`に達した場合、`break $ arr`を返してループを終了します。
そうでない場合は、`idx`番目のフィボナッチ数列の値を計算し、それを`arr`に格納してから、`continue $ (idx+1, arr)`を返してループを継続します。

## ユニオン

では、型`LoopState s b`とは何でしょうか？
これは、2つの型パラメータ`s`と`b`を持つユニオンとして定義されています。

```
type LoopState s b = union { continue : s, break : b };
```

上記の定義は、`LoopState s b`値が型`s`の値または型`b`の値のいずれかを含むことを示しています。
型の値の集合を`|type|`と書くと、`|LoopState s b| = |s| ⨆ |b|`となります。ここで、記号`⨆`は集合の非交差和を表します。

各ユニオン型には、いくつかの基本的なメソッドが自動的に定義されます。たとえば、上記のような`LoopState`の場合、次の関数が名前空間`LoopState`に定義されます。

- `continue : s -> LoopState s b`: 型`s`の値を`LoopState`値に変換します。
- `break : b -> LoopState s b`: 型`b`の値を`LoopState`値に変換します。
- `is_continue : LoopState s b -> Bool`: `LoopState`値が`continue`によって作成されたかどうかを確認します。
- `is_break : LoopState s b -> Bool`: `LoopState`値が`break`によって作成されたかどうかを確認します。
- `as_continue : LoopState s b -> s`: `LoopState`値が`continue`によって作成された場合、型`s`の値を抽出します。それ以外の場合、この関数はプログラムを中止します。
- `as_break : LoopState s b -> s`: `LoopState`値が`break`によって作成された場合、型`b`の値を抽出します。それ以外の場合、この関数はプログラムを中止します。

ユニオンの別の例として、値を「持たない可能性がある」値を表すために使用される`Option`があります。これは次のように定義されています。

```
type Option a = union { none : (), some : a };
```

なお、`Option`の`none`値を作成する場合、`none()`と書く必要があります。なぜなら、`none`は`() -> Option a`という型の関数だからです。`none(())`ではなく`none()`と書けばよい理由は、`f() == f(())`という糖衣構文があるためです。

## 構造体

フィボナッチプログラムの例には登場しませんが、ここでは独自の構造体を定義する方法を説明します。

たとえば、型`I64`のフィールド`price`と型`Bool`のフィールド`sold`を持つ`Product`という構造体を次のように定義できます。

```
type Product = struct { price: I64, sold: Bool };
```

構造体の値は、`{構造体名} { {フィールド名}: {値} }`という構文で構築できます。

```
let product = Product { price: 100, sold: false };
```

ユニオンの場合と同様に、構造体にも自動的に定義されるメソッドがあります。上記の`Product`の場合、次のメソッドが名前空間`Product`に定義されます。

- `@price : Product -> I64` および `@sold : Product -> Bool`
    - `Product`値からフィールドの値を抽出します。
- `set_price : I64 -> Product -> Product` および `set_sold : Bool -> Product -> Product`
    - フィールドを設定することで`Product`値を変更します。
- `mod_price : (I64 -> I64) -> Product -> Product` および `mod_sold : (Bool -> Bool) -> Product -> Product`
    - フィールドに作用する関数によって`Product`値を変更します。

タプルを分解するためにパターンを使用できることはすでに説明しました。
構造体の値を分解するためにもパターンを使用できます。
たとえば、フィールドアクセサ関数`@price : Product -> I64`は次のように再定義できます。

```
get_price : Product -> I64;
get_price = |product| (
    let Product { price: price, sold: sold } = product;
    price
);
```

または、

```
get_price : Product -> I64;
get_price = |Product { price: price, sold: sold }| price;
```

## イテレータ

式`fib.to_iter.map(to_string).join(", ")`について説明します。この式は
- フィボナッチ数列の配列`fib`を整数のイテレータに変換し、
- 各要素に`to_string : I64 -> String`を適用して文字列のイテレータを取得し、
- これらの文字列を`", "`で区切って連結し、
- 結果として`"1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040"`という文字列を得ます。

イテレータは、配列や単方向リストのような、「順にたどる」ことが可能なシーケンス（要素の列）の概念です。
より正確には、イテレータは「現在の状態」をデータとして持ち、次の要素と次の状態を返すメソッド`advance`を持つ型です。

配列や単方向リストのようにすべてのデータを一度にメモリに格納しないため、イテレータはメモリを効率的に使用できます。
また、終わりのないシーケンスを表現することもできます。

Fixでは、イテレータは次のトレイトとして定義されています。

```
// イテレータのトレイト。
// 
// イテレータは「たどる」ことが可能なシーケンス（要素の列）の概念です。
// より正確には、イテレータは「現在の状態」をデータとして持ち、次の要素と次の状態を返すメソッド`advance`を持つ型です。
trait iter : Iterator {
    type Item iter;
    advance : iter -> Option (iter, Item iter);
}
```

ここで、トレイトとは、型が満たすべき特性を表す概念です。
上記の定義は、型`iter`が`Iterator`であるためには、型`Item iter`と状態遷移関数`advance`を持つ必要があることを示しています。

イテレータの簡単な例を見てみましょう。
ある数値から始め、それをカウントアップしていくイテレータは、次の関数で作成されます。

```
// 数値をカウントアップするイテレータを作成します。
// 
// `count_up(start)`は、`start`から始まる無限の数値シーケンスを生成します。
count_up : I64 -> CountUpIterator;
count_up = |start| CountUpIterator { next: start };
```

以下は`CountUpIterator`の定義と`Iterator`トレイトの実装です。

```
type CountUpIterator = unbox struct { next : I64 };

impl CountUpIterator : Iterator {
    type Item CountUpIterator = I64;
    advance = |CountUpIterator { next : next }| some((CountUpIterator { next: next + 1 }, next));
}
```

サンプルプログラムの式`fib.to_iter.map(to_string).join(", ")`では、まず配列が`to_iter`関数によってイテレータに変換されます。
`to_iter`関数の型は次のとおりです。

```
// 配列をイテレータに変換します。
to_iter : Array a -> ArrayIterator a;
```

`ArrayIterator`は配列と現在のインデックスをデータとして保持する型であり、標準ライブラリには`ArrayIterator a : Iterator`の実装が用意されています。

`map`は、関数をイテレータの各要素に適用し、新しいイテレータを生成する関数です。

```
// イテレータに関数をマップします。
// 
// `iter.map(f)`は、`f`を`iter`の各要素に適用するイテレータを返します。
map : [i : Iterator, Item i = a] (a -> b) -> i -> MapIterator i a b;
```

`to_string`は整数を文字列に変換する関数であり、`map(to_string)`によって整数のイテレータが文字列のイテレータに変換されます。

`join`は文字列のイテレータとセパレータを受け取り、文字列を結合する関数です。

```
// 文字列（またはそのイテレータ）をセパレータで結合します。
join : [ss : Iterator, Item ss = String] String -> ss -> String;
```

これで、`fib.to_iter.map(to_string).join(", ")`の動作を理解できると思います。

サンプルプログラムでは、ループを実現するために`loop`関数を導入しました。
他のループの方法としては、ループする範囲のイテレータを作成し、そのイテレータに沿ってループする関数を使用する方法があります。

イテレータに沿ってループする関数の代表例が`fold`です。

```
// イテレータの要素を左から右に畳み込みます。
//
// 概念的には、`[a0, a1, a2, ...].fold(s, op) = s.op(a0).op(a1).op(a2)...`。
fold : [iter : Iterator, Item iter = a] s -> (a -> s -> s) -> iter -> s;
```

`fold`関数は、イテレータの要素から状態更新関数`op(a0)`、`op(a1)`、...を作成し、これらの状態更新関数を順に適用して最終的な状態を計算します。

`fold`を使用すると、サンプルプログラムの`calc_fib`関数は次のように書き換えることができます。

```
calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = Iterator::range(2, n).fold(arr, |idx, arr|
        let x = arr.@(idx-1);
        let y = arr.@(idx-2);
        arr.set(idx, x+y)
    );
    arr
);
```

`fold`はループの途中で中断することはできない点に注意してください。
途中で中断する必要がある場合は、`loop_iter`関数を使用してください。

```
// イテレータの要素をループします。
// 
// この関数は`fold`に似ていますが、より一般的なバージョンです。任意の時点でループを中断することができます。
loop_iter : [iter : Iterator, Item iter = a] s -> (a -> s -> LoopState s s) -> iter -> s;
```

## Fixにおけるミュータビリティと参照カウンタ

Fixの式は、値を記述する文字列にすぎないことを思い出してください。
本質的には「1 + cos(pi/5)^2」のような数学的な式と同じです。
通常の言語で広く使われている「変数の値を変更する」という概念は存在しません。
Fixのすべての値は「不変」です。

たとえば、次のコードを考えてみましょう。

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr0.@(0): " + arr0.@(0).to_string + ".")
);
```

上記のコードは`arr0.@(0): 1.`と出力します。`2`ではありません。
これは、`arr0.set(0, 2)`が「`arr0`の0番目の要素を`2`に変更した別の配列」を表す式であり、「`arr0`の0番目の要素を`2`に更新せよ」という命令ではないためです。

この動作を実現するためには、上記のプログラムの`set`関数は、`arr0`を複製してから、0番目の要素を`2`に更新した新しい配列を返す必要があります。

次に、`calc_fib`の実装を考えてみましょう。

```
calc_fib : I64 -> Array I64;
calc_fib = |n| (
    let arr = Array::fill(n, 0);
    let arr = arr.set(0, 1);
    let arr = arr.set(1, 1);
    let arr = loop((2, arr), |(idx, arr)|
        if idx == arr.get_size {
            break $ arr
        } else {
            let x = arr.@(idx-1);
            let y = arr.@(idx-2);
            let arr = arr.set(idx, x+y);
            continue $ (idx+1, arr)
        }
    );
    arr
);
```

長さNのフィボナッチ数列を計算する最適な時間計算量はO(N)です。
しかし、Fixがループ内の`let arr = arr.set(idx, x+y);`で配列を複製していた場合、各ループステップにO(N)の時間がかかり、合計の時間計算量はO(N^2)になってしまいます。

実際には、上記のプログラムの`set`は配列を複製せず、`calc_fib`は期待通りO(N)時間で動作します。
これは、`set`が、与えられた配列が今後使用されない場合に限り、複製を省略し、与えられた配列をそのまま更新する、という最適化を行っているためです。

次のプログラムを考えてみましょう。

```
main = (
    let arr0 = Array::fill(100, 1);
    let arr1 = arr0.set(0, 2);
    println("arr1.@(0): " + arr1.@(0).to_string + ".")
);
```

（`println`は`arr0`ではなく`arr1`の0番目の要素を出力することに注意してください。）
このプログラムでは、`set`の呼び出しが`arr0`の最後の使用です。
このような場合、`set`は与えられた配列を複製せずに更新します。
`arr0`の変更が観測されることはないため、これはFixの不変性を損なうことはありません。

`calc_fib`関数に戻りましょう。
行`let arr = arr.set(idx, x+y);`では、名前`arr`が再定義され、`set`関数によって返された新しい配列を指すように設定されます。
これにより、この行以降、`set`関数に渡された古い配列が参照されることはありません。
そのため、`set`関数が与えられた配列を複製する必要がないことは明らかであり、実際に複製は行われません。

まとめると、以下のようになります。
- Fixの値は不変であるため、`set : I64 -> a -> Array a -> Array a`関数は基本的に新しい配列を返すと解釈できる。
- ただし、その配列が後で使用されない場合は、複製を省略し、与えられた配列をそのまま更新する。

Fixは値が後で使用される可能性があるかどうかを*参照カウンタ*によって判断します。
Fixはすべてのボックスな値（常にヒープメモリ上に割り当てられ、名前や構造体フィールドによってポインタで参照される値）に参照カウンタを割り当てます。
Fixは参照カウンタを使用してボックスな値への参照の数を追跡します。
参照カウンタが1の場合、その値は「ユニーク」と呼ばれ、それ以外の場合は「共有されている（shared）」と呼ばれます。
便宜上、アンボックスな値は常にユニークであると見なされます。

「ユニーク」「共有されている」という用語を使うと、`set`関数は配列がユニークである場合はその配列を直接変更し、共有されている場合は複製してから変更する、ということになります。

動的計画法のように、`O(1)`時間で配列を変更することに依存するアルゴリズムを実装する場合、ユニークな配列を`set`に渡すことが非常に重要です。
では、`set`に渡す配列がユニークであることをどのように保証すればよいでしょうか？
前述のように、`arr.set(idx, v)`が`arr`の最後の使用である場合、`arr`は`set`の呼び出し時にユニークです（\*）。
特に、`let arr = arr.set(idx, v);`と書くことで、`set`がユニークな配列を受け取ることが保証されます。なぜなら、更新された新しい配列は古い配列の名前を覆い隠してしまうため、`set`の呼び出し後に古い配列が使用されることは決してないからです。

(\*): ただし、`arr`が複数のスレッドから参照されている場合は例外です。

## IO（またはモナド）について少し

サンプルコードの最後の数行を見てみましょう。

```
main : IO ();
main = (
    let fib = calc_fib(30);
    println("The first 30 numbers of Fibonacci sequence are: ");;
    println $ fib.to_iter.map(to_string).join(", ")
);
```

`println : String -> IO ()`は文字列を受け取り、その文字列を画面に出力するIOアクションを生成する関数です。
このコードでは、2つの`println`によって作成された2つのIOアクションがダブルセミコロン構文（`;;`）によって結合され、画面に2行を出力する大きなIOアクションが作成されます。

IOアクションを結合する方法、より一般的にはモナドを結合してより複雑なモナドを作成する方法については、[モナド](#モナド)で説明します。

# 言語と標準ライブラリの詳細

## ブール値とリテラル

ブール値の型は`Bool`であり、ブール値のリテラルは`true`と`false`です。

## 数値とリテラル

数値の型には、`I8`、`I16`、`I32`、`I64`（符号付き整数）、`U8`、`U16`、`U32`、`U64`（符号なし整数）、および`F32`、`F64`（浮動小数点値）があります。

数値リテラルは、小数点を含む場合は浮動小数点リテラルとして解釈され、それ以外の場合は整数リテラルとして解釈されます。
たとえば、`42`は型`I64`の数値リテラルであり、`3.14`は型`F64`の数値リテラルです。

`I64`および`F64`以外の型の数値リテラルを記述するには、リテラルの後にアンダースコアと型名を記述します。
たとえば、`42_I32`は型`I32`の数値リテラルであり、`3.14_F32`は型`F32`の数値リテラルです。

整数リテラルはデフォルトで10進数で表されますが、`0x`プレフィックスを使用して16進数、`0o`プレフィックスを使用して8進数、`0b`プレフィックスを使用して2進数で表すこともできます。
たとえば、`0x2A`は42を表し、`0o52`も42を表します。

10進数の整数リテラルでは、「e」を使用して10のべき乗を表すことができます。
たとえば、`4e2`は400を表します。

シングルクォートで囲まれた文字は、型`U8`の数値リテラルとして解釈されます。
たとえば、`'A'`は65を表します。

さらに、`\n`、`\r`、`\t`、`\0`、`\\`、`\'`は、それぞれ改行、復帰、タブ、ヌル文字、バックスラッシュ、シングルクォートの文字コードを表す型`U8`の数値リテラルとして解釈されます。

浮動小数点リテラルには、小数点の前後に少なくとも1桁の数字が必要です。
たとえば、`1.`や`.1`は有効な浮動小数点リテラルではありません（Cでは有効です）。

## 文字列とリテラル

`String`は文字列を表す型です。内部的には、`U8`のヌル終端配列として表されます。

文字列リテラルはダブルクォートで囲まれた文字列です。
たとえば、`"Hello, world!"`は型`String`の文字列リテラルです。

文字列リテラルでは、`\n`、`\r`、`\t`、`\\`、`\"`は、それぞれ改行、復帰、タブ、バックスラッシュ、ダブルクォートとして解釈されます。

## 配列とリテラル

配列の型は`Array`です。配列リテラルは`["と"]`で囲まれ、各要素は","で区切られます。たとえば、`[1, 2, 3]`です。

## ユニットとタプル

タプルのテキスト名は`Tuple{N}`であり、`N`は自然数（0も含む）です。たとえば、`Tuple2 I64 Bool`は`(I64, Bool)`と同等です。
ユニット型`()`は実際には長さ0のタプル、すなわち`Tuple0`です。

## 構造体

フィールド`f`が型`F`の構造体`S`を定義した場合、次のメソッドが名前空間`S`に定義されます。

### `@f : S -> F`

構造体の値からフィールドの値を抽出します。

### `set_f : F -> S -> S`

構造体の値をフィールドに値を挿入することで変更します。
この関数は、与えられた構造体の値が共有されている場合、それを複製します。

### `mod_f : (F -> F) -> S -> S`

フィールド値に作用することで構造体の値を変更します。
この関数は、与えられた構造体の値が共有されている場合、それを複製します。
この関数の特別な点は、`obj`と`obj.@field`の両方がユニークである場合に`obj.mod_field(f)`を呼び出すと、`f`がユニークなフィールド値を受け取ることが保証されることです。そのため、`obj.mod_field(f)`は`let v = obj.@field; obj.set_field(f(v))`と等価ではありません。

### `act_f : [f : Functor] (F -> f F) -> S -> f S`

構造体値のフィールドに対してファンクター的な操作を実行します。
意味的には、`s.act_f(a)`は`a(s.@f).map(|f| s.set_f(f))`と等価です。
`act_f`の特別な点は、`s`と`s.@f`の両方がユニークである場合に`s.act_f(a)`を呼び出すと、`a`がユニークな値を受け取ることが保証されることです。
`Array::act`のドキュメントも参照してください。

これはHaskellコミュニティで[レンズ](https://hackage.haskell.org/package/lens-5.0.1/docs/Control-Lens-Combinators.html#t:Lens)として知られています。

## ユニオン

型`U`のユニオンを定義し、型`V`のバリアント`v`を持つ場合、次のメソッドが名前空間`U`に定義されます。

### `v : V -> U`

バリアント値からユニオン値を構築します。

### `is_v : U -> Bool`

ユニオン値が指定されたバリアントとして作成されたかどうかを確認します。

### `as_v : U -> V`

ユニオン値をバリアント値に変換します（指定されたバリアントとして作成された場合）。そうでない場合、この関数はプログラムを中止します。

### `mod_v : (V -> V) -> U -> U`

バリアントに作用する関数によってユニオン値を変更します。
`u`と`u`に格納されている値の両方がユニークである場合に`u.mod_v(a)`を呼び出すと、`a`がユニークな値を受け取ることが保証されます。

## モジュールとインポート文

Fixでは、あるソースファイルで定義されたエンティティ（グローバル値、型、トレイト）が集約されて一つのモジュールを形成します。
各ソースファイルは、`module {module_name};`によってモジュールの名前を宣言する必要があります。
モジュール名は大文字で始める必要があります。
モジュール名は、ソースファイルで定義されたエンティティのトップレベル名前空間として使用されます。

他のモジュールを`import {module_name};`でインポートできます。たとえば、2つのソースファイルからなるプログラムを考えてみましょう。

`lib.fix`:
```
module Lib;

module_name : String;
module_name = "Lib";
```

`main.fix`:
```
module Main;

import Lib;

module_name : String;
module_name = "Main";

main : IO ();
main = (
    println $ "This program consists of two modules, `" + Lib::module_name + "` and `" + Main::module_name + "`."
);
```

これらの2つのファイルを同じディレクトリに配置し、`fix run -f main.fix lib.fix`を実行すると、次のように出力されます。

```
This program consists of two modules, `Lib` and `Main`.
```

特別なモジュールが1つあります。それは`Std`です。これは組み込みエンティティのモジュールです。`Std`モジュールはすべてのモジュールから暗黙的にインポートされるため、明示的に`import Std`と書く必要はありません。

## 名前空間とオーバーロード

Fixのエンティティ（グローバル値、型、トレイト）の名前は衝突することができます（オーバーロード）。
ただし、すべてのエンティティは、その完全な名前（名前空間込みの名前）によって一意に区別されなければなりません。

モジュール名は、ソースファイルで定義されたエンティティのトップレベル名前空間として使用されます。
さらに、`namespace TheNameSpace { ... }`によって名前空間を明示的に作成することができます。

名前空間は大文字で始める必要があります。

たとえば、次のプログラムを考えてみましょう。

```
module Main;

namespace BooleanTruth {
    truth : Bool;
    truth = true;
}

namespace IntegralTruth {
    truth : I64;
    truth = 42;
}
```

この場合、`truth`という名前のエンティティが2つ存在します: `Main::BooleanTruth::truth` と `Main::IntegralTruth::truth`。

エンティティの名前空間の接頭辞（またはすべて）を省略すると、Fixはエンティティが使用される時点までに得られた型情報によってその完全な名前を推測しようとします。
たとえば、次のプログラム

```
module Main;

namespace BooleanTruth {
    truth : Bool;
    truth = true;
}

namespace IntegralTruth {
    truth : I64;
    truth = 42;
}

main : IO ();
main = (
    println $ truth.to_string
);
```

は、コンパイルに失敗します。なぜなら、Fixはどの`truth`を使用すべきかを推測できないからです。
一方、次のプログラム

```
module Main;

namespace BooleanTruth {
    truth : Bool;
    truth = true;
}

namespace IntegralTruth {
    truth : I64;
    truth = 42;
}

main : IO ();
main = (
    println $ (0 + truth).to_string
);
```
[プレイグラウンドで実行](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCm5hbWVzcGFjZSBCb29sZWFuVHJ1dGggew0KICAgIHRydXRoIDogQm9vbDsNCiAgICB0cnV0aCA9IHRydWU7DQp9DQoNCm5hbWVzcGFjZSBJbnRlZ3JhbFRydXRoIHsNCiAgICB0cnV0aCA6IEk2NDsNCiAgICB0cnV0aCA9IDQyOw0KfQ0KDQptYWluIDogSU8gKCk7DQptYWluID0gKA0KICAgIHByaW50bG4gJCAoMCArIHRydXRoKS50b19zdHJpbmcNCik7)

はコンパイルできます。なぜなら、Fixは`truth`の型を`I64`型の`0`に加算できるという事実から推測できるからです。

モジュール名にはピリオドを含めることができます。たとえば、`Main.Model.Impl`です。
この場合、`Main.Model.Impl::truth`という値は、`Impl::truth`または`Model.Impl::truth`として参照できます。

## インポート文の詳細: エンティティのフィルタリング

`module {module_name};`と記述することで、モジュール内で定義されたすべてのエンティティがインポートされます。
特定のエンティティのみをインポートしたり、特定のエンティティを除外したりすることも可能です。

たとえば、次のプログラムでは、モジュール`Std`内のすべてのエンティティが暗黙的にインポートされています。
実際、`Std`モジュールから3つの型`Std::IO`、`Std::Tuple0`（これは`()`のテキスト名）、`Std::String`と、シンボル`Std::IO::println`が使用されています。

```
module Main;

main : IO ();
main = println("Hello, World!");
```

実際に使用されるエンティティのみをインポートするには、`Std`を明示的にインポートし、次のように記述する必要があります。

```
module Main;
import Std::{IO, Tuple0, String, IO::println};

main : IO ();
main = println("Hello, World!");
```

さらに`Std::IO::eprintln`をインポートしたい場合は、次のように記述できます。

```
import Std::{IO, Tuple0, String, IO::println, IO::eprintln};
```

または

```
import Std::{IO, Tuple0, String, IO::{println, eprintln}};
```

`Std::IO`名前空間内のすべてのエンティティをインポートしても問題ない場合は、次のように記述できます。

```
module Main;
import Std::{IO, Tuple0, String, IO::*};

main : IO ();
main = println("Hello, World!");
```

別の例を見てみましょう。
`Std`モジュールは`Tuple2`という型を提供していますが、独自の`Tuple2`を定義して使用したいとします。

```
module Main;

type Tuple2 a b = struct { fst : a, snd : b };

impl [a : ToString, b : ToString] Tuple2 a b : ToString {
    to_string = |t| "(" + t.@fst.to_string + ", " + t.@snd.to_string + ")";
}

main : IO ();
main = println $ Tuple2 { fst : "Hello", snd : "World!" }.to_string;
```

上記のコードはコンパイルできません。なぜなら、`Tuple2`という名前の型が2つ存在するためです。

```
error: Type name `Tuple2` is ambiguous. There are `Main::Tuple2`, `Std::Tuple2`.
```

もちろん、`Tuple2`の各出現箇所の前に`Main::`を追加することで、この問題を解決することもできます。
この問題の別の解決策は、`Std`を明示的にインポートし、`Tuple2`を隠すことです。

```
module Main;

import Std hiding Tuple2;

type Tuple2 a b = struct { fst : a, snd : b };

impl [a : ToString, b : ToString] Tuple2 a b : ToString {
    to_string = |t| "(" + t.@fst.to_string + ", " + t.@snd.to_string + ")";
}

main : IO ();
main = println $ Tuple2 { fst : "Hello", snd : "World!" }.to_string;
```

複数のエンティティを隠す場合は、`import Std hiding {symbol0, Type1, Namespace2::*}`のように書くことができます。

## モジュールをインポートするか、必要なエンティティをインポートするか

モジュール全体をインポートする

```
import Lib;
```

のと、必要なエンティティのみをインポートする

```
import Lib::{value0, Type1};
```

のどちらを使用するのが良いでしょうか。

後者は、`Lib`のエンティティを使用する際に毎回`import`文を更新する必要があります。
前者にはこの手間がありません。

一方、後者には保守性の観点での利点があります。
あなたのコードが`value`という値を定義していたとします。
そのあと、ライブラリ`Lib`が更新され、同名の値`value`が追加されたとします。
このとき、`Lib`モジュール全体をインポートしていた場合、あなたのコードでは`value`が曖昧になり、コンパイルエラーになる可能性があります。それに対し、`Lib`から必要なエンティティのみをインポートしていた場合は、自動で`value`をインポートすることはないため、あなたのコードは引き続きコンパイルされます。

現在は、fixコンパイラのLanguage Server Protocol の機能により、エンティティ名の補完操作や、`Unknown name`系のエラーに対するQuick Fixで、自動で`import`文を更新することができます。
よって、競技プログラミングのような一刻も早くコードを書きたい状況を除き、必要なエンティティのみをインポートすることをお勧めします。

## 再帰

多くのプログラミング言語と同様に、再帰的なグローバル関数を作成できます。

```
module Main;

fib : I64 -> I64;
fib = |n| (
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib(n-1) + fib(n-2)
    }
);

main : IO ();
main = print $ fib(30).to_string; // 832040
```
[プレイグラウンドで実行](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCmZpYiA6IEk2NCAtPiBJNjQ7DQpmaWIgPSB8bnwgKA0KICAgIGlmIG4gPT0gMCB7DQogICAgICAgIDANCiAgICB9IGVsc2UgaWYgbiA9PSAxIHsNCiAgICAgICAgMQ0KICAgIH0gZWxzZSB7DQogICAgICAgIGZpYihuLTEpICsgZmliKG4tMikNCiAgICB9DQopOw0KDQptYWluIDogSU8gKCk7DQptYWluID0gcHJpbnQgJCBmaWIoMzApLnRvX3N0cmluZzsgLy8gODMyMDQw)

一方、Fixの`let`バインディングでは再帰的な定義を行うことはできません。ローカルで再帰関数を定義するには、組み込み関数`fix`を使用してください。

## 型注釈

既に述べたように、グローバル値の型は明示的に記述する必要があります。ローカル値にも型を明示的に指定することで、可読性を向上させたり、Fixコンパイラの型推論や名前空間推論を助けることができます。

以下はローカル値に対する型注釈の例です。

```
module Main;

main : IO ();
main = (
    let x = 42 : I64; // 式に対する型注釈。
    let y : I64 = 42; // letバインディングに対する型注釈。
    let f = |v : I64| v * 3; // 関数の引数に対する型注釈。
    
    println $ x.to_string;;
    println $ y.to_string;;
    println $ f(14).to_string;;

    pure()
);
```

## パターンマッチング

パターンマッチングは、構造体（タプルを含む）やユニオンから値を抽出するための構文です。
構造体に対するパターンマッチングは、関数の引数やletバインディングで使用できます。
ユニオンに対するパターンマッチングは、`match`式で使用できます。

```
module Main;

type IntBool = struct { i : I64, b : Bool };

to_pair : IntBool -> (I64, Bool);
to_pair = |IntBool { i : x, b : y }| (x, y); // 関数引数に対するパターンマッチング

main : IO ();
main = (
    let int_bool = IntBool { i : 42, b : true };
    let (i, b) = to_pair(int_bool); // letバインディングでのパターンマッチング
    println $ "(" + i.to_string + ", " + b.to_string + ")"
);
```

```
module Main;

main : IO ();
main = (
    let opt = Option::some(42);

    let x = match opt {
        some(v) => v,
        none(_) => 0
    };
    assert_eq(|_|"", x, 42);;

    let x = match opt {
        some(v) => v,
        none() => 0 // `none(_)`の代わりに`none()`と書くことができます（専用の構文）。
    };
    assert_eq(|_|"", x, 42);;

    let x = match opt {
        some(v) => v,
        _ => 0 // 任意の値は変数パターンでマッチできます。
               // なお、`_`は特別なワイルドカード記号ではなく、単なる名前です。
    };
    assert_eq(|_|"", x, 42);;

    pure()
);
```

## トレイト

トレイトは型の集合です。
トレイトは、そのメンバーである型が実装すべき「メソッド」の集合を定めることによって定義されます。

```
module Main;

// トレイトは型の集合です。
// トレイトは、そのメンバーである型が実装すべき「メソッド」の集合を定めることによって定義されます。

// `Greeter`は型の集合であり、...
trait a : Greeter {
    // メンバーは、型`a`の値を挨拶メッセージに変換する`greeting`メソッドを持ちます。
    greeting : a -> String;
}

// `I64`をトレイト`Greeter`に属するようにし、
impl I64 : Greeter {
    // `greeting`メソッドを以下のように定義します。
    greeting = |n| "Hi! I'm a 64-bit integer " + n.to_string + "!";
}

/*
トレイトは演算子のオーバーロードに使用されます。
例えば、標準ライブラリでは`Eq`トレイトが以下のように定義されています：

```
trait a : Eq {
    eq : a -> a -> Bool
}
```

各式`x == y`は、`Eq::eq(x, y)`の構文糖衣です。
*/

// 別の例として、
type Pair a b = struct { fst: a, snd: b };

// トレイト実装では、`impl`の後の`[]`括弧内で型変数に制約を指定できます。
impl [a : Eq, b : Eq] Pair a b : Eq {
    eq = |lhs, rhs| (
        lhs.@fst == rhs.@fst && lhs.@snd == rhs.@snd
    );
}

// 型シグネチャの前の`[]`括弧内で型変数に制約を指定できます。
search : [a : Eq] a -> Array a -> I64;
search = |elem, arr| loop(0, |idx|
    if idx == arr.get_size { break $ -1 };
    if arr.@(idx) == elem { break $ idx };
    continue $ (idx + 1)
);

// 高階トレイトの定義例。
// すべての型変数はデフォルトでカインド`*`を持ち、高階型変数のカインドは明示的に注釈する必要があります。
trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

// 高階トレイトの実装例。
// `Array`はカインド`* -> *`の型であり、トレイト`MyFunctor`のカインドに一致します。
impl Array : MyFunctor {
    mymap = |f, arr| (
        Array::from_map(arr.get_size, |idx| f(arr.@(idx)))
    );
}

main : IO ();
main = (
    let arr = Array::from_map(6, |x| x); // arr = [0,1,2,...,9].
    let arr = arr.mymap(|x| Pair { fst: x % 2, snd: x % 3 }); // arr = [(0, 0), (1, 1), (0, 2), ...].
    let x = arr.search(Pair { fst: 1, snd: 2}); // 5, x % 2 == 1 かつ x % 3 == 2 を満たす最初の数値。
    println $ x.greeting // "Hi! I'm a 64-bit integer 5!"と出力されるはずです。
);
```
[プレイグラウンドで実行](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCi8vIEEgVHJhaXQgaXMgYSBzZXQgb2YgdHlwZXMuIA0KLy8gQSB0cmFpdCBpcyBkZWZpbmVkIGJ5IGEgc2V0IG9mICJtZXRob2RzIiB0byBiZSBpbXBsZW1lbnRlZCBieSBlYWNoIG1lbWJlciBvZiBpdC4NCg0KLy8gYEdyZWV0ZXJgIGlzIGEgc2V0IG9mIHR5cGVzLCB3aGVyZS4uLg0KdHJhaXQgYSA6IEdyZWV0ZXIgew0KICAgIC8vIHdob3NlIG1lbWJlciBoYXMgYSBtZXRob2QgYGdyZWV0aW5nYCB0aGF0IGNvbnZlcnRzIGEgdmFsdWUgb2YgdHlwZSBgYWAgaW50byBhIGdyZWV0aW5nIG1lc3NhZ2UgZ3JlZXRpbmcuDQogICAgZ3JlZXRpbmcgOiBhIC0%2BIFN0cmluZzsNCn0NCg0KLy8gTGV0IGBJNjRgIGJlbG9uZyB0byB0aGUgdHJhaXQgYE15VG9TdHJpbmdgLCB3aGVyZSANCmltcGwgSTY0IDogR3JlZXRlciB7DQogICAgLy8gdGhlIGBncmVldGluZ2AgbWV0aG9kIGlzIGRlZmluZWQgYXMgZm9sbG93cy4NCiAgICBncmVldGluZyA9IHxufCAiSGkhIEknbSBhIDY0LWJpdCBpbnRlZ2VyICIgKyBuLnRvX3N0cmluZyArICIhIjsNCn0NCg0KLyoNClRyYWl0cyBhcmUgdXNlZCBmb3Igb3ZlcmxvYWRpbmcgb3BlcmF0b3JzLg0KRm9yIGV4YW1wbGUsIGBFcWAgdHJhaXQgaXMgZGVmaW5lZCBpbiBzdGFuZGFyZCBsaWJyYXJ5IGFzIGZvbGxvd3M6IA0KDQpgYGANCnRyYWl0IGEgOiBFcSB7DQogICAgZXEgOiBhIC0%2BIGEgLT4gQm9vbA0KfQ0KYGBgDQoNCkVhY2ggZXhwcmVzc2lvbiBgeCA9PSB5YCBpcyBhIHN5bnRheCBzdWdlciBmb3IgYEVxOjplcSh4LCB5KWAuDQoqLw0KDQovLyBBcyBhbm90aGVyIGV4YW1wbGUsIA0KdHlwZSBQYWlyIGEgYiA9IHN0cnVjdCB7IGZzdDogYSwgc25kOiBiIH07DQoNCi8vIEluIHRoZSB0cmFpdCBpbXBsZW1lbnRhdGlvbiwgeW91IGNhbiBzcGVjaWZ5IGNvbnN0cmFpbnRzIG9uIHR5cGUgdmFyaWFibGVzIGluIGBbXWAgYnJhY2tldCBhZnRlciBgaW1wbGAuDQppbXBsIFthIDogRXEsIGIgOiBFcV0gUGFpciBhIGIgOiBFcSB7DQogICAgZXEgPSB8bGhzLCByaHN8ICgNCiAgICAgICAgbGhzLkBmc3QgPT0gcmhzLkBmc3QgJiYgbGhzLkBzbmQgPT0gcmhzLkBzbmQNCiAgICApOw0KfQ0KDQovLyBZb3UgY2FuIHNwZWNpZnkgY29uc3RyYWludHMgb24gdHlwZSB2YXJpYWJsZXMgaW4gdGhlIGBbXWAgYnJhY2tldCBiZWZvcmUgYSB0eXBlIHNpZ25hdHVyZS4NCnNlYXJjaCA6IFthIDogRXFdIGEgLT4gQXJyYXkgYSAtPiBJNjQ7DQpzZWFyY2ggPSB8ZWxlbSwgYXJyfCBsb29wKDAsIHxpZHh8DQogICAgaWYgaWR4ID09IGFyci5nZXRfc2l6ZSB7IGJyZWFrICQgLTEgfTsNCiAgICBpZiBhcnIuQChpZHgpID09IGVsZW0geyBicmVhayAkIGlkeCB9Ow0KICAgIGNvbnRpbnVlICQgKGlkeCArIDEpDQopOw0KDQovLyBBbiBleGFtcGxlIG9mIGRlZmluaW5nIGhpZ2hlci1raW5kZWQgdHJhaXQuDQovLyBBbGwgdHlwZSB2YXJpYWJsZSBoYXMga2luZCBgKmAgYnkgZGVmYXVsdCwgYW5kIGFueSBraW5kIG9mIGhpZ2hlci1raW5kZWQgdHlwZSB2YXJpYWJsZSBuZWVkIHRvIGJlIGFubm90ZWQgZXhwbGljaXRseS4NCnRyYWl0IFtmIDogKi0%2BKl0gZiA6IE15RnVuY3RvciB7DQogICAgbXltYXAgOiAoYSAtPiBiKSAtPiBmIGEgLT4gZiBiOw0KfQ0KDQovLyBBbiBleGFtcGxlIG9mIGltcGxlbWVudGluZyBoaWdoZXIta2luZGVkIHRyYWl0Lg0KLy8gYEFycmF5YCBpcyBhIHR5cGUgb2Yga2luZCBgKiAtPiAqYCwgc28gbWF0Y2hlcyB0byB0aGUga2luZCBvZiB0cmFpdCBgTXlGdW5jdG9yYC4NCmltcGwgQXJyYXkgOiBNeUZ1bmN0b3Igew0KICAgIG15bWFwID0gfGYsIGFycnwgKA0KICAgICAgICBBcnJheTo6ZnJvbV9tYXAoYXJyLmdldF9zaXplLCB8aWR4fCBmKGFyci5AKGlkeCkpKQ0KICAgICk7DQp9DQoNCm1haW4gOiBJTyAoKTsNCm1haW4gPSAoDQogICAgbGV0IGFyciA9IEFycmF5Ojpmcm9tX21hcCg2LCB8eHwgeCk7IC8vIGFyciA9IFswLDEsMiwuLi4sOV0uDQogICAgbGV0IGFyciA9IGFyci5teW1hcCh8eHwgUGFpciB7IGZzdDogeCAlIDIsIHNuZDogeCAlIDMgfSk7IC8vIGFyciA9IFsoMCwgMCksICgxLCAxKSwgKDAsIDIpLCAuLi5dLg0KICAgIGxldCB4ID0gYXJyLnNlYXJjaChQYWlyIHsgZnN0OiAxLCBzbmQ6IDJ9KTsgLy8gNSwgdGhlIGZpcnN0IG51bWJlciB4IHN1Y2ggdGhhdCB4ICUgMiA9PSAxIGFuZCB4ICUgMyA9PSAyLg0KICAgIHByaW50bG4gJCB4LmdyZWV0aW5nIC8vIFRoaXMgc2hvdWxkIHByaW50ICJIaSEgSSdtIGEgNjQtYml0IGludGVnZXIgNSEiLg0KKTs%3D)

## 関連型

関連型は、トレイト（型の集合とみなす）を定義域とし、新たな型を返す、型レベルの関数だと考えることができます。
代表的な例は、標準ライブラリの`Iterator`トレイトの`Item`関連型です。

```
trait iter : Iterator {
    type Item iter;
    advance : iter -> Option (iter, Item iter);
}
```

ここでは、型レベルの関数`Item`を定義しています。
`Item`はイテレータ型（つまり、`Iterator`トレイトを実装する型）を受け取り、それによって生成される要素の型を返します。

関数の型シグネチャでは、関連型に対する制約を書くことができます。
例えば、2つのイテレータを比較する関数の型を記述することを考えます。
この関数は、`Item`が同じであり、`Eq`トレイトを実装する2つのイテレータを受け取る必要があります。
したがって、次のような型を持ちます：

```
is_equal : [iter1 : Iterator, iter2 : Iterator, Item iter1 = a, Item iter2 = a, a : Eq] iter1 -> iter2 -> Bool;
```

関連型は高いアリティを持つことができます。
以下は、関連型を使用してアリティ2の型レベル関数を定義する例です。

```
module Main;

// 関連型を使用して型レベルの数値に対する加算を定義します。

// まず、型レベルの数値を準備します。
type Zero = unbox struct { data : () };
type Succ n = unbox struct { data : () };
type One = Succ Zero;
type Two = Succ One;
type Three = Succ Two;

// `Value`は型レベルの数値をパラメータとし、その値を保持する型です。
type Value n = unbox struct { data : I64 };

// 型レベルの数値に対するトレイトを定義します。このトレイトは以下を要求します：
// - 2つの型レベルの数値の加算を行う関連型`Add`。
// - 型レベルの数値の値を保持する型`Value n`。
trait n : Nat {
    type Add n m; // アリティ2の関連型。
    value : Value n;
}

// 帰納法を用いて型レベルの数値に対する`Nat`を実装します。
impl Zero : Nat {
    type Add Zero m = m;
    value = Value { data : 0 };
}
impl [n : Nat] Succ n : Nat {
    type Add (Succ n) m = Succ (Add n m);
    value = (
        // 以下は型レベルの数値から値を抽出する方法です：
        // 型注釈を使用してトレイトメソッド`Nat::value`の適切な実装を選択します。
        let n = (Nat::value : Value n).@data;
        Value { data : n + 1 }
    );
}

main : IO ();
main = (
    assert_eq(|_|"", (Nat::value : Value Zero).@data, 0);;
    assert_eq(|_|"", (Nat::value : Value One).@data, 1);;
    assert_eq(|_|"", (Nat::value : Value Two).@data, 2);;
    assert_eq(|_|"", (Nat::value : Value (Add One Two)).@data, 3);;
    pure()
);
```

## トレイトエイリアス

トレイトのエイリアスを定義することができます。
以下のようにトレイトエイリアスを定義することで、

```
trait Foo = Bar + Baz;
```

`a : Bar, a : Baz`と書く代わりに`a : Foo`と書くことができます。

トレイトエイリアスを直接実装することはできません。
型`SomeType`に対して`Foo`を実装したい場合は、`SomeType : Bar`と`SomeType : Baz`を個別に実装してください。

## 型エイリアス

以下のように型エイリアスを定義できます：

```
type Name = String;
```

型エイリアスは新しい型を定義するわけではありません。エイリアスされた型の別名にすぎません。

高階型エイリアスも定義できます。以下は`Std`で定義されたそのような型エイリアスの例です：

```
type Lazy a = () -> a;
```

これはカインド`* -> *`の型エイリアス`Lazy`を定義しています。

## 動的イテレータ

Fixにおける`Iterator`はトレイトであり、多数の型が`Iterator`を実装しています。
したがって「イテレータ」という特定の型は存在せず、イテレータを生成する各関数は異なる型のイテレータを生成します。

たとえば、`Array a`から`to_iter`により作成されたイテレータの型は`ArrayIterator a`ですが、`range`により作成されるイテレータの型は`CountUpIterator`です。
また、すでにあるイテレータに`map`を適用すると、より複雑な型のイテレータが作成されます。
例えば、チュートリアルのコード例における`fib.to_iter.map(to_string)`の型は`MapIterator (ArrayIterator I64) I64 String`です。

このイテレータの設計は、パフォーマンスの向上に大いに貢献しています。
これは、イテレータの型から`advance`関数（`Iterator`トレイトのメソッド）の実装が一意に決定されるため、コンパイラは`advance`関数をインライン化するなどの最適化を行うことができるためです。

一方で、複雑なイテレータ型はプログラミングの障害となる場合があります。
例えば、「イテレータを作成して返す関数」を定義する場合、関数の戻り値の型に非常に複雑なイテレータ型を書く必要があります。
特に、その関数が状況（引数）により異なる方法で作成したイテレータを返すような場合は、関数の戻り値の型を書くことができなくなります。

イテレータ型の複雑さの問題を回避するために、次の型が用意されています。

```
type DynIterator a = unbox struct { next: () -> Option (DynIterator a, a) };
```

`DynIterator`は`Iterator`トレイトを実装しています。

`to_dyn`関数を使用して、任意のイテレータを`DynIterator`に変換できます。

```
// イテレータを動的イテレータに変換します。
to_dyn : [iter : Iterator, Item iter = a] iter -> DynIterator a;
```

「イテレータを作成して返す関数」を定義する場合、作成した複雑なイテレータを`to_dyn`で`DynIterator`に変換してから返すことで、関数の戻り値の型を単純な`DynIterator a`にすることができます。

`DynIterator`は、Haskellの遅延評価リストに似ています。
Haskellのリストを使う美しいコードをFixに移植する場合は、`to_dyn`が活躍するかもしれません。

ただし、パフォーマンスが必要な場合は`DynIterator`を避けることをお勧めします。
「イテレータを作成して返す関数」を実装する際に、`DynIterator`を避けるための対処法を一つ紹介します。

例として、以下のような関数を考えます。

```
pythagorean_triples : I64 -> DynIterator (I64, I64, I64);
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    )).to_dyn
);
```

コードの詳細を理解する必要はありません。`range`、`flat_map`、`filter`、`map`を組み合わせて複雑なイテレータを作成し、`to_dyn`で`DynIterator`に変換して返しているという点に注目してください。

このコードから`DynIterator`を取り除くには、上記のコードを`fix`のLanguage Server Protocolが動作しているテキストエディタにコピーします（参考：[（オプション）VScode拡張機能](#オプションvscode拡張機能)）
次に、`to_dyn`の上でマウスをホバーし、その型を表示させます。以下のように表示されるはずです。

```
Std::Iterator::to_dyn : [a : Std::Iterator, Std::Iterator::Item a = b] a -> Std::Iterator::DynIterator b
Instantiated as:

(非常に複雑なイテレータの型) -> Std::Iterator::DynIterator (Std::I64, Std::I64, Std::I64)
```

この`to_dyn`がどのイテレータを`DynIterator`に変換しているかが表示されるので、この型を型エイリアスとして定義します。
そして、`pythagorean_triples`の戻り値の型をその型エイリアスに変更し、`to_dyn`を削除します。

```
pythagorean_triples : I64 -> PythagorasIterator;
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    ))
);

type PythagorasIterator = (非常に複雑なイテレータの型);
```

これで`DynIterator`を避けることができました。

これはあまりエレガントな方法ではありませんが、現状では`DynIterator`を避けるための実用的な方法です。
将来のFixでは、以下のように書けるようにしたいと考えています。

```
pythagorean_triples : I64 -> impl Iterator<Item = (I64, I64, I64)>;
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    ))
);
```

## モナド

### モナドとは

トレイト`Monad`は以下のように定義されています：

```
trait [m : *->*] m : Monad {
    bind : (a -> m b) -> m a -> m b;
    pure : a -> m a;
}
```

すなわち、モナドとは、（`Array`や`Option`などの）型から型を作り出す写像`m`であり、`bind`と`pure`という2つの関数が定義されているものです。

モナドの定義はこれだけです。モナドを学ぶには、例を知ることが重要です。
以下のセクションでは、実際に使用される3つの典型的なモナドを紹介します。

#### 状態系モナド

「アクション（状態に作用する計算）」を表す型は、しばしばモナドになります。
このようなモナドを「状態系モナド」と呼ぶことにします。

以下の定義を考えます：

```
type State s a = unbox struct { run : s -> (s, a) }
```

`State s`は、`s`の値（「状態」）を受け取り、`a`型の値（「結果」）と、新しい状態を返す計算を表しています。

以下では、任意の型`s`に対し、`State s : Monad`を実装する方法を示します。
したがって、この`State s`は状態系モナドの例を与えます。

状態系モナドでは、`bind`は2つのアクションの結合を表します。
より具体的には、アクション`x.bind(f)`は以下のようなアクションを表します。
- まず、アクション`x`を実行します（ここで状態が更新されます）。アクション`x`の結果を`r`とします。
- 次に、アクション`f(r)`を実行します（ここでも状態が更新されます）。

アクション`pure(v)`は、状態との相互作用なしに単に`v`を返す計算を表します。

以上をまとめると、以下のように`State s : Monad`を実装できます：

```
impl State s : Monad {
    bind = |f, x| State { run : |state| (
        let (state, r) = (x.@run)(state);
        (f(r).@run)(state)
    )};
    pure = |v| State { run : |state| (state, v) };
}
```

Fixの標準ライブラリで定義されている`IO`も状態系モナドの例です。
`IO a`は、コンピュータの状態と相互作用しつつ`a`型の値を返すような「IOアクション」であると考えることができます。
実際、`IO`は、`IOState -> (IOState, a)`型（のラッパー）として定義されています。
ここで`IOState`は「コンピュータの状態」を表す型であるとイメージされるべき型です（実際には空の構造体として定義されています）。

`IO`を例として、`bind`の使い方を見てみましょう。
`print(str) : IO ()`は`str`を標準出力に出力するI/Oアクションです。
標準入力の内容を文字列として読み取るI/Oアクション`read : IO String`があると仮定します。
この場合、標準入力を読み取り、それをそのまま出力するI/Oアクション`echo`は次のように記述できます：

```
echo : IO ();
echo = read.bind(|s| print(s));
```

注意：実際には、Fixの標準ライブラリには`read : IO String`は定義されていません。`read_content(stdin).map(as_ok)`として実装できます。

#### 失敗系モナド

この種のモナドは、計算に失敗した可能性のある値を表します。

Fixの標準ライブラリでは、`Result`は以下のように定義されています。

```
type Result e o = unbox union { ok : o, err: e };
```

`Result e o`は、型`o`の値、または型`e`のエラー値を含みます。

別の例として、`Option`があります。

```
type Option a = union { none: (), some: a };
```

`Option a`は、型`a`の値を持つかもしれないし、持たないかもしれないことを表します。

任意の型`e`に対する`Result e`、および`Option`は、`Monad`を実装しており、失敗系モナドの例を提供します。

結果系モナドでは、`bind`はショートサーキット評価を行う方法を提供します。
`x.bind(f)`は、`x`がエラー（または"none"）値の場合、直ちにエラーを返します。
`x`がok（または"some"）値`v`の場合にのみ、関数`f`が呼び出され、`x.bind(f)`は`f(v)`となります。

また、`pure(v)`は値`v`を持つ成功した計算を表します。

`Option`に対する`Monad`の実装例を以下に示します。

```
impl Option : Monad {
    bind = |f, opt| match opt {
        none(_) => none(),
        some(v) => f(v)
    };
    pure = some;
}
```

`bind`の利用例として、`Option`に包まれた2つの整数を加算する関数`add_opt : Option I64 -> Option I64 -> Option I64`を考えます。
この関数は、両方が`some`値である場合にのみ加算を行い、そうでない場合は`none`を返すものとします。

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| (
    if x.is_none { none() };
    let x = x.as_some;
    if y.is_none { none() };
    let y = y.as_some;
    some(x+y)
);
```

`Option`に対する`bind`を使用すると、上記の関数は次のように簡潔に記述できます。

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| x.bind(|x| y.bind(|y| some(x+y)));
```

#### シーケンス系モナド

配列のように、シーケンス（要素の列）を表す型もモナドのインスタンスとなることがあります。
Fixの標準ライブラリでは、`Array`と`DynIterator`が`Monad`トレイトを実装しています。

シーケンス系モナドでは、`[x, y, z, ...].bind(f)`は`f(x) + f(y) + f(z) + ...`を表します。ここで、`+`は2つのシーケンスの結合を表します。
`bind`は他の言語では"flat_map"と呼ばれることがあります。

`pure(x)`は単一の値`[x]`を表します。

例えば、デカルト積を計算する関数`product : Array a -> Array b -> Array (a, b)`は、`bind`を用いて以下のように実装できます。

```
product : Array a -> Array b -> Array (a, b);
product = |xs, ys| xs.bind(|x| ys.bind(|y| pure $ (x, y)));
```

`xs == [x0, x1, ...]`, `ys == [y0, y1, ...]`とするとき、`product(xs, ys)`は次のように展開され、確かにデカルト積を計算していることがわかります。

```
xs.bind(|x| ys.bind(|y| pure $ (x, y)))
== ys.bind(|y| pure $ (x0, y)) + ys.bind(|y| pure $ (x1, y)) + ...
== (pure $ (x0, y0)) + (pure $ (x0, y1)) + ... + (pure $ (x1, y0)) + (pure $ (x1, y1)) + ... + ...
== [(x0, y0)] + [(x0, y1)] + ... + [(x1, y0)] + [(x1, y1)] + ... + ...
== [(x0, y0), (x0, y1), ..., (x1, y0), (x1, y1), ..., ...]
```

### `do`ブロックとモナドのバインド演算子`*`

Fixの前置単項演算子`*`は、`bind`をより簡潔に使用する方法を提供します。
コード`B(*x)`は`x.bind(|v| B(v))`に展開されます。

ここで、`B(*x)`は式`*x`を囲む最小の`do`ブロックです。
`do`ブロックは明示的あるいは暗黙的に次のように作成されます。

- `do { ... }`で明示的に`do`ブロックを作成できます。
- グローバル定義`name = ...`は暗黙的に`do`ブロック`...`を定義します。
- let定義`let name = val (in|;) ...`は暗黙的に`do`ブロック`...`を定義します。
- ラムダ式`|arg| ...`は暗黙的に`do`ブロック`...`を定義します。
- if式`if cond { ... } else { ... }`は暗黙的に2つの`do`ブロック`...`を定義します。
- マッチ式`match val { pat => ... }`は暗黙的に`do`ブロック`...`を定義します。
- ダブルセミコロン構文（後述）`act;; ...`は暗黙的に`do`ブロック`...`を定義します。

以前のセクションで、状態系モナドである`IO`において、`bind`を使用して`read : IO String`と`print : String -> IO ()`から`echo : IO ()`を作成する例を示しました。

```
echo : IO ();
echo = read.bind(|s| print(s));
```

`bind`をより簡潔に使うための演算子`*`を使うと、上記は次のように書けます。

```
echo : IO ();
echo = print(*read);
```

これは、演算子`*`が`read`というモナド値の内容を取り出し、その内容を`print`に渡している、と解釈することができます。実際、以下のように書いても同じです。

```
echo : IO ();
echo = (
    let s = *read;
    print(s)
);
```

同様に、

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| x.bind(|x| y.bind(|y| Option::some(x+y)));
```

は

```
add_opt : Option I64 -> Option I64 -> Option I64;
add_opt = |x, y| some $ *x + *y;
```

と書けます。
ここでも、演算子`*`でモナド値`x`と`y`の内容を取り出し、その内容を加算して`some`に渡すことで最終的な`Option I64`値を作成しています。

```
product : Array a -> Array b -> Array (a, b);
product = |xs, ys| xs.bind(|x| ys.bind(|y| pure $ (x, y)));
```

は

```
product : Array a -> Array b -> Array (a, b);
product = |xs, ys| pure $ (*xs, *ys);
```

と書くことができます。
ここでは、`*xs`と`*ys`でそれぞれのシーケンスの要素を一つずつ取り出し、その組を`pure`に渡すことでデカルト積を計算しています。

### 明示的な`do`ブロックが必要な場合

ここまでの例では、`*`を使用する際に明示的に`do`ブロックを作成する必要はありませんでした。
以下で、明示的に`do`ブロックを作成する必要がある例を挙げます。

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| do { some $ *x + *y }.as_some;
```

これは、受け取った2つの`Option I64`値を加算し、その結果を`I64`として返す関数です。どちらかが`none`の場合はプログラムが停止します。

上記の`add_opt_unwrap`の定義は次のように展開され、コンパイルできます。

```
add_opt_unwrap = x.bind(|x| y.bind(|y| some $ x + y)).as_some;
```

一方、次のように、明示的に`do`ブロックを作成しない場合、

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| (some $ *x + *y).as_some;
```

これは次のように展開されます。

```
add_opt_unwrap = |x, y| x.bind(|x| y.bind(|y| (some $ x + y).as_some));
```

`do`を使わない後者のコードは型エラーとなり、コンパイルできません。
実際、外側の`bind`の帰り値は`Option I64`型ですが、関数`add_opt_unwrap`は`I64`を返す必要があります。

複雑に見えるかもしれませんが、「`do`ブロックの範囲がモナド値となる」という感覚を持っておくと、`*`を使用する際に`do`ブロックを明示的に作成する必要があるかどうかを判断するのは比較的簡単です。

コンパイルに成功するコード

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| do { some $ *x + *y }.as_some;
```

は、明示的な`do`の範囲がモナド`Option I64`となり、それに`as_some`を適用することができるため、問題ありません。

コンパイルに失敗するコード

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| (some $ *x + *y).as_some;
```

に暗黙的に作成される`do`ブロックの範囲を明示すると、以下のようになります。

```
add_opt_unwrap : Option I64 -> Option I64 -> I64;
add_opt_unwrap = |x, y| do { (some $ *x + *y).as_some };
```

これは、`do`ブロックの範囲が型`Option I64`となりますが、この関数は`I64`を返す必要があるため、型エラーとなります。

### モナドアクションを`;;`構文で連鎖させる

関数`println : String -> IO ()`は、文字列を受け取り、その文字列を標準出力に出力するIOアクションを作成します。
複数回`println`を実行したい場合、演算子`*`を使用して次のように記述できます。

```
module Main;

main : IO ();
main = (
    let _ = *println("The sum of 1 + 2 is: ");
    let _ = *println((1 + 2).to_string);
    pure()
);
```

`println(...)`のIOアクションの結果は必要でないため、変数`_`に結果を代入して無視しています。
また、`pure() : IO ()`は「何もしない」というIOアクションを表します。

ダブルセミコロン構文`{expr0};; {expr1}`は、`let _ = *{expr0}; {expr1}`と同等です。
したがって、上記のコードは次のように記述できます。

```
module Main;

main : IO ();
main = (
    println("The sum of 1 + 2 is: ");;
    println((1 + 2).to_string);;
    pure()
);
```

### Fixのイテレータはモナドではない

要素の列を表す型はしばしば「シーケンス系モナド」となる、と前述しました。
しかしながら、`Iterator`はFixの標準ライブラリはトレイトであり、型ではないため、`Iterator`自体はモナドではありません。

`Std`に定義されているイテレータの中で、唯一、`DynIterator`は`Monad`を実装しており、シーケンス系モナドとなっています。
したがって、`*`演算子を使用して`DynIterator`を操作することができます。

以下は、`1 <= a <= b <= c <= limit`を満たすすべての`a`, `b`, `c`を全探索して、ピタゴラスの三つ組`(a, b, c)`を列挙するプログラムです。
`range(a, b)`で作成したイテレータを`DynIterator`に変換するために、`to_dyn`メソッドを使用しています。

```
pythagorean_triples : I64 -> DynIterator (I64, I64, I64);
pythagorean_triples = |limit| (
    let a = *Iterator::range(1, limit+1).to_dyn;
    let b = *Iterator::range(a, limit+1).to_dyn;
    let c = *Iterator::range(b, limit+1).to_dyn;
    if a*a + b*b != c*c {
        DynIterator::empty
    };
    (a, b, c).pure
);
```

[動的イテレータ](#動的イテレータ) で述べたように、`DynIterator`は他のイテレータに比べてパフォーマンスが劣ります。そこで、上記のコードを`DynIterator`を使用しない形に書き換える方法を紹介します。

既に述べたように、シーケンス系モナドにおける`bind`は"flat map"として知られる操作です。
Fixの標準ライブラリはイテレータに対する`flat_map`を提供しています。
そこで、演算子`*`の定義を思い出し、上記のコードを`bind`を明示的に用いる形に書き換えたあと、`bind`を`flat_map`に置き換えることで、`DynIterator`を使用しないコードを得ることができます。
結果は以下のようになります。結果として得られるイテレータは非常に複雑な型を持つため、最後に`to_array`メソッドを使用して配列に変換しています。

```
pythagorean_triples : I64 -> Array (I64, I64, I64);
pythagorean_triples = |limit| (
    Iterator::range(1, limit+1).flat_map(|a| (
        Iterator::range(a, limit+1).flat_map(|b| (
            Iterator::range(b, limit+1).filter(|c| (
                a*a + b*b == c*c
            )).map(|c| (a, b, c))
        ))
    )).to_array
);
```

## ボックス型とアンボックス型

Fixの型はボックス型とアンボックス型に分けられます。ボックス型とアンボックス型は、それぞれ他の言語で「参照型」や「値型」と呼ばれるものに似ています。

* ボックス型の値はヒープメモリに割り当てられます。型がボックス型であるローカル名や構造体/ユニオンのフィールドは、値へのポインタとしてコンパイルされます。
* アンボックス型の値はスタックメモリ、構造体、ユニオンに直接埋め込まれます。

一般に、大量のデータを含む型（例えば`Array a`）は、コピーコストの低いボックス型が適しています。
一方、少ないデータしか持たない型（例えば`I64`）は、アンボックス型にすることで、参照カウンタの増減コストを削減し、メモリの局所性を向上させることができます。

### 関数

関数はアンボックス型ですが、キャプチャされた値は無名のボックスな構造体に格納されます。

### タプルとユニット

タプル型はアンボックス型です。これは、タプルは少数のフィールドを持つことを意図しているからです。
多くのフィールドを使用したい場合は、新しい構造体を定義するべきでしょう。
タプルはフィールド名が`0`、`1`、`2`などである特別な形式の構造体です。

ユニット型は長さ0のタプル型であるため、ユニット型もアンボックス型です。

### 配列

`Std::Array`はボックス型です。

### 構造体

構造体はデフォルトでアンボックス型です。ボックス構造体型を定義するには、`struct`の前に`box`指定子を記述します。

```
type Product = box struct { price: I64, sold: Bool };
```

### ユニオン

ユニオンはデフォルトでアンボックス型です。ボックスユニオン型を定義するには、`struct`の前に`box`指定子を記述します。

```
type Weight = box union { pound: I64, kilograms: I64 };
```

## 外部関数インターフェース (FFI)

Fixプログラムに静的または共有ライブラリを`--static-link` (`-s`) または`--dynamic-link` (`-s`) コンパイラフラグを使用してリンクすることで、Fixプログラム内でネイティブ関数を呼び出したり、ライブラリ内でFix関数を呼び出すことができます。

ただし、FFIを使用すると、イミュータビリティやメモリ安全性などのFixの保証が外部関数によって破られる可能性があります。
プログラマーには、外部関数の副作用を`IO`に隠し、セグメンテーションフォルトやメモリリークを回避するためにリソースを適切に管理する責任があります。

### Fixで外部関数を呼び出す

Fixで外部関数を呼び出すには、`FFI_CALL(_IO|_IOS)[...]`式を使用します。構文は次のとおりです：

```
FFI_CALL[{function_signature}, {arg_0}, {arg_1}, ...]
```

```
FFI_CALL_IO[{function_signature}, {arg_0}, {arg_1}, ...]
```

```
FFI_CALL_IOS[{function_signature}, {arg_0}, {arg_1}, ..., {iostate}]
```

純粋な外部関数を呼び出すには`FFI_CALL`を使用します。`FFI_CALL[...]`は外部関数と同じ引数を取り、外部関数の戻り値に対応するFixの値を返します。

外部関数に副作用がある場合は、`FFI_CALL_IO`を使用します。これにより、`IO`モナド値が返されます。

`FFI_CALL_IO`の代わりに`FFI_CALL_IOS`を使用することもできます。
この関数は、型`IOState`の追加の引数を取り、型`(IOState, a)`の値を返します。ここで、`a`は外部関数の戻り値の型です。

注：`IOState`はFixの標準ライブラリで定義されている型であり、`IO`モナドの内部状態を表します。実際、`IO`は以下のように定義されています。

```
type IO a = unbox struct { runner : IOState -> (IOState, a) };
```

`FFI_CALL`および`FFI_CALL_IO`の使用例として、`Std::consumed_time_while_io`の実装を紹介します。

```
// I/Oアクションを実行中に経過したクロック（CPU時間）を取得します。
consumed_time_while_io : IO a -> IO (a, F64);
consumed_time_while_io = |io| (
    let s = *FFI_CALL_IO[I64 fixruntime_clock()];
    let r = *io;
    let t = *FFI_CALL_IO[I64 fixruntime_clock()];
    let t = FFI_CALL[F64 fixruntime_clocks_to_sec(I64), t - s];
    pure $ (r, t)
);
```

`fixruntime_clock`および`fixruntime_clocks_to_sec`は、Fixのランタイムライブラリで定義されているC言語の関数です。

`fixruntime_clock`は副作用のある関数であるため、`FFI_CALL_IO`を使用して呼び出しています。
一方、`fixruntime_clocks_to_sec`は純粋な関数であるため、`FFI_CALL`を使用して呼び出しています。

`FFI_CALL`（あるいは`FFI_CALL_IO`, `FFI_CALL_IOS`）の`{c_function_signature}`では、呼び出す外部関数の名前とシグネチャを指定します。
シグネチャは`{return_type} {function_name}({arg_type_0}, {arg_type_1}, ...)`の形式で記述します。
`{return_type}`または`{arg_type_i}`には、以下の型を使用できます：

- ポインタ：`Ptr`
- 明示的なビット幅を持つ数値型：`I8`、`U8`、`I16`、`U16`、`I32`、`U32`、`I64`、`U64`、`F32`、`F64`
- Cの数値型：`CChar`、`CUnsignedChar`、`CShort`、`CUnsignedShort`、`CInt`、`CUnsignedInt`、`CLong`、`CUnsignedLong`、`CLongLong`、`CUnsignedLongLong`、`CSizeT`、`CFloat`、`CDouble`
- `void`の代わり：`()`

### Fixの値や関数を外部言語にエクスポートする

Fixの値を外部言語から利用するには、`FFI_EXPORT[{fix_value_name}, {c_function_name}];`構文を使用します。

```
fix_increment : CInt -> CInt;
fix_increment = |x| x + 1.to_CInt;
FFI_EXPORT[fix_increment, increment]; // 関数`int increment(int)`が定義されます。
```

例えばC言語のライブラリから上記の`fix_increment`を呼び出すには、ソースコード中で`int increment(int);`を宣言し、必要な場所で`increment`を呼び出します。

エクスポートされる関数のシグネチャは、Fix値の型によって自動的に決定されます。
以下で、Fix値の型からどのようにC関数のシグネチャが決定されるかを例示します。

```
x : CInt; 
FFI_EXPORT[x, f]; // int f(void);

x : CInt -> CInt;
FFI_EXPORT[x, f]; // int f(int);

x : CInt -> CInt;
FFI_EXPORT[x, f]; // int f(int);

x : IO ();
FFI_EXPORT[x, f]; // void f(void);

x : IO CInt;
FFI_EXPORT[x, f]; // int f(void);

x : CInt -> IO CInt;
FFI_EXPORT[x, f]; // int f(int);
```

### Fixで外部リソースを管理する

一部のC関数は、最終的に別のC関数によって解放されるべきリソースを割り当てます。
最も有名な例は、`malloc` / `free`や`fopen` / `fclose`です。
Fixから`FFI_CALL`を使用してリソースを割り当てた場合、そのリソースのライフタイムの終わりに再度`FFI_CALL`を使用して解放関数を呼び出す必要があります。

このようなリソース管理を行うために、`Std::FFI::Destructor`を利用できます。
`Destructor a`は、ボックス型であり、データとしては`value : a`と`dtor : a -> IO a`を持ちます。
Fixコンパイラは、`Destructor a`をヒープメモリから解放する際に、`dtor`を`value`に対して呼び出します。

典型的な使い方は、`malloc`や`fopen`を使用して得たリソースへのポインタを`Destructor Ptr`の`value`フィールドに格納し、`free`や`fclose`を呼び出すIO処理を`dtor`フィールドに格納することです。これで、その`Destructor Ptr`型の値がスコープから外れたときに、リソースが自動的に解放されます。

ただし、Destructorを適切に使用するのは容易ではなく、様々な点に注意が必要です。
[`Destructor`のドキュメント](/std_doc/Std.md#Destructor)、[namespace Destructor](/std_doc/Std.md#namespace_Std::FFI::Destructor) にある関数群も確認してください。

### 外部言語でFixのボックス値の所有権を管理する

関数`Std::FFI::boxed_to_retained_ptr : a -> Ptr`は、Fixによって割り当てられた*ボックス型*の値への保持ポインタを返します。
ここで、"保持"とは、ポインタが値の共有所有権を持ち、メモリリークを回避するために参照カウンタをデクリメントする責任があることを意味します。
保持ポインタからFix値を取得するには、`Std::FFI::boxed_from_retained_ptr : Ptr -> a`を使用します。

外部言語でFix値の保持ポインタを持っている場合、ポインタをドロップするときにそれを解放（つまり、参照カウンタをデクリメント）するか、ポインタをコピーするときにそれを保持（つまり、参照カウンタをインクリメント）する必要がある場合があります。
これを行うには、まずFix値の保持/解放関数へのポインタを`Std::FFI::get_funptr_release`および`Std::FFI::get_funptr_retain`で取得します：

- `Std::FFI::get_funptr_release : a -> Ptr`
- `Std::FFI::get_funptr_retain : a -> Ptr`

各関数は、型`void (*)(void*)`の関数ポインタを返します。
次に、関数ポインタを介して型`a`のFix値を保持/解放できます。

注意：
Fixの参照カウントはデフォルトではスレッドセーフではありません。
したがって、Fixのボックス値へのポインタを取得し、それを複数のスレッド間で共有する場合、上記の方法でポインタを保持/解放するとデータ競合が発生する可能性があります。

これを回避するには、まず`--threaded`コンパイラフラグを追加します。
さらに、ポインタを取得する前に`Std::mark_threaded : a -> a`をボックス値に対して呼び出します。
`Std::mark_threaded`関数は、指定された値から到達可能なすべての値をトラバースし、それらをマルチスレッドモードに変更して、参照カウントがスレッドセーフな方法で行われるようにします。

### CからFixの構造体値のフィールドにアクセスする

*ボックス型*の構造体型があると仮定します：
```
type Vec = box struct { x : CDouble, y : CDouble };
```

そして、Cプログラム：
```
struct Vec {
    double x;
    double y;
}

void access_vec(Vec* v) {
    // `v->x`および`v->y`に対して何らかの操作を行います。
}
```
Fixのオブジェクト`vec`のフィールド`x`および`y`にC側からアクセスしたい場合、`Std::FFI::borrow_boxed : (Ptr -> b) -> a -> b`が便利です：
`vec.borrow_boxed(|p| FFI_CALL[() access_vec(Ptr), p])`を使用すると、`access_vec`が`vec.@x`および`vec.@y`で動作するようになります。

注意：
少なくとも現在のFixのバージョンでは、Fixの構造体のメモリレイアウトはLLVMのデフォルトの動作によって決定されており、私の知る限りではCの構造体のメモリレイアウトと同等です。
将来のバージョンでは状況が変わる可能性があります。プログラマーがレイアウトがCと同等であることを保証するための指定子（仮に`expr_c`と記述されるとします）を導入し、`expr_c`指定子のない構造体レイアウトは最適化される（例：フィールド順序の再配置）可能性があります。

## `eval`構文

式`eval {expr0}; {expr1}`は、`{expr0}`と`{expr1}`の両方を評価し、`{expr1}`の値を返します。

Fixは、最適化において、不要な式の評価を省略することがあります。例えば、
```
main : IO () = (
    let x = 1 + 2;
    println("Hello, World!");
);
```
というようなプログラムにおいて、`x = 1 + 2`の評価はプログラムの動作に影響を与えないため、Fixコンパイラはこの評価を省略してしまう可能性があります。

`eval`構文は、Fixコンパイラに対して、式の評価を省略しないように指示するために使用されます。

この構文は、主に、デバッグ目的で使用されます。
例えば、`debug_eprint : String -> ()`は、`IO`モナドを使用せずに、メッセージを標準エラー出力に出力する関数です。
この関数は、
```
my_add : I64 -> I64 -> I64 = |x, y| (
    let z = x + y
    eval debug_eprint("The sum is: " + z.to_string);
    z
);
```
のように、`eval`構文を用いて使用してください。
この例において、`debug_eprint(...)`の呼び出しは`my_add`の結果に影響を与えませんが、`eval`を使用することで、メッセージが確実に出力されることが保証されます。

注意
- `eval`式全体の結果（すなわち`{expr1}`の結果）を使用しないプログラムの場合は、Fixコンパイラは、`eval`式全体を省略し、結果として`{expr0}`を評価しない可能性があります。
- 現時点では、`{expr0}`と`{expr1}`の評価順序は保証されていません。
- コンパイラは、その`eval`式がプログラムの実行に必要である限り、`{expr0}`が少なくとも一回は評価されることを保証しますが、何回評価するかは保証しません。例えば、
```
truth : I64 = eval debug_println("evaluated"); 42;
```
というコードがあった時、`truth`を参照するたびに"evaluated"が出力されるか、最初に参照するときに一回だけ出力されるかは保証されません。

## 演算子

以下は、優先順位でソートされた演算子の表です（優先順位が高いものから低いものへ並べられています）。

| 演算子           | 結合性             | 関数                               | 説明                                                          |
| ---------------- | ------------------ | ---------------------------------- | ------------------------------------------------------------- |
| .                | 左結合の二項演算子 | -                                  | 右から左への関数適用：`x.f` = `f(x)`                          |
| *                | 単項前置           | Std::Monad::bind                   | モナドのバインド                                              |
| <<               | 左結合の二項演算子 | Std::compose                       | 右から左への関数合成：`g << f` = `&#124;x&#124; g(f(x))`      |
| >>               | 左結合の二項演算子 | Std::compose                       | 左から右への関数合成：`(f >> g)(x)` = `&#124;x&#124; g(f(x))` |
| - (マイナス記号) | 単項前置           | Std::Neg::neg                      | 数値の負値                                                    |
| !                | 単項前置           | Std::Not::not                      | 論理NOT                                                       |
| *                | 左結合の二項演算子 | Std::Mul::mul                      | 数値の乗算                                                    |
| /                | 左結合の二項演算子 | Std::Div::div                      | 数値の除算                                                    |
| %                | 左結合の二項演算子 | Std::Rem::rem                      | 除算の余り                                                    |
| +                | 左結合の二項演算子 | Std::Add::add                      | 数値の加算                                                    |
| - (マイナス記号) | 左結合の二項演算子 | Std::Sub::sub                      | 数値の減算                                                    |
| ==               | 左結合の二項演算子 | Std::Eq::eq                        | 等価比較                                                      |
| !=               | 左結合の二項演算子 | -                                  | `x != y`は`!(x == y)`として解釈されます                       |
| <=               | 左結合の二項演算子 | Std::LessThanOrEq::less_than_or_eq | 以下比較                                                      |
| >=               | 左結合の二項演算子 | -                                  | `x >= y`は`y <= x`として解釈されます                          |
| <                | 左結合の二項演算子 | Std::LessThan::less_than           | 未満比較                                                      |
| >                | 左結合の二項演算子 | -                                  | `x > y`は`y < x`として解釈されます                            |
| &&               | 右結合の二項演算子 | -                                  | ショートサーキット論理AND                                     |
| &#124;&#124;     | 右結合の二項演算子 | -                                  | ショートサーキット論理OR                                      |
| $                | 右結合の二項演算子 | -                                  | 右結合の関数適用：`f $ g $ x` = `f(g(x))`                     |
| ;;               | 右結合の二項演算子 | -                                  | モナドアクションの連結：`m0;; m1` = `let _ = *m0; m1`         |

# コンパイラの機能

## プロジェクトファイル

プロジェクトファイルは、Fixプロジェクトに関する情報を含むTOMLファイルです。例えば：

- プロジェクト名、バージョン、著者など
- プロジェクトに含まれるFixソースファイル
- 他のFixプロジェクトへの依存関係
- リンクされる非Fixプログラム（オブジェクトファイル、静的または動的ライブラリなど）
- コンパイル前に実行されるコマンド

プロジェクトファイルの名前は"fixproj.toml"である必要があります。
"fix"コマンドの多くの機能は、現在のディレクトリ内のプロジェクトファイルを読み取り、見つかった場合、その情報を使用します。
さらに、一部のサブコマンド（例："fix deps"、"fix docs"、"fix language-server"）は、プロジェクトファイルが存在することを要求します。

"fix init"コマンドは[テンプレートプロジェクトファイル](./src/docs/project_template.toml)を生成します。
プロジェクトファイルについて詳しく知りたい場合は、その中のコメントを読んでください。

## 依存関係の管理

Fixプロジェクトの依存関係は、"fixproj.toml"ファイル内の[[dependencies]]要素によって表されます。
以下は、リモートリポジトリの"hash"とローカルリポジトリの"mylib"という2つの依存関係を追加する例です。

```
[[dependencies]]
name = "hash"
version = "0.1.0"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git" }

[[dependencies]]
name = "mylib"
version = "*"
path = "/path/to/mylib"
```

ここで、記法`version = "0.1.0"`は、バージョン"0.1.0"またはそれとSemVer互換性のある他のバージョンを要求することを意味します。
SemVer互換性の定義はCargoと同じです。詳細については、https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility を参照してください。

[[dependencies]]要素を手動で追加するか、"fix deps add {name}@{ver-req}"コマンドを使用して依存関係を追加できます。
"fix deps add"コマンドは、指定されたFixプロジェクトを"レジストリファイル"から検索し、見つかった場合はプロジェクトファイルに依存関係を追加します。
デフォルトのレジストリファイルは[このリポジトリ](https://github.com/tttmmmyyyy/fixlang-registry)で管理されています。
他のレジストリファイルを追加するには、それらを[設定ファイル](#configuration-file)に指定します。
レジストリファイルに登録されているすべての利用可能なプロジェクトを一覧表示するには、"fix deps list"コマンドを使用します。

前述のように、[[dependencies]]要素は各依存関係に対して特定のバージョンではなく、バージョンの範囲を指定します。
各依存関係に使用する特定のバージョン（コミット）は、"fixdeps.lock"ファイルに記述されます。
このファイルは"fix deps add"コマンドを実行すると自動的に生成され、"fix deps update"コマンドを実行して新しいバージョンを使用するように更新できます。

"fix deps install"コマンドは、"fixdeps.lock"ファイルに記述された依存関係を".fix"ディレクトリにインストールします。
このコマンドは"fix build"または"fix run"コマンドから自動的に呼び出されます。

## 設定ファイル

"fix"コマンドの動作を、ホームディレクトリ内の".fixconfig.toml"という名前の設定ファイルで指定できます。

設定ファイルで許可されるフィールドは以下の通りです：

```
# レジストリファイルのURL/パス。
# "fix deps add {proj-name}@{ver-req}"コマンドは、最初から最後までレジストリファイル内でプロジェクトを検索し、見つかった場合、現在のディレクトリのプロジェクトファイルに"[[dependencies]]"セクションを追加します。
# デフォルトのレジストリ"https://raw.githubusercontent.com/tttmmmyyyy/fixlang-registry/refs/heads/main/registry.toml"はリストの最後に暗黙的に追加されます。
registries = [
    "https://first-searched-registry.com/registry.toml",
    "https://second-searched-registry.com/registry.toml",
    "/path/to/my_registry.toml"
]
```

## ドキュメントの生成

`fix docs`サブコマンドは、Fixプロジェクトのドキュメント（Markdownファイル）を生成します。
このコマンドを実行するには、現在のディレクトリにプロジェクトファイルが存在する必要があります。

宣言の上に連続する行コメントは、ドキュメントとして認識されます：

```
// これはモジュールのドキュメントコメントです。
module Main;

// これは値のドキュメントコメントです。
truth : I64;
truth = 42;

// これは型のドキュメントコメントです。
type MyType = struct { x : I64 };

// これはトレイトのドキュメントコメントです。
trait a : MyTrait {
    // これはトレイトメソッドのドキュメントコメントです。
    to_number : a -> I64;
}

// これはトレイト実装のドキュメントコメントです。
impl MyType : MyTrait  {
    to_number = |mt| mt.@x;
}
```

## Language Server Protocol

`fix language-server`を実行すると、Language Server Protocol（LSP）をサポートする言語サーバーが起動します。
VSCode用の言語クライアント拡張機能は[こちら](https://marketplace.visualstudio.com/items?itemName=tttmmmyyyy.fixlang-language-client)で利用可能です。
言語サーバーは[プロジェクトファイル](#project-file)を必要とし、Fixソースファイルを認識します。

ファイルを保存するたびに、言語サーバーはFixプログラムを診断しようとします。
最新の診断で得られた情報は、補完、ホバー、定義への移動などに使用されます。
したがって、情報を更新するには、正しいFixコードを書いてファイルを保存する必要があります。
[`Std::undefined`](/std_doc/Std.md#undefined-----a)が役立つ場合があります。

### ドキュメントコメントでパラメータリストを指定して言語サーバーにヒントを与える

言語サーバーは、関数のパラメータリストを知っていると、より良い機能を提供できます。
例えば、パラメータ`x`と`y`を持つ関数名`foo`を補完するとき、`foo(x, y)`のようにプレースホルダー引数を挿入できます。

ただし、Fixは関数型プログラミング言語であるため、次の例のように関数のパラメータリストが曖昧になる場合があります：

```
foo : I64 -> I64 -> I64 -> I64;
foo = |x, y| (
    if x == 1 {
        |z| x + y + z
    } else {
        |k| (x + y) * k
    }
);
```

この関数のパラメータリストは`x`、`y`、`z`ですか、それとも`x`、`y`、`k`ですか？

これに対処するために、関数のドキュメントコメントの"Parameters"セクションでパラメータリストを指定できます。
次のように記述します：

```
// # Parameters
// * `x` - the first argument
// * `y` - the second argument
foo : I64 -> I64 -> I64 -> I64;
```

このコメントは、`foo`が典型的な場合に2つの引数`x`と`y`を持つ関数であることを示しています。
その後、関数名`foo`を補完するとき、言語サーバーは`foo(x, y)`というテキストを挿入します。
もし`foo`がドットの後で補完された場合（例：`y.foo`）、`y.foo(x)`として挿入されます。

ここでは、ドキュメントコメントの仕様をより詳細に説明します。

- 言語サーバーはドキュメントコメントをMarkdownとして解釈し、レベル1または2の"Parameters"セクションを検索します。
- 見つかった場合、すべてのリスト（`* `または`- `で始まる行）からパラメータ名を抽出します。
- パラメータ名はバッククォート（"`"）で囲む必要があります。
- バッククォート内に型注釈（例：`x : I64`）を含めることができますが、言語サーバーは無視します。

## Fixプログラムのデバッグ

`fix build`、`fix run`、または`fix test`を`-g`オプション付きで実行すると、DWARFデバッグ情報を含む実行可能バイナリが生成されます。
その後、lldb、gdb、または[CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)などのGUIデバッガーでバイナリをデバッグできます。

VSCodeでは、デフォルトでは*.fixファイルにブレークポイントを設定できません。回避策として、"Preferences"を開き、"Allow Breakpoints Everywhere"をONにします。

さらに、`fix build`、`fix run`、または`fix test`に`--backtrace`オプションを追加すると、パニックが発生したときにスタックトレースが表示されます。`-g`オプションと併用すると、スタックトレースに関数名と行番号が表示されます。

Fixプログラムのデバッグに関するその他の注意点：
- 他の言語とは異なり、Fixはスコープの終わりでローカル変数を解放せず、最後に使用されたポイントで解放します。そのため、ローカル変数の最後の使用後にブレークすると、デバッガーが無効な値を表示する場合があります。
- 現在、実行時に決定される配列のサイズをデバッガーに伝えることはできません。そのため、デバッグ情報では配列サイズを常に100に設定しています。100を超えるインデックスの要素を表示することはできず、配列が100より短い場合、無効な値が表示されます。

# その他のドキュメント

*[デフォルトレジストリ内のすべてのモジュールのドキュメント](https://tttmmmyyyy.github.io/fixlang-docpage-generator/)
