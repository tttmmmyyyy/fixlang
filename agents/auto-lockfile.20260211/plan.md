# LSP用自動ロックファイルの実装計画

## 概要

現在、言語サーバモード（`fix language-server`）は `fixdeps.test.lock` を参照している。しかし、このファイルが存在しない場合（新しいアプリケーション、テスト未実施のライブラリなど）、言語サーバが動作しない。

この問題を解決するため、言語サーバ専用の自動管理ロックファイル `.fixlang/fixdeps.lsp.lock` を導入する。このファイルは：
- 言語サーバが自動的に生成・更新する（ユーザのコマンド実行不要）
- `.fixlang/` ディレクトリ内に配置（バージョン管理対象外）
- プロジェクトファイルの依存関係（`dependencies` + `test-dependencies`）が変更されたら自動で再生成
- 依存解決に失敗した場合、LSP の診断機能を使ってユーザに通知

## 背景

### 現在の問題点

1. **fixdeps.test.lock が存在しない場合、LSPが動作しない**
   - 新規プロジェクト、アプリケーション開発中、テスト未作成のライブラリなど
   - しかしこれらの状況でも言語サーバは必要

2. **ユーザ操作を強制される**
   - 言語サーバを動かすために `fix deps update --test` の実行が必要
   - 開発フローが中断される

### 解決策

言語サーバ専用の自動管理ロックファイルを導入：
- ファイルパス: `.fixlang/fixdeps.lsp.lock`
- 自動生成・自動更新（`fix deps update` コマンドの実行不要）
- プロジェクトファイル変更検知（ハッシュ値比較）
- 失敗時のLSP診断によるユーザ通知
- `.fixlang/` ディレクトリ内のため、バージョン管理にコミット不要

## 実装フェーズ

### Phase 1: 定数とLockFileTypeの拡張

#### 1.1 定数の追加 (`src/constants.rs`)

新しい定数を追加：
```rust
pub const LOCK_FILE_LSP_PATH: &str = ".fixlang/fixdeps.lsp.lock";
```

#### 1.2 LockFileType の拡張 (`src/configuration.rs`)

`LockFileType` に新しいバリアント `Lsp` を追加：
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockFileType {
    Build,
    Test,
    Lsp,  // 新規追加
}
```

**重要な設計決定:**
- `LockFileType::Lsp` は `LockFileType::Test` と同様に、`dependencies` + `test-dependencies` をマージする
- これにより、テストコードも含めた完全な言語サポートを提供

### Phase 2: ロックファイルパス処理の更新

#### 2.1 `get_lock_file_path` の更新 (`src/dependency_lockfile.rs`)

`LockFileType::Lsp` に対応：
```rust
pub fn get_lock_file_path(mode: LockFileType) -> &'static str {
    match mode {
        LockFileType::Test => LOCK_FILE_TEST_PATH,
        LockFileType::Build => LOCK_FILE_PATH,
        LockFileType::Lsp => LOCK_FILE_LSP_PATH,  // 新規追加
    }
}
```

#### 2.2 依存リスト取得の確認 (`src/project_file.rs`)

既存の `get_dependencies` メソッドが `LockFileType::Lsp` でも正しく動作することを確認：
```rust
pub fn get_dependencies(&self, mode: LockFileType) -> Vec<ProjectFileDependency> {
    match mode {
        LockFileType::Test | LockFileType::Lsp => {  // Lsp を追加
            let mut all_deps = self.dependencies.clone();
            all_deps.extend(self.dependencies_test.clone());
            all_deps
        }
        LockFileType::Build => self.dependencies.clone(),
    }
}
```

同様に `calculate_dependencies_hash` も更新：
```rust
pub fn calculate_dependencies_hash(&self, mode: LockFileType) -> String {
    let deps = match mode {
        LockFileType::Test | LockFileType::Lsp => {  // Lsp を追加
            let mut all_deps = self.dependencies.clone();
            all_deps.extend(self.dependencies_test.clone());
            all_deps
        }
        LockFileType::Build => self.dependencies.clone(),
    };
    // ... ハッシュ計算処理
}
```

### Phase 3: LSPでのロックファイル自動管理

#### 3.1 自動ロックファイル生成・更新関数 (`src/project_file.rs`)

新しいメソッドを追加：
```rust
impl ProjectFile {
    /// ロックファイルを開くか、存在しない/無効な場合は自動で作成・更新する
    /// エラーが発生した場合でも、エラー情報を返すだけでpanicしない
    pub fn open_or_auto_update_lock_file(&self, mode: LockFileType) -> Result<DependecyLockFile, Errors> {
        // まず既存のロックファイルを開いてみる
        match self.open_lock_file(mode) {
            Ok(lock_file) => Ok(lock_file),
            Err(_) => {
                // ロックファイルが存在しないか無効な場合、自動で作成
                // （依存解決とインストールを実行）
                let lock_file = DependecyLockFile::create(self, mode)?;
                
                // ロックファイルを保存
                let content = toml::to_string(&lock_file)
                    .map_err(|e| Errors::from_msg(format!("Failed to serialize lock file: {:?}", e)))?;
                let lock_file_path = get_lock_file_path(mode);
                std::fs::write(lock_file_path, content)
                    .map_err(|e| Errors::from_msg(format!("Failed to write lock file: {:?}", e)))?;
                
                // 依存関係をインストール
                lock_file.install()?;
                
                Ok(lock_file)
            }
        }
    }
}
```

**設計のポイント:**
- エラーが発生してもpanicせず、`Result` で返す
- LSP側でエラーをキャッチして診断として報告

#### 3.2 LSPでの診断実行の更新 (`src/commands/lsp/language_server.rs`)

`run_diagnostics` 関数を更新し、自動ロックファイル管理を使用：

```rust
pub fn run_diagnostics(typecheck_cache: SharedTypeCheckCache) -> Result<DiagnosticsResult, Errors> {
    // Read the project file.
    let proj_file = ProjectFile::read_root_file()?;

    // Determine the source files for which diagnostics are run.
    let files = proj_file.get_files(LockFileType::Lsp);  // Lsp に変更

    // Create the configuration.
    let mut config = Configuration::diagnostics_mode(DiagnosticsConfig { files })?;
    config.type_check_cache = typecheck_cache;

    // Set up the configuration by the project file.
    proj_file.set_config(&mut config, false)?;

    // Set up the configuration by the lock file.
    // 自動でロックファイルを作成・更新
    proj_file
        .open_or_auto_update_lock_file(LockFileType::Lsp)?  // 新しいメソッドを使用
        .set_config(&mut config)?

    // Build the file and get the errors.
    let program = check_program_via_config(&config)?;

    Ok(DiagnosticsResult { program })
}
```

**エラーハンドリングについて:**
- `run_diagnostics` のエラー（依存解決失敗を含む）は、既存の `diagnostics_thread` 実装により自動的にLSP診断として報告される
- 追加の実装は不要

### Phase 4: プロジェクトファイル変更の検知と自動更新

現在の仕組み：
- `open_lock_file` はハッシュ値を比較してロックファイルの有効性を検証
- ハッシュが一致しない場合、エラーを返す

LSP の場合、このエラーをキャッチして自動更新をトリガー：
- Phase 3.1 で実装した `open_or_auto_update_lock_file` が既にこの機能を提供
- `open_lock_file` がエラーを返した場合、自動で `create` を実行

**追加の考慮事項:**
- `.fixlang` ディレクトリが存在しない場合の対応（`LockFileType::Lsp` の場合）
- `open_or_auto_update_lock_file` 内で必要に応じてディレクトリを作成

```rust
pub fn open_or_auto_update_lock_file(&self, mode: LockFileType) -> Result<DependecyLockFile, Errors> {
    match self.open_lock_file(mode) {
        Ok(lock_file) => Ok(lock_file),
        Err(_) => {
            // ロックファイルのパスからディレクトリを取得し、必要なら作成
            let lock_file_path = get_lock_file_path(mode);
            if let Some(parent) = Path::new(lock_file_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Errors::from_msg(format!("Failed to create directory: {:?}", e)))?;
            }
            
            // ロックファイルを作成
            let lock_file = DependecyLockFile::create(self, mode)?;
            
            // ... 以下同じ
        }
    }
}
```

### Phase 5: テストの実装

#### 5.1 テストモジュールの作成

新しいテストモジュールを作成：
- `src/tests/test_lsp.rs` - テストエントリーポイント
- `src/tests/test_lsp/` - ヘルパーモジュールとサンプルプロジェクト
  - `mod.rs` - モジュール定義
  - `lsp_client.rs` - LSPクライアントヘルパー
  - `cases/` - サンプルプロジェクト群

#### 5.2 LSPクライアントヘルパーの実装 (`src/tests/test_lsp/lsp_client.rs`)

LSP プロトコルのテスト用ヘルパー：

```rust
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use serde_json::{json, Value};

pub struct LspClient {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u32,
}

impl LspClient {
    /// fixコマンドを言語サーバモードで起動
    pub fn new(working_dir: &Path) -> Result<Self, String> {
        let mut process = Command::new("fix")
            .arg("language-server")
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn fix language-server: {:?}", e))?;

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(process.stdout.take().unwrap());

        Ok(LspClient {
            process,
            stdin,
            stdout,
            next_id: 1,
        })
    }

    /// LSPリクエストを送信
    pub fn send_request(&mut self, method: &str, params: Value) -> Result<u32, String> {
        let id = self.next_id;
        self.next_id += 1;

        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let content = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize request: {:?}", e))?;
        
        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        
        self.stdin.write_all(header.as_bytes())
            .map_err(|e| format!("Failed to write header: {:?}", e))?;
        self.stdin.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write content: {:?}", e))?;
        self.stdin.flush()
            .map_err(|e| format!("Failed to flush: {:?}", e))?;

        Ok(id)
    }

    /// LSP通知を送信
    pub fn send_notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let content = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize notification: {:?}", e))?;
        
        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        
        self.stdin.write_all(header.as_bytes())
            .map_err(|e| format!("Failed to write header: {:?}", e))?;
        self.stdin.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write content: {:?}", e))?;
        self.stdin.flush()
            .map_err(|e| format!("Failed to flush: {:?}", e))?;

        Ok(())
    }

    /// レスポンスまたは通知を受信
    pub fn receive_message(&mut self) -> Result<Value, String> {
        // Content-Lengthヘッダーを読み取り
        let mut header_line = String::new();
        self.stdout.read_line(&mut header_line)
            .map_err(|e| format!("Failed to read header: {:?}", e))?;

        let content_length: usize = header_line
            .trim()
            .strip_prefix("Content-Length: ")
            .ok_or("Invalid header format")?
            .parse()
            .map_err(|e| format!("Failed to parse content length: {:?}", e))?;

        // 空行を読み飛ばす
        let mut empty_line = String::new();
        self.stdout.read_line(&mut empty_line)
            .map_err(|e| format!("Failed to read empty line: {:?}", e))?;

        // コンテンツを読み取り
        let mut content = vec![0u8; content_length];
        self.stdout.get_mut().read_exact(&mut content)
            .map_err(|e| format!("Failed to read content: {:?}", e))?;

        let message: Value = serde_json::from_slice(&content)
            .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

        Ok(message)
    }

    /// 特定のIDのレスポンスを待つ
    pub fn wait_for_response(&mut self, expected_id: u32) -> Result<Value, String> {
        loop {
            let message = self.receive_message()?;
            
            if let Some(id) = message.get("id") {
                if id.as_u64() == Some(expected_id as u64) {
                    return Ok(message);
                }
            }
            // 他のメッセージ（通知など）はスキップ
        }
    }

    /// 初期化シーケンスを実行
    pub fn initialize(&mut self, root_uri: &str) -> Result<(), String> {
        let params = json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {}
        });

        let id = self.send_request("initialize", params)?;
        let response = self.wait_for_response(id)?;

        if response.get("error").is_some() {
            return Err(format!("Initialize failed: {:?}", response));
        }

        self.send_notification("initialized", json!({}))?;

        Ok(())
    }

    /// シャットダウン
    pub fn shutdown(&mut self) -> Result<(), String> {
        let id = self.send_request("shutdown", json!(null))?;
        let _ = self.wait_for_response(id)?;
        
        self.send_notification("exit", json!(null))?;
        
        // プロセスの終了を待つ
        let _ = self.process.wait();
        
        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // プロセスが実行中なら終了させる
        let _ = self.process.kill();
    }
}
```

#### 5.3 統合テストの実装 (`src/tests/test_lsp.rs`)

**テストケース1: 依存追加による段階的なロックファイル生成・更新**
```rust
#[test]
fn test_lsp_lock_file_lifecycle() {
    // 概要: 依存のないプロジェクトから始めて、依存を段階的に追加し、
    //       ロックファイルが生成・更新されることを確認
    
    install_fix();
    let (_temp_dir, project_dir) = setup_test_env("simple_project");
    
    // テスト実行前にクリーンな状態にする（誤って開いたLSPによる汚染を防ぐ）
    Command::new("fix")
        .arg("clean")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to clean project");
    
    let lsp_lock_file = project_dir.join(".fixlang/fixdeps.lsp.lock");
    
    // LSPを起動し初期化
    let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
    let root_uri = format!("file://{}", project_dir.display());
    client.initialize(&root_uri).expect("Failed to initialize");
    
    // 最初の診断をトリガー（ファイル保存イベント）
    // ...
    
    // 依存がないのでロックファイルは生成されない
    assert!(!lsp_lock_file.exists(), "Lock file should not exist without dependencies");
    
    // 診断に成功することを確認（依存なしでも動作する）
    // ...
    
    // 依存を追加: fix deps add math
    Command::new("fix")
        .arg("deps")
        .arg("add")
        .arg("math")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to add dependency");
    
    // 診断を再実行
    // ...
    
    // ロックファイルが生成されることを確認
    assert!(lsp_lock_file.exists(), "Lock file should be created after adding dependency");
    let content1 = fs::read_to_string(&lsp_lock_file).expect("Failed to read lock file");
    assert!(content1.contains("math"), "Lock file should contain math dependency");
    
    // 診断に成功することを確認
    // ...
    
    // さらに依存を追加: fix deps add character
    Command::new("fix")
        .arg("deps")
        .arg("add")
        .arg("character")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to add another dependency");
    
    // 診断を再実行
    // ...
    
    // ロックファイルが更新されることを確認
    let content2 = fs::read_to_string(&lsp_lock_file).expect("Failed to read lock file");
    assert_ne!(content1, content2, "Lock file should be updated");
    assert!(content2.contains("math"), "Lock file should still contain math dependency");
    assert!(content2.contains("character"), "Lock file should contain character dependency");
    
    // 診断に成功することを確認
    // ...
    
    client.shutdown().expect("Failed to shutdown");
}
```

**テストケース2: 依存解決失敗時のエラー報告**
```rust
#[test]
fn test_lsp_reports_dependency_resolution_failure() {
    // 概要: 依存解決に失敗したとき、LSPが適切にエラーを報告することを確認
    
    install_fix();
    let (_temp_dir, project_dir) = setup_test_env("invalid_dependency_project");
    
    // テスト実行前にクリーンな状態にする
    Command::new("fix")
        .arg("clean")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to clean project");
    
    // LSPを起動し初期化
    let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
    let root_uri = format!("file://{}", project_dir.display());
    client.initialize(&root_uri).expect("Failed to initialize");
    
    // 診断をトリガー
    // ...
    
    // エラー診断が報告されることを確認
    // textDocument/publishDiagnostics 通知を待つ
    // ...
    
    client.shutdown().expect("Failed to shutdown");
}
```

**テストケース3: テスト依存を含む言語サポート**
```rust
#[test]
fn test_lsp_includes_test_dependencies() {
    // 概要: LSPがテスト依存を含むコード補完・診断を提供することを確認
    
    install_fix();
    let (_temp_dir, project_dir) = setup_test_env("project_with_test_deps");
    
    // テスト実行前にクリーンな状態にする
    Command::new("fix")
        .arg("clean")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to clean project");
    
    // LSPを起動し初期化
    let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
    let root_uri = format!("file://{}", project_dir.display());
    client.initialize(&root_uri).expect("Failed to initialize");
    
    // 診断をトリガーしてロックファイルを生成
    // ...
    
    // ロックファイルの内容を確認
    let lsp_lock_file = project_dir.join(".fixlang/fixdeps.lsp.lock");
    let content = fs::read_to_string(&lsp_lock_file).expect("Failed to read lock file");
    
    // 通常依存とテスト依存の両方が含まれることを確認
    assert!(content.contains("normal-dep"), "Should include normal dependency");
    assert!(content.contains("test-dep"), "Should include test dependency");
    
    client.shutdown().expect("Failed to shutdown");
}
```

#### 5.4 サンプルプロジェクトの作成

`src/tests/test_lsp/cases/` に以下のサンプルプロジェクトを作成：

1. **simple_project** - 依存関係なしの単純なプロジェクト（テストケース1で使用）
   ```
   simple_project/
     fixproj.toml
     main.fix
   ```

2. **project_with_test_deps** - テスト依存を持つプロジェクト
   ```
   project_with_test_deps/
     fixproj.toml  (dependencies + test-dependencies あり)
     main.fix
     test.fix
   ```

3. **invalid_dependency_project** - 無効な依存を持つプロジェクト（エラーテスト用）
   ```
   invalid_dependency_project/
     fixproj.toml  (存在しない依存を指定)
     main.fix
   ```

#### 5.5 テストモジュールの登録

`src/tests/mod.rs` に新しいモジュールを追加：
```rust
#[cfg(test)]
mod test_lsp;
```

## 実装の優先順位

1. **Phase 1-2**: 基礎となるデータ構造と定数
2. **Phase 3**: LSPでの自動管理機能
3. **Phase 4**: プロジェクトファイル変更検知（Phase 3に含まれる）
4. **Phase 5**: テストの実装
   - 5.1-5.2: ヘルパーの実装
   - 5.3-5.4: テストケースとサンプルプロジェクト

## 技術的な考慮事項

### エラーハンドリング

1. **依存解決の失敗**
   - ネットワークエラー、バージョン競合など
   - LSP診断として適切にユーザに通知
   - 既存のコードの診断は可能な限り継続

2. **ファイルI/Oエラー**
   - `.fixlang` ディレクトリ作成失敗
   - ロックファイル書き込み失敗
   - 適切なエラーメッセージを提供

3. **部分的な成功**
   - 一部の依存が解決できない場合でも、解決できた依存は使用

### パフォーマンス

1. **キャッシング**
   - プロジェクトファイルのハッシュ値でキャッシュ判定
   - 不要な依存解決を回避

2. **非同期処理**
   - LSPの診断は別スレッドで実行されている（既存の実装）
   - 自動更新も非同期実行で影響最小化

3. **ファイル監視**
   - `fixproj.toml` の変更検知はハッシュ値比較で実現
   - ファイルシステムウォッチャーは不要（LSPの診断実行時にチェック）

### セキュリティ

1. **自動インストール**
   - ユーザ確認なしで依存をインストール
   - 既存の `fix deps install` と同じ挙動
   - レジストリの信頼性に依存（既存の仕組み）

2. **ファイル権限**
   - `.fixlang/` ディレクトリの権限を適切に設定
   - ロックファイルは読み取り専用でなくてOK（自動更新のため）

## 後方互換性

- 既存の `fixdeps.lock` と `fixdeps.test.lock` はそのまま機能
- `fix deps update --test` も引き続き使用可能
- `.fixlang/fixdeps.lsp.lock` は新規追加で既存機能に影響なし

## 将来の拡張可能性

1. **設定オプション**
   - 自動更新の無効化（手動管理モード）
   - プロジェクトファイルでの設定

2. **診断の詳細化**
   - 依存解決の進捗表示
   - より詳細なエラー情報

3. **マルチワークスペース対応**
   - 複数のプロジェクトを同時に開いている場合
   - プロジェクトごとの独立した管理

