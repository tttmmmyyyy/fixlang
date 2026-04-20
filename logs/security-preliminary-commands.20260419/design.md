# 設計: `preliminary_commands` 実行時のユーザ確認

仕様は [spec.md](spec.md) を参照。本書は実装方針を扱う。

## 現状把握

- `preliminary_commands` は `ExtraCommand` に変換されて
  `Configuration::extra_commands` に積まれる
  ([src/metafiles/project_file.rs:598-617](../../src/metafiles/project_file.rs#L598-L617))。
- 実行は `Configuration::run_extra_commands`
  ([src/configuration.rs:642](../../src/configuration.rs#L642)) で、
  各 `ExtraCommand::run` ([src/configuration.rs:244-275](../../src/configuration.rs#L244-L275))
  が `std::process::Command::status()` で同期実行する。
- **既存の `ExtraCommand` というネーミングはユーザ向け名 (`preliminary_commands`)
  と乖離しているため、本改修で `PreliminaryCommand` にリネームする** (後述)。
- 呼び出し元は [src/build/build.rs:21-24](../../src/build/build.rs#L21-L24) 1 箇所
  (subcommand が Build/Run/Test のときのみ)。
- **依存プロジェクトの `preliminary_commands` も積まれる**。
  `set_config_from_proj` の dependent 用早期 return は
  [src/metafiles/project_file.rs:629](../../src/metafiles/project_file.rs#L629)
  にあるが、preliminary_commands の push はそれより前の行 598-617 で行われる。
- lockfile のエントリ `DependencyLockFileEntry` は `git: Option<DependencyLockGit>`
  を持ち、Git 依存は `git.rev` にコミットハッシュを持つ
  ([src/dependency/lockfile.rs:321-360](../../src/dependency/lockfile.rs#L321-L360))。
  ローカルパス依存は `git = None`。

## 全体構成

次の 4 モジュールを追加/改修する。

1. **`ExtraCommand` → `PreliminaryCommand` リネーム + 拡張** — source, mode,
   project_name を持たせる
2. **Trust store モジュール** — `~/.fixtrust.toml` の読み書き
3. **`run_preliminary_commands` (旧 `run_extra_commands`) の書き換え** —
   照合 → プロンプト → 実行
4. **CLI フラグ** — `--allow-preliminary-commands`

## モジュール別設計

### 1. `PreliminaryCommand` リネーム + 拡張

既存シンボルをまず機械的にリネームする (3 ファイル、計 10 箇所程度):

- `struct ExtraCommand` → `struct PreliminaryCommand`
- `Configuration::extra_commands` → `Configuration::preliminary_commands`
- `Configuration::run_extra_commands` → `Configuration::run_preliminary_commands`
- `ExtraCommand::run` → `PreliminaryCommand::run`

対象: [src/configuration.rs](../../src/configuration.rs),
[src/metafiles/project_file.rs](../../src/metafiles/project_file.rs),
[src/build/build.rs](../../src/build/build.rs)。

そのうえで、[src/configuration.rs](../../src/configuration.rs) の
`PreliminaryCommand` を拡張:

```rust
pub struct PreliminaryCommand {
    pub work_dir: PathBuf,
    pub command: Vec<String>,
    // 追加:
    pub project_name: String,           // fixproj.toml の [general] name (表示用)
    pub mode: PreliminaryMode,          // Build | Test
    pub source: PreliminaryCommandSource,  // 承認キーの source 部分
}

pub enum PreliminaryMode { Build, Test }

pub enum PreliminaryCommandSource {
    Local(PathBuf),                     // root / ローカルパス依存 (絶対パス)
    Git { url: String, commit: String },// git 依存 (lockfile の rev を同梱)
}
```

`PreliminaryCommandSource` のシリアライズ (trust file の `source` フィールド):

- `Local(p)` → `p` の絶対パス文字列
- `Git { url, .. }` → `"git+{url}"`

登録側
([src/metafiles/project_file.rs:598-617](../../src/metafiles/project_file.rs#L598-L617))
では、ProjectFile とその親の lockfile エントリから source を決定する:

- root または lockfile の `git == None` なエントリ → `Local(abs_path)`
- lockfile の `git == Some(DependencyLockGit { repo, rev })` → `Git { url: repo, commit: rev }`

root は lockfile に載らないため、`ProjectFile::read_root_file` から来た
ProjectFile は常に `Local` で、絶対パスは `proj_file.path.parent()` から取る。

### 2. Trust store (`~/.fixtrust.toml`)

**ファイル配置**: `dirs::home_dir().join(".fixtrust.toml")`。
既存 `FIX_CONFIG_FILE_NAME` と同じパターン
([src/constants.rs:140](../../src/constants.rs#L140))。

**フォーマット**: TOML (spec.md 参照)。

```toml
[[approval]]
source = "git+https://github.com/foo/bar"
mode = "build"
commit_hash = "abcdef..."
approved_at = "2026-04-20T10:30:00Z"
project_name = "bar"
commands_preview = ["./configure --prefix=/opt/foo"]
```

**新規モジュール**: `src/trust_store.rs`。

```rust
pub struct TrustStore {
    approvals: Vec<Approval>,
}

pub struct Approval {
    pub source: String,
    pub mode: PreliminaryMode,
    pub commit_hash: Option<String>,    // Git 依存のみ
    pub approved_at: DateTime<Utc>,
    pub project_name: String,
    pub commands_preview: Vec<Vec<String>>,
}

impl TrustStore {
    pub fn load() -> Result<Self, Errors>;   // 存在しなければ空、パースエラーは警告+空扱い
    pub fn is_approved(&self, src: &PreliminaryCommandSource, mode: PreliminaryMode) -> bool;
    pub fn record(&mut self, approval: Approval);
    pub fn save(&self) -> Result<(), Errors>;  // atomic: tempfile + rename
}
```

**照合ロジック** (`is_approved`):

- `source` を文字列化した値、`mode`、`commit_hash` の 3 つで一致判定
- Git 依存 (`PreliminaryCommandSource::Git`) は commit_hash 必須一致
- ローカル依存 (`PreliminaryCommandSource::Local`) は `commit_hash` を問わない
  (エントリ側に commit_hash があっても無くても source+mode で一致すれば良し)

**保存の原子性**:

- `~/.fixtrust.toml.tmp` に書いて `rename` で差し替え
- `record()` 1 回につき `save()` 1 回 (承認意思の即時永続化のため)

**書き込み失敗時**:

- `save()` が Err を返したら、呼び出し側で `warn_msg` を出し、そのセッションは
  一時承認相当で続行させる (spec.md 「承認の記録タイミング / 書き込み失敗時の挙動」)

**パース失敗時**:

- 手で編集して壊した場合や新形式との不整合 → 警告を出して空の TrustStore として
  扱い、プロンプトからやり直させる (上書き保存で自動修復される)

### 3. `run_preliminary_commands` の書き換え

リネーム後の `run_preliminary_commands`
([src/configuration.rs:642](../../src/configuration.rs#L642) 相当) を
次の流れに置き換える:

```
1. config.preliminary_commands を (source, mode) でグルーピング
2. TrustStore::load()
3. 各グループを承認済み/未承認に振り分け
4. 承認済みグループを「approved」として stderr に一覧表示
5. 未承認グループがあれば:
     a. --allow-preliminary-commands が指定されていれば:
          「auto-approved」として表示し、TrustStore は更新せず実行フェーズへ
     b. 非対話環境 (stdin が tty でない) なら:
          未承認一覧を表示して失敗
     c. 対話環境なら:
          - 未承認を source で再グルーピング
          - source ごとに 1 プロンプト (3 択 y/o/n):
              * build と test が両方未承認ならまとめて 1 ブロック表示
              * どちらか片方のみ未承認ならそのモードだけ表示
              * プロンプト文面は source 種別 (Git / Local) で切り替える
          - `y`: そのプロジェクトで未承認だった各 (source, mode) の Approval を
                TrustStore に追加 → save()。成功したら保存先パスを stderr に
                1 行表示 (`Recorded to <path> (edit the file to revoke).`)。
                失敗時は警告のみで続行
          - `o`: TrustStore 更新なし
          - `n` / EOF: 即失敗 (以降のプロンプトは出さない)
6. 全グループが承認 (永続または一時) 済みになったら、config.preliminary_commands
   を順次実行 (`PreliminaryCommand::run` をそのまま使う)
```

**プロンプト実装**:

- tty 判定: `std::io::IsTerminal::is_terminal(&std::io::stdin())` (Rust 1.70+)
- 入力は stdin から 1 行読み、先頭文字を小文字化して `y` / `o` / `n` にマップ。
  それ以外 / EOF / 空行 はデフォルトの `n` 扱い
- 出力は全て stderr (ビルド出力の stdout と混ざらないように)
- 依存ライブラリは増やさず自作で十分

**表示フォーマット**: spec.md の表示仕様セクションに準拠。

- Git 依存: `source: <url> (commit <short>)` + `path: <install path>`
- root / ローカル: `path: <abs path>` のみ
- build/test 両方の場合は `(build)` / `(test)` ヘッダで区切り

### 4. CLI フラグ

`fix` 本体の clap 定義で build/run/test サブコマンドに
`--allow-preliminary-commands` を追加。`Configuration` に bool として引き渡す。

## テスト

統合テスト (`src/tests/` 配下、新規 `test_preliminary_commands.rs` を想定)。
[CLAUDE.md](../../CLAUDE.md) に沿い、`install_fix()` + `Command::new("fix")`
+ `setup_test_env()` パターンを用いる。

trust store はホームディレクトリに置かれるため、テストでは以下のいずれかで隔離:

- `HOME` 環境変数をテンポラリパスに差し替える (プロセスレベルの env 操作)
- または `TrustStore::load/save` がパスを受け取る形にして、テスト時だけ差し込む

後者の方が副作用が少ないので推奨。本番は `home_dir` から解決する薄いラッパで呼ぶ。

ケース:

- (A) `preliminary_commands` なしのプロジェクト → 何も起きない
- (B) 初回実行 + `--allow-preliminary-commands` → 通るが trust store は更新されない
- (C) 初回実行 + stdin 閉じ → 失敗 (非対話判定)
- (D) trust store に一致エントリあり → 無プロンプトで通る
- (E) trust store にエントリあるが commit_hash 不一致 (Git 依存) → CHANGED 表示 + 非対話なら失敗
- (F) 依存側 (Git / ローカル両方) に `preliminary_commands` があるケース
- (G) `fix build` で build を承認 → 続けて `fix test` で build は無プロンプト、test のみ承認
- (H) 両モード未承認の `fix test` で `y` → build/test 両方のエントリが記録される
- (I) `o` を選ぶと trust store は更新されない
- (J) trust store 書き込み失敗 (read-only 等) → 警告+続行

(B)(C)(E)(G)(H)(I)(J) は trust store の内容と stderr を検査して確認する。

対話入力のテストは stdin に `y\n` / `o\n` / `n\n` / `` (空) を流し込むことで行う
(`Command::stdin(Stdio::piped())` + `child.stdin.write_all`)。

## マイグレーション

既存プロジェクトへの影響:

- 既に `preliminary_commands` を書いているプロジェクトはデフォルト挙動が変わる
  (無確認実行 → 承認必須)
- 対処ガイド:
  - ローカル開発者: 初回ビルド時に `y` を押せば `~/.fixtrust.toml` に記録され、
    以降は無プロンプトで通る (root/ローカル依存ならパス変えない限り永続)
  - CI: `--allow-preliminary-commands` をビルドコマンドに付ける
- `Document.md` / `Document-ja.md` の `preliminary_commands` 記述に
  この挙動変更を追記する

## 実装順序

1. `ExtraCommand` → `PreliminaryCommand` 機械的リネーム (単独 PR)
2. `PreliminaryCommand` 拡張 (`source`, `mode`, `project_name`) + 登録側の修正
3. `TrustStore` モジュール (load/save/is_approved/record)
4. `run_preliminary_commands` 書き換え + プロンプト実装
5. CLI フラグ `--allow-preliminary-commands`
6. 統合テスト
7. ドキュメント更新

1 は純粋なリネームで独立。2〜3 は並行可能。4 以降は前段が揃ってから。

## 未決事項

- プロンプト時に argv だけでなくスクリプト実体の一部をプレビューするか
  (本仕様では対象外だが、将来の UX 強化候補)
- `~/.fixtrust.toml` が肥大化した場合のディレクトリ分割 (将来の検討)
