# Fix RC 中間言語（RC IR）と一意性チェック除去（unique-check-elim）

ステータス: 設計のみ・未実装。

RC（参照カウント）最適化の基盤として **RC IR**（評価順を固定した ANF ＋ 明示 retain/release ＋ ローカル名グローバル一意）を導入し、その上で uniqueness 解析・unique-check-elim・将来の RC 最適化（retain/release 相殺・reuse・borrow・順序スケジューリング）を行う。

用語: 値の形＝`Shape`（boxed はロケーションへのポインタ）、参照カウント＝`CTRefCnt`（`Static(n)`＝静的に正確に n／`Dynamic`＝不明、`Static(1)`=unique）をコンパイル時の仮想ヒープ `heap: Loc→Cell` に持つ、関数ごとの要約＝`UniqSignature`。

## 0. 動機（なぜ RC IR か）

現状 Fix は retain/release を codegen（`generator.rs`）で**暗黙に**挿入する: `Scope.used_later`（377 付近）＋ `scope_lock_as_used_later`/`unlock`（731/740、評価順に沿って増減）、`get_scoped_obj`（709）が used_later なら `build_retain`、scope 退出で `release`（1462）。

AST レベルで RC 最適化をやると、この **last-use 解析＋RC 挿入を再導出（重複）**することになり、codegen との同期が崩れると不健全になる。

解決: **codegen から last-use 解析＋retain/release 挿入を分離して「IR 生成」にし**、明示 RC を持つ RC IR を作る。すると:
- codegen も最適化も「明示 RC を読む」だけ（ロジック単一化・同期問題の消滅）。
- uniqueness 解析は RC IR を **interpret** する素直な処理になる。
- retain/release 相殺・reuse・borrow・順序スケジューリングも同じ IR 上に乗る。

先行例: Swift の SIL。

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
1. **ANF**: compound op の引数は atom＝`Var`（リテラルも `LLVM` で let 束縛するので atom は変数のみ）。compound は `let` 束縛。評価順は構文（let 逐次）で固定。未順序の兄弟評価位置は存在しない（全部 atom）。
2. **明示 retain/release**: dup/drop が明示ノード。codegen は RC 判断をしない。
3. **ローカル名グローバル一意**: 全束縛変数名が一意（シャドー禁止、スコープ外の名前との衝突も禁止）。**RC IR lowering が変数生成時に fresh なグローバル一意名を発番**する（名前カウンタ＋AST名→新名の env で traverse、AST のシャドーも解消）＝**構築により一意**。**全 RC IR 変換が保存**（clone・特殊化時も fresh 名を発番）。→ 名前→束縛が一意に解決でき、解析はスコープ追跡不要・env が単純。
4. **順序＝データ依存**: IO 順序は IOState threading のデータ依存（`IO a = IOState -> (IOState, a)`、bind が前段出力 iostate を次段へ）として表現され、lowering 後も保たれる。`unsafe_perform` 等の unsafe 脱出は契約上もとから順序保証外（責務外）。
5. **ソースロケーション保持**: 各 RC IR ノードは元 AST の source span（`ExprNode.source` 相当）を保持し、lowering で伝播する。lowering が新設するノード（`Retain`/`Release` など元 AST に対応式が無いもの）は、対象値や囲む式の span を継ぐ。→ codegen が DWARF debug location を正しく出せ、デバッガの行/関数対応・backtrace・サニタイザ報告・`create_debug_subprogram` 等が現状どおり機能する。特殊化（clone）・相殺・unique-check-elim など RC IR 上の全変換も span を保存する。
6. **明示的型付け（explicitly typed）**: 各 RC IR ノード・各変数は具体型（特殊化後の monomorphic な型）を明示保持する。codegen の LLVM 型生成、§3 の uniqueness 解析（型からレイアウト＝`Shape` を導出: boxed/unboxed・struct フィールド・union variant・配列要素・closure ペア）、デバッグ情報がこれを使う。全変換が保存する。

### 1.2 データ型（P1 で確定）

**継続入れ子・単一 enum（終端子つき）**形式を採る。`Let`/`Retain`/`Release`/`MarkGlobal`/`MarkThreaded` は継続を持つ「文」的ノード、`Ret`/`Match` が終端子（継続の終わり）。この形は reset/reuse・borrow 等の RC 最適化を載せやすい。Fix の既存 AST（再帰的）とも同形で流用しやすい。各ノード・各変数は span と具体型を持つ（§1.1-5,6。下では省略）。
```rust
// プログラム = トップレベル関数定義 ＋ global 初期化（§8）
struct RcProgram { funcs: Map<FuncRef, RcFunc>, globals: Vec<RcGlobalInit>, entry: FuncRef }
struct RcFunc {                  // lift した lambda body・global 関数・uncurry funptr 版を一様に表す
    name: FuncRef,
    params: Vec<Var>,            // n 引数。closure ABI は必ず n=1（arrow はカリー化）、funptr ABI は n>=1（uncurry。<= FUNPTR_ARGS_MAX）
    cap: Option<Var>,            // Some(cap)=closure ABI（末尾の捕捉ポインタ引数。body が cap から getter で捕捉を取り出す）／None=funptr ABI（捕捉なし）
    body: RcExpr,                // ＋ 型・span
}
enum RcExpr {
    Let(Var, RcRhs, Box<RcExpr>),          // op の結果を Var（単一）に束縛 → 継続（ANF）
    Retain(Var, RcState, Box<RcExpr>),     // x を dup（root +1） → 継続。RcState で state ディスパッチを選択
    Release(Var, RcState, Box<RcExpr>),    // x を drop（-1; 0 で解放、構造辿り） → 継続
    MarkGlobal(Var, Box<RcExpr>),          // x の到達グラフを GLOBAL state に（= mark_global）。global 値初期化で発行 → 継続
    MarkThreaded(Var, Box<RcExpr>),       // x の refcount を threaded(atomic) mode に（= mark_threaded）。別スレッド送信前 → 継続
    Ret(Var),                             // 終端子
    Match(Var, Vec<MatchArm>),            // 終端子。唯一の分岐構造（Bool もここ）
}
struct MatchArm { variant: usize, payload: Var, body: RcExpr }  // payload を単一 Var に束縛（さらなる分解は getter で）
enum RcRhs {
    App(Var /*callee: closure か funptr の値*/, Vec<Var>),  // closure 呼び出し・直接 funptr 呼び出し両方（codegen が型で振り分け）
    Closure(FuncRef, Vec<Var> /*捕捉*/),  // top-level 関数 + 捕捉変数列 → unboxed の {funptr, 捕捉obj ptr} ペア。捕捉 obj のみ boxed（rc 追跡）、空捕捉は null＝RC-free
    LLVM(InlineLLVM, Vec<Var>),           // 全 builtin: 算術/getter（射影）/set/mod/構築（struct/タプル/ArrayLit/union variant）/fill/リテラル(IntLit 等)/FFI/...。効果は UniqSignature で宣言（alloc 系は fresh・rc=Static(1)・引数をスロットへ）
}
// atom 位置は Var のみ（global funptr 参照を含む）。リテラルは RC 無関係な unboxed 即値で LLVM の IntLit 等として let 束縛するため Atom 型は持たない
enum RcState {            // retain/release の state ディスパッチ。lowering は既定 Unknown（健全）、§6 の state 推論が特殊化（`RcState` を注釈するだけ＝構造変更なし）
    Unknown,             // runtime で refcnt_state を見て 3-way（= 現状の retain/release）
    Local,               // LOCAL 確定: 非 atomic inc/dec、state チェック省略
    Threaded,            // THREADED 確定: atomic inc/dec、state チェック省略
    Global,              // GLOBAL 確定: codegen で no-op（コードを出さない）。最小化したければ後段 cleanup で削除可
}
enum Ownership {       // op が各引数を「所有権ごと受け取る」か「参照だけ」か。AST→RC IR lowering はこれだけで正しく明示 RC（Retain/Release）を挿入できる（§3.3 UniqSignature と同一宣言）
    Own,                 // 所有権を受け取る（C++ shared_ptr 相当）。op が消費＝内部 release か結果へ move。呼び出し側: last-use なら何もしない／非 last-use なら Retain してから呼ぶ
    Ref,                 // 所有権を受け取らない・参照のみ（weak_ptr 相当、RC 操作しない）。呼び出し側: 何もせず呼ぶ
}
// Own 引数が「内部 release」か「結果へ move」か、read か mutate かは op の内部効果で、解析用に Shape 効果（§3.3）が宣言する（どのスロットへ行くか・read/mutate）。
// `Ownership` は RC IR ノードのフィールドではなく `InlineLLVM`（`LLVMGenerator`）から引数位置ごとに取得する: 例 `InlineLLVM::arg_ownership(i) -> Ownership`（variant ごとに dispatch。既定は op 意味から決まりほぼ `Own`）。lowering がこれを引いて明示 RC を挿入。
// 格納されるのは `Ref` 化の上書きだけ（§6 borrow 最適化）: `can_set_ref(i) -> bool` ／ `set_ref(i)`（arg を `Ref` 化＝generator の generate がその引数の内部 release をやめるモードに切替）。`FixBody` 等は `can_set_ref` が false。
```

- **分岐は `Match` のみ（`If` を持たない）**: Bool を union 化する（std.fix: `type Bool = unbox union { _false : (), _true : () }; true = _true(); false = _false();`）。ソースの `if`/`true`/`false`/比較演算子は不変で、AST→RC IR 生成で `Expr::If(c,t,e)` を `Match(c, [_false => e, _true => t])` に desugar するだけ。性能中立（Bool-union ＝ `{i8 tag, [i8;0]}` ＝ i8。比較演算子は今も i8(0/1) を返す＝tag そのものでビット不変。FFI も i8 tag で不変。match は i8 tag の compare+branch で `if` と同等）。`&&`/`||`/`not` は `if` 経由なら desugar で吸収。
- **射影に専用ノードは持たない**: フィールド/variant payload/配列要素の取り出しは getter プリミティブ＝`LLVM` で表す。解析での扱いは §3。
- **構築も getter 同様 `LLVM`（専用 `Construct` ノードを持たない）**: struct/タプル/`ArrayLit`/union variant の構築は alloc 系 `LLVM` プリミティブで表す（射影＝getter を専用ノード化しない方針の双対）。現コンパイラは struct=`MakeStruct`・array=`ArrayLit`（専用 AST ノード）／union=`union_new`・リテラル=`IntLit` 等（InlineLLVM）と混在するが、lowering で全て alloc 系 `LLVM` に寄せて統一する。解析での扱い（fresh・rc=Static(1)・引数をスロットへ）は §3.2/§3.3。
- **`Atom` は持たず atom 位置は `Var` のみ**: リテラル（整数/浮動小数/Bool/nullptr）は現コンパイラでも `Expr::Lit` でなく InlineLLVM（`IntLit` 等）。RC 無関係な unboxed 即値なので `LLVM` で let 束縛し、atom は `Var`（global funptr 参照を含む）に一本化（文字列リテラルは boxed `Array U8` ＝ alloc 系 `LLVM` 側）。
- **`Let` は単一 Var のみ（Pattern を持たない）**: `let (x,y)=s` 等の struct/タプル destructure は **getter プリミティブ列 ＋ `Release(container)`** に lower（役割分担: 構造分解は getter、union 分岐は `Match`）。lowering は現状の destructure codegen（`get_struct_fields`/`get_union_value`）が行う RC 最小化（move-out）を踏襲し、**最初から必要最小の retain/release だけを挿入する**。
- **RC IR は nested lambda を持たない**: lowering が全 lambda を top-level RC IR 関数へ lift し、使用箇所を `Closure(func, 捕捉)` に変換する（クロージャ生成を明示）。各関数の RC が閉じる。クロージャ値は unboxed の `{funptr, 捕捉オブジェクトへのポインタ}` ペアで、捕捉オブジェクトのみ boxed（rc 追跡。空捕捉は nullptr で複製しても RC 増減なし）。`FuncRef` ＝ top-level RC IR 関数への参照（名前/id。lift した lambda body。codegen で funptr に解決）。`Closure` の捕捉リスト（`Vec<Var>`）は**順序つきでノードに保持**する。順序＝捕捉オブジェクトのスロット順＝lifted 関数が cap から射影する順（`cap.@0, cap.@1, …`）。`Closure` 生成時の格納順と lifted 関数の射影順は同順でなければならず、全変換がこの対応を崩さない（捕捉の並べ替え/追加/削除は両側を揃えて行う）。
- **トップレベル定義は `RcFunc`、クロージャ値生成は `RcRhs::Closure`**: lift した lambda body・global 関数・uncurry funptr 版はすべて `RcFunc`。**クロージャは必ず arity-1**（arrow 型はカリー化される）で closure ABI の関数は `(arg, cap)` の2引数（cap が末尾、body が cap から捕捉を射影）。多引数＋捕捉は入れ子の arity-1 クロージャになる（多引数クロージャは存在しない）。**多引数は funptr のみ**（捕捉なし n 引数。uncurry が global lambda から `name#funptr{n}` を生成、`n ≤ FUNPTR_ARGS_MAX`）。`Closure(FuncRef, captures)` は実行時クロージャ値（unboxed `{funptr, 捕捉obj}`）を生成し、捕捉 obj は heap 値で `RcFunc` の一部ではない。`App` は callee 型で振り分け（funptr＝直接 n 引数、closure＝arg ＋抽出した cap）。
- **RC 効果の宣言（lowering と解析が RC を読めること）**: **AST→RC IR lowering** は各引数の `Ownership`（`Own`／`Ref`。§1.2）を見て正しく明示 RC を挿入する——`Own` かつ非 last-use なら使用前に `Retain`（last-use なら何もしない）、`Ref` なら何もしない。codegen は出来上がった明示 `Retain`/`Release` を inc/dec へ翻訳するだけ（`Own` で外部化しない op の内部 release はその op の generate 側）。`mark_global`/`mark_threaded`→`MarkGlobal`/`MarkThreaded`。`make_array_unique` 等の force-unique 内 clone は op の意味に内包する。**外に出すのは「最適化で消せる/動かせる & 消したい RC」だけ**（last-use/ownership の RC。相殺/reuse/borrow が効く）。**最適化で消えては困る意味的 RC は opaque な op の内部に埋めたまま**にする（例: `with_retained` の「呼び出し中 x を shared に見せる」retain。外に出すと相殺で消えて壊れる）＝構築により保護。外に出せない内部 RC は宣言で足りる。`MarkGlobal` 以外に「is-global チェック」専用ノードは不要（状態チェックは状態不明時の runtime `Retain`/`Release` に内包。静的に global/local と分かれば `RcState` を `Global`(no-op)/`Local`(チェック省略) に特殊化＝将来の state 最適化、§6）。**全 InlineLLVM の `Ownership` と `Ref` 化/外部化可否は P1 で全件監査**（`fix`/bulk array は `Ref` 化できない候補。§8）。

### 1.3 意味論（refcount）
- alloc 系（構築・`fill`/`empty`・クロージャ捕捉 obj 等）→ 新規確保、root refcount 1（LOCAL = unique）。クロージャ値自体は unboxed の {funptr, 捕捉obj ptr} で RC 無し（捕捉 obj のみ rc 追跡。空捕捉は null）。
- `Retain(x)` → x の指す値の root refcount +1。`Release(x)` → −1（0 で解放。release は構造を辿る＝既存 `build_release_mark` ＋ `TraverserWorkType`）。
- state（`REFCNT_STATE_LOCAL`/`THREADED`/`GLOBAL`, constants.rs:118-120）は既存どおり。**global は retain/release が no-op、決して unique にならない**。in-place は LOCAL ∧ refcount==1 のときのみ。
- op は引数 atom の参照を消費（move）。同じ値を複数回使うには事前に `Retain`。

### 1.4 AST → RC IR lowering（`generator.rs` から RC を抽出）
`generator.rs` の RC 決定ロジックをこのパスへ移す:
- **ANF 化**（兄弟位置を atom 化、effectful を `let` に）。
- **lambda lifting**: 残存する全 lambda（`Expr::Lam`）を top-level RC IR 関数へ持ち上げ、使用箇所を `Closure(func, 捕捉)` に変換（現状 codegen が `declare_lambda_function`/`eval_lam` でやっている「lambda ごとに関数を宣言＋捕捉でクロージャ生成」を、IR 生成に前出し）。RC IR に nested lambda は残さない。
- **名前は lowering が fresh 発番**（名前カウンタ＋AST名→新名の env で traverse、シャドー解消）＝構築により一意。
- **last-use 解析**（`Scope.used_later` / `scope_lock_as_used_later` 相当。変数ごとの最終使用を求める後ろ向きパス）。
- **明示 retain/release 挿入**（last-use 解析＝関数全体の後ろ向きパス）: (a) non-last-use の使用の前に `Retain`（複数回使用ぶんの参照を用意。現 `get_scoped_obj` の used_later→retain 相当）。(b) 変数の **last use が borrow(`Ref`) なら直後に `Release`**（consume(`Own`)/move が last use ならそこで消費されるので `Release` 無し）。(c) 未使用 let 束縛・分岐 dead 変数も `Release`。
- IOState threading はデータ依存として保持（順序自動保存）。

### 1.5 RC IR → LLVM codegen（付け替え）
- 変数 get は素の get（retain 判断なし）。`Retain`/`Release` ノード → inc/dec（release は構造辿り）。`Scope.used_later`/`scope_lock_as_used_later`/`get_scoped_obj` の retain 分岐は**消滅**。
- 非 RC 部分（クロージャ生成、FFI、struct/array レイアウト、LLVM 構築）はそのまま移植。

### 1.6 検証
- `cargo test --release` 全通過。**CI と同じ全最適化レベルで実施**する: `FIX_MAX_OPT_LEVEL` ∈ {`max`, `basic`, `none`} の各々で `cargo test --release`（codegen 付け替えは全プログラム・全最適化レベルに影響する）。
- RC の挿入数・順序・解放挙動が現状と一致（リグレッションなし）。デバッグ用に「unique と判定した値が実行時に共有なら abort」する assert ビルド。
- **デバッグ情報の一致**: debug ビルドで行/関数の対応・breakpoint・backtrace が現状どおり（span 保持の検証。§1.1-5）。
- **全ベンチマークでリグレッションなし**: `benchmark/speedtest` 全 case ＋ `fix-bench/batch` を走らせ、commit hash 付きで記録・比較。RC IR 導入は挙動を変えない（性能含め）はずなので、速度劣化が無いことを確認。
- **外部ライブラリのテスト**: 一通り走らせて確認する（ユーザが実施）。**P1 完了時にユーザへ連絡し、外部ライブラリテストの実行を依頼する**（このタイミングで手を止めて報告）。

### 1.7 lowering 例（呼び出しと cap release）

呼び出し規約は現 codegen に合わせる＝**callee 所有（consume）**: op は引数参照を消費し、関数/クロージャは自分の引数と cap を所有して release する（`implement_lambda_function`）。値を N 回使うには使用前に `Retain`（最終使用以外）。dead な束縛・分岐 dead 値だけ呼び出し側 scope の `Release` になる。

Fix:
```
concat_len : Array I64 -> Array I64 -> I64 = |a, b| a.@size + b.@size;  // global 2引数 -> uncurry で funptr 版
f : Array I64 -> Array I64 -> I64 = |arr, brr| (
    let g = |b| concat_len(arr, b);   // g は arr（boxed）を捕捉。g : Array I64 -> I64
    g(brr)                            // クロージャ呼び出し
);
```

RC IR:
```
// funptr（uncurry 版・cap なし・2引数）。a,b を所有し最終使用で release。
fn concat_len#funptr2(a, b):
    let sa = LLVM[@size](a)        // a を borrow して size 読み出し
    Release(a)                     // a 最終使用 -> release
    let sb = LLVM[@size](b)
    Release(b)                     // b 最終使用 -> release
    let r  = LLVM[add_i64](sa, sb)
    ret r

// クロージャ関数（arity-1: 引数 b ＋ 末尾 cap）。b と cap を所有。
fn g#lifted(b, cap):               // cap : *cap{ arr : Array I64 }
    let arr2 = LLVM[proj.0](cap)   // 捕捉 arr を取り出し（cap を borrow）
    Retain(arr2)                   // boxed 捕捉 -> retain out（ローカルが所有権を得る）
    Release(cap)                   // callee が cap を所有 -> 捕捉コンテナを release（cap release）
    let r2   = App(concat_len#funptr2, [arr2, b])  // funptr 呼び出し（cap 無し）。arr2,b を consume
    ret r2

fn f(arr, brr):                    // arr, brr : Array I64 を所有
    let g = Closure(g#lifted, [arr])  // cap{arr} を alloc（rc=1）。arr を1参照 move-in
    let r = App(g, [brr])             // クロージャ呼び出し。g を consume（cap 所有権 -> callee）、brr を consume（-> b）
    ret r                             // f 側に cap の release は無い（g は App が consume、cap は callee が release）
```

ポイント:
- **cap の release は callee（`g#lifted`）**。boxed 捕捉は「`Retain`(取り出し)＋`Release(cap)`」のペア（§2 の相殺が move-out に畳んで両方消せる）。空捕捉なら cap は null で `Release(cap)` は no-op。
- **呼び出し側 `f` に cap の release は無い**: クロージャ `g` を `App` が consume し所有権（cap 含む）が callee へ渡る。`g` を2回呼ぶなら使用前に `Retain(g)`（＝cap obj を retain）が入り各 callee-release と釣り合う。
- funptr（`concat_len#funptr2`）は **cap 引数なし**。例は `@size` の配列引数を `Ref`（読むだけ）として release を呼び出し側の明示 `Release` に出した形。base の `Own`（`@size` が内部 release）なら明示 `Release(a)/(b)` は出ず op 内部にある（§6 の borrow 推論が `Own`→`Ref` 化して外出しする）。
- RC 収支（arr）: cap へ1参照 → `g#lifted` で +1 → `Release(cap)` で −1 → `concat_len` で `Release(a)` −1 = 0（リーク・二重解放なし）。
- 捕捉と呼び出しの両方で同じ値を使う版（`g(arr)`）なら、`arr` は2回使用 → `f` で使用前に `Retain(arr)` が1つ入り、`concat_len` は `a==b`（同一配列・rc 2＝shared）を受け取る。

## 2. retain/release 相殺（基盤の一部・uniqueness 解析を簡単にする）

`Retain(x)` の後、その追加参照を必要とする使用が無いまま `Release(x)` が来るなら両方除去（peephole / 簡単な dataflow）。名前一意なので追跡が容易。

効果:
- 「used-later で一旦 retain したが直後の op が release（net-zero）」のような冗長 RC が消える。
- → 後段の uniqueness 解析は「**`Retain` されていない boxed 値 ＝ unique**」を素直に読めるようになる。
- clone 削減としても有用。

これは**純粋な RC 削減の最適化**（最終コードの retain/release が減って速くなる、clone も減る）。**主目的の線形ケースの precision には不要**: §3 はオブジェクトの参照カウントを `Static(n)`（正確に n）で持ち単一 Loc で `Release` が戻すので、net-zero（1→2→1）を**解析自身が回復**する（相殺前でも `Static(1)` と分かる）。よって相殺は順序自由でいつ走らせてもよく、解析の前提ではない（健全性とも無関係）。多重指しでは `Dynamic` になり戻らないので、そこの回復には相殺が効く（§3.1。ただし rare で線形ケースには影響しない）。

## 3. uniqueness 解析（RC IR を抽象解釈）

RC IR を**抽象解釈**し、**コンパイル時の仮想ヒープ**上で参照カウントだけを emulate する。各オブジェクト（ロケーション）に `CTRefCnt`（**静的に正確な refcount**＝`Static(n)` は「正確に n」、決まらなければ `Dynamic`）を持たせ、alloc（構築）=確保、`Retain`/`Release`=増減、射影=エイリアス、を辿る。再帰は実回数まわさず、有限領域上の**不動点**で畳む（合流で join＝rc が一致なら保持・不一致なら `Dynamic`。この不一致→`Dynamic` が widening を兼ね不動点を停止させる）。正確に決まらない場面（不一致 join・多重指しへの retain/release・summary・opaque op）は `Dynamic`（静的に unique と確定できない）になる。`Dynamic` では unique-check-elim が force-unique を除去せず、**force-unique の実行時 uniqueness チェックが残る**（§4。実行時に unique なら in-place、shared なら clone＝現状動作）。

**なぜ per-var の木でなく仮想ヒープか**: refcount は「オブジェクトの属性」であって「変数の属性」ではない。retain する getter（`arr.@i`）は boxed の子に**第二の参照**を作る＝複数の変数が同一オブジェクトを指す（別名）。refcount を変数ごとに持つと別名間で同期できず不健全（`x` 経由で「unique」と誤判定して in-place 破壊しうる）。refcount を**ロケーションの cell に1つ**持てば、`Retain`/`Release` が cell を更新し、それを指す全別名が同じ値を見る。

### 3.1 状態（仮想ヒープ）
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum CTRefCnt { Static(usize), Dynamic } // オブジェクトの **静的に正確な** refcount（決まらなければ Dynamic）。Static(1)=unique、Static(n>1)=shared、Dynamic=不明（⊤。unique と確定できない＝実行時チェックが残る）。
// 意味: Static(n) ＝「静的に正確に n」。正確に決まらなければ Dynamic。alloc→Static(1)、単一 Loc・strong の Retain→+1/Release→−1、join→一致なら保持・不一致（異なる非 Bottom の Static）なら Dynamic。
//   Dynamic 化する不確定要因: 不一致 join／多重指しへの retain/release（どの Loc がどれだけ ±1 か不明）／summary loc／opaque op・global・未知 FFI。過小評価（unique と誤判定）は起こさない。
// 終端性: 不一致→Dynamic が widening。straight-line の Static(n) は retain 数で有限、ループ/再帰の不動点は back-edge join の不一致→Dynamic で停止（K-cap 不要）。
// 散文では unique＝Static(1)、shared＝Static(n>1)/Dynamic の意。

// 抽象ロケーション: アロケーションサイト（構築/クロージャ捕捉obj/force-unique 結果/不明 alloc のプログラム地点）で抽象化。
//   サイトは有限個 → Loc も有限。再帰で同一サイトを再訪すると同じ Loc に集約＝summary loc（複数の具体オブジェクトを表す）。
//   + Top: 外部・global・未知 FFI 由来。常に Dynamic。
type Loc;                  // AllocSite | Summary(AllocSite) | Top
type PtsTo = Set<Loc>;     // points-to 集合（分岐合流で複数 Loc を指しうる。通常は単集合）

#[derive(Clone, PartialEq, Eq)]
enum Shape {                  // env / フィールドが持つ「値の形」。boxed はインラインせず必ずポインタ1個で切れる
    Unboxed,                 // scalar（refcount 無し）
    UnboxedAgg(Vec<Shape>),   // タプル/unboxed struct/unboxed union/クロージャ（refcount 無し、子を持つ。クロージャ＝[Unboxed funptr, Boxed 捕捉obj]、空捕捉は Boxed({})＝null）
    Boxed(PtsTo),            // boxed 値＝仮想ヒープへのポインタ（rc と中身は cell 側）。空集合 {}=null（指す先なし。空捕捉クロージャ等。RC op は no-op）。Bottom とは別（Bottom=無情報/到達不能）
    Bottom,                  // ⊥（到達しない・union の不在 variant）。join 単位元
}

struct Cell { rc: CTRefCnt, contents: Shape } // 1 オブジェクト分。Array は contents=要素 Shape、boxed struct は UnboxedAgg(フィールド)、boxed union は variant ごと
struct State { env: Map<Var, Shape>, heap: Map<Loc, Cell> } // heap ＝ 仮想ヒープ
```

**再帰型は有限グラフ**: `Shape` は boxed のところで必ず `Boxed(PtsTo)`（ポインタ）に切れるので `Shape` 単体は常に有限。`List` 等の再帰は **cell の contents が別の Loc（自分自身も可）を指す巡回グラフ**になり、Loc が有限（アロケーションサイト抽象）なので全体も有限。よって木の打ち切りノードは要らず、有限性は Loc 集合の有限性が担保する。再帰で構築される構造（`List` 等）は同一サイト → summary loc に集約（rc は `Dynamic`）。

**cell rc の更新**（`Retain`/`Release` による cell 更新の話。`is_unique` の読み取り §3.2 末尾とは別軸）: points-to が**単集合 `{L}` かつ `L` 非 summary** のときだけ `L.rc` を正確に ±1（`Retain`→+1、`Release`→−1。`Static(2)`→`Static(1)` の unique 回復ができる＝線形スレッドの精度）。多重指し or summary loc では「どの Loc がどれだけ ±1 されるか」が静的に決まらないので、対象候補の cell を `Dynamic` にする（`Static(n)` は『正確に n』の意味なので、決まらなければ `Dynamic`）。読み取りは別軸: 多重指しでも各候補 cell が `Static(1)` なら `is_unique` は unique（retain/release 操作をしていなければ候補は `Static(1)` のまま）。

**net-zero 回復が効く範囲**: retain→release の打ち消しを解析自身が戻せるのは**単集合・strong の線形ケースだけ**。多重指しでは `Dynamic` になり戻らない。この取りこぼしは retain/release 相殺（§2）が net-zero ペアごと消すことで解消する（rare ケース）。

**join**（合流・不動点）: `Shape` は pointwise（`Bottom` 単位元、`Unboxed` 同士、`UnboxedAgg` は zip、`Boxed(P1)⊔Boxed(P2)=Boxed(P1∪P2)`）。`heap` は Loc ごとに cell を join（`rc` は一致なら保持・不一致なら `Dynamic`、`contents` は join）。**points-to 集合は「指しうる候補」の列挙（各 Loc を個別に追跡・rc も個別）であり summary loc とは別物**: 分岐で別アロケーションを指すと `{L_A, L_B}` になるが、各 cell の rc は join で `Static(1)` のまま（潰して `Dynamic` にはしない。summary loc は再帰で**共存**する複数オブジェクトを1つに畳んだ場合のみ）。PtsTo は有限 Loc 集合の部分集合なので有界（サイズ widening は不要）。rc の不一致→`Dynamic`（widening）と Loc の有限性で不動点は停止。
補助: `top_of(ty)`（保守的 ⊤＝boxed は `Boxed({Top})`、`heap[Top].rc=Dynamic`）。

### 3.2 interpret 規則（RC IR を辿る）
`State`（env, heap）を更新しながら `RcExpr` を順に処理:
- `Let(x, Closure(_, captures), k)`: クロージャ値は unboxed pair `{funptr, capture_ptr}`。捕捉が非空なら捕捉 cell `L`（`Cell{ Static(1), UnboxedAgg(captures の Shape) }`）を alloc し `env[x] = UnboxedAgg([Unboxed /*funptr*/, Boxed({L})])`、捕捉は move-in。捕捉が空なら alloc 無しで `env[x] = UnboxedAgg([Unboxed, Boxed({})])`（null＝RC-free）。これにより捕捉された配列等を追える。
- `Let(x, LLVM(prim, args), k)`: prim の宣言 `UniqSignature`（§3.3）を適用。
  - 射影（getter）`x = get(a, i)`: `env[x]` ＝ `a` の指す cell の `contents` を i 射影した Shape（boxed なら同じ Loc を指す）。**retain する getter**（`Array::@` 等、配列を残して要素を複製）は子 Loc の rc を +1 → `env[x]` と `a` の field i が同一 Loc を共有＝**別名を捕捉**（以後 `Retain`/`Release` が同じ cell を更新）。**move-out する linear get**（`mod`/`act` の `_unsafe..unretained`）は rc 据え置き（参照がフィールドから出るだけ）。
  - force-unique 系（`set`/`mod`/`act`）: 結果は新規 or 再利用 Loc で `rc=Static(1)`、格納値は要素位置へ。
  - 構築（alloc 系。struct/タプル/`ArrayLit`/union variant）: 新規 Loc `L`（初訪は新規、再訪は summary 化）に `Cell{ rc: Static(1)（summary なら join→Dynamic）, contents: args の Shape をスロットへ }`、`env[x]=Boxed({L})`。args は move-in（rc 不変）。
- `Let(x, App(f, args), k)`: `f` の `UniqSignature` を適用（結果 Shape ／ 各 arg cell の rc 効果 ／ 結果が arg を別名化するか、を反映。クロージャ値でも funptr でも対象関数の要約を引く）。
- `Retain(x, k)`: `env[x]` の root PtsTo が単集合・非 summary なら その Loc の rc を正確に +1。多重指し or summary は対象候補の cell を `Dynamic`（どの Loc がどれだけ増えるか不明）。全別名が同一 cell を見るので同期する。
- `Release(x, k)`: 単集合 `{L}` かつ非 summary なら `L.rc` を正確に −1（**`Static(2)`→`Static(1)` で unique 回復**、`Static(1)`→0 で解放し contents を辿って子を再帰 Release）。多重指し or summary は対象候補の cell を `Dynamic`。単一 Loc では `Release` が count を戻すので net-zero（1→2→1）を**解析自身が回復**する。
- `Match(a, arms)`: 各 arm を**分岐前 State のコピー**から解析し、結果 State を `join`。不在 variant の payload は `Bottom`。Bool もここ（2 variant）。
- `Ret(x)`: 結果 ＝ `env[x]`。
- global 参照 → `Boxed({Top})`（`heap[Top].rc=Dynamic`）。

要点: **refcount を仮想ヒープの cell で追い、別名は同一 Loc で共有**。線形スレッド（`a1=set(a0,..); a2=set(a1,..)`）は単一 Loc で rc が `Static(1)` を保ち、retain getter で生じた別名は子 Loc の rc が `Static(2)` になって正しく shared と分かる。

**uniqueness クエリ**（unique-check-elim が使う、§4）: `is_unique(x) = ∀ L ∈ PtsTo(env[x]): heap[L].rc == Static(1)`（指しうる全候補が `Static(1)`）。例: `let arr = if c { [1,2,3] } else { [4,5,6] }` は `env[arr]={L_A,L_B}` で両 cell とも `Static(1)` なので **unique**（実行時はどちらか一方の object で、その rc は正確に 1）。候補に `Dynamic`・summary loc が混じれば非 unique（保守的）。

詳細（getter ごとの retain 有無、summary loc の扱いの精密化）は P2 で確定。

### 3.3 関数の効果（`UniqSignature`, per-input-key concrete）
関数/op の効果は、呼び出し地点 `let r = f(a0..a_{n-1})` で `State`(env,heap) を更新する変換。引数を超えた参照を持たないので、効果は**引数からの相対参照**で記述できる。
```rust
struct UniqSignature {        // 1 回の適用で State をどう変えるか
    actions: Vec<Action>,     // 順に適用: rc 増減・新規 alloc・スロット格納
    result:  ShapeRef,        // 適用後の結果 r の Shape
}
enum ShapeRef {               // 効果内の「値/場所」を引数から相対指定
    Unboxed,                      // rc 無しスカラー
    Arg(usize),                   // a_i をそのまま（result が a_i を alias）
    Field(Box<ShapeRef>, usize),  // 射影（cell の slot。getter。boxed なら同じ子 Loc を指す）
    Fresh(FreshId),               // 下の Alloc で作った新規 Loc を指す Boxed
    Agg(Vec<ShapeRef>),           // unboxed 集約（タプル/struct/closure ペア）
    Null,                         // Boxed({})（空捕捉）
}
enum Action {
    Alloc   { id: FreshId, rc: CTRefCnt, contents: ShapeRef }, // 新規 cell（通常 Static(1)）
    Store   { cell: ShapeRef, slot: usize, value: ShapeRef },  // cell.slot ← value（in-place set 等）
    Retain  { target: ShapeRef },  // target の Loc rc を正確に +1（単一・strong。多重/summary は Dynamic）
    Release { target: ShapeRef },  // target の Loc rc を正確に −1（同上。引数 consume・旧要素 drop 等）
}
type InputKey = Vec<ArgKey>;  // メモ化キー: 引数 Shape を有限に要約（uniqueness が効果を変えるので最低限 unique/shared）
enum ArgKey { Unboxed, UniqueBoxed, SharedBoxed /* 必要なら子の要約も。粒度は精度ノブ、P2 */ }
```
**適用** `apply(state, args, sig)`: `actions` を順に State へ反映（`Alloc`→cell 追加、`Store`→cell スロット更新、`Retain`/`Release`→対象 Loc の rc を exact ±1／多重・summary は `Dynamic`）し、`env[r] = resolve(sig.result)`（`Arg(i)`→`env[a_i]`、`Field`→射影、`Fresh`→`Boxed({新 Loc})`、`Agg`→`UnboxedAgg`、`Null`→`Boxed({})`）。

**保持場所**:
- **ソース関数 = 推論**: 解析が関数ごとに `Map<InputKey, UniqSignature>` を memo（`FuncRef` キー。IR の `RcFunc` とは別の解析テーブルで、ノードに解析状態を載せない）。入力キーごとに body を §3.2 で解析して埋める。§4.1 の特殊化 worklist と共有。再帰は不動点（初期 `Bottom`）。
- **プリミティブ（`InlineLLVM`/FFI）= 宣言**: `LLVMGenerator::signature(&self, key: &InputKey) -> UniqSignature`（variant ごとに宣言、入力キー依存可）。`Ownership`（§1.2、lowering 用）は同じ宣言の射影＝`Release{Arg(i)}` あり or result/fresh へ move なら `Own`、rc 操作も move も無ければ `Ref`。
- global / 未知 FFI = `Dynamic`（`Boxed({Top})`）。assert ビルドで不健全な claim を実行時検出。

例（`elem` は要素 slot。boxed 要素では旧要素の `Release` も要るが簡略化）:
- **retain getter** `Array::@(i, arr)`: `actions=[Retain{ Field(Arg(arr), elem) }]`、`result=Field(Arg(arr), elem)`（要素を複製＝子 Loc rc +1、配列は残す）。
- **set** `set(i, v, arr)`: `arr` が `UniqueBoxed` なら in-place＝`actions=[Store{ cell: Arg(arr), slot: elem, value: Arg(v) }]`、`result=Arg(arr)`（**同 Loc・rc 不変**、v は要素へ move。これでループ `arr=arr.set(..)` が同一 Loc を保ち `Static(1)` 継続）。`SharedBoxed` なら clone＝`actions=[Alloc{f0, Static(1), <要素を v にした Shape>}, Release{Arg(arr)}]`、`result=Fresh(f0)`。
- **構築** `MakeStruct{a,b}`: `actions=[Alloc{f0, Static(1), Agg([Arg(a),Arg(b)])}]`、`result=Fresh(f0)`（引数 move-in）。
- **Closure**(g,[c0]): `actions=[Alloc{f0 /*捕捉obj*/, Static(1), Agg([Arg(c0)])}]`、`result=Agg([Unboxed /*funptr*/, Fresh(f0)])`。
- **fill** `fill(n, x)`: `actions=[Retain{Arg(x)}（多スロットに複製→子 rc Dynamic）, Alloc{f0, Static(1), Arg(x)}]`、`result=Fresh(f0)`。`boxed_to_retained_ptr`→引数 escape（`Retain` ＋ 結果 unboxed ptr）。

## 4. unique-check-elim

force-unique を含む `LLVM` op（`set`/`mod`/`act` 系）で、対象 boxed 値が §3 の `is_unique`（指しうる全候補が `Static(1)`）かつ LOCAL と証明できれば、その RC IR の `LLVM` ノードを **force-unique を行わない版に差し替える**（証明できない＝`Dynamic`/`Static(n>1)`/summary では除去せず、force-unique の実行時 uniqueness チェックを残す＝現状動作）。結果は force-unique 後どのみち unique なので、ループ `let arr = arr.set(…)` で 2 回目以降の入力が unique になり「**初回 checked・以降 unchecked**」が自然に出る。

### 4.1 特殊化（uniqueness 駆動、RC IR 上）
`RcFunc` を、流れてくる引数 uniqueness（§3 の入力 `Shape`）をキーに clone する。呼び出し地点で引数が unique なら unique 用 clone を、shared なら別 clone（または original）を呼ぶ。worklist で不動点（§3.3 の per-input-key 解析と共有）。clone は fresh 名を発番し（名前グローバル一意 §1.1-3 を保存）一意な clone 名を付ける。未使用になった `RcFunc`（original/clone）は RC IR 上の dead-function 除去で掃除する。

### 4.2 force-unique の除去（RC IR の `LLVM` ノード差し替え）
clone した `RcFunc` の body 中で force-unique を担う `LLVM`(InlineLLVM) ノードを、force-unique しない版（`InlineLLVM` の `force_unique=false`／unchecked generator）に差し替える（新規ノードを作って置換。共有呼び出し地点側の clone は checked のまま）。`force_unique` フラグの有無:

| 操作 | force-unique の所在 | フラグ |
|---|---|---|
| Array `set` | `InlineLLVMArraySetBody`（無条件 `make_array_unique`, builtin.rs:2170） | **無し→追加** |
| Array `mod`/`act_identity`/`act_tuple2` | `_unsafe_get_linear_bounds_unchecked_unretained`（`force_unique`, builtin.rs:1901/1936） | 既存→`false` |
| struct `mod_<field>` | `#punch_fu_{field}`（`InlineLLVMStructPunchBody`{true}, `make_struct_unique` @2656） | 既存（非 fu punch あり）→`false` |
| struct `set_<field>` | `InlineLLVMStructSetBody`（無条件 `make_struct_unique`, builtin.rs:3580） | **無し→追加** |
| struct `act_<field>` | 非 fu punch を既に使用（unique 保証） | 対応不要 |

`Const` functor の `act` は force-unique なし（対象外）。generic な `act`（任意 functor）は `unsafe_is_unique` の once-per-call チェックで follow-on。act の functor 特殊化（`optimize_act`。Identity/Const/Tuple2、ホットな `mod`=Identity 含む）は lowering 前に行い、RC IR には上表の具体形（force-unique を持つ op）が現れる——それを §4 が除去する。

## 5. 適用対象・検証

- マイクロ: `batch/arrayrw{,_unsafe,_fn}`、`fannkuch`。正しさ: `cargo test --release`（全最適化レベル、§1.6）＋**共有値テスト**（2 箇所に格納して破壊しないこと）。回帰: `benchmark/speedtest`。assert ビルドで不健全検出。
- 一意文脈でチェックが消える（IR/asm に `build_branch_by_is_unique` 由来分岐が残らない or cachegrind 命令数低下）＋共有文脈で消えない（クローンされる）を各セルで確認:

| 対象 | set | mod | act(Id) | act(Tuple2) | act(Const) | 備考 |
|---|:--:|:--:|:--:|:--:|:--:|---|
| Array | ✓ | ✓ | ✓ | ✓ | — | Const は getter |
| boxed struct field | ✓ | ✓ | ✓ | ✓ | — | `make_struct_unique` を外す |
| union | — | — | — | — | — | `mod_<variant>` は force-unique を踏まない＝対象外 |

入れ子伝播も確認: タプル内配列 `loop((cnt, arr), …)`、struct 内配列・配列内 struct、union 内配列（`LoopState`）。

## 6. 将来の RC 最適化（同じ RC IR 上）
- **retain/release 相殺**（§2）。純粋な perf（冗長 RC 削減）。unique-check-elim の前提ではない（§3 解析が net-zero を回復するため）。
- **reuse**（`Release` した alloc を直後の alloc で再利用＝in-place 再確保）。
- **borrow 推論（`Own`→`Ref`）**: `InlineLLVM`（`LLVMGenerator`）に per-arg の `can_set_ref(i) -> bool`／`set_ref(i)` を持たせる（§1.2）。関数引数は callee 所有＝`Own`。読むだけの引数を `Ref`（借用）に宣言できれば、呼び出し側の `Retain` と op 内部の `release` が不要になる。InlineLLVM の read-only op（§8 分類A）は既に `noretain`＋last-use release で借用的なので `Ref` に素直に対応。`Own` のまま retain してから読む引数があれば `Ref` 化＋release 外出し＋相殺（§2）で削減する。`Ref` 化できない op もある（`fix` 内側の InlineLLVM 等）。
- **順序スケジューリング**（意味を保つ範囲で評価順を並べ替え in-place 機会を増やす。例: `f(arr.set(0,42), arr.@0)` を `arr.@0` 先に並べ替えると set が in-place 化し clone が消える）。
- **state 推論**（各値の refcount-state＝LOCAL/THREADED/GLOBAL を静的に決め、RC・状態チェック・`mark_threaded` を省く）。proven-global → `RcState::Global`（codegen no-op）。proven-local → `RcState::Local`（状態チェック省略）。送信値が proven-deeply-unique → `MarkThreaded` 省略。`MarkGlobal` も静的に分かる範囲で最適化。
- **境界チェック除去**（`idx ∈ [0,size)` を証明し完全 unchecked へ。一意性除去と合成でベクトル化 0.20x）。
- **match-of-known-constructor / case-of-case**（LLVM 未実施を確認の上）。

## 7. マイルストーン
- **P0（P1 前）**: **デバッグ情報の E2E テストを追加**してベースライン化。現状その回帰テストが無いため、`fix build -g`（DWARF 付き）でビルドした小プログラムを **gdb 駆動**（`gdb -batch`: `break main.fix:N` → run → backtrace）で検査する統合テストを作る（CLAUDE.md 規約: サンプルを tempdir にコピー、`fix`/`gdb` を `Command` 実行）。assert は file:line の解決・停止・スタックの行情報（マングル名非依存）。補助で bundled `llvm-dwarfdump` の構造 assert も可。**現 main で通すこと**＝P1 の「デバッグ情報一致」(§1.6) の比較対象。ツール: `/usr/bin/gdb` あり、`llvm-dwarfdump` は `/home/maruyama/llvm-17.0.6/bin/`（system には無し）。
- **P0.5（P1 前提）**: **Bool を union 化**（std.fix: `type Bool = unbox union {_false,_true}; true=_true(); false=_false();` ＋ 比較演算子の結果型 ＋ FFI Bool↔i8 tag）。これが `If` を IR から落とす前提（`If`→`Match` desugar は P1 lowering 内）。性能中立（Bool-union＝i8）。de-risk するなら現 `eval_if` を union Bool 対応にして先行検証、または P1 で `eval_if` 撤去と同時。
- **P1**: RC IR 型 ＋ AST→RC IR lowering（`generator.rs` から RC 抽出。名前は lowering が fresh 発番）＋ codegen 付け替え ＋ 全テスト再検証。**最大の山**。完了ゲート: `cargo test --release` 全最適化レベル（`FIX_MAX_OPT_LEVEL` max/basic/none、§1.6）で全通過・全ベンチでリグレッションなし・デバッグ情報一致（§1.6）を満たし、**ユーザに連絡して外部ライブラリテストを依頼してから次フェーズへ**。lowering は現 codegen の RC（move-out/last-use ＝既に最小 RC）を踏襲。retain/release 相殺は §6/P4 の perf 磨きに回す（§3 解析が net-zero を回復するので相殺前でも動く）。
- **P2**: uniqueness 解析（仮想ヒープ＋`UniqSignature` で RC IR を抽象解釈）。read-only ログから始め arrayrw のループ `set` を unique・共有テストを shared と判定することを確認。
- **P3**: unique-check-elim（force-unique 除去 ＋ 特殊化）。arrayrw/fannkuch 計測、全テスト。
- **P4**: reuse / borrow / 順序スケジューリング / 境界チェック除去 等。

## 8. リスク・未解決
- **P1 の codegen 付け替えの再検証コスト・範囲**が最大リスク（全プログラムに影響）。段階導入できるか（一部関数だけ RC IR 経由、等）も検討。
- `Release` 後の uniqueness 回復は §3 解析が行う（`Release`=−1 で `Static(2)`→`Static(1)`）。§2 の相殺とは独立。
- ローカル名一意の**全変換での保存**（lowering は fresh 発番で構築的に一意。clone/特殊化は fresh 名発番で freshen）。
- getter（射影）の retain 有無・`UniqSignature` の不動点収束・threaded state・FFI escape の RC IR での扱い。捕捉クロージャは `Closure` の captures を cell の contents として追えるが、共有/別名の健全性は検証する。
- 別名健全性は refcount を cell に1つ持つことで担保（§3）。summary loc・PtsTo 集約・多重指しの `Dynamic` 化など精度調整は P2 で詰める。
### 決定事項・要確認
- **（決定）env を仮想ヒープ（store）にする**（§3）: refcount を変数ごとでなくロケーションの cell に持つ。理由: retain する getter が boxed の子に第二参照を作り、変数ごとの木では別名間で同期できず不健全（`x` 経由で unique 誤判定 → in-place 破壊）。cell に refcount を1つ持てば `Retain`/`Release` が全別名に同期する。再帰型は有限 Loc（アロケーションサイト抽象）の巡回グラフで表現するので木の打ち切りノードは不要。線形ケースは単一 Loc で正確に追い、別名・summary loc は `Dynamic`。
- **（決定）`Construct` ノードを設けず構築も `LLVM`**（§1.2）: 集約構築（struct/タプル/`ArrayLit`/union variant）は alloc 系 `LLVM` プリミティブ＋`UniqSignature`（fresh・rc=Static(1)・引数をスロットへ）で表す。InlineLLVM が効果を宣言する設計なので fresh=unique は解析に伝わり、専用ノードは不要（射影＝getter を専用ノード化しない方針の双対）。
- **（決定）参照カウントを `Static(n)`（静的に正確に n）|`Dynamic`（不明）で表す**（`CTRefCnt`、§3.1）: alloc=Static(1)、単一 Loc の `Retain`=+1／`Release`=−1（net-zero を回復＝`Static(2)`→`Static(1)`）、分岐 join＝一致なら保持・不一致なら `Dynamic`、多重指しへの retain/release・summary・opaque も `Dynamic`（これらが widening を兼ね終端性を担保。K-cap 不要）。これにより: (a) 要約は入力ごとの concrete な数値（明示 RC が駆動する）、(b) **retain/release 相殺は順序自由な純粋 perf**（解析が `Release` で net-zero を回復するので、相殺の前後どちらでも `Static(1)` を得る）。§3.3 は per-key memoize。
- **（決定）Bool→union（P0.5、§7）**: std.fix 定義＋比較演算子の結果型＋FFI（Bool↔i8 tag、`_false`=0/`_true`=1）。`If`→`Match` desugar は P1 lowering 内。要確認: 比較 InlineLLVM の結果構築・`&&`/`||`/`not`・typecheck が union Bool で通るか（ビットは i8 不変）。
- **（決定）global 値の表現**: global 初期化を RC IR（init）として表し `MarkGlobal` を init で発行。参照は atom で解析は `Dynamic`。program = top-level 関数集合 ＋ global init。現状の global 機構（lazy/eager・mark_global 発火点）は P1 実装時に確認。
- **（決定）lowering サブパス順**: AST 正規化（ANF 化 → lambda lift → `If`→`Match` desugar → destructure→getter → fresh 命名）→ 最後に last-use 解析＋明示 retain/release 挿入で RC IR 生成（形と名前が確定してから RC を載せる）。
- **（調査済み）RC site 監査の規模**: codegen の RC は `generator.rs` ~38・`builtin.rs` ~29（InlineLLVM `generate` 内部の release/retain）・`object.rs` ~21。builtin の 29 を「primitive 内 atomic（`make_array_unique` の clone-release 等、op 意味に内包）」「明示 `Release` 化すべきもの（引数を使用後に release 等）」「外部化できず宣言で残す内部 RC」に分類するのが P1 の主要監査。
- **（調査済み）`is_var_used_later` 依存の InlineLLVM**（`builtin.rs` 全10 site を分類。いずれも RC 判断のみで計算結果・挙動は used_later に非依存＝in-place/clone はランタイム refcount で決定）: **(A) 借用読み後 last-use なら引数 release**（`noretain` 読み＋`if !used_later release`）＝1855（配列要素 get）/2418（配列 ptr）/2489（get_size）/2540（get_capacity）/3873（union `is`）/4546・4648（retain 関数 ptr 取得）/4755（data ptr 取得）の 8。**(B) 呼び出しをまたぐ retain/release**（`with_retained`: `f(x)` の前後で x を retain→release し呼び出し中 x を生存）＝4206+4214。RC IR では (A) は容器引数を `Ref`（借用）で宣言し、lowering の last-use 解析が容器の明示 `Release` を last-use に配置（getter が boxed 要素を retain するのは別効果）。(B)（`with_retained`）は **opaque な InlineLLVM のまま retain/release を内部に埋める**。この `Retain` は呼び出し中 x を shared に見せ f の in-place 変更を防ぐ**意味的** RC で、最適化で消えては困る＝外に出すメリットが無くリスク（相殺で消える）だけなので内部に残すのが正しい。used_later スキップは落として**常に retain**（内部 RC は codegen 時に used_later を見ない）。P1 の書き換え: (A) は used_later を `Ref`＋lowering の last-use 解析へ移す、(B) は常に retain へ。どちらも `generate` は `is_var_used_later` を呼ばなくなる（grep 由来なので網羅監査）。
- **（要確認）各 InlineLLVM 引数の `Ownership` と `Ref` 化可否**: read-only op（§8 分類A）は既に `noretain`（借用的）なので `Ref` に素直に対応。`Own` で retain してから読む引数があれば `Ref` 化＋release 外出し＋相殺で速くなる（§6）。`Ref` 化できない op もある。全件確認が要る。**`fix`（不動点コンビネータ）・bulk array 系**が `Ref` 化できない候補（`loop` は InlineLLVM op でなく std の再帰関数なので対象外＝§3.3 の不動点で透過的に解析される）。P1 監査で各引数を「`Ref` 化可／`Own` のまま（内部 RC を宣言で残す）」に分類する。
- **force-unique 内 clone の RC 境界**: `make_array_unique`/`make_struct_unique` の clone（共有時に deep copy ＋要素 retain）は op の atomic 意味に内包し、内部 RC は IR ノードに出さない（最適化対象でない共有パスのため）。引数 `Ownership` のみ宣言。
- **（確認済み）`fix`（ローカル再帰の不動点コンビネータ）は RC IR で表現可能**: std `fix = |f| |x| FixBody`（`FixBody` は InlineLLVM、free vars `x,f,cap`）。lift で outer `|f|`／inner `|x|` の `RcFunc` になり、`fix(f)`=`Closure(inner,[f])`、本体は `LLVM(FixBody, [x,f,cap])`（全 `Own`）。FixBody は自己 funptr（codegen の `get_parent`）＋現 cap 再利用で `fixf=fix(f)` を作り `f(fixf)(x)` を呼ぶ（heap alloc なし、RC cycle 無し＝fixf→f だが f→fixf 無し）。内部 RC は宣言で残す（opaque・`Ref` 化不可）ので fix 内再帰は解析から保守的に見える。`Closure(self)+App` へ desugar も可だが cap 再利用を失う。
