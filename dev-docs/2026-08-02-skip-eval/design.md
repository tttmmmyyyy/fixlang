# `eval` の評価を飛ばすビルド設定: 設計

開発中は `eval debug_println(...)` をソースに書いておき、要らなくなったらビルド設定ひとつで落とせるようにする。

## `eval` の現状

構文は `eval {side}; {main}` で、`{side}` と `{main}` の両方を評価し、`{main}` の値を返す。

`Expr::Eval(side, main)` は AST の第一級のノードである。脱糖されず、インスタンス化を経て RC IR の `lower_eval` まで残る。作るのはパーサの `parse_expr_eval` だけで、最適化パスのうち `application_inlining` が `(eval {side}; {main})({a})` を `let x = {a}; eval {side}; {main}(x)` に書き換えるときに作り直す。

`Document.md` の `eval` の節が規定しているのは次のことである。

- Fix は不要な式の評価を省略してよい。
- `eval` は、その省略をしないよう指示する構文である。
- `eval` 式全体の結果が使われないなら、`eval` 式ごと省略されうる。
- `{expr0}` と `{expr1}` の評価順は規定しない。
- `eval` 式がプログラムの実行に必要である限り `{expr0}` は最低 1 回評価されるが、回数は規定しない。

`lower_eval` のコメントが、ノードが何を強制するかを述べている。複合式はそれが積む束縛によって計算され、局所変数は既に計算済みで、グローバル値への参照は（それ自体は何も生成しない atom に下りるので）このノードが観測することで call-once の初期化子が走る。

## 何を作るか

- コンパイラフラグ `--skip-eval`。`fix build` / `fix run` / `fix test` に付ける。
- プロジェクトファイルの `[build]` セクションの `skip_eval` フィールド。ルートプロジェクトのものだけが効く。

どちらかが立っているビルドでは、コンパイラは `eval {side}; {main}` を `{main}` に置き換える。

## 除去の位置

`optimization::run` の先頭、`optimize_act` の前に新しいパスを置く。理由は次の 3 つである。

**型検査とインスタンス化を通った後なので、フラグの有無でコンパイルの成否が変わらない。** `{side}` の型エラーは、フラグを立てたビルドでも報告される。パース直後に落とす案はここで却下される。フラグを立てたビルドだけが通る状態は、フラグを外した瞬間に壊れるソースを許すことになる。

**型検査キャッシュがフラグに依存しない。** 型検査とインスタンス化の間に落とす案には、「`eval` の中でしか使われないシンボルをインスタンス化しなくて済む」というコンパイル時間の利得がある。だが型検査済みの式はモジュール単位でディスクにキャッシュされるので、そこを書き換えるならキャッシュのキーにフラグを混ぜることになり、フラグを切り替えるたびに全モジュールの型検査がやり直しになる。最適化パスの側で落とせば、キャッシュはフラグと無関係のままでよい。代わりに、`eval` の中でしか使われないシンボルはインスタンス化される。`-O max` では `dead_symbol_elimination` が落とす。それ未満ではコードが生成され、リンク時の `--gc-sections` に委ねられる。

**後続のパスすべてが単純化された木を見る。** act 最適化、`inline`、`decapturing`、`uncurry`、`dead_symbol_elimination` の順に、それぞれが小さい木を扱う。

このパスは最適化レベルで切らない。`eval` を落とすのは最適化ではなく指定された意味なので、`-O none` でも同じように効く必要がある。`optimization::run` 自体はどの最適化レベルでも呼ばれ、個々のパスが内部で `config.enable_*` を見て切っている。新しいパスは `config.skip_eval` だけを見る。全レベルで走るパスは `remove_tyanno` に前例がある。

## 何が消え、何が残るか

### 消えるもの

`{side}` の式そのものが AST から消える。`eval debug_eprintln(...)` の呼び出しは、どの最適化レベルでも実行されない。

`{side}` でしか使われていない `let` 束縛は、`-O none` では LLVM のパスも走らないので、計算されたまま捨てられる。`-O basic` 以上では、副作用のない計算なら LLVM の O3 が落とす。

### 残るもの: `*` で束ねたモナド作用

`eval` でモナドを連鎖させている旧来のコードは、そのまま動く。

`parse_expr_eval` は `{side}` を**外側の do コンテキストのまま**パースし、`{main}` にだけ新しい do コンテキストを与える。したがって `*` が積んだ bind は `Eval` ノードの外側で展開される。

```
main : IO () = (
    eval *println("monadic side");
    println(f(41).to_string)
);
```

を `--emit-symbols` で見ると、インスタンス化後の形は次のとおりである。

```
Main::main = bind(
    |#monadic_value0| (
        eval #monadic_value0;
        println(to_string(f(41)))
    ),
    println("monadic side")
);
```

`{side}` は既に束縛済みの変数 `#monadic_value0` だけである。`Eval` ノードを落としても `bind` と `println("monadic side")` は残るので、アクションは実行される。落ちるのは、既に生成された値を forcing する動作だけである。

裏返せば、`eval *println("debug")` の形で書いたデバッグ出力はこのフラグでは消えない。消えるのは `debug_println` のような、IO を経由しない作用の方である。

### 無関係なもの

`;;` は `Monad::bind` への脱糖であって `Eval` ノードを作らないので、影響を受けない。

`Debug::assert : Lazy String -> Bool -> IO ()` は IO なので `;;` か `*` で連ねる。このフラグは assert を落とさない。実行時検査を落とすのは `--no-runtime-check` の側の役目である。

### 落ちて困りうるもの

`{side}` に IO を経由しない作用を置いた式は消える。

- `eval FFI_CALL[() some_c_function()];` -- C の呼び出しが消える。
- `eval SOME_GLOBAL;` -- グローバル値の call-once 初期化子が走らなくなる。初期化子が `undefined` を含むなら、その中断も起きなくなる。
- `eval arr.assert_unique_array(...)` の形で書いた検査。

これはフラグが約束することそのものである。`--no-runtime-check` と同じく、立てるかどうかはビルドする側の判断になる。

### `Std` の中の `eval`

`std.fix` にある `eval` は `IO::unsafe_perform` の `eval ios;` の 1 箇所だけである。

```
unsafe_perform : IO a -> a = |io| (
    let ios = IOState::_unsafe_create;
    let (ios, res) = (io.@runner)(ios);
    eval ios;
    res
);
```

`ios` は `IOState`、すなわちフィールドを持たない boxed オブジェクトである。`res` が同じ呼び出しの結果である以上 runner の呼び出しは残り、`ios` の解放は RC 挿入が行う。落としても意味は変わらないと読めるが、実装時に**全テストをフラグ ON で走らせて確認する**。ここで壊れるものがあれば、それは std の側に意味を担っている `eval` があるということなので、std を直す。

## 適用範囲

プログラム全体に適用する。std も依存プロジェクトも、ユーザが書いたファイルも区別しない。

- フラグの意味が「このビルドでは `eval` を尊重しない」の 1 文で言える。コードがどのファイルにあるかで効き方が変わる規則は説明しにくい。
- 一番近い隣人である `--no-runtime-check` がプログラム全体に効く。ライブラリが前提にしている検査を落とす力も同じだけ持っている。
- 依存プロジェクトの `eval debug_println` も消える。ユーザが黙らせたいのはそれも含む。

**代案: ルートプロジェクトのファイルだけに適用する。** `Configuration.root_source_files` が既に「ユーザが書いたファイル」を表しており、`collect_deprecation_diagnostics` が同じ判定でデプリケーション警告の範囲を絞っている。`Eval` ノードの `source` のファイルがその集合に入るときだけ落とす、という形で実装できる。このパスは `application_inlining` より前に走るので、`source` を持たない `Eval` ノードはこの時点では存在しない。ライブラリの `eval` の保証を黙って壊さない点では、こちらの方が安全である。

どちらを採るかは判断を仰ぐ。

## 実装

| ファイル | 変更 |
| --- | --- |
| `src/configuration.rs` | `Configuration` に `pub skip_eval: bool` を追加、`Default` で `false`。`object_generation_hash` に混ぜる |
| `src/main.rs` | `--skip-eval` の `Arg` を作り、`build` / `run` / `test` サブコマンドに付ける。引数があれば `config.skip_eval = true` |
| `src/metafiles/project_file.rs` | `ProjectFileBuild` に `#[serde(default)] skip_eval: bool`。`set_config` のルート限定部分で `config.skip_eval = self.build.skip_eval` |
| `src/optimization/skip_eval.rs` | 新規。全シンボルの式を走査し、`Eval` ノードを `main` で置き換える |
| `src/optimization/mod.rs` | `mod skip_eval;` |
| `src/optimization/optimization.rs` | `run` の先頭に、他のパスと同じ形（`StopWatch` と `emit_symbols` のダンプ付き）で挿入 |

パスの本体は `end_visit_eval` で `EndVisitResult::changed(expr.get_eval_main())` を返す 1 アームである。ただし `ExprVisitor` は既定実装を持たないので、`remove_tyanno.rs` と同じく残り 23 個のメソッドを素通しで書くことになる。

`ExprVisitor` に既定実装を与えれば、この定型は 12 個の実装すべてから消える。ただし今は、`Expr` に新しいバリアントを足したとき 12 個すべてがコンパイルエラーになることで、対応漏れが止まっている。その安全性を手放す判断になるので、この変更とは別に決めたい。

### コンパイラオプションのヘルプ

`src/main.rs` に置く `Arg` の定義。`no_runtime_check` と同じ形にする。

```rust
let skip_eval = Arg::new("skip-eval")
    .long("skip-eval")
    .takes_value(false)
    .help(
        "Skip the evaluation instructed by the `eval` syntax: build `eval {expr0}; {expr1}` as `{expr1}`.\n\
        A monadic action bound with `*` inside `{expr0}` is still performed. An effect that does not go \
        through the `IO` monad, such as a call to `Debug::debug_println` or an `FFI_CALL`, is dropped."
    );
```

## キャッシュ

`Configuration::object_generation_hash` に `skip_eval` を混ぜる。オブジェクトファイル全体のキャッシュ (`load_build_object_files_cache`) と分割コンパイル単位のハッシュ (`CompileUnit::update_unit_hash`) の両方がこの関数を通るので、1 箇所でよい。

型検査キャッシュは触らない。除去が型検査の後に起きるので、フラグは型検査の結果を変えない。

## テスト

`src/tests/test_skip_eval.rs` を新設する。`run_source_capture` が stdout と stderr を返すので、消えたかどうかを直接見られる。

- `eval debug_eprintln(...)` を書いたプログラムを、フラグ OFF で走らせるとメッセージが stderr に出て、ON で走らせると出ない。どちらでも `{main}` の値は同じ。
- `eval *println("kept"); ...` をフラグ ON で走らせると "kept" が stdout に出る。`*` で束ねたアクションが残ることを固定する。
- `-O none` / `-O basic` / `-O max` のそれぞれで、フラグ ON のときにメッセージが出ない。最適化レベルに依存しないことを固定する。
- プロジェクトファイルに `skip_eval = true` を書いたプロジェクトを `fix run` して、メッセージが出ない。`src/tests/test_skip_eval/cases/` に置き、`setup_test_env()` の形で一時ディレクトリにコピーする。
- 全テストをフラグ ON で 1 回走らせ、std とテストスイートの中に意味を担っている `eval` が無いことを確かめる。これは恒久的なテストではなく、実装時の確認である。

## ドキュメント

見出しは増やさず、既存の節に段落と表の行を足すだけなので、両方の言語版の目次は変わらない。

### `Document.md` の `eval` syntax の節

Notes の箇条書きの後ろに置く。

> The `--skip-eval` compiler option and the `skip_eval` field of the project file build `eval {expr0}; {expr1}` as `{expr1}`. Write a debugging effect with `eval` while developing, and turn this on to leave it out of the built program.
>
> A monadic action bound with `*` inside `{expr0}` is still performed. The bind that `*` desugars into sits outside the `eval` expression, so what this setting drops is the use of the value the action produced. In
> ```
> main : IO () = (
>     eval *println("Hello");
>     pure()
> );
> ```
> the message is printed with the setting on and with it off.
>
> An effect that reaches the outside world without going through the `IO` monad disappears together with `{expr0}`: a call to `Debug::debug_println` and its siblings, an `FFI_CALL`, and the initializer of a global value the expression names. Reserve `eval` for effects a program built with this setting can do without.

### `Document-ja.md` の `eval`構文 の節

同じ位置に置く。

> `--skip-eval`コンパイラオプションおよびプロジェクトファイルの`skip_eval`フィールドは、`eval {expr0}; {expr1}`を`{expr1}`としてビルドします。開発中は`eval`でデバッグ用の作用を書いておき、この設定を有効にすることで、ビルドされるプログラムからそれを外せます。
>
> `{expr0}`の中で`*`によって束ねられたモナドのアクションは、この設定でも実行されます。`*`が展開するbindは`eval`式の外側にあるため、この設定が落とすのは、アクションが生成した値の使用だけです。例えば
> ```
> main : IO () = (
>     eval *println("Hello");
>     pure()
> );
> ```
> では、この設定の有無にかかわらずメッセージが出力されます。
>
> `IO`モナドを経由せずに外界に届く作用は、`{expr0}`とともに消えます。`Debug::debug_println`とその同類の呼び出し、`FFI_CALL`、そして式が名指ししたグローバル値の初期化子がこれにあたります。この設定でビルドするプログラムでは、`eval`に置く作用を、無くても成り立つものに限ってください。

### プロジェクトファイルのフィールド表

両方の言語版の表の、`no_runtime_check` の行の後ろに 1 行足す。列は Field / Option / Type / Dependent Project / Description である。`create_config` はプロジェクトファイルを読んでからコマンドラインオプションを適用するので、`no_runtime_check` と同じく実効的には論理和になる。表の中は HTML なので、説明の欄にバッククォートは使わない。

`Document.md`:

```html
        <tr>
            <td>skip_eval</td>
            <td>--skip-eval</td>
            <td>Merge (OR)</td>
            <td>Does not affect</td>
            <td>Skip the evaluation instructed by the eval syntax</td>
        </tr>
```

`Document-ja.md`:

```html
        <tr>
            <td>skip_eval</td>
            <td>--skip-eval</td>
            <td>マージ（論理和）</td>
            <td>影響しない</td>
            <td>eval構文が指示する評価を飛ばす</td>
        </tr>
```

### `CHANGELOG.md`

`## [Unreleased]` の `### Added` の `#### Tool` に置く。

> - Added the `--skip-eval` compiler option and the `skip_eval` field of the project file, which build `eval {expr0}; {expr1}` as `{expr1}`. Use it to leave a debugging effect written with `eval` out of a built program. A monadic action bound with `*` inside `{expr0}` is still performed, because the bind it desugars into sits outside the `eval` expression.

`CHANGELOG.md` の `## [Unreleased]` の `### Added` / `#### Tool` に 1 行足す。

## 承認を仰ぐ点

1. **名前。** `--skip-eval` / `skip_eval` を提案する。`--no-eval` は `--no-runtime-check` の形に揃うが、「プログラムを評価しない」とも読める。
2. **適用範囲。** プログラム全体を推奨する。ルートプロジェクトのファイルだけに絞る案も実装可能である。
3. **`fix test` での扱い。** `no_runtime_check` はテストモードで強制的に false に戻される。`skip_eval` も同じにするか、`build.test` セクションでの上書きに任せるか。`Debug::assert` は IO なのでこのフラグでは落ちない。テストが `eval` に頼るのは、`eval debug_println` でテストの進行を出している場合くらいである。強制はせず、`[build.test]` の `skip_eval` で他の設定と同じように上書きできる形を提案する。
