# 仕様: `preliminary_commands` 実行時のユーザ確認

対象ユーザから見た挙動の仕様。実装方針は [design.md](design.md) を参照。

## 背景 / 問題

- `fixproj.toml` の `build.preliminary_commands` および
  `build.test.preliminary_commands` は `fix build` / `fix run` / `fix test`
  実行時に**無確認で自動実行される**。
- 依存プロジェクトの `preliminary_commands` も同様に実行される。
- 結果、悪意あるプロジェクトの clone / 信頼済み依存のアップデート経由で
  任意コード実行が可能 (cargo `build.rs`、npm `postinstall` と同型の問題)。

## 脅威モデル

1. 悪意あるプロジェクトを clone して `fix build` する攻撃
2. 信頼済み依存が後から悪意あるコマンドを追加するサプライチェーン攻撃
3. 承認済み Git 依存が `fix deps update` 後にスクリプト本体や依存コミットを
   差し替える攻撃

いずれも単純な trust-on-first-use では防げない。
また、承認情報をリポジトリにコミットして共有するモデルを採ると、
悪意ある側が「承認済み」を偽造した状態で配布できてしまう。
このため、**承認はユーザごとにホームディレクトリで管理し、
リポジトリには共有しない**方針を採る。

## ユーザから見た挙動

### 承認ストアの所在

承認は対話ユーザごとに `~/.fixtrust.toml` に記録する (ツールが書き込む)。
**リポジトリにはコミットしない** (共有すると偽造承認の踏み台になる)。

### 承認の単位

プロジェクトの種類により承認のキーが異なる:

| 種類 | 承認キー | 意味 |
|---|---|---|
| Git 依存 | `(source, mode, commit_hash)` | 指定コミット・モードの内容を承認 |
| root / ローカルパス依存 | `(source, mode)` | このパス・モードを無期限に承認 |

- `source`:
  - Git 依存: `git+<repo URL>` の形式 (lockfile の `git.repo` から生成)
  - root / ローカルパス依存: **絶対パス** (正規化済み)
- `mode`: `build` または `test`
- `commit_hash`: Git 依存のみ。lockfile の `git.rev`

承認は `(source, mode)` ごとに別レコードで記録する。例えば
`fix build` で root プロジェクトの build モードを承認すると、後に `fix test`
を実行したとき、build モードは再プロンプトされずに通過し、test モードの
`preliminary_commands` のみが新たにプロンプトされる。

「root / ローカルパス依存」はどちらもユーザーの手元のディレクトリを直接指すので、
「このパスに置かれているプロジェクトを信頼する」というパスベースの承認にする。
一度承認すれば、そのパスに何があっても (`git pull` でコミットが変わっても、
コマンドが変わっても) 再プロンプトは出ない。これは意図的な設計で、
自分のプロジェクトでの開発を快適にするための割り切り。
ただしその引き換えに「ローカルパスの信頼 = パスそのものへの白紙委任」となるので、
**他人のプロジェクトを clone した / ローカル参照する場合は軽々しく永続承認
せず、一時承認を使う**ことをドキュメントで強く推奨する (後述)。

### 通常のビルド (対話端末あり)

`fix build` / `run` / `test` で `preliminary_commands` が定義されたプロジェクト
(root または依存) があるとき、以下の流れで処理する:

1. この invocation で実行される `(source, mode)` ペアを全て列挙する
   (`fix build` / `fix run` なら build モード、`fix test` なら build と test
   の両モード)。
2. 各ペアについて `~/.fixtrust.toml` の一致エントリを探す。
   一致したものは「approved」として先に一覧表示し、プロンプトを出さずに通過。
3. 未承認のペアはプロジェクト (`source`) ごとにまとめて、1 プロジェクト 1 回の
   3 択プロンプトを出す:
   - 同じプロジェクトで build と test の両方が未承認なら、両モードのコマンドを
     1 つのブロックにまとめて表示し、`y` で両方まとめて承認する。
   - どちらか片方のみ未承認 (`fix test` で build は別セッションで承認済み等) なら、
     そのモードだけ表示・承認する。
   - 選択肢:
     - `y` = 承認して `~/.fixtrust.toml` に記録 (未承認だった各 `(source, mode)`
       を個別のレコードとして書き込む)、実行
     - `o` = 一時承認 (今回だけ実行、記録しない)
     - `n` / EOF = 拒否
4. `n` が選ばれたら即座にビルド全体を失敗させる
   (残りのプロジェクトは尋ねない)。

プロジェクトの処理順序は実装都合でよい (依存グラフ順など)。

### 承認の記録タイミング / 書き込み失敗時の挙動

`y` が選ばれたら、**コマンド実行の前に** `~/.fixtrust.toml` を更新する。
ビルドが途中で失敗・中断されても承認意思は残る (次回同じ状態で再度 `y` を
押し直す必要はない)。

書き込みに成功したら、保存先パスを 1 行表示する。承認を取り消したいときに
ユーザが直接そのファイルを編集できることを示唆するため:

```
  Choice [y/o/N]: y
  Approved. Recorded to /home/user/.fixtrust.toml (edit the file to revoke).
```

書き込みに失敗した場合 (権限エラー、ディスク不足等) は:

- stderr に警告を出す
- そのセッションは `[o]` (一時承認) と同等に扱ってビルドを続行する

「記録できなかったので拒否」にはしない (ユーザが既に承認を意図しているので、
少なくとも今回のビルドは成立させる)。

### 表示仕様

出力先は stderr。

承認済みプロジェクトは先に一覧表示し、その後、承認待ちのプロジェクトを
1 つずつプロンプトにかける。

承認済みの一覧 (プロンプトなしで通過する場合) の表示例:

```
Preliminary commands (already approved):

  [myproj] (approved)
    path: /home/user/projects/my-app
    $ sh setup.sh
    $ make libfoo.a

  [some-dep] (approved)
    source: https://github.com/foo/some-dep (commit abcdef1)
    path: /home/user/proj/.fix/deps/some-dep_1.2.3
    $ ./configure --prefix=/opt/foo
```

プロンプトの文面はプロジェクト種別により変える。**Git 依存の場合**、
承認はコミットに紐づくので `[y]` のリスクは低い:

```
  [some-dep] (NEW)
    source: https://github.com/foo/some-dep (commit abcdef1)
    path: /home/user/proj/.fix/deps/some-dep_1.2.3
    $ ./configure --prefix=/opt/foo

  Approve?
    [y] Yes — remember for this commit
    [o] Yes — just this run
    [n] No                                      (default)
  Choice [y/o/N]: _
```

**root / ローカルパス依存の場合**、承認は絶対パスに紐づき以後の変更すべてに
及ぶので、その点を明示する:

```
  [myproj] (NEW)
    path: /home/user/projects/my-app
    $ sh setup.sh
    $ make libfoo.a

  Approve?
    [y] Yes — trust this path from now on
          (future changes at /home/user/projects/my-app will not prompt)
    [o] Yes — just this run
          (recommended unless this is your own project)
    [n] No                                      (default)
  Choice [y/o/N]: _
```

変更検知 (Git 依存で commit_hash が不一致) の例:

```
  [other-dep] (CHANGED from commit abcd…)
    source: https://github.com/foo/other-dep (commit ef01234)
    path: /home/user/proj/.fix/deps/other-dep_0.5.0
    $ make install
```

`fix test` で build と test の両モードが**どちらも未承認**の場合は、1 プロンプト
にまとめて表示し、`y` で両方一括承認する:

```
  [myproj] (NEW)
    path: /home/user/projects/my-app
    (build)
    $ sh setup.sh
    $ make libfoo.a
    (test)
    $ ./setup_test_env.sh
```

`fix test` で **build は既に承認済みで test のみ未承認**の場合は、test モードだけ
表示する (build モードの commands は approved 一覧側に入る):

```
  [myproj] (NEW — test mode)
    path: /home/user/projects/my-app
    (test)
    $ ./setup_test_env.sh
```

- プロジェクト名は `fixproj.toml` の `[general] name` をそのまま使う
  (root / 依存の別は表示しない)。
- ソース表示:
  - Git 依存: `source: <URL> (commit <short-hash>)` と `path: <install path>` を
    両方表示。URL はクリックで GitHub 等に飛べる形にする。
  - root / ローカルパス依存: `path: <absolute path>` のみ表示
    (`source` に相当する上流情報はないため)。
  - `path:` はコマンドを実行する際の cwd を兼ねる。
- Status は以下のいずれか:
  - `NEW` — 承認履歴なし
  - `CHANGED from commit abcd…` — Git 依存で以前のコミット承認があるが現在と不一致
  - `approved` — `~/.fixtrust.toml` の一致エントリあり
  - `auto-approved (--allow-preliminary-commands)` — フラグによる一括一時承認
- コマンドは `$ ` プレフィックス + POSIX shell-escape で 1 行表示。

### 非対話環境 (CI, パイプ等)

対話端末がない状態で未承認のコマンドに出会った場合は、
プロンプトを出さずに即失敗させる。失敗メッセージの例:

```
error: preliminary commands require approval, but no interactive terminal is available.

  [myproj] (NEW)
    path: /home/user/proj
    $ sh setup.sh
    $ make libfoo.a

  [some-dep] (CHANGED from commit abcd…)
    source: https://github.com/foo/some-dep (commit ef01234)
    path: /home/user/proj/.fix/deps/some-dep_1.2.3
    $ ./configure --prefix=/opt/foo

To approve:
  - Run fix in an interactive terminal and answer the prompt.
  - Or pass --allow-preliminary-commands to bypass for this invocation only.
```

対話環境でユーザが `n` を選んで拒否した場合は、未承認コマンド一覧を
再掲せず以下のみを表示して終了する:

```
error: preliminary commands not approved. aborted.
```

### オプトアウト / バイパス

- `--allow-preliminary-commands`: 全プロジェクトに対して `[o]` (一時承認)
  を与えたのと等価。`~/.fixtrust.toml` は更新しない。CI 用。

### 推奨される使い分け

| 状況 | 推奨 |
|---|---|
| 自分のプロジェクト (root / ローカルパス依存) | `y` (remember) |
| 他人のプロジェクトを clone してビルド | `o` (once) |
| 他人のプロジェクトをローカル参照 | `o` (once) |
| CI / 自動ビルド | `--allow-preliminary-commands` |

他人のコードを走らせる以上、その内容を精査すべきで、永続承認 (`y`) は
長期的に script 差し替え攻撃の入り口になる。

### `~/.fixtrust.toml` の内容

ユーザが直接読める TOML 形式で、承認ごとに 1 エントリを保持。

```toml
# Git 依存 (build モード): commit ごとに 1 エントリ
[[approval]]
source = "git+https://github.com/foo/bar"
mode = "build"
commit_hash = "abcdef..."
approved_at = "2026-04-20T10:30:00Z"
# 参考情報 (照合には使わない)
project_name = "bar"
commands_preview = ["./configure --prefix=/opt/foo"]

# root プロジェクト (build モード): パスベースで無期限
[[approval]]
source = "/home/user/projects/my-app"
mode = "build"
approved_at = "2026-04-20T10:31:00Z"
project_name = "my-app"
commands_preview = ["sh setup.sh", "make libfoo.a"]

# 同じ root プロジェクトの test モードは別エントリ
[[approval]]
source = "/home/user/projects/my-app"
mode = "test"
approved_at = "2026-04-20T10:31:00Z"
project_name = "my-app"
commands_preview = ["./setup_test_env.sh"]
```

照合は `(source, mode, commit_hash)` で行う。`commit_hash` フィールドが
エントリに存在しない場合 (root / ローカル依存) は省いて `(source, mode)`
で照合する。

配置場所の詳細 (ファイルロック、書き込み原子性など) は設計側で決定。

## 識別子の対象

### `commit_hash` (Git 依存のみ)

lockfile に記録されているコミットハッシュ (`DependencyLockGit::rev`) を
そのまま使う。

### `commands_hash` は照合に使わない

コマンド内容のハッシュは `~/.fixtrust.toml` には `commands_preview` として
参考表示のみ残す。照合には使わない。理由:

- Git 依存: コミットハッシュが内容を決定するので、`commit_hash` が一致すれば
  コマンドも一致している。二重に持つ意味がない。
- root / ローカルパス依存: 絶対パスへの信頼をユーザが与えた時点で、そのパス
  配下のコマンド変更も含めて委任されたと解釈する (パスの持ち主 = 自分)。

## 既知の限界

### スクリプト経由の間接実行 (同一コミット内)

`["sh", "setup.sh"]` のようにスクリプトを呼ぶ場合、同一コミット内で
`setup.sh` の中身を差し替えても検知できない。

Git 依存については `fix deps update` でコミットが変われば再プロンプトされる
ので、そこでの差し替えは検知できる。しかしコミット内の動的差し替えは
原理的に防げない。これは「スクリプト実行を承認 = そのスクリプトに白紙委任」
である以上本仕様の対象外とし、ユーザ教育で補う。

### ローカルパスへの白紙委任

root / ローカル依存を `y` で承認すると、そのパス配下のコマンド・スクリプトが
以後どう変わっても再プロンプトは出ない。悪意ある第三者がそのパスに書き込める
状況 (マシン侵害) では防御にならないが、そこまで侵害された時点でこの仕様の
守備範囲外と判断する。

clone してきた他人のプロジェクトを `y` で承認するのは **非推奨**。`o` を使う。

### 依存解決との関係

依存を追加した瞬間にその `preliminary_commands` が起動し得るため、
最低でも `fix build` の初回には全依存ぶんのプロンプトが順に出る可能性がある。
数が多いときは煩雑になりうるが、プロジェクト単位で個別に判断できる利点と
引き換え。一時承認で済ませたいなら `--allow-preliminary-commands`。

## デフォルトの方針

**デフォルトで安全側に倒す**。すなわち、何も指定しない `fix build` では
上記のプロンプト動作が働く。

npm が `ignore-scripts=false` をデフォルトのまま放置して攻撃に使われている
状況を反面教師とする。Fix はまだエコシステムが小さく、デフォルトを
安全側に切る破壊的影響が限定的なうちに実施する。
