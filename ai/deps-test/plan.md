# テスト用依存の実装計画

## 概要

Fixのプロジェクトファイル `fixproj.toml` に `[[dependencies.test]]` セクションを追加し、テスト実行時やLSPでの診断時に使用される依存関係を管理できるようにする。

テストモード（`fix test` コマンド、LSPでの診断時）では：
- `[[dependencies]]` と `[[dependencies.test]]` をマージした依存リストを使用
- ロックファイル名を `fixdeps.test.lock` とする

## 実装フェーズ

### フェーズ1: データ構造の準備

#### 1.1 定数とenumの追加 (`src/constants.rs`, `src/dependency_lockfile.rs`)
```rust
pub const LOCK_FILE_TEST_PATH: &str = "fixdeps.test.lock";
```

```rust
// src/dependency_lockfile.rs または適切な場所
pub enum DependencyMode {
    Build,
    Test,
}
```

#### 1.2 プロジェクトファイル構造の拡張 (`src/project_file.rs`)
- `ProjectFile` 構造体に以下を追加:
  ```rust
  #[serde(default)]
  pub dependencies_test: Vec<ProjectFileDependency>,
  ```
- `calculate_dependencies_hash()` メソッドに `mode: DependencyMode` 引数を追加
  ```rust
  pub fn calculate_dependencies_hash(&self, mode: DependencyMode) -> String {
      let mut deps = match mode {
          DependencyMode::Test => {
              // dependencies と dependencies_test をマージ
              let mut all_deps = self.dependencies.clone();
              all_deps.extend(self.dependencies_test.clone());
              all_deps
          }
          DependencyMode::Build => self.dependencies.clone(),
      };
      // 既存のハッシュ計算処理
      ...
  }
  ```
- `validate()` メソッドを更新
  - `dependencies_test` の検証を追加
  - 通常依存とテスト依存で重複チェック（エラーとする）

### フェーズ2: テストモードの判定と依存マージ

#### 2.1 依存モードの判定 (`src/configuration.rs`)
- 既存の `SubCommand::use_test_files()` メソッドを活用
- 以下のサブコマンドで `DependencyMode::Test` を使用:
  - `SubCommand::Test`
  - `SubCommand::Diagnostics`
  - `SubCommand::Docs`
- その他のサブコマンドでは `DependencyMode::Build` を使用

#### 2.2 依存リストのマージ (`src/project_file.rs`)
- 新しいメソッドを追加:
  ```rust
  impl ProjectFile {
      pub fn get_dependencies(&self, mode: DependencyMode) -> Vec<ProjectFileDependency> {
          match mode {
              DependencyMode::Test => {
                  // dependencies と dependencies_test をマージ
                  // 注: 重複チェックは validate() で既に実施済み
                  let mut all_deps = self.dependencies.clone();
                  all_deps.extend(self.dependencies_test.clone());
                  all_deps
              }
              DependencyMode::Build => self.dependencies.clone(),
          }
      }
  }
  ```

### フェーズ3: ロックファイル処理の更新

#### 3.1 ロックファイルパスの取得 (`src/dependency_lockfile.rs`)
```rust
pub fn get_lock_file_path(mode: DependencyMode) -> &'static str {
    match mode {
        DependencyMode::Test => LOCK_FILE_TEST_PATH,
        DependencyMode::Build => LOCK_FILE_PATH,
    }
}
```

#### 3.2 ロックファイル作成の更新
- `DependecyLockFile::create()` メソッドに `mode: DependencyMode` パラメータを追加
  ```rust
  pub fn create(proj_file: &ProjectFile, mode: DependencyMode) -> Result<DependecyLockFile, Errors>
  ```
- `create()` 内部で `proj_file.get_dependencies(mode)` を呼び出して依存リストを取得
- `proj_file.calculate_dependencies_hash(mode)` を使用してハッシュを計算

#### 3.3 ロックファイル読み書きの更新
- ロックファイルの読み込み時に `get_lock_file_path(mode)` を使用して適切なパスを取得
- ロックファイルの書き込み時も同様に適切なパスを使用
- `ProjectFile::open_lock_file(mode)` で読み込み処理を行うため、`DependecyLockFile` 側の追加ヘルパー関数は不要

### フェーズ4: メインコマンドの統合

#### 4.1 main.rs の更新 (`src/main.rs`)
- ロックファイルの読み込み箇所を更新（テストモードに応じたパス）
- `fix deps update` コマンドの更新
  - デフォルト動作: `dependencies_test` が存在する場合、両方のロックファイルを生成
    - `fixdeps.lock` (通常版)
    - `fixdeps.test.lock` (テスト版)
  - `dependencies_test` が空または存在しない場合は通常版のみ生成
  - `--skip-test` フラグの追加: テスト版の生成をスキップし通常版のみ生成/更新
    ```rust
    if args.get_flag("skip-test") {
        // 通常版のみ生成
    } else {
        // 通常版を生成
        if !proj_file.dependencies_test.is_empty() {
            // テスト版も生成
        }
    }
    ```
- `fix deps install` コマンドの更新
  - デフォルト: 通常版 (`fixdeps.lock`) のみインストール
  - `--with-test` フラグ: テスト版も含めてインストール
    ```rust
    if args.get_flag("with-test") {
        // 通常版とテスト版の両方をインストール
    } else {
        // 通常版のみインストール
    }
    ```
- `fix deps add` コマンドの更新
  - デフォルト: `[[dependencies]]` セクションに追加
  - `--test` フラグ: `[[dependencies.test]]` セクションに追加
    ```rust
    if args.get_flag("test") {
        // dependencies_test に追加
    } else {
        // dependencies に追加
    }
    ```
  - 追加後、適切なロックファイル（通常版またはテスト版）を更新

#### 4.2 コンパイル処理の更新
- `SubCommand` に `dependency_mode()` メソッドを追加して依存モードを導出
  ```rust
  impl SubCommand {
      pub fn dependency_mode(&self) -> DependencyMode {
          match self {
              SubCommand::Test | SubCommand::Diagnostics(_) | SubCommand::Docs(_) => {
                  DependencyMode::Test
              }
              _ => DependencyMode::Build,
          }
      }
  }
  ```
- 依存関係の解決時に `subcommand.dependency_mode()` を使用して適切な依存リストを取得
- `ProjectFile::install_dependencies()` に `mode: DependencyMode` パラメータを追加
  ```rust
  pub fn install_dependencies(&self, config: &mut Configuration, mode: DependencyMode) -> Result<(), Errors>
  ```
- `ProjectFile::open_or_create_lock_file()` に `mode: DependencyMode` パラメータを追加
  ```rust
  pub fn open_or_create_lock_file(&self, mode: DependencyMode) -> Result<DependecyLockFile, Errors>
  ```
- `ProjectFile::open_lock_file()` に `mode: DependencyMode` パラメータを追加
  ```rust
  pub fn open_lock_file(&self, mode: DependencyMode) -> Result<DependecyLockFile, Errors>
  ```
- これらのメソッド内で `get_lock_file_path(mode)` を使用して適切なロックファイルパスを取得
- `fix build/run/test` コマンドでの自動ロックファイル生成・インストール処理
  - `fix build`, `fix run`: `DependencyMode::Build`
  - `fix test`: `DependencyMode::Test`
  - ロックファイルが存在しない場合、自動的に生成（`open_or_create_lock_file()`）
  - 依存がインストールされていない場合、自動的にインストール
- 依存プロジェクトのソースファイルのコンパイル・診断への反映
  - `install_dependencies()` が `lock_file.set_config(config)` を呼び出し
  - `DependecyLockFile::set_config()` が各依存プロジェクトの `ProjectFile::set_config(config, true)` を呼び出し
  - テストモードでは `fixdeps.test.lock` が使用されるため、通常依存+テスト依存のソースファイルが自動的に含まれる
  - 依存プロジェクト自身のテストファイルは `is_dependency = true` により除外される
  - **重要**: ロックファイルを適切に切り替えるだけで、依存のソースファイルも自動的に正しく反映される
    - `DependencyMode::Build` → `fixdeps.lock` → 通常依存のみ
    - `DependencyMode::Test` → `fixdeps.test.lock` → 通常依存+テスト依存
    - 既存の `set_config()` の仕組みをそのまま利用できるため、この部分の追加修正は不要

### フェーズ5: LSPの対応

#### 5.1 LSP診断時の対応 (`src/lsp/`)
- 診断時はテストモードとして扱う
- `fixdeps.test.lock` を使用

### フェーズ6: テストとドキュメント

#### 6.1 ユニットテストの追加
- `test_get_dependencies_build_mode()`: ビルドモードでの依存取得テスト
  - `ProjectFile` に通常依存とテスト用依存の両方を設定
  - `get_dependencies(DependencyMode::Build)` を呼び出し
  - 通常依存のみが返されることを確認
  - テスト用依存が含まれていないことを確認

- `test_get_dependencies_test_mode()`: テストモードでの依存取得テスト
  - `ProjectFile` に通常依存とテスト用依存の両方を設定
  - `get_dependencies(DependencyMode::Test)` を呼び出し
  - 通常依存とテスト用依存の両方が返されることを確認
  - 返されたリストの長さが正しいことを確認

- `test_validate_duplicate_dependency()`: 重複依存の検証テスト
  - 同じ名前の依存を `dependencies` と `dependencies_test` に設定
  - `ProjectFile::validate()` を呼び出し
  - エラーが返されることを確認
  - エラーメッセージに "Duplicate dependency" が含まれることを確認

- `test_get_lock_file_path()`: ロックファイルパス取得テスト
  - `get_lock_file_path(DependencyMode::Build)` が `"fixdeps.lock"` を返すことを確認
  - `get_lock_file_path(DependencyMode::Test)` が `"fixdeps.test.lock"` を返すことを確認

- `test_calculate_dependencies_hash()`: ハッシュ計算のテスト
  - `ProjectFile` に通常依存とテスト用依存を設定
  - `calculate_dependencies_hash(DependencyMode::Build)` と `calculate_dependencies_hash(DependencyMode::Test)` を呼び出し
  - 両者が異なるハッシュ値を返すことを確認
  - ビルドモードのハッシュが通常依存のみから計算されていることを確認
  - テストモードのハッシュがマージされた依存から計算されていることを確認

#### 6.2 統合テストの追加
- テスト用プロジェクトの配置: `src/tests/test_dependencies/`
  - `cases/main_project/`: メインのテストプロジェクト
    - `fixproj.toml`: 通常依存とテスト用依存 (`[[dependencies.test]]`) を含む
    - `main.fix`: メイン実装（通常依存のみを使用）
    - `test.fix`: テストコード（テスト用依存を使用）
  - `cases/normal_dep/`: 通常の依存プロジェクト
    - `fixproj.toml`: 基本的なプロジェクト設定
    - `lib.fix`: ライブラリコード
  - `cases/test_dep/`: テスト用の依存プロジェクト
    - `fixproj.toml`: 基本的なプロジェクト設定
    - `lib.fix`: テストヘルパー関数など

- テスト関数の追加: `src/tests/test_dependencies.rs`
  - `test_dependencies_build_mode()`: 通常ビルドモードのテスト
    1. `install_fix()` でfixコマンドをインストール
    2. テストプロジェクト内の `*.lock` ファイルを削除
    3. `fix build` を実行
    4. `fixdeps.lock` が作成されていることを確認
    5. `fixdeps.test.lock` が作成されて**いない**ことを確認
    6. `fixdeps.lock` の内容を読み込み、通常依存のみが含まれていることを確認（テスト用依存が含まれていないこと）
    7. ビルドが成功することを確認
  
  - `test_dependencies_test_mode()`: テストモードのテスト
    1. `install_fix()` でfixコマンドをインストール
    2. テストプロジェクト内の `*.lock` ファイルを削除
    3. `fix test` を実行
    4. `fixdeps.lock` と `fixdeps.test.lock` の両方が作成されていることを確認
    5. `fixdeps.test.lock` の内容を読み込み、通常依存とテスト用依存の両方が含まれていることを確認
    6. テストが成功することを確認（テスト用依存が正しくリンクされたことの証明）
  
  - `test_dependencies_lock_file_separation()`: ロックファイル分離のテスト
    1. `install_fix()` でfixコマンドをインストール
    2. テストプロジェクト内の `*.lock` ファイルを削除
    3. `fix deps update` を実行
    4. 両方のロックファイルが生成されることを確認
    5. 各ロックファイルの依存リストを比較し、異なることを確認
  
  - `test_dependencies_duplicate_error()`: 重複依存のエラーテスト
    1. 同じ依存を `[[dependencies]]` と `[[dependencies.test]]` に記述したプロジェクトを用意
    2. `fix build` または `fix test` を実行
    3. エラーが発生することを確認
    4. エラーメッセージに "Duplicate dependency" が含まれることを確認

#### 6.3 ドキュメント更新
- `Document.md` に `[[dependencies.test]]` の説明を追加
- `Document-ja.md` にも同様の説明を追加

## 実装の詳細設計

### 依存マージの仕様

**重複がある場合の処理:**
- 同じ名前の依存が `[[dependencies]]` と `[[dependencies.test]]` の両方にある場合、エラーとする
- 既存の実装では `[[dependencies]]` 内の重複もエラーとしており、一貫性がある
- ユーザーに明示的な修正を求めることで、意図しない依存の重複を防ぐ

### ロックファイルのハッシュ計算

テストモード時のハッシュは:
- `dependencies` + `dependencies_test` の内容で計算
- 通常モードとは異なるハッシュ値になる

### fix deps コマンドの動作

```bash
# 通常版とテスト版の両方を生成/更新（dependencies_testが存在する場合）
fix deps update

# 通常版のみ生成/更新（テスト版をスキップ）
fix deps update --skip-test

# 通常版のみインストール（デフォルト）
fix deps install

# 通常版とテスト版の両方をインストール
fix deps install --with-test

# 通常の依存を追加
fix deps add <project-name>

# テスト用の依存を追加
fix deps add --test <project-name>
```

**動作詳細:**
- `fix deps update` は `dependencies_test` セクションが存在し、かつ空でない場合に自動的に両方のロックファイルを生成
- `fix deps install` はデフォルトで通常版のみインストール（ビルド時に不要なテスト依存を避けるため）
- `--with-test` フラグでテスト版も含めてインストール（開発環境やCIで使用）
- ログ出力で何が生成されているか明示（例: "Generating test lock file..."）
- 両方のロックファイルはバージョン管理に含めることを推奨

## 実装順序

1. **Step 1**: 定数とデータ構造の追加（フェーズ1）
2. **Step 2**: テストモード判定と依存マージロジック（フェーズ2）
3. **Step 3**: ロックファイル処理の更新（フェーズ3）
4. **Step 4**: メインコマンドの統合（フェーズ4）
5. **Step 5**: LSPの対応（フェーズ5）
6. **Step 6**: テストとドキュメント（フェーズ6）

## 注意点

- 既存のプロジェクトとの互換性を保つ（`dependencies_test` は optional）
- ロックファイルが2つになることによるユーザー体験への影響を最小化
- CI/CDでの動作を考慮（両方のロックファイルをバージョン管理に含める）
