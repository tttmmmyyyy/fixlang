# 計画: LSP `textDocument/rename` 対応

## 背景

直前の作業で **ローカル名の jump-to-definition** と **find-references**
(ローカル含む) を実装した。本ブランチの本来の目的である rename symbol を
ここから組み立てる。

ユーザーが事前に挙げた論点 4 件:

1. import 文中の使用は references でヒットしない (現仕様)。rename では
   import 文も書き換える必要があるので、refs の方を import を含むように
   拡張して、rename をその上に乗せたほうが単純では?
2. ユーザー定義でないシンボル (Std や依存パッケージ) は rename を断る
   べき。
3. 構造体フィールド `x` には自動実装メソッド `@x` / `set_x` / `mod_x` /
   `act_x` などが付く。union variant も同様。フィールド名 rename 時には
   これらも一緒に rename する必要があり、references でも自動実装メソッドの
   呼び出し箇所を一括で出すべきかもしれない。
4. diagnostics が通っていない / 直近 elaboration 後にバッファが変更された
   状態で rename を要求された場合は拒否すべきか?

各論点への本計画の方針は §3 にまとめる。

## 1. 現状の確認 (前提)

### LSP capability / dispatch
- [server.rs:625](../../src/commands/lsp/server.rs#L625) — `rename_provider: None`。
  `prepare_provider` も未対応。
- [server.rs:455](../../src/commands/lsp/server.rs#L455) 付近にメソッド名で
  分岐するディスパッチがあり、ここに `textDocument/rename` と
  `textDocument/prepareRename` を追加する穴を開ける。
- 直前のリクエストとの整合のため、handler はすべて
  `program: &Program, uri_to_content: &Map<Uri, LatestContent>` を受ける形。

### 既存の references で扱える EndNode
[references.rs:67-104](../../src/commands/lsp/references.rs#L67-L104) の
`find_all_references` の分岐:

- `Expr(var, _)` / `Pattern(var, _)` — local / global 値
- `ValueDecl(name)` — 宣言 LHS
- `Type(tycon)` — 型
- `Trait(trait_id)` — トレイト
- `TypeOrTrait(name)` — 区別がついていない名前 (型エイリアス含む)
- `AssocType(assoc_type)` — 関連型
- `Module(_)` — 未対応 (空 Vec)

[program.rs:2403-2414](../../src/ast/program.rs#L2403-L2414) の `EndNode` には
**フィールド/variant のヴァリアントが無い**。よってフィールド名 / variant 名の
宣言箇所をクリックしても現状は `Pattern` / `Expr` にすら降りてこない可能性が
高い (使用箇所側 `MakeStruct` のフィールド名や、let-pattern の Struct 内の
field-name のスパンも未トラッキング)。

### import 文の構造
[import.rs:13-22, 209-215](../../src/ast/import.rs#L13-L22) — `ImportStatement.items`
と `hiding` は `ImportTreeNode` のツリー。各葉は:

- `Symbol(Name, Option<Span>)` — 小文字始まりの値
- `TypeOrTrait(Name, Option<Span>)` — 大文字始まりの型/トレイト
- `NameSpace(Name, Vec<ImportTreeNode>, Option<Span>)` — 内部ノード
- `Any(Option<Span>)` — `*` (rename 対象外)

`find_node_at` で `EndNode::Expr(var, None)` / `EndNode::TypeOrTrait(name)` /
`EndNode::Module(name)` を返す。**現在、refs 側は import 文内のリーフを
「見つけられているのに集計しない」状態** ではなく、**そもそも
`program.modules / mod_to_import_stmts` を辿っていない**。

### references の global 集計
[references.rs:132-162](../../src/commands/lsp/references.rs#L132-L162) は
`program.global_values` 全部を `collect_symbol_expr_var_refs` で舐める。
ここに「全 import 文の `ImportTreeNode::Symbol` も舐める」一節を加えれば、
import を込みで集計できる。

### 自動実装メソッドの命名
[program.rs:1866-2024](../../src/ast/program.rs#L1866-L2024)、
[constants.rs:77-85](../../src/constants.rs#L77-L85) — 構造体フィールド `x`
について同じ namespace に以下が `add_compiler_defined_method` で追加される:

- `@x` / `set_x` / `mod_x` / `act_x`
- 内部用 `_act_x_identity` / `_act_x_const` / `_act_x_tuple2`
- (構造体パンチ系) `STRUCT_PUNCH_SYMBOL` 系

union variant `V` について:

- `V` (コンストラクタ) / `as_V` / `is_V` / `mod_V`

`GlobalValue.compiler_defined_method: bool`
([program.rs:220](../../src/ast/program.rs#L220)) で識別可能。

### ユーザー定義かどうかの判定材料
- `Span.input` (`SourceFile`) → `file_path: PathBuf` を持つ。
- LSP サーバーは `uri_to_latest_content`
  ([server.rs:163](../../src/commands/lsp/server.rs#L163) 付近) で開いている
  ファイルを把握。
- ただし「ユーザーのプロジェクト配下の全ファイル」と「開いているファイル」は
  別概念。`fixproj.toml` 起点の **プロジェクトルート** を
  `get_current_dir` (LSP 起動時の cwd) が握っている
  ([util.rs:get_current_dir](../../src/commands/lsp/util.rs))。
  依存パッケージはそれ以外の場所 (`.fix/registry` 等) に展開される。

## 2. ゴールと範囲

**ゴール**: 以下のいずれかの上にカーソルを置いて `Rename Symbol` を実行
すると、プロジェクト内の宣言と全使用箇所 (import 文・自動実装メソッド・
それらの呼び出し箇所を含む) が一括で書き換わる:

- ローカル変数 (let / lambda 引数 / pattern)
- グローバル値 (関数・定数)
- 型 / 型エイリアス
- トレイト / トレイトエイリアス
- 関連型
- 構造体フィールド (連動して `@x` / `set_x` / `mod_x` / `act_x` も rename)
- union variant (連動して `V` / `as_V` / `is_V` / `mod_V` も rename)

**範囲外**:
- module 名 rename (ファイル名・全 import を絡めるため別タスク)
- パッケージ越境 rename (依存先のソースは変更不可)
- import 文での名前空間部分 (`Foo::bar` の `Foo`) の rename — module rename と
  同じ理由

## 3. ユーザー論点への回答

### 3.1 import を refs に含めるか
**Yes、含める方向で実装する**。理由:
- rename と refs で「ある global 名の使用箇所」の集合がズレるのは
  ユーザー体験として混乱を招く (refs で出ない箇所が rename されると驚く、
  逆もしかり)。
- import 文の見え方を変えるのが嫌なら、refs 側に `include_imports` フラグを
  追加して既定 `false` にする選択肢もあるが、現状そういうフラグは無いし、
  LSP の慣例的にも import 文は references に含めるのが標準的 (rust-analyzer
  も TypeScript も含める)。**フラグ無しで含める** を初期案とし、もし
  既存テスト `test_references` の期待値が壊れるなら **そのテストを更新する**。

具体的に refs を拡張するのは: `find_global_value_references` /
`find_type_references` / `find_trait_references` / `find_assoc_type_references`
の 4 つ。`program.modules` (各 `ModuleInfo` に `imports: Vec<ImportStatement>`
等を持っているはず — 要確認) を辿って `ImportTreeNode` を再帰的に走り、
リーフ名と target を比較して `Span` を集める。

### 3.2 非ユーザーシンボルの拒否
**Yes、拒否**。判定は **`fixproj.toml` の files セクションに入っている
ソースに定義されているか** で行う。`get_current_dir()` のパス前置比較は
依存を cwd 配下にベンダリングしたケース等で誤判定するので避ける。

実装:
- 既存の前例 [edit_explict_import.rs:52-71](../../src/edit/edit_explict_import.rs#L52-L71) —
  `get_user_source_files(&proj_file)` が
  `proj_file.get_files(BuildConfigType::Test)` (= 依存を除いた
  プロジェクト自身のソース) を **絶対パス** に正規化して返す。
  これを LSP モジュールに引き上げて再利用 (`pub` に変えるか、
  `lsp/util.rs` に薄いラッパを置く)。
- LSP サーバーは起動時に `ProjectFile::read_root_file()` を一度だけ呼んで
  `Set<PathBuf>` をキャッシュする (`Configuration` を作るときに既に
  読んでいるはずなので、その経路を流用)。`fixproj.toml` 自体が編集された
  場合は再起動を要求する (現状の Fix LSP の他の挙動と整合)。
- 対象 EndNode から取れる **宣言箇所 span** (global は
  `program.global_values[name].decl_src`、型は `TypeDefn.name_src`、
  トレイトは `TraitInfo.name_src` 等) の `input.file_path` を
  `to_absolute_path` で正規化し、上記キャッシュに含まれているかで判定。
- 含まれていなければ "Cannot rename symbol defined outside this project
  (e.g., Std or a dependency)" を `ResponseError` (LSP の
  `code: -32603` あたり) で返す。

ローカル名は `decl_src` を持たないが、定義位置のスパンはスコープ走査器
(`find_local_binding` / `find_local_occurrences`) が返してくれるので
同じく `file_path` で判定する。

副次効果: refs / rename どちらも、依存先のソースを「副作用で勝手に開いて
書き換える」ことが構造的に発生しなくなる。

### 3.3 自動実装メソッドの連動
**フィールド/variant を rename するときは自動実装メソッドも一緒に rename する**。
両者の対応関係:

- フィールド `x` ↔ `<TyConNS>::@x` / `set_x` / `mod_x` / `act_x`
  (内部用 `_act_x_identity` 等は user-callable ではないので除外。
  再 elaboration で再生成される)
- variant `V` ↔ `<TyConNS>::V` (コンストラクタ) / `as_V` / `is_V` / `mod_V`

**現状確認**: フィールド/variant に対する references は **未実装**
(`EndNode` にそもそも該当ヴァリアントが無い)。本タスクで新設する。

**設計の核**: **フィールド/variant 共通の collector
`find_member_occurrences(program, tc, name)` を新設し、refs と rename の
両者が同じ collector を使う**。フィールドと variant で API は同じ
(`(program, tc, name)`); 内部で `program.type_defns` から `tc` を引いて
Struct/Union のどちらかに分岐する。collector は span と「ソース上で
そのスパンが持っていた prefix」を組にした構造体を返す。

```rust
pub struct MemberOccurrence {
    pub span: Span,
    // ソース上でこの span が持つ接頭辞。rename 時の new_text 構築に使う。
    // "" (= bare な field/variant 名そのもの。
    //      宣言・MakeStruct・Pattern::Struct/Union・variant constructor 呼び出し)
    // フィールド: "@" / "set_" / "mod_" / "act_"
    // variant:    "as_" / "is_" / "mod_"
    // "^"  (= index syntax `[^field]` の `^field` スパン。フィールド限定)
    pub prefix: &'static str,
}
```

- **refs ハンドラ**: `occ.span` だけを取り出して `Vec<Span>` →
  `Vec<Location>` に変換。ハイライトは **トークン全体**
  (`@x` 全体・`set_x` 全体・`^x` 全体・`x` 単体など)。
- **rename ハンドラ**: `(occ.span, format!("{}{}", occ.prefix, new_name))`
  で TextEdit を直接組む。

理由 (§3.3-末尾 の index syntax との関係): `[^x]` 由来の `Var` と
`act_x` リテラルの `Var` を区別するには Var ノードを訪問するときに
`struct_act_func_in_index_syntax` 旗を見る必要があり、これは
`find_global_value_references` (現状 `Vec<Span>` を返す) では情報が
落ちる。collector を一段抽象化することで、refs はこの情報を捨てるだけ、
rename は使う、と単純に書き分けられる。

実装方針:
1. **EndNode 拡張** — `EndNode::Field(TyCon, Name)` と
   `EndNode::Variant(TyCon, Name)` を追加。`TypeDefn` の構造体/union を
   `find_node_at` で走るとき、各 `Field.source` がカーソルを含めば返す。
   さらに **使用箇所側** からも降りてこられるようにするのが望ましい
   (`MakeStruct(tc, fields)` の field-name 部分、Struct パターンの field-name
   部分)。これらのスパンが現状取れているかは要追加調査 — 取れていなければ
   そのスパンを保持する小改修が必要 (Phase B3 で扱う)。
2. **自動実装メソッド名の機械的列挙** — フィールド/variant 名から
   `Vec<(prefix: &'static str, FullName)>` を生成する内部ヘルパー
   `auto_methods_for(tc, name)` を `program.rs` か `lsp/util.rs` に追加。
   `tc` を `program.type_defns` で引いて Struct/Union を判定し、
   §1 の [program.rs:1866-2024](../../src/ast/program.rs#L1866-L2024) と
   完全に同じ規則で命名を組み立てる。prefix は後段の rename で
   そのまま再利用する。
3. **共通 collector** — `references.rs` に
   `find_member_occurrences(program, tc, name) -> Vec<MemberOccurrence>` を
   新設。`tc` を `program.type_defns` で引いて Struct/Union を判定し、
   それに応じた prefix リストと宣言/使用サイトに分岐。中身:
   - フィールド/variant 名そのものの宣言 span を `prefix=""` で push。
   - Phase B3 で取れるならフィールド名の MakeStruct / Pattern::Struct
     使用 span を `prefix=""` で push。
   - 各 auto-method について、`program.global_values` 全体を走査して
     `Expr::Var` を訪問。`v.name == auto_method_full_name` のとき:
     * `act_*` かつ `struct_act_func_in_index_syntax == true`:
       `prefix="^"` で push (span は Var.source = `^field` スパン)。
     * それ以外: その auto-method の literal prefix (`@`/`set_`/...) で
       push。
   - import 文 (`ImportTreeNode::Symbol(name, span)` 等) を走査し、
     bare 名 + 各 auto-method 名にマッチしたものをそれぞれの prefix で
     push。
4. **refs ハンドラ**: `find_member_occurrences` を呼び、`occ.span` だけ
   抽出して `Vec<Span>` に変換、既存の `spans_to_locations` に流す。
5. **rename ハンドラ**: 同じ `find_member_occurrences` を呼び、
   `(occ.span, format!("{}{}", occ.prefix, new_name))` で TextEdit を
   組む。
6. **rename の発動箇所はフィールド/variant の bare 名のみ** — `@x`/
   `set_x`/`mod_x`/`act_x`/`as_V`/`is_V`/`mod_V` の上にカーソルがある
   状態で rename を起動した場合、ユーザーが新名として `@y` と打つべきか
   `y` と打つべきか曖昧になるので、**rename を拒否** する。
   `index syntax `[^x]` も同様 (`act_x` の Var に脱糖されているため)。
   - 拒否の判定: `EndNode::Expr(var, _)` で
     `program.global_values[var.name].compiler_defined_method == true`
     なら ResponseError を返す。メッセージ: "Cannot rename an
     auto-generated accessor. Rename the field/variant declaration
     instead."
   - rename を起動できる位置:
     - フィールド/variant の **宣言箇所** (`EndNode::Field` / `Variant`)
     - (Phase B3 後) `MakeStruct` の field-name や `Pattern::Struct` /
       `Pattern::Union` の bare 名使用箇所 (これらも `EndNode::Field` /
       `Variant` を返すように `find_node_at` を整備)
7. **refs はこの制限を共有しない** — `@x` の上で refs を実行すれば
   ふつうのグローバル関数 `@x` の参照集計として動く (= `@x` リテラル使用
   箇所と、`@x` を import している import 文を返す)。ユーザーが
   フィールドの全関連箇所を見たいときは、フィールド宣言の上で refs を
   起動する。
8. **prepareRename の出力** — placeholder はフィールド/variant 名
   (`"x"` / `"V"`)。`@x` 等の上では prepareRename は `null` を返し、
   IDE 側に rename 不可と伝える。

#### 3.3-末尾 index syntax の存在 (重要)
Fix には **`obj.[^field]` という index syntax** がある
([parser.rs:1557-1672](../../src/parse/parser.rs#L1557-L1672))。これは parse 時に
`act_field` 関数の呼び出しに脱糖され、生成される `Var` には:

- `name = <TyConNS>::act_field`
- `source` = 元ソースの **`^field`** のスパン (caret 込み)
- `struct_act_func_in_index_syntax = true` 旗
  ([expr.rs:38-39](../../src/ast/expr.rs#L38-L39))

つまり、フィールド `x` を rename するとき `act_x` を target にすると、
`find_global_value_references` は **2 種類の出自のスパン** を返す:

| ソース上の文字列 | スパン内容 | `index_syntax` 旗 | 必要な置換 |
|---|---|---|---|
| `obj.act_x` (リテラル) | `act_x` | `false` | `act_y` |
| `obj.[^x]` (シンタックスシュガー) | `^x` | `true` | `^y` |

`Vec<Span>` を平らに受け取って prefix を `act_` 固定で挿げ替えると、
index syntax 側が壊れる (`[^x]` が `[act_y]` になる)。これが
「refs の内部表現を変えるのではなく rename 専用 collector を持つ」
動機。collector は `Var` を直接訪問するので旗を見られる。

なお `getter` (`@x`)・`setter` (`set_x`)・`mod_x` には対応する index
syntax は無いので、`act_<field>` だけが分岐対象。`tuple_accessor`
(`[^0]`) は数字フィールドなので rename 対象外。

→ ref 側 (LSP refs レスポンス) はこのままでも問題ない。`act_x` の refs
として `act_x` 全体と `^x` 全体の両方がハイライトされるのは「どちらも
`act_x` への参照」という意味で正しい。

### 3.4 ローカル変数の rename
refs が既に実装済み ([util.rs](../../src/commands/lsp/util.rs) の
`find_local_occurrences` / `find_enclosing_binder` /
`collect_uses_of_binding`) なので、rename はその上に薄く乗るだけ。

- **EndNode 解決**: `EndNode::Expr(var, _)` または
  `EndNode::Pattern(var, _)` で `var.name.is_local()` がエントリポイント。
- **occurrence 集め**: `find_local_occurrences(program, &pos, &var.name)`
  が `LocalOccurrences { definition: Span, uses: Vec<Span> }` を返すので、
  これを使う。すべての span に対して `new_text = new_name` で TextEdit。
- **新名チェック**: §4.8 のとおり `Rule::name` で 1 回パースするだけ。
  shadowing は合法なので衝突チェックはしない。
- **ユーザー定義性チェック (§3.2)**: ローカルの `definition` span の
  `file_path` も §3.2 のキャッシュと比較。実用上は常に true (依存先の
  関数の中のローカルにはカーソルが立たない)、念のため。
- **prepareRename**: カーソル直下の `Var` のスパン Range と、
  placeholder = 元の bare name (`var.name.name`) を返す。

ハンドラの分岐としては `EndNode::Expr` / `Pattern` の最上段で
`is_local()` を見て分岐するだけ。グローバル/型/トレイト/フィールド/variant
の動線とは独立。

### 3.5 diagnostics 状態と rename の関係
**バッファが直近の elaboration の前提と一致しないなら拒否する** を採用。
理由:
- 古い AST に対して span を返しても、ユーザーのバッファでは別の文字位置に
  なっており、編集結果が壊れる。
- 「ちょっと型エラーが残っているけど rename したい」ユースケースは確かに
  あるが、Fix の elaboration は型エラーがあると AST 構築が途中で止まる
  ことが多く、AST が部分的だと refs が網羅できない。
- LSP 仕様的には rename を null で返すか、`ResponseError` を返すかが妥当。
  vscode は ResponseError をそのままモーダルに出してくれる。

判定材料:
- サーバーは `uri_to_latest_content` で「最新バッファ内容」と
  「最後にコンパイルに渡した内容」を別管理しているはず — 要確認。
  最後にコンパイルに渡した内容のハッシュを `program` 側に持たせて、
  リクエスト時に `uri_to_latest_content` のハッシュと比較する。
- ファイル単位の比較で十分。target シンボルが定義されているファイルおよび
  refs に含まれるファイルの **どれか一つでも mismatch** なら拒否。
- `last_diag` ([server.rs:160 付近](../../src/commands/lsp/server.rs)) が
  最新で、なおかつ **error 級が無い** ことも合わせて要件にする。

実装は §4.6 にまとめる。

### 3.6 型 rename と auto-namespace の連動 (struct / union 専用)
構造体 (`type Point = struct { ... }`) や union (`type Point = union {
... }`) を rename するとき、コンパイラが裏で持っている auto-namespace
`Foo::Point` (中に `@x` / `set_x` / `mod_x` / `act_x` 等が住む) も
`Foo::Q` に動かす必要がある。型エイリアスや auto-method を持たない
型はこの問題が無いので、本節は struct / union 限定。

**user-visible refs と内部 rename targets の分離**: ユーザーが
references を実行したときに `import` 文中の `Point::*` の `Point`
コンポーネントや、インライン qualified `Point::@x` の `Point` 部分まで
ヒットさせるのは過剰 (= ノイズ)。なので:

- `find_type_references(...)` (user-visible): 現行通り、bare `Point`
  トークンの spans のみ。
- `find_type_rename_targets(...)` (internal): bare `Point` + auto-namespace
  連動分まで含めた `Vec<(Span, String)>`。rename ハンドラ専用。

#### auto / user / mixed の分類 (再掲・整理)
import 文の各 `ImportTreeNode::NameSpace(name, children, span)` で、
**解決後の namespace path が型の auto-namespace `Foo::Point` と一致する**
ものを対象に:

- 各 child を再帰的に解決して `program.global_values[fullname]` を引く
- `compiler_defined_method == true` → **auto**
- それ以外 (= user 定義 helper / 型 / 内部 namespace) → **user**
- `Any(_)` (= `*`) → namespace 内に user 定義が一つでもあれば **mixed
  扱い**、無ければ **all auto 扱い**

children 構成ごとの動作:

| 構成 | 動作 |
|---|---|
| 全部 auto (`Any(*)` で全部 auto を含む場合も含む) | parent `name` span を `Q` に書き換え |
| 全部 user (or auto 0 個) | 何もしない |
| 混在 (mixed) | **M2: ImportStatement 全体を再生成して分割** |

#### M2: mixed import の分割再生成
mixed と判定された ImportStatement に対しては、その文全体の `source` 範囲
([import.rs:18](../../src/ast/import.rs#L18)) を以下の手順で書き換える:

1. 元 `ImportStatement` をクローン
2. 該当 `NameSpace("Point", children, _)` ノードを破壊し、children を
   auto / user で振り分け
3. 振り分け結果で新しい兄弟ノードを構築:
   - `NameSpace("Q", auto_children, None)`
   - `NameSpace("Point", user_children, None)` (空でなければ)
4. これを `ImportStatement.items` に差し替えて `stringify()`
   ([import.rs:123-176](../../src/ast/import.rs#L123-L176)) で文字列化
5. `(import_stmt.source, new_text)` で TextEdit 1 つ

副作用: format / コメントが失われる。**これはこのケースのトレードオフとして
許容する**。実害は (a) mixed import 自体が稀、(b) 失われるのは 1 文だけ、
(c) 編集後ユーザーが整形できる、の 3 点で薄い。

#### インライン qualified 参照 (2C)
`Point::@x` / `[^Point::x]` / `Foo::Point::@x` などは Var ノードに
落ちている。Var.name が `Foo::Point::<x>` で `compiler_defined_method
== true` なら、Var.source の text を再パースして `Point` 部分の
sub-span を割り出す:

1. Var.source の text を取得 (Span.input.string() 経由で)
2. 先頭が `^` (= index syntax) なら 1 char skip
3. `FixParser::parse(Rule::fullname, text)` で再パース
4. 結果の Pair tree を walk し `namespace_item` のうち名前 `Point` の
   ものの relative span を取得
5. Var.source.start に offset を加えて絶対 Span に変換、`(span, "Q")`
   を emit

**user-defined への qualified 参照 (`Foo::Point::my_helper`) は触らない**
(3B 準拠): `program.global_values[Foo::Point::my_helper].compiler_defined_method
== false` なので step 1 のフィルタで自動的に除外される。

## 4. 設計詳細

### 4.1 ファイル構成
新規ファイル: `src/commands/lsp/rename.rs`。`mod.rs` に `pub mod rename;`
を追加。LSP 既存の構造に合わせる。

### 4.2 server.rs の変更
- `rename_provider` を `Some(OneOf::Right(RenameOptions { prepare_provider:
  Some(true), .. }))` に。
- dispatch に `textDocument/rename` と `textDocument/prepareRename` を追加し、
  それぞれ `rename::handle_rename(..)` / `rename::handle_prepare_rename(..)`
  を呼ぶ。
- 引数は他の handler と同じく `(id, params, program, uri_to_content)`。

### 4.3 EndNode 拡張
[program.rs:2403](../../src/ast/program.rs#L2403):

```rust
pub enum EndNode {
    // ... 既存 ...
    Field(TyCon, Name),
    Variant(TyCon, Name),
}
```

- `find_node_at` から呼ばれる `TypeDefn::find_node_at` (要新設、または
  `Program::find_node_at` 内で型定義を走らせる) で、各
  `TypeDeclValue::{Struct, Union}.fields` の `Field.source` を見て
  カーソル位置にあれば返す。
- 使用箇所側からも返したい:
  - `Expr::MakeStruct(tc, fields)` の **field 名** スパン — 現状
    `(Name, Arc<ExprNode>)` のタプルで持っているが、name 側のスパンを
    保持しているか? — 要追加調査 (`src/ast/expr.rs` の `MakeStruct`
    バリアント定義を見る)。無ければ別途の追加が必要。
  - `Pattern::Struct(tc, field_pats)` の **field 名** スパン — 同上。
  - 上記 2 つのスパンが取れない場合、本フェーズでは
    「使用箇所からのフィールド rename 起動」を **対象外** とし、宣言箇所と
    自動実装メソッド経由の起動でカバーする。これは段階的実装として OK。
- references / hover の switch 文を `Field` / `Variant` を網羅するように更新。
  references は §4.5 で実装。hover は当面 "" を返すで構わない (TODO コメント)。

### 4.4 references の拡張: import を含める
新規ヘルパー:

```rust
fn collect_import_refs_value(
    program: &Program,
    target: &FullName,
    refs: &mut Vec<Span>,
);
fn collect_import_refs_type_or_trait(
    program: &Program,
    target_namespace_path: &[Name],   // e.g. ["A","B","Foo"]
    refs: &mut Vec<Span>,
);
```

`program.modules`(または `mod_to_import_stmts`) を走査し、各
`ImportStatement.items` / `hiding` の各 `ImportTreeNode` を再帰。リーフが
`Symbol(name, Some(span))` で `[stmt.module] ++ traversed_namespace ++ [name]`
が target 完全一致 → push。`TypeOrTrait` も同様。値 (`is_local()` 系) と
型/トレイト (`TypeOrTrait`) の判定を間違えないこと。

これを `find_global_value_references` / `find_type_references` /
`find_trait_references` / `find_assoc_type_references` の最後に呼ぶ。

### 4.5 references の拡張: Field / Variant
`references.rs` に **フィールド/variant 共通の occurrence collector** を
新設 (§3.3 の core)。シグネチャ:

```rust
pub struct MemberOccurrence {
    pub span: Span,
    pub prefix: &'static str,
}

pub fn find_member_occurrences(
    program: &Program,
    tc: &TyCon,
    name: &Name,
) -> Vec<MemberOccurrence>;
```

`tc` を `program.type_defns` で引いて Struct か Union かに分岐し、
それに応じた prefix リストと宣言/使用サイトを使う。

実装の概要:
- 宣言スパン (`TypeDeclValue::Struct.fields[i].source` または
  `TypeDeclValue::Union.fields[i].source`) を `prefix=""` で push。
- (Phase B3 後) Struct なら `MakeStruct` / `Pattern::Struct` の field-name
  使用 span、Union なら `Pattern::Union` の variant-name 使用 span を
  `prefix=""` で push。
- `auto_methods_for(tc, name)` (§以下) から得た
  `Vec<(prefix: &'static str, FullName)>` を回す。各 auto-method について
  `program.global_values` を走査し `Expr::Var` を訪問:
    - `act_*` (Struct のみ) かつ `var.struct_act_func_in_index_syntax == true`:
      `prefix="^"` で push (Var.source は `^field` を覆う)。
    - それ以外: その auto-method の literal prefix で push
      (Var.source は `act_x`/`as_V` 等の全体を覆う)。
- import 文 (`ImportTreeNode::Symbol(name, span)`) を走査し、bare 名 +
  各 auto-method 名にマッチしたものをそれぞれの prefix で push。

`auto_methods_for(tc, name)` (内部ヘルパー):
- `tc` の TypeDeclValue を見て:
  - Struct の field `x` → `[("@", @x), ("set_", set_x), ("mod_", mod_x),
    ("act_", act_x)]` を `tc` の namespace 配下に組み立てて返す。
  - Union の variant `V` → `[("", V), ("as_", as_V), ("is_", is_V),
    ("mod_", mod_V)]`。`V` 自体 (constructor) も `prefix=""` で並列に
    扱える点に注意。
- 実在性チェック: `program.global_values.contains_key(&name)` で
  フィルタ (条件付き生成されるエントリを除外するため)。

#### refs ハンドラ側
`find_all_references` の match に分岐を追加:

```rust
EndNode::Field(tc, name) | EndNode::Variant(tc, name) => {
    find_member_occurrences(program, tc, name)
        .into_iter().map(|o| o.span).collect()
}
```

(`include_declaration` フラグの扱いは既存の他 EndNode と同様、
collector 側に渡して条件 push にする。)

### 4.6 ステイル状態検知 (論点 4)
方針: **strict (= 全プロジェクトファイル一致)**。

- `Program` に `pub source_hashes: Map<PathBuf, String>` を追加。
  diagnostics スレッドが elaboration を完了した時点の各ソースファイル
  内容のハッシュ (SHA-256 など) を入れる。
- rename request が来たら以下を全部チェックし、どれか満たさなければ
  `ResponseError` を返す。メッセージは
  > "Cannot rename: source is out of sync with the last successful build.
  >  Save the file and wait for diagnostics to refresh."
  - **last_diag が error 無しで完了している** こと
    (Fix にはまだ warning カテゴリが無いので、エラー有無だけで判定して OK)。
  - **`fixproj.toml` の files セクションに含まれる全ソースファイル**
    について、`Program.source_hashes[path]` と現在の
    `uri_to_latest_content` (LSP が握っている最新バッファ) または
    `uri_to_latest_content` に無ければディスク上のファイル内容のハッシュが
    一致すること。
- 「全ファイル一致」の意図: rename は build artifact (= AST + spans) を
  使って TextEdit を組むので、build artifact がどこかのファイルとズレて
  いる時点でその TextEdit は信用できない。target ファイルだけ一致して
  いても、他ファイルの spans を使って TextEdit を作っているなら同じ
  リスクが残る。
- 将来 Fix に warning ができたら、warning カテゴリは許容するように
  ゆるめる。

### 4.7 rename ハンドラの動線

```
handle_rename(params: RenameParams, program, uri_to_content) -> WorkspaceEdit | Err
  1. ステイル検査 (§4.6) — 最初に実行する
       - program.source_hashes と uri_to_latest_content (またはディスク
         内容) を全プロジェクトファイルで比較。一つでも不一致なら即
         ResponseError。
       - last_diag に error があれば即 ResponseError。
       - 理由: 後続の resolve_source_pos / find_node_at は program 側の
         Span 座標を使うので、バッファがズレているとカーソル位置が
         無関係なノードに「当たって」しまい、後段すべての判断が崩れる。
         ハッシュ比較は安いので最初に弾く。
  2. resolve_source_pos -> SourcePos
  3. program.find_node_at(pos) -> EndNode
  4. EndNode を rename 対象に正規化:
       - EndNode::Expr(var) で var.name が compiler_defined_method なら
         「auto-generated accessor は rename 不可」で ResponseError
         (§3.3 step 6)。`[^x]` 由来の Var もここで弾かれる。
       - その他の正規化 (例えば import 文中のシンボルから本体への解決)
         が必要ならここで。
  5. ユーザー定義性チェック (§3.2)
  6. 新名チェック (§4.8): EndNode 種別に応じた pest Rule
     (`name` / `type_field_name` / `capital_name`) で
     `FixParser::parse(rule, new_name)` を実行し、入力全体が消費される
     ことを確認するだけ。namespace 衝突は事前検査せず、コンパイラに
     任せる。
  7. EndNode 種別ごとに `(Span, new_text)` の Vec を組み立てる:
       - 値/トレイト/関連型: 既存の `find_*_references` を呼んで
         `Vec<Span>` を取り、各 span に `new_text = new_name` で TextEdit。
       - 型: struct / union の場合は `find_type_rename_targets` (§3.6) を
         呼んで `Vec<(Span, String)>` を取得。それ以外 (型エイリアス・
         auto-method を持たない型) は `find_type_references` で OK。
       - フィールド/variant: `find_member_occurrences` (§4.5) を呼び、
         各 occurrence について
         `(occ.span, format!("{}{}", occ.prefix, new_name))` で TextEdit。
       - import 文: 既存の refs 拡張 (Phase A1) で `find_*_references` の
         戻り値に既に含まれているので追加処理不要。フィールド/variant の
         場合は collector が import 文も込みで返す。
  8. URI ごとに groupBy して WorkspaceEdit.changes を組み立て、返す
```

`handle_prepare_rename` は `(1)`〜`(5)` まで実行し、成功したら
カーソル直下のトークン Range と placeholder (元の bare name) を返す。
ステイル検査もここで通すことで、最初から rename UI の起動を抑止できる。

ポイント: refs と rename で **同じ collector** を使う。差分は「prefix
情報を捨てるか・使うか」だけ。これで index syntax (`[^x]`) と
リテラル `act_x` の区別が一箇所に閉じる。

### 4.8 新名のバリデーション
**構文チェックは独立に書かない**。grammar.pest と既存の parser.rs を
そのまま使う。具体的には:

- pest grammar に既に必要なルールが揃っている
  ([grammer.pest](../../src/parse/grammer.pest)):
  - `Rule::name` (line 24): 値・グローバル関数・ローカル名。
    `!keywords ~ (name_head ~ name_char*)` で keyword 除外も込み。
    `name_head` は小文字英字 / `_` / `@`。
  - `Rule::type_field_name` (line 243): 構造体フィールド・union variant
    名。`!keywords ~ (name_head ~ !"@") ~ name_char*` で **`@` 始まり
    を明示的に除外**。フィールド rename 用にちょうど良い。
  - `Rule::capital_name` (line 32): 型・型エイリアス・トレイト・
    トレイトエイリアス・関連型・モジュール・namespace。
    `ASCII_ALPHA_UPPER ~ (ASCII_ALPHA | ASCII_DIGIT)*`。
- `FixParser::parse(Rule::xxx, new_name)` を使い、戻りの Pair が
  入力全体を消費することを確認するだけで valid 判定が完了する。
  keyword・category (大文字 vs 小文字)・許可文字集合・`@` 始まり禁止
  はすべてルール内で表現済み。
- `FixParser` は現状 private (`struct FixParser;` at parser.rs:3) なので、
  parser.rs に薄い `pub fn validate_token_str(s: &str, rule: Rule) ->
  Result<(), Errors>` を追加して LSP 側から呼ぶ。これだけで親モジュール
  からの利用が可能になる。
- もし parser.rs 側で「pest で受理されているが追加で弾いている」
  バリデーションが将来出てきたら、その都度 parser.rs の関数を呼ぶか、
  parser.rs から関数を切り出す。現状は無いので不要。

EndNode 種別ごとのルール選択:

| EndNode 種別 | 使う Rule |
|---|---|
| `Expr/Pattern` (ローカル・グローバル値) | `Rule::name` |
| `ValueDecl` | `Rule::name` |
| `Type` / `TypeOrTrait` / `Trait` / `AssocType` | `Rule::capital_name` |
| `Field` (struct field) | `Rule::type_field_name` |
| `Variant` (union variant) | `Rule::type_field_name` |

**カテゴリ整合**: `Rule` を選んだ時点で「元と同じ category か」が自然に
保証されるので、追加チェックは不要 (例えば型を `y` にしようとすると
`Rule::capital_name` のパースが失敗する)。

**namespace 衝突の事前チェックは行わない**: rename した結果として
`@y`/`set_y`/... が既存名と衝突したり、グローバル名空間で衝突したりする
ケースは、rename 後の elaboration が普通に error を出して気付ける。
事前検査でユーザーをブロックするより、衝突したコードが出ること自体が
「その新名は良くなかった」というシグナルになる場面もあるので、エラーは
コンパイラに任せる。

**ローカル名の shadowing**: 元から合法なので、rename した結果として外側
の名前を新たに shadow することになっても許可する (= 何もチェックしない)。

### 4.9 LatestContent / Program のハッシュ機構
[server.rs:163](../../src/commands/lsp/server.rs#L163) 周辺の `LatestContent`
構造に `hash: String` を持たせ、didChange/didOpen のたびに更新する
(既に持っているかも — 要確認)。`Program` を作る elaboration スレッド側でも、
受け取った `Map<PathBuf, String>` を一緒に保管してリクエスト処理スレッドに
渡す。シンプルには `Program` に `pub source_hashes: Map<PathBuf, String>` を
追加するで足りる。

### 4.10 型 rename targets の実装 (§3.6 の実体)
struct / union 型の rename 用に、`rename.rs` に以下を追加:

```rust
// (Span, new_text) の Vec を返す。rename ハンドラ専用の internal API。
pub fn find_type_rename_targets(
    program: &Program,
    tc: &TyCon,
    new_name: &Name,
) -> Vec<(Span, String)>;
```

実装 3 段:

**(A) bare `Point` トークン**: `find_type_references(program, tc, true)`
の戻り値の各 span に `new_text = new_name` で push。

**(B) import-tree namespace コンポーネント**: 全 ImportStatement を走査。

```rust
fn walk_import_tree(
    stmt: &ImportStatement,
    namespace_path: &[Name],   // 親モジュール + 辿った namespace 列
    target_path: &[Name],      // 例: ["Foo", "Point"]
    program: &Program,
    out: &mut Vec<(Span, String)>,
);
```

各 `ImportTreeNode::NameSpace(name, children, span)` ノードで:
- `path = namespace_path ++ [name]` を構築
- `path == target_path` なら children を分類:
  - 各 `Symbol(child_name, _)` を resolve: fullname = `path ++ [child_name]`、
    `program.global_values[fullname].compiler_defined_method` で auto/user 判定
  - `Any(_)` (= `*`): `program.global_values` を走査して target_path 配下に
    user 定義があるかチェック。あれば mixed、無ければ all auto
  - `TypeOrTrait(_, _)` / `NameSpace(_, _, _)` (子レベルの namespace) は
    user 扱い (auto-namespace 内に出現することはほぼ無いが安全側)
  - 全部 auto → `(span, new_name.to_string())` を push
  - 全部 user → 何もしない
  - mixed → 当該 ImportStatement 全体を分割再生成 (下記 (B-mixed))
- `path != target_path` なら children に再帰 (`namespace_path` を更新して)

**(B-mixed) ImportStatement 分割再生成**:

```rust
fn rebuild_split_import(
    stmt: &ImportStatement,
    target_path: &[Name],
    new_type_name: &Name,
    auto_predicate: impl Fn(&FullName) -> bool,  // = compiler_defined_method
) -> ImportStatement;
```

- 元 stmt をクローン
- target ノードを auto / user に振り分けて 2 個の `NameSpace` ノードに分解
  - `NameSpace(new_type_name, auto_children, None)`
  - `NameSpace(old_type_name, user_children, None)` (user が空なら省略)
- `stmt.items` 中の元ノードを抜き、上記 2 個に置換
- 戻り値: 新 ImportStatement

呼び出し側で `(stmt.source.unwrap(), new_stmt.stringify())` を push。
**format / コメントは失われる** が、当該 1 文限定 (他の import 文や
ファイル全体は影響を受けない)。

**(C) インライン qualified Var 参照**: `program.global_values` の各 expr を
walk。`Expr::Var` のうち:
- `var.name == target_path ++ [<x>]` の形 (target の auto-namespace に
  住む function を参照している)
- かつ `program.global_values[var.name].compiler_defined_method == true`

を見つけたら、Var.source の text を取り出して再パースし `Point` の
sub-span を抽出 → `(sub_span, new_name.to_string())` を push。

```rust
fn extract_namespace_subspan(
    var_source: &Span,
    target_namespace_name: &Name,
) -> Option<Span>;
```

実装:
1. `var_source.input.string()` でソース全体を取得し、
   `[var_source.start_byte..var_source.end_byte]` を切り出す
2. 先頭が `^` なら 1 byte skip (= index syntax)
3. `FixParser::parse(Rule::fullname, &text)` で再パース
4. 結果の Pair を walk し `Rule::namespace_item` で `as_str() ==
   target_namespace_name` のものを発見
5. その Pair の relative span を `var_source.start_byte` に加算して
   絶対 Span を構築、返す

**(D) アグリゲート**: (A) (B) (B-mixed) (C) の結果を結合して返す。
重複 span は dedup (主に (A) と (C) で同じ位置を二重に push する可能性は
無いはずだが念のため)。

## 5. 実装フェーズ分割

大きいので段階的に進める。各段階で commit & 統合テスト追加。

**Phase A — 下準備**

A1. import 文を refs の集計対象に含める。
    `find_global_value_references` / `find_type_references` /
    `find_trait_references` / `find_assoc_type_references` の最後に
    `collect_import_refs_*` を呼ぶだけ。`test_references.*` の期待値は
    更新が必要 (import 行に増える)。

A2. ステイル検知用に `Program.source_hashes` を保持する経路を作る
    (まだ rename からは使わない)。

**Phase B — Field / Variant の references**

B1. `EndNode::Field` / `Variant` を追加し、`Program::find_node_at` から
    型定義 LHS のフィールド/variant 名で降りられるようにする。
    `references` は宣言箇所のみで OK。

B2. 自動実装メソッド名生成ヘルパー `auto_methods_for(tc, name)` を追加し、
    `find_member_occurrences` を実装。

B3. `MakeStruct` / `Pattern::Struct` / `Pattern::Union` の field/variant
    名スパンを保存するように parser/AST を拡張し、`find_node_at` と
    `find_member_occurrences` に含める。

**Phase C — rename 本体 (struct/union 型の auto-namespace 連動を除く)**

C1. `rename.rs` を新設、`handle_prepare_rename` をローカル + グローバル
    のみで実装。`server.rs` を配線。簡単なテストを通す。

C2. 型エイリアス / トレイト / トレイトエイリアス / 関連型 /
    フィールド / variant に拡張。型の rename は **bare トークンのみ**
    (struct/union 型でも auto-namespace 連動はまだしない)。

C3. ステイル検知 (§4.6)、新名バリデーション (§4.8)、非ユーザー定義拒否
    (§3.2)、auto-method 拒否 (§3.3 step 6) を組み込む。

**Phase D — struct/union 型 rename の auto-namespace 連動 (§3.6 / §4.10)**

D1. `find_type_rename_targets` の (A) (B) を実装 (bare token + import-tree
    の all-auto / all-user 分類)。**まだ mixed 対応せず**: mixed を検出
    したら警告ログを出して当該ノードをスキップ。テストで動作確認。

D2. (B-mixed) ImportStatement 分割再生成を実装。`stringify()` 経由の
    全文置換。テストで mixed import の正しい split を確認。

D3. (C) インライン qualified 参照の sub-span 抽出を実装。
    `extract_namespace_subspan` を `parser.rs` から `Rule::fullname` で
    再パースする形で実装。`[^Point::x]` (index syntax) のテストを通す。

C3. ステイル検知・新名衝突検知・category チェックを追加。エラーメッセージを
    LSP 仕様の `ResponseError` に乗せる。

C4. 非ユーザーシンボル拒否を追加。

各 Phase で以下のコマンドを通す:

```
cargo test --release test_references
cargo test --release test_rename     # Phase C 以降
```

## 6. テスト

`src/tests/test_lsp/test_rename.rs` 新設。`LspTestCtx::rename(file, line,
col, new_name)` および `LspTestCtx::prepare_rename(file, line, col)` を
追加し、`textDocument/rename` / `textDocument/prepareRename` を送って
結果を assert する。フィクスチャは `cases/rename_*` 以下の最小プロジェクト。

### 6.1 受理系: 各 EndNode 種別

- **rename_local** — `cases/rename_local`。let / lambda / match /
  構造体分解 / shadowing (内側だけ書き換わる)。`goto_local` のフィクスチャ
  を流用。
- **rename_gv** — `cases/rename_gv`。多ファイル + import 文での書き換え
  を必ず含む (`refs_gv` をベース)。起動箇所のバリアント:
  - 宣言箇所 (LHS) クリック
  - 同ファイル内の使用箇所クリック
  - 別ファイルの使用箇所クリック
  - import 文 (`import Foo::{bar};`) の `bar` 上クリック
- **rename_ty_alias** — `cases/rename_ty_alias`。型エイリアス。
  auto-namespace 連動は無いので bare token のみ。
- **rename_ty_struct** — `cases/rename_ty_struct`。**§3.6 の連動を実機
  検証する中心ケース**。サブテスト:
  - **bare**: type 宣言・MakeStruct・Pattern::Struct・型注釈・impl 内
    `for Point` がすべて `Q` に
  - **import_all_auto**: 別ファイルで `import Foo::{Point::act_x};` →
    `import Foo::{Q::act_x};` に書き換わる (B-allauto)
  - **import_all_user**: 別ファイルで `import Foo::{Point::my_helper};`
    (`namespace Point { my_helper : ... }` を user 定義) → **触らない**
    (B-alluser)
  - **import_mixed**: `import Foo::{Point::{act_x, my_helper}};` →
    `import Foo::{Q::{act_x}, Point::{my_helper}};` に分割再生成
    (B-mixed)
  - **import_wildcard_pure_auto**: `namespace Point { }` が空で
    `Point::*` が全部 auto → `Q::*` に (B-allauto)
  - **import_wildcard_mixed**: user 定義が同居している namespace で
    `Point::*` → 分割再生成 (B-mixed)
  - **inline_qualified**: 別ファイルの式中で `Point::@x p` →
    `Q::@x p` に書き換わる (C)
  - **inline_index_syntax**: `obj[^Point::x]` → `obj[^Q::x]` に
    書き換わる (C)
  - **inline_user_qualified_skip**: `Point::my_helper p` は **触らない**
    (3B 準拠、user 定義への qualified 参照)
- **rename_ty_union** — `cases/rename_ty_union`。union 型版。
  rename_ty_struct と同じ subtest 群を `as_V` / `is_V` / `mod_V` /
  constructor `V` で構成。
- **rename_tr** — `cases/rename_tr`。トレイト名 + すべての `impl X for
  ...` ブロックの `X` + 全 trait constraint。
- **rename_tr_alias** — `cases/rename_tr_alias`。`trait Eq = ...` の
  左辺名。
- **rename_at** — `cases/rename_at`。関連型 — トレイト本体の宣言・
  実装ブロックの `Item iter = ...`・型シグネチャ中の `Item iter`。
- **rename_field** — `cases/rename_field`。連動して書き換わるべきもの:
  - `@x` / `set_x` / `mod_x` / `act_x` のリテラル使用箇所
  - `Point { x: ..., y: ... }` の MakeStruct 中 `x` (Phase B3)
  - `let Point { x, y } = p` の Struct パターン中 `x` (Phase B3)
  - import 文の `import Foo::{Point::act_x};` の `act_x` 部分
  - **別モジュールから `[^x]` (index syntax) で使われている箇所** —
    `^x` → `^y` に書き換わる (§3.3-末尾 を実機検証する唯一のケース)
- **rename_variant** — `cases/rename_variant`。連動して書き換わるべき
  もの:
  - constructor 呼び出し (`Cons(...)`)
  - `as_Cons` / `is_Cons` / `mod_Cons`
  - `Pattern::Union` 中の `Cons` (Phase B3)
  - import 文の variant 系名

### 6.2 拒否系

- **reject_external_global** — `Std::IO::println` を rename しようと
  して ResponseError が返る。
- **reject_external_type** — `Std::Array` を rename しようとして
  ResponseError。
- **reject_auto_method** — `@x` / `set_x` / `mod_x` / `act_x` / `as_V` /
  `is_V` / `mod_V` の上で rename を起動 → ResponseError "Cannot rename
  an auto-generated accessor."。**§3.3 step 6 の動作を実機検証する
  唯一のケース**。
- **reject_index_syntax** — `[^x]` の `^x` の上で rename を起動 →
  ResponseError (内部的には `act_x` の Var 経由)。
- **reject_stale** — didChange でバッファをいじって didSave せず即
  rename → ResponseError。target ファイル自身が dirty なケースと、
  別ファイルだけが dirty なケース両方。
- **reject_diagnostics_error** — コンパイルエラーが残っている状態で
  rename → ResponseError。stale とは独立。
- **reject_invalid_name** — pest grammar で弾かれるケース:
  - category 違反 (型を小文字 / 値を大文字に rename)
  - 空文字 / 空白入り
  - keyword (例: `let` / `match`)
  - フィールド/variant に `@y` を打つ (`Rule::type_field_name` の
    `!"@"` 制約)
  - namespace 衝突は **テスト対象外** (= コンパイラに任せる方針)

### 6.3 prepareRename

`test_prepare_rename.rs` (または `test_rename.rs` 内の別 mod):

- **prepare_rename_field** — フィールド宣言上 → Range と placeholder
  `"x"` が返る。
- **prepare_rename_local** — ローカル変数上 → Range と placeholder。
- **prepare_rename_global** — グローバル関数上 (宣言・使用)。
- **prepare_rename_reject_auto_method** — `@x` 上 → null (rename UI
  抑止)。
- **prepare_rename_reject_external** — Std の関数上 → null。
- **prepare_rename_reject_stale** — ステイル状態 → null。

### 6.4 WorkspaceEdit の構造

ここまでの個別ケース内で以下を assert する:

- `WorkspaceEdit.changes` が **URI ごとに groupBy** されている
  (rename_gv で多ファイル、rename_local で 1 ファイルを別々に確認)。
- 依存先ファイルの URI は決して含まれない (rename_gv で `Std::*` を
  使ったコードを書き、edit が user files のみであることを assert)。
- 同一ファイル内の複数編集が **どれも互いに重ならない** (Range の
  分離性)。
- mixed import の B-mixed ケースで、ImportStatement 全体を覆う
  TextEdit が出ていること (= 部分置換の重ね掛けではないこと)。
- 元名 == 新名 (noop) のケース: 空 `WorkspaceEdit { changes: {} }` を
  返す (= `null` ではなく成功扱い)。

### 6.5 既存テスト (refs) の更新 (Phase A1 で発生)

- `test_references.rs` の各ケース (`refs_gv` / `refs_ty` / `refs_tr` /
  `refs_at`) で、**import 文中の参照が結果に含まれる** よう期待値を更新。
- `hiding` 句中の名前もヒットすることを確認するケースを 1 件追加。

### 6.6 実行コマンド

```
cargo test --release test_references     # Phase A1 後
cargo test --release test_rename         # Phase C1 以降、段階的に増える
cargo test --release test_prepare_rename # Phase C1 後
```

CLAUDE.md の指針に従い `--release` 前提。

## 7. 確定した設計判断 (旧・開放残し論点)

すべて確定済み。実装着手可能。

- **論点 1 (refs に import を含める)**: 確定 — `test_references` の期待値
  を import 込みに更新。
- **論点 2 (非ユーザーシンボル拒否)**: 確定 — `fixproj.toml` の files で
  判定 (§3.2)。
- **論点 3 (auto-method 連動)**: 確定 — Field/Variant 共通 collector
  `find_member_occurrences` で refs / rename を共有 (§3.3)。
- **論点 4 (ステイル判定)**: 確定 — strict (= 全プロジェクトファイル
  ハッシュ一致 + last_diag エラー無し)。warning カテゴリができたら再訪
  (§4.6)。
- **EndNode::Field の使用箇所スパン保存** (Phase B3): 確定 — 本タスクに
  入れる。
- **新名バリデーション**: 確定 — pest grammar の Rule で 1 回パースする
  だけ。namespace 衝突は事前検査せずコンパイラに任せる (§4.8)。
- **rename 起動箇所**: 確定 — auto-method の上 (`@x` 等) では拒否、
  フィールド/variant の bare 名のみ許可 (§3.3 step 6)。
- **struct/union 型 rename と auto-namespace 連動**: 確定 — 1A + 2C +
  3B + M2。bare token + import-tree namespace + インライン qualified
  + mixed import 分割再生成、すべて対応 (§3.6 / §4.10)。実装は Phase D。
- **`mod_x` の "mod" 衝突**: 設計上自然に守れる (`tc` をキーに含める
  `auto_methods_for(tc, name)` ヘルパーで命名生成するため、フィールドと
  variant の `mod_x` は別 namespace として扱われる)。

## 8. 完了条件

- `cargo test --release test_references` が通る (import 込みの新期待値)。
- `cargo test --release test_rename` の全ケースが通る。
- 手動で VS Code の Fix LSP で、各 rename ケースが期待通り編集を提示し、
  外部パッケージシンボルでは "Cannot rename" のメッセージが出ること。
- `cargo clippy` / `cargo build --release` が通ること。
