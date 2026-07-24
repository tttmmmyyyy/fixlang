# 計画: ローカル名に対する LSP `textDocument/definition` 対応

## 背景

このワークツリーは rename symbol を追加するために切ったが、その前提として
**ローカル名 (let 束縛・ラムダ引数・パターン束縛) の jump-to-definition** が
未実装であることが判明した。rename でも「ある利用箇所からその束縛箇所および
スコープを求める」処理が必要になるため、先にこちらを片付ける。

現在の振る舞い — `src/commands/lsp/goto_definition.rs:46-49`:

```rust
let full_name = &var_name;
if full_name.is_local() {
    def_src = None;            // ← 明示的にここで諦めている
}
```

グローバルは elaboration 中に `program.global_values` の `decl_src` /
`defn_src` に定義位置が記録されているが、ローカルにはそのような中央集約の
束縛マップが無い。AST を走査して囲みスコープの束縛子を探す必要がある。

ゴール: ローカル変数の使用箇所にカーソルを置いて定義へジャンプすると、
最内の囲みスコープの束縛子 (let のパターン、またはラムダ引数のパターン) に
飛ぶこと。

## 調査で分かった重要事項

- **ラムダはパース時に let に脱糖される**
  (`src/parse/parser.rs:1899-1925`)。`|pat| body` は
  `\#param. let pat = #param in body` になる。したがってユーザー可視な
  ローカル束縛はすべて `let` の `Pattern` 上にあり、
  `PatternNode.info.source` に束縛位置のスパンが入っている。`Var` 単位の
  パラメータスパンは不要。
- `ExprNode::find_node_at` (`src/ast/expr.rs:962`) と
  `PatternNode::find_node_at_pos` (`src/ast/pattern.rs:219`) が既に
  カーソル直下の最小ノードまで降りて
  `EndNode::Expr(var, _)` / `EndNode::Pattern(var, _)` を返してくれるので、
  ローカル名の `FullName` は取得済み。
- `Program::find_node_at` (`src/ast/program.rs:2284`) が
  `program.global_values` 全てを舐める形になっているので、同様に
  カーソルを含む `SymbolExpr` を見つける処理を再利用できる。
- `Span` → LSP の変換は既存: `span_to_location`
  (`src/commands/lsp/util.rs:171`)。
- Fix の `let` は **非再帰**。`let pat = bound in val` のパターン束縛は
  `val` 側のみで可視で、`bound` では不可視
  (`src/ast/expr.rs:1095-1098` の `calc_free_vars` のコメント参照)。
- シャドウイングは合法。内側の束縛子が勝つ挙動は「スコープスタックを
  トップから検索して最初にヒットしたものを返す」で自然に満たされる。

## 方針

「`Program` ・ カーソル `SourcePos` ・ 対象ローカル `FullName` を与えると、
その名前を束縛しかつカーソル位置をスコープに含む **最内の束縛子のスパン**
を返す」スコープスタック走査器を追加する。

### 変更対象ファイル

1. **`src/ast/pattern.rs`** — パターン内の全 `Pattern::Var` について
   `(名前, スパン)` を収集するヘルパーを追加:

   ```rust
   /// このパターン木内の全ての Pattern::Var について (名前, スパン) を集める。
   /// スパンは最も内側の var-pattern の PatternNode.info.source を使う。
   pub fn var_bindings(self: &Arc<PatternNode>) -> Vec<(FullName, Span)>
   ```

   `Pattern::Struct` / `Pattern::Union` は再帰。`info.source` が `Some` の
   ときだけ emit。後述の走査器、および後続の rename 実装の双方から
   再利用する。

2. **`src/commands/lsp/util.rs`** — 以下を追加:

   ```rust
   pub(super) fn find_local_binding(
       program: &Program,
       pos: &SourcePos,
       target: &FullName,
   ) -> Option<Span>
   ```

   戦略:
   - `program.global_values` を走査し、各 `GlobalValue` について既存の
     `SymbolExpr::find_node_at(name, pos)` で早期に絞り込み。`Some` を
     返したものについてその `TypedExpr.expr` 木を走査する。
   - トレイトメンバ実装 (`SymbolExpr::Method`) は
     `TraitMemberImpl.expr.expr` を走査対象にする。
   - 走査器は `Vec<(FullName, Span)>` のスタックを持つ。
     `ExprNode.source` が `pos` を含むサブ式にのみ降下
     (`find_node_at` と同じ LSP 位置包含判定を使う)。降下毎に、現スタックの
     **トップから** 検索し `target` と一致する最初のエントリのスパンを返す。
   - 式種別ごとの処理:
     - `Expr::Let(pat, bound, val)` — まず `bound` を **push せず** に
       再帰 (非再帰 let なので)。その後 `pat.var_bindings()` を push し、
       `val` に再帰、pop。
     - `Expr::Match(cond, arms)` — `cond` に再帰。各 `(pat, val)` について
       `pat.var_bindings()` を push、`val` に再帰、pop。
     - `Expr::Lam(args, body)` — 脱糖後に見えるのは合成された `#param`
       のみ。内部整合のため push (スパンなし or スキップ) して `body` に
       再帰。ユーザーが `#param` をクリックすることは無いのでスパン欠落は
       問題なし。
     - `Expr::App` / `If` / `TyAnno` / `MakeStruct` / `ArrayLit` /
       `FFICall` / `Eval` — 単純再帰。
     - `Expr::Var` / `Expr::LLVM` — ベースケース。
   - 最初に `Some(span)` を返した時点で確定。

   なぜ見つけた `Var` 直接ではなく木を走る必要があるか: 「カーソル位置での
   スコープスタックの状態」が必要で、それはルートから辿らないと得られない
   ため。

3. **`src/commands/lsp/util.rs`** (小規模な手直し) — `get_node_at` は
   現状 `Option<EndNode>` のみ返し、`SourcePos` を内部で捨てている。
   呼び出し側で束縛探索にその `pos` を再利用したいので、戻り値を
   `Option<(EndNode, SourcePos)>` に変更。呼び出し箇所
   (`goto_definition.rs:19`、`references.rs:34`、必要なら `hover.rs`)
   を更新する。

4. **`src/commands/lsp/goto_definition.rs`** — `is_local` 分岐を差し替え:

   ```rust
   if full_name.is_local() {
       def_src = find_local_binding(program, &pos, full_name);
   } else {
       def_src = program.global_values.get(full_name).and_then(|gv| gv.decl_src.clone());
   }
   ```

   ハンドラ残りの処理 (`None` 応答、`span_to_location`) は `None` / `Some`
   のいずれにも既に正しく対応している。

5. **`src/commands/lsp/server.rs`** — 変更不要。`definition_provider`
   capability は既に立っている (ハンドラも配線済み。穴はハンドラ内部のみ)。

### 明示的に扱うエッジケース

- **束縛子自身の上にカーソル** (`let x = 1` の `x`) — `find_node_at` は
  `EndNode::Pattern(var, _)` を返し、ハンドラは `var_name = Some(var.name)`
  として進む。走査器は同じパターンを見つけてそのスパンを返すので、その場に
  ジャンプする結果になる (LSP 慣例どおりで OK)。
- **内側でのシャドウイング** — スタック top-match で自然に解決。
- **構造体分解パターン** (`let Point { x, y } = p`) — `var_bindings` が
  `Pattern::Struct` を再帰するので、各フィールド変数に個別のスパンが付く。
- **未使用引数をクリック** — グローバルは従前どおり `gv.decl_src` を使うので
  リグレッションしない。
- **どのシンボル式にも入っていない位置** — 走査器は `None` を返し、
  ハンドラは従前どおり `None` を応答。

### 検証

1. **統合テスト** — `src/tests/test_lsp/test_references.rs` のパターンに
   倣って `src/tests/test_lsp/test_goto_definition.rs` を新設
   (`LspTestCtx` ヘルパー、`setup_test_env`、`install_fix` を流用)。
   `LspTestCtx::goto_definition(file, line, col)` を追加し、
   `textDocument/definition` を送って返却 `Location` を assert する
   ユーティリティを作る。フィクスチャプロジェクト
   `src/tests/test_lsp/cases/goto_local/` に以下を含む `lib.fix` を用意:
   - 単純 let: `let x = 1 in x + 1` → `x` の使用をクリック。
   - ラムダ引数: `|a| a + 1` → `a` の使用をクリック。
   - match アーム: `match xs { Cons(h, t) => h }` → `h` をクリック。
   - 構造体分解 let: `let Point { x, y } = p in x` → `x` をクリック。
   - シャドウイング: `let x = 1 in let x = 2 in x` → 内側の `x` に解決。
   - リグレッション: グローバル名 → 従前どおり `decl_src` へジャンプ。

   実行: `cargo test --release test_goto_definition`
   (CLAUDE.md の指針どおり、広範なテストは `--release` で)。

2. **手動スモークテスト** — 再ビルドして (`cargo install --path .` など)、
   Fix LSP を設定したエディタで `.fix` を開き、let / lambda / match に
   またがって数箇所 Ctrl-クリックして正しい束縛子に飛ぶか確認する。

### スコープ外 (rename 本体での積み残し)

- rename symbol 本体 — 本計画は jump-to-definition のみ。rename は
  `var_bindings` と、同じ走査器から派生させる「ローカル名の囲みスコープを
  返す」双子関数を再利用する。
- ファイル横断 / スコープ横断のローカル参照 — Fix のローカルは単一スコープに
  閉じるので意味を持たない。
- `Var` への per-parameter スパン付与 — 脱糖のおかげで機能要件上は不要。
