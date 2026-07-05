# Fix RC 中間言語（RC IR）と一意性チェック除去（unique-check-elim）

ステータス: 設計のみ・未実装。

RC（参照カウント）最適化の基盤として **RC IR**（評価順を固定した ANF ＋ 明示 retain/release ＋ ローカル名グローバル一意）を導入し、その上で uniqueness 解析・unique-check-elim・将来の RC 最適化（retain/release 相殺・reuse・borrow・順序スケジューリング）を行う。

用語: 変数末端の由来＝`Provenance`（各 boxed 末端が `Fresh`/`Dyn`/`Arg` のどれ由来か。解析が追う値）、それを入力に resolve した uniqueness＝`UniquenessShape`（boxed 末端が `CTRefCnt`＝`Unique`/`Dynamic`）、関数の効果＝結果 `Provenance`（入力非依存。`Map<FuncRef, Provenance>`）。

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
6. **明示的型付け（explicitly typed）**: 各 RC IR ノード・各変数は具体型（特殊化後の monomorphic な型）を明示保持する。codegen の LLVM 型生成、§3 の uniqueness 解析（型からレイアウト＝値の shape 木を導出: boxed/unboxed・struct フィールド・union variant・配列要素・closure ペア。`Provenance`/`UniquenessShape` はこの木の leaf 違い）、デバッグ情報がこれを使う。全変換が保存する。

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
    Var(Var),                             // move/rename: y := x（x を消費、y が由来 Provenance を引き継ぐ）。rc 中立で別名を作らない（別名化は手前の Retain が担う）。copy propagation で消せる
    App(Var /*callee: closure か funptr の値*/, Vec<Var>),  // closure 呼び出し・直接 funptr 呼び出し両方（codegen が型で振り分け）
    Closure(FuncRef, Vec<Var> /*捕捉*/),  // top-level 関数 + 捕捉変数列 → unboxed の {funptr, 捕捉obj ptr} ペア。捕捉 obj のみ boxed（rc 追跡）、空捕捉は null＝RC-free
    LLVM(InlineLLVM, Vec<Var>),           // 全 builtin: 算術/getter（射影）/set/mod/構築（struct/タプル/ArrayLit/union variant）/fill/リテラル(IntLit 等)/FFI/...。効果は Provenance で宣言（alloc 系は結果 Unique・引数をスロットへ）
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
// 引数の生存への作用は各引数の OwnershipShape の boxed 末端ごと（RC 側、§1.4/§2.1）: `Own`+`Unique`->last-use で dead／`Own`+`Dynamic`->据え置き／`Borrow`->存続。uniqueness 解析（§3）が関数固有に読むのは結果の由来 `Provenance`（§3.1/§3.3）だけ。
// OwnershipShape は RC IR ノードのフィールドでなく `InlineLLVM`（`LLVMGenerator`）から引数位置ごとに取得: `InlineLLVM::arg_ownership(i) -> OwnershipShape`（variant ごとに dispatch。単純型の引数はほぼ `Boxed(Own)`）。source 関数は all-`Own` から §2.1 で `Borrow` へ書き換えて決める。lowering がこれを引いて明示 RC を挿入。
// `Borrow` 化の上書き（§2.1/§6 borrow 最適化）: `set_borrow(i)` が arg i の OwnershipShape 末端を `Borrow` に切替（generate がその引数の内部 release をやめる）。`can_set_borrow(i)` が false の op（`FixBody` 等）もある。
```

- **分岐は `Match` のみ（`If` を持たない）**: Bool を union 化する（std.fix: `type Bool = unbox union { _false : (), _true : () }; true = _true(); false = _false();`）。ソースの `if`/`true`/`false`/比較演算子は不変で、AST→RC IR 生成で `Expr::If(c,t,e)` を `Match(c, [_false => e, _true => t])` に desugar するだけ。性能中立（Bool-union ＝ `{i8 tag, [i8;0]}` ＝ i8。比較演算子は今も i8(0/1) を返す＝tag そのものでビット不変。FFI も i8 tag で不変。match は i8 tag の compare+branch で `if` と同等）。`&&`/`||`/`not` は `if` 経由なら desugar で吸収。
- **射影に専用ノードは持たない**: フィールド/variant payload/配列要素の取り出しは getter プリミティブ＝`LLVM` で表す。解析での扱いは §3。
- **構築も getter 同様 `LLVM`（専用 `Construct` ノードを持たない）**: struct/タプル/`ArrayLit`/union variant の構築は alloc 系 `LLVM` プリミティブで表す（射影＝getter を専用ノード化しない方針の双対）。現コンパイラは struct=`MakeStruct`・array=`ArrayLit`（専用 AST ノード）／union=`union_new`・リテラル=`IntLit` 等（InlineLLVM）と混在するが、lowering で全て alloc 系 `LLVM` に寄せて統一する。解析での扱い（引数をスロットへ move。結果 `Provenance` は boxed 集約＝`Fresh`〔alloc〕、unboxed 集約〔タプル・unboxed struct・unboxed union〕＝子の由来を担ぐ〔union は構築した variant のみ〕）は §3.2/§3.3。
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
- **明示 retain/release 挿入**（last-use 解析＝関数全体の後ろ向きパス）: (a) non-last-use の使用の前に `Retain`（複数回使用ぶんの参照を用意。現 `get_scoped_obj` の used_later→retain 相当）。(b) 変数の **last use が borrow(`Borrow`) なら直後に `Release`**（consume(`Own`)/move が last use ならそこで消費されるので `Release` 無し）。ただし**末尾呼び出しの引数は例外**——後続 `Release` が残ると tail call が壊れるので、相殺で消えないなら `Own` に留めて callee に release させる（§2.1）。(c) 未使用 let 束縛・分岐 dead 変数も `Release`。
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

**どこまで `Borrow` にできるか**: 引数（の boxed 末端）`p` が body 内で**読むだけ**（escape/consume されない）なら `Borrow` にできる。**消費** ＝ callee の `Own` 位置へ渡す（`App`/`LLVM` の該当引数。`MakeStruct` 等の構築 alloc も引数 `Own` 宣言なのでここに含まれる。`Closure` 捕捉も move-in で同様）／`Ret(p)`（return で escape）。**消費でない** ＝ `Borrow` 位置での read（getter・比較・size）・`Match(p)` の tag 読み・未使用（drop するだけ）で、**「読むだけ・捨てるだけ」は `Borrow` 可**（own-then-release ＝ borrow）。move-bind `Let(y, Var(p))` は透過（`p` の消費 ＝ `y` の消費）。unboxed 集約の子取り出しは子変数を辿って親引数の末端へ帰着。callee が `Own` かは相手の現在の `OwnershipShape` に依存するので、**コールグラフ上の最大不動点**で決める（初期は全 `Borrow`、消費を見つけた末端を `Own` に降格、`Borrow -> Own` の一方向で有限停止。間接呼び出しは全 `Own`＝保守的）。解く順序は**コールグラフを SCC 分解し condensation を bottom-up（callee 先）に処理、SCC 内は不動点**（再帰の自己参照はこの不動点が捌く）。結果は各引数の `OwnershipShape`（§1.2）。

**書き換え**: `p` を `Borrow` にしたら、callee は `p` の内部 `Release` を落とす（もう所有しない）。呼び出し側 `f(x)` は callee が release しなくなったぶんを引き受ける——`x` の last-use がその呼び出しなら直後に `Release(x)`（`x` はそこで drop）、後でも使うなら all-Own 時に入れていた呼び出し前の `Retain(x)` が余り、呼び出しをまたぐ `Retain`/`Release` を §2.2 の相殺が net-zero として消す。結果、`x` は `Unique` のまま後続（`set` 等）へ届く。

**末尾呼び出しを壊さない（制約）**: 「呼び出し**後**の `Release`」が末尾位置の呼び出しに**残る**（相殺で消えない）と tail call でなくなる（`let r = App(f, args); Ret(r)` が `…; Release(x); Ret(r)`）。無限に深くなるのは**閉路の末尾呼び出し**だけ——非末尾化で伸びるスタックは、閉路（intra-SCC の tail back-edge）なら再帰深さぶん（オーバーフロー）だが、cross-SCC なら condensation（DAG）の深さ＝静的定数ぶん（O(1)、無害）。よって**守るのは閉路の末尾呼び出しだけ**で、cross-SCC の末尾は非末尾化してよい（O(1) と引き換えに `Borrow` の利得を取る）。閉路の末尾呼び出しの扱い:
- **(A) read-only 値を SCC 内で param から param へ受け渡すだけ（素通し）**: 不動点がその値の通る全 param を `Borrow` にするので、閉路末尾は borrow -> borrow で release が出ず、tail は**自動で保たれる**（所有者は SCC 外の呼び出し元）。普通の read-only 再帰はこれ。
- **(B) その関数が所有する fresh 値を borrow 引数として閉路末尾に渡す**: fresh は借用できず後続 `Release` が残るので、その引数を `Own` に留める（callee が consume＝tail 保持。fresh を受ける param は uniqueness を運ばないので `Borrow` 化の利得は元々無い）。(B) で `Own` 固定した param は外部呼び出し元にも `Own` になり、そこが再使用すると `Dynamic` になる（両取りは特殊化。P2）。

**パス順**: lowering で全 `Own` の RC IR（§1.4）→ borrow 化（`OwnershipShape` 確定＋上記 RC 書き換え）→ §2.2 相殺 → uniqueness 解析（§3 が `OwnershipShape` を読む）。borrow 化は uniqueness と独立（消費の構造だけで決まり、引数が unique かに依らない）。

**具体例（read-only 再帰）**: read-only な再帰 `sum` を呼んでから `set` する `main`。
```
sum : Array I64 -> I64 -> I64 -> I64 = |arr, i, acc|
    if i == arr.@size { acc } else { sum(arr, i+1, acc + arr.@(i)) };  // arr は読むだけ
main = ( let arr = fill(100, 0); let s = sum(arr, 0, 0); arr.set(0, s) );  // arr を再使用してから set
```
（各引数の所有権は本来 `OwnershipShape`。この例の `arr` は単一 boxed なので末端1個＝`Boxed(Own/Borrow)` に潰れ、以下その末端を `Own`/`Borrow` と略記する。引数が unboxed タプル/struct なら `OwnershipShape` は `Agg([…])` で子ごとに分かれる——例: `(cnt, arr)` で `cnt` 消費・`arr` 借用なら `Agg([Unboxed, Boxed(Borrow)])`。）

baseline（source 引数は全 `Own`。`@`/`@size` は宣言 `Borrow`）:
```
fn sum(arr /*Own*/, i, acc):
    let n = LLVM[@size](arr); let c = LLVM[eq_i64](i, n)
    Match c {
      _true  => Release(arr); Ret(acc)                  // arr は Own でこの枝 dead -> release
      _false => let e = LLVM[@](i, arr); let a2 = LLVM[add](acc, e); let i2 = LLVM[add](i, 1)
                let r = App(sum, [arr, i2, a2]); Ret(r) // arr -> Own 引数 = 消費（tail）
    }
fn main:
    let arr = LLVM[fill](100, 0)          // Unique
    Retain(arr)                           // sum(非last,Own)+set(last,Own) の2消費ぶん -> arr が Dynamic
    let s   = App(sum, [arr, 0, 0])
    let a2  = LLVM[set](0, s, arr)         // Retain のせいで Dynamic -> force-unique チェックが残る
```
不動点で `sum.arr` を決める（閉路 {`sum`}）:
- 初期化: `sum.arr = Borrow`。
- `sum` body を走査、`arr` は消費されるか?
  - `@size(arr)`・`@(i, arr)` は `Borrow`（宣言）-> 消費でない。
  - `App(sum, [arr, …])` の位置0は `sum.arr` の現在値 `Borrow` -> 消費でない（再帰の自己参照はここ）。
  - `_true` の `Release(arr)` は drop（own-then-release ＝ borrow と両立）-> 消費でない。
- どこでも消費されない -> `sum.arr = Borrow` で収束（(A) の素通し受け渡し）。

書き換え（`sum.arr` を `Borrow` に）:
- (a) `sum` 内部の `Release(arr)`（`_true` 枝）を落とす（もう所有しない）。
- (b) 呼び出し地点で release を引き受ける: `main` は `arr` を所有するので `sum(arr,…)` の直後に `Release(arr)`。`sum` の再帰呼び出しは `arr` を借用（所有しない）ので**足さない**（借用を借用へ渡すだけ＝末尾保持）。

書き換え直後（相殺前）:
```
fn sum(arr /*Borrow*/, i, acc):
    let n = LLVM[@size](arr); let c = LLVM[eq_i64](i, n)
    Match c {
      _true  => Ret(acc)                                // (a) で Release(arr) 消滅（借用のみ）
      _false => let e = LLVM[@](i, arr); …
                let r = App(sum, [arr, i2, a2]); Ret(r) // borrow->borrow: release 無し = tail 保持
    }
fn main:
    let arr = LLVM[fill](100, 0)          // rc 1（Unique）
    Retain(arr)                           // rc 2 : baseline から残った Retain
    let s   = App(sum, [arr, 0, 0])        // 借用（rc 2 のまま、消費しない）
    Release(arr)                          // rc 1 : (b) で足した Release
    let a2  = LLVM[set](0, s, arr)         // Own 消費（rc 1->0）
```
`main` の `Retain(arr) … Release(arr)` は借用呼び出し `sum(arr,0,0)` をまたぐだけ（間に consume が無い）の net-zero なので §2.2 が両方消す -> `let s = App(sum, [arr,0,0]); let a2 = LLVM[set](0, s, arr)`。結果 `arr` は `fill`(Unique) -> `sum`(借用・rc 不変) -> `set` と `Unique` で届き **elision 成立**。`sum` の再帰は borrow->borrow で release が出ず tail のまま。

**具体例（case B: fresh を閉路 tail へ）**: `buf` を読むだけ（consume しない）だが、再帰は毎回 fresh な `buf` を tail に渡す `loop_fresh`。
```
loop_fresh : I64 -> Array I64 -> I64 = |n, buf|
    if n == 0 { buf.@size }                          // buf は読むだけ
    else { loop_fresh(n - 1, Array::fill(n, 0)) };   // 毎回 fresh を tail で渡す
main = loop_fresh(3, Array::fill(1, 0));
```
baseline（全 `Own`）:
```
fn loop_fresh(n, buf /*Own*/):
    let c = LLVM[eq_i64](n, 0)
    Match c {
      _true  => let sz = LLVM[@size](buf); Release(buf); Ret(sz)      // 読んで drop
      _false => let n1 = LLVM[sub](n, 1); let fr = LLVM[fill](n, 0)   // fr : Unique
                Release(buf)                                          // buf 未使用 -> drop
                let r = App(loop_fresh, [n1, fr]); Ret(r)             // fr -> Own = 消費（tail）
    }
```
不動点（閉路 {`loop_fresh`}、`own[loop_fresh.1@[]]`＝`buf` 末端、初期 `Borrow`）:
- `consume_sites`: `@size` は `Borrow`・`Release(buf)` は drop・再帰位置1 の `fr` は `own[loop_fresh.1@[]]=Borrow` ゆえ非消費 ⇒ ∅（phase(i) 降格なし）。
- phase(ii) tail `App(loop_fresh,[n1,fr]);Ret(r)`: `owns(loop_fresh, fr@[])`＝`root` が fill producer ⇒ **True**、`last_use` ⇒ True ⇒ **`own[loop_fresh.1@[]]=Own`**（case B 降格）。

**`Own` が要る理由（`Borrow` だと tail が壊れる）**: 仮に `buf` を `Borrow` にすると phase 2 は内部 `Release(buf)` を消し、`fr`（fresh・この frame 所有）を借用位置へ渡すので呼び出し**後**に `Release(fr)` が要る ⇒ `…; App(loop_fresh,[n1,fr]); Release(fr); Ret(r)` ＝非 tail ＝閉路で深さ分スタック → overflow。case B が `buf` を `Own` に留めるので `fr` は callee が consume ⇒ 後続 `Release` 無し ⇒ **tail 保持**（結果は baseline のまま・RC 不変）。fresh を受ける `buf` は uniqueness を運ばないので `Borrow` 化の利得は元々無い（散文(B)）。外部の `main` でも `loop_fresh.buf` は `Own` なので `a0` を消費する（main が `a0` を再使用するなら `Retain` -> `Dynamic`。両取りは特殊化 P2）。

**手順（擬似コード）**: 自己/相互/非再帰で一様（SCC は不動点のスケジュールと「閉路 tail」の定義に使うだけ）。所有権は **boxed 末端単位**で、`own` は末端をキーに `Own|Borrow` を引く（`own[g.q@π]`＝g の param q の末端 π）。値 `x` を位置 q へ渡すとき各 boxed 末端 `x@π` は callee param q の末端 π に対応する。値の末端は、別名辺（move-bind と unboxed 集約/union の子取り出し＝getter LLVM・Match payload）を後ろ向きに辿った**定義位置 `root`**（producer。= object 同一性）で識別する（散文の「親引数の末端へ帰着」）。`owns`（所有判定）・phase(i)（消費降格）・§2.2 相殺は同じ `root` を共有する。g が未知＝間接呼び出しの位置は `Own` 固定（散文の「間接呼び出しは全 Own」）。
```
borrow_ify(prog):
  # 1. 借用可能性: 全 source param の全 boxed 末端を Borrow と仮定 -> 単調降格で不動点
  own = { source 関数の全 param の全 boxed 末端: 初期値 Borrow }   # 楽観初期化
  for scc in bottom_up(condensation(call_graph(prog))):          # callee 先
    repeat 変化が無くなるまで:                                    # SCC 内不動点（自己/相互再帰）
      for f in scc:
        for c@π' in consume_sites(f):                            # (i) 消費された末端を source param へ帰属して降格
          if root(f, c@π') が (param p, π0): own[p@π0] = Own
        for f の各「閉路 tail 呼び出し」App(g, args); Ret(r):      # intra-SCC の tail 辺だけ
          for (q, x) in enumerate(args), x の各 boxed 末端 x@π:    # q=位置
            # last_use(_,_,q)=位置 q の x が最終使用か。tail 後続は Ret だけだが同一 tail に x は
            # 複数回現れ得る（g(x, x)）ので恒真でない。最終でない出現を Own 降格すると後続位置が
            # 使う前に consume＝use-after-free。ゆえ最終使用の位置だけ降格する。
            if owns(f, x@π) and last_use(f, x, q): own[g.q@π] = Own   # (ii) case B -> 降格

  # 2. RC 書き換え（own 確定後、all-Own の RC を patch）
  for f in prog:
    remove: own==Borrow な param 末端の内部 Release                # callee は所有しない
    for f.body 中の呼び出し App/Closure(g, args):
      for (q, x) in enumerate(args), x の各 boxed 末端 x@π:        # q=位置
        if own[g.q@π]==Borrow and owns(f, x@π):
          呼び出し直後に x@π の Release を挿入                      # 借用末端なら何もしない

  # 3. §2.2 相殺が「Retain … (借用呼び出し) … Release」の net-zero を消す

root(f, x@π):    # x@π の別名鎖を辿った定義位置（producer）= object 同一性。owns/phase(i)/相殺(§2.2) が共有
                 # 別名辺 = move-bind / LLVM 射影(LeafSource が単一 Arg) / Match payload(unboxed union)。producer で停止
  x が f の param                      -> (param x, π)             # producer: param 末端
  Let(x, Var(y))                      -> root(f, y@π)              # 別名辺: move-bind
  Let(x, LLVM(op, args)):             # op の結果 Provenance(§3.3) の末端 π の LeafSource s
     s == {Arg(j, p)}（単一）          -> root(f, args[j]@p)         # 別名辺: 射影/unboxed 構築（例: タプル/struct 射影）
     それ以外（Fresh/Dyn/複数 join）    -> (この Let, π)              # producer: 新規 alloc / boxed 容器 getter・global / 分岐 merge
  Match payload of s（s が unboxed union の variant k）-> root(f, s@(k::π))       # 別名辺: payload 取り出し
  Match payload of s（s が boxed union）      -> (この payload 束縛, π)             # producer: boxed union getter
  Let(x, App 結果 | Closure)           -> (この Let, π)                            # producer: 呼び出し結果/クロージャ

owns(f, x@π):      # f が末端 x@π を所有するか（借用でなく）。root の分類（param なら own、他 producer なら f 所有）
  root(f, x@π) が (param p, π0) -> own[p@π0]==Own    # root が param -> その末端の own
  それ以外の producer            -> True             # alloc/getter/call/closure = f 所有

consume_sites(f):  # 所有権が f から出て行く末端の集合（別名辺で結果へ抜けない Own 位置 = sink）
  App(g, [..x@位置 i..])   -> {x@π | own[g.i@π]==Own}                            # 呼び出し境界（未知 g は Own）
  LLVM(op, [..x@位置 i..]) -> {x@π | x@π が結果へ写らず arg_ownership(i)@π==Own}   # boxed 構築等（射影/unboxed 構築は別名辺で除外）
  Closure(_, [..x..])      -> x の全 boxed 末端                                   # capture = move-in
  Ret(x)                   -> x の全 boxed 末端                                   # return で escape
```

### 2.2 retain/release 相殺
borrow 化（§2.1）が余らせた「呼び出しをまたぐ `Retain`/`Release`」を net-zero として消し、`Unique` を後段（§3）へ届ける（`Unique -> Dynamic` を起こすのは `Retain` だけ）。clone 削減にも効く。健全性とは無関係（消しても付けても健全）・順序自由で、borrow 化と並ぶ precision の 2 本柱。

**正規化＝末端ごとに分解**: 相殺の前に `Retain(x)`/`Release(x)` を **boxed 末端ごと**に分解する（`Retain(x@π)`：unboxed 集約なら各 boxed 末端を個別に retain/release、単一 boxed なら 1 個）。codegen が出す機械語は whole-value 辿りと同じ。これで相殺は**末端単位で一様**になり、「whole を部分的に消して残り末端へ縮約」する手間が消える（`Retain(x)`＋`Release(x.f0)` は分解後 `Retain(x@f0)` と `Release(x@f0)` の対消滅に落ち、`Retain(x@f1)` が残るだけ）。（`Retain`/`Release` が leaf path を持つ＝§1.2 の小拡張。または各末端を getter で名付けてから retain。）

**照合＝object identity（`root`）**: `Retain(x@π)` と `Release(y@π')` は `root`（§2.1）が**同一 object（定義位置）**を返すとき対消滅候補。`root` が別名辺（move-bind・射影・Match payload）を辿って正準化するので、move-bind rename（`Release(arr2)`, `root(arr2)=arr@π`）も部分 field release（`Release(g)`, `root(g)=x@f0`）も同じ照合に落ちる。copy-prop で move-bind を先に畳めば照合は同名で済む。

**消せる条件と、その理由**: その object 末端の**消費使用**（`consume_sites`：Own 位置 / `Ret` / `Closure` 捕捉。§2.1）が `Retain` から対応 `Release` までに無いこと。借用（getter・比較・`Match` tag・`Borrow` 位置）は追加参照を要らない。**なぜ単純な +1/-1 除去でないか**: 間に消費使用が挟まると、`Retain` の +1 はその消費が奪い、`Release` の -1 は別の参照を落とす——往復でないので消すと use-after-free。
```
# x は rc=1 で入る
Retain(x)      # rc 1->2
App(g, [x])    # g Own = retain 参照を consume。rc 2->1
App(h, [x])    # h Borrow = 読むだけ。rc=1（生存）
Release(x)     # rc 1->0
```
間に消費（`g`）があるので消さない（安易に両方消すと `g` が唯一参照を consume -> `h` が解放済みを読む）。

**分岐**: `Retain` は**全経路で**「消費を挟まず `Release` に至る」ときだけ消せる（ある枝で消費されるならその retain は必要）。all-paths（must）判定。

**例（§2.1 の main、borrow 化直後）**:
```
Retain(arr); let s = App(sum, [arr,0,0]); Release(arr); let a2 = LLVM[set](0, s, arr)
```
`Retain(arr)`〜`Release(arr)` 間の使用は `sum(arr)` のみ。borrow 化で `sum.arr` は `Borrow`＝借用（消費でない）。よって対消滅し、`arr` は `fill`(Unique) のまま `set` に届く（elision 成立）。

**手順（擬似コード）**（前向き dataflow・末端単位・`root` 照合）:
```
cancel(f):
  # 前提: Retain/Release は末端ごとに分解済み（Retain(x@π)）。キー o = root(f, x@π)（§2.1）= object 同一性。
  # cancel 可能な Retain R（object o）⇔ R から前向きの【全経路】で、o の consume（consume_sites。§2.1）
  # より前に対応 Release(o) に至る（all-paths / must）。★削除は走査中に枝ローカルで即行わず、走査後に
  #  この条件を確認してからコミットする（枝ローカルで即削除すると、別の枝が o を consume する場合に
  #  その枝が under-retain となり use-after-free。例: `Retain(x); Match { A => Release(x); B => App(g,[x]/*Own*/) }`
  #  で枝A の走査だけで Retain も消すと、枝B〔g が +1 を消費〕が under-retain）。
  各 Retain ノード R: needed[R]=false, pairs[R]={}
  前向き走査（pend[o] = いま生きている未対応 Retain の集合。分岐は枝ごと pend コピー、合流は下記）:
    Retain(x@π):          R=このノード; pend[root(f,x@π)].add(R)
    c@π' ∈ consume_sites: o=root(f,c@π'); pend[o] の各 R を needed[R]=true にし pend から外す
                          （consume が対 Release より先着＝その経路で R は必要。恒久確定）
    Release(y@π):         o=root(f,y@π); pend[o] 非空なら R を1つ取り pairs[R].add(この Release), pend から外す
                          （この経路で R と対消滅。空なら本物の Release＝据え置き）
    分岐合流:             needed は or（ある枝で needed なら全体で needed）。pend は must（全枝で pending な R のみ継続候補）
  # 走査後の commit: needed[R]=false かつ「R から到達する全 leaf 経路が pairs[R] のいずれかで閉じる」
  #   （＝どの経路も consume より先に対 Release を通る）Retain R を、pairs[R] の Release ごと IR から削除。
  #   分岐をまたぐ厳密な bracket 対応は must-dataflow の実装詳細で、核心は「全経路で cancel されない限り消さない」。
```
`consume_sites`/`root` は §2.1 と共有する（相殺・borrow 化・uniqueness が同じ別名知識で動く）。

## 3. Provenance 解析

RC IR を**抽象解釈**し、各変数末端の**由来（`Provenance`）**を追う。uniqueness（`Unique`/`Dynamic`）はその由来を関数の入力に **resolve** して得る（resolve は解析の出力を消費する側＝unique-check-elim §4）——解析は「由来を追う 1 本」で、関数の効果（結果の由来）も同じ解析から出る。由来の基底は `Fresh`（新規＝resolve で `Unique`）／`Dyn`（不明＝`Dynamic`。boxed 容器 getter・global 等）／`Arg(i,p)`（入力 i の末端を引き継ぐ）。**`Retain`（複製＝2つ目の参照）だけが `Fresh -> Dyn`（＝`Unique -> Dynamic`）**に一方向で倒し、`Dyn` は吸収状態。`unique_ptr`/`shared_ptr` の対応そのもの（複製したければ `shared_ptr` に変換するしかない）。ループ・再帰は有限領域上の**不動点**で畳む。`Dynamic` では unique-check-elim が force-unique を除去せず**実行時 uniqueness チェックが残る**（§4。実行時に unique なら in-place、shared なら clone）。

precision（どれだけ `Unique` を保てるか）は「`Retain` を減らすこと」に帰着する。lowering の all-`Own` では呼び出し前に `Retain` が入るが、読むだけの引数を `Borrow` に書き換えるとその `Retain` が余り、相殺で消えて `Unique` が保たれる。これを §2.1（borrow 化）＋ §2.2（相殺）で行う。

### 3.1 状態
```rust
// 解析は各変数末端の由来 Provenance を追う。3 つの Shape は同じ木・leaf 型だけ違う（木構造は値の型が与える）:
//   Provenance = Boxed(LeafSource)／OwnershipShape(§1.2) = Boxed(Ownership)／resolve 出力 UniquenessShape = Boxed(CTRefCnt)
enum Provenance { Unboxed, UnboxedAgg(Vec<Provenance>), Boxed(LeafSource) }
type LeafSource = Set<BaseSource>;   // boxed 末端の由来（join）。通常 singleton、分岐 merge で複数、空 Set = ⊥
enum BaseSource {
    Fresh,                    // 新規（resolve で Unique）: 構築・set/fill 等
    Dyn,                      // 不明（resolve で Dynamic）: boxed 容器 getter・global・boxed_from_retained_ptr・Retain 後
    Arg(usize, Vec<usize>),   // 入力 i の末端 path を引き継ぐ（id・射影〔struct/tuple/unboxed union variant〕。boxed union は Dyn）
}
enum CTRefCnt { Unique, Dynamic }    // resolve（§4）の結果（boxed 末端ごと）。Unique < Dynamic の 2 点束
struct State { env: Map<Var, Provenance> }
```
- **boxed の中身は追わない**: boxed 容器（`Array a`・`Box a`・boxed struct/union）から取り出した boxed 値は `Dyn`（中身の rc を静的に持たない）。→ 容器自身の in-place（フィールド `set` 等、容器が `Unique` なら可）は効き、容器の中の値の in-place は効かない（保守的に clone）。
- **unboxed 集約は子を追う**: タプル・unboxed struct・unboxed union（`LoopState` 等）・クロージャは `UnboxedAgg`。→ unboxed 容器越しの boxed 値（例: `(cnt, arr)` の `arr`）の由来を追える。

**join**（合流・不動点）: pointwise（`Unboxed`/`UnboxedAgg` は zip、`Boxed` の `LeafSource` は集合 union、空 Set が単位元 ⊥）。有限領域・単調（`Fresh -> Dyn`、集合は増える一方）なので不動点は停止。

### 3.2 interpret（由来を追う）
`State`(env) を更新しながら `RcExpr` を順に処理し、各変数の `Provenance` を求める:
- **param**: 初期 `env[param]` の各 boxed 末端 = `Arg(i, π)`（入力そのもの、記号のまま）。
- `Let(x, Var(y), k)`: `env[x] = env[y]`（move。別名を作らないので `Dyn` 化しない）。
- `Let(x, LLVM(prim, args), k)`: prim の宣言 `Provenance`（§3.3）を実引数の由来で合成（`Arg(j,p)` を `env[a_j]@p` に置換）。alloc→`Fresh`、boxed 容器 getter→`Dyn`、unboxed 集約の子取り出し→親の子末端、はこの宣言に含まれる。
- `Let(x, Closure(_, caps), k)`: `env[x] = UnboxedAgg([Unboxed /*funptr*/, cap])`。捕捉非空なら cap の boxed 末端＝`Fresh`（新規捕捉obj に move-in）、空なら null（RC-free）。
- `Let(x, App(f, args), k)`: callee が既知なら `f` の `Provenance` を実引数の由来で合成（`Arg` 置換）。callee 不明の間接呼び出し（closure パラメータ等で decapturing が特殊化できなかった残り）は結果の boxed 末端を保守的に `Dyn`。
- `Retain(x, k)`: `env[x]` の boxed 末端を `Dyn` に倒す（**唯一の `Fresh -> Dyn`**。既 `Dyn` はそのまま）。
- `Release(x, k)`: 由来は不変（`x` は dead）。
- `Match(x, arms)`: 各 arm を**分岐前 env のコピー**から解析し join（末端 `LeafSource` を union）。unboxed union payload → move 取り出しで scrutinee の子末端（`Arg` 系。不在 variant は空 Set）、boxed union payload → getter＝`Dyn`。Bool もここ（2 variant）。
- `Ret(x)`: 関数の結果 `Provenance` ＝ `env[x]`。
- global 参照 → 型どおり（boxed 末端は `Dyn`：GLOBAL 状態で unique にならない。unboxed 部は `Unboxed`/`UnboxedAgg`）。

呼び出し・`Release` は引数の由来を変えない（`Fresh -> Dyn` は `Retain` のみ）。引数の**生存**（`Own`->last-use で dead／`Borrow`->存続）は RC 側（§1.4/§2.1 の `OwnershipShape`）が決める話で、由来解析はそれに関与しない。

**関数の効果 ＝ 結果 `Provenance`**（`Ret` の由来）: param を記号 `Arg` のまま残すので**入力非依存**（関数ごとに 1 つ。再帰は不動点、初期 ⊥＝空 Set）。呼び出し `g(a…)` の結果は g の `Provenance` の `Arg(j,p)` を実引数 `a_j` の由来で埋めて（合成）得る。複製は `Retain -> Dyn` が捌く（例 `(y,y)` -> `(Dyn,Dyn)`、§5 テスト）。

### 3.3 プリミティブ宣言（`result_prov`）
`InlineLLVM` プリミティブが結果の `Provenance` を宣言する（§3.2 の interpret が引く transfer function）。
- **プリミティブ（`InlineLLVM`）= 宣言**: `LLVMGenerator::result_prov() -> Provenance`（引数の型に依存し得る）。`OwnershipShape`（§1.2）は別 API（`arg_ownership(i)`）で宣言。
- global（値の型どおりの Provenance。boxed 末端は `Dyn`、unboxed 部は型どおり）／`boxed_from_retained_ptr`（ptr→boxed → `Dyn`）。FFI（`CALL_C`）は boxed を返さない（結果 unboxed）ので rc 対象外。assert ビルドで不健全な claim を実行時検出。

例（`OwnershipShape` は §2.1／§1.2 の宣言、ここでは result `Provenance` のみ）:
- **retain getter** `Array::@(i, arr)`: `arr`＝`Borrow`、要素が boxed なら `result=Boxed({Dyn})`（容器から取り出す＝別名）、unboxed なら `Unboxed`。
- **set** `set(i, v, arr)`: `arr`＝`Own`・`v`＝`Own`（要素へ move）・`result=Boxed({Fresh})`。ループ `arr=arr.set(..)` が `Unique` を継続。`set` は shared なら clone・unique なら in-place だが**どちらも結果は単独所有の配列**なので、返る物理 object は入力 rc 次第でも uniqueness の由来は一定＝結果は入力非依存に `Fresh`（この clone-on-shared が `Provenance` を入力非依存にしている本体）。
- **構築** `MakeStruct{a,b}`: boxed struct なら `a`,`b`＝`Own`・`result=Boxed({Fresh})`。unboxed struct/タプルなら `result=UnboxedAgg([Boxed({Arg(0,[])}), Boxed({Arg(1,[])})])`。
- **union variant 構築** `continue(x)`（unboxed union `LoopState` 等）: `result=UnboxedAgg`（variant ごとの payload shape）で、構築した variant のスロット＝`x` の由来（`Boxed({Arg(0,[])})` 等）・他 variant＝⊥（空 Set 末端）。§3.2 Match 読み出しの双対。boxed union なら `result=Boxed({Fresh})`（alloc。読み出しは `Dyn`）。
- **id** `id(x)`: `x`＝`Own`・`result=Boxed({Arg(0,[])})`（結果は入力 0 を引き継ぐ）。

### 3.4 例: 配列ループ更新の Provenance（`arrayrw`）
```
main = ( let arr = Array::fill(1000, 0);
         let arr = loop((0, arr), |(i, arr)|
             if i == 1000 { break $ arr }
             else { continue $ (i+1, arr.set(i, arr.@(i) + 1)) });
         ... );
```
`loop : s -> (s -> LoopState s r) -> r`（std）。状態 `s = (I64, Array I64)`（unboxed タプル、arr 末端 path `[1]`）。

**前提（既存の decapturing）**: RC IR lowering の前に **decapturing の closure specialization**（`src/optimization/decapturing.rs`。`inline` 後・`uncurry` 前）が、body closure を引数に取る `loop` を「その body を焼き込んだ特殊版 `loop#lam`」へ書き換え、内部の `body(s0)` を lift 済み body への**直接呼び出し**にする（`loop` は再帰・自己呼び出しで body を同 index に渡す＝specializable。std `fold` と同型）。§3/§4 はこの直接化後の形（`loop#lam`）に働く——generic な `loop` 単体では `body(s0)` が間接呼び出しで結果 Dyn になり精度が出ない。以下その `loop#lam` の body を単に body と書く。

**body の Provenance**（param `s` ＝ `UnboxedAgg([Unboxed, Boxed({Arg(0,[1])})])`）:
- `arr = s.@1`: `Boxed({Arg(0,[1])})`（入力 arr を引き継ぐ）。`arr.@(i)` は借用 read で結果 unboxed（I64）＝arr の由来を変えない。
- break 枝 `break(arr)`: 結果 `LoopState` の break payload（Array）＝ `{Arg(0,[1])}`。
- continue 枝 `arr2 = arr.set(i, …)`: `set` 宣言より `{Fresh}`（§3.3）。continue payload の arr 末端＝ `{Fresh}`。
- 関数結果＝2 枝 join（各枝は 1 variant を構築＝§3.3 の union 構築宣言: 構築 variant のスロットに payload 由来・他 variant は ⊥）: `continue.arr = {Fresh}`、`break.r = {Arg(0,[1])}`。

**loop の結果 Provenance**（`loop = |s0,f| match f(s0){ continue(s1)=>loop(s1,f); break(r)=>r }`、結果 Array 末端を `L`）: `L` は「loop の結果を**入力の関数として**書いた式」で、入力 arr を指す記号が `Arg(0,[1])`。
- **直感（展開）**: loop が返すのは break した時点の配列。0 反復なら即 `break(arr)` で入力そのもの＝`Arg(0,[1])`、1 反復以上なら最後の `set` 結果＝`Fresh`。反復回数は静的に不明なので**起こりうる全部の和**を取る＝`{Fresh, Arg(0,[1])}`。以下これを不動点で（展開せず）求める。
- **方程式**: body 結果（上）から、break 枝＝`{Arg(0,[1])}`（入力を返す）、continue 枝＝`loop(s1)` の結果。ここで `loop(s1)` は「同じ式 `L` の入力を s1 に差し替えたもの」＝`L` 中の `Arg(0,[1])` を `s1.arr={Fresh}` で置換したもの（§3.2 の呼び出し合成）。両枝の join で `L = join( L[Arg(0,[1]):={Fresh}], {Arg(0,[1])} )`（`L` が両辺＝再帰方程式）。
- **不動点**（初期 ∅。各段が「反復回数を 1 つ増やした場合」を足していく）:
  - L₀=∅ → L₁=`{Arg(0,[1])}`（0 反復＝入力を返す）→ L₂=join(`L₁[Arg(0,[1]):=Fresh]`, `{Arg(0,[1])}`)=join(`{Fresh}`,`{Arg(0,[1])}`)=`{Fresh,Arg(0,[1])}`（1 反復＝Fresh 追加）→ L₃=L₂（2 反復も Fresh で既出＝**収束**）。基底集合は有限・置換は `Arg` を消し `Fresh` を増やすだけ（単調）ゆえ必ず停止。
- ∴ `loop` 結果＝ `Boxed({Fresh, Arg(0,[1])})`＝「1 回以上更新すれば Fresh、0 反復なら入力 arr」。

**使い道（resolve）**: `L` は入力非依存なので 1 回求めれば全 call site で使い回せる。呼び出し地点で `Arg(0,[1])` を実引数の由来で埋め、各要素を uniqueness に写して ⊔（`Unique < Dynamic`＝一つでも Dynamic なら Dynamic。§4）:
- **main（`fill`＝Unique）**: `Arg(0,[1]):=Fresh` → `{Fresh, Fresh}={Fresh}` → **Unique**（0 反復でも入力が Fresh、1 反復以上でも set が Fresh。どちらの経路も unique）。∴ 最終 arr は Unique で後続 `set` に届く。
- **共有配列を渡す場合**: `Arg(0,[1]):=Dyn` → `{Fresh, Dyn}` → `Unique ⊔ Dynamic = Dynamic`（0 反復で共有入力をそのまま返す経路があるので unique 保証できない）。

`Fresh` は常に Unique なので、実質**結果の uniqueness は入力 arr が unique かで決まる**。同じ `L` を入力ごとに resolve するだけで両方正しく出る。（実ベンチは 2 重ループだが、内ループ結果 `{Fresh,Arg(0,[1])}` を外ループが同様に畳んで外も `{Fresh,Arg(0,[1])}`、main で `{Fresh}`。）

### 3.5 例: read-only 引数の素通し（`sum`）
§2.1 の `sum`（read-only 再帰）を Provenance 解析にかける。§3.4（結果が `Array`・不動点が非自明）に対し、こちらは**結果が I64＝boxed 末端なし**で、**「読むだけの引数は `Retain` されず `Arg`/`Fresh` のまま素通しする」**という §3 の肝と、§2（borrow化＋相殺）との接続を示す。

pass 順（§2.1: lowering → borrow化 → 相殺 → 解析）より、解析は**borrow化＋相殺後**の RC IR に働く（§2.1 の書き換え結果）:
```
fn sum(arr /*Borrow*/, i, acc):
    let n = LLVM[@size](arr); let c = LLVM[eq_i64](i, n)
    Match c {
      _true  => Ret(acc)
      _false => let e = LLVM[@](i, arr); let a2 = LLVM[add](acc, e); let i2 = LLVM[add](i, 1)
                let r = App(sum, [arr, i2, a2]); Ret(r)
    }
fn main:
    let arr = LLVM[fill](100, 0)
    let s   = App(sum, [arr, 0, 0])
    let a2  = LLVM[set](0, s, arr)
```

**sum の Provenance**（param: `env[arr]=Boxed({Arg(0,[])})`、i・acc は `Unboxed`）:
- `n = @size(arr)`: I64 を返す → `env[n]=Unboxed`。arr は借用 read ゆえ由来不変（`Arg(0,[])` のまま）。
- `c = eq_i64(i,n)` → `Unboxed`。
- Match c: `_true => Ret(acc)`＝`Unboxed`／`_false`: `e=@(i,arr)`（要素 I64）→`Unboxed`、`a2`・`i2`→`Unboxed`、`r=App(sum,…)`＝sum の結果を合成 →`Unboxed`、`Ret(r)`。join＝`Unboxed`。
- **sum の結果 Provenance ＝ `Unboxed`**（I64 返却＝boxed 末端なし。結果に boxed 末端が無いので再帰の不動点も自明に収束）。
- 要点: **arr は sum 内で終始 `Arg(0,[])`**。@size/@ で読む（借用）だけで `Retain` が無い（borrow化が内部 Release を消し、呼び出し側の Retain を相殺が消した）ので `Dyn` に倒れない。

**main の Provenance**:
- `arr = fill(100,0)`: `env[arr]=Boxed({Fresh})`。
- `s = App(sum,[arr,0,0])`: sum の結果 Provenance（`Unboxed`）を合成 → `env[s]=Unboxed`。**呼び出しは引数の由来を変えない**（§3.2、`Fresh -> Dyn` は `Retain` のみ）ので `env[arr]` は `Boxed({Fresh})` のまま。
- `a2 = set(0,s,arr)`: `is_unique(arr@[])`＝`env[arr]={Fresh}` を resolve（main は boxed 入力なし）→ **Unique** → **set 除去（elision 成立）**。

∴ **arr は `fill`(Fresh) → `sum`(借用・由来不変) → `set` と `Unique` で届く**。§2.1 の結論（elision 成立）を Provenance 側から裏付ける。

**§2 が §3 の precision を作る確認（baseline 対比）**: borrow化前の baseline は `fill` 直後に `Retain(arr)` が残る（§2.1 baseline）。すると `Retain(arr)` が §3.2 の唯一の `Fresh -> Dyn` を発火し `env[arr]=Boxed({Dyn})` → `set` で resolve すると `Dynamic` → **除去されない**。つまり Provenance の結果（arr が `Fresh` か `Dyn` か）は `Retain` の有無で決まり、その `Retain` を borrow化＋相殺（§2）が消すことで `Fresh` が保たれ elision が成立する——「precision は `Retain` を減らして作る」（§2 冒頭／§3 の precision 節）が、この 1 例で具体化される。

| | sum の結果 | main の arr @ `set` | elision |
|---|---|---|---|
| §3.2 エミュ（post-§2） | `Unboxed` | `Fresh` = Unique | 成立 |
| （baseline 対比・§2 未適用） | `Unboxed` | `Dyn` = Dynamic | 不成立 |

## 4. unique-check-elim

§3 の解析が出す `Provenance` を、関数の入力 uniqueness に **resolve** して各 boxed 末端が `Unique` か判定し、それを使って force-unique を除去する。

- **`resolve`（由来 → uniqueness）**: `Provenance` を関数の入力 uniqueness に解決し `UniquenessShape`（＝`Boxed(CTRefCnt)` の木、§3.1）を得る。木を辿り各 `Boxed(leafsrc)` を `Boxed(⊔_{s∈leafsrc} rc(s))` に（`rc(Fresh)=Unique`・`rc(Dyn)=Dynamic`・`rc(Arg(i,p))=入力 i の末端 p の uniqueness`）。`Unboxed`/`UnboxedAgg` は素通し。（§3.2 の呼び出し結果 `Provenance` 合成も同形の `Arg` 置換だが、そちらは `Provenance` を返して解析内で閉じる。）
- **`is_unique`**: `is_unique(x@π)` ＝ `env[x]` の末端 π の由来を、その関数の入力 uniqueness に resolve して `Unique` になること。§4.1 の特殊化 clone 内では入力 uniqueness が既知なので確定する（entry `main` は boxed 入力が無く `Fresh`/`Dyn` に底打ち）。`Unique` は複製（`Retain`）を経ていない単独所有でしか付かないので真に unique。
- **LOCAL（v1）**: `Provenance` は uniqueness の由来だけを追い state（LOCAL/THREADED/GLOBAL、§6）は追わない。`Unique` な boxed 値は由来をたどると Fresh alloc（LOCAL）に至る（main は boxed 入力なし・global は `Dyn`）ので、その object の流れに `MarkThreaded` が無ければ LOCAL。in-place は LOCAL ∧ rc==1 のときだけ（§1.3）なので、v1 は「`is_unique` かつ その object の流れに `MarkThreaded` 無し」を LOCAL の十分条件とする（単スレッドのベンチは自明成立）。厳密な state 判定は §6 の state 推論で一般化。

force-unique を含む `LLVM` op（`set`/`mod`/`act` 系）で、対象 boxed 値が `is_unique`（`Boxed(Unique)`）かつ LOCAL と証明できれば、その RC IR の `LLVM` ノードを **force-unique を行わない版に差し替える**（証明できない＝`Dynamic` では除去せず、force-unique の実行時 uniqueness チェックを残す＝現状動作）。結果は force-unique 後どのみち unique なので、ループ `let arr = arr.set(…)` で 2 回目以降の入力が unique になり「**初回 checked・以降 unchecked**」が自然に出る。

### 4.1 特殊化（uniqueness 駆動、RC IR 上）
`RcFunc` を、流れてくる**引数の `UniquenessShape`（§3.1。各引数の resolve 済み uniqueness＝`Boxed(Unique)`/`Boxed(Dynamic)`・`Unboxed`・`UnboxedAgg` の木）をキー**に clone する（`Unique|Dynamic` が有限なのでキーも有限）。**key は force-unique に効く入力末端だけに射影する**（各 force-unique 対象の `is_unique` はその `Provenance` を resolve する＝そこに現れる `Arg(i,p)` 末端の uniqueness だけが判定を変える。他の末端で分けても中身が同一の clone になるので分けない）＝無損失で clone 爆発を抑える（この「force-unique 関連末端」集合は uniqueness 非依存の静的性質で、reachable な force-unique の `is_unique` が参照する入力末端を**呼び出し越しに後ろ向き**に集めて precompute する〔`loop` の関連末端 `arr` は callee `body` の set を辿って判る〕。射影せず入力 `UniquenessShape` 全体を key にしても正しい＝clone が増えるだけ）。呼び出し地点で引数が `Unique` なら unique 用 clone を、`Dynamic` なら別 clone（または original）を呼ぶ。各 clone の uniqueness は §3.2 の入力非依存 `Provenance` を resolve（§4 冒頭）して得る（入力で分けるのはこの特殊化だけ）。worklist で到達 clone を閉包（下記の駆動）。clone は fresh 名を発番し（名前グローバル一意 §1.1-3 を保存）一意な clone 名を付ける。**特殊化は関数を clone するので、未使用になった `RcFunc`（どの call site からも到達しない original/clone）の dead-function 除去を初版で必ず実装する**（さもないと未到達 clone がバイナリに残る＝回帰。到達解析＋未到達 `RcFunc` 削除の 1 パス）。

**駆動（worklist、clone あたり 1 パス）**: エントリ `main`（boxed 入力なし＝key 自明）から出発。clone は `(RcFunc, key)` で一意化し、未生成を queue に積む。1 つ取り出して body を前から走査する:
1. **force-unique op**: 対象の `Provenance` を今の clone の入力 uniqueness で resolve し、`is_unique` かつ LOCAL なら unchecked 版へ差し替える（§4.2）。
2. **`App(g, a…)`**: 各引数の uniqueness を resolve → key（force-unique 関連末端に射影）→ `(g, key)` を未生成なら queue へ積む → callee を clone 名に書き換える。呼び出し結果の uniqueness は g の結果 `Provenance`（§3.2、入力非依存・全 clone 共有）を実引数の由来で resolve するだけで即得る（g の clone body の実装を待たない）。

その他のノード（非 force-unique の `LLVM`・`Let(x, Var)`・`Retain`/`Release`・`Match`）は §3.2 の interpret どおり env（各変数の resolve 済み uniqueness）を進めるだけ（`Match` は各 arm を分岐前 env のコピーで辿って合流）。`Ret` で 1 clone 分の走査が終わり、queue が空になるまで繰り返す。key は有限（末端の `Unique|Dynamic`）なので clone 集合は有限＝停止。結果 uniqueness が precompute 済み `Provenance` から即決まるため各 clone は 1 回処理で済み再訪不要（＝「到達 clone 集合の閉包」で、反復不動点ではない）。

### 4.2 force-unique の除去（RC IR の `LLVM` ノード差し替え）
clone した `RcFunc` の body 中で force-unique を担う `LLVM`(InlineLLVM) ノードを、force-unique しない版（`InlineLLVM` の `force_unique=false`／unchecked generator）に差し替える（新規ノードを作って置換。共有呼び出し地点側の clone は checked のまま）。`force_unique` フラグの有無:

| 操作 | force-unique の所在 | フラグ |
|---|---|---|
| Array `set` | `InlineLLVMArraySetBody`（無条件 `make_array_unique`, builtin.rs:2170） | **無し→追加** |
| Array `swap` | 新 builtin `InlineLLVMArraySwapBody`（force-unique 内蔵。§7 P0.7 で `_unsafe_swap`＋linear-get を置換） | **新設（フラグ付き）** |
| Array `mod`/`act_identity`/`act_tuple2` | §7 P0.7 で PunchedArray punch/plug 化（punch が force-unique）。旧 `_unsafe_get_linear_bounds_unchecked_unretained`（`force_unique`, builtin.rs:1901/1936）は廃止 | punch の `force_unique`→`false` |
| struct `mod_<field>` | `#punch_fu_{field}`（`InlineLLVMStructPunchBody`{true}, `make_struct_unique` @2656） | 既存（非 fu punch あり）→`false` |
| struct `set_<field>` | `InlineLLVMStructSetBody`（無条件 `make_struct_unique`, builtin.rs:3580） | **無し→追加** |
| struct `act_<field>` | 非 fu punch を既に使用（unique 保証） | 対応不要 |

`Const` functor の `act` は force-unique なし（対象外）。generic な `act`（任意 functor）は `unsafe_is_unique` の once-per-call チェックで follow-on。act の functor 特殊化（`optimize_act`。Identity/Const/Tuple2、ホットな `mod`=Identity 含む）は lowering 前に行い、RC IR には上表の具体形（force-unique を持つ op）が現れる——それを §4 が除去する。

### 4.3 例: `arrayrw` の unique-check-elim（§3.4 の続き）
エントリ `main` の初期 arr は `fill`＝Unique。
- main は `loop((0,arr), body)` を状態 arr＝Unique で呼ぶ → key「状態 arr = Unique」で `loop@U` を要求し、呼び出しを `loop@U` に書き換え。
- `loop@U` の走査: `f(s0)` は `body@U`（状態 arr = Unique）を要求。`body@U` 内の `arr.set(i, …)` の対象は `Arg(0,[1])`→resolve→Unique・LOCAL → **`set` を unchecked 版へ差し替え**。continue payload の arr＝Fresh→Unique。
- `loop@U` の再帰 `loop(s1, f)` は `s1.arr`＝Fresh→Unique → 同 key → `loop@U`（自己）。∴ **全反復 unchecked**（fill から一貫して Unique）。ベンチのコメント「set は unique（in-place）path を取る」がこれ。

**共有入力の場合**（arr が shared で loop に入る）:
- main は状態 arr＝Dynamic で `loop@D` を呼ぶ。`loop@D`→`body@D`: 対象 `Arg(0,[1])`→Dynamic → **除去せず force-unique を残す**（初回 checked。shared なら実行時 clone）。set 結果＝Fresh。
- continue payload の arr＝Fresh→Unique。`loop@D` の再帰の key＝状態 arr = Unique → **`loop@U`**（別 clone）。以降 `loop@U` は全 unchecked。
- ∴ 「**初回 checked・以降 unchecked**」（§4 冒頭）が `loop@D -> loop@U` の 2 clone として実現する。

## 5. 適用対象・検証

- マイクロ: `batch/arrayrw{,_unsafe,_fn}`、`fannkuch`、**ソート**（`sort_by`/introsort・heapsort＝`swap` が hot、§7 P0.7）。正しさ: `cargo test --release`（全最適化レベル、§1.6）＋**共有値テスト**（2 箇所に格納して破壊しないこと）。回帰: `benchmark/speedtest`。assert ビルドで不健全検出。
- 一意文脈でチェックが消える（IR/asm に `build_branch_by_is_unique` 由来分岐が残らない or cachegrind 命令数低下）＋共有文脈で消えない（クローンされる）を各セルで確認:

| 対象 | set | swap | mod | act(Id) | act(Tuple2) | act(Const) | 備考 |
|---|:--:|:--:|:--:|:--:|:--:|:--:|---|
| Array | ✓ | ✓ | ✓ | ✓ | ✓ | — | Const は getter。swap はソートで hot（§7 P0.7 の builtin） |
| boxed struct field | ✓ | — | ✓ | ✓ | ✓ | — | `make_struct_unique` を外す。swap は Array 専用 |
| union | — | — | — | — | — | — | `mod_<variant>` は force-unique を踏まない＝対象外 |

入れ子伝播も確認: タプル内配列 `loop((cnt, arr), …)`、struct 内配列・配列内 struct、union 内配列（`LoopState`）。

- **Provenance 解析の直接テスト**: `|x| let y = [x]; (y, y)` の結果が `(Dyn, Dyn)` になること（同一値を複数末端へ置くと複製 `Retain`→`Dynamic` が効き、`(Fresh, Fresh)` にならない。§3.2/§3.3）。併せて `id` の結果由来 `Arg(0,[])`（入力 uniqueness を素通し）・`set` 結果 `Fresh` 等の基本形も確認。

## 6. 将来の RC 最適化（同じ RC IR 上）
（borrow 化・retain/release 相殺は uniqueness の precision の前処理として §2 で初版から入れる。）
- **reuse**（`Release` した alloc を直後の alloc で再利用＝in-place 再確保）。
- **順序スケジューリング**（意味を保つ範囲で評価順を並べ替え in-place 機会を増やす。例: `f(arr.set(0,42), arr.@0)` を `arr.@0` 先に並べ替えると set が in-place 化し clone が消える）。
- **state 推論**（各値の refcount-state＝LOCAL/THREADED/GLOBAL を静的に決め、RC・状態チェック・`mark_threaded` を省く）。proven-global → `RcState::Global`（codegen no-op）。proven-local → `RcState::Local`（状態チェック省略）。送信値が proven-deeply-unique → `MarkThreaded` 省略。`MarkGlobal` も静的に分かる範囲で最適化。
- **境界チェック除去**（`idx ∈ [0,size)` を証明し完全 unchecked へ。一意性除去と合成でベクトル化 0.20x）。
- **match-of-known-constructor / case-of-case**（LLVM 未実施を確認の上）。

## 7. マイルストーン
各フェーズの検証入力（何を入力し何を観測するか）は **§9 フェーズ別テスト計画**に対応。
- **P0（P1 前）**: **デバッグ情報の E2E テストを追加**してベースライン化。現状その回帰テストが無いため、`fix build -g`（DWARF 付き）でビルドした小プログラムを **gdb 駆動**（`gdb -batch`: `break main.fix:N` → run → backtrace）で検査する統合テストを作る（CLAUDE.md 規約: サンプルを tempdir にコピー、`fix`/`gdb` を `Command` 実行）。assert は file:line の解決・停止・スタックの行情報（マングル名非依存）。補助で bundled `llvm-dwarfdump` の構造 assert も可。**現 main で通すこと**＝P1 の「デバッグ情報一致」(§1.6) の比較対象。ツール: `/usr/bin/gdb` あり、`llvm-dwarfdump` は `/home/maruyama/llvm-17.0.6/bin/`（system には無し）。
- **P0.5（P1 前提）**: **Bool を union 化**（std.fix: `type Bool = unbox union {_false,_true}; true=_true(); false=_false();` ＋ 比較演算子の結果型 ＋ FFI Bool↔i8 tag）。これが `If` を IR から落とす前提（`If`→`Match` desugar は P1 lowering 内）。性能中立（Bool-union＝i8）。de-risk するなら現 `eval_if` を union Bool 対応にして先行検証、または P1 で `eval_if` 撤去と同時。
- **P0.7（早期・P1 と独立に入れられる）: 配列 read-modify-write を atomic builtin へ集約**（builtin `PunchedArray`＋builtin `swap`、`_unsafe_get_linear_bounds_unchecked_unretained` 系・`_unsafe_swap` を廃止）。狙い: **隠れ穴を作る composable primitive を無くして §8 (a) の DOESNT-FIT を完全消滅**させ、各 atomic op に force_unique フラグを付けて **§4 の除去対象に揃える**（swap もソートで全 unchecked 化）。std の swap 版 `PunchedArray`（std.fix:2648）の要素 move 2 回も無くす。作業:
  - **表現は `Array` を使い回す**: `PunchedArray a` は内部に**通常の `Array a` オブジェクトをそのまま持つ**（buffer/LEN/CAP・要素アクセス・`@size` 等は Array と**同一・変更なし**。LEN も減らさない＝穴は idx に据え置き）＋ punch した `idx`（unboxed）を携える（std と同じ `unbox { arr, idx }` レイアウトでよいが **swap はしない**）。
  - **`Array` と違うのは release と clone だけ**: `release(PunchedArray)` ＝ Array の要素解放ループを **idx をスキップ**した版（`[0,idx)` と `(idx,size)` の 2 レンジ、または per-element の skip 分岐）で内側 array を解放。`clone`/`make_unique` も同様に idx をスキップ。**boxed 要素型のときだけ Array と差が出る**（unboxed 要素は per-element release が無く、release/clone は Array と完全同一）。→ 実装は**既存の array デストラクタ/clone 生成コードに「skip index」を渡す変種を足すだけ**（新規のメモリ表現もアロケーションも増やさない）。
  - **release/clone 生成の機構（既存を局所拡張・大幅変更なし）**: release/retain/mark は `create_traverser(ty, work)` が**型のフィールド構成から生成**し、**unboxed 型でも呼ばれる**（`build_release_mark` の unboxed 分岐、`generator.rs:1259`）＝unboxed 型も型ごとの専用 traverser（release 含む）を持つ。フィールド単位のスキップは `ObjectFieldType::SubObject(ty, is_punched)` の `is_punched` で表せる（`build_traverse` の `if *is_punched { continue }`、`object.rs:1646`。struct 側 punch がこれを使う）。**足すのは「配列要素 1 個のスキップ」のみ**——`build_traverse` の `Array` 分岐（`object.rs:1670-1674` の `release_or_mark_array_buf`）に、兄弟 `I64`(idx) を読んで idx を飛ばす skip-idx 版（既存 `loop_over_array_buf`〔`object.rs:321`〕を `[0,idx)`+`(idx,size)` で 2 回呼ぶ）を足し、`PunchedArray` 型の traverser でそれを使う。retain/mark も同じ traverser（work 違い）で一括スキップされる。clone（`make_unique`）は traverser でなく `clone_array_buf` 経由なので、共有され得る場合は skip-idx clone も要る（**線形フローでは PunchedArray は常に unique で clone は走らない**ため堅牢性目的）。∴ **既存機構の局所拡張のみで、codegen の大幅変更・boxing は不要**（`unbox { Array a, I64 }` のまま skip-idx traverser を持たせ、std 版の swap-to-end を skip-idx traverser に置き換える）。型名は `PunchedArray`（std 同名 struct を builtin で置換）。
  - **punch/plug（新 builtin・swap 無し）**: `punch : I64 -> Array a -> (PunchedArray a, a)` ＝ idx の要素を no-retain で move-out、穴は idx に据え置き、idx を tag。**force-unique 版も用意**（`mod`/`act` の §4 除去がこの上に乗る）。`plug : PunchedArray a -> a -> Array a` ＝ idx に書き戻し（unreleased）て Array へ戻す。所有権は **PunchedArray＝idx 以外の全要素／取り出した要素＝idx** に分割される（∴ 取り出した要素は正真正銘の所有末端＝§8 (a) が CLEAN 化）。
  - **抽象に保つ**: 内側 `Array` をフィールド取り出しさせない（取り出して通常 `Array` として release すると idx 二重解放）。punch/plug/release/clone 以外の操作を生やさない。
  - **builtin `swap`（force_unique フラグ付き）**: `swap : I64 -> I64 -> Array a -> Array a` を単一 InlineLLVM op（`ArraySwapBody`）で codegen 直書き（`set` 同様＝超高速）。`i,j` の要素を op 内部で move 交換（`read_from_array_buf_noretain`＋`write_to_array_buf` を op 内に閉じ込め、穴を外に出さない）。`set` と同じ **`force_unique` フラグ**を持ち、§4 が配列を静的 unique と証明できれば unchecked 版へ差し替え → ソート（introsort/heapsort、std.fix:439/461/466/473/527/555）の O(n log n) 回の swap から uniqueness チェックが消える。arg: `i,j` unboxed・`array`=`Own`、result=`Fresh`（force-unique 版・unchecked 版とも結果は unique）＝**CLEAN**（穴を露出しないので §8 (a) の窓が消える）。
  - **廃止と置換**: `_unsafe_get_linear_bounds_unchecked_unretained`(+`_forceunique`) と `_unsafe_swap_bounds_uniqueness_unchecked` を**廃止**し、利用者を atomic builtin へ移す—— `mod`/`act`（単一穴、std.fix:166/174/190）→ PunchedArray punch/plug、`swap`（二重穴、570/571）→ builtin swap、`pop_back`（`set_size(len-1)` 後に**境界外**の末端を取り出す、366）→ 専用 builtin（shrink＋末端 move-out を op 内に閉じ込め）、generic `act`・std swap 版 `PunchedArray`（2668/2692/2698）→ builtin PunchedArray。**`_unsafe_set_bounds_uniqueness_unchecked_unreleased` は残す**（穴埋め用途は消えるが、append/push_back/map/reserve の**未初期化スロット書き込み**という別用途で正当に使う。265/307/383/401/689）。生 codegen helper（`read_from_array_buf_noretain` 等）は各 atomic builtin の**内部**に閉じ、Fix レベルの composable op としては露出させない → **隠れ穴 primitive はどこにも残らない**。
  - **削除チェックリスト（P0.7 完了後に確実に消す。残すと再利用される）**: (i) `_unsafe_get_linear_bounds_unchecked_unretained`(+`_forceunique`)＝builtin.rs `InlineLLVMArrayUnsafeGetLinearBoundsUncheckedUnretained`＋stdlib.rs 登録＋std.fix 呼び出し元（166/174/190/366/570/571/2668/2692/2698）＋doc。(ii) `_unsafe_swap_bounds_uniqueness_unchecked`（`*_uniqueness_unchecked` 族。std.fix:567 定義＋呼び出し元 439/461/466/473/527/555）。(iii) std `PunchedArray` 型（2648）＋`_plug_in`（2658）＋`_unsafe_punch_bounds_uniqueness_unchecked`（`*_uniqueness_unchecked` 族。2686）。**`#[deprecated]` 等で残さず物理削除**（CLAUDE.md の dead-code 方針）。
  - **`_unsafe_set_bounds_uniqueness_unchecked_unreleased`（Rust builtin）は P0.7 で完全には消えない**: hole-plug 利用（mod/act/swap）は上記で消えるが、**未初期化 fill 利用（push/append/map/reserve、265/307/383/401/689）が残る**。これも `*_uniqueness_unchecked` 族なので **P3.5 で置換・削除**（下記）。**恒久的に残すもの（別軸）**: `_unsafe_set_size`・`_unsafe_empty_capacity_unchecked`・`_unsafe_fill_size_unchecked`（size/capacity）・`_unsafe_get_bounds_unchecked`（bounds＝§6 の軸）・`_unsafe_force_unique`（安全な force-unique）。
  - **穴 release のデッドコード確認（要望）**: `mod`／total-functor `act` は punch を必ず plug するので PunchedArray を drop せず、その skip-idx release を**呼ばない**。PunchedArray を unbox 型にして release を drop 地点でインライン生成する実装なら、drop 地点の無いプログラムでは skip-idx release が**そもそも生成されない**（望ましい）。`emit_symbols`／IR を grep して `mod`/`act`(total) のみのプログラムで skip-idx release が出ないことを確認。型ごと無条件生成する実装なら LLVM/リンカ DCE で strip されることを最終バイナリで確認。generic `act`（失敗し得る functor は PunchedArray を drop し得る）では release は**実使用**なので大域 dead ではない（total path でのみ dead）。
  - **独立性・検証**: RC IR 導入前に単独で入る（挙動不変の内部最適化＝changelog 不要）。共有値テスト（2 箇所格納で破壊しないこと）＋回帰ベンチ（`benchmark/speedtest`・`fix-bench/batch`）で std PunchedArray の swap トリック除去分（generic `act`）の改善と非劣化を確認。assert ビルドで穴の二重解放を検出。
- **P1**: RC IR 型 ＋ AST→RC IR lowering（`generator.rs` から RC 抽出。名前は lowering が fresh 発番）＋ codegen 付け替え ＋ 全テスト再検証。**最大の山**。完了ゲート: `cargo test --release` 全最適化レベル（`FIX_MAX_OPT_LEVEL` max/basic/none、§1.6）で全通過・全ベンチでリグレッションなし・デバッグ情報一致（§1.6）を満たし、**ユーザに連絡して外部ライブラリテストを依頼してから次フェーズへ**。lowering は現 codegen の RC（move-out/last-use ＝既に最小 RC）を踏襲し、引数は全 `Own`（ベースライン）。source 関数の borrow 化・相殺（§2）は P2 で uniqueness と併せて入れる（前処理）。なお `InlineLLVM` の `OwnershipShape` 宣言（read getter は `Borrow`。§1.2）だけでも hot loop の array getter は借用扱いになり、ベンチの elision の主要部はそれで届く。
- **P2**: 前処理（borrow 化 §2.1・相殺 §2.2 で `OwnershipShape` を確定し `Retain` を削る）＋ uniqueness 解析（`Provenance` を追い resolve で uniqueness を得る）。read-only ログから始め arrayrw のループ `set` を `Unique`・共有される配列のテストを `Dynamic`（非 unique）と判定することを確認。
- **P3**: unique-check-elim（force-unique 除去 ＋ 特殊化）。**特殊化 clone の dead-function 除去（到達解析＋未到達 `RcFunc` 削除）を同時に実装**——未使用 clone を掃除しないと未到達関数がバイナリに残る（§4.1）。arrayrw/fannkuch 計測、全テスト。
- **P3.5（§4 後の `*_uniqueness_unchecked` 掃除）**: **`*_uniqueness_unchecked` 族**（uniqueness チェックを飛ばして possibly-shared を黙って mutate する危険関数）は §4 が uniqueness を静的に扱えるので価値を失う → 全廃する。swap/punch は P0.7 で builtin 化済み。残るのは `_unsafe_set_bounds_uniqueness_unchecked_unreleased` の**未初期化 fill 利用**（push/append/map/reserve、265/307/383/401/689）——これを、上流の `_unsafe_force_unique`（§4 が elide）＋「未初期化スロット write」の狭い内部 primitive（uniqueness は上流で保証、名前から `uniqueness_unchecked` を落とす）へ置換して削除。手順: **安全版に置換 → §4 が overhead を消すことを確認 → 物理削除**（`#[deprecated]` で残さない）。§4 が unique を証明できない箇所はチェックが残る（安全側の正しい挙動＝precision は §2 で上げる）。`_unsafe_force_unique` 自体は安全（force-unique する）ので残す。狙い: possibly-shared を黙って壊す誤用しやすい unsafe を撲滅し、安全版と同速を §4 で担保。
- **P4**: reuse / 順序スケジューリング / 境界チェック除去 等。
- **（途中経過の性能に関する注意）**: `*_uniqueness_unchecked` の除去（P0.7 の swap/punch、P3.5 の未初期化 fill）は、**§4（P3）が force-unique を静的に飛ばすまでの間、一時的に性能を下げる**。特に **swap** は現状 uniqueness チェックを完全に飛ばしているので、force-unique フラグ付き builtin へ置換すると P0.7〜P3 の間はソートの各 swap に refcount チェックが 1 個乗る（`mod`/`act` は元々 `_forceunique` なのでほぼ中立）。**これは unsafe 除去による想定内の劣化で §4 で解消予定**。中間マイルストーンで benchmark を取るときは、劣化を「後で §4 が解消する分（unsafe 除去起因）」と「意図しない劣化」に**振り分ける**こと（commit hash＋どの unsafe をいつ外したかを記録し、§4 適用後に回復するかで判定）。回避したいなら P0.7 の abolish/置換を §4 と同じ P3 に寄せる選択肢もある（早期独立性と引き換え）。

## 8. リスク・未解決
- **P1 の codegen 付け替えの再検証コスト・範囲**が最大リスク（全プログラムに影響）。段階導入できるか（一部関数だけ RC IR 経由、等）も検討。
- uniqueness の precision は「`Retain` を入れないこと」で作る（borrow 化 §2.1・相殺 §2.2）。`Retain` は `Unique -> Dynamic` の一方向で回復しないので、`Retain` を減らせるかが要。
- ローカル名一意の**全変換での保存**（lowering は fresh 発番で構築的に一意。clone/特殊化は fresh 名発番で freshen）。
- getter（射影）の retain 有無・`Provenance` の不動点収束・threaded state・boxed の escape（`boxed_to_retained_ptr`）の RC IR での扱い。捕捉クロージャは `Closure` の captures を `UnboxedAgg` の子の `Provenance` として追えるが、共有/別名の健全性は検証する。
- 別名健全性は「別名を作る操作で対象を `Dynamic` にする」で担保（§3.2）。

### InlineLLVM の `OwnershipShape`/`result_prov` 全件監査（67 件）
全 `LLVMGenerator` variant（`src/fixstd/builtin.rs`）を精読し、2 属性（`arg_ownership(i)`／`result_prov`）を割り当て可能か確認した。内訳の目安: **CLEAN ~41**（数値/比較/cast/literal＝unboxed in/out で宣言不要・結果 `Unboxed`、構築＝`Fresh`、read-only getter〔size/capacity/`union is`〕＝`Borrow`・結果 `Unboxed`、`empty`/`fill`＝`Fresh`、`DestructorMake`＝子を move-in し `Fresh`）、**NONTRIVIAL ~19**（下記(1)。割り当て可だが実装者が誤りやすい）、**DOESNT-FIT ~7**（下記(2)。2 属性で綺麗に表せない）。

**(1) 宣言を誤りやすい（P1 監査のチェックリスト）**:
- **force-unique 内包 → 結果 `Fresh`**（返るポインタが入力と同一でも一意保証）: `set`・`Array::force_unique`・struct `set_`/`mod_`(fu)・`plug_in`(fu)・`_unsafe_mutate_boxed_*`。
- **force-unique しない in-place → 結果 `Arg(i)`**（同じ・共有かもしれないポインタ。**`Fresh` にしない**）: `_unsafe_set_size`・`_unsafe_set_bounds_uniqueness_unchecked_unreleased`・`punch`(fu=false)・`plug_in`(fu=false)。後者2つは「古いスロットを release しない」内容 RC も伴い、呼び出し側の move-out 前提でのみ健全。
- **boxed 容器から要素/フィールドを取り出す getter → `Dyn`**（retain されるが容器と共有＝unique でない。**`Fresh` にしない**）: `_unsafe_get_bounds_unchecked`（要素 boxed 時）・boxed struct の `@field`・boxed union の `as_`。
- **container の boxed/unbox で結果が変わる**（一律に扱わない）: struct `@field`／union `as_` は boxed 容器 → `Dyn`、unbox 容器 → `Arg(i,[field])`（move-out・no-retain の素通し。retain されないので「retain 済みコピー」と誤ると二重計上）。
- **passthrough（値を返すが refcount は変えない）→ 結果 value 部 = `Arg(0)`**: `unsafe_is_unique`（`(Bool, a)` の `a`）・`mark_threaded`。前者は refcount を**読む**だけ、後者は **state を変える**だけで、どちらも `Retain` ではない（`Fresh` を潰さない）。
- **`union mod` の結果は分岐の phi = `{Fresh, Arg(1)}`**（match 枝＝新規構築、mismatch 枝＝入力 union 素通し）。
- **値未使用・型 witness のみ → arg `Borrow`・結果 `Ptr`＝`Unboxed`**: `get_retain_function_of_boxed_value`／`get_release_function_of_boxed_value`（引数の**型**だけ使い値は捨てる。返すのは helper 関数ポインタ＝rc 対象外）。
- **`union is` の RC idiom**: `get_scoped_obj_noretain`＋`if !used_later release`（実効 `Own` だが transient retain 無し）。`get_scoped_obj` ベースの `Own` パターンと形が違うので検査が見落としやすい（§8 分類A の read getter 群と同類）。

**(2) 2 属性で綺麗に表せない（opaque 化 or 別扱いが要る）**:
- **(a) タプル内の move-out 穴 → builtin `PunchedArray` で解消（決定・早期フェーズ）**: `_unsafe_get_linear_bounds_unchecked_unretained`（`mod`/`act` の中核）は `(Array a, a)` を返し、要素を **no-retain で move-out**（builtin.rs:1946）＝tuple.0 の配列は `idx` に**穴が空く**（stale スロット）。tuple.0 の型が平の `Array a` で**穴が型に出ない**ため、通常の配列デストラクタが `idx` も解放しにいき、tuple.1 と **idx 要素を二重に所有**するように見える（`Provenance` の各 boxed 末端＝所有参照、の前提を破る）。これが破綻の根本（症状としては「借用末端／refcount 共有」）。
  - std の `PunchedArray`（`unbox struct {_arr, _idx}`, std.fix:2648）は穴を**末尾へ swap＋@size 減で境界外へ追い出して** release-safe にしているが、`idx != 末尾`で**要素 move が 2 回余分**（drop され得る generic functor 対応のため）。
  - **解消: `PunchedArray` を builtin 抽象型化**（`{Array a, idx}`、専用デストラクタ／clone が **idx をスキップ**）。穴は idx に据え置き（**swap 無し・実行時ゼロコスト**、idx は unbox でレジスタ）、所有権が **PunchedArray＝idx 以外／element＝idx** にきれいに分割され、tuple.1 は正真正銘の所有末端・穴は型に出る → shape 解析が普通の boxed 型として扱え **(a) は CLEAN 化**。droppable なので最適化パス（total functor）と generic パスを一本化でき、std の swap 版を上位互換で置換、「linear 窓を触らない」壊れやすい不変条件も不要。punch/plug のみの抽象型に保つ（内側 `Array` を取り出させない）。
  - この置換で linear-get の unsafe 3 点のうち **(3) move-out の refcount プロトコルが型で安全化**され、残る unsafe は (1) 境界（＋非 forceunique 版の (2) 一意性）だけになる。§4 の force-unique 除去は punch 内で効くので不変。早期フェーズで入れる（§7）。
  - **PunchedArray が扱うのは穴 1 個**（`mod`/`act`）。`swap` のように**穴を 2 つ同時**に開ける op は単一 idx の PunchedArray に載らないが、**builtin `swap`（force_unique フラグ付き）として atomic 化**する（§7 P0.7）——穴を op 内部に閉じるので窓が露出せず、これも CLEAN・§4 除去対象（ソートで全 unchecked 化）。生 `_unsafe_get_linear_bounds_unchecked_unretained`/`_unsafe_swap` 系は廃止（§7 P0.7）＝**隠れ穴を作る composable primitive はどこにも残さない**ので、DOESNT-FIT (a) は完全消滅する。
- **(b) boxed 内部への生ポインタ**: `Array::_unsafe_get_ptr`（`Array a -> Ptr`, builtin.rs:2393）・`get_boxed_data_ptr`（`a -> Ptr`）は結果 `Ptr`（`Unboxed`）が**引数 boxed の内部バッファを alias**する。現 codegen は引数を last-use で release し得る＝返した `Ptr` が dangling しうる（`unsafe` op）。RC IR では引数を **`Borrow`**（呼び出し側が Ptr の生存中 配列を保持）と宣言するのが正しく、`Ptr` 結果は `Unboxed` で解析は alias を追わない——寿命義務はモデル外（呼び出し側責務）。「unboxed 結果が boxed 引数の内部を alias しその寿命に縛られる」は 2 属性の外。
- **(c) RC 追跡域の境界**: `boxed_to_retained_ptr`（boxed の 1 参照を生 `Ptr` へ escape。arg=`Own`、結果 `Ptr`=`Unboxed`＝生きた参照が帳簿外へ）／`boxed_from_retained_ptr`（生 `Ptr` の参照を boxed `a` へ materialize。結果=`Dyn`）。前者は「`Unboxed` の引数/結果が実は所有権を運ぶ」非対称で `Own`/`Borrow` 軸に載らない（§8 冒頭の boxed escape 未解決項目）。健全側には「to→arg `Own`・結果は追跡外／from→結果 `Dyn`」で倒す。
- **(d) `with_retained`**: 呼び出しをまたぐ意味的 retain。arg=`Own`/`Own` では意味が抜ける。**opaque のまま常に retain**（§8 の (B) の結論を再確認）。
- **(e) `fix`（`FixBody`）**: `free_vars` に Fix レベルの仮引数でない暗黙 capture `#CAP` が含まれ、それを合成 closure `fixf` に alias 格納して `f` が消費する（中間 closure のフロー）。RC IR では `LLVM(FixBody,[x,f,cap])` の全 `Own`＋内部 RC opaque で扱う（§8 の fix 項で既述）。per-Fix-arg だけ見ると `cap`（=`#CAP`）を見落とす点に注意。

**(3) 監査で裏取りできた既存 PLAN 決定**:
- unbox union の結果は「variant ごとの `UnboxedAgg`＋非活性 variant ⊥」で表す（§3.1/§3.3）。`make_union`/`union as`/`union mod` の unbox 版がこの表現を要求＝§3.3 で足した union 構築宣言が load-bearing と確認（tag 付き sum を positional product＋⊥ で表す点が肝）。
- `unsafe_is_unique` の value 部＝`Arg(0)` passthrough（§8）で `Fresh` を保つ、は妥当と確認。force-unique 内 clone を op に内包（§3.3/§4.2/§8）も確認。

### 決定事項・要確認
- **（決定）状態は変数ごとの `Provenance`（`State{env: Map<Var,Provenance>}`）、uniqueness は resolve して得る**（§3）: 各 boxed 末端の由来（`Fresh`/`Dyn`/`Arg`）を追い、`is_unique` は入力に resolve して `Unique` か見る。boxed 容器の中身は追わない（取り出しは `Dyn`）。`Fresh -> Dyn`（＝`Unique -> Dynamic`）を起こすのは `Retain`（複製＝2つ目の参照）だけで、`Dyn` は吸収状態（`unique_ptr`/`shared_ptr` 対応）。global・`boxed_from_retained_ptr`・join も由来を `Dyn` にする。move（`Let(x, Var(y))`）は由来を引き継ぐ。unboxed 集約は `UnboxedAgg` で子の由来を追う（`LoopState` 越しの配列など線形な受け渡しの精度）。
- **（決定）`Construct` ノードを設けず構築も `LLVM`**（§1.2）: 集約構築（struct/タプル/`ArrayLit`/union variant）は alloc 系 `LLVM` プリミティブ＋`Provenance`（引数をスロットへ move。結果は boxed 集約＝`Fresh`、unboxed 集約＝子の由来を担ぐ）で表す。InlineLLVM が効果を宣言する設計なので boxed alloc の Fresh も unboxed 集約が担ぐ子の由来も解析に伝わる（専用ノードを持たない。射影＝getter を専用ノード化しない方針の双対）。
- **（決定）boxed rc を `Unique | Dynamic` の 2 点で表す**（`CTRefCnt`、§3.1）: `alloc=Unique`、`Retain`->`Dynamic`（一方向）、`Release`・呼び出しは `Dynamic` を回復させない。これにより (a) **関数効果 ＝ 結果の由来 `Provenance`** で書ける（引数の生存は `OwnershipShape` が決める＝`Own`+`Unique` は last-use で dead、`Dynamic` は据え置き、`Borrow` は存続。§1.2）、(b) precision は borrow 化（§2.1）＋相殺（§2.2）で `Retain` を減らして作る、(c) 2 点束かつ単調遷移なので不動点は自明。`Provenance` は入力非依存なので `FuncRef` ごとに 1 つ（入力で分けるのは §4.1 の特殊化だけ）。
- **（決定）force-unique op の結果 `Provenance` は `Fresh` 固定（unchecked 化しても `Arg` 素通しにしない）**（§3.3/§4）: `set`/`swap`/`mod`(punch) 等は、入力が unique なら in-place で**入力と同一 object**を返し、shared なら clone で**別 object**を返す——結果の identity は入力依存だが、**uniqueness はどちらも `Unique` で入力非依存**。`Fresh` はこの入力非依存な uniqueness を表す。§4 で unchecked 版に差し替えた op は実際には入力を alias する（真の identity は `Arg(i)`）が、そこで provenance を `Arg(i)` に更新すると「checked=`Fresh`／unchecked=`Arg(i)`」＝§4 の除去判断に依存＝**入力依存**になり、§3↔§4 の反復不動点が要る（「provenance を 1 回計算・入力非依存」という設計が壊れる）。§4 は uniqueness しか読まず `Fresh` と `Arg(i)` は resolve 結果が同じなので**除去の得はゼロ**。identity/別名が効くのは §6（reuse・順序スケジューリング）で、そこは elim 後に「どの結果が入力を alias するか」を独立の局所パスで求めればよく、コア provenance に混ぜない。∴ `Fresh` 固定。
- **（決定）`RcRhs::Var(Var)`（move-bind）を持つ**（§1.2/§3.2）: `let y = x` を表せる。意味は move（`x` 消費・`y` が由来 `Provenance` を引き継ぐ、`Unique` も継ぐ）で rc 中立、それ自体は `Dynamic` トリガーでない。エイリアス（`x`,`y` 両方生存）は「後でも `x` を使う＝non-last use なので手前に `Retain`（`->Dynamic`）」で出る＝copy = `Retain` + move。copy propagation で消せる。
- **（決定）borrow 化を source 関数へ拡張**（§2.1）: lowering の all-`Own` を、読むだけの引数に限り `Own` -> `Borrow` へ**書き換える**。引数ごとの `OwnershipShape`（`UniquenessShape` 同型、末端 boxed に `Ownership`）を、消費の有無からコールグラフ上の最大不動点（初期 `Borrow`、消費で `Own` に降格）で決め、callee の内部 `Release` を落として呼び出し側へ出す（余る `Retain`/`Release` は §2.2 が相殺）。uniqueness 解析の前に走らせる。P2。
- **（決定）Bool→union（P0.5、§7）**: std.fix 定義＋比較演算子の結果型＋FFI（Bool↔i8 tag、`_false`=0/`_true`=1）。`If`→`Match` desugar は P1 lowering 内。要確認: 比較 InlineLLVM の結果構築・`&&`/`||`/`not`・typecheck が union Bool で通るか（ビットは i8 不変）。
- **（決定）global 値の表現**: global 初期化を RC IR（init）として表し `MarkGlobal` を init で発行。参照は atom で、解析は値の型どおり（boxed 末端は `Dynamic`）。program = top-level 関数集合 ＋ global init。現状の global 機構（lazy/eager・mark_global 発火点）は P1 実装時に確認。
- **（決定）lowering サブパス順**: AST 正規化（ANF 化 → lambda lift → `If`→`Match` desugar → destructure→getter → fresh 命名）→ 最後に last-use 解析＋明示 retain/release 挿入で RC IR 生成（形と名前が確定してから RC を載せる）。
- **（調査済み）RC site 監査の規模**: codegen の RC は `generator.rs` ~38・`builtin.rs` ~29（InlineLLVM `generate` 内部の release/retain）・`object.rs` ~21。builtin の 29 を「primitive 内 atomic（`make_array_unique` の clone-release 等、op 意味に内包）」「明示 `Release` 化すべきもの（引数を使用後に release 等）」「外部化できず宣言で残す内部 RC」に分類するのが P1 の主要監査。
- **（調査済み）`is_var_used_later` 依存の InlineLLVM**（`builtin.rs` 全10 site を分類。いずれも RC 判断のみで計算結果・挙動は used_later に非依存＝in-place/clone はランタイム refcount で決定）: **(A) 借用読み後 last-use なら引数 release**（`noretain` 読み＋`if !used_later release`）＝1855（配列要素 get）/2418（配列 ptr）/2489（get_size）/2540（get_capacity）/3873（union `is`）/4546・4648（retain 関数 ptr 取得）/4755（data ptr 取得）の 8。**(B) 呼び出しをまたぐ retain/release**（`with_retained`: `f(x)` の前後で x を retain→release し呼び出し中 x を生存）＝4206+4214。RC IR では (A) は容器引数を `Borrow`（借用）で宣言し、lowering の last-use 解析が容器の明示 `Release` を last-use に配置（getter が boxed 要素を retain するのは別効果）。(B)（`with_retained`）は **opaque な InlineLLVM のまま retain/release を内部に埋める**。この `Retain` は呼び出し中 x を shared に見せ f の in-place 変更を防ぐ**意味的** RC で、最適化で消えては困る＝外に出すメリットが無くリスク（相殺で消える）だけなので内部に残すのが正しい。used_later スキップは落として**常に retain**（内部 RC は codegen 時に used_later を見ない）。P1 の書き換え: (A) は used_later を `Borrow`＋lowering の last-use 解析へ移す、(B) は常に retain へ。どちらも `generate` は `is_var_used_later` を呼ばなくなる（grep 由来なので網羅監査）。
- **（要確認）各 InlineLLVM 引数の `OwnershipShape` と `Borrow` 化可否**: read-only op（§8 分類A）は既に `noretain`（借用的）なので `Borrow` に素直に対応。`Own` で retain してから読む引数があれば `Borrow` 化＋release 外出し＋相殺で速くなる（§6）。`Borrow` 化できない op もある。全件確認が要る。**`fix`（不動点コンビネータ）・bulk array 系**が `Borrow` 化できない候補（`loop` は InlineLLVM op でなく std の再帰関数なので対象外＝§3.2 の不動点で透過的に解析される）。P1 監査で各引数を「`Borrow` 化可／`Own` のまま（内部 RC を宣言で残す）」に分類する。
- **force-unique 内 clone の RC 境界**: `make_array_unique`/`make_struct_unique` の clone（共有時に deep copy ＋要素 retain）は op の atomic 意味に内包し、内部 RC は IR ノードに出さない（最適化対象でない共有パスのため）。引数 `OwnershipShape` のみ宣言。
- **（調査済み）ソースレベルの `unsafe_is_unique` 分岐**（std.fix 3 箇所）は §4 の対象外だが健全なので削除しない: (1) generic `Array::act`（`_unsafe_act_bounds_unchecked`、任意 functor）が `arr.unsafe_is_unique` で unique(punch)／shared(clone+set) を実行時分岐する。`optimize_act`（`src/optimization/optimize_act.rs`、既存・`enable_act_optimization`）が Identity/Const/Tuple2 の act body を force-unique op 版（`_unsafe_act_bounds_unchecked_{identity,const,tuple2}`）へ lowering 前に置換するので、ホットな act はこの分岐を通らず §4 の対象になる（`mod` は元から force-unique op 版）。それ以外の functor の act だけ分岐が残り、RC IR では `unsafe_is_unique` の結果 `Bool` 上の runtime `Match` に lower される（§3.2 が健全に扱い、§4 は差し替えない）。(2) `Destructor::mutate_unique_io`（FFI 資源の複製判断）・(3) `assert_unique`（デバッグ、assert ビルド §1.6）は配列/struct ホットパス外の正当用途で残す。削除は不適（(1) は op の途中でユーザ closure を呼ぶため単一 op に畳めない）。P1 監査: `unsafe_is_unique` の `result_prov` を passthrough（`Arg(0)`）と宣言して `Fresh` を保つ（精度）。unique-check-elim 実行時は act 最適化を有効にしておく。
- **（確認済み）高階イテレータの直接化は既存 decapturing に依存**: `loop`/`fold` 等は body を closure 引数で受け、その内部呼び出しは間接。uniqueness 特殊化（§4）が内部の force-unique op に届き、Provenance 解析（§3）が精度を出すには、この間接呼び出しが直接化されている必要がある。既存の **decapturing の closure specialization**（`src/optimization/decapturing.rs`、`enable_decapturing_optimization`。`inline` 後・`uncurry` 前＝RC IR lowering の前）が、body を焼き込んだ特殊版（`loop#lam` 等）を生成し内部呼び出しを直接化する（`pull_let` が適用範囲を広げる）。ベンチの `loop((0,arr), |…|)` は適用対象（lambda を直接渡す・自己呼び出しで同 index）。適用外の形（doc の制限: lambda をタプル等に入れて渡す等）は間接のまま残り、§3.2 の規則で結果 Dyn＝除去は効かない（健全・許容）。unique-check-elim を回すときは decapturing を有効にしておくのが前提。
- **（確認済み）`fix`（ローカル再帰の不動点コンビネータ）は RC IR で表現可能**: std `fix = |f| |x| FixBody`（`FixBody` は InlineLLVM、free vars `x,f,cap`）。lift で outer `|f|`／inner `|x|` の `RcFunc` になり、`fix(f)`=`Closure(inner,[f])`、本体は `LLVM(FixBody, [x,f,cap])`（全 `Own`）。FixBody は自己 funptr（codegen の `get_parent`）＋現 cap 再利用で `fixf=fix(f)` を作り `f(fixf)(x)` を呼ぶ（heap alloc なし、RC cycle 無し＝fixf→f だが f→fixf 無し）。内部 RC は宣言で残す（opaque・`Borrow` 化不可）ので fix 内再帰は解析から保守的に見える。`Closure(self)+App` への desugar は避ける（cap 再利用を失うため）。

## 9. フェーズ別テスト計画（検証入力）
各フェーズが「想定どおり動く」ことを、どの入力コードで・何を観測して確認するか（§7 のマイルストーンに対応）。統合テストは CLAUDE.md 規約（`main` から実際に実行・参照、tempdir へコピー、`Command::new("fix")` で `fix` 実行）で書く。

### 9.0 共通の検証基盤（複数フェーズで使う。P0/P1 で先に整える）
- **shared-value テスト（最重要の正しさパターン）**: 値を 2 箇所に格納（＝別名を作り共有に）→ 一方を in-place 系 op で mutate → **他方が壊れていないこと**を assert。`let a = [1,2,3]; let keep = (a, a); let a2 = a.set(0, 99); eval (keep.@0.@0, a2.@0)` が `(1, 99)`。clone-on-shared の健全性を突く（`set`/`mod`/`act`/`swap`/struct 系 全部に効く）。
- **assert ビルド（健全性）**: 「unique と判定した値が実行時に共有なら abort」するビルドモード。全テストをこれで走らせ、uniqueness 判定の誤りをゼロ検出で確認（§1.6/§5）。
- **除去の観測（IR/asm チェック）**: 一意文脈で `build_branch_by_is_unique` 由来の分岐が emit IR/asm から**消えている**こと、共有文脈で**残っている**ことを grep で確認。`fix` に IR/asm ダンプ経路が要る（既存の emit を利用）。
- **provenance/ownership ダンプ（P2 の照合用）**: 各関数の結果 `Provenance`・各変数末端の由来・各引数の `OwnershipShape` を出す debug 出力（`optimize_act` 等が使う `emit_symbols` と同様の仕組み）。P2 はこれで期待値照合する（この経路を作るのが P2 の前提作業）。
- **cachegrind 計測**: `fix-bench/batch/arrayrw{,_unsafe,_fn}`・`fannkuch`・ソートを commit hash 付きで instruction 数記録（§0 の目安表と比較）。
- **leak/double-free**: valgrind か assert ビルドで、boxed 要素の `mod`/`act`/`swap`・深い再帰・クロージャ捕捉を回すストレス。

### 9.1 P0（デバッグ情報ベースライン）
テスト自体が成果物。入力＝行構造が既知の小プログラム、`fix build -g` → `gdb -batch`（`break main.fix:N` → run → backtrace）。file:line 解決・停止・スタックの行情報（マングル名非依存）を assert。**現 main で通す**（P1 の比較基準）。

### 9.2 P0.5（Bool→union）
Bool の挙動・ビットが不変であることを突く。入力（`main` から eval して結果 assert）:
- 比較: `3 == 3`, `5 < 2`, `2 <= 2`, `1.0 < 2.0`。`&&`/`||`/`not`: `true && false`, `true || false`, `not(true)`。
- 分岐: `if b {..} else {..}`・`match b { true() => .. }`。
- FFI: Bool を i8 として C 関数へ渡す/受ける（tag `_false`=0/`_true`=1）。
観測: 実行結果一致＋既存全テスト通過。性能中立（i8 のまま）。

### 9.3 P0.7（builtin PunchedArray + builtin swap）
`mod`/`act`/`swap` の正しさ・PunchedArray の drop 安全・リーク無しを突く。入力:
- 正しさ（unboxed/boxed 要素）: `[1,2,3].mod(1, |x| x+10)` == `[1,12,3]`、`[[1],[2]].mod(0, |x| x.push(9))`、`[1,2,3,4].swap(0,3)` == `[4,2,3,1]`、ソート結果。
- **shared-value**: 上記 9.0 の型を `mod`/`swap` でも（`keep` 側が不変であること）。
- **PunchedArray drop 安全（skip-idx release を突く核）**: 失敗し得る functor の generic `act`（`[[1],[2]].act(0, |_| Option::none())`）で **plug されず PunchedArray が drop** される経路 → リーク/二重解放なし（valgrind/assert）。
観測: 実行結果＋shared-value＋leak。ソートベンチ（§4 前なので劣化は想定内＝§7 の注意書きどおり振り分け）。

### 9.4 P1（RC IR + lowering + codegen 付け替え）
挙動保存（無回帰）の**最大の検証**。入力＝**既存全テスト＋全ベンチ**（§1.6）＋ RC ストレス（boxed 捕捉クロージャ・深い再帰・ループ内 boxed・共有構造）。観測: `cargo test --release` を `FIX_MAX_OPT_LEVEL` max/basic/none で全通過／全ベンチ commit hash 比較で無回帰／9.1 の gdb テストでデバッグ情報一致／leak チェック／RC 挿入数・順序・解放挙動が現状一致（assert ビルド or RC ダンプ）。

### 9.5 P2（borrow化 + 相殺 + provenance 解析）
解析が期待どおりの値を出すことを 9.0 の provenance/ownership ダンプで照合。入力と期待:
- `|x| let y = [x]; (y, y)` → 両末端 `(Dyn, Dyn)`（複製 Retain。§5）。`id(x)` 結果 `Arg(0,[])`。`set(i,v,arr)` 結果 `Fresh`。
- §3.4 arrayrw ループ → `loop` 結果 `{Fresh, Arg(0,[1])}`、`main` で `{Fresh}`＝Unique。§3.5 read-only `sum` → `arr` は `Arg` 素通し、`main` の set 前 `Fresh`＝Unique。同じ配列を共有してから渡す版 → `Dynamic`。
- borrow化: read-only 再帰 `sum` → `own[sum.arr] = Borrow`（§2.1）。case B `loop_fresh` → fresh を受ける param が `Own` 固定。相殺 → 借用呼び出しをまたぐ `Retain … Release` が消える。

### 9.6 P3（unique-check-elim + 特殊化 + dead-func 除去）
§5 の全項目。入力と観測:
- **一意文脈で除去**: arrayrw ループ（threaded unique array の `set`/`mod`/`swap`）→ 分岐消滅（IR grep or cachegrind 減）。§5 マトリクス全セル（Array の set/mod/act(Id)/act(Tuple2)/swap、boxed struct field の set/mod/act）。
- **共有文脈で非除去**: 9.0 shared-value → 分岐残存（IR grep）＋他方不変。
- **初回 checked・以降 unchecked**: shared で入り初回 `set` で unique 化するループ → `loop@D -> loop@U` の 2 clone（IR に 2 clone、cachegrind で初回だけ高コスト）。
- **入れ子伝播**（§5）: タプル内配列 `loop((cnt, arr), …)`・struct 内配列・配列内 struct・union 内配列（`LoopState`）。
- **dead-func 除去**: 特殊化後に未到達 clone・元関数が消える（emit symbol 数／バイナリサイズが膨れない）。
- **健全性**: 全テストを assert ビルドで（unique 誤判定 abort）。
観測: IR/asm 分岐 grep／cachegrind（§0 目安の array 部を狙う）／shared-value／symbol・size／assert。

### 9.7 P3.5（`*_uniqueness_unchecked` 削除）
安全版＋§4 で同速・関数消滅を突く。入力: 同じホットパス（ソート・push/append/map）を安全版で → §4 が除去し**旧 unsafe 版と同速**（cachegrind 比較）。削除した関数への参照が無い（使えばコンパイルエラー）。全テスト＋shared-value。観測: cachegrind 同等／grep で不在／全通過。

### 9.8 P4（reuse / 順序スケジューリング / 境界チェック除去）
- reuse: `Release` 直後の alloc が再利用される小例（cachegrind で alloc 数減）。
- 順序スケジューリング: `f(arr.set(0,42), arr.@0)` を並べ替えて `set` が in-place 化（clone 消滅、cachegrind）。
- 境界チェック除去: `idx ∈ [0,size)` 証明で完全 unchecked、一意性除去と合成でベクトル化 → arrayrw が C 比 0.20x（§0）。
各 正しさ（実行結果）＋cachegrind。
