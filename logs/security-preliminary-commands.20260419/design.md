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
- 呼び出し元は [src/build/build.rs:21-24](../../src/build/build.rs#L21-L24) 1 箇所
  (subcommand が Build/Run/Test のときのみ)。
- **依存プロジェクトの `preliminary_commands` も積まれる**。
  `set_config_from_proj` の dependent 用早期 return は
  [src/metafiles/project_file.rs:629](../../src/metafiles/project_file.rs#L629)
  にあるが、preliminary_commands の push はそれより前の行 598-617 で行われる。

## 全体構成

次の 4 モジュールを追加/改修する。

1. **`ExtraCommand` の拡張** — プロジェクト識別・モードを持たせる
2. **ハッシュ計算** — 正規化 + SHA-256
3. **Trust lockfile モジュール** — 読み書きとエントリ検索
4. **`run_extra_commands` の書き換え** — 照合 → プロンプト → 実行

## モジュール別設計

### 1. `ExtraCommand` の拡張

[src/configuration.rs](../../src/configuration.rs) の `ExtraCommand` に以下を追加:

```rust
pub struct ExtraCommand {
    pub work_dir: PathBuf,
    pub command: Vec<String>,
    // 追加:
    pub project_name: String,        // fixproj.toml の [general] name
    pub mode: PreliminaryMode,       // Build | Test
}

pub enum PreliminaryMode { Build, Test }
```

`[general] name` は必須フィールドなので常に埋まる。
登録側
([src/metafiles/project_file.rs:598-617](../../src/metafiles/project_file.rs#L598-L617))
で `project_name` と `mode` を適切に埋める。

プロジェクト名は依存ツリー内で一意である前提に立つ (cargo/npm 同様)。
root と同名の依存があると lockfile エントリが衝突する可能性があるが、
エコシステム上のコンベンションとして避ける想定。

### 2. ハッシュ計算

```rust
fn hash_preliminary_commands(commands: &[Vec<String>]) -> String {
    // 正規化: JSON (配列の配列、文字列はそのまま) としてシリアライズ
    let serialized = serde_json::to_vec(commands).unwrap();
    let digest = Sha256::digest(&serialized);
    format!("sha256:{}", hex::encode(digest))
}
```

正規化として JSON を採用する理由: シリアライズ順序が配列/文字列で一意に決まる、
既存の `serde` 依存で済む、後で手動検証がしやすい。

**プロジェクト+モード単位で集約してハッシュ化**する
(コマンド 1 つごとではない)。これにより「承認単位 = プロンプト単位」となり
ロックファイルのエントリ数がコンパクトになる。

### 3. Trust lockfile

**ファイル配置**: リポジトリ直下 `fix.trust.lock` (新規)。

既存 `fix.lock` (存在する場合) との相乗りも検討したが、
ロックファイルの目的 (依存解決結果 vs 信頼情報) が異なるため分離する。

**フォーマット**: TOML。

```toml
version = 1

[[trusted_commands]]
project = "myproj"         # [general] name そのまま
mode = "build"
commands_hash = "sha256:3f2c…"
approved_at = "2026-04-19T10:00:00Z"
# 検証には使わない。人間向け参考表示:
approved_commands = [
  ["sh", "setup.sh"],
]
```

**読み書き API** (新規モジュール `src/trust_lockfile.rs`):

```rust
pub struct TrustLockfile { /* ... */ }
impl TrustLockfile {
    pub fn load(project_root: &Path) -> Result<Self, Errors>;  // 存在しなければ空
    pub fn lookup(&self, project_name: &str, mode: PreliminaryMode) -> Option<&Entry>;
    pub fn upsert(&mut self, entry: Entry);
    pub fn save(&self, project_root: &Path) -> Result<(), Errors>;
}
```

### 4. `run_extra_commands` の書き換え

現行 [src/configuration.rs:642](../../src/configuration.rs#L642) を、
次の流れに置き換える:

```
1. extra_commands を (project_name, mode) でグルーピング
2. Trust lockfile をロード
3. 各グループについて:
     a. 実コマンドのハッシュを計算
     b. lockfile の該当エントリと照合
     c. 一致すれば skip (ただしコマンド内容は常時表示)
     d. 不一致または未承認なら:
         - --allow-preliminary-commands が指定されていれば:
             承認とみなして実行する (**lockfile は更新しない**)
         - 非対話環境なら失敗
         - 対話環境ならプロンプト:
             - `y` なら lockfile を更新して保存し、実行
             - `N` / EOF なら失敗
4. 全グループが承認済みになったら順次実行 (現行の ExtraCommand::run)
```

プロンプトの実装は `dialoguer` クレートまたは自作 stdin 読み取り。
tty 判定は `is_terminal` で行う。

### CLI

- `fix` 本体の clap 定義に `--allow-preliminary-commands` を追加。
  サブコマンド build/run/test で共通のフラグにする。

### LSP / 非対話環境

Fix の LSP モードは tty を持たないため、プロンプトは出せない。
LSP 配下では subcommand が Build/Run/Test になることは通常ないため、
`run_preliminary_commands()`
([src/configuration.rs:101](../../src/configuration.rs#L101)) が false を返す
Diagnostics/Docs でこの仕組みはそもそも発動しない。
発動する経路がもしあれば `--allow-preliminary-commands` 未指定なら失敗で十分。

## テスト

統合テスト (`src/tests/` 配下、新規 `test_preliminary_commands.rs` を想定)。
[CLAUDE.md](../../CLAUDE.md) に沿い、`install_fix()` + `Command::new("fix")`
+ `setup_test_env()` パターンを用いる。

ケース:

- (A) `preliminary_commands` なしのプロジェクト: 何も起きない。
- (B) 初回実行で `--allow-preliminary-commands`: 通るが lockfile は更新されない。
- (C) 初回実行で stdin 閉じ: 失敗する。
- (D) 承認済み lockfile を置いた状態 + ハッシュ一致: 無プロンプトで通る。
- (E) 承認済み lockfile を置いた状態 + コマンド変更: 非対話なら失敗。
- (F) 依存側に `preliminary_commands` がある場合: 依存分も扱われる。
- (G) build 用と test 用が別エントリとして扱われる。

(B)(C)(E) はロックファイル内容と stdout/stderr を検査して確認する。

## マイグレーション

既存プロジェクトへの影響:

- 既に `preliminary_commands` を書いているプロジェクトはデフォルト挙動が変わる。
- 対処ガイド:
  - ローカル開発者は初回に 1 度 `y` を押せば以降は lockfile 経由で通る。
  - CI は開発者が対話環境で生成してコミットした lockfile をそのまま使う運用を推奨。
    変更があったときだけ人間がレビューして再承認する。
- `Document.md` / `Document-ja.md` の `preliminary_commands` 記述に
  この挙動変更を追記する。

## 実装順序

1. `ExtraCommand` 拡張 + ハッシュ関数 (単独で PR 可能)
2. Trust lockfile モジュール (単独で PR 可能)
3. `run_extra_commands` 書き換え + CLI フラグ + プロンプト
4. 統合テスト
5. ドキュメント更新

## 未決事項

- ロックファイル名・配置 (`fix.trust.lock` 案で仮決定)
- プロンプト時に argv だけでなく script 実体の一部をプレビューするか
  (本仕様では対象外だが、UX 強化として候補)
