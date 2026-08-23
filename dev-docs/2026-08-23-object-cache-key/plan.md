# ビルドのキャッシュの鍵を、列挙漏れが起きない形にする

## 何が壊れているか

`fix` は生成したオブジェクトファイルを再利用する。再利用してよいのは「同じコードを生成する
ビルド」の間だけなので、鍵は**生成されるコードを変える設定をすべて**含んでいなければならない。
いまの鍵はその設定を手で列挙していて、列挙は 6 か所で漏れている。漏れた設定を切り替えた
2 度目のビルドは、前のビルドのオブジェクトを黙って使う。

対になるテスト `test_object_generation_hash_separates_code_generation_settings` は
**列挙したものが鍵を動かすこと**しか見ていない。生成コードを変える設定が全部列挙されて
いるかは誰も見ていないので、列挙漏れはテストを通る。`Configuration` の doc コメントが
「生成コードを変えるフィールドは `object_generation_hash` に加えること」と述べているのが、
いまある唯一の歯止めである。

## いまの形

ビルドが読み書きする鍵は 2 つある。

- `build_object_files_cache_hash` -- ビルド 1 回分のオブジェクトファイルの一覧を指す鍵。
  `Configuration::object_generation_hash` と、各モジュールのソースのハッシュを畳む。
  `build_object_files` はこの鍵で当たった時点で、オブジェクトファイルの一覧を返して**即座に戻る**。
- `CompileUnit::update_unit_hash` -- コンパイル単位 1 つのオブジェクトファイルを指す鍵。
  `object_generation_hash` と、その単位が持つシンボルと、依存モジュールのソースを畳む。

どちらも `object_generation_hash` を通るので、列挙漏れは両方に効く。

## 直す 7 点

### 1. 列挙漏れを、テストではなくコンパイルエラーで止める (#478 = #287, #256 = #434, #292, #286)

`object_generation_hash` と `elaboration_hash` を、`Configuration` の**すべてのフィールドを
書き出す分解束縛**から始める。

```rust
let Configuration {
    // 生成されるコードに届く。
    fix_opt_level, debug_info, ..., develop_mode, emit_symbols, host_cpu,
    // 届かない。理由を群ごとに述べる。
    ld_flags: _, linked_libraries: _, ...,
} = self;
```

`Configuration` にフィールドを足した人は、この 2 か所が**コンパイルできなくなる**ので、
その設定が生成コードに届くかを必ず答えることになる。いまの doc コメントと片側だけのテストが
頼っている「2 か所を同時に思い出す」ことが要らなくなる。

この分解束縛と同時に、いま漏れている 4 つを鍵へ入れる。

| issue | 設定 | 漏れると起きること |
| --- | --- | --- |
| #478 = #287 | `develop_mode` | 検査入りのビルドが、検査の入っていないオブジェクトを再利用する |
| #286 | `emit_symbols` | シンボル名を書き換えたビルドと書き換えないビルドがオブジェクトを共有する |
| #292 | ホストの CPU | 別の機械が作った、その機械に無い命令を含むオブジェクトを再利用する |
| #256 = #434 | `max_cu_size` | 2 度目以降のビルドで `--max-cu-size` が黙って無視される |

`max_cu_size` だけは `object_generation_hash` ではなく `build_object_files_cache_hash` に置く。
この設定が決めるのは**単位の分け方**であって、単位 1 つが生成するコードではない。上限を上げた
ビルドは、上限より小さい単位のオブジェクトをそのまま再利用してよい。

### 2. ホストの CPU を、コード生成と鍵で 1 つの値から引く (#292)

`get_target_machine` は `TargetMachine::get_host_cpu_name()` と `get_host_cpu_features()` を
その場で呼んでコードを生成する。鍵が集めているのは `disable_cpu_features_regex` -- **無効化する
機能の正規表現**だけで、CPU 名も、結果として有効になった機能の集合も入っていない。正規表現は
CPU の代理でしかなく、代理は元の値が変わったときに動かない。

`Configuration` に `host_cpu` を持たせ、`Configuration::new` が 1 度だけ読む。
`get_target_machine` はそこから引き、鍵もそこから引く。値が 1 つになるので、コード生成が
使う CPU と鍵が指す CPU が食い違わない。

### 3. `fix build` と `fix run` がオブジェクトを共有する (#285)

鍵は `subcommand.command_type_string()` (`"build"` / `"run"` / `"test"`) を混ぜている。
`subcommand` がコード生成に届く経路は 1 つだけで、`elaborate_via_config` が
`matches!(config.subcommand, SubCommand::Test)` でエントリポイントを選ぶところである。
`Build` と `Run` はどこでも同じ道を通る。

鍵に混ぜるものを、コマンドの種類から**エントリポイントの選び方**へ変える。同じ判定を
elaboration と鍵の 2 か所が別々に書いている形もここで 1 つにまとめる
(`Configuration::entry_point_runs_tests`)。`Test` は今までどおり分かれる。

### 4. ダンプを求めたビルドは、キャッシュを読まない (#211, #400)

`--emit-llvm` / `--emit-rc-ir` / `--emit-symbols` は、生成の途中でファイルを書き出す。
書き出す 3 か所はすべて、キャッシュが当たって戻る門より後ろにある。したがって一度ビルドした
ディレクトリで同じコマンドをもう一度走らせると、**ダンプは 1 つも書かれない**。エラーも警告も
出ず、終了コードは 0 である。前のビルドのダンプが残っていれば、それが今回のビルドのものとして
読まれる (`-O basic` のダンプを `-O max` のものとして読む形が #400 に実測されている)。

ダンプを求められたビルドは、ビルド全体のキャッシュも単位ごとのキャッシュも読まない形にする。
ダンプは生成の途中でしか書けないので、生成を飛ばした単位はダンプを書けない。代償はダンプを
求めたビルドが毎回作り直すことで、ダンプを求めるのは調査のときだけなので釣り合う。

ダンプの指定を鍵に入れる直し方は採らない。この指定はオブジェクトの中身を変えないので、
鍵に入れるとダンプ付きとダンプ無しでオブジェクトを 2 セット持つことになる。

### 5. 鍵を作る連結を、境目が値に依存しない形にする (#361)

鍵は値を連結したものから計算する。規約は「隣り合う値の境目が値の形に依存しないこと」で、
これが破れると 2 つの違う入力が 1 つの鍵を共有する。`HashSource` はこの規約を型に閉じた
もので (値ごとに固定長のハッシュを足す)、4 か所が通っている。通っていないのが 4 か所ある。

- `build.rs` の `runtime_obj_hash_source` -- ランタイムのオブジェクトの名前。可変長の
  マクロ列を `_` で繋いだものに、`"exe"` / `"dylib"` と `"none"` / `"thread"` が区切り無しで続く。
- `project_file.rs` の `calculate_dependencies_hash` -- ロックファイルを作り直すかの判定。
  依存の JSON と、パス依存のプロジェクトファイルの内容そのものが、どちらも区切り無しで続く。
- `Symbol::hash` と `CompileUnit::update_unit_hash` -- `<name>` のようなマーカーで区切る。
  マーカーは値の中にも書ける文字列である。

4 か所を `HashSource` に載せる。

## 防壁

- **分解束縛** (1)。列挙漏れがコンパイルエラーになる。設定が増えたときに鍵が追随することを、
  人の記憶ではなく型が保証する。
- **単体テスト**。`test_object_generation_hash_separates_code_generation_settings` に、
  今回鍵へ入れる設定の行を足す。`fix build` と `fix run` が同じ鍵になることも 1 本で述べる。
- **統合テスト**。上の 4 点はどれも「同じディレクトリで 2 度ビルドしたときだけ」現れる。
  ダンプを読む既存のテストはどれも新しい一時ディレクトリで 1 度だけビルドするので、この軸を
  持つテストが 1 本も無い。同じディレクトリで 2 度ビルドするテストを置く。

## 閉じる issue

#478, #287 (同一), #256, #434 (同一), #292, #286, #211, #400 (同一), #285, #361。

## 付録: コードを読んで確かめたこと

今日の `main` (`e7aed75c`) で確認した。

- `Configuration::object_generation_hash` が集めているもの: `fix_opt_level`, `debug_info`,
  `compilation_directory` (`debug_info` のときだけ), `threaded`, `sanitizer`, `backtrace`,
  `no_runtime_check`, `skip_eval`, `c_type_sizes`, `max_split_scalars`, `output_file_type`,
  `disable_cpu_features_regex`, `llvm_passes()`, `subcommand.command_type_string()`,
  `build_time_utc!()`。`develop_mode`, `emit_symbols`, `max_cu_size`, ホストの CPU は無い。
- `subcommand` がコード生成に届く経路は `elaboration/mod.rs` の
  `instantiate_entry_io_value(&typechecker, matches!(config.subcommand, SubCommand::Test))` だけ。
  他の読み手 (`build_mode`, `produces_output_file`, `typecheck`, 診断の許容) は、どのソースを
  集めるか・何を出力するか・型検査をどう回すかを決めるもので、単位のコードには届かない。
- `enable_separated_compilation` は `fix_opt_level` から導かれるので、鍵に既に入っている。
- ランタイムのオブジェクトは別の鍵 (`runtime_obj_hash_source`) で名前が決まる。その入力は
  マクロ列・`output_file_type`・`sanitizer`・`no_elim_frame_pointers()` で、最後の 1 つは
  `backtrace` から導かれ、`backtrace` はマクロ列に `BACKTRACE` として入る。
