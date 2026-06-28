# Fix RC 中間言語（RC IR）と一意性チェック除去（unique-check-elim）

ステータス: 設計のみ・未実装。

RC（参照カウント）最適化の基盤として **RC IR**（評価順を固定した ANF ＋ 明示 retain/release ＋ ローカル名グローバル一意）を導入し、その上で uniqueness 解析・unique-check-elim・将来の RC 最適化（retain/release 相殺・reuse・borrow・順序スケジューリング）を行う。

用語: 値の形＝`AVal`（boxed はロケーションへのポインタ）、参照カウント上界＝`CTRefCnt`（`Static(n)`/`Dynamic`、`Static(1)`=unique）をコンパイル時の仮想ヒープ `heap: Loc→Cell` に持つ、関数ごとの要約＝`UniqSignature`。

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
    Let(Var, RcRhs, Box<RcExpr>),          // op の結果を Var（単一）に束縛 → 継続（ANF）
    Retain(Var, RcState, Box<RcExpr>),     // x を dup（root +1） → 継続（Lean の inc）。RcState で state ディスパッチを選択
    Release(Var, RcState, Box<RcExpr>),    // x を drop（-1; 0 で解放、構造辿り） → 継続（Lean の dec）
    MarkGlobal(Var, Box<RcExpr>),          // x の到達グラフを GLOBAL state に（= mark_global）。global 値初期化で発行 → 継続
    MarkThreaded(Var, Box<RcExpr>),       // x の refcount を threaded(atomic) mode に（= mark_threaded）。別スレッド送信前 → 継続
    Ret(Atom),                            // 終端子
    Match(Atom, Vec<MatchArm>),           // 終端子。唯一の分岐構造（Bool もここ）
}
struct MatchArm { variant: usize, payload: Var, body: RcExpr }  // payload を単一 Var に束縛（さらなる分解は getter で）
enum RcRhs {
    App(Atom /*callee: closure か funptr の値*/, Vec<Atom>),  // closure 呼び出し・直接 funptr 呼び出し両方（codegen が型で振り分け）
    Closure(FuncRef, Vec<Atom> /*捕捉*/),  // top-level 関数 + 捕捉変数列 → boxed closure 値（root unique、捕捉を保持）
    LLVM(InlineLLVM, Vec<Atom>),          // set/mod/算術/union_new/... ＋ フィールド/variant payload/配列要素 の射影（getter）も含む。特別扱いしない
    Construct(Ctor, Vec<Atom>),           // MakeStruct（タプルは MakeStruct の特殊系）／ArrayLit → 新規 alloc（boxed なら root unique）。union 構築は `union_new`＝`LLVM`
}
enum Atom { Var(Var), Lit(Literal) }     // Var は global funptr 参照も含む
enum RcState {            // retain/release の state ディスパッチ。lowering は既定 Unknown（健全）、§6 の state 推論が特殊化（`RcState` を注釈するだけ＝構造変更なし）
    Unknown,             // runtime で refcnt_state を見て 3-way（= 現状の retain/release）
    Local,               // LOCAL 確定: 非 atomic inc/dec、state チェック省略
    Threaded,            // THREADED 確定: atomic inc/dec、state チェック省略
    Global,              // GLOBAL 確定: codegen で no-op（コードを出さない）。最小化したければ後段 cleanup で削除可
}
```
**分岐は `Match` のみ（`If` を持たない）**: Bool を union 化する（std.fix: `type Bool = unbox union { _false : (), _true : () }; true = _true(); false = _false();`）。ソースの `if`/`true`/`false`/比較演算子は不変で、AST→RC IR 生成で `Expr::If(c,t,e)` を `Match(c, [_false => e, _true => t])` に desugar するだけ。性能中立（Bool-union ＝ `{i8 tag, [i8;0]}` ＝ i8。比較演算子は今も i8(0/1) を返す＝tag そのものでビット不変。FFI も i8 tag で不変。match は i8 tag の compare+branch で `if` と同等）。`&&`/`||`/`not` は `if` 経由なら desugar で吸収。

**射影に専用ノードは持たない**: フィールド/variant payload/配列要素の取り出しは getter プリミティブ＝`LLVM` で表し、解析は他プリミティブ同様 `UniqSignature` で扱う（getter を名指し特別扱いしない既決方針と一貫）。
**`Let` は単一 Var のみ（Pattern を持たない）**: `let (x,y)=s` 等の struct/タプル destructure は **getter プリミティブ列 ＋ `Release(container)`** に lower（役割分担: 構造分解は getter、union 分岐は `Match`）。`get_struct_fields`/`get_union_value` の retain/release を getter プリミティブ＋明示 RC で再現し、相殺が move-out に最適化する。
**RC IR は nested lambda を持たない**: lowering が全 lambda を top-level RC IR 関数へ lift し、使用箇所を `Closure(func, 捕捉)` に変換する（Lean/Koka 同様、クロージャ生成を明示）。各関数の RC が閉じ、クロージャ値も普通の boxed 値になる。`FuncRef` ＝ top-level RC IR 関数への参照（名前/id。lift した lambda body。codegen で funptr に解決）。`Closure` の捕捉リスト（`Vec<Atom>`）は**順序つきでノードに保持**する（順序＝closure の env レイアウトで lifted 関数の env パラメータ順と一致。free vars から再計算は可能だが順序が一意でない・生成時の retain 等 RC に要る・再計算回避のため保持。全変換が lifted 関数 env と整合を保つ）。
**RC 完全性（IR が単一の真実）**: codegen が現在行う全 RC 操作を IR ノードで表す。retain/release → `Retain`/`Release`、`mark_global`/`mark_threaded` → `MarkGlobal`/`MarkThreaded`。**`InlineLLVM` は内部で RC をしない**——使用引数の disposition（consume→release ／ 戻り値へ move→release しない ／ borrow）を宣言し、IR 生成が明示 `Release` を挿入する（この disposition は §3.3 `UniqSignature` と同一宣言で兼ねる）。`make_array_unique` 等の force-unique 内 clone は op の意味に内包するが、引数 disposition は宣言する。`MarkGlobal` 以外に「is-global チェック」専用ノードは不要（状態チェックは状態不明時の runtime `Retain`/`Release` に内包。静的に global/local と分かれば `RcState` を `Global`(no-op)/`Local`(チェック省略) に特殊化する＝将来の state 最適化、§6）。**P1 で codegen の全 RC site を監査**し、漏れなく IR ノード化されることを確認する（§1.6 受け入れ条件）。

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
- → 後段の uniqueness 解析は「**`Retain` されていない boxed 値 ＝ unique**」を素直に読めるようになる。
- clone 削減としても有用。

これは**純粋な RC 削減の最適化**（最終コードの retain/release が減って速くなる、clone も減る）。**uniqueness 解析の precision には不要**: §3 はオブジェクトの参照カウントを上界 `Static(n)` で持ち `Release` で戻すので、net-zero（1→2→1）を**解析自身が回復**する（相殺前でも `Static(1)` と分かる）。よって相殺は順序自由でいつ走らせてもよく、解析の前提ではない（健全性とも無関係）。

## 3. uniqueness 解析（RC IR を抽象解釈）

RC IR を**抽象解釈**し、**コンパイル時の仮想ヒープ**上で参照カウントだけを emulate する。各オブジェクト（ロケーション）に `CTRefCnt`（refcount 上界）を持たせ、`Construct`=確保、`Retain`/`Release`=増減、射影=エイリアス、を辿る。ループ・再帰は実回数まわさず、有限領域上の**不動点**で畳む（合流で join、終端のため cap で widen）。`CTRefCnt` は上界なので、合流・要約ロケーションでは shared 側（保守的）へ倒れる。

**なぜ per-var の木でなく仮想ヒープか**: refcount は「オブジェクトの属性」であって「変数の属性」ではない。retain する getter（`arr.@i`）は boxed の子に**第二の参照**を作る＝複数の変数が同一オブジェクトを指す（別名）。refcount を変数ごとに持つと別名間で同期できず不健全（`x` 経由で「unique」と誤判定して in-place 破壊しうる）。refcount を**ロケーションの cell に1つ**持てば、`Retain`/`Release` が cell を更新し、それを指す全別名が同じ値を見る。

### 3.1 状態（仮想ヒープ）
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum CTRefCnt { Static(usize), Dynamic } // オブジェクトの refcount **上界**。Static(1)=unique、Static(n>1)/Dynamic=shared、Dynamic=⊤。
// 健全性: Static(n) ＝「real refcount ≤ n」。Construct→Static(1)、Retain→+1、Release→−1、join→max。
// 終端性: count に上限 K（K=2 で実用十分）。超過は Dynamic に widen（実質 {Static(1),Static(2),Dynamic} の有限格子）。
// 散文では unique＝Static(1)、shared＝Static(n>1)/Dynamic の意。

// 抽象ロケーション: アロケーションサイト（Construct/Closure/force-unique 結果/不明 alloc のプログラム地点）で抽象化。
//   サイトは有限個 → Loc も有限。ループ等で同一サイトを再訪すると同じ Loc に集約＝summary loc（複数の具体オブジェクトを表す）。
//   + Top: 外部・global・未知 FFI 由来。常に Dynamic。
type Loc;                  // AllocSite | Summary(AllocSite) | Top
type PtsTo = Set<Loc>;     // points-to 集合（分岐合流で複数 Loc を指しうる。通常は単集合）

#[derive(Clone, PartialEq, Eq)]
enum AVal {                  // env / フィールドが持つ「値の形」。boxed はインラインせず必ずポインタ1個で切れる
    Unboxed,                 // scalar（refcount 無し）
    UnboxedAgg(Vec<AVal>),   // タプル/unboxed struct/unboxed union（refcount 無し、子を持つ）
    Boxed(PtsTo),            // boxed 値＝仮想ヒープへのポインタ（rc と中身は cell 側）
    Bottom,                  // ⊥（到達しない・union の不在 variant）。join 単位元
}

struct Cell { rc: CTRefCnt, contents: AVal } // 1 オブジェクト分。Array は contents=要素 AVal、boxed struct は UnboxedAgg(フィールド)、boxed union は variant ごと
struct State { env: Map<Var, AVal>, heap: Map<Loc, Cell> } // heap ＝ 仮想ヒープ
```

**再帰型は有限グラフ**: `AVal` は boxed のところで必ず `Boxed(PtsTo)`（ポインタ）に切れるので `AVal` 単体は常に有限。`List` 等の再帰は **cell の contents が別の Loc（自分自身も可）を指す巡回グラフ**になり、Loc が有限（アロケーションサイト抽象）なので全体も有限。よって木の打ち切りノードは要らず、有限性は Loc 集合の有限性が担保する。ループ生成された再帰構造は同一サイト → summary loc に集約（rc は `Dynamic`、更新は弱更新）。

**強更新 / 弱更新**: points-to が**単集合 `{L}` かつ `L` が非 summary** のときだけ `L` の cell を**強更新**（`Release` で `Static(2)`→`Static(1)` の unique 回復ができる＝線形スレッドの精度）。多重指し or summary loc では**弱更新**（`Retain` は各 Loc を +1＝上界、`Release` は減算しない＝上界を保つ）。これで「線形ケースは精度を保ち、別名・要約は健全に保守的」を両立する。

**join**（合流・不動点）: `AVal` は pointwise（`Bottom` 単位元、`Unboxed` 同士、`UnboxedAgg` は zip、`Boxed(P1)⊔Boxed(P2)=Boxed(P1∪P2)`）。`heap` は Loc ごとに cell を join（`rc` は max、`contents` は join）。PtsTo が増えすぎたら summary loc に集約（widen）。`CTRefCnt` の K-cap と Loc の有限性で格子は有限 → 不動点は停止。
補助: `top_of(ty)`（保守的 ⊤＝boxed は `Boxed({Top})`、`heap[Top].rc=Dynamic`）。

### 3.2 interpret 規則（RC IR を辿る）
`State`（env, heap）を更新しながら `RcExpr` を順に処理:
- `Let(x, Construct(_, args), k)`: このサイトの Loc `L`（初訪は新規、再訪は summary 化）に `Cell{ rc: Static(1)（summary なら join → Dynamic）, contents: args の AVal }` を確保。`env[x] = Boxed({L})`。args は move-in（参照がフィールドへ移るだけで arg-cell の rc は不変）。
- `Let(x, Closure(_, captures), k)`: 同様に新規 cell `Cell{ Static(1), UnboxedAgg(captures の AVal) }`、`env[x]=Boxed({L})`。捕捉は move-in。これにより捕捉された配列等を追える。
- `Let(x, LLVM(prim, args), k)`: prim の宣言 `UniqSignature`（§3.3）を適用。
  - 射影（getter）`x = get(a, i)`: `env[x]` ＝ `a` の指す cell の `contents` を i 射影した AVal（boxed なら同じ Loc を指す）。**retain する getter**（`Array::@` 等、配列を残して要素を複製）は子 Loc の rc を +1 → `env[x]` と `a` の field i が同一 Loc を共有＝**別名を捕捉**（以後 `Retain`/`Release` が同じ cell を更新）。**move-out する linear get**（`mod`/`act` の `_unsafe..unretained`）は rc 据え置き（参照がフィールドから出るだけ）。
  - force-unique 系（`set`/`mod`/`act`）: 結果は新規 or 再利用 Loc で `rc=Static(1)`、格納値は要素位置へ。
- `Let(x, App(f, args), k)`: `f` の `UniqSignature` を適用（結果 AVal ／ 各 arg cell の rc 効果 ／ 結果が arg を別名化するか、を反映。クロージャ値でも funptr でも対象関数の要約を引く）。
- `Retain(x, k)`: `env[x]` の root PtsTo の各 Loc の rc を +1（単集合・非 summary は強更新、多重・summary は各 +1 の弱更新）。全別名が同一 cell を見るので同期する。
- `Release(x, k)`: 単集合 `{L}` かつ非 summary なら `L.rc` を −1（**`Static(2)`→`Static(1)` で unique 回復**、`Static(1)`→0 で解放し contents を辿って子を再帰 Release）。多重・summary は据え置き（上界維持）。`Release` が count を戻すので net-zero（1→2→1）は**解析自身が回復**する。
- `Match(a, arms)`: 各 arm を**分岐前 State のコピー**から解析し、結果 State を `join`。不在 variant の payload は `Bottom`。Bool もここ（2 variant）。
- `Ret(atom)`: 結果 ＝ atom の AVal。
- global 参照 → `Boxed({Top})`（`heap[Top].rc=Dynamic`）。

要点: **refcount を仮想ヒープの cell で追い、別名は同一 Loc で共有**。線形スレッド（`a1=set(a0,..); a2=set(a1,..)`）は単集合・強更新で rc が `Static(1)` を保ち、retain getter で生じた別名は子 Loc の rc が ≥2 になって正しく shared と分かる。

詳細（getter ごとの retain 有無、summary loc の弱更新の精密化、PtsTo 集約しきい値）は P2 で確定。

### 3.3 関数の効果（`UniqSignature`, per-input-key concrete）
関数の効果は、入力（各引数の AVal ＋ 関係する cell）をキーに body を §3.2 で解析した結果＝`(結果 AVal, 各 arg cell の rc 効果, 結果↔引数の別名関係)` を memo する（入力ごとの concrete な要約）。
- **ソース関数 = 推論**: 入力キーごとに body 解析を memo（§4.1 の特殊化 worklist と共有。再帰は不動点、初期 `Bottom`）。
- **プリミティブ（`LLVM`/FFI）= 宣言**: retain getter `@i`→結果が field i の Loc を別名化（子 rc +1）、linear get→field を move-out、`set`/`mod`/`act`→配列引数 consume・結果 `rc=Static(1)`・格納値は要素位置へ、`fill`→要素を多スロットへ複製（要素 rc を `Dynamic`）、`boxed_to_retained_ptr`→引数 escape。未知 FFI は保守的（引数を `Dynamic`、結果 `top_of`）。
- global = `Dynamic`。assert ビルドで不健全な claim を実行時検出。

## 4. unique-check-elim

force-unique を含む `LLVM` op（`set`/`mod`/`act` 系）で、対象 boxed 値の refcount==1（LOCAL）が §3 の解析で証明できれば、**force-unique を行わない版に差し替える**。結果は force-unique 後どのみち unique なので、ループ `let arr = arr.set(…)` で 2 回目以降の入力が unique になり「**初回 checked・以降 unchecked**」が自然に出る。

### 4.1 特殊化（uniqueness 駆動）
呼び出し地点ごとに流れてくる引数 uniqueness をキーに関数を clone（`decapturing::SpecializationRequest` に倣う、md5 命名）。worklist で不動点。clone 時は fresh 名を発番して freshen（不変条件 1.3 を保存）。`dead_symbol_elimination` が未使用 original/clone を掃除。

### 4.2 lowering（`InlineLLVM` ノードの force-unique を外す）
clone した body 中の force-unique を担う `LLVM`(InlineLLVM) ノードを `force_unique=false` で作り直す（AST ノードは不変なので新ノードに差し替え。共有呼び出し地点側は checked のまま）。`force_unique` フラグの有無:

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
- **retain/release 相殺**（§2）。純粋な perf（冗長 RC 削減）。unique-check-elim の前提ではない（`Static(n)` leaf が net-zero を回復するため。lowering を最小 RC にすれば仕事も少ない）。
- **reuse / Perceus**（`Release` した alloc を直後の `Construct` で再利用＝in-place 再確保）。
- **borrow 推論**（引数を借用にして retain/release を削減）。
- **順序スケジューリング**（意味を保つ範囲で評価順を並べ替え in-place 機会を増やす。例: `f(arr.set(0,42), arr.@0)` を `arr.@0` 先に並べ替えると set が in-place 化し clone が消える。Koka/Lean reuse の類）。
- **state 推論**（各値の refcount-state＝LOCAL/THREADED/GLOBAL を静的に決め、RC・状態チェック・`mark_threaded` を省く）。proven-global → `RcState::Global`（codegen no-op）。proven-local → `RcState::Local`（状態チェック省略）。送信値が proven-deeply-unique → `MarkThreaded` 省略。`MarkGlobal` も静的に分かる範囲で最適化。
- **境界チェック除去**（`idx ∈ [0,size)` を証明し完全 unchecked へ。一意性除去と合成でベクトル化 0.20x）。
- **match-of-known-constructor / case-of-case**（LLVM 未実施を確認の上）。

## 7. マイルストーン
- **P0（P1 前）**: **デバッグ情報の E2E テストを追加**してベースライン化。現状その回帰テストが無いため、`fix build -g`（DWARF 付き）でビルドした小プログラムを **gdb 駆動**（`gdb -batch`: `break main.fix:N` → run → backtrace）で検査する統合テストを作る（CLAUDE.md 規約: サンプルを tempdir にコピー、`fix`/`gdb` を `Command` 実行）。assert は file:line の解決・停止・スタックの行情報（マングル名非依存）。補助で bundled `llvm-dwarfdump` の構造 assert も可。**現 main で通すこと**＝P1 の「デバッグ情報一致」(§1.6) の比較対象。ツール: `/usr/bin/gdb` あり、`llvm-dwarfdump` は `/home/maruyama/llvm-17.0.6/bin/`（system には無し）。
- **P0.5（P1 前提）**: **Bool を union 化**（std.fix: `type Bool = unbox union {_false,_true}; true=_true(); false=_false();` ＋ 比較演算子の結果型 ＋ FFI Bool↔i8 tag）。これが `If` を IR から落とす前提（`If`→`Match` desugar は P1 lowering 内）。性能中立（Bool-union＝i8）。de-risk するなら現 `eval_if` を union Bool 対応にして先行検証、または P1 で `eval_if` 撤去と同時。
- **P1**: RC IR 型 ＋ AST→RC IR lowering（`generator.rs` から RC 抽出。名前は lowering が fresh 発番）＋ codegen 付け替え ＋ 全テスト再検証。**最大の山**。完了ゲート: `cargo test --release` 全通過・全ベンチでリグレッションなし・デバッグ情報一致（§1.6）を満たし、**ユーザに連絡して外部ライブラリテストを依頼してから次フェーズへ**。lowering は現 codegen の RC（move-out/last-use ＝既に最小 RC）を踏襲し冗長 RC を出さない方針 → retain/release 相殺は早期不要（§3 の `Static(n)` leaf が net-zero を回復するので解析にも不要）。相殺は §6/P4 の perf 磨きに回す。
- **P2**: uniqueness 解析（仮想ヒープ＋`UniqSignature` で RC IR を抽象解釈）。read-only ログから始め arrayrw のループ `set` を unique・共有テストを shared と判定することを確認。
- **P3**: unique-check-elim（force-unique 除去 ＋ 特殊化）。arrayrw/fannkuch 計測、全テスト。
- **P4**: reuse / borrow / 順序スケジューリング / 境界チェック除去 等。

## 8. リスク・未解決
- **P1 の codegen 付け替えの再検証コスト・範囲**が最大リスク（全プログラムに影響）。段階導入できるか（一部関数だけ RC IR 経由、等）も検討。
- `Release` 後の uniqueness 回復は §3 解析が `Static(n)` leaf で行う（`Release`=−1 で `Static(2)`→`Static(1)`）。§2 の相殺とは独立。
- ローカル名一意の**全変換での保存**（lowering は fresh 発番で構築的に一意。clone/特殊化は fresh 名発番で freshen）。
- getter（射影）の retain 有無・`UniqSignature` の不動点収束・threaded state・FFI escape の RC IR での扱い。捕捉クロージャは `Closure` の captures を cell の contents として追える（旧 TODO 解消方向）が、共有/別名の健全性は検証する。
- 別名健全性は refcount を cell に1つ持つことで担保（§3）。summary loc・PtsTo 集約・強/弱更新しきい値の精度調整は P2 で詰める。
- 旧 TODO「汎用 metadata フィールド」は、RC IR を新設するなら RC IR ノード側に最初から付随情報欄を設ければよく、AST 改修は不要になりうる（P1 設計で判断）。

### 決定事項・要確認（設計レビューで確定したもの）
- **（決定）env を仮想ヒープ（store）にする**（§3）: refcount を変数ごとでなくロケーションの cell に持つ。理由: retain する getter が boxed の子に第二参照を作り、変数ごとの木では別名間で同期できず不健全（`x` 経由で unique 誤判定 → in-place 破壊）。cell に refcount を1つ持てば `Retain`/`Release` が全別名に同期する。再帰型は有限 Loc（アロケーションサイト抽象）の巡回グラフで表現するので木の打ち切りノードは不要。線形ケースは単集合・強更新で精度維持、別名・summary loc は弱更新で保守的。
- **（決定）参照カウントを上界 `Static(n)|Dynamic`（`CTRefCnt`）で表す**（§3.1）: `Construct`=Static(1)、`Retain`=+1、`Release`=−1（net-zero を回復＝`Static(2)`→`Static(1)`）、分岐=max、終端性のため cap で `Dynamic` に widen（K=2 で実用十分）。これにより: (a) 要約は入力ごとの concrete な数値（明示 RC が駆動する）、(b) **retain/release 相殺は順序自由な純粋 perf**（解析が `Release` で net-zero を回復するので、相殺の前後どちらでも `Static(1)` を得る）。§3.3 は per-key memoize。
- **（決定）Bool→union（P0.5、§7）**: std.fix 定義＋比較演算子の結果型＋FFI（Bool↔i8 tag、`_false`=0/`_true`=1）。`If`→`Match` desugar は P1 lowering 内。要確認: 比較 InlineLLVM の結果構築・`&&`/`||`/`not`・typecheck が union Bool で通るか（ビットは i8 不変）。
- **（決定）global 値の表現**: global 初期化を RC IR（init）として表し `MarkGlobal` を init で発行。参照は atom で解析は `Dynamic`。program = top-level 関数集合 ＋ global init。現状の global 機構（lazy/eager・mark_global 発火点）は P1 実装時に確認。
- **（決定）lowering サブパス順**: AST 正規化（ANF 化 → lambda lift → `If`→`Match` desugar → destructure→getter → fresh 命名）→ 最後に last-use 解析＋明示 retain/release 挿入で RC IR 生成（形と名前が確定してから RC を載せる）。
- **（調査済み）RC site 監査の規模**: codegen の RC は `generator.rs` ~38・`builtin.rs` ~29（InlineLLVM `generate` 内部の release/retain）・`object.rs` ~21。builtin の 29 を「primitive 内 atomic（`make_array_unique` の clone-release 等、op 意味に内包し disposition 宣言）」と「明示 `Release` 化すべきもの（引数を使用後に release 等）」に分類するのが P1 の主要監査。
- **force-unique 内 clone の RC 境界**: `make_array_unique`/`make_struct_unique` の clone（共有時に deep copy ＋要素 retain）は op の atomic 意味に内包し、内部 RC は IR ノードに出さない（最適化対象でない共有パスのため）。引数 disposition のみ宣言。
