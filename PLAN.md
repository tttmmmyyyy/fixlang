# Fix RC 中間言語（RC IR）と一意性チェック除去（unique-check-elim）

ステータス: 設計のみ・未実装。

RC（参照カウント）最適化の基盤として **RC IR**（評価順を固定した ANF ＋ 明示 retain/release ＋ ローカル名グローバル一意）を導入し、その上で uniqueness 解析・unique-check-elim・将来の RC 最適化（retain/release 相殺・reuse・borrow・順序スケジューリング）を行う。

用語: 値の形＝`UniquenessShape`（変数がいま指す値の形。boxed はその参照カウントを持つ）、参照カウント＝`CTRefCnt`（`Unique`＝単独所有／`Dynamic`＝不明）、関数ごとの要約＝`UniqSignature`（入力 UniquenessShape → 結果 UniquenessShape）。

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
6. **明示的型付け（explicitly typed）**: 各 RC IR ノード・各変数は具体型（特殊化後の monomorphic な型）を明示保持する。codegen の LLVM 型生成、§3 の uniqueness 解析（型からレイアウト＝`UniquenessShape` を導出: boxed/unboxed・struct フィールド・union variant・配列要素・closure ペア）、デバッグ情報がこれを使う。全変換が保存する。

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
    Var(Var),                             // move/rename: y := x（x を消費、y が UniquenessShape を引き継ぐ）。rc 中立で別名を作らない（別名化は手前の Retain が担う）。copy propagation で消せる
    App(Var /*callee: closure か funptr の値*/, Vec<Var>),  // closure 呼び出し・直接 funptr 呼び出し両方（codegen が型で振り分け）
    Closure(FuncRef, Vec<Var> /*捕捉*/),  // top-level 関数 + 捕捉変数列 → unboxed の {funptr, 捕捉obj ptr} ペア。捕捉 obj のみ boxed（rc 追跡）、空捕捉は null＝RC-free
    LLVM(InlineLLVM, Vec<Var>),           // 全 builtin: 算術/getter（射影）/set/mod/構築（struct/タプル/ArrayLit/union variant）/fill/リテラル(IntLit 等)/FFI/...。効果は UniqSignature で宣言（alloc 系は結果 Unique・引数をスロットへ）
}
// atom 位置は Var のみ（global funptr 参照を含む）。リテラルは RC 無関係な unboxed 即値で LLVM の IntLit 等として let 束縛するため Atom 型は持たない
enum RcState {            // retain/release の state ディスパッチ。lowering は既定 Unknown（健全）、§6 の state 推論が特殊化（`RcState` を注釈するだけ＝構造変更なし）
    Unknown,             // runtime で refcnt_state を見て 3-way（= 現状の retain/release）
    Local,               // LOCAL 確定: 非 atomic inc/dec、state チェック省略
    Threaded,            // THREADED 確定: atomic inc/dec、state チェック省略
    Global,              // GLOBAL 確定: codegen で no-op（コードを出さない）。最小化したければ後段 cleanup で削除可
}
enum Ownership { Own, Borrow }  // boxed 末端 1 個の所有権。Own=所有を受け取る（C++ shared_ptr 相当。消費＝内部 release か結果へ move。呼び出し側は非 last-use なら Retain してから呼ぶ）／Borrow=借用のみ（weak_ptr 相当、RC 操作しない。呼び出し側はそのまま呼ぶ）。AST→RC IR lowering はこれだけで正しく明示 RC を挿入できる
enum OwnershipShape {           // 引数 1 個の所有権を UniquenessShape と同型で表す（末端 boxed ごとに Own|Borrow）。§2.1 borrow 化の出力型でもある
    Unboxed,
    UnboxedAgg(Vec<OwnershipShape>),
    Boxed(Ownership),
}
// 引数の後始末（§3.2）は各引数の OwnershipShape の boxed 末端ごとに適用: `Own`+`Unique`->dead／`Own`+`Dynamic`->据え置き／`Borrow`->存続。解析（§3）が関数固有に読むのは結果 UniquenessShape（§3.3）だけ（後始末と別 API）。
// OwnershipShape は RC IR ノードのフィールドでなく `InlineLLVM`（`LLVMGenerator`）から引数位置ごとに取得: `InlineLLVM::arg_ownership(i) -> OwnershipShape`（variant ごとに dispatch。単純型の引数はほぼ `Boxed(Own)`）。source 関数は all-`Own` から §2.1 で `Borrow` へ書き換えて決める。lowering がこれを引いて明示 RC を挿入。
// `Borrow` 化の上書き（§2.1/§6 borrow 最適化）: `set_borrow(i)` が arg i の OwnershipShape 末端を `Borrow` に切替（generate がその引数の内部 release をやめる）。`can_set_borrow(i)` が false の op（`FixBody` 等）もある。
```

- **分岐は `Match` のみ（`If` を持たない）**: Bool を union 化する（std.fix: `type Bool = unbox union { _false : (), _true : () }; true = _true(); false = _false();`）。ソースの `if`/`true`/`false`/比較演算子は不変で、AST→RC IR 生成で `Expr::If(c,t,e)` を `Match(c, [_false => e, _true => t])` に desugar するだけ。性能中立（Bool-union ＝ `{i8 tag, [i8;0]}` ＝ i8。比較演算子は今も i8(0/1) を返す＝tag そのものでビット不変。FFI も i8 tag で不変。match は i8 tag の compare+branch で `if` と同等）。`&&`/`||`/`not` は `if` 経由なら desugar で吸収。
- **射影に専用ノードは持たない**: フィールド/variant payload/配列要素の取り出しは getter プリミティブ＝`LLVM` で表す。解析での扱いは §3。
- **構築も getter 同様 `LLVM`（専用 `Construct` ノードを持たない）**: struct/タプル/`ArrayLit`/union variant の構築は alloc 系 `LLVM` プリミティブで表す（射影＝getter を専用ノード化しない方針の双対）。現コンパイラは struct=`MakeStruct`・array=`ArrayLit`（専用 AST ノード）／union=`union_new`・リテラル=`IntLit` 等（InlineLLVM）と混在するが、lowering で全て alloc 系 `LLVM` に寄せて統一する。解析での扱い（結果 `Unique`・引数をスロットへ move）は §3.2/§3.3。
- **`Atom` は持たず atom 位置は `Var` のみ**: リテラル（整数/浮動小数/Bool/nullptr）は現コンパイラでも `Expr::Lit` でなく InlineLLVM（`IntLit` 等）。RC 無関係な unboxed 即値なので `LLVM` で let 束縛し、atom は `Var`（global funptr 参照を含む）に一本化（文字列リテラルは boxed `Array U8` ＝ alloc 系 `LLVM` 側）。
- **`Let` は単一 Var のみ（Pattern を持たない）**: `let (x,y)=s` 等の struct/タプル destructure は **getter プリミティブ列 ＋ `Release(container)`** に lower（役割分担: 構造分解は getter、union 分岐は `Match`）。lowering は現状の destructure codegen（`get_struct_fields`/`get_union_value`）が行う RC 最小化（move-out）を踏襲し、**最初から必要最小の retain/release だけを挿入する**。
- **RC IR は nested lambda を持たない**: lowering が全 lambda を top-level RC IR 関数へ lift し、使用箇所を `Closure(func, 捕捉)` に変換する（クロージャ生成を明示）。各関数の RC が閉じる。クロージャ値は unboxed の `{funptr, 捕捉オブジェクトへのポインタ}` ペアで、捕捉オブジェクトのみ boxed（rc 追跡。空捕捉は nullptr で複製しても RC 増減なし）。`FuncRef` ＝ top-level RC IR 関数への参照（名前/id。lift した lambda body。codegen で funptr に解決）。`Closure` の捕捉リスト（`Vec<Var>`）は**順序つきでノードに保持**する。順序＝捕捉オブジェクトのスロット順＝lifted 関数が cap から射影する順（`cap.@0, cap.@1, …`）。`Closure` 生成時の格納順と lifted 関数の射影順は同順でなければならず、全変換がこの対応を崩さない（捕捉の並べ替え/追加/削除は両側を揃えて行う）。
- **トップレベル定義は `RcFunc`、クロージャ値生成は `RcRhs::Closure`**: lift した lambda body・global 関数・uncurry funptr 版はすべて `RcFunc`。**クロージャは必ず arity-1**（arrow 型はカリー化される）で closure ABI の関数は `(arg, cap)` の2引数（cap が末尾、body が cap から捕捉を射影）。多引数＋捕捉は入れ子の arity-1 クロージャになる（多引数クロージャは存在しない）。**多引数は funptr のみ**（捕捉なし n 引数。uncurry が global lambda から `name#funptr{n}` を生成、`n ≤ FUNPTR_ARGS_MAX`）。`Closure(FuncRef, captures)` は実行時クロージャ値（unboxed `{funptr, 捕捉obj}`）を生成し、捕捉 obj は heap 値で `RcFunc` の一部ではない。`App` は callee 型で振り分け（funptr＝直接 n 引数、closure＝arg ＋抽出した cap）。
- **RC 効果の宣言（lowering と解析が RC を読めること）**: **AST→RC IR lowering** は各引数の `OwnershipShape`（`Own`／`Borrow`。§1.2）を見て正しく明示 RC を挿入する——`Own` かつ非 last-use なら使用前に `Retain`（last-use なら何もしない）、`Borrow` なら何もしない。codegen は出来上がった明示 `Retain`/`Release` を inc/dec へ翻訳するだけ（`Own` で外部化しない op の内部 release はその op の generate 側）。`mark_global`/`mark_threaded`→`MarkGlobal`/`MarkThreaded`。`make_array_unique` 等の force-unique 内 clone は op の意味に内包する。**外に出すのは「最適化で消せる/動かせる & 消したい RC」だけ**（last-use/ownership の RC。相殺/reuse/borrow が効く）。**最適化で消えては困る意味的 RC は opaque な op の内部に埋めたまま**にする（例: `with_retained` の「呼び出し中 x を shared に見せる」retain。外に出すと相殺で消えて壊れる）＝構築により保護。外に出せない内部 RC は宣言で足りる。状態チェックは状態不明時の runtime `Retain`/`Release` に内包する（`MarkGlobal` 以外に「is-global チェック」専用ノードを持たない。静的に global/local と分かれば `RcState` を `Global`(no-op)/`Local`(チェック省略) に特殊化＝将来の state 最適化、§6）。**全 InlineLLVM の `OwnershipShape` と `Borrow` 化/外部化可否は P1 で全件監査**（`fix`/bulk array は `Borrow` 化できない候補。§8）。

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
- **明示 retain/release 挿入**（last-use 解析＝関数全体の後ろ向きパス）: (a) non-last-use の使用の前に `Retain`（複数回使用ぶんの参照を用意。現 `get_scoped_obj` の used_later→retain 相当）。(b) 変数の **last use が borrow(`Borrow`) なら直後に `Release`**（consume(`Own`)/move が last use ならそこで消費されるので `Release` 無し）。ただし**末尾呼び出しの引数は例外**——後続 `Release` を出すと tail call が壊れるので `Own` に留めて callee に release させる（§2.1）。(c) 未使用 let 束縛・分岐 dead 変数も `Release`。
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
- **cap の release は callee（`g#lifted`）**。boxed 捕捉は「`Retain`(取り出し)＋`Release(cap)`」のペア（§2.2 の相殺が move-out に畳んで両方消せる）。空捕捉なら cap は null で `Release(cap)` は no-op。
- **呼び出し側 `f` に cap の release は無い**: クロージャ `g` を `App` が consume し所有権（cap 含む）が callee へ渡る。`g` を2回呼ぶなら使用前に `Retain(g)`（＝cap obj を retain）が入り各 callee-release と釣り合う。
- funptr（`concat_len#funptr2`）は **cap 引数なし**。例は `@size` の配列引数を `Borrow`（読むだけ）として release を呼び出し側の明示 `Release` に出した形。base の `Own`（`@size` が内部 release）なら明示 `Release(a)/(b)` は出ず op 内部にある（§2.1 の borrow 化が `Own`→`Borrow` 化して外出しする）。
- RC 収支（arr）: cap へ1参照 → `g#lifted` で +1 → `Release(cap)` で −1 → `concat_len` で `Release(a)` −1 = 0（リーク・二重解放なし）。
- 捕捉と呼び出しの両方で同じ値を使う版（`g(arr)`）なら、`arr` は2回使用 → `f` で使用前に `Retain(arr)` が1つ入り、`concat_len` は `a==b`（同一配列・rc 2＝shared）を受け取る。

## 2. RC 前処理（`Retain` を減らして uniqueness の precision を上げる）

uniqueness 解析（§3）は「`Retain` されていない boxed ＝ `Unique`」を読む。`Retain` は `Unique -> Dynamic` の一方向遷移なので、precision は「`Own` を `Borrow` に書き換えて `Retain` を落とす（borrow 化 §2.1）・残った冗長な `Retain`/`Release` を消す（相殺 §2.2）」ことに帰着する。両方を uniqueness 解析の前に行い、初版から入れる。

### 2.1 borrow 化（`Own` 引数の `Borrow` 化）
lowering が作る RC IR は**全引数 `Own`**（現 codegen が全引数 `Own` 前提で RC を書いており、そこから抽出する初版がそうなる）。ここで、可能な引数を `Own` -> `Borrow` に**書き換える**。狙いは呼び出し側の `Retain` を落として `Unique` を保つこと（§3）。`InlineLLVM` の `OwnershipShape` は宣言済み（§1.2 `arg_ownership`）で、書き換え対象は source 関数の引数。

**どこまで `Borrow` にできるか**: 引数（の boxed 末端）`p` が body 内で**読むだけ**（escape/consume されない）なら `Borrow` にできる。**消費** ＝ callee の `Own` 位置へ渡す（`App`/`LLVM` の該当引数、`Closure` 捕捉、構築 alloc 系 `LLVM` へ move-in）／`Ret(p)`（return で escape）。**消費でない** ＝ `Borrow` 位置での read（getter・比較・size）・`Match(p)` の tag 読み・未使用（drop するだけ）で、**「読むだけ・捨てるだけ」は `Borrow` 可**（own-then-release ＝ borrow）。move-bind `Let(y, Var(p))` は透過（`p` の消費 ＝ `y` の消費）。unboxed 集約の子取り出しは子変数を辿って親引数の末端へ帰着。callee が `Own` かは相手の現在の `OwnershipShape` に依存するので、**コールグラフ上の最大不動点**で決める（初期は全 `Borrow`、消費を見つけた末端を `Own` に降格、`Borrow -> Own` の一方向で有限停止。間接呼び出しは全 `Own`＝保守的）。結果は各引数の `OwnershipShape`（§1.2）。

**書き換え**: `p` を `Borrow` にしたら、callee は `p` の内部 `Release` を落とす（もう所有しない）。呼び出し側 `f(x)` は callee が release しなくなったぶんを引き受ける——`x` の last-use がその呼び出しなら直後に `Release(x)`（`x` はそこで drop）、後でも使うなら all-Own 時に入れていた呼び出し前の `Retain(x)` が余り、呼び出しをまたぐ `Retain`/`Release` を §2.2 の相殺が net-zero として消す。結果、`x` は `Unique` のまま後続（`set` 等）へ届く。

**末尾呼び出しを壊さない（制約）**: 上の「呼び出し**後**の `Release`」が**末尾位置の呼び出し**に付くと tail call でなくなる（`let r = App(f, args); Ret(r)` が `let r = App(f, args); Release(x); Ret(r)` になり、末尾再帰なら stack を消費する）。よって**末尾呼び出しに渡す「所有・last-use」の引数は `Borrow` 化せず `Own`**（callee が release）に留め、末尾位置に後続 `Release` を出さない。read-only 引数を末尾再帰で素通しする形（`Borrow` 引数を `Borrow` 位置へ渡す）は release が出ないので影響しない。

**パス順**: lowering で全 `Own` の RC IR（§1.4）→ borrow 化（`OwnershipShape` 確定＋上記 RC 書き換え）→ §2.2 相殺 → uniqueness 解析（§3 が `OwnershipShape` を読む）。borrow 化は uniqueness と独立（消費の構造だけで決まり、引数が unique かに依らない）。

§3 冒頭の例が届く仕組み:
```
x: Unique
f(x)                  // f の引数が Borrow に書き換わっていれば x は消費されず
let y = x.set(i, v)   // x は Unique のまま -> set が Unique を見て check 除去
```
`f` の引数が `Own` のまま（x を escape/格納するので `Borrow` 化不可）なら x は `f(x)` の手前で `Retain` -> `Dynamic` となり set には届かない（正しい: x は実際に共有される）。

### 2.2 retain/release 相殺
`Retain(x)` の後、その追加参照を必要とする使用が無いまま `Release(x)` が来るなら両方除去（peephole / 簡単な dataflow）。名前一意なので追跡が容易。

効果:
- 「used-later で一旦 retain したが直後の op が release（net-zero）」のような冗長 RC が消える。
- → 後段の uniqueness 解析は「**`Retain` されていない boxed 値 ＝ unique**」を素直に読めるようになる。
- clone 削減としても有用。

これは **RC 削減の最適化**であり、同時に uniqueness の precision に効く: `Unique -> Dynamic` を起こすのは `Retain` だけ（§3）なので、冗長な `Retain`/`Release` ペアを消せば `Unique` が保たれ後段の unique-check-elim に届く。precision のもう一つの主役は borrow 化（§2.1。読むだけの引数を `Borrow` に書き換え、余った呼び出し前 `Retain` を相殺対象にする）。どちらも「`Retain` を減らす」方向で precision を作る。健全性とは無関係（消しても付けても健全）で、順序自由に走らせてよい。

## 3. uniqueness 解析（RC IR を抽象解釈）

RC IR を**抽象解釈**し、各変数がいま指す boxed 値が **`Unique`（単独所有＝rc は静的に 1）か `Dynamic`（静的には不明＝実行時に unique/shared を判定）か**を追う。`Dynamic` は force-unique の実行時チェックに委ねる値で、実行時に unique なら in-place になる。alloc（構築・`set` 結果等）は `Unique`。`Retain`（複製＝2つ目の参照を作る）だけが `Unique -> Dynamic` に一方向で倒し、`Dynamic` は吸収状態（`Release` でも呼び出しでも `Dynamic` のまま）。`unique_ptr` と `shared_ptr` の対応そのもので、`unique_ptr` を複製したくなったら `shared_ptr` に変換するしかない、というのと同じ。unboxed 集約（タプル・unboxed struct/union・クロージャ）は子の UniquenessShape を持つ。ループ・再帰は有限領域上の**不動点**で畳む（`Unique | Dynamic` の 2 点束なので停止は自明）。`Dynamic` では unique-check-elim が force-unique を除去せず、**force-unique の実行時 uniqueness チェックが残る**（§4。実行時に unique なら in-place、shared なら clone）。

precision（どれだけ `Unique` を保てるか）は「`Retain` を減らすこと」に帰着する。lowering の all-`Own` では呼び出し前に `Retain` が入るが、読むだけの引数を `Borrow` に書き換えるとその `Retain` が余り、相殺で消えて `Unique` が保たれる。これを §2.1（borrow 化）＋ §2.2（相殺）で行う。

### 3.1 状態
```rust
enum CTRefCnt { Unique, Dynamic } // boxed 値の参照カウント。Unique=単独所有（rc 静的に 1）、Dynamic=静的に不明（実行時に unique/shared を判定＝force-unique チェックを残す）。Unique < Dynamic の 2 点束
enum UniquenessShape {                 // 変数がいま指す値の形
    Unboxed,                 // scalar（rc 無し）
    UnboxedAgg(Vec<UniquenessShape>),  // タプル/unboxed struct/unboxed union/クロージャ（子の UniquenessShape を持つ。クロージャ＝[Unboxed funptr, 捕捉obj の UniquenessShape]）
    Boxed(CTRefCnt),         // boxed 値。自身の rc のみ持ち、中身は追わない
    Bottom,                  // ⊥（到達しない・union の不在 variant）。join 単位元
}
struct State { env: Map<Var, UniquenessShape> }
```
- **boxed の中身は追わない**: `Boxed(CTRefCnt)` は自身の rc だけを持つ。boxed 容器（`Array a`・`Box a`・boxed struct/union）から取り出した boxed 値は `Dynamic`（中身の rc を静的に持たないため）。→ 容器自身の in-place（フィールド `set` 等、容器が `Unique` なら可）は効き、容器の中の値の in-place は効かない（保守的に clone）。
- **unboxed 集約は子を追う**: タプル・unboxed struct・unboxed union（`LoopState` 等）・クロージャは `UnboxedAgg` で子の UniquenessShape を保持。destructure（move で取り出し）は子 UniquenessShape をそのまま引き継ぐ。→ unboxed 容器越しの boxed 値（例: `(cnt, arr)` の `arr`）は uniqueness を追える。

**join**（合流・不動点）: pointwise（`Bottom` 単位元、`UnboxedAgg` は zip、`Boxed` の rc は `Unique ⊔ Dynamic = Dynamic`）。2 点束かつ `Unique -> Dynamic` の単調遷移のみなので不動点は自明に停止する。

### 3.2 interpret 規則
`State`(env) を更新しながら `RcExpr` を順に処理:
- `Let(x, Var(y), k)`: **move**。`env[x] = env[y]`、`y` は消費（dead）。`Unique` も含め UniquenessShape をそのまま引き継ぐ（別名を作らないので **`Dynamic` 化しない**）。
- `Let(x, LLVM(prim, args), k)`: prim の `UniqSignature`（§3.3）で結果 UniquenessShape を組む（getter/set/construct を特別扱いせず一律）。alloc 系（構築・`set`/`mod`/`act` の結果・`fill` 等）→ `Boxed(Unique)`。boxed 容器からの取り出し（getter）→ `Boxed(Dynamic)`。unboxed 集約からの取り出し → 子 UniquenessShape を引き継ぐ。引数の後始末は下の共通規則。
- `Let(x, Closure(_, captures), k)`: `env[x] = UnboxedAgg([Unboxed /*funptr*/, cap])`。捕捉が非空なら cap＝`Boxed(Unique)`（新規捕捉obj に捕捉値を move-in）、空なら null（RC-free）。捕捉された各値は `Own` 引数として後始末（§3.2 共通規則）。
- `Let(x, App(f, args), k)`: `f` の `UniqSignature` で結果 UniquenessShape を組む。引数の後始末は下の共通規則。
- `Retain(x, k)`: `env[x]` の boxed root を `Dynamic` に倒す（複製＝2つ目の参照。`Unique -> Dynamic`、`Dynamic` はそのまま）。**これが唯一の `Unique -> Dynamic` 遷移**。
- `Release(x, k)`: `Dynamic` はそのまま。`Unique` の `Release` は drop（`x` は dead）。
- `Match(x, arms)`: 各 arm を**分岐前 State のコピー**から解析し join。unbox union の payload は move 取り出しで子 UniquenessShape を引き継ぐ（不在 variant は `Bottom`）。boxed union の payload 取り出しは getter＝`Boxed(Dynamic)`。Bool もここ（2 variant）。
- `Ret(x)`: 結果 ＝ `env[x]`（`x` escape）。
- global 参照 → `Boxed(Dynamic)`。

**引数の後始末（共通）**: `App`/`LLVM`/`Closure` の各引数 `a_i` を、callee のその位置の `OwnershipShape`（§1.2。source 関数は §2.1 の borrow 化、`InlineLLVM` は宣言）で処理する:
- `Own` かつ `env[a_i]` が `Unique`: **これは `a_i` の last-use**（後で使うなら手前に `Retain` が入り既に `Dynamic` のはず）。`a_i` は消費されて dead。
- `Own` かつ `Dynamic`: `Dynamic` のまま（後で使われても `Dynamic`）。
- `Borrow`（借用）: `env[a_i]` 不変（`Unique` は `Unique` のまま生き残る）。

つまり呼び出しは **`Unique` の生存**にしか関与せず（`Own`->dead / `Borrow`->存続）、`a_i` の rc を書き換えない。`Unique -> Dynamic` を起こすのは `Retain` だけ。

**`Dynamic` の発生源**（`Unique` を保てない所）: `Retain`（複製）／boxed 容器からの取り出し（getter・boxed union payload）／global／`boxed_from_retained_ptr`／opaque op／join 不一致。`Let(x, Var(y))`（move）と capture-while-live は**発生源でない**（前者は rename、後者は手前の `Retain` が既に倒している）。

**uniqueness クエリ**（unique-check-elim が使う、§4）: `is_unique(x)` ＝ `env[x]` が `Boxed(Unique)`。`Unique` は複製（`Retain`）を経ていない単独所有でしか付かないので真に unique。

### 3.3 関数の効果（`UniqSignature` ＝ 入力 UniquenessShape → 結果 UniquenessShape）
§3.2 で見たとおり、呼び出しが env に及ぼす作用のうち rc に関わる部分は引数の `OwnershipShape`（§1.2/§2.1）だけで決まる（`Own`+`Unique`->dead／`Borrow`->存続）。よって関数固有の効果は**結果 UniquenessShape の組み立て方**だけを持てばよい:
```rust
struct UniqSignature { result: UniquenessShapeRef }  // 入力キー（引数 UniquenessShape の要約）ごとの結果 UniquenessShape
enum UniquenessShapeRef {
    Unboxed,
    FreshBoxed,               // 新規 boxed（Unique）: 構築・set/fill の結果 等
    DynBoxed,                 // 中身不明の boxed（Dynamic）: boxed 容器からの取り出し・global・boxed_from_retained_ptr
    Arg(usize),               // a_i をそのまま（結果が a_i を引き継ぐ。id 等）
    Field(usize, Vec<usize>), // unboxed 集約な a_i の子（move 取り出し）。boxed 容器の子は DynBoxed
    Agg(Vec<UniquenessShapeRef>),       // unboxed 集約（タプル/struct/closure ペア）
}
type InputKey = Vec<ArgKey>;              // メモ化キー: 引数 UniquenessShape を有限に要約（集約は木）
enum ArgKey { Unboxed, UniqueBoxed, DynBoxed, Agg(Vec<ArgKey>) } // 粒度は精度ノブ（P2）
```
呼び出し `let r = f(a0..)`: `env[r] = resolve(sig.result, args)`（`Arg(i)`->`env[a_i]`、`Field`->射影、`Fresh/DynBoxed`->`Boxed(Unique/Dynamic)`、`Agg`->`UnboxedAgg`）。引数の後始末は §3.2 共通規則（`OwnershipShape` で）。
- **ソース関数 = 推論**: 関数ごとに `Map<InputKey, UniqSignature>` を memo（解析側テーブル、`FuncRef` キー。IR の `RcFunc` には載せない）。入力キーごとに body を §3.2 で解析し結果 UniquenessShape を得る。§4.1 の特殊化 worklist と共有。再帰は不動点（初期 `Bottom`）。
- **プリミティブ（`InlineLLVM`）= 宣言**: `LLVMGenerator::result_shape(key) -> UniquenessShapeRef`（variant ごと、入力キー依存可）。`OwnershipShape`（§1.2）は別 API（`arg_ownership(i)`）で宣言。
- global／`boxed_from_retained_ptr`（ptr→boxed で rc 不明）→ 結果 `DynBoxed`。FFI（`CALL_C`）は boxed を返さない（結果 unboxed）ので rc 対象外。assert ビルドで不健全な claim を実行時検出。

例（`OwnershipShape` は §2.1／§1.2 の宣言、ここでは result のみ）:
- **retain getter** `Array::@(i, arr)`: `arr`＝`Borrow`、要素が boxed なら `result=DynBoxed`（容器から取り出す＝別名）、unboxed なら `result=Unboxed`。
- **set** `set(i, v, arr)`: `arr`＝`Own`・`v`＝`Own`（要素へ move）・`result=FreshBoxed`（in-place/clone どちらでも結果は `Unique`）。これでループ `arr=arr.set(..)` が `Unique` を継続。
- **構築** `MakeStruct{a,b}`: boxed struct なら `a`,`b`＝`Own`・`result=FreshBoxed`。unboxed struct/タプルなら `result=Agg([Arg(a),Arg(b)])`。
- **id** `id(x)`: `x`＝`Own`・`result=Arg(0)`（入力が `Unique` なら last-use で dead になり、結果がその `Unique` を引き継ぐ）。

## 4. unique-check-elim

force-unique を含む `LLVM` op（`set`/`mod`/`act` 系）で、対象 boxed 値が §3 の `is_unique`（`Boxed(Unique)`）かつ LOCAL と証明できれば、その RC IR の `LLVM` ノードを **force-unique を行わない版に差し替える**（証明できない＝`Dynamic` では除去せず、force-unique の実行時 uniqueness チェックを残す＝現状動作）。結果は force-unique 後どのみち unique なので、ループ `let arr = arr.set(…)` で 2 回目以降の入力が unique になり「**初回 checked・以降 unchecked**」が自然に出る。

### 4.1 特殊化（uniqueness 駆動、RC IR 上）
`RcFunc` を、流れてくる引数 uniqueness（§3 の入力 `UniquenessShape`）をキーに clone する。呼び出し地点で引数が `Unique` なら unique 用 clone を、`Dynamic` なら別 clone（または original）を呼ぶ。worklist で不動点（§3.3 の per-input-key 解析と共有）。clone は fresh 名を発番し（名前グローバル一意 §1.1-3 を保存）一意な clone 名を付ける。未使用になった `RcFunc`（original/clone）は RC IR 上の dead-function 除去で掃除する。

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
（borrow 化・retain/release 相殺は uniqueness の precision の前処理として §2 で初版から入れる。）
- **reuse**（`Release` した alloc を直後の alloc で再利用＝in-place 再確保）。
- **順序スケジューリング**（意味を保つ範囲で評価順を並べ替え in-place 機会を増やす。例: `f(arr.set(0,42), arr.@0)` を `arr.@0` 先に並べ替えると set が in-place 化し clone が消える）。
- **state 推論**（各値の refcount-state＝LOCAL/THREADED/GLOBAL を静的に決め、RC・状態チェック・`mark_threaded` を省く）。proven-global → `RcState::Global`（codegen no-op）。proven-local → `RcState::Local`（状態チェック省略）。送信値が proven-deeply-unique → `MarkThreaded` 省略。`MarkGlobal` も静的に分かる範囲で最適化。
- **境界チェック除去**（`idx ∈ [0,size)` を証明し完全 unchecked へ。一意性除去と合成でベクトル化 0.20x）。
- **match-of-known-constructor / case-of-case**（LLVM 未実施を確認の上）。

## 7. マイルストーン
- **P0（P1 前）**: **デバッグ情報の E2E テストを追加**してベースライン化。現状その回帰テストが無いため、`fix build -g`（DWARF 付き）でビルドした小プログラムを **gdb 駆動**（`gdb -batch`: `break main.fix:N` → run → backtrace）で検査する統合テストを作る（CLAUDE.md 規約: サンプルを tempdir にコピー、`fix`/`gdb` を `Command` 実行）。assert は file:line の解決・停止・スタックの行情報（マングル名非依存）。補助で bundled `llvm-dwarfdump` の構造 assert も可。**現 main で通すこと**＝P1 の「デバッグ情報一致」(§1.6) の比較対象。ツール: `/usr/bin/gdb` あり、`llvm-dwarfdump` は `/home/maruyama/llvm-17.0.6/bin/`（system には無し）。
- **P0.5（P1 前提）**: **Bool を union 化**（std.fix: `type Bool = unbox union {_false,_true}; true=_true(); false=_false();` ＋ 比較演算子の結果型 ＋ FFI Bool↔i8 tag）。これが `If` を IR から落とす前提（`If`→`Match` desugar は P1 lowering 内）。性能中立（Bool-union＝i8）。de-risk するなら現 `eval_if` を union Bool 対応にして先行検証、または P1 で `eval_if` 撤去と同時。
- **P1**: RC IR 型 ＋ AST→RC IR lowering（`generator.rs` から RC 抽出。名前は lowering が fresh 発番）＋ codegen 付け替え ＋ 全テスト再検証。**最大の山**。完了ゲート: `cargo test --release` 全最適化レベル（`FIX_MAX_OPT_LEVEL` max/basic/none、§1.6）で全通過・全ベンチでリグレッションなし・デバッグ情報一致（§1.6）を満たし、**ユーザに連絡して外部ライブラリテストを依頼してから次フェーズへ**。lowering は現 codegen の RC（move-out/last-use ＝既に最小 RC）を踏襲し、引数は全 `Own`（ベースライン）。source 関数の borrow 化・相殺（§2）は P2 で uniqueness と併せて入れる（前処理）。なお `InlineLLVM` の `OwnershipShape` 宣言（read getter は `Borrow`。§1.2）だけでも hot loop の array getter は借用扱いになり、ベンチの elision の主要部はそれで届く。
- **P2**: 前処理（borrow 化 §2.1・相殺 §2.2 で `OwnershipShape` を確定し `Retain` を削る）＋ uniqueness 解析（`UniquenessShape`＋`UniqSignature` で RC IR を抽象解釈）。read-only ログから始め arrayrw のループ `set` を `Unique`・共有される配列のテストを `Dynamic`（非 unique）と判定することを確認。
- **P3**: unique-check-elim（force-unique 除去 ＋ 特殊化）。arrayrw/fannkuch 計測、全テスト。
- **P4**: reuse / 順序スケジューリング / 境界チェック除去 等。

## 8. リスク・未解決
- **P1 の codegen 付け替えの再検証コスト・範囲**が最大リスク（全プログラムに影響）。段階導入できるか（一部関数だけ RC IR 経由、等）も検討。
- uniqueness の precision は「`Retain` を入れないこと」で作る（borrow 化 §2.1・相殺 §2.2）。`Retain` は `Unique -> Dynamic` の一方向で回復しないので、`Retain` を減らせるかが要。
- ローカル名一意の**全変換での保存**（lowering は fresh 発番で構築的に一意。clone/特殊化は fresh 名発番で freshen）。
- getter（射影）の retain 有無・`UniqSignature` の不動点収束・threaded state・boxed の escape（`boxed_to_retained_ptr`）の RC IR での扱い。捕捉クロージャは `Closure` の captures を `UnboxedAgg` の子 UniquenessShape として追えるが、共有/別名の健全性は検証する。
- 別名健全性は「別名を作る操作で対象を `Dynamic` にする」で担保（§3.2）。`ArgKey` の粒度など精度調整は P2 で詰める。
### 決定事項・要確認
- **（決定）状態は変数ごとの `UniquenessShape`（`State{env: Map<Var,UniquenessShape>}`）、boxed rc は `Unique | Dynamic`**（§3）: boxed 値は自身の rc（`CTRefCnt`）だけを持ち、boxed 容器の中身は追わない（nested uniqueness は追わない）。`Unique -> Dynamic` を起こすのは `Retain`（複製＝2つ目の参照）だけで、`Dynamic` は吸収状態（`unique_ptr`/`shared_ptr` 対応）。boxed 容器からの取り出し・global・opaque・join 不一致も `Dynamic`。move（`Let(x, Var(y))`・consume）は `Unique` を引き継ぐ。unboxed 集約は `UnboxedAgg` で子 UniquenessShape を追う（`LoopState` 越しの配列など線形スレッドの精度）。
- **（決定）`Construct` ノードを設けず構築も `LLVM`**（§1.2）: 集約構築（struct/タプル/`ArrayLit`/union variant）は alloc 系 `LLVM` プリミティブ＋`UniqSignature`（結果 `Unique`・引数をスロットへ move）で表す。InlineLLVM が効果を宣言する設計なので結果 unique は解析に伝わる（専用ノードを持たない。射影＝getter を専用ノード化しない方針の双対）。
- **（決定）boxed rc を `Unique | Dynamic` の 2 点で表す**（`CTRefCnt`、§3.1）: `alloc=Unique`、`Retain`->`Dynamic`（一方向）、`Release`・呼び出しは `Dynamic` を回復させない。これにより (a) **関数効果 ＝ 入力 UniquenessShape → 結果 UniquenessShape の map** で書ける（要約は結果 UniquenessShape のみ。引数の生存は `OwnershipShape` が決める＝`Own`+`Unique` は last-use で dead、`Dynamic` は据え置き、`Borrow` は存続。§3.3）、(b) precision は borrow 化（§2.1）＋相殺（§2.2）で `Retain` を減らして作る、(c) 2 点束かつ単調遷移なので不動点は自明。§3.3 は per-input-key memoize。
- **（決定）`RcRhs::Var(Var)`（move-bind）を持つ**（§1.2/§3.2）: `let y = x` を表せる。意味は move（`x` 消費・`y` が UniquenessShape を引き継ぐ、`Unique` も継ぐ）で rc 中立、それ自体は `Dynamic` トリガーでない。エイリアス（`x`,`y` 両方生存）は「後でも `x` を使う＝non-last use なので手前に `Retain`（`->Dynamic`）」で出る＝copy = `Retain` + move。copy propagation で消せる。
- **（決定）borrow 化を source 関数へ拡張**（§2.1）: lowering の all-`Own` を、読むだけの引数に限り `Own` -> `Borrow` へ**書き換える**。引数ごとの `OwnershipShape`（`UniquenessShape` 同型、末端 boxed に `Ownership`）を、消費の有無からコールグラフ上の最大不動点（初期 `Borrow`、消費で `Own` に降格）で決め、callee の内部 `Release` を落として呼び出し側へ出す（余る `Retain`/`Release` は §2.2 が相殺）。uniqueness 解析の前に走らせる。P2。
- **（決定）Bool→union（P0.5、§7）**: std.fix 定義＋比較演算子の結果型＋FFI（Bool↔i8 tag、`_false`=0/`_true`=1）。`If`→`Match` desugar は P1 lowering 内。要確認: 比較 InlineLLVM の結果構築・`&&`/`||`/`not`・typecheck が union Bool で通るか（ビットは i8 不変）。
- **（決定）global 値の表現**: global 初期化を RC IR（init）として表し `MarkGlobal` を init で発行。参照は atom で解析は `Dynamic`。program = top-level 関数集合 ＋ global init。現状の global 機構（lazy/eager・mark_global 発火点）は P1 実装時に確認。
- **（決定）lowering サブパス順**: AST 正規化（ANF 化 → lambda lift → `If`→`Match` desugar → destructure→getter → fresh 命名）→ 最後に last-use 解析＋明示 retain/release 挿入で RC IR 生成（形と名前が確定してから RC を載せる）。
- **（調査済み）RC site 監査の規模**: codegen の RC は `generator.rs` ~38・`builtin.rs` ~29（InlineLLVM `generate` 内部の release/retain）・`object.rs` ~21。builtin の 29 を「primitive 内 atomic（`make_array_unique` の clone-release 等、op 意味に内包）」「明示 `Release` 化すべきもの（引数を使用後に release 等）」「外部化できず宣言で残す内部 RC」に分類するのが P1 の主要監査。
- **（調査済み）`is_var_used_later` 依存の InlineLLVM**（`builtin.rs` 全10 site を分類。いずれも RC 判断のみで計算結果・挙動は used_later に非依存＝in-place/clone はランタイム refcount で決定）: **(A) 借用読み後 last-use なら引数 release**（`noretain` 読み＋`if !used_later release`）＝1855（配列要素 get）/2418（配列 ptr）/2489（get_size）/2540（get_capacity）/3873（union `is`）/4546・4648（retain 関数 ptr 取得）/4755（data ptr 取得）の 8。**(B) 呼び出しをまたぐ retain/release**（`with_retained`: `f(x)` の前後で x を retain→release し呼び出し中 x を生存）＝4206+4214。RC IR では (A) は容器引数を `Borrow`（借用）で宣言し、lowering の last-use 解析が容器の明示 `Release` を last-use に配置（getter が boxed 要素を retain するのは別効果）。(B)（`with_retained`）は **opaque な InlineLLVM のまま retain/release を内部に埋める**。この `Retain` は呼び出し中 x を shared に見せ f の in-place 変更を防ぐ**意味的** RC で、最適化で消えては困る＝外に出すメリットが無くリスク（相殺で消える）だけなので内部に残すのが正しい。used_later スキップは落として**常に retain**（内部 RC は codegen 時に used_later を見ない）。P1 の書き換え: (A) は used_later を `Borrow`＋lowering の last-use 解析へ移す、(B) は常に retain へ。どちらも `generate` は `is_var_used_later` を呼ばなくなる（grep 由来なので網羅監査）。
- **（要確認）各 InlineLLVM 引数の `OwnershipShape` と `Borrow` 化可否**: read-only op（§8 分類A）は既に `noretain`（借用的）なので `Borrow` に素直に対応。`Own` で retain してから読む引数があれば `Borrow` 化＋release 外出し＋相殺で速くなる（§6）。`Borrow` 化できない op もある。全件確認が要る。**`fix`（不動点コンビネータ）・bulk array 系**が `Borrow` 化できない候補（`loop` は InlineLLVM op でなく std の再帰関数なので対象外＝§3.3 の不動点で透過的に解析される）。P1 監査で各引数を「`Borrow` 化可／`Own` のまま（内部 RC を宣言で残す）」に分類する。
- **force-unique 内 clone の RC 境界**: `make_array_unique`/`make_struct_unique` の clone（共有時に deep copy ＋要素 retain）は op の atomic 意味に内包し、内部 RC は IR ノードに出さない（最適化対象でない共有パスのため）。引数 `OwnershipShape` のみ宣言。
- **（確認済み）`fix`（ローカル再帰の不動点コンビネータ）は RC IR で表現可能**: std `fix = |f| |x| FixBody`（`FixBody` は InlineLLVM、free vars `x,f,cap`）。lift で outer `|f|`／inner `|x|` の `RcFunc` になり、`fix(f)`=`Closure(inner,[f])`、本体は `LLVM(FixBody, [x,f,cap])`（全 `Own`）。FixBody は自己 funptr（codegen の `get_parent`）＋現 cap 再利用で `fixf=fix(f)` を作り `f(fixf)(x)` を呼ぶ（heap alloc なし、RC cycle 無し＝fixf→f だが f→fixf 無し）。内部 RC は宣言で残す（opaque・`Borrow` 化不可）ので fix 内再帰は解析から保守的に見える。`Closure(self)+App` への desugar は避ける（cap 再利用を失うため）。
