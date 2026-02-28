# Git コミット指定による依存関係の固定

## 概要

`fixproj.toml` の `[[dependencies]]` / `[[test_dependencies]]` で、git依存に対してタグ・コミットハッシュを明示的に指定できるようにする。

現状、git依存はリポジトリ内のバージョンタグ（`v0.1.0` 等）を元にSemVerで解決されるが、以下のようなユースケースに対応できない：

- **特定のコミットに固定したい**（まだタグが付いていないバグ修正等）
- **特定のタグを直接指定したい**（SemVer解決をバイパスしたい）

## 現状の仕様

### 現在の `[[dependencies]]` の構造

```toml
[[dependencies]]
name = "hash"
version = "0.1.0"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git" }
```

- `name`: プロジェクト名（必須）
- `path` or `git`: ソースの指定（いずれか一つが必須）
  - `path`: ローカルディレクトリパス
  - `git`: `{ url = "..." }` の形式でGitリポジトリURL
- `version`: SemVerバージョン要件（省略時は `"*"`）

### 現在の依存解決フロー（git依存）

1. リポジトリをクローン
2. タグを走査し、`v{major}.{minor}.{patch}` の形式のタグからバージョン一覧を取得
3. タグがない場合、HEADの `fixproj.toml` からバージョンを取得
4. SemVerの `version` フィールドでバージョン要件を照合
5. 解決結果（コミットハッシュ含む）をロックファイルに記録

### 現在のロックファイル構造

```toml
[[dependencies]]
name = "hash"
version = "0.1.2"
path = ".fix/hash_0.1.2"
git = { repo = "https://github.com/tttmmmyyyy/fixlang-hash.git", rev = "abc1234..." }
```

## 提案する仕様

### 記法

`git` フィールドに `rev` または `tag` を追加で指定できるようにする。

```toml
# コミットハッシュで指定
[[dependencies]]
name = "hash"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git", rev = "a1b2c3d4e5f6" }

# タグで指定（SemVer解決をバイパス）
[[dependencies]]
name = "hash"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git", tag = "v0.2.0-rc1" }

# rev/tag + version：ref先のバージョンが要件を満たすかチェック
[[dependencies]]
name = "hash"
version = "0.1"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git", rev = "a1b2c3d4e5f6" }
```

### フィールドの相互排他ルール

`git` テーブル内で以下のフィールドは組み合わせ不可（最大1つ）：

| フィールド | 型       | 説明                                                     |
|-----------|----------|----------------------------------------------------------|
| `rev`     | `String` | 完全または先頭一致のコミットハッシュ（最低7文字を推奨）       |
| `tag`     | `String` | Gitタグ名                                                 |

いずれも指定しない場合は現在と同じ動作（タグベースのSemVer解決）。

### `version` フィールドとの関係

| 指定パターン | `version` | `rev`/`tag` | 動作 |
|-------------|-----------|-------------|------|
| 従来通り     | あり      | なし        | SemVerでバージョン解決（現状と同じ） |
| ref固定のみ  | なし      | あり        | 指定されたref先のコミットを使用。バージョンは `fixproj.toml` から取得するが、SemVer照合はしない |
| 両方指定     | あり      | あり        | ref先のコミットを使用し、そのバージョンが `version` の要件を満たすかチェックする |
| 両方なし     | なし      | なし        | 従来通り（`version = "*"` と同等） |

**`version` と `rev`/`tag` の併用を許可する理由**:

ルートプロジェクトが A と B に依存し、A を `rev`/`tag` で固定しているとする。B も A に依存しており、B の `fixproj.toml` には A に対する SemVer バージョン要求が記述されている。この場合、ref固定された A のバージョンが B の要求を満たすかのチェックは必須であり、推移的依存経由のバージョン要求は常に発生しうる。

したがって、ルートプロジェクトでも ref 固定と同時にバージョン要求を記述できるのが自然。`version` は「このバージョン範囲を満たさなければエラー」という制約として機能する。

### `path` との関係

`path` と `git` は引き続き排他。変更なし。

## 実装の詳細

### Phase 1: データ構造の変更

#### `ProjectFileDependencyGit` の拡張 (`src/project_file.rs`)

```rust
#[derive(Deserialize, Serialize, Default, Clone, Hash)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileDependencyGit {
    pub url: String,
    pub rev: Option<String>,     // コミットハッシュ
    pub tag: Option<String>,     // タグ名
}
```

#### バリデーションの追加 (`src/project_file.rs`)

`validate_dependency_entry` に以下を追加：

1. `git.rev` と `git.tag` が同時に指定されていないことを検証

`version` と `rev`/`tag` の併用は許可する（`version` はバージョン制約として機能）。

```rust
fn validate_dependency_entry(dep: &ProjectFileDependency, span: Span) -> Result<(), Errors> {
    // ... 既存のバリデーション ...

    // git ref 指定の排他チェック
    if let Some(git) = &dep.git {
        if git.rev.is_some() && git.tag.is_some() {
            return Err(Errors::from_msg_srcs(
                "Only one of `rev` or `tag` can be specified in a git dependency."
                    .to_string(),
                &[&Some(span.clone())],
            ));
        }
        // version と rev/tag の併用は許可（version はバージョン制約として機能する）
    }

    Ok(())
}
```

### Phase 2: 依存解決の変更 (`src/dependency_lockfile.rs`)

#### 方針

`resolve_dependency` や `PackageRetriever`/`VersionRetriever` のシグネチャは変更しない。

既存の依存解決は以下の流れで動作する:

1. `DependecyLockFile::create` が `ProjectsInfo`（プロジェクト情報のリスト）を作り、2つのクロージャ `package_retriever` と `version_retriever` を生成する
2. `resolve_dependency` に上記クロージャを渡して依存解決を行う
3. `version_retriever(pkg_name)` は内部で `ProjectInfo::retrieve_versions()` を呼び、当該パッケージの利用可能バージョン一覧を返す
4. `package_retriever(pkg_name, version)` は内部で `ProjectInfo::get_project_file(version)` を呼び、そのバージョンの `Package`（名前・バージョン・依存リスト）を返す。このとき推移的依存を `ProjectsInfo` に追加登録する

ここで `ProjectInfo` は `versions: Option<Vec<VersionInfo>>` と `proj_files: Vec<ProjectFile>` をキャッシュとして持つ。`retrieve_versions()` は `self.versions.is_some()` なら何もせず即座に返り、`get_project_file(version)` は `self.proj_files` にキャッシュがあればそれを返す。

つまり、**resolver が動く前に `ProjectInfo` の `versions` と `proj_files` を事前に埋めておけば**、resolver はタグスキャンもチェックアウトもせずに、そのデータをそのまま使う。rev/tag で固定した依存に対しては:

- `versions` を pinned version 1件だけのリスト `[VersionInfo { version, rev: oid }]` で埋める
- `proj_files` にそのコミットの `fixproj.toml` を埋める

こうすると、既存の resolver・retriever のコードを一切変更せずに、pinned dep が自然に解決される。

#### なぜ既存コードがそのまま動くか

以下、既存の各コンポーネントが pinned dep に対してどう振る舞うかを説明する。

**1. `version_retriever` → pinned version 1つだけ返る**

`create_version_retriever` のクロージャは `ProjectInfo::retrieve_versions()` を呼んでから `versions` を返す。pinned dep では `versions` が既にセット済みなので `retrieve_versions()` は早期リターンし、結果として pinned version 1つだけが返る。通常の git 依存ではタグスキャンで複数バージョンが返るが、pinned dep は1つだけ。

**2. resolver が pinned dep を優先的に解決する**

`try_use_package` 内で依存の解決順を「候補バージョン数が少ない順」にソートする（`dep_range.sort_by_key(|(_, count)| *count)`）。pinned dep は候補が1つしかないため、自然に他の依存より先に解決され、`fixed` リストに早い段階で入る。

**3. `package_retriever` → キャッシュ済み `proj_file` を使用**

`create_package_retriever` のクロージャは `ProjectInfo::get_project_file(version)` を呼ぶ。pinned dep では `proj_files` に該当バージョンのプロジェクトファイルが既にキャッシュされているため、リポジトリへのチェックアウトなしで即座に返る。返された `ProjectFile` は `project_file_to_package` で `Package { name, version, deps }` に変換される。

**4. 推移的依存が pinned dep を要求した場合**

pinned dep は resolver の `fixed` リストに既に入っている。`try_resolve_dependency` は依存名が `fixed` 内にある場合、VersionReq チェック（`dependency.requirement.matches(&package.version)`）のみを行う。要件を満たせば OK、満たさなければバックトラック。これは既存コードそのまま。

**5. 推移的依存の rev/tag は自然に無視される**

`create_package_retriever` は推移的依存を `ProjectsInfo` に追加登録する際、`proj_file.get_dependency_source(&dep.name)` で `ProjectSource` を取得する。この関数は `ProjectSource::Git(git.url.clone(), None)` を返し、rev/tag 情報を含まない。また、`project_file_dep_to_dependency` は `Dependency { name, requirement }` だけを返し、git 情報（rev/tag）は含まれない。したがって推移的依存は通常の SemVer 解決で処理される。

#### 新規追加: `resolve_git_ref` ヘルパー

rev/tag から OID を解決する関数:

```rust
fn resolve_git_ref(
    repo: &Repository,
    git: &ProjectFileDependencyGit,
) -> Result<git2::Oid, Errors> {
    if let Some(rev) = &git.rev {
        let obj = repo.revparse_single(rev)
            .map_err(|e| Errors::from_msg(format!("Failed to find rev \"{}\": {}", rev, e)))?;
        Ok(obj.id())
    } else if let Some(tag) = &git.tag {
        let refname = format!("refs/tags/{}", tag);
        let reference = repo.find_reference(&refname)
            .map_err(|e| Errors::from_msg(format!("Failed to find tag \"{}\": {}", tag, e)))?;
        let obj = reference.peel_to_commit()
            .map_err(|e| Errors::from_msg(format!("Failed to peel tag \"{}\" to commit: {}", tag, e)))?;
        Ok(obj.id())
    } else {
        unreachable!("No git ref specified")
    }
}
```

#### 新規追加: `ProjectInfo::resolve_pinned_ref` メソッド

pinned ref を解決し、`versions` と `proj_files` を pre-populate する。
既存の `prepre_git_repository`（内部で `clone_git_repo`）と `get_project_file` のチェックアウトパターンを再利用:

```rust
impl ProjectInfo {
    /// Resolve a pinned git ref (rev or tag) and pre-populate versions and proj_files.
    /// Reuses `prepre_git_repository` for cloning and the checkout pattern from `get_project_file`.
    fn resolve_pinned_ref(
        &mut self,
        git_ref: &ProjectFileDependencyGit,
    ) -> Result<(Version, git2::Oid), Errors> {
        // Clone the repository (reuses existing prepre_git_repository).
        self.source.prepre_git_repository()?;
        let repo = self.source.get_git_repository();

        // Resolve rev/tag to OID.
        let oid = resolve_git_ref(repo, git_ref)?;

        // Checkout and read fixproj.toml (same pattern as get_project_file).
        let commit = repo.find_commit(oid)
            .map_err(|e| Errors::from_msg_err("Failed to find commit", e))?;
        let mut checkout_opts = CheckoutBuilder::default();
        checkout_opts.force();
        repo.checkout_tree(&commit.into_object(), Some(&mut checkout_opts))
            .map_err(|e| Errors::from_msg_err("Failed to checkout commit", e))?;
        let proj_file = ProjectFile::read_file(
            &repo.workdir().unwrap().join(PROJECT_FILE_PATH)
        )?;

        let version = proj_file.general.version();

        // Pre-populate versions (single entry) and proj_files cache.
        self.versions = Some(vec![VersionInfo {
            version: version.clone(),
            rev: oid,
            tagged: true,
        }]);
        self.proj_files.push(proj_file);

        Ok((version, oid))
    }
}
```

**再利用しているコード:**
- `prepre_git_repository()` → 内部で `clone_git_repo()` を呼ぶ既存メソッド
- `CheckoutBuilder` + `checkout_tree` → `get_project_file` と同じパターン
- `ProjectFile::read_file` → 既存
- `VersionInfo` → 既存構造体

**新規コードは `resolve_git_ref` のみ。**

#### `DependecyLockFile::create` の変更

`resolve_dependency` 呼び出し前に、pinned dep を `ProjectsInfo` に登録する:

```rust
pub fn create(
    proj_file: &ProjectFile,
    mode: BuildConfigType,
) -> Result<DependecyLockFile, Errors> {
    let prjs_info = ProjectsInfo {
        projects: Arc::new(Mutex::new(vec![ProjectInfo::from_project_file(proj_file)])),
    };

    // --- pinned dep を ProjectsInfo に事前登録 ---
    let all_deps = proj_file.get_dependencies(mode);
    for dep in all_deps.iter().filter(|d| d.git.as_ref().map_or(false, |g| g.has_ref())) {
        let git = dep.git.as_ref().unwrap();

        // ProjectInfo を作成し、pinned ref を解決
        let mut proj_info = ProjectInfo {
            name: dep.name.clone(),
            source: ProjectSource::Git(git.url.clone(), None),
            versions: None,
            proj_files: Vec::new(),
        };
        let (version, _oid) = proj_info.resolve_pinned_ref(git)?;

        // name 一致を検証
        if proj_info.proj_files[0].general.name != dep.name {
            return Err(Errors::from_msg(format!(
                "Dependency \"{}\" pinned to {} resolves to project \"{}\", which has a different name.",
                dep.name, git.ref_description(), proj_info.proj_files[0].general.name
            )));
        }

        // version 制約を検証（dep.version が指定されている場合）
        let requirement = dep.version();
        if !requirement.matches(&version) {
            return Err(Errors::from_msg(format!(
                "Dependency \"{}\" is pinned to {} (version {}), but the version requirement \"{}\" is not satisfied.",
                dep.name, git.ref_description(), version, requirement
            )));
        }

        // ProjectsInfo に登録
        prjs_info.projects.lock().unwrap().push(proj_info);
    }

    // --- 従来通りの resolve_dependency（変更なし） ---
    let packages_retriever = create_package_retriever(prjs_info.clone());
    let versions_retriever = create_version_retriever(prjs_info.clone());
    let res = dependency_resolver::resolve_dependency(
        proj_file,
        packages_retriever.as_ref(),
        versions_retriever.as_ref(),
        mode,
    )?;
    // ... 以下既存コード ...
}
```

#### 動作の流れ

以下、具体例として Root が `math` を `tag = "v1.0.0"` で固定し、`local-a`（path 依存）が `math` を `version = "1.0"` で通常依存している場合の流れを示す。

```
DependecyLockFile::create(root_proj_file, mode):

  ── Step 1: ProjectsInfo の初期化 ──
  ProjectsInfo に root の ProjectInfo を登録:
    ProjectInfo {
      name: "my-project",
      source: Local("./"),
      versions: Some([VersionInfo { version: 0.1.0, rev: 0000, tagged: false }]),
      proj_files: [root_proj_file],
    }

  ── Step 2: pinned dep の事前登録 ──
  root の依存を走査 → math に tag = "v1.0.0" がある → pinned dep として処理:

    2a. ProjectInfo を作成:
        ProjectInfo { name: "math", source: Git("https://...math.git", None), versions: None, proj_files: [] }

    2b. resolve_pinned_ref を呼ぶ:
        - prepre_git_repository() で math リポジトリをクローン（既存コード再利用）
        - resolve_git_ref() で tag "v1.0.0" → OID 6b1c381 を取得
        - OID のコミットをチェックアウト（get_project_file と同じパターン）
        - fixproj.toml を読む → version = "1.0.0"
        - versions = [VersionInfo { version: 1.0.0, rev: 6b1c381, tagged: true }] をセット
        - proj_files = [math の ProjectFile] をセット

    2c. バリデーション:
        - fixproj.toml の name が "math" であることを確認（一致しなければエラー）
        - root が version 制約を指定している場合、pinned version がそれを満たすか確認

    2d. ProjectsInfo に push:
        → ProjectsInfo = [root, math(versions事前セット済み)]

  ── Step 3: retriever 作成（既存コードそのまま） ──
  package_retriever = create_package_retriever(prjs_info)
  version_retriever = create_version_retriever(prjs_info)

  ── Step 4: resolve_dependency（既存コードそのまま、シグネチャ変更なし） ──
  resolve_dependency(root_proj_file, package_retriever, version_retriever, mode):

    4a. try_use_package("my-project", "0.1.0", fixed=[]):
        package_retriever("my-project", "0.1.0") → Package { deps: [local-a, math(version="*")] }
        fixed = [my-project@0.1.0]

        各 dep の候補数を取得してソート:
          version_retriever("local-a") → [0.1.0]                ... 候補1つ
          version_retriever("math")    → [1.0.0]  ← 事前セット済み！  ... 候補1つ
        ソート結果: [local-a(1), math(1)]  （候補が少ない順）

    4b. try_resolve_dependency("local-a@*", fixed=[my-project]):
        local-a は fixed にない → version_retriever で [0.1.0] を取得
        try_use_package("local-a", "0.1.0"):
          package_retriever("local-a", "0.1.0"):
            → local-a の fixproj.toml を読む
            → deps: [math(version="1.0")]
            → math はすでに ProjectsInfo にある → source 一致を確認 → OK（再登録しない）
          fixed = [my-project, local-a@0.1.0]

          try_resolve_dependency("math@^1.0", fixed=[my-project, local-a]):
            math は fixed にない → version_retriever("math") → [1.0.0]  ← 事前セット済み！
            ^1.0 にマッチする候補: [1.0.0]
            try_use_package("math", "1.0.0"):
              package_retriever("math", "1.0.0"):
                → proj_files キャッシュに 1.0.0 がある → 即座に返す（チェックアウト不要！）
                → Package { name: "math", version: 1.0.0, deps: [] }
              fixed = [my-project, local-a@0.1.0, math@1.0.0]
              → OK

    4c. try_resolve_dependency("math@*", fixed=[my-project, local-a, math]):
        math は既に fixed にある → fixed 内の math@1.0.0 が requirement "*" を満たすか → Yes → OK

    結果: [my-project@0.1.0, local-a@0.1.0, math@1.0.0]

  ── Step 5: ロックファイル構築（既存コードそのまま） ──
  resolved packages を走査:
    math@1.0.0 の VersionInfo を ProjectsInfo から取得:
      → VersionInfo { version: 1.0.0, rev: 6b1c381, tagged: true }
    → ロックファイルに rev = "6b1c381..." として記録
```

**ポイント:**
- Step 4a で `version_retriever("math")` が呼ばれるが、`retrieve_versions()` は `self.versions.is_some()` で早期リターンし、タグスキャンは行われない。結果として pinned version 1件のみが返る
- Step 4b の推移的依存解決では、`local-a` が `math@^1.0` を要求するが、候補は pinned version の 1.0.0 のみ。`^1.0` は `>=1.0.0, <2.0.0` なので 1.0.0 はマッチし、解決成功
- Step 4c では root の `math` 依存（`version = "*"`）が処理されるが、math は既に fixed にあるため、requirement チェックのみ（"*" は何でもマッチ）
- Step 5 のロックファイル構築は完全に既存コード。`VersionInfo.rev` から OID を取得する処理は元々ある

#### `resolve_dependency` の変更

**変更なし。** pinned dep は `ProjectsInfo` 経由で retriever に渡され、resolver は通常の dep と同じように処理する。

#### この方式の利点

- **`resolve_dependency`, `PackageRetriever`, `VersionRetriever` のシグネチャ変更なし**
- **既存コードの最大限の再利用**: `prepre_git_repository`（clone）、`get_project_file` のチェックアウトパターン、`retrieve_versions` の早期リターン（`versions.is_some()` → return）
- **推移的依存の pin 無視が自然に実現**: `project_file_dep_to_dependency` は `Dependency { name, requirement }` のみを返し、`get_dependency_source` は `ProjectSource::Git(url, None)` を返す
- **`fix deps update` が自然に動作**: pinned dep の version_retriever は pinned version 1つのみを返すため、resolver は常にそのバージョンを選択する（「より新しい互換バージョン」が存在しない）
- **新規コードは `resolve_git_ref` と `resolve_pinned_ref` のみ**
```

### Phase 3: ロックファイルの変更

ロックファイルの構造自体は変更不要。現在でも `rev` フィールドにコミットハッシュが記録されており、インストール時にはそのOIDをチェックアウトしている。

ただし、ロックファイルの再生成（`fix deps update`）の動作を考慮する必要がある：

- **`rev` 指定**: 更新しても同じコミットに固定される（意図的）
- **`tag` 指定**: 更新しても同じタグのコミットに固定される（意図的）

### Phase 4: `fix deps update` の動作

| 指定方法      | `fix deps update` の動作                          |
|--------------|--------------------------------------------------|
| `version`    | SemVer互換の最新バージョンに更新（現状と同じ）         |
| `rev`        | 変更なし（固定される）                               |
| `tag`        | 変更なし（タグは不変のため）                          |
| なし         | SemVer互換の最新バージョンに更新（`version = "*"` 相当） |

### Phase 5: ハッシュ計算への影響 (`calculate_dependencies_hash`)

`calculate_dependencies_hash` は使用する `ProjectFileDependency` をシリアライズしてハッシュ化するため、`git.rev`, `git.tag` フィールドが追加されれば自動的にハッシュに含まれる。変更があればロックファイルの再生成がトリガされる。

`ProjectFileDependencyGit` は既に `Serialize` を実装しているため、フィールドを追加するだけで対応できる。

### Phase 6: ref固定依存が他の依存から推移的に必要とされるケース

ref固定は `fixproj.toml` に直接記述した依存にのみ適用される。推移的な依存（依存の依存）の `rev`/`tag` は無視され、`version` フィールドのみがバージョン制約として使用される（`version` 未指定の場合は `"*"`）。

**推移的依存の pin が無視される場合の警告**: 推移的依存の `fixproj.toml` に `rev`/`tag` が指定されている場合、依存解決ログ中に warning を出力する。これにより、ユーザーが意図せず pin が無視されている状況に気づけるようにする。

```
Warning: Dependency "math" in "local-a" specifies tag = "v1.0.0", but git ref pinning
         is only applied to direct dependencies. The tag specification will be ignored,
         and only the version requirement will be used for dependency resolution.
```

また、名前の衝突に注意が必要：

- ルートプロジェクトが `hash` を `rev` で固定
- 別の依存 `crypto` が `hash` を `version = "0.1"` で要求

この場合、ref固定で解決した `hash` のバージョンが、`crypto` が要求するバージョンレンジ内であるかチェックし、満たさない場合はエラーとする。ルートプロジェクトで `version` も同時に指定していた場合は、そのチェックも同様に行う。

```
Error: Dependency "hash" is pinned to commit abc1234 (version 0.2.0),
       but "crypto" requires version "^0.1".

Error: Dependency "hash" is pinned to commit abc1234 (version 0.2.0),
       but the version requirement "^0.1" specified in the root project is not satisfied.
```

### Phase 7: `fix deps add` コマンドへの影響

`fix deps add` はレジストリからプロジェクトを検索して `[[dependencies]]` を追加するコマンド。ref指定の依存は手動で追加する想定のため、`fix deps add` への変更は不要。

ただし将来的には以下のような拡張を検討しうる：
```bash
fix deps add hash --rev a1b2c3d
fix deps add hash --tag v0.2.0-rc1
```

これは初期実装のスコープ外としてよい。

## 使用例

### 特定のコミットに固定する

```toml
[[dependencies]]
name = "hash"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git", rev = "a1b2c3d4e5f6" }
```

`fix deps update` しても変わらない。

### プレリリースタグを使う

```toml
[[dependencies]]
name = "hash"
git = { url = "https://github.com/tttmmmyyyy/fixlang-hash.git", tag = "v0.2.0-rc1" }
```

SemVerタグとしては認識されないタグ名でも直接指定できる。

## 他言語・ツールとの比較

| ツール      | コミット | タグ   | ブランチ | 備考                          |
|------------|---------|--------|---------|-------------------------------|
| Cargo      | `rev`   | `tag`  | `branch` | `version` との併用可（追加制約） |
| Go modules | commit hash で `require` | - | - | `go get pkg@commit` |
| npm/yarn   | `#commit-ish` | URL fragment | URL fragment | `"pkg": "git+url#ref"` |
| pip        | `@commit` | `@tag` | `@branch` | `git+url@ref` |
| **Fix (提案)** | `rev` | `tag` | なし | `version` との併用可（制約として機能） |

### Cargoとの違い

Cargoは `version` と `rev`/`branch`/`tag` の併用を許可し、`branch` も指定可能。この場合、`version` は推移的依存の解決で使われ、`rev` 等は実際のチェックアウト先を決める。

Fixでも Cargo と同様に `version` と `rev`/`tag` の併用を許可する。`version` は「ref先のバージョンがこの範囲を満たさなければエラー」という制約として機能する。推移的依存からのバージョン要求が常に発生しうるため、ルートプロジェクトでも同じ制約を記述できるのが自然。

ただし `branch` 指定は `fix deps update` のたびにHEADが変わる不安定な動作となるため、サポートしない。特定時点に固定したい場合はそのコミットの `rev` を指定すればよい。将来的に需要があれば `branch` を追加する方向に拡張可能。

## エラーメッセージ例

```
Error: Only one of `rev` or `tag` can be specified in the `git` field of dependency "hash".

Error: Dependency "hash" is pinned to commit a1b2c3d (version 0.2.0 from its fixproj.toml),
       but "crypto" requires "hash" version "^0.1". Consider updating the pin or the version requirement.

Error: Dependency "hash" is pinned to commit a1b2c3d (version 0.2.0 from its fixproj.toml),
       but the version requirement "^0.1" specified in the root project is not satisfied.

Warning: Dependency "math" in "local-a" specifies tag = "v1.0.0", but git ref pinning
         is only applied to direct dependencies. The tag specification will be ignored,
         and only the version requirement will be used for dependency resolution.
```

## テスト計画

### 統合テスト

テスト対象リポジトリとして [fixlang-math](https://github.com/tttmmmyyyy/fixlang-math) を使用する。

**利用するタグとコミット:**

| タグ | コミット | 備考 |
|------|---------|------|
| `1.1.0` | `7602fba` | `v` prefix なし |
| `v1.0.0` | `6b1c381` | `v` prefix あり |
| `v0.1.3` | `ba88096` | `v` prefix あり |

#### テストケース配置

`src/tests/test_dependencies/cases/git_ref_tests/` にテストプロジェクトを配置。
各テストは `setup_test_env()` パターンに従い、一時ディレクトリにコピーして実行する。

---

#### テスト1: `rev` 指定でビルド成功

**目的**: コミットハッシュで固定した依存が正しく解決・インストール・ビルドできること。

**プロジェクト構成** (`git_ref_tests/rev_basic/`):

```
fixproj.toml
main.fix
```

`fixproj.toml`:
```toml
[general]
name = "rev-basic-test"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", rev = "7602fba" }
```

`main.fix`:
```fix
module Main;
import Math;
main: IO ();
main = println(Math::pi64.to_string);
```

**手順と期待値**:
1. `fix build` を実行 → 終了コード 0
2. `fixdeps.lock` が生成される
3. ロックファイル内の `math` エントリの `rev` が `7602fba` で始まること（先頭一致で完全ハッシュに展開される）
4. ロックファイル内の `math` エントリの `version` が `"1.1.0"` であること（そのコミットの `fixproj.toml` のバージョン）

---

#### テスト2: `tag` 指定でビルド成功

**目的**: タグ名でコミットを固定して正しくビルドできること。

**プロジェクト構成** (`git_ref_tests/tag_basic/`):

`fixproj.toml`:
```toml
[general]
name = "tag-basic-test"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", tag = "v1.0.0" }
```

`main.fix`: （テスト1と同じ）

**手順と期待値**:
1. `fix build` を実行 → 終了コード 0
2. `fixdeps.lock` 内の `math` の `rev` が `6b1c381` で始まること
3. ロックファイル内の `version` が `"1.0.0"` であること

---

#### テスト3: `rev` + `version`（バージョン要件を満たす）

**目的**: コミット固定と `version` 制約を併用し、要件を満たす場合に成功すること。

**プロジェクト構成** (`git_ref_tests/rev_with_version_ok/`):

`fixproj.toml`:
```toml
[general]
name = "rev-ver-ok-test"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "math"
version = "1.1"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", rev = "7602fba" }
```

`main.fix`: （テスト1と同じ）

**手順と期待値**:
1. `fix build` を実行 → 終了コード 0
2. コミット `7602fba` のバージョンは `1.1.0` であり、`version = "1.1"` の要件（`^1.1`）を満たすので成功

---

#### テスト4: `rev` + `version`（バージョン要件を満たさない → エラー）

**目的**: コミット固定のバージョンが `version` 制約を満たさない場合にエラーになること。

**プロジェクト構成** (`git_ref_tests/rev_with_version_fail/`):

`fixproj.toml`:
```toml
[general]
name = "rev-ver-fail-test"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "math"
version = "0.1"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", rev = "7602fba" }
```

`main.fix`: （テスト1と同じ）

**手順と期待値**:
1. `fix build` を実行 → **終了コード ≠ 0**
2. stderr にバージョン不一致のエラーメッセージが含まれること
   - 期待するメッセージの部分文字列: `"version requirement"` または `"is not satisfied"`
   - コミット `7602fba` のバージョンは `1.1.0` だが、`version = "0.1"` は `^0.1` = `>=0.1.0, <0.2.0` なので不一致

---

#### テスト5: `rev` と `tag` の同時指定 → バリデーションエラー

**目的**: `rev` と `tag` の両方を指定するとバリデーションエラーになること。

**プロジェクト構成** (`git_ref_tests/rev_and_tag_conflict/`):

`fixproj.toml`:
```toml
[general]
name = "rev-tag-conflict-test"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", rev = "7602fba", tag = "v1.0.0" }
```

`main.fix`: （テスト1と同じ）

**手順と期待値**:
1. `fix build` を実行 → **終了コード ≠ 0**
2. stderr に `"Only one of"` を含むエラーメッセージがあること

---

#### テスト6: `fix deps update` で `tag` 指定が変化しないこと

**目的**: `tag` で固定した依存は、SemVer互換の新バージョンが存在しても `fix deps update` でアップデートされないことを確認する。

**背景**: fixlang-math には `v1.0.0` (1.0.0) 以降に SemVer互換の `1.1.0`, `1.1.1`, `1.1.2`, `1.2.0`, `1.2.1` が存在する。通常の `version = "1.0"` 指定であれば `fix deps update` で最新互換バージョンに更新されるが、`tag` 固定ではそうならないことを検証する。

**プロジェクト構成** (`git_ref_tests/tag_update_stable/`):

`fixproj.toml`:
```toml
[general]
name = "tag-update-stable-test"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", tag = "v1.0.0" }
```

`main.fix`: （テスト1と同じ）

**手順と期待値**:
1. `fix build` を実行 → 終了コード 0、`fixdeps.lock` 生成
2. ロックファイル内の `math` の `rev` が `6b1c381` で始まること、`version` が `"1.0.0"` であることを確認
3. `fix deps update` を実行 → 終了コード 0
4. 再度ロックファイルを読み取り、`math` の `rev` が依然 `6b1c381` で始まること、`version` が依然 `"1.0.0"` であることを確認（1.2.1 等にアップデートされていないこと）

---

### 推移的依存の統合テスト

以下のテストでは、ルートプロジェクトとローカルプロジェクト A の両方が math に依存する構成を使用する。A は path 指定のローカル依存。

**共通プロジェクト構成:**

```
fixproj.toml          # ルートプロジェクト
main.fix
local_a/
  fixproj.toml        # ローカルプロジェクト A
  lib.fix
```

**共通 Fix コード:**

`local_a/lib.fix`:
```fix
module LocalA;

import Math;

local_a_value : F64;
local_a_value = Math::pi64;
```

`main.fix`:
```fix
module Main;

import LocalA;

main : IO ();
main = println(LocalA::local_a_value.to_string);
```

---

#### テスト7: Root が tag 指定、推移的依存 A がバージョンレンジ指定（互換あり → 成功）

**目的**: ルートが math を tag で固定し、path 依存の A が math を互換性のあるバージョンレンジで要求する場合に成功すること。

**プロジェクト構成** (`git_ref_tests/transitive_root_pins_ok/`):

Root `fixproj.toml`:
```toml
[general]
name = "transitive-root-pins-ok"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "local-a"
path = "local_a"

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", tag = "v1.0.0" }
```

`local_a/fixproj.toml`:
```toml
[general]
name = "local-a"
version = "0.1.0"
fix_version = "*"

[build]
files = ["lib.fix"]

[[dependencies]]
name = "math"
version = "1.0"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git" }
```

**手順と期待値**:
1. `fix build` を実行 → 終了コード 0
2. ロックファイル内の `math` の `rev` が `6b1c381` で始まること（Root の tag 指定が使用される）
3. ロックファイル内の `version` が `"1.0.0"` であること
   - A の `version = "1.0"` → `^1.0`（`>=1.0.0, <2.0.0`）を満たす

---

#### テスト8: Root が tag 指定、推移的依存 A がバージョンレンジ指定（互換なし → エラー）

**目的**: ルートが math を tag で固定し、A が math を互換性のないバージョンレンジで要求する場合にエラーになること。

**プロジェクト構成** (`git_ref_tests/transitive_root_pins_fail/`):

Root `fixproj.toml`:
```toml
[general]
name = "transitive-root-pins-fail"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "local-a"
path = "local_a"

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", tag = "v1.0.0" }
```

`local_a/fixproj.toml`:
```toml
[general]
name = "local-a"
version = "0.1.0"
fix_version = "*"

[build]
files = ["lib.fix"]

[[dependencies]]
name = "math"
version = "1.1"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git" }
```

**手順と期待値**:
1. `fix build` を実行 → **終了コード ≠ 0**
2. stderr にバージョン不一致を示すエラーメッセージがあること
   - math v1.0.0（tag 固定）は A の `version = "1.1"` → `^1.1`（`>=1.1.0, <2.0.0`）を満たさない

---

#### テスト9: Root がレンジ指定、A が tag 指定 → A の pin は無視される

**目的**: A が math を tag で固定していても、推移的依存では pin が無視され、ルートの SemVer 解決が使用されること。

**背景**: ref 固定は `fixproj.toml` に直接記述した依存にのみ適用される。推移的依存の `rev`/`tag` は無視され、`version` フィールドのみがバージョン制約として使用される。A が `version` を指定していない場合、制約は `"*"` となる。

**プロジェクト構成** (`git_ref_tests/transitive_local_pins/`):

Root `fixproj.toml`:
```toml
[general]
name = "transitive-local-pins"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "local-a"
path = "local_a"

[[dependencies]]
name = "math"
version = "1.0"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git" }
```

`local_a/fixproj.toml`:
```toml
[general]
name = "local-a"
version = "0.1.0"
fix_version = "*"

[build]
files = ["lib.fix"]

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", tag = "v1.0.0" }
```

（A は `tag = "v1.0.0"` のみ指定、`version` なし）

**手順と期待値**:
1. `fix build` を実行 → 終了コード 0
2. ロックファイル内の `math` の `version` が `"1.0.0"` **ではなく**、`^1.0` 互換の最新バージョン（現時点では `"1.2.1"`）であること
   - A の `tag = "v1.0.0"` は推移的依存では無視される
   - A の `version` は未指定 → 制約 `"*"` → Root の `version = "1.0"` による SemVer 解決のみが有効

---

#### テスト10: Root と A の両方が tag 指定（Root の pin が優先、A の pin は無視される）

**目的**: ルートと推移的依存の両方が math を異なる tag で固定していても、ルートの pin のみが使用され、A の pin は無視されること。

**プロジェクト構成** (`git_ref_tests/transitive_both_pin/`):

Root `fixproj.toml`:
```toml
[general]
name = "transitive-both-pin"
version = "0.1.0"
fix_version = "*"

[build]
files = ["main.fix"]

[[dependencies]]
name = "local-a"
path = "local_a"

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", tag = "v1.0.0" }
```

`local_a/fixproj.toml`:
```toml
[general]
name = "local-a"
version = "0.1.0"
fix_version = "*"

[build]
files = ["lib.fix"]

[[dependencies]]
name = "math"
git = { url = "https://github.com/tttmmmyyyy/fixlang-math.git", tag = "1.1.0" }
```

（Root は `v1.0.0` を、A は `1.1.0` を tag 指定 → 異なるコミットを指す）

**手順と期待値**:
1. `fix build` を実行 → 終了コード 0
2. ロックファイル内の `math` の `rev` が `6b1c381` で始まること（Root の `tag = "v1.0.0"` が使用される）
3. ロックファイル内の `version` が `"1.0.0"` であること
   - A は `tag = "1.1.0"` を指定しているが、推移的依存では無視される
   - A は `version` 未指定 → 制約 `"*"` → v1.0.0 は `"*"` を満たす → 成功

---

#### Rustテストコードの概形

```rust
#[test]
fn test_git_rev_basic() {
    install_fix();
    let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_basic");
    cleanup_test_project(&project_dir);

    // fix build
    let output = Command::new("fix")
        .arg("build")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run fix build");
    assert!(output.status.success(), "fix build failed: {}", String::from_utf8_lossy(&output.stderr));

    // Check lock file
    let lock_content = fs::read_to_string(project_dir.join("fixdeps.lock"))
        .expect("Lock file not found");
    assert!(lock_content.contains("7602fba"), "Lock file should contain the pinned rev");
    assert!(lock_content.contains("version = \"1.1.0\""), "Lock file should show version 1.1.0");
}

#[test]
fn test_git_rev_with_version_fail() {
    install_fix();
    let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_with_version_fail");
    cleanup_test_project(&project_dir);

    let output = Command::new("fix")
        .arg("build")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run fix build");
    assert!(!output.status.success(), "fix build should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("version") && stderr.contains("not satisfied"),
        "Error message should mention version mismatch: {}", stderr
    );
}
```

`setup_git_ref_test_env(case_name)` は `get_test_cases_dir().join("git_ref_tests").join(case_name)` を一時ディレクトリにコピーし、そのパスを返すヘルパー。

## まとめ

| 項目 | 内容 |
|------|------|
| 新規フィールド | `git.rev`, `git.tag` |
| `version` との関係 | 併用可（`version` はバージョン制約として機能） |
| `path` との関係 | 変更なし（`path` と `git` は排他） |
| ロックファイル変更 | 不要（既に `rev` を保持） |
| 依存解決方式 | ref指定ありはProjectsInfoにpre-register（既存resolver・retrieverは変更なし） |
| 推移的依存 | ref固定は直接依存のみ。推移的依存のバージョン要件との整合性チェックあり |
| `fix deps update` | `rev`/`tag` は固定（変化しない） |
