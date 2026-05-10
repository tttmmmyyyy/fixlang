# LSP completion `obj.xxx` の型ベース並び替え — 実装方針

このドキュメントは **Step 1 以降を担当する実装者向けの仕様書**。Step -1 / 0 / 0.5 は完了済み。

---

## Implementation status (2026-05-10 時点)

完了 (コミット済み):

- **Step -1** 現状回帰テスト: `completion_insert` フィクスチャと `test_completion_insert_text_for_function_with_two_params` を追加 ([commit ea1b5740](../../)).
- **Step 0** Grammar 拡張: `expr_hole = { "?" ~ name_char* }` を [src/parse/grammer.pest](../../src/parse/grammer.pest) に追加。既存 `expr_hole = { expr | hole }` は `expr_or_hole` にリネーム。`parse_expr_hole` を [src/parse/parser.rs](../../src/parse/parser.rs) に追加。`hole_user_*` テスト 5 件追加 ([commit ec05189c](../../)).
- **Step 0.5** 補完挿入の `?x` snippet 化: [src/commands/lsp/completion.rs](../../src/commands/lsp/completion.rs) で `${N:?<name>}` を生成、`InsertTextFormat::Snippet` を設定 ([commit ec05189c](../../) → [commit f7bbd018](../../)).
- 副次: deprecation を completion item の `deprecated` / `tags` / documentation に反映 ([commit db4b7d73](../../)).
- **Step 1** 受信者型抽出 prototype: A0 + full re-elaborate で hole の curried 型から受信者を取り出す ([commit 13bcfc58](../../)).
- **Step 2** CompletionIndex + Tier 1/2/3 sort_text: bucket index 構築と sort_text 付与 ([commit bcd41763](../../)).
- **Step 3** Tier 1 → Tier 0 unify 昇格: 受信者位置を unify して具体型一致なら Tier 0 ([commit b52d880c](../../)).
- **Step 4** Repair の周辺修復 (pest error-driven loop): 名前付き token に加え、`{};` などの構造的 fallback を試行する pick_insertion で `if`-body / 閉じブラケット欠落を救済 ([commit 4c765543](../../) → [commit bfcc83e5](../../)).
- バグ修正: A0 numeric-literal 判定の絞り込み + 多行 dot 対応 ([commit 3d077437](../../))、override path 不一致と curry chain 1 階層しか剥かない問題 ([commit f749b8f1](../../))、parse-error snapshot で Main:: が落ちる問題に live-buffer Program を candidate 一覧にも使う ([commit 11723e0e](../../))、code-review 結果を反映 ([commit 9576b9cb](../../))、ネスト if-body の repair 単体テスト追加 ([commit 84a5cd25](../../)).

`cargo test --release` 全 806 件 pass。

**残作業**: Step 5 〜 Step 7。

> **Step 番号変更 (2026-05-10)**: 旧 Step 5「Incremental elaborate」を Step 6 にずらし、新 Step 5「エラー耐性 typecheck」を挿入。詳細は §A.11。

---

# Part A: 実装仕様

## A.1 全体の構造

新規ファイル:

```
src/commands/lsp/completion/
├── mod.rs       既存 src/commands/lsp/completion.rs を分割 (handle_completion 等のエントリ)
├── repair.rs    Repair pre-pass (A.4)
├── index.rs     CompletionIndex (バケットインデックス、A.5)
└── score.rs     Tier 判定 + sort_text 生成 (A.5)
```

既存 `src/commands/lsp/completion.rs` は `mod.rs` に rename し、内部 helper を上記 3 ファイルに切り出す。`Cargo.toml` の調整は不要 (modules 構造)。

## A.2 設計の前提と方針

### スコープ

- **P1 受信者型でのソート** のみ実装。フィルタはしない (全候補を返す)。
- ドット文脈 (`is_dot_function == true`) のときのみ新フローを起動。それ以外は現状維持。
- 型抽出に失敗したらアルファベット順全件 (= 現状の挙動) にフォールバック。

### Tier の決め方

`a.foo(b1, b2)` は parser ([parser.rs:1781-1787](../../src/parse/parser.rs#L1781-L1787)) で `App(foo(b1)(b2), [a])` として表現される — **受信者 `a` は curried 適用の末尾**。

カーソル位置に hole を埋めて elaborate すると、hole の型は

```
A1 → A2 → ... → An → Self → Ret
```

の形になる。`n` はユーザーが既に書いた追加引数の個数 (`a.<cursor>` なら n=0, `a.foo(arg1, <cursor>)` なら n=1)。`Self` が受信者型。

候補 c の Scheme を fresh instantiate した curried 型を `S1 → ... → Sm` (右端は戻り値) とすると:

| Tier | 条件 |
|---|---|
| **0** | `m ≥ n+1` かつ末尾から (n+1) 番目の引数 `S_{m-n-1}` が `Self` と完全に unify 可能 |
| **1** | `Self` の TyCon と `S_{m-n-1}` の TyCon が一致 (バケット段階での一致) だが unify 失敗 or 未試行 |
| **2** | `S_{m-n-1}` が型変数 (wildcard バケット) |
| **3** | その他 |

`sort_text = format!("{}_{}", tier, name)` を completion item に設定。LSP クライアント側のソートとプレフィックスマッチで「**型が合う + 名前が合う**」候補が最上位になる。

## A.3 補完リクエスト処理フロー

入力: `(live_buffer: &str, cursor: SourcePos, program: &Program)`

```rust
// pseudocode
fn handle_dot_completion(...) -> Vec<CompletionItem> {
    if !is_dot_function(typing_text) {
        return handle_completion_legacy(...);
    }

    // Step 1
    let (repaired_src, repaired_cursor) = match repair::repair_for_completion(live_buffer, cursor) {
        Some(x) => x,
        None    => return all_candidates_alphabetical(program),
    };

    // Step 2: パース
    let module_ast = match parse_source(&repaired_src) {
        Ok(m) => m,
        Err(_) => return all_candidates_alphabetical(program),
    };

    // Step 3: ホールノード特定
    let hole_node = match find_innermost_hole_at(&module_ast, repaired_cursor) {
        Some(n) => n,
        None    => return all_candidates_alphabetical(program),
    };

    // Step 4: incremental elaborate (Step 5 で本実装)
    //         Step 1 の prototype 段階では full re-elaborate でよい。
    let hole_type = match elaborate_for_completion(module_ast, &hole_node, program) {
        Some(t) => t,
        None    => return all_candidates_alphabetical(program),
    };

    // Step 5: 受信者型抽出
    let (receiver_type, n) = decompose_hole_type(&hole_type, &hole_node);

    // Step 6: スコアリング
    let index = get_or_build_index(program);
    let items = program.global_values.iter()
        .filter(|(name, _)| !name.to_string().contains('#') /* 既存フィルタ */)
        .map(|(name, gv)| {
            let tier = score::assign_tier(name, gv, &index, &receiver_type, n, program);
            create_item_with_tier(name, gv, tier)
        })
        .collect();
    items
}
```

## A.4 Repair pre-pass (Step 4)

**入力**: `(live_buffer: &str, cursor: SourcePos)`
**出力**: `Option<(String, SourcePos)>` (修復済みソース + 修復後のカーソル位置。失敗時は `None`)

ファイル: `src/commands/lsp/completion/repair.rs`

### A.4.1 A0 — post-dot を `?` に置換 (必ず最初に行う)

カーソルを含む「ドット直後の identifier」を `?` 1 文字に置き換える。

**識別子境界の決め方**:

- カーソル直前にある最も近い `.` (ただし数値リテラルの中の `.` は除外。判定は「ドット直前が identifier 系の文字か」で行う) を見つける。見つからなければ A0 はスキップ。
- ドット位置の直後から、`name_char` ([grammer.pest](../../src/parse/grammer.pest) の `name_char` rule に準ずる: `[a-zA-Z0-9_]`) が連続する範囲を「post-dot identifier」と定義。
- **カーソル位置がこの範囲の中または末尾** にあるなら、ドット直後から identifier 末尾までを `?` に置換。範囲外 (例: ドットからカーソルまでに空白) なら、ドット直後にカーソルがあるとみなして `?` を 1 文字挿入。

**`obj.foo<cursor>bar` の扱い**: ドットから `bar` の末尾までを丸ごと `?` で置換する (suffix `bar` も消す)。「実在する symbol を打鍵中にも置換するか」については **常に置換** する方針 (LSP client 側の string match で `foo` が依然として効くため候補が消えるわけではない)。

**カーソル位置の補正**: 置換によって文字数が変わるので、修復後カーソル位置 = (元のドット位置 + 1) (= `?` の直後を指す位置) に補正する。後段の hole ノード特定でこの位置を使う。

### A.4.2 周辺修復 — pest error-driven loop

A0 を適用しただけでは、外側構文 (let の `;`、未閉じ括弧、未完の `if` など) が壊れていればパースは通らない。pest が返すエラー情報を使って **挿入を繰り返す**。

```rust
fn repair_outer(mut src: String, mut cursor: SourcePos) -> Option<(String, SourcePos)> {
    const MAX_ATTEMPTS: usize = 8;
    for _ in 0..MAX_ATTEMPTS {
        match parse_module(&src) {
            Ok(_) => return Some((src, cursor)),
            Err(err) => {
                let (pos, insertion) = decide_insertion(&err)?;
                src.insert_str(pos, insertion);
                if pos <= cursor.byte_offset {
                    cursor.byte_offset += insertion.len(); // 必要なら line/column も更新
                }
            }
        }
    }
    None
}
```

`decide_insertion` の判定ルール (pest エラーの `expected` トークン集合を見て):

| pest が期待しているもの | 挿入する文字列 |
|---|---|
| `";"` を含む | `;` |
| `")"` を含む | `)` |
| `"}"` を含む | `}` |
| `"]"` を含む | `]` |
| `expression` 系 (= `expr_or_hole` / `expr_hole` 等の grammar rule) | `?` |
| その他 | `None` を返してフォールバック |

挿入位置は pest の `pest::error::Error::location` から取得 (`InputLocation::Pos(p)` または `InputLocation::Span((s, _))` の `s`)。

### A.4.3 Repair 全体の順序

1. A0 (post-dot を `?` 化) を適用。
2. その結果に対して `repair_outer` (A.4.2) を実行。
3. 8 回の試行で通らなければ `None` を返してフォールバック。

### A.4.4 失敗時のフォールバック

`repair_for_completion` が `None` を返したら、呼び出し側 (`mod.rs` の `handle_completion`) は **全候補をアルファベット順** で返す (= 現状の挙動と同じ)。

## A.5 バケットインデックスと Tier 判定 (Step 2 / 3)

ファイル: `src/commands/lsp/completion/index.rs`, `score.rs`

### A.5.1 CompletionIndex

```rust
pub struct CompletionIndex {
    program_ptr: *const Program,           // Arc::as_ptr で同一性判定
    by_receiver_tycon: Map<TyCon, Vec<FullName>>,
    wildcard: Vec<FullName>,
}

impl CompletionIndex {
    pub fn build(program: &Program) -> Self { ... }
    pub fn is_for(&self, program: &Program) -> bool {
        std::ptr::eq(self.program_ptr, Arc::as_ptr(program))
    }
}
```

**構築アルゴリズム** (各 `(name, gv) in program.global_values`):

1. `gv.scm.ty` を `get_lambda_srcs` ([types.rs:632](../../src/ast/types.rs#L632)) で繰り返し分解し curried 引数列 `S1, ..., Sm` を取り出す。`m == 0` (関数でない値) はスキップ。
2. 末尾引数 `Sm` に対して `toplevel_tycon` ([types.rs:937 付近](../../src/ast/types.rs#L937)) を呼ぶ。
   - `Some(tc)` → `by_receiver_tycon[tc]` に push
   - `None` (型変数) → `wildcard` に push
3. `compiler_defined_method: true` のシンボル (`@field` 等の field accessor) は **除外しない** (正当な dot 候補)。`#` 入り名前は呼び出し側 ([completion.rs:69](../../src/commands/lsp/completion.rs#L69)) で別途弾かれる。

**キャッシュ**: 起動時には作らない。最初の dot 補完で lazy 構築し、`Mutex<Option<CompletionIndex>>` 等で server 側に保持。Program 切替は `is_for` で判定、不一致なら破棄して再構築。

### A.5.2 Tier 判定

```rust
pub enum Tier { Zero, One, Two, Three }

pub fn assign_tier(
    name: &FullName,
    gv: &GlobalValue,
    index: &CompletionIndex,
    receiver_type: &TypeNode,
    n: usize,
    program: &Program,
) -> Tier {
    let receiver_tc = receiver_type.toplevel_tycon();

    let in_tycon_bucket = receiver_tc
        .as_ref()
        .and_then(|tc| index.by_receiver_tycon.get(tc))
        .map(|b| b.contains(name))
        .unwrap_or(false);
    let in_wildcard = index.wildcard.contains(name);

    if in_tycon_bucket {
        if try_unify_receiver(gv, receiver_type, n, program).is_ok() {
            Tier::Zero
        } else {
            Tier::One
        }
    } else if in_wildcard {
        Tier::Two
    } else {
        Tier::Three
    }
}
```

### A.5.3 try_unify_receiver

```rust
fn try_unify_receiver(
    gv: &GlobalValue,
    receiver_type: &TypeNode,
    n: usize,
    program: &Program,
) -> Result<(), ()> {
    let mut tc = program.create_typechecker();
    let inst = tc.instantiate_scheme(&gv.scm);
    let curried = collect_lambda_srcs(&inst.ty); // [S1, ..., Sm]
    if curried.len() < n + 1 { return Err(()); }
    let recv_pos = &curried[curried.len() - 1 - n];
    tc.unify(recv_pos, receiver_type).map_err(|_| ())
}
```

注意点:

- `Program::create_typechecker` を **候補ごとに新しく** 呼ぶ。連続使用すると substitution が混ざる。
- `collect_lambda_srcs` は `get_lambda_srcs` を `is_funptr() / is_closure()` の限り繰り返し適用するヘルパ。関数型でなくなったら停止。
- trait constraint (Scheme の `predicates`) は MVP では無視。受信者位置が型変数の trait method は wildcard (Tier 2) のまま。

### A.5.4 n の決め方

**Step 1 prototype**: `n = 0` で固定して動作確認。`a.<cursor>` に限定するため。

**Step 2 以降**: hole ノードを含む AST チェーンを上に辿り、`AppSourceCodeOrderType::XDotF` がついた App の引数を 0 として、外側に `App(_, [arg])` を適用するごとに +1。

```rust
fn count_extra_args(hole_node: &Arc<ExprNode>, ast: &ProgramAst) -> usize {
    // hole_node の親を辿り、App の app_order を見ながら数える
    // 詳細は src/ast/expr.rs の Expr::App / AppSourceCodeOrderType を参照
}
```

## A.6 ホールノード特定 (Step 1)

ファイル: `src/commands/lsp/completion/mod.rs`

```rust
fn find_innermost_hole_at(
    ast: &ProgramAst,
    cursor: SourcePos,
) -> Option<Arc<ExprNode>>;
```

**アルゴリズム**:

1. AST を深さ優先で走査。
2. `Expr::Var(name)` で `name == hole_full_name()` ([src/ast/expr.rs:1657](../../src/ast/expr.rs#L1657)) なノードを集める。
3. その中で **カーソル位置の SourcePos を span に含む** ものに絞る。
4. **最も span が短いもの** を選ぶ (innermost = ネスト最深)。

候補 0 個なら `None` (フォールバック)。

## A.7 受信者型抽出 (Step 1)

ファイル: `src/commands/lsp/completion/mod.rs`

```rust
fn decompose_hole_type(hole_type: &TypeNode, n: usize) -> Option<TypeNode> {
    let curried = collect_lambda_srcs(hole_type); // [A1, ..., An, Self]
    if curried.len() < n + 1 { return None; }
    let self_idx = curried.len() - 1 - 0; // 注: hole 自体には Ret が無い (戻り値の前で止まっている形)
    Some(curried[curried.len() - 1].clone())
}
```

**hole の型の形について**: `obj.<hole>(arg1, ..., argn)` は AST 上 `App(...App(App(hole, [argn]), ..., [arg1]), [obj])` (Fix の curry は受信者が末尾)。elaborate 後、hole の type は `A1 → ... → An → Self → Ret` ではなく、curry の途中までしか確定しないことがある。**Step 1 では n=0 限定なので hole の型 = `Self → Ret` 形** で素直に末尾の curry argument を `Self` として取る。n>0 サポートは Step 2 以降で動作確認しながら詰める。

## A.8 incremental elaborate (Step 5)

Step 1 では full re-elaborate でよい (動かすことを優先)。Step 5 で TypeCheckCache 経由の単一 global value だけの再 elaborate に切り替える。

**やること**:

1. 修復済みパース結果から、カーソルを含む global value 1 個だけを抜き出す。
2. `Program::create_typechecker` + `unify_type_of_expr` ([typecheck.rs:709](../../src/elaboration/typecheck.rs#L709)) で当該 expr のみを elaborate。
3. `collect_hole_errors` ([check_holes.rs:25](../../src/elaboration/check_holes.rs#L25)) は **呼ばない** (補完専用フローでスプリアス診断を避ける)。
4. TypeCheckCache の他の global value のキャッシュは破壊しない。

設計選択:

- 既存 `run_diagnostics` ([server.rs:1095](../../src/commands/lsp/server.rs#L1095)) にオプションを追加して呼ぶ vs. 別関数を新設する。**別関数推奨** (診断ロジックと補完ロジックを混ぜたくない)。

## A.9 Sort text の埋め込み (Step 2)

`create_item` ([completion.rs:262 付近](../../src/commands/lsp/completion.rs)) を `tier: Option<Tier>` 引数を取れるよう拡張、または wrapper を追加:

```rust
fn create_item_with_tier(...) -> CompletionItem {
    let mut item = create_item(...);
    if let Some(tier) = tier {
        item.sort_text = Some(format!("{}_{}", tier as u8, name_str));
    }
    item
}
```

ドット文脈でないときは `tier == None` で従来挙動。

## A.10 失敗時フォールバックの統一

以下のすべてで **フォールバック = アルファベット順全件返却** (現状挙動と等価):

- A.4 repair が失敗 (`None`)
- A.5 パースは通ったが hole ノードが見つからない
- A.7 elaborate が parse-time / 環境依存で失敗 (lockfile 読み込み失敗等)
- A.7 hole の `type_` フィールドが None のまま typecheck が通った
  - 旧来は「型エラーで上位 expr の elaborate が打ち切られて hole に届かない」が主な原因。Step 5 のエラー耐性 typecheck (A.13) を入れたあとは、hole の文脈型が完全に未拘束 (`fresh tyvar` のまま) なケースだけが残る。
- A.7 hole の型が curried 関数型に分解できない

各ポイントで **silent に fallback**。LSP 側にエラーは出さない。

## A.11 実装順序

| Step | 内容 | 依存 | 性質 |
|---|---|---|---|
| **1** | 受信者型抽出 prototype (n=0 固定、A0 のみ、full re-elaborate) | — | 機能 |
| **2** | CompletionIndex 構築 + Tier 1/2/3 を sort_text に反映 | Step 1 | 機能 |
| **3** | unify 段で Tier 1 → Tier 0 昇格 | Step 2 | 機能 |
| **4** | Repair の周辺修復 (A.4.2 pest error-driven loop) | Step 1 | 機能 |
| **5** | エラー耐性 typecheck (A.13) | Step 1 | 機能 |
| **6** | Incremental elaborate (TypeCheckCache 経路、A.8) | Step 1 | 性能 |
| **7** | テスト追加 (A.12) | Step 2 〜 6 | テスト |

各 Step 完了時に `cargo test --release` 全件 pass を確認。

> **2026-05-10 更新**: 旧計画の Step 5 (Incremental elaborate) を Step 6 にずらし、新 Step 5 として「エラー耐性 typecheck」を追加。理由は §A.13 参照。

## A.12 テスト

新規テストフィクスチャ: `src/tests/test_lsp/cases/completion_dot_sort/` (lib.fix / main.fix を新規。既存フィクスチャに line shift 干渉しないため別フォルダ推奨)。

`src/tests/test_lsp/test_completion.rs` に追加するテスト:

| 名前 | 検証内容 |
|---|---|
| `test_completion_dot_sort_struct_field` | `let s = MyStruct{...}; s.<cursor>` で `MyStruct` の field accessor が Tier 0 |
| `test_completion_dot_sort_array_method` | `let arr = [1,2,3]; arr.<cursor>` で `Array I64` メソッドが Tier 0 |
| `test_completion_dot_sort_unify_filters_wrong_typearg` | `Array I64` receiver に対し `Array String` 専用関数は Tier 1 (Tier 0 にしない) |
| `test_completion_dot_sort_chain_extra_args` | `arr.fold(0, <cursor>)` で n=1 ケース、第2引数位置が `Array I64` と整合する候補が Tier 0 |
| `test_completion_dot_sort_repair_let` | ファイル末尾 `let y = arr.<cursor>` (`;` なし) で repair が `;` を補い完了 |
| `test_completion_dot_sort_repair_trailing_comma` | `f(arr.<cursor>, )` で repair が `,)` を `?,)` 又は同等に補正 |
| `test_completion_dot_sort_fallback_on_repair_fail` | repair 8 回でも通らないケースで全件アルファベット順が返る |
| `test_completion_dot_sort_no_dot_unchanged` | ドット文脈でないときは sort_text が付かない (現状挙動維持) |
| `test_completion_dot_sort_inside_if_body_with_bad_cond` | `if 42.myfunc2(7) { 42.<cursor> }\n    pure()` — cond が型エラー (`I64` vs `Bool`) でも内側 hole は I64 として型付けされ、myfunc2 が Tier 0。**Step 5 (エラー耐性 typecheck) 必須**。 |

assert は「**期待する候補が `0_<name>` の sort_text で返る**」を中心に、全件返却も同時に確認。

## A.13 エラー耐性 typecheck (Step 5)

### 動機

Step 1〜4 までで A0 + outer-repair が通ったあと `elaborate_via_config` を回せば、ホールの周辺がどんなに型エラーだらけでもパースは通る。しかし、Fix の typechecker は `unify_type_of_expr` の途中で `?` 演算子を介してエラーを伝播させる作りなので、**ある式の typecheck が失敗するとその式の sibling や子の elaborate が打ち切られ**、補完位置の hole が `type_ = None` のまま帰ってくる。

実例 (実装中に発見、ネスト if-body の repair 単体テスト [commit 84a5cd25](../../) で repair 側は通る):

```fix
main : IO () = (
    if 42.myfunc2(7) {  // cond は I64、Bool が期待される → 型エラー
        42.<cursor>     // hole の文脈型は IO () であるべきだが、cond エラーで body が elaborate されない
    }
    pure()
);
```

repair 後のソースは parse できるが、`unify_type_of_expr` が `expr_if` の cond で `unify(I64, Bool)` に失敗して `?` を返し、then_expr の elaborate が呼ばれない。結果 hole の `type_` フィールドが None のまま `find_innermost_hole_at` に届く。`decompose_hole_type_n0` が None を返し、A.10 のサイレント fallback が起動する。

エラー耐性 typecheck はこの「**型エラーで sibling の elaborate が打ち切られる**」現象を補完専用フローでだけ抑止する。

### 設計

`TypeCheckContext` に `error_tolerant: bool` フィールドを追加 (`#[derive(Clone)]` 由来でクローンしても引き継がれる)。`unify_type_of_expr_inner` ([typecheck.rs:719](../../src/elaboration/typecheck.rs#L719)) の各 match arm で発生する unify 失敗を、

```rust
// 概略 (実装は arm 単位で必要)
let res = self.unify_type_of_expr(child, expected);
let typed_child = match res {
    Ok(c) => c,
    Err(e) if self.error_tolerant => {
        self.deferred_errors.push(e); // 後で診断に出す場合のみ
        // 既に typecheck が始まっていた `child` の AST 構造を残しつつ
        // 型は「使えない」を表す fresh tyvar に置き換える。
        child.clone().set_type(self.new_tyvar_star())
    }
    Err(e) => return Err(e),
};
```

の形に書き換える。具体的な置き換え点は

- `Expr::App` の func / args
- `Expr::Let` の bound / val
- `Expr::If` の cond / then_expr / else_expr
- `Expr::Match` の cond / 各 arm の val
- `Expr::TyAnno` の e
- `Expr::MakeStruct` / `ArrayLit` / `FFICall` の各引数
- `Expr::Eval` の side / main
- `Expr::Lam` の body

`unify` (両側の型を unify する側) も同じく失敗時は `error_tolerant` のとき返り値だけ swallow する選択肢があるが、両端を放置して進むと substitution が壊れるので、**unify は失敗のままで、ただし呼び出し元が `?` で抜けるかどうかをフラグで決める**、という方針。

`Program::create_typechecker_for_completion(&Configuration) -> TypeCheckContext` を追加し、`error_tolerant: true` をセット。既存の `create_typechecker` は `false` のまま。

### 補完フローからの呼び出し

`run_completion_elaborate` は今は `elaborate_via_config(&config)` を呼んでいるだけで `error_tolerant` を立てる場所がない。Configuration に `completion_error_tolerant: bool` を追加するか、補完専用の elaborate 関数を別に切り出すかは Step 5 実装時に判断。

### 注意

- 補完以外の経路 (通常診断) には影響しない (`error_tolerant: false` のまま動作不変)。
- `collect_hole_errors` ([check_holes.rs:25](../../src/elaboration/check_holes.rs#L25)) は補完経路では引き続き呼ばない (A.8 参照)。
- 副作用として「hole の型が `fresh tyvar` (どの具体型でもない)」というケースが増える可能性がある。`decompose_hole_type_n0` は引き続き `is_funptr || is_closure` で先にチェックするので、tyvar の hole 型は素直に None を返してフォールバックする。

---

# Part B: Reference

## B.1 既存 API (再利用するもの)

| 名前 | 場所 | 用途 |
|---|---|---|
| `Std::#hole : a` ビルトイン | [src/fixstd/builtin.rs:4163](../../src/fixstd/builtin.rs) | 多相プレースホルダ |
| `expr_hole(span)` | [src/ast/expr.rs:1670](../../src/ast/expr.rs#L1670) | AST 上のホールノード生成関数 (Rust 関数。grammar rule とは別物) |
| `hole_full_name()` | [src/ast/expr.rs:1657](../../src/ast/expr.rs#L1657) | hole 識別子 `Std::#hole` の FullName |
| `Program::create_typechecker` | [src/ast/program.rs](../../src/ast/program.rs) | unify 用の context を作る |
| `TypeCheckContext::instantiate_scheme` | [src/elaboration/typecheck.rs](../../src/elaboration/typecheck.rs) | Scheme を fresh 化 |
| `TypeCheckContext::unify` | [src/elaboration/typecheck.rs:1593](../../src/elaboration/typecheck.rs#L1593) | 受信者位置の単一化 |
| `TypeCheckContext::unify_type_of_expr` | [src/elaboration/typecheck.rs:709](../../src/elaboration/typecheck.rs#L709) | expr 単位の elaborate (incremental 用) |
| `TypeNode::get_lambda_srcs` / `get_lambda_dst` | [src/ast/types.rs:632](../../src/ast/types.rs#L632) / [645](../../src/ast/types.rs#L645) | curried 型分解。**panic するので `is_funptr() / is_closure()` で先にチェック** |
| `TypeNode::toplevel_tycon` | [src/ast/types.rs:937 付近](../../src/ast/types.rs#L937) | バケットインデックスのキー |
| `TypeCheckCache` / `MemoryCache` | [src/elaboration/typecheckcache.rs:14](../../src/elaboration/typecheckcache.rs#L14) | incremental elaborate のキャッシュ層 |
| `is_dot_function` | [src/commands/lsp/completion.rs:145](../../src/commands/lsp/completion.rs#L145) | ドット文脈判定 (既存) |
| `resolve_source_pos` | [src/commands/lsp/util.rs:87](../../src/commands/lsp/util.rs#L87) | LSP カーソル → SourcePos |

## B.2 Fix 言語の癖 (実装で踏みやすい点)

### Dot syntax は受信者が末尾

`a.foo(b1, b2)` は `App(foo(b1)(b2), [a])` になる ([parser.rs:1781-1787](../../src/parse/parser.rs#L1781-L1787), [grammer.pest:147](../../src/parse/grammer.pest#L147))。Rust/Swift/Kotlin と逆。**Tier 判定はこの「受信者 = 末尾引数」前提**。

### 1 引数関数の `f()` は unit-call

Fix では `f()` は「引数なし呼び出し」ではなく `f(unit)` と解釈される ([parser.rs:1930-1935](../../src/parse/parser.rs#L1930-L1935))。**Step 0.5 が `?x` snippet を入れる理由はこれ** — `f()` を入れると 1 引数関数で型エラーになる。

### `program` (型推論結果) は保存時のみ更新

[server.rs:863-872](../../src/commands/lsp/server.rs#L863-L872)。ライブバッファは didChange ごとに更新されるが、`program` は反映されない。だから補完では **リクエスト時にライブバッファを repair + parse + elaborate する** 設計になっている。snapshot AST は当てにしない。

### `Std::#hole` は user-typeable ではない

`#` を含む名前は内部用 ([grammer.pest](../../src/parse/grammer.pest) の `name_head` が `#` を許さない)。Step 0 で追加した `?x` 構文は user-writable な hole で、AST 段階で `Std::#hole` ノードに帰着する。

### `program.global_values` に trait method も入る

[program.rs:1666-1678](../../src/ast/program.rs#L1666)。dot 補完の主対象はここ 1 つ。trait method の Scheme は受信者位置が型変数なので wildcard バケットに入る (Tier 2)。

### Compiler-defined methods は通常の global_values

`@field`, `set_field`, `mod_field`, `act_field` などは `compiler_defined_method: true` で `program.global_values` に入っている ([program.rs:2145](../../src/ast/program.rs#L2145))。**正当な dot 候補なので除外しない**。`#` 入り名前 (`Std::#hole` 等) のみ別途 [completion.rs:69](../../src/commands/lsp/completion.rs#L69) のフィルタで弾かれる。

## B.3 既存テストフィクスチャ

| パス | 内容 |
|---|---|
| [src/tests/test_lsp/cases/completion/](../../src/tests/test_lsp/cases/completion/) | namespace ベースの基本補完 |
| [src/tests/test_lsp/cases/completion_insert/](../../src/tests/test_lsp/cases/completion_insert/) | `?x` snippet 挿入の確認 (`Hoge::func` 二引数) |
| [src/tests/test_lsp/cases/completion_deprecated/](../../src/tests/test_lsp/cases/completion_deprecated/) | deprecation 反映 |

新規 `completion_dot_sort/` は **別フォルダで** 作る (line shift 干渉を避ける)。

## B.4 LSP テストヘルパ

- [src/tests/test_lsp/lsp_client.rs](../../src/tests/test_lsp/lsp_client.rs): 薄い LSP client wrapper
- [src/tests/test_lsp/test_completion.rs](../../src/tests/test_lsp/test_completion.rs) の `LspCompletionCtx`: 補完テスト setup ラッパ。`complete()` / `resolve()` 利用可能
- 全 LSP テスト約 1 分 (release ビルド)。debug は遅いので `cargo test --release`

## B.5 Step 完了時のチェック

各 Step 後に必ず:

```bash
cargo test --release
```

全件 pass を確認してから次の Step へ。

---

# Part C: 未確定事項 (実装中に判断)

実装着手後に経験的に判断する点。決め打ちは避け、テストで顕在化したら方針決定:

- **trait method の精度向上**: 受信者位置が型変数の trait method (`to_string : [a:ToString] a -> String` 等) は wildcard (Tier 2) のまま。`predicates` を見て `TraitEnv::instances_of(trait_id)` 相当のルックアップで「receiver が ToString instance なら Tier 0 に昇格」を実装するかは Step 6 のテスト結果次第。
- **修復で複数の hole が出るケース**: A0 の hole 以外に、周辺修復 (例: `if c { } else { }` の空 body) で別の hole が AST 上に現れることがある。`find_innermost_hole_at` は **カーソル位置の SourcePos を span に含む最短** で選ぶので問題ないはずだが、テストで確認。
- **post-cursor の扱い**: A0 で `obj.foo<cursor>bar` の `bar` を消す方針だが、UX 上 `bar` を残したいケースが出てきたら revisit。
- **A.4.2 の expected トークン抽出 API**: pest 0.x の `pest::error::Error` の構造を確認しつつ、`ErrorVariant::ParsingError { positives, .. }` の中身を見る形になる想定。実装時に詳細を詰める。
- **n>0 ケースでの hole の型の curry 形**: Step 1 prototype は n=0 固定。n>0 サポート時に「hole の型が `A1 → ... → Self`」なのか curry の途中で止まるのか実機で確認。
