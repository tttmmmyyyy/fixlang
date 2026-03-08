# `fix check` コマンド実装計画

## 概要

`fix check` コマンドは、Fixプロジェクトが正常にコンパイルできるかを検証する。バイナリの生成は行わず、パース・名前解決・型チェックまでを実行し、エラーがなければ成功とする。テストコードも含めた全エンティティの型チェックを行う。

## 設計方針

### SubCommand::Diagnostics の流用

新しい `SubCommand` バリアントは追加せず、既存の `SubCommand::Diagnostics` を流用する。`check` コマンドでは、プロジェクトの全ソースファイルを `DiagnosticsConfig.files` に設定した `Diagnostics` を使用する。

`elaborate` 関数内の Diagnostics パスは以下の処理を行う：
1. パースエラーを遅延させる（ルートプロジェクトのパースエラーでも処理を続行）
2. `diag_config.files` に含まれるモジュールの全エンティティを型チェック
3. 型チェックエラーを `deferred_errors` に蓄積

`check` コマンドでは `elaborate_via_config` 呼出し後、返却された `Program` の `deferred_errors` にエラーがあればそれを返してコマンドを失敗させる。

### build_mode について

テストコードも含めて型チェックするため、`BuildConfigType::Test` を使う。`Diagnostics` の `build_mode()` は元々 `Test` を返すため、そのまま利用できる。

## 変更対象ファイル

### 1. `src/commands/check.rs` — checkコマンドの実装（新規ファイル）

`docs.rs` の構造を参考に実装する。全ソースファイルを `DiagnosticsConfig.files` に設定し、`elaborate_via_config` で型チェックを行う。`deferred_errors` にエラーがあれば失敗とする。

```rust
use crate::configuration::{BuildConfigType, Configuration, DiagnosticsConfig};
use crate::dependency::lockfile::LockFileType;
use crate::elaboration::elaborate_via_config;
use crate::error::Errors;
use crate::metafiles::project_file::ProjectFile;
use crate::misc::info_msg;

pub fn check(mut config: Configuration) -> Result<(), Errors> {
    info_msg("Checking...");

    // Set up the configuration by the project file.
    let proj_file = ProjectFile::read_root_file()?;
    proj_file.set_config(&mut config, false)?;

    // Set up the configuration by the lock file.
    // Use Test mode to include test dependencies.
    proj_file
        .open_lock_file(LockFileType::from_build_config_type(BuildConfigType::Test))?
        .set_config(&mut config)?;

    // Set all source files as diagnostics target files.
    match &mut config.subcommand {
        crate::SubCommand::Diagnostics(diag_config) => {
            diag_config.files = config.source_files.clone();
        }
        _ => unreachable!(),
    }

    // Elaborate (parse, resolve, type-check) all entities.
    let program = elaborate_via_config(&config)?;

    // Check for deferred errors (parse errors and type errors accumulated during diagnostics).
    if program.deferred_errors.has_error() {
        return Err(program.deferred_errors);
    }

    info_msg("No errors found.");
    Ok(())
}
```

### 2. `src/commands/mod.rs` — checkモジュールの登録

- `pub mod check;` を追加。

### 3. `src/configuration.rs` — check_mode コンストラクタの追加

`diagnostics_mode()` と同様のパターンで `check_mode()` を追加。内部的には全ファイルが空の `DiagnosticsConfig` で初期化し、`check` コマンド内でファイルを設定する。

```rust
pub fn check_mode() -> Result<Configuration, Errors> {
    let mut config = Self::new(SubCommand::Diagnostics(DiagnosticsConfig::default()))?;
    config.num_worker_thread = num_cpus::get();
    Ok(config)
}
```

### 4. `src/main.rs` — CLIサブコマンドの追加

- `check` サブコマンドの定義を追加。
- `app` に `.subcommand(check_subc)` を追加。
- マッチ文に `Some(("check", _args))` ブランチを追加。

CLIの定義：
```rust
let check_subc = App::new("check")
    .about("Checks whether a Fix project compiles without errors. \
            Type-checks all entities including test code.");
```

マッチ文：
```rust
Some(("check", _args)) => {
    let config = panic_if_err(Configuration::check_mode());
    panic_if_err(commands::check::check(config));
}
```

## テスト方針

copilot-instructions.md の指示に従い、コマンドの動作変更なのでユニットテストは書かない。動作確認は以下の方法で行う：

- 正常なプロジェクトで `fix check` を実行し、エラーなく終了することを確認。
- 型エラーのあるプロジェクトで `fix check` を実行し、エラーが報告されることを確認。

## 実装順序

1. `src/commands/check.rs` を作成
2. `src/commands/mod.rs` に登録
3. `src/configuration.rs` に `check_mode()` を追加
4. `src/main.rs` にCLIサブコマンドを追加
5. ビルド確認（`cargo build`）
6. 動作確認
