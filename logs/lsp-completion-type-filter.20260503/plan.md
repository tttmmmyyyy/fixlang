# LSP completion `obj.xxx` の型ベース並び替え — 実装方針

## Implementation status (2026-05-04 時点)

すでに以下が完了しコミット済み:

- **Step -1** (現状回帰テスト): `completion_insert` フィクスチャと `test_completion_insert_text_for_function_with_two_params` を追加 ([commit ea1b5740](../../)).
- **Step 0** (Grammar 拡張): `expr_hole = { "?" ~ name_char* }` を [src/parse/grammer.pest](../../src/parse/grammer.pest) に追加し `expr_nlr` の選択肢に組込済。既存の `expr_hole = { expr | hole }` は **`expr_or_hole` にリネーム済**。`parse_expr_hole` を [src/parse/parser.rs](../../src/parse/parser.rs) に追加 ([commit ec05189c](../../)).
- **Step 0.5** (補完挿入の `?x` 化): [completion.rs:282-296](../../src/commands/lsp/completion.rs) で各パラメタを `?<name>` 形式にラップ済。テスト期待値も `func(?x, ?y)` / `func(?x)` 形式に更新済 ([commit ec05189c](../../)).
- 副次的に **deprecation の completion 反映** ([commit db4b7d73](../../)): `gv.deprecation` を completion item の `deprecated` / `tags` / documentation に反映する処理を追加。`DeprecationInfo::to_markdown()` で `fix docs` と LSP の表記を統一。

**残作業**: Step 1 以降 (受信者型抽出 prototype → バケットインデックス → unify 段階 → repair pre-pass → incremental elaborate)。

`cargo test --release` 全 792 件 pass。

---

## Context

Fix の LSP の補完 (`textDocument/completion`) は、ドット記法 `obj.xxx` を打っているときも obj の型を見ずに、namespace プレフィックスだけで全候補を返している。Std だけでも数千シンボルあり、ノイズが大きい。

これを **受信者の型でスコアリングし、適合するものを上位に並べる方式** に変える。fillter ではなく **sort** にすることで:

- 型抽出が誤った場合や repair で意図と違う AST を作った場合でも「欲しい候補が消える」事故を避けられる (下位に押されるだけ)。
- 全件は引き続き返るので、型推論に乗らないコーナーケースでもユーザーは目的の関数にたどり着ける。
- 型抽出失敗時のフォールバックが「自然劣化 = アルファベット順」になり、失敗が静かに済む。

型情報をどう確実に得るかが本筋の難しさで、その点はユーザーとの議論で次の認識に到達した:

- 診断 ([server.rs:863](../../src/commands/lsp/server.rs#L863-L872)) は **保存時にしか走らない**。仮に didChange でも走らせるとしても、コードを前から書く過程で「パース可能な状態」が必ずキャプチャされるとは限らない (`let y = a` 単独はパース不能、`let y = a.` も不能)。
- snapshot AST + `find_node_at` 系の方針は破綻する。
- 代わりに、補完リクエスト時に **ライブバッファを (必要なら) 修復してパース** → **AST 内のホールが elaborate でどんな型に推論されるかを読む** → **その型から受信者型を取り出す** が筋がよい。Fix にはすでに `Std::#hole : a` がビルトインで存在する ([src/ast/expr.rs:1656-1672](../../src/ast/expr.rs#L1656-L1672), [src/elaboration/check_holes.rs](../../src/elaboration/check_holes.rs))。
- 期待戻り値型ランキング (P3) は **廃案** — UX で混乱を招くため。受信者型でのソート (P1) のみに絞る。
- **Grammar 拡張は 1 点のみ** (詳細は後述):
  - (G1) **ユーザーが書ける hole 構文** `?x` / `?` を式位置で受け入れる。これだけで以下の 3 つの用途を全部カバー:
    - 補完挿入時に `f(?x, ?y)` 形式で入れて undefined name / unit-call 誤解釈の問題を構造的に解決。
    - repair pre-pass で「post-dot を `?` に置換」することで `obj.<cursor>` を `obj.?` 化 → そのまま通常の `expr_dot_seq` 経由で `App(hole, [obj])` としてパースされる。
    - ユーザーが mid-typing で `?` を書いて hole を意図的に残す書き方も可能 (副作用としての言語機能)。
- **検討したが採用しなかった grammar 変更**:
  - 当初検討した「dot 右辺の hole 許容 (`expr_dot_seq` の右辺を `(expr_index | hole)` に)」は、(G1) の `?x` 構文を入れたことで repair pre-pass 側の「`?` 挿入」で代替可能となり、不要に。
  - 当初検討した「`arg_list` の各 `expr` を `expr_hole` 許容に」は、補完挿入を snippet 路線 ($1, $2) ではなく `?x` 路線で行う方針にしたことで動機消失。`f(a, )` のようなユーザー mid-typing の末尾カンマは repair pre-pass で文字列レベル除去する。
  - **結果として grammar への侵襲を最小化**。`?x` 構文という *言語機能としても自然* な拡張ひとつで、補完まわりの全課題を解決する形になった。
- パーサそのものの error-recovery 化 (pest 置換) は今回射程外。tree-sitter 併用は future work として記録。

---

## 設計サマリ

### スコープ

- **P1: 受信者型でのソート** のみ実装する (フィルタはしない、すなわち全候補を返す)。
- ドット文脈 (`is_dot_function` が true) でない補完は現状の挙動を維持。
- 型抽出に失敗しても **全件をアルファベット順で返す** (= 現状の挙動と同じ) ので degrade が静か。

### スコアリング基準

ホール `h` を `obj.xxx` の `xxx` 位置に AST 上で差し替えると、`h` のスコープ上の役割は「dot-call の関数位置」になる ([grammer.pest:147](../../src/parse/grammer.pest#L147), [parser.rs:1781-1787](../../src/parse/parser.rs#L1781-L1787) より、`a.foo(b1, b2)` は `App(foo(b1)(b2), [a])` と parse される — **受信者 `a` は最後に適用される**)。

elaborate 後、`h` の推論型は

```
A1 → A2 → ... → An → Self → Ret
```

の形 (n は `obj.xxx` の後ろにユーザがすでに書いた引数の個数、n=0 の場合は単に `Self → Ret`)。

候補 c の Scheme を fresh instantiate した curried 型を `S1 → ... → Sm` (右端が戻り値) としたとき、c に **以下の Tier を割り当てる**:

- **Tier 0 (最優先)**: m ≥ n + 1 かつ、末尾から (n+1) 番目の引数 `S_{m-n-1}` (= 受信者位置) が `Self` と **完全に unify 可能**。
- **Tier 1**: 受信者位置の TyCon と `Self` の TyCon が一致 (バケットマッチだが unify までは検証しない or 失敗)。型変数等 ([toplevel_tycon](../../src/ast/types.rs#L937) で TyCon が決まらない候補) は除く。
- **Tier 2**: 受信者位置が型変数で「何にでもマッチしうる」候補 (例: `to_string : [a : ToString] a -> String` のような trait method)。
- **Tier 3 (最低)**: その他、明らかに型が合わない候補。

これらを `sort_text` の prefix として埋め込み (`"0_<name>"`, `"1_<name>"`, `"2_<name>"`, `"3_<name>"`)、LSP クライアント側のソートに任せる。クライアントの string match (打鍵中の prefix とのマッチ) は別軸で効くので、結果として「**型マッチ + 名前マッチが両方あるものが最上位**」という直感的な並びになる。

これにより `a.<cursor>` 形 (n=0) でも `a.foo(arg1, <cursor>` 形 (n=1) でも自然に動く。

#### バケットインデックス (段階1) の詳細

**目的**: 数千件の `program.global_values` を毎リクエストでフルスキャンするのを避ける。受信者の TyCon でハッシュ引きするための前計算 map。

**データ構造**:

```rust
pub struct CompletionIndex {
    program_ptr: *const Program,                // Arc::as_ptr で識別、置換検出用
    by_receiver_tycon: Map<TyCon, Vec<FullName>>,
    wildcard: Vec<FullName>,
}
```

**構築アルゴリズム**: 各 `(name, gv) in program.global_values` について:

1. `gv.scm.ty` を [`get_lambda_srcs`](../../src/ast/types.rs#L632) で繰り返し分解し、curried 型の引数列 `S1, ..., Sm` を取り出す。`m == 0` (関数でない値) ならスキップ。
2. 末尾引数 `Sm` に対して `toplevel_tycon()` ([types.rs:937 付近](../../src/ast/types.rs#L937)) を試す。
   - `Some(tc)` なら `by_receiver_tycon[tc]` に `name` を push。
   - `None` (型変数の場合) なら `wildcard` に push。
3. `compiler_defined_method: true` のシンボルや `#` 入りの名前 (内部用) は **除外しない** — struct field accessor (`@field` 等) は正当な dot 補完候補。`#` 系 (`Std::#hole` 等) は別途 [completion.rs:69](../../src/commands/lsp/completion.rs#L69) のフィルタで弾かれている。

**具体例** (Std):

| 関数 | 末尾引数の型 | TyCon | 入るバケット |
|---|---|---|---|
| `Array::get : I64 → Array a → a` | `Array a` | `Array` | `by_receiver_tycon[Array]` |
| `String::split : String → String → Iter` | `String` | `String` | `by_receiver_tycon[String]` |
| `Option::map : (a → b) → Option a → Option b` | `Option a` | `Option` | `by_receiver_tycon[Option]` |
| `ToString::to_string : [a : ToString] a → String` | `a` (型変数) | — | `wildcard` |
| `I64::to_string : I64 → String` | `I64` | `I64` | `by_receiver_tycon[I64]` |

**ライフサイクル**: 起動時には作らない。最初の dot 補完リクエスト時に lazy 構築し、`program_ptr` で現 Program との同一性を判定。Program が置き換わったら破棄して再構築。

**補完時の使い方**:

```rust
let receiver_tc = receiver_type.toplevel_tycon();
let mut tier1: Vec<FullName> = vec![];
if let Some(tc) = receiver_tc {
    if let Some(bucket) = index.by_receiver_tycon.get(&tc) {
        tier1.extend(bucket.iter().cloned());
    }
}
let tier2: Vec<FullName> = index.wildcard.clone(); // wildcard は常に対象
let tier3: Vec<FullName> = (それ以外の global_values);
```

#### Unify 段階 (段階2) の詳細

**目的**: バケット段階は最外側 TyCon しか見ないので、`Array I64` の receiver に `Array String` 専用関数が混じる。型引数まで見て **完全 unify するもの** だけを Tier 0 に昇格させる。

**アルゴリズム** (Tier 1 候補の各 `name` について):

```rust
// receiver_type は hole から取り出した Self
let mut tc = program.create_typechecker();
let scheme = program.global_values[&name].scm.clone();
let inst = tc.instantiate_scheme(&scheme);              // ∀a. ... → ∀a₁ fresh
let curried_args = collect_lambda_srcs(&inst.ty);       // 末尾は戻り値の前まで
let recv_pos_idx = curried_args.len() - 1 - n;           // n は user-supplied args 数
let recv_pos = &curried_args[recv_pos_idx];
match tc.unify(recv_pos, &receiver_type) {
    Ok(_)  => Tier::Zero,    // 完全に整合 → 昇格
    Err(_) => Tier::One,     // バケット一致だが型引数が合わず → そのまま
}
```

**具体例** (受信者 `arr : Array I64`, n = 0):

| 候補 | instantiate 後 | 受信者位置 | unify | Tier |
|---|---|---|---|---|
| `Array::get` | `I64 → Array a₁ → a₁` | `Array a₁` | 成功 (`a₁ := I64`) | 0 |
| `Array::map` | `(a₁ → b₁) → Array a₁ → Array b₁` | `Array a₁` | 成功 | 0 |
| 仮想 `Array_String_specific : Array String → String` | `Array String → String` | `Array String` | 失敗 (`I64 ≠ String`) | 1 (留まる) |
| `String::split` | `String → String → Iter` | `String` | (バケット段で除外) | 3 |

**注意点**:

- `instantiate_scheme` は **必ず候補ごとに新しい `TypeCheckContext`** で呼ぶ。連続呼び出しで型変数や substitution が混ざると誤判定する。`Program::create_typechecker` を毎回呼べばよい (既存 API)。
- 受信者位置の取り方は curried 型 `S1 → ... → Sm` の末尾から (n+1) 番目。実装としては `srcs.iter().rev().nth(n)` 形が安全。
- 候補の Scheme に trait constraint (`[a : Trait]` 部分) があっても、ここでは無視。trait check は MVP では省略 (Tier 2 で wildcard 扱いのまま)。**未解決の不確実性** に詳述。

#### Tier 判定の決め方の決定論

実装としては「全 global_values を一周し、各候補について Tier を返す」という関数を書く。優先順:

```rust
fn assign_tier(name: &FullName, gv: &GlobalValue, idx: &CompletionIndex,
               receiver_type: &TypeNode, n: usize, program: &Program) -> Tier {
    let receiver_tc = receiver_type.toplevel_tycon();
    let in_tycon_bucket = receiver_tc
        .as_ref()
        .and_then(|tc| idx.by_receiver_tycon.get(tc))
        .map(|b| b.contains(name)).unwrap_or(false);
    let in_wildcard = idx.wildcard.contains(name);

    if in_tycon_bucket {
        // unify 試行
        if try_unify_receiver(name, gv, receiver_type, n, program).is_ok() {
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

n の決め方は AST 構造から。ホールノードを含む `App` のチェーンを上に辿り、`AppSourceCodeOrderType::XDotF` がついた App の引数を 0 として、外側に `(arg)` を適用するごとに +1。詳細は [src/ast/expr.rs](../../src/ast/expr.rs) の `Expr::App` と `app_order` を参照。

### Grammar 拡張 (1点)

このプラン全体の前提として、grammar に **式位置での `?` / `?x` 構文** を追加する。これ 1 つで補完まわりの構造的課題 (unscoped 引数名挿入、unit-call 誤解釈、post-dot empty パース) を全部カバーする。

#### (G1) 式位置での `?x` / `?` 構文 — **実装済**

```pest
// 既存の `expr_hole = { expr | hole }` は `expr_or_hole` にリネーム済。
expr_hole = { "?" ~ name_char* }
expr_nlr = { expr_lit | expr_hole | expr_var | expr_let | ... }
```

意味:

- `?` 単独 = 名前無しのホール (AST では `Std::#hole`)。
- `?x` = label `x` 付き。label は parser 段階で **読み捨て** (AST には残らない)。`?x` も `?` も同じ hole ノードになる。今回はそれで十分。将来 IDE 側で label を表示したくなったら AST に aux info として持たせる選択肢が残る。
- 式位置でしか起動しない。型位置の `?a` (opaque type variable, [grammer.pest:225](../../src/parse/grammer.pest#L225)) とは context が分離されており衝突しない。

3 つの用途を兼ねる:

1. **補完挿入を `f(?x, ?y)` 形式にできる** → 引数名がそのままソースに見えつつ、型推論上はホール扱い。`x`, `y` がスコープに無くても undefined name にならず、ホール診断のみが出る。**1引数関数の補完で unit-call 誤解釈** (snippet 路線で `f($1)` を入れると source は `f()` になり Fix の unit injection ([parser.rs:1930-1935](../../src/parse/parser.rs#L1930-L1935)) で `f(unit)` として型チェックされる問題) **も発生しない**。`f(?x)` は `App(f, [hole])` 1 引数として正しく型推論される。
2. **repair pre-pass の post-dot 置換**: `obj.foo<cursor>` を `obj.?` に書き換えるだけで、通常の `expr_dot_seq` 経由で `App(hole, [obj])` としてパースされる。dot 右辺に grammar-level の hole 許容を別途入れる必要はない。
3. **ユーザーが mid-typing で `?` を書いて hole を意図的に残す**こともできる (言語機能としての副次効果)。

性質:

- **エディタ非依存**: LSP の snippet / inlay hints の特殊機能に依存しないので、どのエディタでも同じ挙動。
- **Self-documenting**: source を読むだけで「ここはまだ埋まっていない」「この位置の引数名は `x`」が分かる。
- 既存の有効プログラムへの解釈は不変 (`?` で始まる識別子は元々無い)。

シジル `?` 選定の根拠: 式位置では現在未使用 (型位置の `?a` のみで使われている)。完全な未使用シジル (`~` 等) と迷ったが、「不確定/未確定」という意味で型位置と路線が揃うという判断。

### 全体フロー

補完リクエスト到着時:

1. **ドット文脈判定**: `typing_text` を見て `is_dot_function` が真でなければ、現状の namespace ベース補完にそのまま流す。
2. **Repair pre-pass (= ソースレベル書き換え)**: ライブバッファに対して以下を順に適用:
   - **A0**: ドット直後の途中入力識別子 (例: `obj.foo<cursor>bar` の `foobar`) を `?` に置換する。`obj.foo<cursor>bar` → `obj.?` の形になる。**これが「post-dot を hole に書き換える」操作の本体**で、(G1) の `?` 構文を使うことで通常の `expr_dot_seq` でそのままパースされる。
   - 加えて、grammar で吸収しきれない構文崩れがあれば修復する (詳細は次節)。
3. **パース**: 修復済み文字列を通常のパーサに通す。`obj.?` は `App(Std::#hole, [obj])` として AST 化される。失敗したら全件フォールバック。
4. **ホールノード特定**: パース結果の AST から、カーソル位置の SourcePos を含む `Var(Std::#hole)` ノードを位置ベースで探す。`?` から生成された hole がそれ。
5. **incremental elaborate**: カーソルを含む global value だけを TypeCheckCache ([typecheckcache.rs](../../src/elaboration/typecheckcache.rs)) 経由で再 elaborate する。他の global value は前回キャッシュをそのまま使う。失敗したら全件フォールバック。`collect_hole_errors` ([check_holes.rs:25](../../src/elaboration/check_holes.rs#L25)) は **走らせない** (補完専用フローなのでスプリアス診断を出さないため)。
6. **受信者型抽出**: ホールノードの `type_` を読み、`A1 → ... → An → Self → Ret` 形に分解 ([`get_lambda_srcs`](../../src/ast/types.rs#L632) の繰り返し適用)。`Self` を取り出す。`n` は AST 構造から (App のネスト数) も判別可能。
7. **候補スコアリング**: 全候補をループし、各候補の Tier (0〜3) を判定する。
   - **段階1 (バケット)**: 起動時 / `Program` 切替時に作っておくインデックス `Map<TyCon, Vec<FullName>>` を引いて、TyCon が一致するものを Tier 1 候補として抽出 (型変数しか出てこない候補は wildcard バケットで Tier 2 候補)。
   - **段階2 (unify)**: Tier 1 候補について、`Program::create_typechecker` で context を作り、Scheme を `instantiate_scheme` → 末尾から (n+1) 番目の引数を `Self` と `unify` ([`TypeCheckContext::unify`](../../src/elaboration/typecheck.rs#L1593))。成功なら Tier 0 に昇格、失敗なら Tier 1 のまま (除外しない)。
   - インデックスにも Tier 1 にも該当しない候補は Tier 3。
8. **整形して返却**: 既存の `create_item` 経路に合わせ、`sort_text = format!("{}_{}", tier, name)` を設定。**全件を返す** (フィルタしない)。docs/import 解決は既存の `handle_completion_resolve_document` ([completion.rs:220](../../src/commands/lsp/completion.rs#L220)) のままでよい。

---

## Repair pre-pass

**目的**: 構文崩れを **カーソル位置の意味は変えずに** パース可能な形に直す。

### 必須の修復

- **A0. (常に最初に行う) post-dot を `?` に置換**: `obj.foo<cursor>bar` のように、補完対象のドット直後に途中入力された識別子があれば、ドット直後からその識別子の末尾までを `?` に置き換える。`obj.?` の形になり、(G1) の `?` 構文経由で `App(Std::#hole, [obj])` としてパースされる。**これが「post-dot をホールに書き換える」ソースレベル操作の本体**。残りの A〜D はその上で文脈の整合性を取るための補助的な修復。
- **A. 文脈別の最小補完**: 構文要素のガワが未完なまま打鍵が止まっているケースを補う。各構文要素の body / arm 位置は既に grammar 側で `expr_hole` 経由で空が許されるので、**追加するのは骨組みのトークンだけ** で、ダミー値は不要。
  - `let y = obj.<cursor>` → 末尾に `;` がなければ追加 (これで `let` 構文の `in_of_let` を満たし、本体は hole で空が許される)。
  - `if obj.<cursor>` → `{ } else { }` を補う。`if` 構文は condition 後に `{` `}` `else` 一式が必須で、これは grammar 側で hole 化されていない。body は `expr_hole` で空でよい。
  - `match obj.<cursor>` → `{ _ => }` を補う (右辺は `expr_hole` で空可)。
  - `do { ...; obj.<cursor>` → 閉じ波括弧 `}` を追加。
- **B. 未閉じ括弧の補完**: カーソルから前方に向かって `(` `[` `{` の深さをカウントし、不足分をファイル末尾 (またはカーソル位置の文末) に追加。
- **C. dangling な二項演算子の除去**: `a + <cursor>` のような operator 直後で打鍵が止まっているケース。`+`, `=`, `&&`, `||`, `<<`, `>>` 等の dangling 右辺は除去 (or 右辺に `?` を挿入)。
- **D. 末尾カンマ + 閉じ括弧の除去**: `f(arr.<cursor>, )` のような `, )` パターンは parse error になるので、`,` を除去して `f(arr.<cursor>)` にする。同様に `[a, ]` `(a, )` 等の末尾カンマも除去。

### 実装方針

- repair pre-pass は **ライブバッファの文字列を加工する関数** として独立に書く。入力 = (ライブバッファ, カーソル位置)、出力 = 修復済みバッファ。
- 修復順序: A0 (post-dot を `?` に置換) → D (末尾カンマ除去) → C (dangling 演算子除去) → B (未閉じ括弧補完) → A (構文単位のガワ補完)。
- パース失敗時に「もう一段の修復を試す」ループは持たない (実装複雑化を避ける)。一発で通らなければフォールバック。
- カバレッジは経験的に育てる前提。テストケースで頻出パターンを増やしていく。

### A の実装戦略 (`pest error driven` → `structure tracker`)

A (構文単位のガワ補完) はとくに「どんなトークンを足せばよいか」をどう判定するかが課題。検討した中で実用に乗りそうな 2 案を **段階的に試す**:

#### 段階1: pest のエラー情報による driven repair

パース失敗時、pest の `Error` は **位置** と **expected ルール** を返すので、それを使って次に何を追加すべきかを決める。

```rust
let mut source = repaired_after_a0;
for _attempt in 0..MAX_ATTEMPTS { // 例: 5 回
    match parse(&source) {
        Ok(_) => return Some(source),
        Err(e) => {
            let pos = e.location;
            let token = guess_token_from_expected(&e.expected); // ";", "}", "{}else{}" 等
            if token.is_none() { return None; }
            source.insert_str(pos, &token.unwrap());
        }
    }
}
return None;
```

- **長所**: 数十行で書ける。pest の機構をそのまま使うので Fix の grammar 変更にも自動追従。
- **短所**: pest のエラーメッセージは PEG バックトラックの結果、必ずしも「ユーザー意図的に直すべきトークン」を指していない。`expected expression` のような曖昧なエラーが多い。
- **適性**: MVP として最初に書く。これで救えるケースを観測して、足りなければ段階2 へ。

#### 段階2: lightweight tokenizer + 構造スタックマシン

段階1 で取りこぼすケースが目立ったら、構造的に正確な分析に切り替える。アルゴリズム:

**(a) Lightweight tokenizer (80-100 行)**

完全な lexer ではなく、構造的に意味のあるトークンだけ識別:

- キーワード: `let` / `if` / `else` / `match` / `do` / `eval` / `in`
- 記号: `(` `)` `[` `]` `{` `}` `=` `;` `,` `=>` `|`
- 文字列リテラル `"..."` とコメント `//...` `/*...*/` は skip (内部の `{` を誤検出しないため)
- 識別子・リテラル・その他は stream を消費するだけで `None` 扱い

pest の lexer rule を流用するか、独立に hand-written するかは実装時判断。

**(b) 開いた構造のスタック (50 行)**

```rust
enum OpenConstruct {
    Let(LetState),     // SeenLet → SeenPat → SeenEq → SeenValue
    If(IfState),       // SeenIf → SeenCond → SeenThenBody → SeenRBraceThen → SeenElseKw → ...
    Match(MatchState), // SeenMatch → SeenScrutinee → SeenLBrace → InArm → ...
    Do(DoState),       // SeenDo → SeenLBrace → InBody
    Eval(EvalState),   // SeenEval → SeenExpr (待 `;`)
    Lambda(LambdaState), // SeenBar → InArgs → SeenBar2 → InBody
    Paren,             // ( ... )
    Bracket,           // [ ... ]
    Brace(Option<BraceOwner>), // { ... } (owner は紐付いている上位構造)
}
```

**(c) Forward 走査 (100-150 行)**

カーソル位置までトークンを順に処理し、stack を更新:

```rust
match tok {
    Token::Let   => stack.push(Let(SeenLet)),
    Token::If    => stack.push(If(SeenIf)),
    Token::Eq    => /* 直近の Let が SeenPat なら SeenEq へ */,
    Token::Semi  => /* 直近の Let/Eval を pop。Brace 越えはしない */,
    Token::LBrace => {
        let owner = top_expects_brace(); // If/Match/Do なら紐付け
        stack.push(Brace(owner));
        // owner の状態を遷移 (例: If(SeenCond) → If(SeenThenBody))
    }
    Token::RBrace => {
        stack.pop(); // Brace を閉じる。owner があればその状態を遷移
    }
    // ... 他の遷移規則
    _ => {} // 識別子等は構造に影響しない
}
```

**(d) Closures 生成 (50-80 行)**

カーソル時点の stack を **innermost (= 末尾) から順に閉じる**。**stack を pop しながら上位構造の状態も逐次遷移させる** 二段構え:

```rust
fn close_remaining(&self) -> String {
    let mut s = String::new();
    let mut stack = self.stack.clone();
    while let Some(c) = stack.pop() {
        match c {
            Let(SeenEq | SeenValue) => s.push(';'),
            If(SeenIf | SeenCond)   => s.push_str(" {} else {}"),
            If(SeenThenBody)        => s.push_str("} else {}"),
            If(SeenRBraceThen)      => s.push_str(" else {}"),
            If(SeenElseKw)          => s.push_str(" {}"),
            Match(SeenMatch | SeenScrutinee) => s.push_str(" { _ => }"),
            Match(SeenLBrace)       => s.push_str(" _ => }"),
            Do(SeenDo)              => s.push_str(" { }"),
            Do(SeenLBrace | InBody) => s.push('}'),
            Eval(_)                 => s.push(';'),
            Paren                   => s.push(')'),
            Bracket                 => s.push(']'),
            Brace(Some(owner)) => {
                s.push('}');
                advance_owner_state(&mut stack, owner); // ← 重要
            }
            Brace(None) => s.push('}'),
            // ...
        }
    }
    s
}
```

**ワークドエグザンプル**: 入力 `if c { let x = obj.?` のとき:

| トークン | アクション | スタック (上が outer) |
|---|---|---|
| `if` | push If(SeenIf) | `[If(SeenIf)]` |
| `c` | If: SeenCond へ遷移 | `[If(SeenCond)]` |
| `{` | Brace push、If を SeenThenBody へ | `[If(SeenThenBody), Brace(If)]` |
| `let` | push Let(SeenLet) | `[..., Let(SeenLet)]` |
| `x` | Let: SeenPat へ | `[..., Let(SeenPat)]` |
| `=` | Let: SeenEq へ | `[..., Let(SeenEq)]` |
| `obj`, `.`, `?` | 影響なし | `[..., Let(SeenEq)]` |

カーソル時点 stack: `[If(SeenThenBody), Brace(If), Let(SeenEq)]`。closures 生成:

1. Pop `Let(SeenEq)` → `;` 出力
2. Pop `Brace(If)` → `}` 出力。If の状態を SeenThenBody → SeenRBraceThen に遷移
3. Pop `If(SeenRBraceThen)` → ` else {}` 出力

最終文字列: `if c { let x = obj.?;} else {}` → parseable。

**実装規模**: 合計 400-500 行程度 (tokenizer + 構造定義 + 走査 + closures + テスト)。

#### ハマりどころ (段階2)

- **`;` の解釈**: do ブロック内の文末セパレータ vs. let/eval 終端。Brace が間にあれば「文末 sep」、無ければ「let/eval 終端」と判別。
- **`{` の用途**: if/match/do の body / struct literal `Foo { ... }` / 単独ブロック。直前トークンが大文字始まり識別子なら struct lit と判別。
- **lambda の `|`**: Fix では `|` の単独使用は無いはずだが、要確認 (二項演算子と衝突しないか)。
- **カーソル位置の正確な扱い**: カーソルが文字列やコメント内なら skip。
- **post-cursor の扱い**: カーソルが文末でない場合、**後ろを捨てる** のが現実的 (補完目的では post-cursor の elaborate は不要)。

#### 段階分け

1. まず段階1 (pest error driven) で実装。これだけで MVP に十分なケースは救えるはず。
2. 経験的に段階1 で取りこぼすパターンを観測 (テストケースで明確化) → 段階2 (構造トラッカ) に置換。
3. 段階2 を入れた後も、エッジケースは出続ける。それは「フォールバックで全件返す」で許容。

### 既知の限界

- **遠方の壊れ**: カーソルから離れた場所が壊れていると、修復対象がそこでないため救えない。これは TypeCheckCache 経由の incremental elaborate で「壊れた global value 以外は再利用」できれば緩和されるが、name resolution が全体に依存している場合は影響が出る (要調査)。
- **複合的な崩れ**: 複数の A 系修復が同時に必要なケースなどはカバー外になる可能性。段階2 の構造トラッカで innermost-first に閉じればある程度は対応できる。
- **誤った修復**: 修復が「ユーザの真の意図とは違う AST」を作ってしまうと、受信者型が誤って推論される可能性。ただし「型が誤る」ケースは「補完候補がやや変」になるだけで、致命的ではない。
- **段階1 の pest エラーの曖昧さ**: PEG のバックトラック起源で、`expected expression` 等の曖昧なエラーから「何を入れるべきか」を機械的に判定できないケースがある。これが段階2 への移行動機になる。

---

## 実装上の注意

### ホールノード特定

- repair pre-pass の A0 で挿入した `?` が、パース時点で `Var(Std::#hole)` ノードとして AST に現れる。AST 書き換えは不要。
- カーソル位置の SourcePos を含む `Var(Std::#hole)` ノードを位置ベースで探す。
- 候補が複数ある (例: 他の修復や、ユーザーが別箇所に既に書いた `?` がある) 場合は、**カーソル位置に最も近い** ものを選ぶ。
- AST 上に位置整合する hole が見つからないときは全件フォールバック。

### incremental elaborate

- `Program::create_typechecker` を使い、当該 global value の AST だけ elaborate する経路を組む。
- TypeCheckCache はシンボル単位のキャッシュ ([typecheckcache.rs:14](../../src/elaboration/typecheckcache.rs#L14))。補完対象 global value の **シグネチャや let-binding 構造が変わっている場合**、その global value のキャッシュは破棄して再計算 (これは正しい挙動)。
- `collect_hole_errors` を呼ばない経路を作る必要がある。既存の elaborate driver を流用するなら、フラグを追加するか別関数として分ける。

### バケットインデックス

- `Program` が新しいものに置き換わったら作り直す。`Arc::ptr_eq` で検出するか、`Program` に世代カウンタを持たせる。
- インデックスは Std のような不変部分のみキャッシュ、ユーザコード部分は毎回再計算、という分割もありうる (最適化として後回しでよい)。

### コンパイラ自動生成メソッドの扱い

- struct field accessor (`@field`, `set_field`, `mod_field`, `act_field`) や union variant accessor は `program.global_values` に `compiler_defined_method: true` で登録 ([program.rs:2145](../../src/ast/program.rs#L2145))。これらは **正当な補完候補**として残す。
- 現状 [completion.rs:69](../../src/commands/lsp/completion.rs#L69) の `'#'` フィルタは内部用シンボルを弾く目的なので、自動生成メソッドが弾かれていないことを確認 (= 自動生成メソッドの FullName は `#` を含まないはず)。

---

## 主な変更ファイル

| ファイル | 変更内容 |
|---|---|
| [src/parse/grammer.pest](../../src/parse/grammer.pest) | (G1) `expr_hole_user = "?" ~ (name_head ~ name_char*)?` を追加し `expr_nlr` の選択肢に組み込む。 |
| [src/parse/parser.rs](../../src/parse/parser.rs) | grammar 変更に追従。`expr_nlr` の dispatch ([parser.rs:1951-1968](../../src/parse/parser.rs#L1951-L1968)) に `expr_hole_user` の handler を追加し、`expr_hole(span)` ([src/ast/expr.rs:1670](../../src/ast/expr.rs#L1670)) を返す。label は AST ノードの aux info として保持するか、もしくは label 抽象を諦めて単なる `Std::#hole` として扱うかは実装時に決定。 |
| [src/commands/lsp/completion.rs](../../src/commands/lsp/completion.rs) | `handle_completion` をドット文脈で分岐。新フローのドライバを追加。挿入テキストを `params.iter().map(\|p\| format!("?{}", p)).collect::<Vec<_>>().join(", ")` ベースに変更 ([completion.rs:262-277](../../src/commands/lsp/completion.rs#L262-L277))。 `sort_text` を Tier-prefix で設定。 |
| 新ファイル `src/commands/lsp/completion_repair.rs` | repair pre-pass の実装。 |
| 新ファイル `src/commands/lsp/completion_index.rs` | バケットインデックスの構築・キャッシュ。 |
| 新ファイル `src/commands/lsp/completion_score.rs` | 受信者型に基づく候補スコアリング (バケット引き + unify による Tier 判定)。 |
| [src/elaboration/](../../src/elaboration/) | 単一 global value の elaborate ドライバ (補完用、`collect_hole_errors` 抜き) を追加。既存 driver を再利用しつつ extension point を切る形が望ましい。 |

既存で再利用するもの:

| 既存 API | 場所 | 用途 |
|---|---|---|
| `expr_hole` | [src/ast/expr.rs:1670](../../src/ast/expr.rs#L1670) | AST 上のホール挿入 |
| `hole_full_name` | [src/ast/expr.rs:1657](../../src/ast/expr.rs#L1657) | ホール識別子 |
| `Std::#hole : a` ビルトイン | [src/fixstd/builtin.rs:4163](../../src/fixstd/builtin.rs#L4163) | 多相プレースホルダ |
| `Program::create_typechecker` | [src/ast/program.rs](../../src/ast/program.rs) | unify 用 context |
| `TypeCheckContext::instantiate_scheme` | [src/elaboration/typecheck.rs](../../src/elaboration/typecheck.rs) | 候補 Scheme の fresh 化 |
| `TypeCheckContext::unify` | [src/elaboration/typecheck.rs:1593](../../src/elaboration/typecheck.rs#L1593) | 受信者位置の単一化 |
| `TypeNode::get_lambda_srcs` / `get_lambda_dst` | [src/ast/types.rs:632](../../src/ast/types.rs#L632) / [645](../../src/ast/types.rs#L645) | curried 型の分解 |
| `TypeNode::toplevel_tycon` | [src/ast/types.rs:937 付近](../../src/ast/types.rs#L937) | バケットキー |
| `TypeCheckCache` / `MemoryCache` | [src/elaboration/typecheckcache.rs](../../src/elaboration/typecheckcache.rs) | incremental elaborate |
| `is_dot_function` | [src/commands/lsp/completion.rs:145](../../src/commands/lsp/completion.rs#L145) | ドット文脈判定 |

---

## 検証方法

### Grammar 拡張 (G1) の検証 (先行)

- (G1) を入れた直後に `cargo test --release` で全テスト通し。失敗するテストの予想:
  - `?` で始まる識別子を扱うパターンがあれば見直し (現在 `?` は型位置のみで使われていることを再確認)。
  - パースエラーメッセージのスナップショットがあれば文言を確認。
- `?`, `?x` を式位置で書くテストを新規追加 (例: `let y = ?; y` が `Std::#hole` で parse され、hole error が出ること)。

### 補完機能の検証

- `src/tests/test_lsp/cases/completion/` の既存サンプル ([lib.fix](../../src/tests/test_lsp/cases/completion/lib.fix), [main.fix](../../src/tests/test_lsp/cases/completion/main.fix)) に dot-completion 用ケースを追加。各ケースは「**期待する候補が Tier 0 (= sort_text の prefix が `0_`) で返ること**」を assert する形 (= 全件は引き続き返るが、上位に並ぶことを確認)。最低限カバーするケース:
  - **基本**: `let s = MyStruct { ... }; s.<cursor>` → `s` の field accessor / 関連メソッドが Tier 0 になる。
  - **チェーン**: `let arr = [1,2,3]; arr.<cursor>` → `Array I64` 用メソッドが Tier 0 になる。
  - **途中引数**: `let arr = [1,2,3]; arr.fold(0, <cursor>)` → 第2引数位置で受信者型が `Array I64` と整合する候補が Tier 0 になる (n=1 ケース)。
  - **末尾カンマ**: `f(arr.<cursor>, )` → repair pre-pass D で `,` 除去後にパース成功し、`Array I64` 用メソッドが Tier 0 になる。
  - **let 末尾**: ファイル末尾が `let y = arr.<cursor>` で終わるケース → repair で `;` 補完。
  - **if / match 内部**: `if arr.<cursor> { ... }` → repair で本体補完。
  - **遠方の壊れ**: 別の global value が壊れていてもカーソル位置の補完は機能する (TypeCheckCache の効きを確認)。
  - **型抽出失敗時のフォールバック**: 型情報が取れないケースでも全件アルファベット順で返ること。
  - **挿入形式**: 候補 `f : Int -> String -> X` を選択した結果が `f(?x, ?y)` 形式 (引数名は doc 由来) で、source に直接 `x`, `y` が入らないこと。1引数関数 `g : Int -> X` でも `g(?x)` で挿入され `g()` (unit-call) にはならないこと。
- [src/tests/test_lsp/test_completion.rs](../../src/tests/test_lsp/test_completion.rs) にこれらを追加。
- CLAUDE.md の方針に従い `cargo test --release` で実行。

---

## 実装順序 (チェックリスト)

0. ~~**Step -1 (補完機能の現状回帰テスト)**~~ — 完了 ([commit ea1b5740](../../)). `completion_insert` フィクスチャと `test_completion_insert_text_for_function_with_two_params` を追加。
1. ~~**Step 0 (Grammar 拡張・先行)**~~ — 完了 ([commit ec05189c](../../)). `expr_hole = { "?" ~ name_char* }` 追加、既存 `expr_hole` を `expr_or_hole` にリネーム。`parse_expr_hole` 追加。`hole_user_*` テスト 5 件追加。
2. ~~**Step 0.5 (補完挿入の `?x` 化)**~~ — 完了 ([commit ec05189c](../../)). [completion.rs:282-296](../../src/commands/lsp/completion.rs) で `params.iter().map(|p| format!("?{}", p))` に書き換え済。
3. **Step 1 (受信者型抽出 prototype)**: `let x = obj.<cursor>` の最小ケースに限定して end-to-end で動かす。
   - **入力**: 補完リクエスト時のライブバッファとカーソル位置。
   - **処理**: A0 (post-dot を `?` 化) のみ適用 → ライブバッファをパース → cursor 位置を含む `Var(Std::#hole)` ノードを位置で特定 → 該当 global value だけを elaborate (この時点では full elaborate でよい、incremental は Step 5) → ホールの `type_` から受信者型を取り出す。
   - **副産物**: ホールの推論型を `A1 → ... → An → Self → Ret` に分解する `decompose_hole_type` ヘルパを書く。
   - **判定**: 取れたら debug log に出すだけで OK。LSP 応答にはまだ反映しない。
4. **Step 2 (バケットインデックス + Tier 付与)**: 上記の「バケットインデックス」節に従って `CompletionIndex` を構築。Step 1 で取れた受信者型を使って各候補に Tier 1/2/3 を付け、`sort_text = format!("{}_{}", tier, name)` を completion item に設定。**Tier 0 は次の Step で**。
5. **Step 3 (unify 段階で Tier 1 → Tier 0 昇格)**: 上記の「Unify 段階」節のアルゴリズムを実装。`Array I64` の receiver に対し `Array I64` 用は Tier 0、`Array String` 用は Tier 1 のまま、ということを統合テストで確認。
6. **Step 4 (repair pre-pass の拡張)**:
   - **Step 4-a**: A0 / B / C / D を実装 (post-dot 置換 / 未閉じ括弧 / dangling 演算子 / 末尾カンマ)。これらは構造的に独立で実装容易。
   - **Step 4-b**: A (構文単位のガワ補完) を **段階1: pest error driven repair** で実装。テストで救えるケースを確認。
   - **Step 4-c**: 段階1 で取りこぼすパターンが目立てば **段階2: 構造スタックマシン** に置換。実装規模 400-500 行 (詳細は「A の実装戦略」節)。
7. **Step 5 (incremental elaborate)**: Step 1 で full elaborate していた箇所を TypeCheckCache 経由の単一 global value だけの再 elaborate に切り替え。`Program::create_typechecker` + `unify_type_of_expr` で組む経路を新設するか、`run_diagnostics` をオプション付きで呼べるようにするか、設計選択は実装時に。性能を測定。
8. **Step 6 (テスト追加)**: 検証セクションに挙げた全ケースをカバー。
9. **Step 7 (性能測定)**: Std + 中規模ユーザコードで体感速度を確認。問題なければクローズ。

---

## 未解決の不確実性 (実装着手後に解消すべき)

- **trait method の第1引数バケット**: trait method (例: `to_string : [a : ToString] a -> String`) は第1引数も末尾も型変数。バケットでは wildcard (Tier 2) に入る。sort 路線では「受信者が ToString instance なら Tier 0 に昇格する」を `predicates` + `TraitEnv::instances_of(trait_id)` 相当のルックアップで実現できると更に精度が上がる。filter 路線と違い、ここを誤って Tier 2 のままにしても候補が消えるわけではないので、初期実装は wildcard 扱いで十分。
- **incremental elaborate の単位**: 現状 `run_diagnostics` は全ファイルを elaborate する経路 ([server.rs:1095](../../src/commands/lsp/server.rs#L1095))。「単一 global value だけ elaborate」する API を新設するのか、`run_diagnostics` をオプション付きで呼ぶのか、設計選択が必要。
- **repair の網羅性**: どのパターンまでカバーすれば実用に足りるかは経験的。最初は最小セットで出して、テストケースを増やす形で育てる。
- **ホール位置整合**: A 系修復で挿入したガワトークン (例: `if c { } else { }` の空 body 部) が、補完対象でない別の hole として AST に現れないか確認。**カーソル位置の SourcePos を含む** hole ノードだけを取るロジックで弾く想定だが、修復が複数の hole を生むケース (例: 末尾カンマ修復で wildcard hole も併せて出来る) では「最も近い hole」の選び方を実装で決める必要がある。
- **`is_dot_function` の頑健性**: 数値リテラル `1.foo` のような誤検出が起きないか確認 ([number_lit_body_dec](../../src/parse/grammer.pest#L48) は `.` の後に数字が必須なので `1.foo` は数値ではないが、`is_dot_function` の文字列スキャンは数値かどうかを区別しない。`1.<cursor>` のとき受信者は `1 : I64`、ドット補完が `I64`-receiving メソッドを上位にする — これはむしろ望ましい挙動だが、ユーザーの意図と一致するかは要確認)。
- **`obj.foo<cursor>` で `foo` が実在の symbol だった場合**: A0 で `foo` を `?` に書き換えてしまうと、ユーザーの「すでに正しい method 名を打鍵済み」という意図と乖離する可能性。挙動として「既存 symbol の prefix に一致しているなら書き換えない」ヒューリスティックを足すか、常に書き換えるか、判断が必要。常に書き換えても LSP client の string-match で `foo` で絞られるので大きな問題にはならない可能性が高い。
- **A0 の partial identifier 検出範囲**: ドット直後にカーソルがある場合は明確だが、`obj.foo<cursor>bar` のようにカーソルが識別子の中ほどにある場合に「どこまでを `?` で置換するか」の判定。LSP client は通常「カーソル前の prefix」と「カーソル後の suffix」を区別して扱うので、suffix 側 (`bar`) を残すか消すかは UX 判断。**第一候補: ドット直後から識別子末尾までを丸ごと `?` に**。後で UX 改善が必要になれば revise。

---

## Future work (今回は射程外)

- **tree-sitter 併用**: LSP 側の構文解析を tree-sitter で error-tolerant に行い、repair の精度を上げる。コンパイラ本体は pest のまま。`hover` / `goto-def` / `diagnostics` の即時性向上にも効く投資。
- **chumsky への全面置換**: コンパイラ・LSP 両方で error-recovery 可能なパーサを採用する大型プロジェクト。長期的には魅力的だが今回はスコープ外。
- **期待戻り値型を活用した補完**: P3 を再検討する場合は UI 上の挙動を含めて議論する必要がある。
- **didChange 駆動の incremental diagnostics**: 現状診断は保存時のみ走る。didChange でも軽量に走らせるには差分単位の elaborate が要るが、本プランの incremental elaborate (TypeCheckCache) と路線が一致するので、補完を入れた後の自然な拡張になる。
- **grammar の hole 許容をさらに広げる**: 例えば operator 直後 (`a + <hole>`) を grammar で吸収すれば、repair pre-pass の C (dangling 演算子除去) も消える。今回入れる (G1) と同じ路線の延長。
- **`?x` の label 表示の高度化**: 補完挿入後の `?x` `?y` を、IDE 上で「タブで順に移動できるプレースホルダ」として扱う統合。LSP の SemanticTokens / Inlay Hints との組合せで、`?x` を視覚的に強調する等の改善余地。本プランの範囲外だが、(G1) を入れた後の自然な拡張。
- **strict filter の選択肢化**: sort-only 方式は性能上 (毎リクエストで全件返す) のコストがある。「Tier 0/1 のみ返却」を `isIncomplete: true` 付きで返す strict-filter モードを設定で切り替えられるようにすれば、大規模プロジェクトで負荷を下げられる。経験的に必要になれば検討。

---

## 次の実装者向け参考情報

このプランの会話中で繰り返し出てきた前提や、Fix の癖を実装着手時に思い出すための要点メモ。

### Fix 言語の dot syntax 仕様 (重要)

`a.foo(b1, b2)` の parse 結果は `App(foo(b1)(b2), [a])` で、**受信者 `a` は最後に curry 適用される**。これは多くの言語と逆 (Rust / Swift / Kotlin は `a` が第1引数になる)。Plan 全体の Tier 判定はこの「**受信者 = 末尾引数**」前提で書かれている。具体的なパース構造は [parser.rs:1781-1787](../../src/parse/parser.rs#L1781-L1787) と [grammer.pest:147](../../src/parse/grammer.pest#L147) を読むのが速い。

### 診断のタイミング

- `program` (型推論結果) は **保存時にしか更新されない** ([server.rs:863-872](../../src/commands/lsp/server.rs#L863-L872))。
- ライブバッファ (`LatestContent`) は didChange ごとに更新される。
- 補完リクエスト時に snapshot AST を引いても、ユーザーが今書いている行はそこに無い可能性が高い。だから本プランは **リクエスト時にライブバッファを再パース + 再 elaborate** する設計。snapshot は当てにしない。

### `Std::#hole` は user-typeable ではない

内部用のシンボル名は `#` を含む (e.g., `Std::#hole`, `Std::#tuple0`)。grammar の `name_head` は `#` を許さないので **ユーザーは書けない**。本プランで追加した user-writable hole は `?x` という別構文。両者は AST 段階で同じ `Std::#hole` ノードに帰着する。

### 既存の hole 機構

- `Std::#hole : a` ビルトイン: [src/fixstd/builtin.rs:4163 付近](../../src/fixstd/builtin.rs)
- AST ノード作成関数 `expr_hole(span)`: [src/ast/expr.rs:1670](../../src/ast/expr.rs#L1670) ← grammar rule の名前と被っているが Rust 関数。本プランの `parse_expr_hole` でもこれを呼んでホールノードを作る。
- ホール診断: [`collect_hole_errors`](../../src/elaboration/check_holes.rs#L25) — ERR_HOLE を吐く。**補完専用フローではこれを呼ばない** (スプリアス診断回避)。
- ホールの推論型: 普通の elaborate を通すと、ホール `: a` が周囲の expected 型と単一化されて、`hole.type_` に決定的な型が入る。これが本プランのキーアイデア。

### type_env / trait_env の用語

- `program.type_env.kinds()`: 全 TyCon (型コンストラクタ) の列挙。`I64`, `Array`, ユーザ定義の struct/union 等。
- `program.trait_env.assoc_ty_kind_info()`: 全関連型 (`Iter::Item` 等)。
- `program.traits_with_aliases()`: 全 trait + trait alias。
- `program.global_values`: グローバル値。**trait method もここに集約されている** ([program.rs:1666-1678](../../src/ast/program.rs#L1666))。dot 補完の主な対象はこのマップ。

### TypeNode の便利メソッド

- `get_lambda_srcs() / get_lambda_dst()`: curried 関数型の引数列・戻り値を取り出す ([types.rs:632, 645](../../src/ast/types.rs#L632))。**panic する** ので関数型でないときは `is_funptr() / is_closure()` で先にチェック。
- `toplevel_tycon()`: TyApp ネストの最外側 TyCon を取り出す。型変数なら `None`。バケットインデックスのキー作りに使う。

### 既に書かれているテストフィクスチャの場所

- [src/tests/test_lsp/cases/completion/](../../src/tests/test_lsp/cases/completion/) — namespace ベースの基本補完テスト
- [src/tests/test_lsp/cases/completion_insert/](../../src/tests/test_lsp/cases/completion_insert/) — `Hoge::func` の dot 挿入挙動
- [src/tests/test_lsp/cases/completion_deprecated/](../../src/tests/test_lsp/cases/completion_deprecated/) — deprecation 反映

新しいケース (受信者型ベースの並び替え検証) は **新フィクスチャを足す** のが行番号干渉を避ける良策。Step 6 で追加する。

### LSP テストヘルパ

- [src/tests/test_lsp/lsp_client.rs](../../src/tests/test_lsp/lsp_client.rs): LSP client (request/response, diagnostic, progress) を扱う薄いラッパ。
- [src/tests/test_lsp/test_completion.rs](../../src/tests/test_lsp/test_completion.rs) の `LspCompletionCtx`: 補完テスト用 setup ラッパ。`complete()` / `resolve()` が使える。新ケースはこれを再利用。
- `cargo test --release` で全 LSP テスト約 1 分弱。debug ビルドだと遅いので必ず `--release`。
