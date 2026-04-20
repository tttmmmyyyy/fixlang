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

単純な trust-on-first-use では 2 を防げないため、「変更検知」を含む仕様にする。

## ユーザから見た挙動

### 通常のビルド (対話端末あり)

`fix build` / `run` / `test` で `preliminary_commands` が定義されたプロジェクト
(root または依存) があるとき:

1. 実行されるコマンド一覧を表示する (詳細は下記「表示仕様」)。
2. 以下のいずれかの条件で y/N プロンプトを出す:
   - そのプロジェクト+モードの承認履歴がない (**初回**)
   - 承認履歴があるが、現在のコマンド内容のハッシュが記録と一致しない
     (**変更検知**)
3. `y` なら承認を **ロックファイルに記録** した上でコマンドを実行する。
4. `N` (デフォルト) / EOF ならビルドを失敗させる。
5. 承認済みかつハッシュが一致する場合はプロンプトを出さずに実行する
   (ただしコマンド表示は常時行う)。

### 表示仕様

1 つの `(project, mode)` ごとに以下を表示する。出力先は stderr。

例 (承認待ち):

```
Preliminary commands to approve:

  [myproj] (NEW)
    cwd: /home/user/proj
    $ sh setup.sh
    $ make libfoo.a

  [some-dep] (CHANGED from sha256:abcd…)
    cwd: /home/user/proj/.fix/deps/some-dep
    $ ./configure --prefix=/opt/foo

Approve and run? [y/N]:
```

例 (承認済みで通過):

```
Preliminary commands:

  [myproj] (approved)
    $ sh setup.sh
    $ make libfoo.a
```

- プロジェクト名は `fixproj.toml` の `[general] name` をそのまま使う
  (root / 依存の別や、どのフィールド由来かは表示しない)。
- Status は `NEW` / `CHANGED` / `approved` / `auto-approved (--allow-preliminary-commands)` のいずれか。
- コマンドは `$ ` プレフィックス + POSIX shell-escape で 1 行表示。
- 承認待ちが複数グループあるときは縦に並べ、**最後に 1 回** `y/N` を問う
  (部分承認はなし)。

### 非対話環境 (CI, パイプ等)

対話端末がない状態で未承認のコマンドに出会った場合は、
プロンプトを出さずに即失敗させる。失敗メッセージは以下の形式:

```
error: preliminary commands require approval.

  [myproj] (NEW)
    cwd: /home/user/proj
    $ sh setup.sh
    $ make libfoo.a

  [some-dep] (CHANGED from sha256:abcd…)
    cwd: /home/user/proj/.fix/deps/some-dep
    $ ./configure --prefix=/opt/foo

Reason: no interactive terminal available.

To approve:
  - Run fix in an interactive terminal and answer `y` at the prompt.
  - Or pass --allow-preliminary-commands to bypass for this invocation only.
```

対話環境でユーザが `N` を押して拒否した場合は、未承認コマンド一覧を
再掲せず以下のみを表示して終了する:

```
error: preliminary commands not approved. aborted.
```

CI で回すときは以下のいずれかで明示オプトイン:

- コマンドラインフラグ `--allow-preliminary-commands` で全承認しつつ実行。
- または、承認済みのロックファイルをリポジトリにコミットしておく。
  この場合ハッシュ一致すれば無プロンプトで通る。
  不一致なら失敗 (プロンプトには降格しない)。

### オプトアウト / バイパス

- `--allow-preliminary-commands`: 今回のコマンドを無条件承認して実行する。
  **ロックファイルは更新しない** (単なる一時バイパスのため)。

ロックファイルへの記録は対話プロンプトで `y` と答えたときだけ行う
(「熟慮された承認」だけを永続化する)。CI 等で承認を共有したい場合は、
開発者が対話環境で一度 `y` を押してロックファイルを更新→commit する。

### ロックファイルに記録される内容

ユーザが直接読める形式 (例: TOML) で、次の情報を保持:

- プロジェクト識別
- モード (build / test)
- コマンド配列のハッシュ (SHA-256)
- 人間向け参考情報 (承認時点のコマンド内容、承認日時)

ファイル名・配置場所は設計側で決定。リポジトリにコミットして共有する運用を想定。

## ハッシュの対象

**`preliminary_commands` 配列の正規化シリアライズを SHA-256**。

「正規化シリアライズ」は以下で定義する:

- 入力: `Vec<Vec<String>>` (コマンドの配列、各コマンドは argv の配列)
- 出力: UTF-8 の JSON 配列リテラルのバイト列
- 規則:
  - 外側・内側とも配列順序は入力のまま保つ (ソートしない)
  - 文字列は JSON 標準のエスケープを適用
    (`"` / `\` / 制御文字 `\u0000`–`\u001F` のみ)
  - 空白・改行・インデントは挿入しない (compact 形式)
  - 非 ASCII 文字は UTF-8 バイトのまま (`\uXXXX` エスケープはしない)

例: `[["sh", "setup.sh"], ["make", "libfoo.a"]]` は
`[["sh","setup.sh"],["make","libfoo.a"]]` の 41 バイトが入力となる。

実装は `serde_json::to_vec` 相当の出力と一致する。

採用理由:

- 攻撃面は「Fix ビルド時に任意コード実行される」という一点で、
  その実体は `preliminary_commands` の文字列そのもの。
- コマンドが不変なら依存側が他の箇所をどう変えても新たな攻撃は発生しない。
- コミットハッシュや `fixproj.toml` 全体のハッシュを使うと、
  無関係な変更で再プロンプトが多発してアラーム疲労を起こし、
  ユーザが反射的に yes を押す癖がつく → 検知能力が下がる。

## 既知の限界

### スクリプト経由の間接実行

`["sh", "setup.sh"]` のようにスクリプトを呼ぶ場合、`setup.sh` の中身は
ハッシュ対象外なので後から差し替え可能。

これは本質的に「スクリプト実行を承認 = そのスクリプトに白紙委任」である
ため、原理的に防ぎきれない。本仕様の対象外とし、ユーザ教育で補う。

### 依存解決との関係

依存を追加した瞬間にその `preliminary_commands` が起動し得るため、
最低でも `fix build` の初回には全依存ぶんのプロンプトが出る可能性がある。
UX 上は一覧表示して一括承認できるのが望ましい (実装詳細は設計側)。

## デフォルトの方針

**デフォルトで安全側に倒す**。すなわち、何も指定しない `fix build` では
上記のプロンプト動作が働く。

npm が `ignore-scripts=false` をデフォルトのまま放置して攻撃に使われている
状況を反面教師とする。Fix はまだエコシステムが小さく、デフォルトを
安全側に切る破壊的影響が限定的なうちに実施する。
