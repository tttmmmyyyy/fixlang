# 計画: 書きかけ式 (`#hole` プレースホルダ) の自動挿入

## 結論 (要約)

**実装は十分に可能。** 二つの設計選択でほぼ完成形になる:

1. **文法**: 「`expr` を許す位置を `expr | hole` に置き換える」 +
   「`hole = { &(ANY | EOI) }` でゼロ幅マッチを正規ルールにする」。
   PEG の優先選択により、`expr` がマッチすれば従来挙動、マッチしないときだけ
   `hole` がゼロ幅でマッチする。**既存挙動と完全互換**。

2. **AST**: 新バリアントを **追加しない**。hole は内部ビルトイン
   **`::Std::#hole : a`** (`undefined` と同様の polymorphic global value) への
   `Expr::Var` として表現する。これにより codegen / optimization /
   serialization / traverse の `match Expr { ... }` を **一切触らずに** 済む。

検出は型推論完了後に AST を走査して `Var(::Std::#hole)` を集める post-pass で
行い、`E_HOLE` エラーとして出力する。エラーが出ても elaboration は完走している
ため、LSP の hover (各ノードの `type_`) は通常通り提供される。

リスクは下記「既知のリスク・落とし穴」に整理した範囲。
最も慎重さが要る箇所は **lambda body の貪欲マッチ** と
**`else_of_if` / `else_of_if_with_space` (`;` 形式の else を含む)** の
2 箇所。これらはコーナーケースで予期せぬ解釈になる可能性があるため、
テストを厚めに用意する。

---

## 背景と動機

Fix のような関数型言語では、`let pat = bound; body` の `body` が必須なので、
編集中の `value : I64 = (let x = 10; );` は構文エラーになり、型推論が
一切走らない。LSP は `x` の型をホバー表示できず、ユーザーは型推論結果を
得るために `Std::undefined("")` を手で書く回避策を強いられる。

これを文法側で吸収し、書きかけ式に対しても型推論を走らせ、
LSP の hover/diagnostics として情報提供できるようにする。

ユーザー要望のスコープ:

- `let pat = bound; ` の body 省略
- `if cond { } else { ... }` のいずれか (または両方) のブロック内 expr 省略
- `if cond { ... } else <expr>` の else 後 expr 省略
  - **Fix では `else` の代わりに `;` も使える** ([grammer.pest:97-99](src/parse/grammer.pest#L97))。
    `else_of_if = { semicolon | "else" }`、
    `else_of_if_with_space = { semicolon | ("else" ~ sep) }` の両形式があり、
    `if cond { 1 }; <expr>` や `if cond { 1 } ; { 2 }` も合法。
    本計画ではこれら **全ての else 形式** で hole を許す。
- `match x { p => , q => 1 }` のアーム body 省略
- `|x| ` (lambda body 省略)
- `;;` (monad and-then 演算子) の右辺省略 — `expr1 ;; ` を受理する。
  cf. [grammer.pest:181-182](src/parse/grammer.pest#L181)
  `operator_and_then = { ";;" }` /
  `expr_and_then_sequence = { expr_type_annotation ~ (sep* ~ operator_and_then ~ sep* ~ expr)* }`

これらが満たせれば最低限の目的は達成。
追加で広げたい候補は §6 (拡張範囲) で議論する。

---

## 1. 現状のコードベース把握

### 1.1 パーサー
- pest ベース。文法は [src/parse/grammer.pest](src/parse/grammer.pest)、
  解釈は [src/parse/parser.rs](src/parse/parser.rs)。
- 関連ルール:
  - `expr_let` ([grammer.pest:83](src/parse/grammer.pest#L83))
    — `(let pat = bound in_of_let)+ ~ expr` (末尾 expr が body)
  - `expr_if` ([grammer.pest:101](src/parse/grammer.pest#L101))
    — `if cond { expr } (else { expr } | else expr)`
  - `expr_match` ([grammer.pest:87](src/parse/grammer.pest#L87))
    — 各アームは `pattern_case => expr`
  - `expr_lam` ([grammer.pest:103](src/parse/grammer.pest#L103))
    — `| pat,... | expr` (body は最も低優先度の `expr`)
  - `expr_eval` ([grammer.pest:95](src/parse/grammer.pest#L95))
    — `eval expr ; expr` (副作用と本体)
- 解釈側:
  - `parse_expr_let` [parser.rs:2035](src/parse/parser.rs#L2035)
  - `parse_expr_eval` [parser.rs:2080](src/parse/parser.rs#L2080)
  - `parse_expr_lam` [parser.rs:2090](src/parse/parser.rs#L2090)
    — 内部で `let` への脱糖を行う
  - `parse_expr_if` [parser.rs:2118](src/parse/parser.rs#L2118)
  - `parse_expr_match` [parser.rs:2134](src/parse/parser.rs#L2134)

### 1.2 AST / 型推論
- `Expr` 列挙体 [src/ast/expr.rs:1382](src/ast/expr.rs#L1382)。
  既存バリアント: `Var, LLVM, App, Lam, Let, If, Match, TyAnno, ArrayLit,
  MakeStruct, FFICall, Eval`。
- 型推論: `unify_type_of_expr` [src/elaboration/typecheck.rs:708](src/elaboration/typecheck.rs#L708)。
  各 `Expr` バリアントを再帰的に走査し、目標型 `ty` と単一化する。
- 既存の `undefined` の実装:
  - Fix 側: [src/fixstd/std.fix:42](src/fixstd/std.fix#L42)
    — `undefined : String -> a = |msg| msg.@_data._undefined_internal;`
  - ビルトイン本体: `undefined_internal_function` [src/fixstd/builtin.rs:4095](src/fixstd/builtin.rs#L4095)
    — 任意の型 `a` を返す inline LLVM (実体は unreachable)。
  - LSP code action: `quickfix_stub_text` [src/commands/lsp/code_action.rs:235](src/commands/lsp/code_action.rs#L235)
    が trait member 雛形に `::Std::undefined("unimplemented")` を挿入している。

### 1.3 LSP / 診断
- 診断は `error_to_diagnostics` [src/commands/lsp/server.rs:953](src/commands/lsp/server.rs#L953)
  経由で出力される。型推論完走時の hover には各ノードの推論型がそのまま使える。

---

## 2. 文法戦略 (中核)

### 2.1 採用案: ゼロ幅 `hole` ルール

```pest
hole       = { &(ANY | EOI) }       // ゼロ幅で常に成功する番兵ルール
expr_hole  = { expr | hole }        // 「式があればそれ、無ければ穴」
```

- `&` は positive lookahead で消費しない。
- `ANY | EOI` は EOF も含めて常に成功する。
  → `hole` は **任意位置でゼロ幅でマッチ可能** な Pair を生成する。
- pest の選択 `|` は **prioritized choice** なので、`expr` が成功する場合
  そちらが採用され、`hole` には決して落ちない。**既存パースは完全に保存**。
- `hole` Pair は span を持つ (start == end の点 span)。
  これを `Expr::Hole` の source span に転用する。

### 2.2 `expr_hole` を使う位置 (フェーズ 1)

差し替え対象は次の **2 グループ**。

#### A. 式の中の hole 位置 (7 箇所)

| ルール                    | 変更前 (末尾 / 該当部)              | 変更後                |
| ------------------------ | ----------------------------------- | --------------------- |
| `expr_let`               | `... ~ in_of_let ~ sep*)+ ~ sep* ~ expr`     | `... ~ expr_hole`     |
| `expr_eval`              | `eval ~ sep+ ~ expr ~ sep* ~ semicolon ~ sep* ~ expr` | 末尾 `expr` のみ `expr_hole` |
| `expr_if`                | `... ~ "{" ~ sep* ~ expr ~ sep* ~ "}"` (then/else 両方) と `else_of_if_with_space ~ sep* ~ expr` | それぞれ `expr_hole` |
| `expr_match`             | `pattern_case ~ sep* ~ match_arrow ~ sep* ~ expr` (両出現箇所) | `expr_hole` |
| `expr_lam`               | `... ~ "|" ~ sep* ~ expr`                    | `expr_hole`     |
| `expr_do`                | `"do" ~ sep* ~ "{" ~ sep* ~ expr ~ sep* ~ "}"` | 中身 `expr` を `expr_hole` |
| `expr_and_then_sequence` | `expr_type_annotation ~ (sep* ~ operator_and_then ~ sep* ~ expr)*` | 右辺 `expr` を `expr_hole` |

#### B. トップレベル定義の rhs (4 箇所、構造的に同じ「`= expr ;`」位置)

| ルール                          | 変更前                                      | 変更後 |
| ------------------------------ | ------------------------------------------ | ------ |
| `global_name_defn`             | `name ~ "=" ~ expr ~ semicolon`            | `expr` を `expr_hole` |
| `global_name_type_sign`        | `name ~ ":" ~ ... ~ ("=" ~ expr)? ~ semicolon` | `expr` を `expr_hole` |
| `trait_member_value_impl`      | `name ~ "=" ~ expr ~ semicolon`            | `expr` を `expr_hole` |
| `trait_member_value_type_sign` | `name ~ ":" ~ ... ~ ("=" ~ expr)? ~ semicolon` | `expr` を `expr_hole` |

#### 補足
- `expr_do` の中身は意味的には body と同じで、`do {}` を hole として
  受理する方がユーザー体験上一貫する。
- `expr_and_then_sequence` (`;;` 演算子) は monad and-then。`expr1 ;; ` で
  右辺が hole になる。連鎖 `a ;; b ;; c` は従来通り (右辺が `expr` で
  greedy にマッチするため、`expr_hole` でも先に `expr` を試して同じ結果)。
- B グループは「最も素朴な書きかけ」 (`value = ;`、`impl の名前 = ;`) なので
  式の中の hole と同じくフェーズ 1 で対応する。`global_name_type_sign` /
  `trait_member_value_type_sign` の `("=" ~ expr ~ ...)?` は **rhs 全体が
  optional** なので、`value : T;` (定義無し) はもともと合法。今回追加するのは
  `value : T = ;` という「`=` を書いたが式は無い」中間状態の受理。

### 2.3 採用しない案

**(a) `expr` 自体を optional にする (`expr?`)**

`(...)?` で実現可能だが、欠点が大きい:

- マッチ無し時に Pair が生成されない → **穴の位置 (span) が取れない**。
  surrounding pair の前 Pair の終端から推測することは可能だが汚い。
- 「`expr?` を採用した位置」と「将来的に追加した位置」の挙動差を文法上で
  視覚的に区別しづらい。

**(b) パース失敗時のリトライ (エラー駆動回復)**

pest のエラー位置と `positives` から「`expr` が期待されていた」と分かれば、
そこに `_hole_` を文字列レベルで挿入してリトライ、というアプローチ。

- 利点: 文法を一切変更しない。
- 欠点:
  - 文字列差し込みで span が全部ズレる → オフセット変換マップの保守が必要。
  - 失敗位置の解釈が pest のエラーの実装詳細に強く依存する。
  - リトライがループする可能性 (進展しない場合の停止条件設計が面倒)。
  - ユーザーが本当に typo した場合と「穴を意図」した場合の区別が困難。

これらは保守困難なので採用しない。

**(c) `Std::undefined("")` をユーザーが書くのを LSP の補完で支援**

回避策の自動化に過ぎず、根本問題 (構文エラーで型推論が走らない) を解決しない。

---

## 3. AST 表現 (新バリアントなし)

### 3.1 設計方針: ビルトイン関数 `::Std::#hole` への参照

`Expr::Hole` のような新バリアントは **追加しない**。代わりに、
既存の `Std::undefined` と同じ仕組みで **ビルトイン値 `::Std::#hole`** を
定義し、hole 位置にはこれへの `Expr::Var` を入れる。

```rust
// 概念的には:
//   #hole : a    (任意型に単一化可能なグローバル値)
//
// Hole を作る関数:
pub fn expr_hole(src: Option<Span>) -> Arc<ExprNode> {
    let mut name = FullName::from_strs(&[STD_NAME], HOLE_NAME); // "#hole"
    name.global_to_absolute();
    expr_var(name, src)  // 既存の expr_var ヘルパーを使う
}
```

### 3.2 利点
- `Expr` 列挙体に手を入れなくて済む → **既存の全 `match Expr { ... }` を
  触らない**。具体的には:
  - `Expr::stringify` ([src/ast/expr.rs:1432](src/ast/expr.rs#L1432))
  - `calc_free_vars` 系
  - `traverse` 系
  - optimization 配下の各走査
  - `generator.rs` (codegen) — `Var` として通常通り扱われる
  - serialization (`serde` 派生) — `Var` のまま
- `name.is_local() == false` (グローバル名) なので、`free_vars` 計算等の
  既存ロジックがそのまま通る。
- LSP の hover も「`Var` の type_ を表示する」既存パスでそのまま動く。

### 3.3 命名

`#hole` の `#` プレフィックスは **`name` ルールで弾かれる文字** なので、
ユーザーコードと衝突しない (cf. `name_head = ASCII_ALPHA_LOWER | "_" | "@"` —
`#` は含まれない、[grammer.pest:22](src/parse/grammer.pest#L22))。
既に内部生成名 `#monadic_value0` 等で `#` が使われているのと同じ扱い
([parser.rs:100](src/parse/parser.rs#L100))。

定数として:
```rust
// src/constants.rs
pub const HOLE_NAME: &str = "#hole";
```

### 3.4 ビルトイン定義

`undefined_internal_function`
([src/fixstd/builtin.rs:4095](src/fixstd/builtin.rs#L4095)) と同様の構造で、
`Std::#hole` を定義する:

```rust
// src/fixstd/builtin.rs
pub fn hole_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const A_NAME: &str = "a";

    // body は到達しない想定だが、防御的に undefined と同等の inline LLVM
    // (unreachable + 未定義値) で良い。
    let expr = expr_llvm(
        LLVMGenerator::HoleFunctionBody(InlineLLVMHoleBody {}),
        type_tyvar_star(A_NAME),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], type_tyvar_star(A_NAME));
    (expr, scm)
}
```

これを `Std` モジュールのビルトインとして登録 (既存の登録箇所に
1 行追加するだけ)。

> 補足: `#hole` は `String -> a` を取らない (引数なしの `a`) ことで、
> パース時に挿入する Var が `expr_var(...)` 単体で済む (`expr_app` を
> 噛ませる必要がない)。

---

## 4. パーサー側の対応

### 4.1 共通ヘルパー
```rust
fn parse_expr_or_hole(pair: Pair<Rule>, ctx: &mut ParseContext)
    -> Result<Arc<ExprNode>, Errors>
{
    match pair.as_rule() {
        Rule::expr      => parse_expr(pair, ctx),
        Rule::hole      => Ok(expr_hole(Some(Span::from_pair(&ctx.source, &pair)))),
        Rule::expr_hole => {
            // expr_hole は (expr | hole) を内部に 1 つ持つ
            let inner = pair.into_inner().next().unwrap();
            parse_expr_or_hole(inner, ctx)
        }
        _ => unreachable!(),
    }
}
```

ここで `expr_hole(src)` は §3.1 の通り `Expr::Var(::Std::#hole)` を構築する
だけのヘルパー。新しい AST バリアントは導入しない。

ただし `src` には **hole の Pair の zero-width span をそのまま使ってはいけない**
(下線が引けない)。各 `parse_expr_*` 関数で先行/後続トークンの位置から
範囲を計算した上で渡す。詳細は §5.4 「下線範囲の決め方」を参照。

### 4.2 各 parse 関数の修正
`parse_expr_let`, `parse_expr_lam`, `parse_expr_if`, `parse_expr_match`,
`parse_expr_eval`, `parse_expr_do` で、対象 expr を取り出す箇所を
`parse_expr_with_new_do` ではなく `parse_expr_or_hole_with_new_do` に置換。

`parse_expr_with_new_do` の hole 版:
```rust
fn parse_expr_or_hole_with_new_do(...) -> Result<Arc<ExprNode>, Errors> {
    let old_doctx = std::mem::replace(&mut ctx.do_context, DoContext::default());
    let value = parse_expr_or_hole(pair, ctx)?;
    let value = ctx.do_context.expand_binds(value);
    ctx.do_context = old_doctx;
    Ok(value)
}
```

### 4.3 lambda の脱糖との整合
`parse_expr_lam` では `|pat| body` を `|#param| let pat = #param in body`
に脱糖している ([parser.rs:2090-2115](src/parse/parser.rs#L2090))。

body が hole (`Var(::Std::#hole)`) でも、その let の body 位置にこの Var を
置けばよい。`expr_let(pat, bound, hole_var, span)` で問題なく構築できる
(`hole_var` も普通の `ExprNode` なので追加対応不要)。

---

## 5. 型推論との統合 (型推論コードは無変更)

### 5.1 中核

`#hole` は `Std::#hole : a` という polymorphic global value として登録される
ので、型推論側の `unify_type_of_expr_inner` の `Expr::Var` ハンドラ
([typecheck.rs:725-887](src/elaboration/typecheck.rs#L725)) が **完全にそのまま
動く**。

- `Var(::Std::#hole)` を見ると、scope から `Scheme::generalize(a)` が引かれる。
- 期待型 `ty` と単一化 → `a := ty` で必ず成功する (制約なしの自由変数なので)。
- `ei = ei.set_type(ty.clone())` ([typecheck.rs:723](src/elaboration/typecheck.rs#L723))
  により hole ノードの `type_` に **期待型** が記録される。

→ **型推論コード本体に追加するロジックはゼロ**。`undefined` と完全に同じ
扱いを受ける。

### 5.2 LSP / hover への波及
- hole 自身に対する hover はサポートしない (どうせエラーで、必要な情報は
  E_HOLE のメッセージに全部入る)。zero-width span で hover を当てに行くのは
  実装も使い勝手も微妙なので、明示的に **非サポート**。
- ただし hole 周辺の他の変数 (let で束縛された `x` など) の hover は、
  elaboration が完走するので **通常通り機能する**。これがそもそもの目的。

### 5.3 診断収集の post-pass

elaboration 完走後、`Program::global_values` の各 expression を走査して
**`Var` の名前が `::Std::#hole` であるノード** を集め、それぞれを
`E_HOLE` エラーとして出力する。

```rust
// 概念コード (どこに置くかは要設計; 既存 traverse の上に薄く)
fn collect_holes(expr: &Arc<ExprNode>, out: &mut Vec<Arc<ExprNode>>) {
    if let Expr::Var(v) = &*expr.expr {
        if v.name == FullName::from_strs(&[STD_NAME], HOLE_NAME).absolute() {
            out.push(expr.clone());
        }
    }
    // 子ノードに再帰 (既存 traverse 利用)
    ...
}
```

走査位置は `src/ast/traverse.rs` 上に薄いビジターを追加するか、
あるいは elaboration の最終段で全 global value を一度走査するパスを
1 つ加えるだけ。

### 5.4 診断メッセージ

既存のメッセージ規約 (cf. `"Type mismatch. Expected `T`, found `U`. ..."`
[typecheck.rs:1297-1305](src/elaboration/typecheck.rs#L1297)) に揃える。
英語的には Rust 等が使う `missing X` よりも、本プランでは `Expected
expression` 寄りに振った (`Expected expression of type \`T\`.`)。

> 補足: 既存の `ERR_LACKING_TRAIT_IMPL` の `"Lacking implementation of ..."`
> も同じ機会に `"Missing implementation of ..."` に揃え、エラーコード
> `"lacking-trait-impl"` も `"missing-trait-impl"` にリネーム済み。

#### 文面 (実装済み)

```
Expected expression of type `I64`.
```

hole の型が type 推論で解決できなかった場合 (周囲の文脈が型変数のままで
固まらないとき) は、内部表記の type variable 名 (`#a0` 等) がそのまま出る:

```
Expected expression of type `#a0`.
```

完全に型情報がない場合 (典型的には typecheck の前段で他のエラーが先に出て
substitution が走らなかったケース) は、型部分を省略する:

```
Expected expression.
```

> **計画から削除**: 残存制約 (`tc.predicates` / `tc.equalities`) を
> `Scheme::to_string()` で `[a : Show] a` のように描画する案は **スコープ外**
> とした。type variable 単独で出すだけでも十分実用的で、constraint 抽出は
> 複雑度に見合わない。

#### エラーコード

`src/constants.rs`:
```rust
pub const ERR_HOLE: &str = "missing-expression";
```

#### 実装メモ

`check_holes::collect_hole_errors` ([src/elaboration/check_holes.rs](src/elaboration/check_holes.rs))
が type-checked AST を walk し、各 `Var(::Std::#hole)` ノードに対して:
1. `node.type_` から substitution 適用済みの型を取得 (なければ "Expected expression." のみ)
2. `Error` 生成 (`code = Some(ERR_HOLE)`)

優先順位 (実装は `check_type` 内、§7 参照):
```
hole > cannot-infer > predicate > equality
```
hole があるときは cannot-infer は出さない (hole が原因のことが多い)。

- **常にエラー扱い**。`fix build`, `fix check`, LSP のすべてで `ERR_HOLE` は
  エラーとして扱う。
- elaboration は完走するので、**周辺ノードの hover は通常通り提供される**。
- 1 ファイルに hole が複数あれば全て個別の `ERR_HOLE` として報告。

#### 下線範囲 (source span)

hole pair の zero-width span をそのまま使う。LSP / 既存の error renderer が
zero-width span を 1 文字幅の下線として描画してくれるので、これで十分実用的。

> **計画から削除**: 「先行〜後続トークン間に下線、隙間ゼロは後続 1 文字に
> 伸ばす」案は **採用しなかった**。`compute_hole_span` ヘルパー設計は過剰で、
> 各 parse 関数の signature が肥大化していたため、ユーザー指示でゼロ幅
> span 採用に簡素化した。

### 5.5 codegen 経由のフェイルセーフ

`fix build` モードで万一 hole が codegen まで到達した場合に備えて、
`InlineLLVMHoleBody` の codegen 実装は `undefined` と同等の
unreachable + 未定義値生成にする。`--no-runtime-check` 等で hole がランタイムに
到達した場合は `undefined` と同じく未定義動作。

ただし、診断パス (§5.3) は **code generation の前段で走らせる** ので、
通常運用では codegen に到達しない。

---

## 6. 拡張範囲 (フェーズ 2 以降の検討)

将来的に追加すると嬉しい位置:

- `expr_let` の **bound** (`= bound` の `bound`) — `let x = ;` を許す
- `arg_list` 内の各引数 — `f(x, , z)` を許す
  → ただし、tuple/array literal も含めて「カンマで区切られた要素位置」での
  empty を許すと文法的にやや繊細になる (誤解の余地が増える)。慎重に。
- `expr_make_struct` の各 field value (`Foo { x: , y: 1 }`)
- 演算子の右辺 (`1 + `, `f >> `, `x . `) — これは演算子優先順位と
  絡んで複雑なので **要検討** (フェーズ 3 候補)。

フェーズ 1 では §2.2 の A + B 全 11 箇所に絞り、運用で困った位置を順次
追加する方針。

---

## 7. コードジェネレータ・最適化への対応

新 AST バリアントを追加しないので、optimization / codegen 配下の
`match Expr { ... }` には **追加分岐は不要**。`#hole` は単に
`Var(::Std::#hole)` であり、その scheme/type は `undefined` 系と同じく
`InlineLLVM` 経由で codegen される。

唯一行うのは `InlineLLVMHoleBody` の codegen 実装。`undefined_internal` の
`InlineLLVMUndefinedInternalBody` ([builtin.rs:4070-4090](src/fixstd/builtin.rs#L4070))
を雛形にして、`build_unreachable` + `Object::undef(ty)` をそのまま流用する。

通常運用では §5.3 の post-pass が elaboration 後に E_HOLE を出すので、
`fix build` は codegen に到達せずエラー終了する。
`fix check` / LSP は診断を出した後も型情報の提供を続ける。

---

## 8. 実装ステップ (推奨順)

### ステップ 1: パーサー全変更 + ビルトイン登録 + 既存テスト全 pass + システムインストール

このステップだけで完結する区切り。**新機能としての診断 (ERR_HOLE) はまだ
出さない** が、既存コードに対しては完全に互換 (PEG の優先選択で `expr` 側が
先に試されるため)。完了後にシステムにインストールしてユーザーに連絡し、
外部プロジェクトでの回帰テストを依頼する。

#### 文法 + パーサー
- [ ] `grammer.pest` に `hole = { &(ANY | EOI) }` と
      `expr_hole = { expr | hole }` を追加。
- [ ] §2.2 の **A グループ全 7 箇所** を `expr_hole` に差し替え。
- [ ] §2.2 の **B グループ全 4 箇所** を `expr_hole` に差し替え。
- [ ] `parse_expr_or_hole` ヘルパーを実装。
- [ ] `parse_expr_let`, `parse_expr_lam`, `parse_expr_if`, `parse_expr_match`,
      `parse_expr_eval`, `parse_expr_do`, `expr_and_then_sequence` の解釈、
      および B グループの解釈関数 (`parse_global_name_defn` 等) を
      `parse_expr_or_hole` を呼ぶ形に修正。
- [ ] 下線範囲計算 (`compute_hole_span`) ヘルパーを実装し、各 parse 関数で
      先行/後続トークンの位置を渡す (§5.4 「下線範囲の決め方」)。

#### ビルトイン
- [ ] `src/constants.rs` に `HOLE_NAME = "#hole"`, `ERR_HOLE = "missing-expression"`
      を追加。
- [ ] `src/fixstd/builtin.rs` に `hole_function()` を実装
      (`undefined_internal_function` を雛形に)。
- [ ] `InlineLLVMHoleBody` の codegen を `InlineLLVMUndefinedInternalBody`
      ([builtin.rs:4070](src/fixstd/builtin.rs#L4070)) を雛形に追加。
- [ ] Std モジュールへの登録 (既存 builtin の登録箇所に並べる)。
- [ ] `expr_hole(src)` ヘルパー (= `expr_var(FullName(Std, #hole), src)`)
      を実装。

#### テスト

**このタスクの最大の懸念は「pest の文法でこれだけの位置に hole を許せるのか」
の実現可能性**。よってステップ 1 で §10.1 の **全ケース** をパース成功
レベルまで検証してしまう (ERR_HOLE 出力は後回しでよい)。

- [ ] **既存テストが全部 pass** (`cargo test --release`)。回帰ゼロが必須。
      ここで失敗するテストがあれば §2.1 / §9 で挙げた
      コーナーケースが顕在化している → 原因追跡。
- [ ] §10.1 の **全ケース** を `src/tests/test_basic.rs` 等に追加し、
      **パース成功 + 型推論完走** を確認:
  - A グループ (括弧あり版 — `)` が hole の closer):
    `hole_in_let_paren`, `hole_in_eval_paren`, `hole_in_lam_paren`,
    `hole_in_and_then_paren`
  - A グループ (括弧なし版 — global の `;` が hole の closer):
    `hole_in_let_bare`, `hole_in_eval_bare`, `hole_in_lam_bare`,
    `hole_in_and_then_bare`
  - if 系: `hole_in_if_then`, `hole_in_if_else`, `hole_in_if_else_word`,
    `hole_in_if_else_semi_block`, `hole_in_if_else_semi_expr`,
    `hole_in_if_then_semi`
  - match: `hole_in_match_first`, `hole_in_match_last`,
    `hole_in_match_last_trailing_comma`, `hole_in_match_only`,
    `hole_in_match_all`
  - その他: `hole_in_do`
  - monadic bind との相互作用: `hole_in_let_after_bind_paren`,
    `hole_in_let_after_bind_bare`, `hole_in_do_after_bind`
  - B グループ: `hole_global_defn`, `hole_global_with_sign`,
    trait impl rhs の hole
- [ ] この時点では ERR_HOLE はまだ出さないので、テストの期待は
      「コンパイル成功」または「ERR_HOLE 以外のエラーで止まらない」のいずれか。
      不正な codegen に進まないことも併せて確認。
- [ ] §9.1 / §9.2 のコーナーケース (lambda body 貪欲マッチ、
      `;` 形式の else) も最低限のパーステストでカバー。
      ここで意図せぬ解釈になっていれば文法戦略 (§2.1) を再検討。
- [ ] §10.3 ネスト・組み合わせ系 (let の中の let、if の中の let、
      入れ子 lambda、複数 hole)。
- [ ] §10.4 ホワイトスペース・コメントとの組み合わせ
      (隙間ゼロ、改行多数、ブロック/行コメント混在)。
- [ ] §10.5 EOF 境界 (`&(ANY | EOI)` の `EOI` 分岐確認)。

#### システムインストール
- [ ] `cargo build --release` (release 失敗時は debug でも可)。
- [ ] `target/release/fix` を `~/.cargo/bin/fix` に上書きコピー
      (既存テスト util `install_fix()` と同じ手順、[test_util.rs:17](src/tests/test_util.rs#L17))。
- [ ] `fix version` で動作確認。
- [ ] **ユーザーに連絡** — 外部プロジェクトでの回帰テストを依頼。
      何か壊れていればこの時点で発見・修正してから次へ進む。

---

### ステップ 2: 診断 post-pass (ERR_HOLE 出力)
- [ ] elaboration 完走後に `Var(::Std::#hole)` を集める post-pass を追加。
- [ ] §5.4 の文面で `ERR_HOLE` (`Missing expression of type \`T\`.`) を出力。
      残存制約は `tc.predicates` から hole の自由型変数を含むものを抽出して
      合成 Scheme として描画。
- [ ] §10.1 の全ケースで「パース成功 + 型推論完走 + ERR_HOLE が
      期待箇所に出る」ことを確認。

### ステップ 3: LSP 経由で hover / diagnostic を確認
- [ ] `fix lsp` 起動して、書きかけ式の中の `let` 束縛変数に hover で
      型が表示されることを目視確認。
- [ ] hole 位置の diagnostic が「Missing expression of type ...」を含むことを
      確認 (LSP クライアントで赤波線が引かれること)。

### ステップ 4: 追加のコーナーケース・ネガティブテスト
ステップ 1 で基本的なパース可能性は確認済み。ここでは boundary を厚めにする。

- [ ] §10.4 のネガティブテスト: 通常の構文エラー (typo 等) が hole として
      誤って受理されないこと (例: `let x = 10 in {`、`if cond` 単独 等)。
- [ ] ステップ 2 で追加した ERR_HOLE 出力に対する文面・下線範囲・
      残存制約の表示の正確性確認。
- [ ] 既存の各エラーメッセージとの干渉なし確認 (回帰)。

### ステップ 5: ドキュメンテーション
- [ ] `Document.md` / `Document-ja.md` に書きかけ式の挙動を追記。

---

## 9. 既知のリスク・落とし穴

### 9.1 lambda body の貪欲マッチ
`expr_lam = "|" pat,... "|" sep* expr_hole` で body を貪欲に取るため、
意図しない解釈が起き得る:

```fix
foo = |x| ;     // body = hole, あとは ; が global terminator → OK
foo = |x|       // body = hole, ; が無いと global parse 失敗 → 普通の構文エラー
bar = |x| g(y)  // body = g(y), 従来通り
```

問題例として:
```fix
let f = |x| in g(x);   // 意図不明だが、`in` は keyword なので expr に来ない
                       // → body = hole, `in` 以降は let の continuation
```

→ ` in ` は `in_of_let` のキーワードなので `expr` の先頭に来れない (
`name = !keywords ~ ...`)。よって lambda body は hole になり、`in` は
let の区切り、`g(x)` が let の body、と自然に解釈される。**問題なし。**

### 9.2 `else_of_if` / `else_of_if_with_space` との相互作用 (`;` 形式を含む)

Fix の `if` の else 節は **4 通りの書き方** を許す
([grammer.pest:97-101](src/parse/grammer.pest#L97)):

```pest
else_of_if            = { semicolon | "else" }
else_of_if_with_space = { semicolon | ("else" ~ sep) }

expr_if = { "if" ~ ... ~ "{" ~ ... ~ "}" ~ sep* ~
            (  (else_of_if ~ sep* ~ "{" ~ expr ~ "}")
             | (else_of_if_with_space ~ sep* ~ expr) ) }
```

つまり:

| 書き方                          | 経路                              |
| ------------------------------- | --------------------------------- |
| `if c { 1 } else { 2 }`         | `else_of_if` + `{ expr }`         |
| `if c { 1 }; { 2 }`             | `else_of_if` (semicolon) + `{ expr }` |
| `if c { 1 } else 2`             | `else_of_if_with_space` + `expr`  |
| `if c { 1 }; 2`                 | `else_of_if_with_space` (semicolon) + `expr` |

末尾 `expr` を `expr_hole` に変えると、上記 **すべての else 形式** で空 body
が許容される:

```fix
if c { 1 } else { }     // hole in {}
if c { 1 }; { }         // hole in {}, ; 形式
if c { 1 } else ;       // hole in expr 位置, else 形式
if c { 1 } ; ;          // hole in expr 位置, ; 形式 ★要注意
if c { } else { 1 }     // then 側 hole
if c { } ; { 1 }        // then 側 hole, ; 形式
```

最後から 2 番目 `if c { 1 } ; ;` が **重要なコーナーケース**:

- 1 個目の `;` = `else_of_if_with_space` のセミコロン
- そのあとの `expr_hole` が hole にマッチ (空)
- 2 個目の `;` = global definition の終端 (`global_name_defn` の `semicolon`)

PEG が貪欲に `expr` を試してから hole に落ちる順序のため、
`expr` の先頭になり得ない `;` は必ず hole 経由でパースされる。OK。

ただし、`if c { 1 } ;` だけ書いて global の `;` が無い場合:

- `expr_hole` は hole で空マッチ
- そのあと global の `;` が必要だが見つからずパース失敗

→ 「else 節の代わりに `;` で書きかけて global terminator もまだ書いていない」
ケースは構文エラーのまま (hole では救えない)。これは **意図的な制限**。

`else if` 連鎖については、`else_of_if_with_space` の後の `expr_hole` の
`expr` 側が `expr_if` を含むので、`if c { 1 } else if d { 2 } else { 3 }`
は従来通り正しく解釈される。hole に落ちない。**問題なし。**

`;` 形式の else については、テストケースでも明示的にカバーする (§10.1 参照)。

### 9.3 monadic bind (`*` 演算子) と hole の相互作用
`*expr` は `parse_expr` の中で `DoContext::push_monad` によって
`Var(#monadic_value0)` に置換され、外側の `expand_binds` で `>>=` に脱糖される
([parser.rs:97-129](src/parse/parser.rs#L97))。

hole は単なる `Var(::Std::#hole)` であり `expand_binds` の対象では **ない**。
そのため:

```fix
let s = *read_file; <hole>
```

は次のように脱糖される:
```
let_<- expand_binds は expr_let より外側で動く
read_file >>= |#monadic_value0|
    let s = #monadic_value0 in
        ::Std::#hole
```

hole は bind chain の最も内側に自然に収まり、型推論時には期待型 (`a` 型変数)
にバインドされる。**特殊扱い不要**、フェーズ 1 でこの組み合わせも自動的に
動く。

#### `expr_do` も同様

`expr_do = { "do" ~ "{" ~ expr ~ "}" }` の `expr` を `expr_hole` にすれば
`do {}` も受理される。`do { let s = *m; }` のように bind と hole が共存しても、
`do` ブロックの最外周にある `parse_expr_with_new_do` の `expand_binds`
が bind を脱糖し、その内側に hole が残る。

→ フェーズ 1 で `expr_do` も含めて差し支えなし (§2.2 の表参照)。
§10.1 の `hole_in_let_after_bind_*` / `hole_in_do_after_bind` テストで
明示的に網羅。

### 9.4 既存エラーメッセージへの影響
これまで「式が必要ですよ」と出していた構文エラーが、hole として受理され、
代わりに型エラー (E_HOLE) になる。エラー文言が変わる回帰として
テスト出力の更新が必要。

### 9.5 traverse / serialization
新 AST バリアントを導入しないので、serde 派生・traverse・キャッシュ
(`typecheckcache.rs`) のいずれも **変更不要**。`#hole` は単なる
`Expr::Var` として扱われ、既存の serialization パイプラインで完全に保存される。

### 9.6 hole の連鎖と型推論の停止性
`(let x = (let y = ; ); )` のような多重 hole でも、それぞれ独立に hole として
扱われ、各々が任意型を取れるので、型推論はループしない。**問題なし。**

### 9.7 演算子優先順位を跨ぐ hole
フェーズ 1 では `expr_lam` body などが `expr_hole` だが、その配下の演算子
(`+`, `*` 等) の右辺は変更しない。なので `1 +` のような演算子右辺 hole は
フェーズ 1 では受理しない (構文エラーのまま)。ユーザー要望にも含まれていない
ので **意図的な制限**。

---

## 10. テスト計画

### 概観: テスト分類とステップ対応

| 節    | カテゴリ                       | ステップ |
| ----- | ----------------------------- | ------- |
| 10.1  | パース成功 (各位置の hole 受理)   | 1       |
| 10.2  | 既存テスト回帰なし               | 1       |
| 10.3  | ネスト・組み合わせ系             | 1       |
| 10.4  | ホワイトスペース・コメント       | 1       |
| 10.5  | EOF 境界                       | 1       |
| 10.6  | 診断内容の正確性 (文面・型・下線) | 2       |
| 10.7  | 複数 hole / 他のエラーとの共存   | 2       |
| 10.8  | LSP インテグレーション          | 3       |
| 10.9  | fix コマンド end-to-end         | 3       |
| 10.10 | ネガティブ (誤受理しない)        | 4       |
| 10.11 | キャッシュ整合性                | 4       |

ステップ 1 完了時点 (パーサー変更 + インストール) では §10.1〜10.5 が
通っていることが必要。これがステップ 1 の合格条件。

### 10.1 ユニットテスト (パース成功確認)
`src/tests/test_basic.rs` 等に Fix コードを追加:
```fix
module Test;

// 各位置の hole 受理 — 括弧で囲んだ版 (`)` が hole の closer)
hole_in_let_paren : I64 = (let x = 10; );
hole_in_eval_paren : I64 = (eval 1; );
hole_in_lam_paren : I64 -> I64 = (|x| );
hole_in_and_then_paren : IO () = (IO::println("hi") ;; );

// 同じ位置の hole 受理 — 括弧なし版 (global の `;` が hole の closer)
// closer が違うとパース文脈も違うので両方テストする
hole_in_let_bare : I64 = let x = 10; ;
hole_in_eval_bare : I64 = eval 1; ;
hole_in_lam_bare : I64 -> I64 = |x| ;
hole_in_and_then_bare : IO () = IO::println("hi") ;; ;

// if 系
hole_in_if_then : I64 = if true { } else { 1 };
hole_in_if_else : I64 = if true { 1 } else { };
hole_in_if_else_word : I64 = if true { 1 } else ;
// `;` 形式の else (else_of_if / else_of_if_with_space のセミコロン形式)
hole_in_if_else_semi_block : I64 = if true { 1 }; { };
hole_in_if_else_semi_expr  : I64 = if true { 1 }; ;
hole_in_if_then_semi       : I64 = if true { }; { 1 };

// match / do
hole_in_match_first : I64 = match 0 { 0 => , _ => 1 };  // 最初のアーム body が hole
hole_in_match_last  : I64 = match 0 { 0 => 1, _ => };   // 最後のアーム body が hole (`}` が closer)
hole_in_match_last_trailing_comma : I64 = match 0 { 0 => 1, _ => , };  // trailing comma あり
hole_in_match_only  : I64 = match 0 { _ => };           // 単一アームかつ body が hole
hole_in_match_all   : I64 = match 0 { 0 => , _ => };    // 全アーム body が hole
hole_in_do : IO () = do { };

// monadic bind 演算子 `*` との相互作用
// `*expr` は parser.rs の `expand_binds` で >>= に脱糖される。
// hole は `Var(::Std::#hole)` という普通の Var なので、let body の位置で
// hole になっても expand_binds の対象にならず、bind の連鎖が
// `read_file >>= |s| #hole` のように自然に組み立たる。
some_io : IO String = undefined("");
hole_in_let_after_bind_paren : IO () = (let s = *some_io; );  // 括弧あり版
hole_in_let_after_bind_bare  : IO () = let s = *some_io; ;    // 括弧なし版
hole_in_do_after_bind        : IO () = do { let s = *some_io; };  // do 内 + bind + hole body

// トップレベル定義 rhs (B グループ)
hole_global_defn = ;
hole_global_with_sign : I64 = ;
// trait impl の rhs もテストするが Fix コードが長くなるので別ファイルで
```
これらは **パースが成功して型推論が走り、最終的に `ERR_HOLE` が
出ること** を期待する (実行はしない)。

### 10.2 ユニットテスト (回帰)
既存 `test_basic.rs` の全テストが回帰しないこと (式テストの大半に影響しうる)。

### 10.3 ネスト・組み合わせ系 (パース・型推論)

複数 hole が交わるケースは PEG 探索順や span 計算で予想外の挙動が出やすい。

```fix
// hole の入れ子
nested_let_let : I64 = (let x = (let y = 10; ); );  // let の body が hole の let
nested_if_let  : I64 = if true { let x = 10; } else { 1 };  // let が if の中
nested_match_let : I64 = match 0 { 0 => let x = 10; , _ => 1 };  // let が match arm 内
nested_lam_if  : I64 -> I64 = |x| if x == 0 { } else { x };  // lam body 内に hole-if
nested_lam_lam : I64 -> I64 -> I64 = |x| |y| ;  // 入れ子 lambda の最内 body が hole
multi_holes : I64 = (let x = (let y = ; ); );  // 2 つの hole (期待: 両方とも報告)
```

### 10.4 ホワイトスペース・コメントとの組み合わせ

`hole = { &(ANY | EOI) }` は sep を消費しないため、sep の挙動と相性が
悪い場合がある。

```fix
// 隙間ゼロ (前後トークン直結) — 下線を後続 1 文字に伸ばすフォールバック発動
ws_zero : I64 = (let x = 10;);

// 大量の whitespace
ws_many : I64 = (let x = 10;     


);

// hole 直前のブロックコメント
comment_block : I64 = (let x = 10; /* todo */ );

// hole 直前の行コメント
comment_line : I64 = (let x = 10; // todo
);

// hole 直前後で改行入れまくり
mixed_ws : I64 =
    if true {
    }
    else {
        1
    };
```

### 10.5 EOF 境界 (`&(ANY | EOI)` の検証)

ファイル末尾が hole で終わるケースは `&(ANY | EOI)` の `EOI` 分岐が
効くかの確認。

```fix
// ファイル末尾が hole — 最後の global 定義の直後で EOF
trailing_hole : I64 = (let x = 10; );
// このあと何も無い (EOF)
```

`trailing_hole` を **ファイル末尾に置いた版** をテストファイルとして用意。

### 10.6 診断内容の正確性 (ステップ 2 完了後)

ERR_HOLE のメッセージ文面・位置・型情報が期待通りであることを確認する
スナップショット系テスト。

```fix
// 期待型 = I64
expect_i64 : I64 = ;
// 期待: "Expected expression of type `Std::I64`."

// 期待型 = String
expect_string : I64 -> String = |x| ;
// 期待: "Expected expression of type `Std::String`."

// 多相: 残った型変数 (内部表記) が表示される
expect_polymorphic : a -> a = |x| ;
// 期待: "Expected expression of type `#a0`." (または同等の自由型変数名)
```

> **計画から削除**: 残存制約 (predicates) を `[a : Show] a` 形式で表示する
> 案 (旧 §5.4) は **スコープ外** とした。それに伴い `expect_with_constraint`
> パターンのテストも不要。

下線範囲については、hole pair の zero-width span そのまま (LSP renderer が
1 文字幅で描画) なので、特別な自動チェックは不要。

### 10.7 複数 hole / 他のエラーとの共存

post-pass の早期 return バグ防止。

```fix
// 同一ファイル内に複数 hole — 全て個別の ERR_HOLE が出ること
multi_a : I64 = ;
multi_b : I64 = ;
multi_c : I64 = ;

// hole + 型エラー — 両方報告されること
type_mismatch : I64 = "string";  // 既存の型エラー
hole_after_type_err : I64 = ;
```

### 10.8 LSP インテグレーションテスト

`src/tests/test_lsp/` 配下に追加。

- 書きかけ式中の `let x = ...; ` の `x` にホバーして型が出ること
  (これがそもそもの目的)。
- hole 位置に diagnostic (赤波線) が表示されること。
- hole 周辺で **goto definition** / **find references** などが従来通り
  動くこと (hole があっても他の機能を巻き込まない)。
- hole に対する hover は何も返さない (非サポート、§5.2 決定事項)。

### 10.9 fix コマンド end-to-end

`src/tests/` のインテグレーションテストパターン (`install_fix()` →
`Command::new("fix")`) に従って、各サブコマンドで一貫した挙動になることを
確認:

- `fix check` — hole 入りファイルで ERR_HOLE が stderr に出る、exit code は失敗
- `fix build` — ERR_HOLE で codegen に進まずビルド失敗
- `fix run` — 同上、実行に進まない

### 10.10 ネガティブテスト

通常の構文エラーが hole として誤って受理されないこと。

```fix
// `in` のあとに `{` だけ — let の body が「`{` で始まる expr」として
// 処理途中で失敗する。hole で救えてはいけない (本物の構文エラー)。
let x = 10 in {

// if の cond が無い
if {

// match の `{` だけ — 空 match は禁止 (現在の挙動を維持、§ parse_expr_match)
match x {

// `let` キーワード単独
let

// hole を許していない位置 (フェーズ 2 領域) で書きかけ
let x = ; body  // bound の hole は受理しない、構文エラー
1 + ;           // 演算子右辺の hole は受理しない
f(x, , z)       // arg_list の hole は受理しない
```

これらが **構文エラーのまま** であることを確認 (= hole として救われていない)。

### 10.11 キャッシュ整合性 (`typecheckcache.rs`)

新 AST バリアントを導入していないので serialize/deserialize は触らないが、
hole が入った状態でキャッシュされる/復元されるシナリオで挙動が壊れないこと:

- hole 入りファイルを `fix check` 実行 → キャッシュ保存
- 同じファイルを再度 `fix check` → キャッシュから復元、ERR_HOLE が同じく出る
- hole を消したファイルに更新 → キャッシュ無効化、エラーなし

(実装上は `Var` のまま保存されるので追加対応は不要なはずだが、
念のため確認するテスト。)

---

## 11. 決定事項

- hole は **常にエラー** (`ERR_HOLE = "missing-expression"`)。
  `fix build` / `fix check` / LSP のすべてでエラー扱いとする。
  warning 運用や二段運用はしない。
- メッセージ文面は **`Expected expression of type \`{type}\`.`** に統一
  (§5.4)。`{type}` は `TypeNode::to_string()` の出力 (例: `Std::I64`、
  または未解決の場合は `#a0`)。型情報がない場合は型部分を省略して
  `Expected expression.` にする。
- 残存制約 (predicates) を `[a : Show] a` 形式で描画する案は **スコープ外**
  とした (実装が複雑度に見合わない)。
- hole 自身に対する hover は **非サポート** (`#`-prefixed 名はすべて
  `commands/lsp/hover.rs` で suppress)。
- 下線範囲は **hole pair の zero-width span をそのまま使う**。LSP renderer が
  1 文字幅で描画するので実用上問題なし。
- hole を `?` 等の **明示記号** で書ける機能 (= `undefined("")` のシンタックス
  シュガー) は **本フェーズではスコープ外**。必要になれば別途検討。
- typecheck cache は **エラーがあるとき保存しない** (
  `resolve_namespace_and_check_type_sub` 参照)。古い cache で hole 検出が
  バイパスされるのを防ぐ。

---

## 付録 A: pest における `&(ANY | EOI)` の挙動メモ

- `&` は positive lookahead で、消費せずにマッチ判定だけ行う。
- `ANY` は任意の 1 文字、`EOI` は end-of-input。
- 通常は `ANY` だけだと EOF 直前で失敗するので `| EOI` を OR する。
- `&(ANY | EOI)` は **常にゼロ幅で成功** する番兵パターン。
- マッチした位置の `Pair` の span は `start == end` の点 span。
  これを Hole の source として扱える。

## 付録 B: 関連ファイル一覧

実装で **編集** が必要なファイル:

- [src/parse/grammer.pest](src/parse/grammer.pest)
  — `hole`, `expr_hole` ルール追加と 5 箇所の差し替え
- [src/parse/parser.rs](src/parse/parser.rs)
  — `parse_expr_or_hole` ヘルパーと各 `parse_expr_*` の修正
- [src/ast/expr.rs](src/ast/expr.rs)
  — `expr_hole(src)` ヘルパー (1 関数追加のみ。`Expr` 列挙体は無変更)
- [src/constants.rs](src/constants.rs)
  — `HOLE_NAME = "#hole"` 定数追加
- [src/fixstd/builtin.rs](src/fixstd/builtin.rs)
  — `hole_function()` と `InlineLLVMHoleBody` 追加 (既存
  `undefined_internal_function` を雛形に)
- [src/elaboration/](src/elaboration/) 配下のどこか (要設計)
  — elaboration 完走後に `Var(::Std::#hole)` を集めて E_HOLE を出す post-pass
- [src/error.rs](src/error.rs)
  — エラーコード `ERR_HOLE` 追加
- [src/tests/test_basic.rs](src/tests/test_basic.rs)
  — 受理テスト・回帰テスト
- [Document.md](Document.md), [Document-ja.md](Document-ja.md)
  — 仕様追記

実装で **編集が不要** なファイル (新方針の利点):

- `src/ast/traverse.rs` — `Var` のまま走査されるので変更なし
- `src/generator.rs` — `Var → InlineLLVM` の既存経路で codegen される
- `src/optimization/` 配下 — `match Expr { ... }` への新分岐追加なし
- typecheckcache の serialization — `Var` のまま保存される
