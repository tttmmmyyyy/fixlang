# Fix RC 中間言語（RC IR）と一意性チェック除去（unique-check-elim）

ステータス: 設計のみ・未実装。

RC（参照カウント）最適化の基盤として **RC IR**（評価順を固定した ANF ＋ 明示 retain/release ＋ ローカル名グローバル一意）を導入し、その上で uniqueness 解析・unique-check-elim・将来の RC 最適化（retain/release 相殺・reuse・borrow・順序スケジューリング）を行う。

用語: 各値の uniqueness 表現＝`UniqTree`（leaf の格子＝`Uniqueness`（`Unique`/`Shared`））、関数ごとの要約＝`UniqSignature`。

## 0. 動機（なぜ RC IR か）

現状 Fix は retain/release を codegen（`generator.rs`）で**暗黙に**挿入する: `Scope.used_later`（377 付近）＋ `scope_lock_as_used_later`/`unlock`（731/740、評価順に沿って増減）、`get_scoped_obj`（709）が used_later なら `build_retain`、scope 退出で `release`（1462）。

AST レベルで RC 最適化をやると、この **last-use 解析＋RC 挿入を再導出（重複）**することになり（AST 版 plan の `received`/`shareize`/disposition 機械がまさにそれ）、codegen との同期が崩れると不健全になる。

解決: **codegen から last-use 解析＋retain/release 挿入を分離して「IR 生成」にし**、明示 RC を持つ RC IR を作る。すると:
- codegen も最適化も「明示 RC を読む」だけ（ロジック単一化・同期問題の消滅）。
- uniqueness 解析は RC IR を **interpret** する素直な処理になる。
- retain/release 相殺・reuse・borrow・順序スケジューリングも同じ IR 上に乗る。

先行例: Koka の Perceus、Lean 4 の RC IR、Swift の SIL。

当面のゴール: 配列等の force-unique チェック除去でベンチ競争力（下表）。将来: 上記の各 RC 最適化。

### 効果の目安（検証ターゲット, `/home/maruyama/fix-bench/batch/arrayrw*`, cachegrind work-only）

| 版 | C比 inst | 命令/要素 |
|---|--:|--:|
| 両チェックあり（現状） | 5.33x | 16 |
| 境界のみ off（`--no-runtime-check`） | 4.33x | 13 |
| 両チェック off（unchecked 直書き） | 0.20x | 0.6 |

C 超え（0.20x、ベクトル化）には一意性チェックと境界チェックの**両方**の除去が要る。本作業は一意性チェック側。境界チェック除去は将来（§6）。

## 1. RC IR 設計

### 1.1 不変条件
1. **ANF**: compound op の引数は atom（変数/定数）。compound は `let` 束縛。評価順は構文（let 逐次）で固定。未順序の兄弟評価位置は存在しない（全部 atom）。
2. **明示 retain/release**: dup/drop が明示ノード。codegen は RC 判断をしない。
3. **ローカル名グローバル一意**: 全束縛変数名が一意（シャドー禁止、スコープ外の名前との衝突も禁止）。**RC IR lowering が変数生成時に fresh なグローバル一意名を発番**する（名前カウンタ＋AST名→新名の env で traverse、AST のシャドーも解消）＝**構築により一意**。**全 RC IR 変換が保存**（clone・特殊化時も fresh 名を発番）。→ 名前→束縛が一意に解決でき、解析はスコープ追跡不要・env が単純。（別途 AST パス `unique_local_names` を走らせる必要はない＝新名を直接生成するため。）
4. **順序＝データ依存**: IO 順序は IOState threading のデータ依存（`IO a = IOState -> (IOState, a)`、bind が前段出力 iostate を次段へ）として表現され、lowering 後も保たれる。`unsafe_perform` 等の unsafe 脱出は契約上もとから順序保証外（責務外）。
5. **ソースロケーション保持**: 各 RC IR ノードは元 AST の source span（`ExprNode.source` 相当）を保持し、lowering で伝播する。lowering が新設するノード（`Retain`/`Release` など元 AST に対応式が無いもの）は、対象値や囲む式の span を継ぐ。→ codegen が DWARF debug location を正しく出せ、デバッガの行/関数対応・backtrace・サニタイザ報告・`create_debug_subprogram` 等が現状どおり機能する。特殊化（clone）・相殺・unique-check-elim など RC IR 上の全変換も span を保存する。

### 1.2 データ型（たたき台。P1 で確定）

**継続入れ子・単一 enum（終端子つき）**形式を採る。`Let`/`Retain`/`Release`/`MarkGlobal`/`MarkThreaded` は継続を持つ「文」的ノード、`Ret`/`Match` が終端子（継続の終わり）。**Lean 4 の `FnBody`（`inc`/`dec` が継続つき、`ret`/`case` が終端子）と同形**で、reset/reuse・borrow 等の RC 最適化を載せた実績がある。Fix の既存 AST（再帰的）とも同形で流用しやすい。各ノードは span を持つ（§1.1-5。下では省略）。
```rust
// 関数 = (params: Vec<Var>, body: RcExpr)
enum RcExpr {
    Let(Var, RcOp, Box<RcExpr>),          // op の結果を Var（単一）に束縛 → 継続（ANF）
    Retain(Var, RcMode, Box<RcExpr>),     // x を dup（root +1） → 継続（Lean の inc）。RcMode で state ディスパッチを選択
    Release(Var, RcMode, Box<RcExpr>),    // x を drop（-1; 0 で解放、構造辿り） → 継続（Lean の dec）
    MarkGlobal(Var, Box<RcExpr>),          // x の到達グラフを GLOBAL state に（= mark_global）。global 値初期化で発行 → 継続
    MarkThreaded(Var, Box<RcExpr>),       // x の refcount を threaded(atomic) mode に（= mark_threaded）。別スレッド送信前 → 継続
    Ret(Atom),                            // 終端子
    Match(Atom, Vec<MatchArm>),           // 終端子。唯一の分岐構造（Bool もここ）
}
struct MatchArm { variant: usize, payload: Var, body: RcExpr }  // payload を単一 Var に束縛（さらなる分解は getter で）
enum RcOp {
    App(Atom /*callee: closure か funptr の値*/, Vec<Atom>),  // closure 呼び出し・直接 funptr 呼び出し両方（codegen が型で振り分け）
    Closure(FuncRef, Vec<Atom> /*捕捉*/),  // top-level 関数 + 捕捉変数列 → boxed closure 値（root unique、捕捉を保持）
    Llvm(InlineLLVM, Vec<Atom>),          // set/mod/算術/union_new/... ＋ フィールド/variant payload/配列要素 の射影（getter）も含む。特別扱いしない
    Construct(Ctor, Vec<Atom>),           // MakeStruct（タプルは MakeStruct の特殊系）／ArrayLit → 新規 alloc（boxed なら root unique）。union 構築は `union_new`＝`Llvm`
}
enum Atom { Var(Var), Lit(Literal) }     // Var は global funptr 参照も含む
enum RcMode {            // retain/release の state ディスパッチ。lowering は既定 Unknown（健全）、§6 の state 推論が特殊化（mode を注釈するだけ＝構造変更なし）
    Unknown,             // runtime で refcnt_state を見て 3-way（= 現状の retain/release）
    Local,               // LOCAL 確定: 非 atomic inc/dec、state チェック省略
    Threaded,            // THREADED 確定: atomic inc/dec、state チェック省略
    Global,              // GLOBAL 確定: codegen で no-op（コードを出さない）。最小化したければ後段 cleanup で削除可
}
```
**分岐は `Match` のみ（`If` を持たない）**: Bool を union 化する（std.fix: `type Bool = unbox union { _false : (), _true : () }; true = _true(); false = _false();`）。ソースの `if`/`true`/`false`/比較演算子は不変で、AST→RC IR 生成で `Expr::If(c,t,e)` を `Match(c, [_false => e, _true => t])` に desugar するだけ。性能中立（Bool-union ＝ `{i8 tag, [i8;0]}` ＝ i8。比較演算子は今も i8(0/1) を返す＝tag そのものでビット不変。FFI も i8 tag で不変。match は i8 tag の compare+branch で `if` と同等）。`&&`/`||`/`not` は `if` 経由なら desugar で吸収。

**射影に専用ノードは持たない**: フィールド/variant payload/配列要素の取り出しは getter プリミティブ＝`Llvm` で表し、解析は他プリミティブ同様 `UniqSignature` で扱う（getter を名指し特別扱いしない既決方針と一貫）。
**`Let` は単一 Var のみ（Pattern を持たない）**: `let (x,y)=s` 等の struct/タプル destructure は **getter プリミティブ列 ＋ `Release(container)`** に lower（役割分担: 構造分解は getter、union 分岐は `Match`）。`get_struct_fields`/`get_union_value` の retain/release を getter プリミティブ＋明示 RC で再現し、相殺が move-out に最適化する。
**RC IR は nested lambda を持たない**: lowering が全 lambda を top-level RC IR 関数へ lift し、使用箇所を `Closure(func, 捕捉)` に変換する（Lean/Koka 同様、クロージャ生成を明示）。各関数の RC が閉じ、クロージャ値も普通の boxed 値になる。`FuncRef` ＝ top-level RC IR 関数への参照（名前/id。lift した lambda body。codegen で funptr に解決）。`Closure` の捕捉リスト（`Vec<Atom>`）は**順序つきでノードに保持**する（順序＝closure の env レイアウトで lifted 関数の env パラメータ順と一致。free vars から再計算は可能だが順序が一意でない・生成時の retain 等 RC に要る・再計算回避のため保持。全変換が lifted 関数 env と整合を保つ）。
**RC 完全性（IR が単一の真実）**: codegen が現在行う全 RC 操作を IR ノードで表す。retain/release → `Retain`/`Release`、`mark_global`/`mark_threaded` → `MarkGlobal`/`MarkThreaded`。**`InlineLLVM` は内部で RC をしない**——使用引数の disposition（consume→release ／ 戻り値へ move→release しない ／ borrow）を宣言し、IR 生成が明示 `Release` を挿入する（この disposition は §3.3 `UniqSignature` と同一宣言で兼ねる）。`make_array_unique` 等の force-unique 内 clone は op の意味に内包するが、引数 disposition は宣言する。`MarkGlobal` 以外に「is-global チェック」専用ノードは不要（状態チェックは状態不明時の runtime `Retain`/`Release` に内包。静的に global/local と分かれば `RcMode` を `Global`(no-op)/`Local`(チェック省略) に特殊化する＝将来の state 最適化、§6）。**P1 で codegen の全 RC site を監査**し、漏れなく IR ノード化されることを確認する（§1.6 受け入れ条件）。

直線列の peephole（retain/release 相殺等）は「直線スパンを `Vec` に集めて変換し再構築」ヘルパで扱う。代替形式 = `Block { stmts: Vec<Stmt>, term: Term }`（文/終端子を型で分離、Vec 操作が楽だが二段構造・既存 AST と別形・型増）。Fix は構造化制御フロー（任意 jump 無し）なので継続入れ子で十分。

### 1.3 意味論（refcount）
- `Construct`/alloc 系 → 新規確保、root refcount 1（LOCAL = unique）。
- `Retain(x)` → x の指す値の root refcount +1。`Release(x)` → −1（0 で解放。release は構造を辿る＝既存 `build_release_mark` ＋ `TraverserWorkType`）。
- state（`REFCNT_STATE_LOCAL`/`THREADED`/`GLOBAL`, constants.rs:118-120）は既存どおり。**global は retain/release が no-op、決して unique にならない**。in-place は LOCAL ∧ refcount==1 のときのみ。
- op は引数 atom の参照を消費（move）。同じ値を複数回使うには事前に `Retain`。

### 1.4 AST → RC IR lowering（`generator.rs` から RC を抽出）
`generator.rs` の RC 決定ロジックをこのパスへ移す:
- **ANF 化**（兄弟位置を atom 化、effectful を `let` に）。
- **lambda lifting**: 残存する全 lambda（`Expr::Lam`）を top-level RC IR 関数へ持ち上げ、使用箇所を `Closure(func, 捕捉)` に変換（現状 codegen が `declare_lambda_function`/`eval_lam` でやっている「lambda ごとに関数を宣言＋捕捉でクロージャ生成」を、IR 生成に前出し）。RC IR に nested lambda は残さない。
- **名前は lowering が fresh 発番**（名前カウンタ＋AST名→新名の env で traverse、シャドー解消）＝構築により一意。別途 `unique_local_names` パスは不要。
- **last-use 解析**（`Scope.used_later` / `scope_lock_as_used_later` 相当。`find_usage_of_name` も利用可）。
- **明示 retain/release 挿入**: `get_scoped_obj` の used_later→retain を `Retain` ノードに、scope 退出（未使用 let 束縛・分岐 dead 変数）の release を `Release` ノードに。
- IOState threading はデータ依存として保持（順序自動保存）。

### 1.5 RC IR → LLVM codegen（付け替え）
- 変数 get は素の get（retain 判断なし）。`Retain`/`Release` ノード → inc/dec（release は構造辿り）。`Scope.used_later`/`scope_lock_as_used_later`/`get_scoped_obj` の retain 分岐は**消滅**。
- 非 RC 部分（クロージャ生成、FFI、struct/array レイアウト、LLVM 構築）はそのまま移植。

### 1.6 検証
- `cargo test --release` 全通過（codegen 付け替えは全プログラムに影響する）。
- RC の挿入数・順序・解放挙動が現状と一致（リグレッションなし）。デバッグ用に「unique と判定した値が実行時に共有なら abort」する assert ビルド。
- **デバッグ情報の一致**: debug ビルドで行/関数の対応・breakpoint・backtrace が現状どおり（span 保持の検証。§1.1-5）。
- **全ベンチマークでリグレッションなし**: `benchmark/speedtest` 全 case ＋ `fix-bench/batch` を走らせ、commit hash 付きで記録・比較。RC IR 導入は挙動を変えない（性能含め）はずなので、速度劣化が無いことを確認。
- **外部ライブラリのテスト**: 一通り走らせて確認する（ユーザが実施）。**P1 完了時にユーザへ連絡し、外部ライブラリテストの実行を依頼する**（このタイミングで手を止めて報告）。

## 2. retain/release 相殺（基盤の一部・uniqueness 解析を簡単にする）

`Retain(x)` の後、その追加参照を必要とする使用が無いまま `Release(x)` が来るなら両方除去（peephole / 簡単な dataflow）。名前一意なので追跡が容易。

効果:
- 「used-later で一旦 retain したが直後の op が release（net-zero）」のような冗長 RC が消える。
- → 後段の uniqueness 解析は「**`Retain` されていない boxed 値 ＝ unique**」を素直に読めるようになり、AST 版で苦労した last-use/net-zero の再導出が不要になる。
- clone 削減としても有用。

これは**純粋な RC 削減の最適化**（最終コードの retain/release が減って速くなる、clone も減る）。**uniqueness 解析の precision には不要**: §3 は leaf を refcount 上界 `Static(n)` にして `Release` で count を戻すので、net-zero（1→2→1）を**解析自身が回復**する（相殺前でも `Static(1)` と分かる）。よって相殺は順序自由でいつ走らせてもよく、解析の前提ではない（健全性とも無関係）。

## 3. uniqueness 解析（RC IR の interpret）

`UniqTree`（各値の uniqueness を静的に表す、保守的に `Shared` 側へ倒した近似）を RC IR を辿って求める。**名前グローバル一意**なので env は `Map<Var, UniqTree>` のみ（スコープ push/pop 不要）。

### 3.1 `UniqTree` と格子
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum Uniqueness { Static(usize), Dynamic } // boxed root の refcount **上界**。Static(1)=unique、Static(n>1)/Dynamic=shared、Dynamic=⊤。
// 健全性: Static(n) ＝「real refcount ≤ n」。Construct→Static(1)、Retain→+1、Release→−1、join→max。
// 終端性: count に上限 K を設け超過は Dynamic に widen（K=2 で実用十分。実質 {Static(1),Static(2),Dynamic} の有限格子）。
// 以降の散文では unique＝Static(1)、shared＝Static(n>1)/Dynamic の意。

#[derive(Clone, PartialEq, Eq)]
enum UniqTree {
    Bottom,                               // ⊥。情報なし/到達しない。join 単位元。union の出現しない variant。
    UnboxedLeaf,                          // boxed 内容なし（scalar / 正規化で畳まれた all-unboxed aggregate）。
    BoxedAggregate(Uniqueness, Vec<UniqTree>), // Array(len1=要素)/boxed struct(フィールド)/boxed union(variant payload)。root + 中身。
    UnboxedAggregate(Vec<UniqTree>),      // タプル/unboxed struct/unboxed union（root 無し）。
}
```
join は pointwise（`Bottom` 単位元、leaf は `max`（`Static(a)`,`Static(b)`→`Static(max(a,b))`、`Dynamic` が絡めば `Dynamic`、cap 超は `Dynamic`）、`BoxedAggregate` は root と中身を join、aggregate は zip）。
正規化（スマートコンストラクタで強制）: `UnboxedAggregate` は boxed root を持つ子が 1 つ以上あるときだけ作り、全子が `UnboxedLeaf`/`Bottom` なら `UnboxedLeaf` に畳む（`(I64,I64)` は一意に `UnboxedLeaf`）。`BoxedAggregate` は自身が root を持つので畳まない（`Array I64` = `BoxedAggregate(_, [UnboxedLeaf])`）。
補助: `top_of(ty)`（保守的 ⊤＝全 boxed root を `Shared`）、`project(u, i)`（aggregate の i 番目。Array は i=0 が要素）。

### 3.2 interpret 規則（RC IR を辿る。retain/release 相殺後を入力に想定）
env: `Map<Var, UniqTree>`。`RcExpr` を順に処理:
- `Let(x, op, k)`: `env[x] = U(op)`; `k` へ。
  - `Construct` → 新規確保: `BoxedAggregate(Static(1), [引数の UniqTree])` または `UnboxedAggregate(...)`。
  - `Llvm(prim, args)` → prim の宣言 `UniqSignature` を適用（force-unique 系の**結果 root は常に `Unique`**）。
  - `App(f, args)` → `f` の `UniqSignature`（推論）を適用（`f` がクロージャ値でも funptr でも、対象 top-level 関数の要約を引く）。
  - `Closure(func, captures)` → `BoxedAggregate(Unique, [captures の UniqTree])`（新規 closure。捕捉を保持）。これにより捕捉された配列等の uniqueness を追える（捕捉クロージャ越しの健全性が扱える）。
  - 射影（フィールド/variant payload/配列要素）は getter プリミティブ＝`Llvm` なので上の `Llvm` 規則で扱う（`project(env[a], i)` 相当は getter の `UniqSignature` の中身。専用規則なし）。
- `Retain(x, k)`: `env[x]` の root を +1（`Static(n)`→`Static(n+1)`、cap 超は `Dynamic`）。`k` へ。
- `Release(x, k)`: `env[x]` の root を −1（`Static(n)`→`Static(n-1)`。**`Static(2)`→`Static(1)` で unique に回復**、`Static(1)`→消費/dead、`Dynamic` は据え置き）。`k` へ。これにより net-zero を**解析自身が回復**し、相殺は precision には不要（§2）。
- `Match(a, arms)`: 各 arm を**分岐前 env のコピー**から解析し、結果と env を `join`（名前一意なので merge は単純）。`Bottom` で variant payload を扱う（union の出現しない variant）。Bool もここ（2 variant の union）。
- global 変数参照 → `top_of`（`Shared`）。env 対象外。

要点: **明示 RC を `Static(n)` の上下で追うだけ**（`Construct`=1, `Retain`=+1, `Release`=−1, 分岐=max, cap で widen）。`Static(1)`＝unique。`Release` が count を戻すので net-zero は解析自身が回復し、AST 版の `received`/`shareize`/disposition/last-use 再導出は不要。

詳細（特に getter の retain 有無、`Release` 後の精密な扱い、相殺との連携）は P2 で確定。

### 3.3 関数の効果（per-input-key concrete。cardinality 不要）
RC IR は複製を明示 `Retain` で表すので、AST 版の cardinality 半環（`{1,2}`, `·` で `1·1=2`）・parametric signature は**不要**。`·`（複製）は明示 `Retain` → `Shared`、`+`（分岐合流）は `UniqTree` の 2 点 leaf join（`Unique∧Unique=Unique`、他 `Shared`）に吸収される。関数の効果は、入力 uniqueness（キー）ごとに body を §3.2 で解析した結果＝`(結果 UniqTree, 各引数の呼び出し後状態)` を memo するだけ（concrete、parametric 変数なし）。
- **ソース関数 = 推論**: 入力キーごとに body 解析を memo（§4.1 の特殊化 worklist と共有。再帰は不動点、初期 `Bottom`）。
- **プリミティブ（`Llvm`/FFI）= 宣言**（concrete な効果）: getter `@i`→結果＝入力 field i／対象 field を consume・他 release、`set`/`mod`/`act`→配列引数 consume・結果 root `Unique`・格納値は要素位置へ、`fill`→要素 `Shared`（多数スロットに同値＝複製）、`boxed_to_retained_ptr`→引数 escape。未知 FFI は保守的（引数 `Shared`・結果 `top_of`）。
- global = `Shared`。assert ビルドで不健全な claim を実行時検出。

## 4. unique-check-elim

force-unique を含む `Llvm` op（`set`/`mod`/`act` 系）で、対象 boxed 値の refcount==1（LOCAL）が §3 の解析で証明できれば、**force-unique を行わない版に差し替える**。結果は force-unique 後どのみち unique なので、ループ `let arr = arr.set(…)` で 2 回目以降の入力が unique になり「**初回 checked・以降 unchecked**」が自然に出る。

### 4.1 特殊化（uniqueness 駆動）
呼び出し地点ごとに流れてくる引数 uniqueness をキーに関数を clone（`decapturing::SpecializationRequest` に倣う、md5 命名）。worklist で不動点。clone 時は fresh 名を発番して freshen（不変条件 1.3 を保存）。`dead_symbol_elimination` が未使用 original/clone を掃除。

### 4.2 lowering（`InlineLLVM` ノードの force-unique を外す）
clone した body 中の force-unique を担う `Llvm`(InlineLLVM) ノードを `force_unique=false` で作り直す（AST ノードは不変なので新ノードに差し替え。共有呼び出し地点側は checked のまま）。`force_unique` フラグの有無:

| 操作 | force-unique の所在 | フラグ |
|---|---|---|
| Array `set` | `InlineLLVMArraySetBody`（無条件 `make_array_unique`, builtin.rs:2170） | **無し→追加** |
| Array `mod`/`act_identity`/`act_tuple2` | `_unsafe_get_linear_bounds_unchecked_unretained`（`force_unique`, builtin.rs:1901/1936） | 既存→`false` |
| struct `mod_<field>` | `#punch_fu_{field}`（`InlineLLVMStructPunchBody`{true}, `make_struct_unique` @2656） | 既存（非 fu punch あり）→`false` |
| struct `set_<field>` | `InlineLLVMStructSetBody`（無条件 `make_struct_unique`, builtin.rs:3580） | **無し→追加** |
| struct `act_<field>` | 非 fu punch を既に使用（unique 保証） | 対応不要 |

`Const` functor の `act` は force-unique なし（対象外）。generic な `act`（任意 functor）は `unsafe_is_unique` の once-per-call チェックで follow-on。`optimize_act` が Identity/Const/Tuple2（ホットな `mod`=Identity 含む）を上記形に落とす。

## 5. 適用対象・検証

- マイクロ: `batch/arrayrw{,_unsafe,_fn}`、`fannkuch`。正しさ: `cargo test --release`＋**共有値テスト**（2 箇所に格納して破壊しないこと）。回帰: `benchmark/speedtest`。assert ビルドで不健全検出。
- 一意文脈でチェックが消える（IR/asm に `build_branch_by_is_unique` 由来分岐が残らない or cachegrind 命令数低下）＋共有文脈で消えない（クローンされる）を各セルで確認:

| 対象 | set | mod | act(Id) | act(Tuple2) | act(Const) | 備考 |
|---|:--:|:--:|:--:|:--:|:--:|---|
| Array | ✓ | ✓ | ✓ | ✓ | — | Const は getter |
| boxed struct field | ✓ | ✓ | ✓ | ✓ | — | `make_struct_unique` を外す |
| union | — | — | — | — | — | `mod_<variant>` は force-unique を踏まない＝対象外 |

入れ子伝播も確認: タプル内配列 `loop((cnt, arr), …)`、struct 内配列・配列内 struct、union 内配列（`LoopState`）。

## 6. 将来の RC 最適化（同じ RC IR 上）
- **retain/release 相殺**（§2。実は基盤の一部）。
- **reuse / Perceus**（`Release` した alloc を直後の `Construct` で再利用＝in-place 再確保）。
- **borrow 推論**（引数を借用にして retain/release を削減）。
- **順序スケジューリング**（意味を保つ範囲で評価順を並べ替え in-place 機会を増やす。例: `f(arr.set(0,42), arr.@0)` を `arr.@0` 先に並べ替えると set が in-place 化し clone が消える。Koka/Lean reuse の類）。
- **state 推論**（各値の refcount-state＝LOCAL/THREADED/GLOBAL を静的に決め、RC・状態チェック・`mark_threaded` を省く）。proven-global → `RcMode::Global`（codegen no-op）。proven-local → `RcMode::Local`（状態チェック省略）。送信値が proven-deeply-unique → `MarkThreaded` 省略。`MarkGlobal` も静的に分かる範囲で最適化。
- **境界チェック除去**（`idx ∈ [0,size)` を証明し完全 unchecked へ。一意性除去と合成でベクトル化 0.20x）。
- **match-of-known-constructor / case-of-case**（LLVM 未実施を確認の上）。

## 7. マイルストーン
- **P0（P1 前）**: **デバッグ情報の E2E テストを追加**してベースライン化。現状その回帰テストが無いため、`fix build -g`（DWARF 付き）でビルドした小プログラムを **gdb 駆動**（`gdb -batch`: `break main.fix:N` → run → backtrace）で検査する統合テストを作る（CLAUDE.md 規約: サンプルを tempdir にコピー、`fix`/`gdb` を `Command` 実行）。assert は file:line の解決・停止・スタックの行情報（マングル名非依存）。補助で bundled `llvm-dwarfdump` の構造 assert も可。**現 main で通すこと**＝P1 の「デバッグ情報一致」(§1.6) の比較対象。ツール: `/usr/bin/gdb` あり、`llvm-dwarfdump` は `/home/maruyama/llvm-17.0.6/bin/`（system には無し）。
- **P0.5（P1 前提）**: **Bool を union 化**（std.fix: `type Bool = unbox union {_false,_true}; true=_true(); false=_false();` ＋ 比較演算子の結果型 ＋ FFI Bool↔i8 tag）。これが `If` を IR から落とす前提（`If`→`Match` desugar は P1 lowering 内）。性能中立（Bool-union＝i8）。de-risk するなら現 `eval_if` を union Bool 対応にして先行検証、または P1 で `eval_if` 撤去と同時。
- **P1**: RC IR 型 ＋ AST→RC IR lowering（`generator.rs` から RC 抽出。名前は lowering が fresh 発番）＋ codegen 付け替え ＋ 全テスト再検証。**最大の山**。完了ゲート: `cargo test --release` 全通過・全ベンチでリグレッションなし・デバッグ情報一致（§1.6）を満たし、**ユーザに連絡して外部ライブラリテストを依頼してから次フェーズへ**。
- **P1.5（順序自由）**: retain/release 相殺（§2）。純粋な RC 削減（perf）。uniqueness 解析の precision には不要（§3 の `Static(n)` leaf が `Release` で net-zero を回復するため）。P2/P3 と独立に入れられる。
- **P2**: uniqueness 解析（`UniqTree`/`UniqSignature` を RC IR interpret）。read-only ログから始め arrayrw のループ `set` を unique・共有テストを shared と判定することを確認。
- **P3**: unique-check-elim（force-unique 除去 ＋ 特殊化）。arrayrw/fannkuch 計測、全テスト。
- **P4**: reuse / borrow / 順序スケジューリング / 境界チェック除去 等。

## 8. リスク・未解決
- **P1 の codegen 付け替えの再検証コスト・範囲**が最大リスク（全プログラムに影響）。段階導入できるか（一部関数だけ RC IR 経由、等）も検討。
- RC IR の正確な意味論: `Release` 後の uniqueness 回復は §2 の相殺に委ね、§3 解析は保守的に（回復しない）。
- ローカル名一意の**全変換での保存**（lowering は fresh 発番で構築的に一意。clone/特殊化は fresh 名発番で freshen）。
- getter（射影）の retain 有無・`UniqSignature` の不動点収束・threaded state・FFI escape の RC IR での扱い。捕捉クロージャは `Closure` で captures を `BoxedAggregate` として追える（旧 TODO 解消方向）が、共有/別名の健全性は検証する。
- `UniqTree` の boxed aggregate 内部追跡は設計済み（RC IR でも同じ）。
- 旧 TODO「汎用 metadata フィールド」は、RC IR を新設するなら RC IR ノード側に最初から付随情報欄を設ければよく、AST 改修は不要になりうる（P1 設計で判断）。

### 決定事項・要確認（設計レビューで確定したもの）
- **（決定）leaf を refcount 上界 `Static(n)|Dynamic` にする**（§3.1）: `Construct`=Static(1)、`Retain`=+1、`Release`=−1（net-zero を回復＝`Static(2)`→`Static(1)`）、分岐=max、終端性のため cap で `Dynamic` に widen（K=2 で実用十分）。これにより: (a) AST 版の parametric cardinality 半環・記号変数は不要（明示 RC が駆動する concrete 数値に置換。`·`/parametric なし）、(b) **retain/release 相殺は precision に不要**（解析が `Release` で回復するため。相殺は純粋 perf）。§3.3 は per-key memoize に簡素化済み。
- **（決定）Bool→union（P0.5、§7）**: std.fix 定義＋比較演算子の結果型＋FFI（Bool↔i8 tag、`_false`=0/`_true`=1）。`If`→`Match` desugar は P1 lowering 内。要確認: 比較 InlineLLVM の結果構築・`&&`/`||`/`not`・typecheck が union Bool で通るか（ビットは i8 不変）。
- **（決定）global 値の表現**: global 初期化を RC IR（init）として表し `MarkGlobal` を init で発行。参照は atom で解析は `Shared`。program = top-level 関数集合 ＋ global init。現状の global 機構（lazy/eager・mark_global 発火点）は P1 実装時に確認。
- **（決定）lowering サブパス順**: AST 正規化（ANF 化 → lambda lift → `If`→`Match` desugar → destructure→getter → fresh 命名）→ 最後に last-use 解析＋明示 retain/release 挿入で RC IR 生成（形と名前が確定してから RC を載せる）。
- **（調査済み）RC site 監査の規模**: codegen の RC は `generator.rs` ~38・`builtin.rs` ~29（InlineLLVM `generate` 内部の release/retain）・`object.rs` ~21。builtin の 29 を「primitive 内 atomic（`make_array_unique` の clone-release 等、op 意味に内包し disposition 宣言）」と「明示 `Release` 化すべきもの（引数を使用後に release 等）」に分類するのが P1 の主要監査。
- **force-unique 内 clone の RC 境界**: `make_array_unique`/`make_struct_unique` の clone（共有時に deep copy ＋要素 retain）は op の atomic 意味に内包し、内部 RC は IR ノードに出さない（最適化対象でない共有パスのため）。引数 disposition のみ宣言。
