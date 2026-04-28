# Fix の deprecation 機能仕様（プラグマ式）

## Context

ライブラリ作者がグローバル値（トップレベルの関数・値）およびトレイトメンバーを「deprecated」として宣言し、利用者が参照したときにコンパイル時警告を出せるようにする。

設計方針として、既存の `FFI_EXPORT[name, c_name];` トップレベル文と同じ流儀の **プラグマ方式** を採用する。理由：

- Fix では宣言 (`name : type;`) と定義 (`name = expr;`) を別個に書けるため、属性方式 `#[deprecated(...)]` を導入するとどちらに書くべきかという迷いが生じる。プラグマ方式ならこの問題が原理的に発生しない。
- 既存の `FFI_EXPORT` の判断と一貫した文法になる。
- Haskell の `{-# DEPRECATED foo "msg" #-}` プラグマと思想が同じで、利用者が学びやすい。

`fix` コンパイラには現状、警告（非致命的診断）の仕組みが存在せずすべてエラー扱いになっているので、本機能の追加に合わせて最小限の警告基盤も整備する。

## 目標と非目標

### 目標 (v1)

- 新しいトップレベル文 `DEPRECATED[name_path, "msg"];` を導入。
  - グローバル値とトレイトメンバーを deprecated にできる。
  - トップレベル、`namespace { ... }` 内、`trait { ... }` 内に書ける。
  - 第 1 引数は `::` 区切りの **パス記法**（grammar の `fullname` ルールを流用するが、意味は **書かれた場所からの相対パス**。プロジェクトのトップレベルモジュールからの絶対パスではない）。
- 既存 `FFI_EXPORT[name, c_name];` の第 1 引数も同様に `::` 区切りパス記法を受理するよう拡張（一貫性のため。意味は同じく相対パス）。
- 該当宣言を参照する箇所で警告を出す（コンパイルは継続）。警告メッセージにはユーザー指定文字列、参照位置、宣言位置（および DEPRECATED プラグマ位置）を含める。
- CLI フラグ `--allow-deprecated`（抑制）と `--deny-deprecated`（昇格してエラー扱い）を提供。両方同時指定は CLI エラー。
- LSP では `DiagnosticSeverity::WARNING` + `DiagnosticTag::DEPRECATED` で配信。
- LSP の rename / find-references / goto-definition が `FFI_EXPORT[...]` および `DEPRECATED[...]` プラグマ内の名前にも作用するようにする（既存 LSP 機能の漏れ対応も兼ねる）。

### 非目標 (将来作業)

- 属性構文 `#[deprecated(...)]` / `#[export(...)]` の導入。今回は採用しない。
- 個別の `#[allow(deprecated)]` のような局所抑制。
- 型・トレイト本体・モジュールの deprecation。
- since バージョン情報や複数名指定 (`DEPRECATED[foo, bar, "msg"];` 等) の対応。
- 他種プラグマの追加（`INLINE` 等）。

## 構文

### `DEPRECATED` プラグマ

```
deprecated_statement = { "DEPRECATED" ~ sep* ~ "[" ~ sep* ~ fullname ~ sep* ~ "," ~ sep* ~ string_lit ~ sep* ~ "]" ~ sep* ~ semicolon }
```

許容位置：
- トップレベル
- `namespace { ... }` 内（`namespace_member` の選択肢に追加）
- `trait { ... }` 内（`trait_member_defn` の選択肢に追加）

### 対象同定

書いた場所のコンテナパスと書かれた相対パスを **連結** して `FullName` を作り、その値で直接 `Program` 内のエンティティを引き当てる（候補探索や名前空間推論は一切行わない）。書いた場所の名前空間の **子孫** に属するエンティティしか deprecate できないのは、この連結規則の帰結。

具体的には：

- パース時、書いた場所の「コンテナパス」を記録する。
  - トップレベル → ルート（空パス）
  - `namespace Foo { ... }` 内 → `Foo`
  - `namespace Foo::Bar { ... }` 内 → `Foo::Bar`
  - `trait MyTrait { ... }` 内（namespace `Foo` に書かれた trait であれば）→ `Foo::MyTrait`
- 連結後の `FullName` で `Program` のグローバル値テーブル / トレイトメンバーレジストリを引く。import 経由・親 namespace への遡上・他名前空間の検索はしない。連結結果で見つからなければエラー。
- **絶対パス記法（先頭 `::Foo::bar`）は禁止**。ただし文法レベルでは弾かず（パースエラーは原因が分かりにくい）、通常の `fullname` ルールでパースしたあとエラボレーション時に検査し、`"DEPRECATED の対象に絶対パスは指定できません。書かれた場所からの相対パスで指定してください。"` のような分かりやすいエラーを出す。`FFI_EXPORT` についても同様に検査する。
- 連結結果は単一の `FullName` なので、候補探索を行うフェーズが無く、曖昧性は発生し得ない（連結結果が存在するか・対象種別が許容されるかだけを判定する）。
- ターゲットが `GlobalValue`（普通のグローバル値）またはトレイトメンバーのいずれであっても受理。型・トレイト本体・モジュールは受理しない（v1 では未対応。型自体を deprecate せず、その型を使う関数を deprecate していけば利用は減るという発想で十分なはず）。
- 同じ対象に対して複数の `DEPRECATED` 宣言があった場合はエラー。

この規則の帰結：

- トレイト本体内では当該トレイトのメンバーしか deprecate できない（外部のトレイトメンバーや別 namespace の関数は不可）。
- `namespace Foo` の中で `Bar::baz` を deprecate するとそれは `Foo::Bar::baz` を指す。`Bar::baz`（外部にある）を指すことはできない。
- 同じ規則を `FFI_EXPORT` にも適用する。`namespace Foo { FFI_EXPORT[bar, c_bar]; }` は `Foo::bar` を export し、`namespace Foo { FFI_EXPORT[Baz::qux, c_qux]; }` は `Foo::Baz::qux` を export する（既存挙動と互換）。

例：

```fix
// 古い関数。代わりに `new_func` を使ってください。
old_func : I64 -> I64;
old_func = |x| x + 1;
DEPRECATED[old_func, "Use `new_func` instead."];

namespace Foo {
    bar : I64 -> I64;
    bar = |x| x;
    DEPRECATED[bar, "Will be removed in 2.0."];   // 短名で OK
}

// トレイト内
trait a : MyTrait {
    old_method : a -> String;
    DEPRECATED[old_method, "Use `new_method` instead."];   // 短名で OK
}

// トレイト外から指す場合はパスで
DEPRECATED[MyTrait::old_method, "Use `new_method` instead."];
```

### `FFI_EXPORT` 拡張

文法を `name` から `fullname`（grammar 上のルール名。`::` 区切りのパス記法を許容するもので、bare name もこれに含まれる）に変更：

```
export_statement = { export_symbol ~ sep* ~ "[" ~ sep* ~ fullname ~ sep* ~ "," ~ sep* ~ exported_c_function_name ~ sep* ~ "]" ~ sep* ~ semicolon }
```

挙動：
- 既存の `FFI_EXPORT[bar, c_bar];`（bare name）はそのまま動く（bare name は valid な fullname の特殊形）。
- `FFI_EXPORT[Foo::bar, c_bar];` のようにパス指定も書けるようになる。
- 連結ルール（コンテナパス ++ 書かれた相対パス）と絶対パス禁止は `DEPRECATED` と完全に共通。
- 既存 [parse_export_statement](src/parse/parser.rs#L823) の `FullName::new(&ctx.namespace, &fix_value_name)` を、`fullname` ルールでパースされた `FullName` 値（こちらは Rust の構造体）と `ctx.namespace` を連結する関数に置き換える。

`FFI_EXPORT` には既に [validate_export_statements](src/ast/program.rs#L1610) という elaboration 時検証ステップがあり、ここで未知の `value_name` などが検出されている。`DEPRECATED` の存在検査・絶対パス検査・対象種別検査も同じ場所（または隣接するパス）に同居させ、絶対パスや存在しないターゲットの検出ロジックを `FFI_EXPORT` と `DEPRECATED` の共通ヘルパに切り出す。差分最小化は目的としない — 理想的なコード構造を優先する。

## AST 変更

新規ファイル `src/ast/deprecation.rs`:

```rust
#[derive(Clone)]
pub struct DeprecationStatement {
    // パース時に得られる、書かれたままの相対パス（連結前）。
    // `FullName` は Rust の構造体名であり、ここに入る値はトップレベル絶対パスではなく
    // 「書かれた場所からの相対パス」を表す。
    pub target_path: FullName,
    // 名前トークン部分の span（rename / find-references 用）。
    pub target_name_src: Option<Span>,
    // 連結時のコンテナパス。書かれた場所が
    //   - トップレベル → ルート（空）
    //   - `namespace Foo { ... }` 内 → `Foo`
    //   - `namespace Foo { trait a : MyTrait { ... } }` 内 → `Foo::MyTrait`
    // となる。トレイト本体は名前空間の一段として `origin_namespace` に折り畳んで保持する。
    pub origin_namespace: NameSpace,
    // 警告メッセージ。
    pub message: String,
    // プラグマ全体の span。
    pub src: Option<Span>,
}

#[derive(Clone)]
pub struct DeprecationInfo {
    pub message: String,
    // DEPRECATED プラグマの span（警告の related info に出す）。
    pub statement_src: Option<Span>,
}
```

[src/ast/program.rs](src/ast/program.rs):
- `Program` に `deprecation_statements: Vec<DeprecationStatement>` フィールドを追加（パース時に積む）。
- `GlobalValue` に `deprecation: Option<DeprecationInfo>` を追加。エラボレーション中に `deprecation_statements` のターゲットを引き当てて該当 `GlobalValue` にセットする。
- `merge`（[program.rs:2133](src/ast/program.rs#L2133) 周辺）で `deprecation_statements` も結合。

[src/ast/traits.rs](src/ast/traits.rs):
- `TraitMember` に `deprecation: Option<DeprecationInfo>` を追加。
- トレイトメンバーから派生する `GlobalValue` 群に伝播コピーする（既存の trait method 展開ロジックに 1 行追加）。これにより `Expr::Var` 解決後の検出は `GlobalValue.deprecation` だけ見れば済む。

[src/ast/export_statement.rs](src/ast/export_statement.rs):
- `ExportStatement` に `value_name_src: Option<Span>` を追加（`FFI_EXPORT[<ここ>, c_name];` の名前トークン span。LSP rename / find-references 用）。
- 既存 `src` フィールドはプラグマ全体の span として残す。後段（[src/optimization/](src/optimization/) 配下の `program.export_statements` ループ等）への波及はなし。

## 文法・パーサ変更

[src/parse/grammer.pest](src/parse/grammer.pest):
- `deprecated_statement` ルールを追加（上記）。
- `export_statement` の第 1 引数を `name` → `fullname` に変更。
- トップレベルおよび `namespace { ... }` の本体に `deprecated_statement` を許容（既存 `export_statement` が許可されている箇所と同じ）。
- `trait_member_defn` の選択肢に `deprecated_statement` を追加。

[src/parse/parser.rs](src/parse/parser.rs):
- 新規 `parse_deprecated_statement(pair, ctx) -> DeprecationStatement`。`ctx.namespace` をそのまま `origin_namespace` に格納する（後述のとおり trait 本体内では `ctx.namespace` 自体に trait 名が含まれている前提）。
- [parse_export_statement](src/parse/parser.rs#L823) を fullname 対応に修正。`FullName::new(&ctx.namespace, &fix_value_name)` を、`fullname` ルールでパースされた `FullName` 値を `ctx.namespace` を起点に連結する関数に置き換え。
- `parse_trait_defn`（[parser.rs:509 周辺](src/parse/parser.rs#L509)）で trait 本体に入るときに `ctx.namespace` の末尾に trait 名を push、抜ける時に pop する。これにより trait body 内のパースは「コンテナパスを 1 段深くした namespace に入っている」だけの扱いになる（`origin_trait` のような追加フィールドは不要）。
- トップレベル / namespace 本体のディスパッチ箇所（[parser.rs:429 付近の `Rule::export_statement` 分岐](src/parse/parser.rs#L429)）に `Rule::deprecated_statement` 分岐を追加。`Program.deprecation_statements` に push。
- 同様に `parse_trait_member_defn`（[parser.rs:576](src/parse/parser.rs#L576)）に `Rule::deprecated_statement` 分岐を追加。trait body 内では `ctx.namespace` がすでに trait まで伸びているので、構築ロジックはトップレベル / namespace 内と完全に同じ。

## エラボレーション変更

新規ステップとして「deprecation 対象同定パス」を追加（[src/elaboration/mod.rs](src/elaboration/mod.rs) で名前解決後・型推論前あたり、`validate_export_statements` の隣接ポジションが望ましい）:

- `Program.deprecation_statements` を順に処理する。
- 各文について、`origin_namespace` と `target_path` を **連結** して `FullName` を作り、`Program` のグローバル値テーブル / トレイトメンバーレジストリを直接ルックアップ：
  - ターゲットが `GlobalValue` → `GlobalValue.deprecation` を `Some(...)` でセット。
  - ターゲットがトレイトメンバー → `TraitMember.deprecation` をセット、加えて派生 `GlobalValue`（メソッドオーバーロード）にも伝播。
  - 引き当て失敗 → エラー（"DEPRECATED target `<path>` not found under `<container>`."）。
  - ターゲットが型・トレイト本体・モジュール等 → `"v1 では関数およびトレイトメンバーのみ deprecation 対応"` エラー。
- 同じ対象に対して複数の DEPRECATED が当たった場合：エラーにする（重複定義は意図的でないことが多い）。

## 診断インフラ変更（警告サポート）

[src/error.rs](src/error.rs):
- 新しい `enum Severity { Error, Warning }` を追加。
- `Error` 構造体に `pub severity: Severity` を追加。デフォルトは `Severity::Error`（既存コンストラクタは従来通り）。
- 新コンストラクタ `Error::warning_from_msg_srcs(msg, srcs) -> Error`（`severity = Warning`）。
- `Errors::has_error(&self) -> bool` は **エラーのみ** をカウント（警告は失敗扱いしない）。
- 新メソッド `Errors::has_diagnostics(&self) -> bool`（エラー＋警告）。
- `Error::to_string()` で severity に応じてラベル `"error"`/`"warning"` と色（赤/黄）を切り替え。
- 定数 `WARN_DEPRECATED = "deprecated"` を追加。

[src/commands/lsp/server.rs](src/commands/lsp/server.rs#L952):
- `error_to_diagnostics` で `severity: err.severity` に応じて `DiagnosticSeverity::ERROR` / `WARNING` を選択。
- `code == WARN_DEPRECATED` のとき `tags: vec![DiagnosticTag::DEPRECATED]` を付与（IDE 上で取り消し線表示）。

## 使用箇所での検出

検出ポイント：[src/elaboration/typecheck.rs:725](src/elaboration/typecheck.rs#L725) の `Expr::Var` 解決成功側（`ok_count == 1` ブランチ、l.873）。

ここで一意に解決された `(ns, var.name.name)` から `Program` の `GlobalValue` を引き、`deprecation` が `Some(info)` なら警告を生成。トレイトメソッドも `GlobalValue` として解決されるため、ここ一箇所で網羅できる（前述のとおり `TraitMember.deprecation` を派生 `GlobalValue` 群に伝播済みであることが前提）。

警告の構造：
- `severity`: `Warning`
- `code`: `Some(WARN_DEPRECATED)`
- `msg`: ``"`{fullname}` is deprecated: {message}"``
- `srcs[0]`: 使用箇所 `ei.source.clone()`
- `srcs[1]`: `("Declared here:", decl_src)` 宣言位置
- `srcs[2]`: `("Deprecation declared here:", statement_src)` プラグマ位置（任意）

警告の蓄積先：既存の `Program.deferred_errors`（[program.rs:508](src/ast/program.rs#L508)）を流用するか、新フィールド `Program.deferred_warnings` を追加。前者の方が変更が小さい（`Errors::has_error()` の意味変更で `Severity::Warning` は失敗扱いされなくなるため、deferred_errors にそのまま入れても OK）。

`--no-deprecation-warnings` モードでは警告を生成しない。`--deny-deprecated` モードでは `severity = Error` として生成。

## CLI フラグ

[src/main.rs](src/main.rs):
- `--allow-deprecated`: deprecation 警告を抑制（生成しない）。
- `--deny-deprecated`: deprecation 警告をエラー扱いに昇格（コンパイル失敗）。
- 両方同時指定は CLI エラー（早期に検出して `fix --help` を案内）。

`Configuration` 構造体（要調査して該当ファイル特定）に `deprecation_mode: DeprecationMode { Warn, Allow, Deny }` を追加。検出時に参照して挙動を切り替える：
- `Allow` → 警告生成スキップ
- `Warn` → `Severity::Warning`
- `Deny` → `Severity::Error`（他のエラーと同様に集約報告。早期失敗ではなく `Errors` に積むだけ）

`fix lsp` モードでは常に `Warn`（IDE 側のサーバ起動時にフラグを渡さない設計を維持。エディタ統合で deny 表示が必要になったら将来検討）。

## LSP rename / find-references / goto-definition 対応

`FFI_EXPORT[name, ...]` および `DEPRECATED[name, ...]` プラグマ内の名前トークンも、既存の rename / find-references / goto-definition 機能で対象になるようにする。これは本タスクで追加する `DEPRECATED` 用の対応であると同時に、既存の `FFI_EXPORT` 漏れ（v1.3.0-beta.6 までの rename では対象外）の修正も兼ねる。

### 対象データの取得点

1. `ExportStatement.value_name_src` および `DeprecationStatement.target_name_src` に名前トークンの span を保持する（既述の AST 変更）。
2. これらは `FullName` への解決後の名前を持つので、find-references 側で「ある `FullName` を参照している箇所」のリストに含めればよい。

### 変更ファイル

- [src/commands/lsp/references.rs](src/commands/lsp/references.rs) — グローバル値・トレイトメンバーの参照収集に、`Program.export_statements` と `Program.deprecation_statements` をスキャンするロジックを追加。マッチした場合 `value_name_src` / `target_name_src` を結果に含める。
- [src/commands/lsp/rename.rs](src/commands/lsp/rename.rs) — references が新たな span を返すようになるだけで自動的に rename 対象に含まれるはずだが、`collect_*_rename_edits`（[rename.rs:222 以降](src/commands/lsp/rename.rs#L222)）の実装が「ASTノードのうち何を巡回するか」を直接持っている場合、プラグマ走査を追記する。実装時に確認。
- [src/commands/lsp/goto_definition.rs](src/commands/lsp/goto_definition.rs) — カーソルがプラグマ内の名前トークン上にあるとき、対応する宣言にジャンプできるよう、`Program::find_node_at` 系の探索（または goto_definition 側の手前のディスパッチ）にプラグマ巡回を追加。`EndNode` のバリアントとしては既存の `EndNode::Expr(var, _)`（グローバル値参照）または専用の `EndNode::ExportRef` / `EndNode::DeprecationRef` を新設するかは実装時判断。最小は前者（プラグマ内の名前を「グローバル値の参照」と同等に扱う）。
- [src/ast/program.rs](src/ast/program.rs) — `Program::find_node_at` 相当の関数（要調査）にプラグマ巡回を追加。

### 注意点

- rename で名前を書き換える際、プラグマ内の名前トークンは **短名形式** で書かれていることが多い（`DEPRECATED[bar, "..."]` のように）。新しい名前が short の場合はそのまま置換、qualified に変わる場合は適切に書き直す必要がある。既存の global value rename ロジックで同等のケースをどう扱っているか確認して合わせる。
- パスを含む形（`DEPRECATED[Foo::bar, "..."]`）の場合、rename の対象は **末尾の name 部分のみ**（`Foo::` は名前空間で、rename 対象ではない）。span を「`bar` だけ」に絞って記録するよう注意。

## 重要な変更ファイル

- [src/parse/grammer.pest](src/parse/grammer.pest) — `deprecated_statement` ルール追加、`export_statement` の `name` を `fullname` に変更、トップレベル / namespace / trait の各位置に許容。
- [src/parse/parser.rs](src/parse/parser.rs) — `parse_deprecated_statement` 追加、`parse_export_statement` を fullname 対応に修正、trait 本体での `ctx.namespace` push/pop、ディスパッチ箇所更新。
- 新規 [src/ast/deprecation.rs](src/ast/deprecation.rs) — `DeprecationStatement` / `DeprecationInfo` 型。
- [src/ast/program.rs](src/ast/program.rs) — `Program.deprecation_statements`、`GlobalValue.deprecation` フィールドを追加、`merge` 更新。
- [src/ast/traits.rs](src/ast/traits.rs) — `TraitMember.deprecation` フィールド。
- [src/elaboration/mod.rs](src/elaboration/mod.rs) — deprecation 対象同定パスの追加（`validate_export_statements` 周辺と共通ヘルパを共有）。
- 既存の trait method 展開ロジック — `TraitMember.deprecation` を派生 `GlobalValue` に伝播（実装時に該当箇所を確定）。
- [src/elaboration/typecheck.rs](src/elaboration/typecheck.rs#L725) — `Expr::Var` 解決成功時に warning 発火。
- [src/error.rs](src/error.rs) — `Severity` 追加、`Error.severity`、警告コンストラクタ、`Errors::has_error` の意味変更、`to_string` の色分け、`WARN_DEPRECATED` 定数。
- [src/commands/lsp/server.rs](src/commands/lsp/server.rs#L952) — `error_to_diagnostics` で severity と DEPRECATED tag を反映。
- [src/commands/lsp/references.rs](src/commands/lsp/references.rs) — `FFI_EXPORT` / `DEPRECATED` プラグマ内の名前 span を find-references の結果に含める。
- [src/commands/lsp/rename.rs](src/commands/lsp/rename.rs) — references の拡張で自動的に rename 対象になることを確認、必要なら直接巡回を追加。
- [src/commands/lsp/goto_definition.rs](src/commands/lsp/goto_definition.rs) — プラグマ内名前トークンからの goto-definition を有効化。
- [src/ast/export_statement.rs](src/ast/export_statement.rs) — `ExportStatement.value_name_src` 追加。
- [src/main.rs](src/main.rs) — `--allow-deprecated` / `--deny-deprecated` フラグ。
- `Configuration` の定義箇所（要調査）— `deprecation_mode` を追加。

## ドキュメント / CHANGELOG 更新

### [Document.md](Document.md) と [Document-ja.md](Document-ja.md)

- 既存の FFI_EXPORT セクション（[Document.md](Document.md) line 2161 付近、[Document-ja.md](Document-ja.md) line 2265 付近）を更新：
  - 第 1 引数が `::` 区切りのパスを受理することを追記。
  - 「書かれた場所からのコンテナパス連結」というルールの説明と例（`namespace Foo { FFI_EXPORT[Bar::baz, c_baz]; }` → `Foo::Bar::baz` を export）。
  - 絶対パスは指定不可。
- 新規セクション「Deprecation」を追加：
  - `DEPRECATED[name_path, "msg"];` の構文と意味。
  - 許容位置（トップレベル / namespace 内 / trait 本体内）。
  - パスの連結ルール（FFI_EXPORT と完全に共通）。
  - 利用例（グローバル値、トレイトメンバー、namespace 内、trait 内）。
  - エラーケース（存在しないターゲット、絶対パス、対象種別違反、重複）。
- CLI セクションに `--allow-deprecated` / `--deny-deprecated` の説明を追加（両者排他）。

ja 版にも同等の和訳を追加する。

### [CHANGELOG.md](CHANGELOG.md) の `[Unreleased]`

- `### Added`:
  - **Language**: `DEPRECATED[name, "msg"];` プラグマで関数・トレイトメンバーを deprecated にできるようになった。参照箇所でコンパイラ警告を出す。
  - **Tool**: `--allow-deprecated` / `--deny-deprecated` CLI フラグ。前者は警告抑制、後者は警告をエラーに昇格。
  - **LSP**: deprecated アイテムは IDE 上で取り消し線表示、ホバーでメッセージ表示。`FFI_EXPORT[...]` / `DEPRECATED[...]` プラグマ内の名前トークンにも rename / find-references / goto-definition が効くようになった（`FFI_EXPORT` 側は従来の漏れの修正を兼ねる）。
- `### Changed`:
  - **Language**: `FFI_EXPORT[name, c_name];` の第 1 引数が `::` 区切りパスを受理するようになった（`FFI_EXPORT[Foo::bar, c_bar];` のように書ける）。bare name 形の既存コードは引き続き動作。

## 検証計画

`CLAUDE.md` の方針に従い、`fix` コマンドの挙動はインテグレーションテストで検証する（ユニットテストは増やさない）。

新規テストプロジェクトを `src/tests/test_deprecation/cases/` 配下に置く：

1. **基本警告**: グローバル関数 `old_func` を `DEPRECATED[old_func, "..."]` でマークし、別ファイルから呼び出して `fix build` を走らせ、stderr に `warning:` と指定メッセージが含まれること、終了コード 0 であることを確認。
2. **トレイトメンバー警告 (内側 DEPRECATED)**: `trait { ... DEPRECATED[old_method, "..."]; }` で短名指定し、impl 経由で呼び出して警告が出ること。
3. **トレイトメンバー警告 (外側 DEPRECATED)**: `DEPRECATED[MyTrait::old_method, "..."]` のパス指定で同様の警告が出ること。
4. **namespace 内**: `namespace Foo { bar : ...; DEPRECATED[bar, "..."]; }` が動作すること。
5. **コンパイル成功**: deprecation 警告のみの状態で `fix run` が正常に実行されることを確認。
6. **`--deny-deprecated`**: 同プロジェクトに `--deny-deprecated` を渡すと終了コード非 0 で失敗すること。
7. **`--no-deprecation-warnings`**: フラグを渡すと stderr に warning が出ないこと。
8. **エラーケース**:
   - 存在しない名前 `DEPRECATED[no_such_func, "..."]` → エラー
   - 同じ対象に複数の DEPRECATED → エラー
   - 型を指す `DEPRECATED[I64, "..."]` → エラー
   - **コンテナ外の名前を指す**: `trait MyTrait { DEPRECATED[OtherTrait::foo, "..."]; }` → エラー（`MyTrait` の子孫でないため）
   - **コンテナ外の名前を指す（namespace 越え）**: `namespace Foo { DEPRECATED[Bar::baz, "..."]; }` で `Foo::Bar::baz` が存在しなければエラー（外部の `Bar::baz` には到達不能）
9. **`FFI_EXPORT` パス対応**: `FFI_EXPORT[Foo::bar, c_bar];` がパスありで動作することを確認。既存の bare name 形 `FFI_EXPORT[bar, c_bar];` も引き続き動作すること（既存 [test_basic.rs](src/tests/test_basic.rs#L6834) のテスト群が変更なしで通る）。
10. **絶対パス禁止**: `DEPRECATED[::Foo::bar, "..."]` および `FFI_EXPORT[::Foo::bar, c_bar];` がエラボレーション時に分かりやすいエラー（パースエラーではなく "絶対パスは指定できません" 系）になること。
11. **LSP rename**: 既存の rename テスト（[src/tests/test_lsp/](src/tests/test_lsp/) 配下）と同形式で、deprecated / exported された関数を rename したときに `DEPRECATED[...]` / `FFI_EXPORT[...]` 内の名前も書き換わることを確認。

加えて、Fix 標準ライブラリの簡単な関数を一時的に `DEPRECATED` でマークしてフルテストを通し、無関係箇所での誤発火がないこと、リグレッションがないことを確認。

LSP の手動確認：VS Code で deprecated 関数を呼び出すコードに取り消し線が引かれ、ホバーでメッセージが表示されることを目視確認。`DEPRECATED` / `FFI_EXPORT` プラグマ内の名前で goto-definition / rename が効くことも目視確認。

## 実装順序の目安

1. 診断インフラ拡張（`Severity`, `Errors::has_error`, LSP 経路、`WARN_DEPRECATED` 定数）— 既存テストが通ることを確認。
2. AST スケルトン（`DeprecationStatement` / `DeprecationInfo` 型、`Program` / `GlobalValue` / `TraitMember` のフィールド追加。`ExportStatement.value_name_src` 追加。デフォルト値で埋めてビルドが通る状態に）。
3. `FFI_EXPORT` の fullname 対応 + `value_name_src` セット（先に出してリグレッションテストで安心したい）。
4. 文法・パーサ拡張（`DEPRECATED[...]` 受理、trait 本体での `ctx.namespace` push/pop）。
5. エラボレーション対象同定パス（`deprecation_statements` のターゲット引き当て、重複検出、絶対パス検出エラー、`validate_export_statements` と共通ヘルパ抽出）。
6. トレイトメンバー → 派生 `GlobalValue` への deprecation 伝播。
7. `Expr::Var` 解決時の警告発火（`Configuration.deprecation_mode` を反映）。
8. CLI フラグ (`--allow-deprecated` / `--deny-deprecated`)。
9. LSP find-references / rename / goto-definition をプラグマ内名前に拡張。
10. インテグレーションテスト追加（deprecation 機能 + rename 拡張）。
11. [Document.md](Document.md) / [Document-ja.md](Document-ja.md) / [CHANGELOG.md](CHANGELOG.md) を更新。
