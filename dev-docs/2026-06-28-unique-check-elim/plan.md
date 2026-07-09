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

**継続入れ子・単一 enum（終端子つき）**形式を採る。`Let`/`Retain`/`Release` は継続を持つ「文」的ノード、`Ret(Var)`（式の値＝返す変数）が唯一の終端子。`mark_threaded`/`mark_global` は専用ノードでなく**値を産む InlineLLVM op**（下記）。`Match` は**値を産む cexp（`RcRhs`）**で分岐を表し、**常に `Let(x, Match, k)`** に現れる（教科書 ANF の `let x = <複合式> in …`）。tail-position は IR に符号化せず**先読み**で導出する（`let x=cexp; Ret(Var(x))` の形＝§1.2 tail 先読み）。この形は reset/reuse・borrow 等の RC 最適化を載せやすい。Fix の既存 AST（再帰的）とも同形で流用しやすい。各ノード・各変数は span と具体型を持つ（§1.1-5,6。下では省略）。
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
    Retain(Var, Path, RcState, Box<RcExpr>),  // x の Path が指す部分木の【全 boxed 末端】を dup（root +1） → 継続。Path=[]＝値全体（全 boxed 末端）、Path=π＝末端/部分木 π。unboxed union 部分木は **tag 分岐**して active variant の boxed 末端のみ dup（現ランタイム traverser と同じ）＝**RC ノードの Path は union 部分木の根で止まり variant をまたがない**（[#F1]: tag 不明箇所で不在 variant を無条件 dup するとゴミ経由 refcount++＝heap 破壊）。RcState で state ディスパッチ。lowering は値全体 `Retain(x, [])` を出し、§2.2 分解・§3.2 降格・§2.1 除去が per-末端 `Retain(x, π)` を生む
    Release(Var, Path, RcState, Box<RcExpr>), // x の Path が指す部分木の全 boxed 末端を drop（-1; 0 で解放、構造辿り） → 継続。Path の意味は Retain と同じ（[]＝全体）
    Ret(Var),                             // 唯一の終端子。「この式（関数本体 or match arm）の値 = この変数」。**Ret は Var のみ**（[#R10-3round10]・lowering と全変換の不変条件）。App/Match/LLVM/Closure は**必ず let 束縛**し、その consume/provenance は `Let` 規則で一箇所処理する（Ret に複合 cexp を許すと escape/consume/tail を各 pass の Ret 規則へ複製する羽目になる——例: `Ret(LLVM(素通し op, [t]))` は t の escape が consume_sites の LLVM 規則〔素通し末端を除外〕でも Ret 規則〔Var 用〕でも計上されず借用誤分類→double-free。正準化で塞ぐ）。tail-position は IR に符号化せず**先読み**で導出（下記 tail 先読み・§2.1）
    // mark_threaded/mark_global は専用ノードを持たず LLVM(mark_*, [x]) op（Own 引数・結果 Dyn）。id 同型だが結果 Dyn＋state 副作用（x を消費し threaded/global handle を返す）
}
type Path = Vec<usize>;  // 値の unboxed 構造への index 列（struct/タプル＝field 番号、unboxed union＝variant 番号を前置＝§2.1 root の k::π）。boxed 末端または部分木を名指す。[]＝値全体。UniquenessShape/OwnershipShape/Provenance（§3.1）の木もこの Path で末端を引く。**用途で 2 種**（[#F1]）: **RC ノード（Retain/Release）の Path は unboxed-union 部分木の根で止まり `k::π` を使わない**（物理的な retain/release は tag 安全でなければならない）／**解析の root/provenance パスは tag 既知文脈（match arm 内）で `k::π` を使う**（コンパイル時の object 同一性・由来で実行時コードを産まない）
struct MatchArm { variant: usize, payload: Var, body: RcExpr }  // payload を単一 Var に束縛（さらなる分解は getter で）。body はその arm の値を末尾 `Ret(Var(..))` に持つ式
enum RcRhs {                              // cexp（複合式）: **Let の RHS にのみ現れる**（Ret は Var のみ〔[#R10-3round10]〕）。App/LLVM の引数は Var（atom）
    Var(Var),                             // move/rename: y := x（x を消費、y が由来 Provenance を引き継ぐ）。rc 中立で別名を作らない（別名化は手前の Retain が担う）。copy propagation で消せる
    App(Var /*callee: closure か funptr の値*/, Vec<Var>),  // closure 呼び出し・直接 funptr 呼び出し両方（codegen が型で振り分け）
    Closure(FuncRef, Vec<Var> /*捕捉*/),  // top-level 関数 + 捕捉変数列 → unboxed の {funptr, 捕捉obj ptr} ペア。捕捉 obj のみ boxed（rc 追跡）、空捕捉は null＝RC-free
    LLVM(InlineLLVM, Vec<Var>),           // 全 builtin: 算術/getter（射影）/set/mod/構築（struct/タプル/ArrayLit/union variant）/fill/リテラル(IntLit 等)/FFI/...。効果は Provenance で宣言（alloc 系は結果 Unique・引数をスロットへ）
    Match(Var, Vec<MatchArm>),            // 唯一の分岐構造（Bool もここ）。値を産む cexp: 常に `Let(x, Match, k)`。tail match は継続が `Ret(Var(x))` の形（先読みで判定）
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
// `Borrow` 化の上書き（**§6** borrow 最適化・[#F6]。§2.1 の 2 版化は source param のみで InlineLLVM は触らない）: `set_borrow(i)` が arg i の OwnershipShape 末端を `Borrow` に切替（generate がその引数の内部 release をやめる）。`can_set_borrow(i)` が false の op（`FixBody` 等）もある。read getter を最初から `Borrow` 宣言するのは P1（別物）。
```

- **分岐は `Match` のみ（`If` を持たない）**: Bool を union 化する（std.fix: `type Bool = unbox union { _false : (), _true : () }; true = _true(); false = _false();`）。ソースの `if`/`true`/`false`/比較演算子は不変で、AST→RC IR 生成で `Expr::If(c,t,e)` を `Match(c, [_false => e, _true => t])` に desugar するだけ。性能中立（Bool-union ＝ `{i8 tag, [i8;0]}` ＝ i8。比較演算子は今も i8(0/1) を返す＝tag そのものでビット不変。FFI も i8 tag で不変。match は i8 tag の compare+branch で `if` と同等）。`&&`/`||` は `if` 経由で desugar 吸収。**`not` は `if` 経由でなく専用 InlineLLVM（`BoolNegBody`＝`icmp eq x,0`・branchless）で残す**——Fix ソースは union tag を直接触れないので source 実装（`match`）だと分岐になる。so `not` は Match に脱糖しない（比較演算子も i8 tag を直接産む builtin のまま）。debug/is_boolean の Bool 特別扱いは維持（§7 P0.5・§10）。
- **tail position・末尾呼び出し・phi 回避（tail 先読み `tail_of`）**: `Ret` は Var のみ〔[#R10-3round10]〕なので tail position は IR に無く、下の 2 関数で**前向き（rename 追従）**に導出する（本体を `mark_tail(f.body, true)` で走査し、各 `Let(x, cexp, k)` の cexp が tail かを `is_tail[x]` に記録）:
```
is_tail_cont(k : RcExpr, x : Var) -> bool:   # 継続 k が x を rename（move-bind）の連鎖だけで Ret へ運ぶか
  match k {
    Ret(r)                     => r == x                          # r : Var（Ret は Var を持つ・§1.2）
    Let(y, RcRhs::Var(x'), k') => x' == x && is_tail_cont(k', y)  # x を y へ rename → 継続を追う（y : Var, k' : RcExpr）
    _                          => false                           # 実 op / Retain / Release / 別変数の Ret は非 tail
  }
mark_tail(expr : RcExpr, tail : bool) -> ():   # 副作用 is_tail : Map<Var, bool>。本体は mark_tail(f.body, true) で開始
  match expr {
    Ret(_)                                   => ()
    Retain(_, _, _, k) | Release(_, _, _, k) => mark_tail(k, tail)          # RC ノードは飛ばして継続へ（tail 文脈は不変）
    Let(x, cexp, k)                          => {
      is_tail[x] := tail && is_tail_cont(k, x)                             # cexp が tail ⟺ tail 文脈 ∧ 継続が x の rename 連鎖→Ret
      if let RcRhs::Match(_, arms) = cexp { for arm in arms: mark_tail(arm.body, is_tail[x]) }  # tail match の arm へ tail 伝播
      mark_tail(k, tail)
    }
  }
```
（名前は大域一意〔§1.1-3〕なので束縛変数 `x : Var` で `Let` を一意に指せる。`App` が末尾呼び出し・`Match` が tail match ⟺ その `Let` で `is_tail[x]`。）so `let z=Match; let n=@size(z); …; Ret(z)` は継続に実 op が挟まり**非 tail**（誤コンパイルしない）、`let r=App; let s=Var(r); Ret(s)` は rename 連鎖で**tail**（copy-prop の前後に依らず頑健＝copy-prop は純粋 cleanup。§2.1）。この `is_tail`（＝`tail_of`）を **routing（§2.1）・RC 書き換え・codegen が共有**（一貫性は §2.1 の不変条件）。**codegen 規則**: tail position の match は **merge/phi を作らず各 arm が直接 return**する（`call; ret` の隣接を保ち LLVM の tail-call elim を効かせる。複数 arm でも各 arm が独立に `tail call; ret`）。非 tail の match だけ merge ブロック（phi/slot）で結果を x に束縛し k へ。これは現 codegen の `tail` フラグ伝播（`eval_match(…,tail)`→各 arm `eval_expr(val,tail)`、generator.rs:2287,2307）と同じ挙動を、`Ret(Var(x))` の先読みで駆動する形（現 codegen は `Let` の bound を常に非 tail 評価するので、この先読みが RC IR codegen 側の追加点）。加えて、この compiler の tail-call elim は LLVM の `tail` マーカー（`apply_lambda` の `set_tail_call`。generator.rs:1010。**`-g`＝debug info 時は付けない**）にも依存するので、RC IR codegen 付け替え時もこのマーカー付与を維持する（`call; ret` 隣接**と** `tail` マーカーの両方が要る）。
- **射影に専用ノードは持たない**: フィールド/variant payload/配列要素の取り出しは getter プリミティブ＝`LLVM` で表す。解析での扱いは §3。
- **構築も getter 同様 `LLVM`（専用 `Construct` ノードを持たない）**: struct/タプル/`ArrayLit`/union variant の構築は alloc 系 `LLVM` プリミティブで表す（射影＝getter を専用ノード化しない方針の双対）。現コンパイラは struct=`MakeStruct`・array=`ArrayLit`（専用 AST ノード）／union=`union_new`・リテラル=`IntLit` 等（InlineLLVM）と混在するが、lowering で全て alloc 系 `LLVM` に寄せて統一する。解析での扱い（引数をスロットへ move。結果 `Provenance` は boxed 集約＝`Fresh`〔alloc〕、unboxed 集約〔タプル・unboxed struct・unboxed union〕＝子の由来を担ぐ〔union は構築した variant のみ〕）は §3.2/§3.3。
- **`Atom` は持たず atom 位置は `Var` のみ**: リテラル（整数/浮動小数/Bool/nullptr）は現コンパイラでも `Expr::Lit` でなく InlineLLVM（`IntLit` 等）。RC 無関係な unboxed 即値なので `LLVM` で let 束縛し、atom は `Var`（global funptr 参照を含む）に一本化（文字列リテラルは boxed `Array U8` ＝ alloc 系 `LLVM` 側）。
- **`Let` は単一 Var のみ（Pattern を持たない）**: `let (x,y)=s` 等の struct/タプル destructure は **getter プリミティブ列 ＋ `Release(container)`** に lower（役割分担: 構造分解は getter、union 分岐は `Match`）。lowering は現状の destructure codegen（`get_struct_fields`/`get_union_value`）が行う RC 最小化（move-out）を踏襲し、**最初から必要最小の retain/release だけを挿入する**。
- **`Match`（union 分岐）の RC**: 現 union-match codegen（`get_union_value` object.rs:822／`eval_match` generator.rs:2283）を踏襲する（上の Let destructure の union 版）。**boxed union** は tag 分岐後、各 arm で payload を **retain-getter で取り出し（要素 retain を op 内に持つ・別ノードの明示 `Retain` にしない・[#R10-1]）**＋scrutinee 容器を `Release`（`get_union_value` の boxed 分岐 831-834）。**unboxed union** は payload の retain と容器 release が相殺＝何も出さない（move-out。835-837 のコメント）。容器 `Release` は**各 arm 内**（tag 分岐後・payload 取り出し時）に入る——非 tail match の合流点 k は結果値の merge であって容器 release の場所ではない（各 arm が自分の payload を取り出し容器を release する）。容器が unique なら payload の move-out（[3] と同型・§6）が乗る。
- **RC IR は nested lambda を持たない**: lowering が全 lambda を top-level RC IR 関数へ lift し、使用箇所を `Closure(func, 捕捉)` に変換する（クロージャ生成を明示）。各関数の RC が閉じる。クロージャ値は unboxed の `{funptr, 捕捉オブジェクトへのポインタ}` ペアで、捕捉オブジェクトのみ boxed（rc 追跡。空捕捉は nullptr で複製しても RC 増減なし）。`FuncRef` ＝ top-level RC IR 関数への参照（名前/id。lift した lambda body。codegen で funptr に解決）。`Closure` の捕捉リスト（`Vec<Var>`）は**順序つきでノードに保持**する。順序＝捕捉オブジェクトのスロット順＝lifted 関数が cap から射影する順（`cap.@0, cap.@1, …`）。`Closure` 生成時の格納順と lifted 関数の射影順は同順でなければならず、全変換がこの対応を崩さない（捕捉の並べ替え/追加/削除は両側を揃えて行う）。
- **トップレベル定義は `RcFunc`、クロージャ値生成は `RcRhs::Closure`**: lift した lambda body・global 関数・uncurry funptr 版はすべて `RcFunc`。**クロージャは必ず arity-1**（arrow 型はカリー化される）で closure ABI の関数は `(arg, cap)` の2引数（cap が末尾、body が cap から捕捉を射影）。多引数＋捕捉は入れ子の arity-1 クロージャになる（多引数クロージャは存在しない）。**多引数は funptr のみ**（捕捉なし n 引数。uncurry が global lambda から `name#funptr{n}` を生成、`n ≤ FUNPTR_ARGS_MAX`）。`Closure(FuncRef, captures)` は実行時クロージャ値（unboxed `{funptr, 捕捉obj}`）を生成し、捕捉 obj は heap 値で `RcFunc` の一部ではない。`App` は callee 型で振り分け（funptr＝直接 n 引数、closure＝arg ＋抽出した cap）。
- **RC 効果の宣言（lowering と解析が RC を読めること）**: **AST→RC IR lowering** は各引数の `OwnershipShape`（`Own`／`Borrow`。§1.2）を見て正しく明示 RC を挿入する——`Own` かつ非 last-use なら使用前に `Retain`（last-use なら何もしない）、`Borrow` なら何もしない。codegen は出来上がった明示 `Retain`/`Release` を inc/dec へ翻訳するだけ（`Own` で外部化しない op の内部 release はその op の generate 側）。`mark_global`/`mark_threaded` は **`Own` 引数・結果 `Dyn` の InlineLLVM op**（`let y = mark_threaded(x)`＝x を消費して threaded handle y〔物理的に同一 object・provenance `Dyn`〕を返す。意味論上「送れるのは y であって x でない・x は消費済み」を Own が担保。`id` 同型だが結果 `Dyn`＋state 副作用）。x を mark 後も別に使うなら dual-use の `Retain(x)` が入り x は `Dyn`。`make_array_unique` 等の force-unique 内 clone は op の意味に内包する。**外に出すのは「最適化で消せる/動かせる & 消したい RC」だけ**（last-use/ownership の RC。相殺/reuse/borrow が効く）。**最適化で消えては困る意味的 RC は opaque な op の内部に埋めたまま**にする（例: `with_retained` の「呼び出し中 x を shared に見せる」retain。外に出すと相殺で消えて壊れる）＝構築により保護。外に出せない内部 RC は宣言で足りる。状態チェックは状態不明時の runtime `Retain`/`Release` に内包する（`mark_global` op 以外に「is-global チェック」専用ノードを持たない。静的に global/local と分かれば `RcState` を `Global`(no-op)/`Local`(チェック省略) に特殊化＝将来の state 最適化、§6）。**全 InlineLLVM の `OwnershipShape` と `Borrow` 化/外部化可否は P1 で全件監査**（`fix`/bulk array は `Borrow` 化できない候補。§8）。

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
- **明示 retain/release 挿入**（last-use 解析＝関数全体の後ろ向きパス）: (a) **`Own` 位置での** non-last-use の使用の前に `Retain`（複数回使用ぶんの参照を用意。`Borrow` 位置の read には入れない＝§1.2。初版 lowering は全 `Own` なので全 non-last-use に入り、`Borrow` 化〔§2.1〕が借用位置ぶんの Retain を除く〔[#F1] の孤児 Retain 除去〕。read-only op は現 codegen では `get_scoped_obj_noretain`＋条件 release）。(b) 変数の **last use が borrow(`Borrow`) なら直後に `Release`**（consume(`Own`)/move が last use ならそこで消費されるので `Release` 無し）。ただし**末尾呼び出しの引数は例外**——後続 `Release` が残ると tail call が壊れるので、相殺で消えないなら `Own` に留めて callee に release させる（§2.1）。(c) 未使用 let 束縛・分岐 dead 変数も `Release`（**変数が dead になる最も早い地点**に置く＝未使用なら束縛直後・分岐 dead なら arm 先頭。**tail call の後には置かない**——(b) 同様に後続 `Release` は非末尾化するので、相殺で消えないなら `Own` に留めて callee に release させる）。
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
    let arr2 = LLVM[retain_get.0](cap)  // 捕捉 arr を retain-getter で取り出し（要素 retain を op 内に持つ・配列 @ と同型・[#R10-1round10]）
    Release(cap)                        // callee が cap を所有 -> 捕捉コンテナを release（arr2 は独立所有〔op 内 retain〕なので生存）
    let r2   = App(concat_len#funptr2, [arr2, b])  // funptr 呼び出し（cap 無し）。arr2,b を consume
    ret r2

fn f(arr, brr):                    // arr, brr : Array I64 を所有
    let g = Closure(g#lifted, [arr])  // cap{arr} を alloc（rc=1）。arr を1参照 move-in
    let r = App(g, [brr])             // クロージャ呼び出し。g を consume（cap 所有権 -> callee）、brr を consume（-> b）
    ret r                             // f 側に cap の release は無い（g は App が consume、cap は callee が release）
```

ポイント:
- **cap の release は callee（`g#lifted`）**。boxed 捕捉の取り出しは **retain-getter**（要素を retain して返す・配列 `@`〔builtin.rs:1855〕と同型）＋`Release(cap)` で表す（[#R10-1round10]: 「no-retain 取り出し＋別ノードの明示 `Retain`」形は**採らない**——取り出し要素 arr2 の borrow-read last-use 後に §1.4(b) が置く `Release(arr2)` を、§2.2 が明示 `Retain(arr2)` と〔同 root・間に consume 無し〕対消滅させると、窓内の `Release(cap)` の間接デクリメント〔cap スロット→捕捉 object〕で arr2 が早期解放＝UAF になるため。retain-getter は要素 retain を op 内に持つので、相殺され得る Retain ノード自体が無い）。取り出した arr2 は独立所有（retain-getter の +1）なので `Release(cap)` が cap スロットを decrement しても生存する。**cap が unique なら move-out**——**§6 reuse フェーズの構造的 rewrite**: lowering の形（`let arr2 = retain_get.0(cap); …; Release(cap)`）から「arr2＝cap のスロット」と分かるので、cap が unique なら **retain-getter を no-retain-getter に差し替え＋`Release(cap)` を殻解放のみ（中身は arr2 へ移譲）**に書き換える → 要素 retain も殻中身 release も消え、殻解放だけ残る。cap の uniqueness は §4 の特殊化（unique-cap clone）が与えるので §6（§2 前処理は §3 前で uniqueness 未確定ゆえ不可）。現 codegen の `get_struct_fields`/`get_union_value` の move-out 最小化の一般化。空捕捉なら cap は null で `Release(cap)` は no-op。
- **呼び出し側 `f` に cap の release は無い**: クロージャ `g` を `App` が consume し所有権（cap 含む）が callee へ渡る。`g` を2回呼ぶなら使用前に `Retain(g)`（＝cap obj を retain）が入り各 callee-release と釣り合う。
- funptr（`concat_len#funptr2`）は **cap 引数なし**。例は `@size` の配列引数を `Borrow`（読むだけ）として release を呼び出し側の明示 `Release` に出した形。base の `Own`（`@size` が内部 release）なら明示 `Release(a)/(b)` は出ず op 内部にある（[#F6]: read getter を `Borrow` にするのは **P1 の宣言**〔`@size` 等の InlineLLVM を最初から `Borrow` 宣言〕であって §2.1 ではない——§2.1 の 2 版化は **source 関数の param のみ**を対象にする。InlineLLVM 引数の `set_borrow` は §6）。
- RC 収支（arr）: cap へ1参照 → `g#lifted` で +1 → `Release(cap)` で −1 → `concat_len` で `Release(a)` −1 = 0（リーク・二重解放なし）。
- 捕捉と呼び出しの両方で同じ値を使う版（`g(arr)`）なら、`arr` は2回使用 → `f` で使用前に `Retain(arr)` が1つ入り、`concat_len` は `a==b`（同一配列・rc 2＝shared）を受け取る。

## 2. RC 前処理（`Retain` を減らして uniqueness の precision を上げる）

uniqueness 解析（§3）は「`Retain` されていない boxed ＝ `Unique`」を読む。`Retain` は `Unique -> Dynamic` の一方向遷移なので、precision は「`Own` を `Borrow` に書き換えて `Retain` を落とす（borrow 化 §2.1）・残った冗長な `Retain`/`Release` を消す（相殺 §2.2）」ことに帰着する。両方を uniqueness 解析の前に行い、初版から入れる。

### 2.1 borrow 化（`Own` 引数の `Borrow` 化）
lowering が作る RC IR は**全引数 `Own`**（現 codegen が全引数 `Own` 前提で RC を書いており、そこから抽出する初版がそうなる）。ここで、可能な引数を `Own` -> `Borrow` に**書き換える**。狙いは呼び出し側の `Retain` を落として `Unique` を保つこと（§3）。`InlineLLVM` の `OwnershipShape` は宣言済み（§1.2 `arg_ownership`）で、書き換え対象は source 関数の引数。

**どこまで `Borrow` にできるか**: 引数（の boxed 末端）`p` が body 内で**読むだけ**（escape/consume されない）なら `Borrow` にできる。**消費** ＝ callee の `Own` 位置へ渡す（`App`/`LLVM` の該当引数。`MakeStruct` 等の構築 alloc も引数 `Own` 宣言なのでここに含まれる。`Closure` 捕捉も move-in で同様）／**`App` の callee（closure 値）自身**（[#2round9]: クロージャを呼ぶ＝それを consume・cap は callee が release・§1.7。so「呼ぶだけ」の closure param も Own）／`Ret(p)`（return で escape）。**消費でない** ＝ `Borrow` 位置での read（getter・比較・size）・`Match(p)` の tag 読み・未使用（drop するだけ）で、**「読むだけ・捨てるだけ」は `Borrow` 可**（own-then-release ＝ borrow）。move-bind `Let(y, Var(p))` は透過（`p` の消費 ＝ `y` の消費）。unboxed 集約の子取り出しは子変数を辿って親引数の末端へ帰着。callee が `Own` かは相手の現在の `OwnershipShape` に依存するので、**コールグラフ上の最大不動点**で決める（初期は全 `Borrow`、消費を見つけた末端を `Own` に降格、`Borrow -> Own` の一方向で有限停止。間接呼び出しは全 `Own`＝保守的）。解く順序は**コールグラフを SCC 分解し condensation を bottom-up（callee 先）に処理、SCC 内は不動点**（再帰の自己参照はこの不動点が捌く）。結果は各引数の `OwnershipShape`（§1.2）。

**2 版モデル（`f_own`/`f_borrow`）＋呼び出し地点ごとの routing（[#2]/[F2]）**: borrow 化は呼び出し規約（`Own`＝callee が release／`Borrow`＝呼び出し側が release）を変えるが、**呼び出し地点ごとに要求が違う**（間接呼びは all-Own ABI が要る・所有値の tail 呼びは Own でないと非末尾化・所有非 last-use の非 tail 呼びは Borrow が retain を減らす）。単一の所有権に潰さず、各 source 関数 f を概念的に **2 版**持つ:
- **`f_own`**: 全 param `Own`（＝baseline lowering・callee が param を release）。**param-RC は baseline のまま**だが、**body 内の呼び出し地点は routing の対象**（[#F2]）——`f_own` の body も所有非 last-use 値を `g_borrow` へ routing して後続 `Release` を要する。param 無しの関数（`main` 等）は借用可能 param を持たないので**単一版＝`f_own`** だが、その body の呼び出しはやはり routing・RC 書き換えを受ける（`main` の `sum` 呼びが `sum_borrow` へ行くのはこれ）。
- **`f_borrow`**: 借用可能 param を `Borrow`（上の不動点の `OwnershipShape`・caller が借用値を release）。param-RC の書き換え（借用 param の内部 Release・孤児 Retain 削除）はこちらのみ。呼び出し地点の retain/release は全 body（下記 step 3-(ii)）。

**呼び出し地点ごとに routing でどちらを呼ぶか決める**（引数ごとの所有/借用・last-use を見て call 単位で 1 版に合流）:
- **間接呼び出し**（`Closure` 値/funptr への `App`・callee 不明）→ **`f_own`**（固定 all-Own ABI。間接呼びされ得る全関数が all-Own で合意＝リーク/under-release を防ぐ）。
- **直接呼び出し**→ **`f_borrow` が「得あり」かつ「安全」なら `f_borrow`・そうでなければ `f_own`**:
  - **得あり** ＝ ある引数が「**所有かつ非 last-use、かつ `g_borrow` のその位置が Borrow**」（`Own` なら後で使うため呼ぶ前に `Retain` が要るのを borrow が消す。[#1round9]: `f_borrow` は**混在 shape**——消費される param は `g_borrow` でも Own のまま——なので、Own 位置に渡しても `g_borrow` が baseline どおり consume し得は無い）、または「**借用値の受け渡し**」（Borrow→Borrow で借用連鎖を伸ばす）。**所有かつ last-use** の引数は得が無い（`Own` でも move で retain 不要）ので `f_own` を選好（callee が早期解放＝#4 のメモリ遅延を避ける）。
  - **安全** ＝ **非 tail 呼び、または所有引数を渡さない**。`f_borrow` は所有引数を `Borrow` にする → 呼び出し**後**に `Release` が要る → tail だと非末尾化。so **tail ＋所有引数のときは `f_own`** に落とす（[#2]: tail を一切壊さない）。tail は必ず last-use なので、所有引数の tail 呼びは「所有 last-use」＝そもそも得が無く `f_own` が自然。
  - **多引数の合流**: `f(x,y)` は x・y 両方を見て、**どれか 1 引数でも「得あり」かつ call 全体が「安全」なら `f_borrow`・そうでなければ `f_own`**。混在 call（所有非 last-use の x ＋所有 last-use の y）は得のある x を優先して `f_borrow` へ寄せ、y は借用され解放が呼び出し後にずれる（＝残り #4・per-param 精密化〔§6〕の担当）。
- **dead-func（§4.1）が routing で到達しない版を刈る**（片使いの関数は 1 版に落ちる・両使いのみ 2 版残る。escape-reachability の到達根は §4.1 dead-func と同じ〔`Closure` 参照・funptr atom〕）。

この routing 1 本が **[#2] の tail 保護**（tail＋所有→`f_own`）・**間接呼び出し規約**（間接→`f_own`）・**#4 のメモリ遅延の大部分**（所有 last-use→`f_own` で早期解放）を包含する。**case B・大域不動点・escape-clone の別立ては不要**（それらを置き換える）。**per-param 精密化**（引数ごとに `Own`/`Borrow` を選ぶ最大 2^k 版で、混在 call の残り #4 も消す）は将来（§6）。

**書き換え（`f_borrow` 側）**: 借用 param `p` の内部 `Release` を落とす（もう所有しない）。呼び出し側は、**`g_borrow` へ routing され f 所有の値 `x` を渡す【各】呼び出しの直後に `Release(x)` を挿入**する（callee が release しなくなったぶんを一律に引き受ける・[#F6]: last-use のときだけ入れるのでなく該当する全呼び出しで入れる。さもないと同じ `x` を借用呼びに複数回渡すと under-release）。`x` を**後でも使う**場合は、この `Release(x)` と lowering が非 last-use 用に入れた呼び出し**前**の `Retain(x)` が、pure-borrow の呼び出しをまたぐ **net-zero 対**になり §2.2 が消す → `x` は `Unique` のまま後続（`set` 等）へ届く。`x` の**last-use**が該当呼び出しなら手前に `Retain` が無く、挿入した `Release(x)` が `x` を drop する（相殺しない）。**[#R10-6]: 挿入は「呼び出しごと」でなく「渡した Borrow 位置×末端ごと」**（擬似コード step 3-(ii) は `enumerate(args)` で位置×末端を回る）——同一呼びの複数 Borrow 位置に同じ所有値を渡す `g_borrow(x,x)`（call が x の last use）は lowering Retain 1 個＋base 参照 1 個＝2 参照に対し **Release が 2 本**要る（per-call で 1 本だと under-release＝リーク）。

**末尾呼び出しは routing が守る（非末尾化しない・[#2]）**: 「呼び出し**後**の `Release`」が**末尾位置（tail position。§1.2 の tail 先読み `tail_of`——`let r=App(…); Ret(Var(r))` の形〔rename 追従〕。match arm 内の呼び出しは、その match が tail のときだけ tail）**の呼び出しに残ると tail call でなくなる（`let r = App(f, args); Ret(Var(r))` が `let r = App(f, args); Release(x); Ret(Var(r))`）。上の routing で「**所有引数の tail 呼び → `f_own`**」（安全条件）なので、tail に後続 `Release` が入らず**一切非末尾化しない**（閉路判定に依存せず自明に baseline と同等。間接サイクルの穴〔[#2]〕も「間接→`f_own`」「所有 tail→`f_own`」で塞がる）。tail での**借用値受け渡し**（`sum`／素通しラッパ `sum2 = |a,i,c| sum(a,i,c)` の再帰）は `f_borrow`→`f_borrow` の Borrow→Borrow で、caller が所有しないので後続 `Release` が出ず tail 保持——借用は何段ラッパを挟んでも伝播する。同じ所有値が tail の**複数位置に出る**（`g(x,x)`）ときも、call 全体が「所有引数あり＋tail」なので `f_own` へ（全出現 consume＝tail 保持）。**混在 call（借用値＋所有値）を `g_own` へ渡すとき、借用値ぶんは呼び出し前に `Retain` を挿す**（[#F1round8]・step 3-(ii-c)）——`g_own` は全 Own 版なので借用値をそのまま release すると caller が所有しない object を解放して二重解放/UAF になる。呼び出し前 `Retain` で `g_own` に**本物の所有 +1** を渡せば、`g_own` がそれを **release でも戻り値に返す（escape）でも複製でも一様に正しく扱う**（all-Own baseline の定義）——caller は `g_own` の中身を区別しなくてよい。`Retain` は呼び出し**前**なので tail は保たれる。**実測**: 現コンパイラは間接 tail サイクルを release ビルドで TCO し O(1) で回す（generator.rs:1008-1010 が全 `App` を tail マーク）ので、非末尾化は退行——routing がこれを防ぐ。

**パス順**: lowering で全 `Own` の RC IR（§1.4）→ **末端分解の正規化**（whole-value `Retain(x,[])`/`Release(x,[])` を per-末端に分解・unboxed union で止める〔§2.2 の分解規則・[#F1]〕。[#F7]: §2.1 の RC 書き換えも §2.2 相殺も per-末端で動くので、両者の**前**に 1 回だけ走らせる名前付きステップ）→ borrow 化（`f_borrow` の `OwnershipShape` を不動点で確定 → routing で各呼び出し地点の版を決定 → `f_borrow` の per-末端 RC 書き換え）→ §2.2 相殺 → dead-func で未使用版を刈る → uniqueness 解析（§3 は materialize 済み callee＝版ごとの `FuncRef` 単位で解析する。[#F4]: §3 が読むのは結果の由来 `Provenance`〔§1.2〕であって `OwnershipShape` ではない——版が FuncRef で分かれているので版ごとに正しく解析される）。borrow 化は uniqueness と独立（消費の構造だけで決まり、引数が unique かに依らない）。globals（`RcGlobalInit` の body）も param 無し関数として同じ処理を受ける（[#F8]）。

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
`sum_borrow` の借用可能性を不動点で決める（SCC {`sum`}・解析の再帰）:
- 初期化: `sum.arr = Borrow`。
- `sum` body を走査、`arr` は消費されるか?
  - `@size(arr)`・`@(i, arr)` は `Borrow`（宣言）-> 消費でない。
  - `App(sum, [arr, …])` の位置0は `sum.arr` の現在値 `Borrow` -> 消費でない（再帰の自己参照はここ＝解析の再帰）。
  - `_true` の `Release(arr)` は drop（own-then-release ＝ borrow と両立）-> 消費でない。
- どこでも消費されない -> `sum_borrow.arr = Borrow` で収束。

routing と書き換え（`sum_borrow`）:
- **routing**: `main` の `sum(arr,0,0)` は `arr` を**所有かつ非 last-use**（後で `set` に再利用）→ 得あり・非 tail で安全 → **`sum_borrow` へ**。`sum` の再帰 `sum(arr,i+1,…)` は `arr` を**借用**して渡す → 借用値の受け渡し → **`sum_borrow` へ**（Borrow→Borrow・tail 安全）。∴ 全て `sum_borrow`（`sum_own` は未使用で dead-func が刈る）。
- (a) `sum_borrow` 内部の `Release(arr)`（`_true` 枝）を落とす（もう所有しない）。
- (b) 呼び出し地点で release を引き受ける: `main` は `arr` を所有するので `sum_borrow(arr,…)` の直後に `Release(arr)`。再帰は `arr` を借用（所有しない）ので**足さない**（借用を借用へ渡すだけ＝末尾保持）。

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

**具体例（fresh を tail へ渡す再帰・routing で `f_own`）**: `buf` を読むだけ（consume しない）だが、再帰は毎回 fresh な `buf` を tail に渡す `loop_fresh`。
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
不動点（`loop_fresh_borrow` の借用可能性・`own[loop_fresh.1@[]]`＝`buf` 末端、初期 `Borrow`）:
- `consume_sites`: `@size` は `Borrow`・`Release(buf)` は drop・再帰位置1 の `fr` は `own[loop_fresh.1@[]]=Borrow` ゆえ非消費 ⇒ ∅ ⇒ **`loop_fresh_borrow.buf = Borrow`**（buf は読むだけ）。
- **routing**: 再帰 `App(loop_fresh, [n1, fr])` は **tail** で `fr` は fresh＝**所有 last-use**。safe 条件（tail＋所有引数）より **`loop_fresh_own` へ routing**（得も無い＝所有 last-use）。so 再帰は `loop_fresh_own` を回す。`main` の初回 `loop_fresh(3, fill(1,0))` も所有値の tail 呼び → `loop_fresh_own`。∴ **`loop_fresh_borrow` は未使用で dead-func が刈る＝実質全 `loop_fresh_own`**。

**`f_own` へ routing する理由（`f_borrow` だと tail が壊れる）**: 仮に再帰を `loop_fresh_borrow`（`buf` Borrow）へ回すと、その版は内部 `Release(buf)` を落とし、`fr`（fresh・この frame 所有）を借用位置へ渡すので呼び出し**後**に `Release(fr)` が要る ⇒ `…; App(loop_fresh,[n1,fr]); Release(fr); Ret(r)` ＝非 tail ＝閉路で深さ分スタック → overflow。routing が「所有値の tail 呼び → `f_own`」に落とすので `fr` は callee が consume ⇒ 後続 `Release` 無し ⇒ **tail 保持**（結果は baseline のまま・RC 不変）。fresh を受ける `buf` は借用しても得が無い（uniqueness を運ばない）ので loss も無い。

**手順（擬似コード）**: SCC は **f_borrow の借用可能性不動点の「解析の再帰」用**（再帰関数の借用可能性が自己参照的なので反復が要る。上向き依存のみで bottom-up・大域反復不要）——**tail 保護は routing** が行い SCC/閉路判定には依存しない（[#2]）。所有権は **RC 単位**（[#R10-2]: boxed 末端 ＋ **unboxed-union 部分木の根**＝§2.2 分解と同じ単位〔[#F1]〕。RC ノードは union 根で止まり `k::π` で variant 内へ入らないので、`own[]`・step (i)/(ii)・routing も同じ単位で動かす）で、`own` は単位をキーに `Own|Borrow` を引く（`own[g.q@π]`＝g_borrow の param q の単位 π。f_own は全 Own）。**union unit 内のどれか 1 leaf の consume は unit 全体を `Own` に降格**（不動点の demote を unit にクランプ——借用したコンテナに payload の穴を開けられない＝混在 ownership は不成立）。step (ii) の `Retain`/`Release` 挿入も unit 根パス（tag 分岐 traverser）で行う。値 `x` を位置 q へ渡すとき各 RC 単位 `x@π` は callee param q の単位 π に対応する。値の末端は、別名辺（move-bind と unboxed 集約/union の子取り出し＝getter LLVM・Match payload）を後ろ向きに辿った**定義位置 `root`**（producer。= object 同一性）で識別する（散文の「親引数の末端へ帰着」）。`owns`（所有判定）・借用可能性不動点の消費降格・§2.2 相殺は同じ `root` を共有する。g が未知＝間接呼び出しの位置は `Own` 固定（散文の「間接呼び出しは全 Own」）。
```
borrow_ify(prog):
  # 1. f_borrow の借用可能性を不動点で決める（bottom-up SCC・上向き依存のみ）
  #    依存は「callee の own → caller param の借用可能性」の【上向き】だけ（case B の下向き力は無いので
  #    大域反復も不要）。再帰関数の借用可能性は自己参照的（f.p の判定が f.p の own に依存）なので、その
  #    「解析の再帰」を SCC 内不動点が捌く（tail 保護とは別物・散文 §2.1）。
  own = { source 関数の全 param の全 boxed 末端: 初期値 Borrow }   # f_borrow の OwnershipShape。楽観初期化
  for scc in bottom_up(condensation(direct_call_graph(prog))):     # 直接呼び辺の callee 先
    repeat SCC 内で変化が無くなるまで:                              # 自己/相互再帰の自己参照
      for f in scc, for c@π' in consume_sites(f):                   # 消費された末端を source param へ帰属して降格
        if clamp_unit(root(f, c@π')) が (param p, π0): own[p@π0] = Own   # RC 単位にクランプ（[#R10-2]: payload consume は k::π を union unit 根に truncate＝unit 全体を Own に）。間接 App 位置は all-Own ABI ゆえ consume に含む
  # ※ case B は無い。tail 保護は下の routing（[#2]＝B 案を 2 版 routing で実現）。

  # 2. routing: 各呼び出し地点で g_own / g_borrow を選び、【App の callee を選んだ版の FuncRef に書き換える】
  #    （[#F3]・side table のままにしない）。Closure/funptr 参照は f_own に向ける。以降 §2.2/§3/§4/§9.0 validator は
  #    callee の版-正確な OwnershipShape を見る。version v ∈ {f_own, f_borrow} の body ごとに解く（f_own の body は
  #    param が Own＝所有、f_borrow の body は param が Borrow＝借用、で owns が変わる）。間接 App/Closure は常に g_own。
  for v ∈ {f_own, f_borrow (存在すれば)}, for v.body 中の直接 App(g, args):
    benefit = ∃ (q, x@π) ∈ args: (owns(v, x@π) ∧ ¬last_use(x@π) ∧ own[g_borrow.q@π]==Borrow)  # 所有非last-use を g_borrow の【Borrow 位置】へ＝Retain を消せる（得）。Own 位置へでは g_borrow も consume＝得なし（[#1round9]）
                             ∨ (¬owns(v, x@π))                        # 借用値の受け渡し（Borrow→Borrow・借用連鎖。[#R10-4]: 借用値が届く g_borrow 位置は不動点により必ず Borrow〔Own 位置へ届くなら consume で param が Own 降格済み＝借用値が生じない〕ので Borrow 位置条件は含意・明示不要）
    safe    = (この App が非 tail) ∨ (¬∃ owns(v, x@π))              # 所有引数を tail で Borrow に渡すと非末尾化
    g_ver   = if benefit ∧ safe then g_borrow else g_own           # 間接/Closure は g_own 固定
    App の callee を g_ver（の FuncRef）に書き換える                # [#F3] materialize

  # 3. RC 書き換え。(i) param-RC の remove は f_borrow のみ。(ii) 呼び出し地点の retain/release は【全 body】（[#F2]）
  #    順序: (i) → (ii) は必須（[#5round9]）。(ii) が挿す「借用値→Own位置」の Retain は root が Borrow param 末端なので、
  #    (i) を後や融合で走らせるとその Retain を消して under-retain＝[#F1round8] 再発。(i) は lowering 由来の Retain のみ削る。
  # (i) f_borrow の param-RC remove（f_own は baseline のまま触らない）
  for f_borrow in prog:
    remove: root(f_borrow, y@π) が own==Borrow の param 末端に解決される全 `Release(y@π)`・`Retain(y@π)` を削除（どちらも per-末端・root ベース）   # callee は所有しない。root ベース([6])：p 直接名指しだけでなく別名の Release も拾う——unboxed union payload drop の Release(payload)(root=p@(k::π))・passthrough drop（let (b,_)=p.unsafe_is_unique の @1 の Release, root=p）。変数名マッチだと残って過剰解放→caller UAF
    # Retain も消す理由（消さないと孤児 Retain がリーク・[#F1round6]）: lowering は非 last-use の【Own 消費】用に Retain を入れる。
    # param p の末端が Borrow に確定した ⟺ 不動点が p@π0 に Own 消費 sink を 1 つも見なかった ⟺ その末端の
    # 全使用位置は Borrow/read/drop、【または routing が (ii) で「借用値→Own位置→呼び出し前 Retain」を挿す Own 位置（g_own への混在 tail 等）】
    # （[#F1round8] 訂正: 後者は真の consume でなく、その Retain が供給する +1 が消費される＝p 本体は消費されないので p は Borrow のまま正しい）。
    # ゆえに p 向けの lowering Retain(p@π0) は宛先を失った stale な +1（対応 Release を持たない孤児で §2.2 も消せない・呼び出しごとに漏れる）。
    # remove がここで消して net-0（例 `|arr| sum(arr)+sum(arr)`）。per-末端なので多末端値でも Borrow 末端の Retain だけ消える。f_own は対象外。
  # (ii) 呼び出し地点の retain/release（全 body v・step 2 で App の callee は選ばれた版に materialize 済み）。
  #      materialize 済み callee の【位置 q の shape】× 引数の所有/借用 の 2x2（[#1round9]）。
  #      条件は own[callee.q@π] で見る——g_own は全 Own なので own[callee.q]==Borrow は自動的に g_borrow の Borrow 位置
  #      ＝別途の「g_ver==g_borrow」判定は不要（routed 版の位置 shape が全てを決める）。
  for v ∈ {f_own, f_borrow}, for v.body 中の直接 App(callee, args):    # callee = step 2 が選んだ版（g_own or g_borrow）
    for (q, x) in enumerate(args), x の各 boxed 末端 x@π:            # q=位置
      if own[callee.q@π]==Borrow and owns(v, x@π):                    # 所有値 → Borrow 位置: 呼び出し【後】Release（caller が引き受け）
        呼び出し直後に Release(x@π) を挿入                            # safe 条件より必ず非 tail（tail+所有→g_own の全 Own 位置へ）＝非末尾化しない
      if own[callee.q@π]==Own and ¬owns(v, x@π):                      # 借用値 → Own 位置: 呼び出し【前】Retain（[#F1round8] 混在 tail の g_own 等）
        呼び出し前に Retain(x@π) を挿入                               # callee に本物の所有 +1 を渡す。callee は release でも return(escape) でも複製でも一様に扱う（all-Own baseline）。Retain は【前】ゆえ tail 保持
      # 残り 2 セル（Own位置+所有＝baseline move / Borrow位置+借用＝Borrow→Borrow）は何もしない
      # [#3round9] Release の挿入位置: 非 tail の App は必ず `Let(r,App,k)`（Ret は Var のみ〔§1.2〕）なので「呼び出し直後＝k の先頭」に置ける。
      #   非 tail match の arm 末尾の App も `Let(t, App, Ret(Var(t)))` の形（arm 値 = t）なので、その arm 内で
      #   `Let(t, App, Release(x@π); Ret(Var(t)))` と arm の `Ret` 直前に挿す。合流後の k（`Ret(Var(z))`）に置くと、
      #   Retain を挿さない他 arm 経路（その arm では x を借用呼びに渡していない）で over-release＝UAF。

  # 4. §2.2 相殺が「Retain … (借用呼び出し) … Release」の net-zero を消す（全 body）

root(f, x@π):    # x@π の別名鎖を辿った定義位置（producer）= object 同一性。owns/phase(i)/相殺(§2.2) が共有
                 # 別名辺 = move-bind / LLVM 射影(LeafSource が単一 Arg) / Match payload(unboxed union)。producer で停止
                 # RC 単位キー = clamp_unit(root(...))＝root 結果の path から unboxed-union 変内 `k::π` を union 根で truncate（§2.2 相殺・不動点 demote が共有・[#R12-1]/[#R10-2]）
  x が f の param                      -> (param x, π)             # producer: param 末端
  Let(x, Var(y))                      -> root(f, y@π)              # 別名辺: move-bind
  Let(x, LLVM(op, args)):             # op の結果 Provenance(§3.3) の末端 π の LeafSource s で分岐（射影かどうかでなく s で決める）
     s == {Arg(j, p)}（単一）          -> root(f, args[j]@p)         # 別名辺: 結果末端が単一 Arg（純粋別名）＝射影/unboxed 構築/恒等 op
     それ以外                          -> (この Let, π)              # producer: 単一 Arg 以外の全 op（Fresh=新規 alloc / Dyn=boxed 容器 getter・global / 複数 join=条件付き素通し）
  Match payload of s（s が unboxed union の variant k）-> root(f, s@(k::π))       # 別名辺: payload 取り出し
  Match payload of s（s が boxed union）      -> (この payload 束縛, π)             # producer: boxed union getter
  Let(o, LLVM(union_new_k, [payload])) の unit 根 o@[]  -> root(f, payload@[])    # 別名辺: unboxed union 単一 variant 構築の unit 中身＝構築した variant の payload（RC 単位の帰着先。merge 産 union は下の Match 規則で producer）
  Let(x, Match(...))                  -> (この Let, π)                            # producer: 分岐 merge（arm 値の join）。param 消費は各 arm の Ret〔consume_sites〕が捕捉するので merge 後は producer で停止
  Let(x, App 結果 | Closure)           -> (この Let, π)                            # producer: 呼び出し結果/クロージャ

owns(f, x@π):      # f が末端 x@π を所有するか（借用でなく）。root の分類（param なら own、他 producer なら f 所有）
  root(f, x@π) が (param p, π0) -> own[p@π0]==Own    # root が param -> その末端の own
  それ以外の producer            -> True             # alloc/getter/call/closure = f 所有

consume_sites(f):  # 所有権が消費される（escape or release）boxed 末端の集合（別名辺で結果へ抜けない Own 位置 = sink）。本体の全ノードを歩く（値 match の arm 内も）
  App(g /*callee*/, [..x@位置 i..])   -> g の全 boxed 末端（callee closure の cap）           # [#2round9]: クロージャを呼ぶ＝それを consume（callee が cap を release・§1.7・generator.rs:1980-1984）。funptr/空捕捉は boxed 末端無し＝no-op。「呼ぶだけ」の closure param が Borrow 誤分類→cap 二重解放を防ぐ
                             ∪ {x@π | own[g.i@π]==Own}                          # 呼び出し境界の引数（未知 g は Own）
  LLVM(op, [..x@位置 i..]) -> {x@π | 結果のどの末端の root も x@π でない ∧ arg_ownership(i)@π==Own}   # 「結果へ抜ける＝除外」は結果末端の root が x@π（単一 Arg 純粋別名辺）のときだけ。条件付き素通し（boxed union mod の {Fresh,Arg}＝複数 join）は producer 扱い＝consume（借用のまま残すと match 枝の release で二重解放）
  Closure(_, [..x..])      -> x の全 boxed 末端                                   # capture = move-in
  Ret(x)                   -> x の全 boxed 末端                                   # **すべての Ret（値 match の arm 末尾 Ret(a) を含む）**で consume。arm 値の所有権は match 結果へ move し結果が escape/release されるため（root で param に帰着すればその param が Own＝borrow 化されない）。Ret は Var のみ〔§1.2〕
```

**tail 認識の一貫性（不変条件）**: routing の safe 判定（所有引数の tail 呼び → `f_own`）・RC 書き換え（借用呼びの後に `Release` 挿入）・codegen（phi 回避）は **§1.2 の同じ tail 先読みヘルパ `tail_of`（rename 追従）**を使う（`let r=App(…); Ret(Var(r))`・tail match arm 内の App を含む）。**注意（model B の footgun）**: tail-ness は node 型に現れず（tail match も `Let(z,Match,Ret(Var(z)))` で普通の値 match と同形）`tail_of` でしか判らないので、tail を気にする全 pass が `tail_of` を必ず参照すること。routing が tail を漏らす（所有値の tail 呼びを `f_borrow` に回す）と、書き換えが借用末端に後続 `Release` を挿入して**非末尾化 → overflow**する（例: `if` 脱糖で arm 内 tail になる `loop_fresh` の再帰。§2.1 の worked example は arm 内 tail に適用済み）。「呼び出しの直後に `Ret` があるか」という rename 非追従の flat 判定は不可（`let r=App; let s=Var(r); Ret(Var(s))` を取りこぼす）。

**抜けない末尾再帰ループ（`main`/event/server ループ）も同じ tail-call 保存で扱える**: 静的解析は有限束の不動点で必ず停止する（実行時に停止するかに非依存）。borrow 化の routing（[#2]＝所有値の tail 呼びを `f_own` に落とす）が tail に後続 `Release` を出さないので、実行時にスタックが伸びず overflow しない——抜けないループはむしろ tail-call 保存が「有限スタックで回り続ける」ための核心ケース。非末尾の無限再帰は RC に関係なく実行時 overflow するので対象外（末尾ループのみ）。テストは §9.6。

**同一値の複製（多重出現）**: 同じ値が複数箇所に出る形——`g(x,x)`（引数位置）・`let y=(x,x); g(y)`（unboxed 集約末端）・`[x,x]`——は、**末端単位＋`root`** で同一 object に帰着させて一様に扱う（`root(y@[0])=root(y@[1])=x`）。複製ぶんの参照は lowering の `Retain`（値が複数回使われるので入る）が供給し、その値は `Dynamic`（§3 の `|x| let y=[x]; (y,y) -> (Dyn,Dyn)` テストと整合）。この `Retain` は、複製末端が escape / `Own` / `Ret` されるとそこが `consume_sites` になり §2.2 から守られる。read して drop するだけなら §2.2 が `Retain` ＋一方の末端の `Release` を net-zero で消すが、+1/-1 の対消滅なので早期解放を生まず健全（最後の `Release` で rc0）。tail の複製は上記 (B)（f 所有末端を全 `Own`）で捌く。**この一様扱いは要検証**（レビュー周で `(x,x)`・`[x,x]`・入れ子・tail の集約複製を stress-test する）。**併せて条件付き素通し op**（boxed `union mod` の結果 `{Fresh, Arg}` 等・複数 join）が `root`=producer で consume_sites に正しく入り、流入 param が `Borrow` のまま残らない（match 枝の release で二重解放しない）ことも stress-test する。

### 2.2 retain/release 相殺
borrow 化（§2.1）が余らせた「呼び出しをまたぐ `Retain`/`Release`」を net-zero として消し、`Unique` を後段（§3）へ届ける（`Unique -> Dynamic` を起こすのは `Retain` だけ）。clone 削減にも効く。健全性とは無関係（消しても付けても健全——RC の総量・メモリ安全は保つ。boxed コンテナ取り出しを retain-getter で lower する〔下記 [#R10-1]〕ので、間接デクリメントで生きる load-bearing な Retain が RC IR に現れず、無条件に成り立つ）・順序自由で、borrow 化と並ぶ precision の 2 本柱。**注（rc 観測 op）**: `unsafe_is_unique` のような **rc を観測する op** は、§2 が spurious な `Retain` を消すと**真の一意性**（spurious 除去後の rc）を観測し結果が変わり得る（false→true）。これは**健全**（消せる `Retain` は借用のみで永続参照を作らない＝値は本当に unique・runtime 分岐は正しい・unsafe/impure の契約内）だが、std/ユーザは特定の is_unique 値に依存しないこと（将来 §6 で静的 unique を is_unique に反映し分岐を畳む）。**test 影響**: `unsafe_is_unique` が used-later で false を返す既存テスト（test_basic.rs:2490-2493 の `assert_eq(unique,false)`）は、P2 で spurious `Retain` 除去により **true に反転し得る＝更新対象**（最適化前の spurious 値を固定しているため。P2 の作業項目）。

**正規化＝末端ごとに分解**（[#F7]・lowering 直後の名前付きステップ。§2.1 の per-末端 RC 書き換えも下の相殺も分解済みを前提にするので、両者の前に 1 回だけ走らせる）: `Retain(x)`/`Release(x)` を **boxed 末端ごと**に分解する。**分解は tuple/struct 境界へは再帰する**（子が同時に全存在＝割って安全）が、**unboxed union 部分木の根で止める**（[#F1]・boxed 末端でも止める＝1 単位）。variant は排他で tag=k のときしか payload が存在しないので union は 1 単位として扱い、codegen は**既存の tag 分岐 traverser**（active variant のみ retain/release）＝「機械語は whole-value 辿りと同じ」を本当に成立させる。分解で辿り着く boxed 末端は union の外だけ＝常在＝無条件 dup で安全。これで相殺は**単位ごとに一様**になり、「whole を部分的に消して残り単位へ縮約」する手間が消える（`Retain(x)`＋`Release(x.f0)` は分解後 `Retain(x@f0)` と `Release(x@f0)` の対消滅に落ち、`Retain(x@f1)` が残るだけ）。分解後の `Retain(x@π)`/`Release(x@π)` は §1.2 の `Path` フィールドで単位 π を名指す first-class ノード〔`Retain(x, π, …)`〕。`x@π` はその略記。
- **失う相殺（[#F1]・正確な損失・許容）**: union を 1 単位で扱うため、`Retain(o, [])`（キー `o@[]`）と **match arm 内の payload release** `Release(arr)`（root＝`o@some::[]`・キー `o@some::[]`）の **cross-level 相殺は行われない**。具体的には「`o : Option (Array I64)` を borrow 関数に通してから同じ `o` を match し、boxed payload を read して捨てる」形——`let pre=g(o); match o { some(arr)=>…arr.get_size…, none()=>… }`（`g` borrow 化）——で、union をまたぐ分解なら `Retain(o, some::[])` が `Release(arr)` と対消滅できたが、union で止めると **some 経路に payload の refcount 操作 2 個（+1/−1）が残る**（net-zero・健全・ピーク不変・clone は起きない）。効くのは「union を borrow 使用 → 再 match → boxed payload を read-drop」の形のみ（丸ごと release の古典的相殺 `Retain(o,[])…Release(o,[])` は両方 `o@[]` で**修正後も対消滅**・payload を返す arm は Release 無しで無関係・match 1 回だけの通常形は Retain 自体無し）。頻度は中〜低・コストは 2 RC ops/exec と小。**回収は将来**: (b) tag ガード付き RC ノード〔F1 の tag ガード責務を全ノードに戻すので v1 では採らない〕、または限定拡張〔`Retain(o,[])` が variant arm を支配し arm 内 payload release と間に consume 無しなら、none 経路 no-op・some 経路相殺として除去〕——ベンチで union-payload RC がホットに出たら足す。v1 は tag 安全性を優先して union で止める。
  - **[#R12-1] これは相殺の“損失”（benign）であって、`consume` を跨ぐ**誤相殺**とは別問題**: whole-union `Retain(o,[])`（キー `o@[]`）を挟む **payload consume**（arm `Ret(a)` の move-out・root `o@some::[]`）が、キー不一致で pend[`o@[]`] に見えないと、後続 `Release(o,[])` が **load-bearing な Retain を誤相殺 → UAF**（[#F1] が「損失」と述べる方向の逆）。§2.2/不動点は全キーを `clamp_unit`（root の `k::π` 尾部を union 根で truncate）で RC 単位に揃えるので、payload consume が `o@[]` に落ちて whole-union Retain を正しく `needed` にする。§9.5 に回帰テスト。

**照合＝object identity（`root`）**: `Retain(x@π)` と `Release(y@π')` は `root`（§2.1）が**同一 object（定義位置）**を返すとき対消滅候補。`root` が別名辺（move-bind・射影・Match payload）を辿って正準化するので、move-bind rename（`Release(arr2)`, `root(arr2)=arr@π`）も部分 field release（`Release(g)`, `root(g)=x@f0`）も同じ照合に落ちる。copy-prop で move-bind を先に畳めば照合は同名で済む。

**消せる条件と、その理由**: その object 末端の**消費使用**（`consume_sites`：Own 位置 / `Ret` / `Closure` 捕捉。§2.1）が `Retain` から対応 `Release` までに無いこと。借用（getter・比較・`Match` tag・`Borrow` 位置）は追加参照を要らない。**なぜ単純な +1/-1 除去でないか**: 間に消費使用が挟まると、`Retain` の +1 はその消費が奪い、`Release` の -1 は別の参照を落とす——往復でないので消すと use-after-free。
- **[#R10-1round10] boxed コンテナ取り出しは retain-getter で lower する（相殺の穴を構造的に無くす）**: もし取り出しを「**no-retain 取り出し＋別ノードの明示 `Retain`(要素)＋`Release`(コンテナ)**」で lower すると、要素の last use が borrow-read のとき §1.4(b) が置く `Release(要素)` を、§2.2 が明示 `Retain(要素)` と〔同 root・間に consume 無し〕対消滅させ、窓内の **`Release(コンテナ)` の間接デクリメント**〔コンテナのスロット→中身 object〕で要素を早期解放＝**UAF**（例: `|i| c.@(i)` の `Retain(c2); Release(cap); read; Release(c2)` で両者消すと `Release(cap)` が `c` を解放）。`consume_sites` は間接デクリメントを拾えない。**対処（[#R10-1]・選択肢 (a)）: boxed コンテナ取り出しは retain-getter〔要素 retain を op 内に持つ・配列 `@` と同型〕で lower し、相殺され得る明示 `Retain` ノードを RC IR に作らない**（cap 取り出し〔§1.7〕・boxed union match arm〔§1.2〕・boxed struct destructure すべて）。so 上の「消しても付けても健全」が**無条件に成り立つ**（load-bearing な Retain がそもそも現れない）。move-out（unique コンテナで要素 retain を省く）は §6 が retain-getter を no-retain-getter に差し替える構造的 rewrite で実現〔§1.7〕。§9.5 に回帰テスト（read-only capture closure／owned boxed-union payload の read-drop を valgrind/ASan——retain-getter 形で UAF が出ないこと）。
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
  前向き走査（pend[o] = いま生きている未対応 Retain の集合。分岐 `Match` は枝ごと pend コピー、合流は下記）:
    # 全キーは RC 単位に正規化する: clamp_unit(root(...)) = root の k::π 尾部を unboxed-union 根で truncate（[#R12-1]・[#R10-2] のクランプを §2.2 にも課す）。
    # RC ノード（Retain/Release）は #F1 で元々 union 根止まりだが、consume は root が payload の k::π を返す。clamp しないと
    # whole-union `Retain(o,[])` のキー `o@[]` と payload consume のキー `o@some::[]` が別バケットになり、payload consume が
    # Retain を needed にできず【load-bearing な Retain を後続 `Release(o,[])` が誤相殺 → UAF】（[#F1] の「相殺の損失」でなく不健全）。
    Retain(x@π):          R=このノード; pend[clamp_unit(root(f,x@π))].add(R)
    c@π' ∈ consume_sites: o=clamp_unit(root(f,c@π')); pend[o] の各 R を needed[R]=true にし pend から外す
                          （consume が対 Release より先着＝その経路で R は必要。恒久確定）
    Release(y@π):         o=clamp_unit(root(f,y@π)); pend[o] 非空なら R を1つ取り pairs[R].add(この Release), pend から外す
                          （この経路で R と対消滅＝R の一時 +1 を打ち消す【un-bump】Release。空なら本物の Release＝据え置き。
                           【zeroing-release 不変条件】pend[o] 非空のときだけ対消滅するので、消す Release は必ず「先行 Retain が
                           rc を上げたのを戻す非 zeroing な un-bump」で、rc を 0 にする zeroing release〔＝解放・dtor 発火〕は
                           消さない。実装が Retain を後方の任意 Release とペアにする〔pend を介さず遠くの Release を選ぶ〕と、
                           un-bump が zeroing に化けて object を使用中に早期解放＝UAF・dtor 早期発火になる。pend から取る
                           ＝最も手前の Release と対にする first-Release ペアリングがこれを保証する）
    分岐 `Match`:          非 tail（`tail_of`〔§1.2〕で非 tail＝k が x の rename 連鎖→Ret でない）は arm が継続 k で合流: needed は or（ある arm で needed なら全体）・pend は must（全 arm で pending な R だけ k へ継続）。tail（`Let(z,Match,Ret(z))`＝`tail_of` で tail）は各 arm が終端（合流 k なし＝各 arm が leaf）
  # 走査後の commit: needed[R]=false かつ「R から到達する全 leaf 経路が pairs[R] のいずれかで閉じる」
  #   （＝どの経路も consume より先に対 Release を通る）Retain R を、pairs[R] の Release ごと IR から削除。
  #   分岐をまたぐ厳密な bracket 対応は must-dataflow の実装詳細で、核心は「全経路で cancel されない限り消さない」。
```
`consume_sites`/`root` は §2.1 と共有する（相殺・borrow 化・uniqueness が同じ別名知識で動く）。

## 3. Provenance 解析

RC IR を**抽象解釈**し、各変数末端の**由来（`Provenance`）**を追う。uniqueness（`Unique`/`Dynamic`）はその由来を関数の入力に **resolve** して得る（resolve は解析の出力を消費する側＝unique-check-elim §4）——解析は「由来を追う 1 本」で、関数の効果（結果の由来）も同じ解析から出る。由来の基底は `Fresh`（新規＝resolve で `Unique`）／`Dyn`（不明＝`Dynamic`。boxed 容器 getter・global 等）／`Arg(i,p)`（入力 i の末端を引き継ぐ）。**`Retain`（複製＝2つ目の参照）だけが `Fresh`/`Arg -> Dyn`（＝`Unique -> Dynamic`）**に一方向で倒し、`Dyn` は吸収状態（`mark_threaded`/`mark_global` は降格でなく結果 `Dyn` を**生成**する op＝global 参照・`boxed_from_retained_ptr` と同じ Dyn 源）。`unique_ptr`/`shared_ptr` の対応そのもの（複製したければ `shared_ptr` に変換するしかない）。ループ・再帰は有限領域上の**不動点**で畳む。`Dynamic` では unique-check-elim が force-unique を除去せず**実行時 uniqueness チェックが残る**（§4。実行時に unique なら in-place、shared なら clone）。

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
- **cap**（closure ABI の捕捉ポインタ引数。`params` とは別フィールド §1.2）: 入力なので `env[cap]` の boxed 末端 = `Arg(cap)`（[#R10-7]: cap の `BaseSource::Arg` index は `params.len()` を用いる〔param の後ろに 1 つ足した位置〕。構築側の `Fresh`〔§3.2 の `Closure` 生成〕と対で、callee 側は入力＝`Arg`）。ただし **Fix から lower した RC IR では body は cap を getter で取り出す（→ `Dyn`）だけで cap 自身を `Ret`/mark しない**（whole 捕捉 object を名指す source 構文が無い）ので、この seed は interpret を total にする定義上のもので、いかなる結果 provenance にも影響しない（取り出しは常に `Dyn`）。
- `Let(x, Var(y), k)`: `env[x] = env[y]`（move。別名を作らないので `Dyn` 化しない）。
- `Let(x, LLVM(prim, args), k)`: prim の宣言 `Provenance`（§3.3）を実引数の由来で合成（`Arg(j,p)` を `env[a_j]@p` に置換）。alloc→`Fresh`、boxed 容器 getter→`Dyn`、unboxed 集約の子取り出し→親の子末端、はこの宣言に含まれる。
- `Let(x, Closure(_, caps), k)`: `env[x] = UnboxedAgg([Unboxed /*funptr*/, cap])`。捕捉非空なら cap の boxed 末端＝`Fresh`（新規捕捉obj に move-in）、空なら null（RC-free）。
- `Let(x, App(f, args), k)`: callee が既知なら `f` の `Provenance` を実引数の由来で合成（`Arg` 置換）。callee 不明の間接呼び出し（closure パラメータ等で decapturing が特殊化できなかった残り）は結果の boxed 末端を保守的に `Dyn`。
- `Retain(y, k)`: **retain した各 boxed 末端 π について、`root(y@π)`〔§2.1 の root。§2.2/uniqueness で共有〕を共有する全生存変数の当該末端**の `LeafSource` を **`Fresh`/`Arg`/混在を問わず `{Dyn}` に置換**する（whole `Retain(y)` は y の**全** boxed 末端、§2.2 分解後の per-末端 `Retain(y@π)` は末端 π のみ——どちらも root ベース。`let y=(x,z); Retain(y)` なら `root(y@[0])=x` と `root(y@[1])=z` の両方を各別名ごと Dyn。複製で別名を作った＝共有＝Dynamic。**「別名を作る操作で対象を `Dyn`」の原則〔§3.1〕**）。**root ベースが要る理由**: unboxed aggregate は getter が中身を追う（`env[y]=env[t]@[0]` の別名辺）。`let y=t.@0; Retain(y)` を**変数キーで `env[y]` だけ**降格すると `env[t]@[0]` を漏らし、後の再取り出し `a=t.@0` が `Fresh` に化けて §4 が誤 elide → 共有末端を破壊。root ベースなら y も `t.@0`〔root 共有〕も再取り出しも一斉に `Dyn`。boxed 容器は getter が元々 `Dyn`（中身不追跡）なのでこの漏れは起きない。既 `Dyn` はそのまま。`root(x@π)` を**共有しない**他末端は不変＝多末端値で sibling の一意性を保つ（§2.2 が per-末端に分解済み）。`Fresh`/`Arg -> Dyn` 降格を起こすのは `Retain`〔別名・per-末端・root ベース〕**のみ**。
- `Release(x, k)`: 由来は不変（`x` は dead）。
- `Let(y, LLVM(mark_threaded/mark_global, [x]), k)`（**値生成 op**）: x＝`Own`（消費）、結果 y の全 boxed 末端 ＝ `{Dyn}`（threaded/global 化した handle。§3.3）。標準の `Let(x, LLVM)` 規則で処理し専用遷移は不要——降格でなく **`Dyn` を生成**する（x は消費され、別に使うなら dual-use の `Retain(x)` が x を `Dyn` に）。so「threaded/global 値を `Fresh` handle で持てない」が保たれ、§4 が誤って elide しない。意味論: 送れる/global なのは結果 y、x は消費済み。
- `Match(s, arms)`（cexp。常に `Let(x, Match, k)` に現れる）: 各 arm を**分岐前 env のコピー**から解析し、各 arm の末尾（`Ret(Var(..))`）の値を join（末端 `LeafSource` を union）＝この Match の provenance。payload 束縛は unboxed union payload → move 取り出しで scrutinee の子末端（`Arg` 系。不在 variant は空 Set）、boxed union payload → getter＝`Dyn`。Bool もここ（2 variant）。**非 tail（`tail_of`〔§1.2〕で非 tail）** は k を全 arm で共有するので、x に結果値 join を束縛するだけでなく、**k の env は全生存変数末端について各 arm 出口 env を pointwise join（§3.1 の union）**する（片枝でのみ生じた `Dyn`——`Retain` 降格や mark op の結果——を k へ持ち越す。怠ると k で誤って `Unique` 扱い → elide → 破壊）。**tail（`Let(z, Match, Ret(Var(z)))`＝`tail_of` で tail）** は継続 k が `Ret(Var(z))` だけなので各 arm が実質 leaf（関数結果＝arm 結果値の join）。tail/非 tail いずれもこの `Let(x, Match, k)` 規則ひとつで捌ける。
- `Ret(x)`: この式の値の `Provenance` ＝ `env[x]`。**Ret は Var のみ**（[#R10-3round10]・§1.2）——App/Match/LLVM/Closure は let 束縛されその provenance は上の `Let` 規則で `env` に載せ済み。関数本体の `Ret` はその関数の結果 `Provenance`、match arm の `Ret` はその arm の値。
- global 参照 → 型どおり（boxed 末端は `Dyn`：GLOBAL 状態で unique にならない。unboxed 部は `Unboxed`/`UnboxedAgg`）。

呼び出し・`Release` は引数の由来を変えない（`Fresh -> Dyn` は `Retain` のみ）。引数の**生存**（`Own`->last-use で dead／`Borrow`->存続）は RC 側（§1.4/§2.1 の `OwnershipShape`）が決める話で、由来解析はそれに関与しない。

**関数の効果 ＝ 結果 `Provenance`**（`Ret` の由来）: param を記号 `Arg` のまま残すので**入力非依存**（関数ごとに 1 つ。再帰は不動点、初期 ⊥＝空 Set）。呼び出し `g(a…)` の結果は g の `Provenance` の `Arg(j,p)` を実引数 `a_j` の由来で埋めて（合成）得る。複製は `Retain -> Dyn` が捌く（例 `(y,y)` -> `(Dyn,Dyn)`、§5 テスト）。

**解析の停止性・抜けない関数**: 基底集合は有限（RC IR は monomorphic〔§1.2〕＋boxed 中身を追わない〔§3.1〕ので provenance 木の末端は有限、各末端 `LeafSource` も BaseSource〔`Fresh`/`Dyn`/`Arg(i,π)`・π 有限〕の有限集合）で反復は単調（union）ゆえ、**解析は実行の停止性に依らず有限時間で停止**する。**抜けない関数**（`serve = |s| let s1=step(s); let r=App(serve,[s1]); Ret(Var(r))` 等、到達可能な非再帰 `Ret(Var)` が無い）の結果 `Provenance` はこの不動点で **⊥（空 Set）に収束**する（値を産まない）。この ⊥ は戻らない関数の**呼び出し後＝到達不能コード**にしか現れず（`resolve(⊥)` は `Unique` だが dead ゆえ無害）、状態（ループを流れる値）の由来は §3.4 と同じ不動点で収束するので **in-loop の elision は効く**（抜けるかに依らず body の解析は同じ）。

### 3.3 プリミティブ宣言（`result_prov`）
`InlineLLVM` プリミティブが結果の `Provenance` を宣言する（§3.2 の interpret が引く transfer function）。
- **プリミティブ（`InlineLLVM`）= 宣言**: `LLVMGenerator::result_prov() -> Provenance`（引数の型に依存し得る）。`OwnershipShape`（§1.2）は別 API（`arg_ownership(i)`）で宣言。
- global（値の型どおりの Provenance。boxed 末端は `Dyn`、unboxed 部は型どおり）／`boxed_from_retained_ptr`（ptr→boxed → `Dyn`）。FFI（`CALL_C`）は boxed を返さない（結果 unboxed）ので rc 対象外。assert ビルドで不健全な claim を実行時検出。

例（`OwnershipShape` は §2.1／§1.2 の宣言、ここでは result `Provenance` のみ）:
- **retain getter** `Array::@(i, arr)`: `arr`＝`Borrow`、要素が boxed なら `result=Boxed({Dyn})`（容器から取り出す＝別名）、unboxed なら `Unboxed`。
- **set** `set(i, v, arr)`: `arr`＝`Own`・`v`＝`Own`（要素へ move）・`result=Boxed({Fresh})`。ループ `arr=arr.set(..)` が `Unique` を継続。`set` は shared なら clone・unique なら in-place だが**どちらも結果は単独所有の配列**なので、返る物理 object は入力 rc 次第でも uniqueness の由来は一定＝結果は入力非依存に `Fresh`（この clone-on-shared が `Provenance` を入力非依存にしている本体）。
- **punch** `punch(i, arr)` → `(PunchedArray, elem)`（§7 P0.7 の builtin・force-unique op）: `arr`＝`Own`。arr を force-unique するので **`PunchedArray` の内側 array 末端＝`Boxed({Fresh})`**（`set` 同様に入力非依存で unique。idx フィールドは `Unboxed`）。**取り出す `elem` は要素型 shape の全 `Boxed` leaf を `Boxed({Dyn})` に・`Unboxed` leaf はそのまま**（配列からの move-out＝boxed 容器の中身は追わない〔§3.1〕＝上の retain getter と同一規則）。
- **plug** `plug(elem, punched)` → `Array a`: `punched`＝`Own`・`elem`＝`Own`（穴へ move-back）。**2 版**（struct の `#plug_in`(fu)/(fu=false) と同型・§8）: **fu 版**——`punched` が共有なら force-unique（＝skip-idx clone）し `result=Boxed({Fresh})`。**汎用 `act` が `map` で plug を複数回呼び punched が共有される**ので必須（[1]）。**非 fu 版**——`punched` unique 前提で穴へ書き戻し（旧スロット unreleased）、`result=Boxed({Arg(1)})`（move-back。呼び出し側 move-out 前提でのみ健全）。線形の `mod`/`act` 特殊版が使う。
- **構築** `MakeStruct{a,b}`: boxed struct なら `a`,`b`＝`Own`・`result=Boxed({Fresh})`。unboxed struct/タプルなら `result=UnboxedAgg([Boxed({Arg(0,[])}), Boxed({Arg(1,[])})])`。
- **union variant 構築** `continue(x)`（unboxed union `LoopState` 等）: `result=UnboxedAgg`（variant ごとの payload shape）で、構築した variant のスロット＝`x` の由来（`Boxed({Arg(0,[])})` 等）・他 variant＝⊥（空 Set 末端）。§3.2 Match 読み出しの双対。boxed union なら `result=Boxed({Fresh})`（alloc。読み出しは `Dyn`）。
- **id** `id(x)`: `x`＝`Own`・`result=Boxed({Arg(0,[])})`（結果は入力 0 を引き継ぐ）。
- **mark_threaded / mark_global** `mark_*(x)`: `x`＝`Own`（消費）・`result` は全 boxed 末端 `Boxed({Dyn})`（threaded/global 化した handle）。`id` 同型だが結果 `Dyn`＋state 副作用（graph 全体を threaded/global 化）。`x` を消費するので「送れる/global なのは結果 y であって x でない」を担保。`mark_global` は user-callable でなく global 初期化点で codegen が発行（§8）。

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
- **LOCAL は `is_unique` に吸収される（別 state 解析は不要・マルチスレッドでも健全）**: in-place は `LOCAL ∧ rc==1`（§1.3）。`is_unique` は rc==1 を保証し、LOCAL（＝not THREADED。GLOBAL は global 値が `Dyn` なので自動で弾かれる）は**「threaded な値は必ず `Dyn`」**という不変条件で自動的に満たされる。threaded になる経路は (a) `mark_threaded`（RC IR では `Own` 引数・結果 `Dyn` の op。x を消費し threaded handle を**全 boxed 末端 `Dyn`** で返す。§3.3）、(b) スレッド間受け渡し＝`boxed_from_retained_ptr`（→`Dyn`）だけで、graph-walk で内側 object が threaded になっても**別 handle で触れば必ず `Dyn`**（boxed 容器への格納は `Own` consume で handle が消える／dual-use なら Retain→`Dyn`／取り出しは容器 getter＝`Dyn`）。ゆえ **threaded 値を `Fresh`(is_unique) handle で持てない ⟹ `is_unique` ⟹ not-threaded ⟹ LOCAL**。除去条件 `is_unique ∧ LOCAL` は **`is_unique` のみ**に簡約される。マルチスレッド arrayrw も、他スレッドから来た配列＝`Dyn`→`loop@D`→初回 `set` が force-unique（shared なら clone、threaded rc==1 なら atomic 確認で LOCAL 化）で結果 `Fresh`=LOCAL→`loop@U`→以降 unchecked、と「初回 checked（threaded 解除）・以降 unchecked」で捌ける。§6 の state 推論は健全性でなく最適化（初回 op の atomic-rc-check 回避等）へ。

force-unique を含む `LLVM` op（`set`/`mod`/`act` 系）で、対象 boxed 値が `is_unique`（`Boxed(Unique)`）かつ LOCAL と証明できれば、その RC IR の `LLVM` ノードを **force-unique を行わない版に差し替える**（証明できない＝`Dynamic` では除去せず、force-unique の実行時 uniqueness チェックを残す＝現状動作）。結果は force-unique 後どのみち unique なので、ループ `let arr = arr.set(…)` で 2 回目以降の入力が unique になり「**初回 checked・以降 unchecked**」が自然に出る。

### 4.1 特殊化（uniqueness 駆動、RC IR 上）
`RcFunc` を、流れてくる**引数の `UniquenessShape`（§3.1。各引数の resolve 済み uniqueness＝`Boxed(Unique)`/`Boxed(Dynamic)`・`Unboxed`・`UnboxedAgg` の木）をキー**に clone する（`Unique|Dynamic` が有限なのでキーも有限）。**v1 は入力 `UniquenessShape` 全体を key にする**（健全・clone は最悪で末端数の指数まで増え得るが、ベンチの対象関数は boxed param が少なく実際上は増えない）。**冗長 clone は「実質同一 `RcFunc` の併合パス」で潰す**: 全 key で分けても、force-unique に効かない末端の違いは elision 判定を変えないので **body は局所変数名だけ違う α 同値**になる（RC IR は名前グローバル一意〔§1.1-3〕で clone は fresh 名だけ違う）。**局所名を正規化し callee `FuncRef` を origin（元関数）に正規化した skeleton**（[#4round9]。**ただし force-unique フラグ等 body の op レベルの差は skeleton に含める**——`f@U`〔unchecked set〕と `f@D`〔checked set〕の body 差を消して誤併合すると checked 呼び出しが unchecked 版に張り替わり共有配列を in-place 破壊するので、op 差は skeleton で必ず区別する）**で初期分割**し、**対応 callee が同クラスかで partition refinement（最大不動点）**して α 同値な clone を 1 つに併合、呼び出し側を張り替える（DFA 最小化／bisimulation 同型）。callee を origin に正規化するので相互再帰でも楽観初期化が働く（`f@U`/`f@D` の callee がどちらも origin `g` で skeleton 一致し、あとは g のクラスで refine）。**[#F3round10]: この origin は §4.1 の uniqueness キー〔`@U`/`@D`〕を潰すだけで、§2.1 の版〔`g_own`/`g_borrow`〕は別 origin として保つ**（両版は body の RC ノード構成が違うので混ぜない）。**skeleton は Retain/Release ノードを含む body 全構造**（op 差だけでなく RC ノード差も区別）——「op レベルの差は含める」を RC ノードまで及ぼす。**自己/相互再帰の扱いが要点**: 自己再帰は `f@U`→`f@U`・`f@D`→`f@D` の自己呼び名が違うだけなので **self トークンに正規化**すればハッシュで一致。だが相互再帰（`f@U`→`g@U`・`f@D`→`g@D`／`g@U`→`f@U`・`g@D`→`f@D`）は callee が互いに違い、**単純な bottom-up 不動点は循環で止まる**——so「**同じ元関数の clone は全部同値」と楽観初期化し、body の対応 callee が別クラスなら分割**する coinductive な refinement（`f@U≡f@D` と `g@U≡g@D` が互いを同クラスと仮定して同時に成立）。健全性影響なし・clone 衛生（バイナリサイズ）のみ。未到達 clone の dead-function 除去と対で clone 衛生を保つ。**後段の最適化**として、そもそも冗長 clone を作らない**関連末端への射影**（各 force-unique の `is_unique` が resolve する `Provenance` に現れる `Arg(i,p)` 末端だけを key にする＝無損失。この「force-unique 関連末端」集合は uniqueness 非依存の静的性質で、reachable な force-unique が参照する入力末端を**呼び出し越しに後ろ向き**に集めて precompute する〔`loop` の関連末端 `arr` は callee `body` の set を辿って判る〕）を足せる。呼び出し地点で引数が `Unique` なら unique 用 clone を、`Dynamic` なら別 clone（または original）を呼ぶ。各 clone の uniqueness は §3.2 の入力非依存 `Provenance` を resolve（§4 冒頭）して得る（入力で分けるのはこの特殊化だけ）。worklist で到達 clone を閉包（下記の駆動）。clone は fresh 名を発番し（名前グローバル一意 §1.1-3 を保存）一意な clone 名を付ける。**特殊化は関数を clone するので、未使用になった `RcFunc`（どの call site からも到達しない original/clone）の dead-function 除去を初版で必ず実装する**（さもないと未到達 clone がバイナリに残る＝回帰。到達解析＋未到達 `RcFunc` 削除の 1 パス）。**到達根は `main`・全 `RcGlobalInit`（global 初期化子。実行時に必ず走る生きたエントリ）と、直接 `App` の既知 callee だけでなく `Closure(FuncRef)` 参照・funptr atom（global funptr 参照）も含める**——間接でしか呼ばれない lifted 関数（[6] の `with_retained` に渡す closure・ユーザ定義高階関数に渡す body 等）や、global 初期化子からのみ呼ばれる関数を「未到達」と誤判定して消すと、funptr 経由や global 初期化時の呼び出しが未定義参照になる（main は global 値を atom として読むだけで init 関数への辺を辿らない）。[6]（間接到達は特殊化せず original を残す）と対で、その original を**消さない**保証。

**駆動（worklist、clone あたり 1 パス）**: エントリ `main`（boxed 入力なし＝key 自明）と全 global init（`RcGlobalInit`）から出発。clone は `(RcFunc, key)` で一意化し、未生成を queue に積む。1 つ取り出して body を前から走査する:
1. **force-unique op**: 対象の `Provenance` を今の clone の入力 uniqueness で resolve し、`is_unique` かつ LOCAL なら unchecked 版へ差し替える（§4.2）。
2. **`App(g, a…)`**: 各引数の uniqueness を resolve → key（force-unique 関連末端に射影）→ `(g, key)` を未生成なら queue へ積む → callee を clone 名に書き換える。呼び出し結果の uniqueness は g の結果 `Provenance`（§3.2、入力非依存・全 clone 共有）を実引数の由来で resolve するだけで即得る（g の clone body の実装を待たない）。callee 不明の**間接 `App`**（closure パラメータ等・decapturing が特殊化できなかった残り）は特殊化せず（clone を積まず callee も書き換えない）、結果 uniqueness は `Dyn`（§3.3）。間接でしか到達しない関数は original（全 `Dynamic` ⟹ elision 無し）が funptr 経由で呼ばれ健全（`with_retained` に渡す closure 等もこれで original のまま）。

その他のノード（非 force-unique の `LLVM`・`Let(x, Var)`・`Retain`/`Release`・`Match`）は §3.2 の interpret どおり env（各変数の resolve 済み uniqueness）を進めるだけ（`Match` は各 arm を分岐前 env のコピーで辿り、非 tail は継続 k で**結果値と全生存変数末端の env を join**〔§3.2〕、tail は各 arm が leaf）。`Ret` で 1 clone 分の走査が終わり、queue が空になるまで繰り返す。key は有限（末端の `Unique|Dynamic`）なので clone 集合は有限＝停止。結果 uniqueness が precompute 済み `Provenance` から即決まるため各 clone は 1 回処理で済み再訪不要（＝「到達 clone 集合の閉包」で、反復不動点ではない）。

### 4.2 force-unique の除去（RC IR の `LLVM` ノード差し替え）
clone した `RcFunc` の body 中で force-unique を担う `LLVM`(InlineLLVM) ノードを、force-unique しない版（`InlineLLVM` の `force_unique=false`／unchecked generator）に差し替える（新規ノードを作って置換。共有呼び出し地点側の clone は checked のまま）。`force_unique` フラグの有無:

| 操作 | force-unique の所在 | フラグ |
|---|---|---|
| Array `set` | `InlineLLVMArraySetBody`（無条件 `make_array_unique`, builtin.rs:2170） | **無し→追加** |
| Array `swap` | 新 builtin `InlineLLVMArraySwapBody`（force-unique 内蔵。§7 P0.7 で `_unsafe_swap`＋linear-get を置換） | **新設（フラグ付き）** |
| Array `mod`/`act_identity`/`act_tuple2` | §7 P0.7 で PunchedArray punch/plug 化（punch が force-unique）。旧 `_unsafe_get_linear_bounds_unchecked_unretained`（`force_unique`, builtin.rs:1901/1936）は廃止 | punch の `force_unique`→`false` |
| struct `mod_<field>` | `#punch_fu_{field}`（`InlineLLVMStructPunchBody`{true}, `make_struct_unique` @2656） | 既存（非 fu punch あり）→`false` |
| struct `set_<field>` | `InlineLLVMStructSetBody`（無条件 `make_struct_unique`, builtin.rs:3580） | **無し→追加** |
| struct `act_<field>` | `optimize_act` の特殊版 `_act_{field}_{identity,tuple2}`（builtin.rs:3252/3103）が `#punch_fu_{field}`（force-unique, `STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL`）を使う。generic act は `unsafe_is_unique` 分岐（builtin.rs:2934、unique 枝＝非 fu punch）で §4 対象外 | punch の `force_unique`→`false`（struct mod と同じ） |

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
- **move-out（unique コンテナからの取り出し）**: 「unique なコンテナから中身を取り出し `Retain` してコンテナを `Release`」のパターンを**構造的に認識**し、「**`Retain` を省き `Release(container)` を殻解放のみ（中身は取り出した変数へ移譲）**」に直接 rewrite。boxed 捕捉の取り出し（§1.7）や boxed struct/union の destructure に効く。**§2.2 の root 照合は使わない**——boxed コンテナは中身を追わず、getter は毎回別 root なので相殺できない。代わり lowering の形から「取り出した変数＝コンテナのスロット」と分かる**構造認識**で行う（現 codegen の `get_struct_fields`/`get_union_value` の move-out 最小化の一般化）。コンテナが param のときの uniqueness は §4 の特殊化（unique-container clone）が与える（§2 前処理では §3 前で未確定）。
- **順序スケジューリング**（意味を保つ範囲で評価順を並べ替え in-place 機会を増やす。例: `f(arr.set(0,42), arr.@0)` を `arr.@0` 先に並べ替えると set が in-place 化し clone が消える）。
- **borrow routing の per-param 精密化（利得解析・[#4] 完全解消）**: §2.1 の v1 は「call 単位で `f_own`/`f_borrow` の 2 版」だが、多引数の**混在 call**（所有非 last-use の引数と所有 last-use の引数が同居）では、得のある引数を優先して call 全体を `f_borrow` に寄せるため last-use 引数の解放が呼び出し後にずれる（残り [#4]）。**引数ごとに `Own`/`Borrow` を選ぶ版**（call の各引数の所有/借用・last-use に一致する `OwnershipShape` の版。最大 2^(借用可能 param 数)・dead-func で刈る）を作れば、混在 call でも「得のある引数だけ Borrow・last-use 引数は Own で早期解放」となり [#4] が完全に消える。必要な情報（各引数の所有/借用・last-use・tail・間接）は RC の後ろ向きパスが既に計算済み。版の増殖と引き換えの最適化なので、ベンチで混在 call の [#4] がホットに出たら入れる。
- **借用クロージャ（call = borrow の呼び出し規約・[#2round9]）**: 現規約は「クロージャを呼ぶ＝それを consume（callee が cap を release）」なので、使われるクロージャ param は必ず `Own`（§2.1・consume_sites）。**call = borrow 版の呼び出し規約**（呼んでも cap を release せず caller が最後に release）を用意すれば、クロージャ param を `Borrow` にでき、高階関数がクロージャを retain せず複数回呼び/再利用できる。ただし**間接呼び出しの ABI 合意**（全 escape クロージャが同規約）に関わる大きめの変更で、クロージャは配列/struct とは別レイヤ。高階コードでクロージャ retain がホットに出たら検討。
- **`unsafe_is_unique` の静的解決（if 分岐の固定）**: §3 の uniqueness 解析が値を unique と証明できたら、`unsafe_is_unique` の結果を静的に `true` に畳み、`if unique {…} else {…}` を unique 枝へ固定する（runtime check 除去＋shared 枝 dead-code 除去）。これで generic `act` 等の is_unique 分岐が「**静的 unique＝check 無し・move-out〔clone 無し〕／dynamic＝runtime 分岐〔lazy clone〕**」の両取りになる（§8 の generic act はこれ待ち）。§4（force-unique 除去）の兄弟で、`unsafe_is_unique` を返す op を §4 driver が resolve して定数畳み込み・分岐 fold。
- **state 推論**（各値の refcount-state＝LOCAL/THREADED/GLOBAL を静的に決め、RC・状態チェック・`mark_threaded` を省く）。proven-global → `RcState::Global`（codegen no-op）。proven-local → `RcState::Local`（状態チェック省略）。送信値が proven-deeply-unique → `mark_threaded` op 省略（copy-prop で `y:=x`）。`mark_global` も静的に分かる範囲で最適化。
- **境界チェック除去**（`idx ∈ [0,size)` を証明し完全 unchecked へ。一意性除去と合成でベクトル化 0.20x）。
- **match-of-known-constructor / case-of-case**（LLVM 未実施を確認の上）。

## 7. マイルストーン
各フェーズの検証入力（何を入力し何を観測するか）は **§9 フェーズ別テスト計画**に対応。
- **P0（P1 前）**: **デバッグ情報の E2E テストを追加**してベースライン化。現状その回帰テストが無いため、`fix build -g`（DWARF 付き）でビルドした小プログラムを **gdb 駆動**（`gdb -batch`: `break main.fix:N` → run → backtrace）で検査する統合テストを作る（CLAUDE.md 規約: サンプルを tempdir にコピー、`fix`/`gdb` を `Command` 実行）。assert は file:line の解決・停止・スタックの行情報（マングル名非依存）。補助で bundled `llvm-dwarfdump` の構造 assert も可。**現 main で通すこと**＝P1 の「デバッグ情報一致」(§1.6) の比較対象。ツール: `/usr/bin/gdb` あり、`llvm-dwarfdump` は `/home/maruyama/llvm-17.0.6/bin/`（system には無し）。
- **P0.5（P1 前提）**: **Bool を union 化**（std.fix: `type Bool = unbox union {_false,_true}; true=_true(); false=_false();` ＋ 比較演算子の結果型 ＋ FFI Bool↔i8 tag）。これが `If` を IR から落とす前提（`If`→`Match` desugar は P1 lowering 内）。性能中立（Bool-union＝i8）。de-risk するなら現 `eval_if` を union Bool 対応にして先行検証、または P1 で `eval_if` 撤去と同時。**Bool は union 化後も debug/is_boolean で特別扱いを維持**（決定・§10）: `is_boolean()`〔object.rs:1715〕が Bool-union の tycon を認識し debug は `DW_ATE_BOOLEAN`〔1722〕のまま出す → P0 debug ベースライン〔§9.1〕不変・デバッガは true/false 表示（Bool-union は boolean と bit 同一なので debug 型も正確）。**`not`・比較演算子は i8 tag を直接触る専用 InlineLLVM**（`not`=`BoolNegBody`＝`icmp eq x,0`・branchless）で残し **Match に脱糖しない**（Fix ソースは union tag を触れず source 実装〔match〕だと分岐＝遅くなるため）。要確認は §10。
- **P0.7（早期・P1 と独立に入れられる）: 配列 read-modify-write を atomic builtin へ集約**（builtin `PunchedArray`＋builtin `swap`、`_unsafe_get_linear_bounds_unchecked_unretained` 系・`_unsafe_swap` を廃止）。狙い: **隠れ穴を作る composable primitive を無くして §8 (a) の DOESNT-FIT を完全消滅**させ、各 atomic op に force_unique フラグを付けて **§4 の除去対象に揃える**（swap もソートで全 unchecked 化）。std の swap 版 `PunchedArray`（std.fix:2648）の要素 move 2 回も無くす。作業:
  - **表現は `Array` を使い回す**: `PunchedArray a` は内部に**通常の `Array a` オブジェクトをそのまま持つ**（buffer/LEN/CAP・要素アクセス・`@size` 等は Array と**同一・変更なし**。LEN も減らさない＝穴は idx に据え置き）＋ punch した `idx`（unboxed）を携える（std と同じ `unbox { arr, idx }` レイアウトでよいが **swap はしない**）。
  - **`Array` と違うのは release と clone だけ**: `release(PunchedArray)` は内側 array の要素を **idx をスキップ**して解放（`[0,idx)` と `(idx,size)` の 2 レンジ）。`clone`/`make_unique` も同様に idx をスキップ。**boxed 要素型のときだけ Array と差が出る**（unboxed 要素は per-element release が無く release/clone は Array と完全同一）。→ 新規のメモリ表現もアロケーションも増やさず、`PunchedArray` **専用 traverser**の追加で実現する（機構の詳細は下）。
  - **release/clone 生成の機構（`PunchedArray` 専用 traverser を 1 つ追加）**: release/retain/mark は `create_traverser(ty, work)` が**型のフィールド構成から生成**し、unboxed 型でも呼ばれる（`build_release_mark` の unboxed 分岐、`generator.rs:1259`）。フィールド**単位**のスキップは `SubObject(ty, is_punched)` の `is_punched` で表せる（`object.rs:1646`。struct 側 punch が使用）。**ただし配列要素 1 個のスキップは既存 array 分岐への差し込みでは実現できない**: `PunchedArray = unbox { Array a, I64 }` の内側 `Array a` は**独立 refcount の boxed 値**で、要素解放は「PunchedArray を release → 内側 array の rc をデクリメント → rc==0 で初めて要素解放」の順に走り、その要素解放は**型 `Array a` で共有される名前付き traverser**（`build_traverse` の `Array` 分岐、`object.rs:1670`）を通る。これは (a) 全 `Array a` 共有ゆえ skip-idx を足すと通常配列まで壊れ、(b) enclosing struct の `idx` にアクセスできない。→ 解決は **`PunchedArray` 専用 traverser の追加**: field1(idx) を読み、field0 の内側 array を**通常の `SubObject` 再帰に委ねず**、その rc をデクリメントし、rc==0 なら **skip-idx 要素解放（idx を scope に持ち `[0,idx)` と `(idx,size)` の 2 レンジを解放）＋buffer 解放**を traverser 内で直接行う（既存 array 解放ロジックの skip-idx 複製）。`loop_over_array_buf`〔`object.rs:321`〕は `size` と `buffer` を取り loop_body が `buffer[idx]` を GEP するので、**`(idx,size)` は buffer を (idx+1) 要素オフセットして渡せば既存ヘルパをそのまま再利用できる**（`[0,idx)` は size=idx）。あるいは `loop_over_array_buf` に start／skip 引数を足してもよい——**最速のものを選ぶ（性能が同じならどれでも可）**。**mark（threaded/global）も同 traverser（work 違い）で idx スキップ**（[#R10-8a] 訂正: `TraverserWorkType` は release/mark のみ。**retain は root refcount+1 で要素を辿らない**〔generator.rs:1034-1064〕ので PunchedArray の retain に idx スキップは不要）。clone（`make_unique`）は traverser でなく `clone_array_buf` 経由なので skip-idx clone を別途用意。**この skip-idx clone は load-bearing**（堅牢性目的でなく実走する）——汎用 `act` の `f(e).map(|e'| plug(e', p))` は p を捕捉し `map` が **plug を複数回呼ぶ**ので p が共有になり、fu plug が p を force-unique して skip-idx clone を走らせる（各 plug が独立配列を返す。§3.3 plug・[1]）。線形の `mod`/`act` 特殊版だけ p が単一使用＝unique で clone は走らない。∴ 「既存 array 分岐への 1 行追加」ではなく **`PunchedArray` 専用 traverser の追加**（既存 `loop_over_array_buf` を再利用する限定的な実装）。boxing は不要（`unbox { Array a, I64 }` のまま、std 版の swap-to-end をこの専用 traverser に置換）。型名は `PunchedArray`（std 同名 struct を builtin で置換）。
  - **punch/plug（新 builtin・swap 無し）**: `punch : I64 -> Array a -> (PunchedArray a, a)` ＝ idx の要素を no-retain で move-out、穴は idx に据え置き、idx を tag。**force-unique 版も用意**（`mod`/`act` の §4 除去がこの上に乗る）。`plug : a -> PunchedArray a -> Array a`（elem 先・punched が dot subject＝`punched.plug(elem)`。現 std `_plug_in : a -> PunchedArray a -> Array a` と同順・§3.3 の `plug(elem, punched)`〔Arg(0)=elem・Arg(1)=punched〕と一致。[#3]）＝ idx に書き戻し（unreleased）て Array へ戻す。**plug も fu／非 fu の 2 版**（fu＝共有 `punched` を force-unique〔skip-idx clone〕・汎用 `act` 用／非 fu＝`punched` unique 前提・線形 `mod`/`act` 用。§3.3・§8）。所有権は **PunchedArray＝idx 以外の全要素／取り出した要素＝idx** に分割される（∴ 取り出した要素は正真正銘の所有末端＝§8 (a) が CLEAN 化）。
  - **抽象に保つ**: 内側 `Array` をフィールド取り出しさせない（取り出して通常 `Array` として release すると idx 二重解放）。punch/plug/release/clone 以外の操作を生やさない。
  - **builtin `swap`（force_unique フラグ付き）**: `swap : I64 -> I64 -> Array a -> Array a` を単一 InlineLLVM op（`ArraySwapBody`）で codegen 直書き（`set` 同様＝超高速）。`i,j` の要素を op 内部で move 交換（`read_from_array_buf_noretain`＋`write_to_array_buf` を op 内に閉じ込め、穴を外に出さない）。`set` と同じ **`force_unique` フラグ**を持ち、§4 が配列を静的 unique と証明できれば unchecked 版へ差し替え → ソート（introsort/heapsort、std.fix:439/461/466/473/527/555）の O(n log n) 回の swap から uniqueness チェックが消える。arg: `i,j` unboxed・`array`=`Own`、result=`Fresh`（force-unique 版・unchecked 版とも結果は unique）＝**CLEAN**（穴を露出しないので §8 (a) の窓が消える）。
  - **廃止と置換**: `_unsafe_get_linear_bounds_unchecked_unretained`(+`_forceunique`) と `_unsafe_swap_bounds_uniqueness_unchecked` を**廃止**し、利用者を atomic builtin へ移す—— `mod`/`act`（単一穴、std.fix:166/174/190）→ PunchedArray punch/plug、`swap`（二重穴、570/571）→ builtin swap、`pop_back`（`set_size(len-1)` 後に**境界外**の末端を取り出す、366）→ 専用 builtin（shrink＋末端 move-out を op 内に閉じ込め）、generic `act`・std swap 版 `PunchedArray`（2668/2692/2698）→ builtin PunchedArray。**`_unsafe_set_bounds_uniqueness_unchecked_unreleased` は残す**（穴埋め用途は消えるが、append/push_back/map/reserve の**未初期化スロット書き込み**という別用途で正当に使う。265/307/383/401/689）。生 codegen helper（`read_from_array_buf_noretain` 等）は各 atomic builtin の**内部**に閉じ、Fix レベルの composable op としては露出させない → **隠れ穴 primitive はどこにも残らない**。
  - **削除チェックリスト（P0.7 完了後に確実に消す。残すと再利用される）**: (i) `_unsafe_get_linear_bounds_unchecked_unretained`(+`_forceunique`)＝builtin.rs `InlineLLVMArrayUnsafeGetLinearBoundsUncheckedUnretained`＋stdlib.rs 登録＋std.fix 呼び出し元（166/174/190/366/570/571/2668/2692/2698）＋doc。(ii) `_unsafe_swap_bounds_uniqueness_unchecked`（`*_uniqueness_unchecked` 族。std.fix:567 定義＋呼び出し元 439/461/466/473/527/555）。(iii) std `PunchedArray` 型（2648）＋`_plug_in`（2658）＋`_unsafe_punch_bounds_uniqueness_unchecked`（`*_uniqueness_unchecked` 族。2686）。**`#[deprecated]` 等で残さず物理削除**（CLAUDE.md の dead-code 方針）。
  - **`_unsafe_set_bounds_uniqueness_unchecked_unreleased`（Rust builtin）は P0.7 で完全には消えない**: hole-plug 利用（mod/act/swap）は上記で消えるが、**未初期化 fill 利用（push/append/map/reserve、265/307/383/401/689）が残る**。これも `*_uniqueness_unchecked` 族なので **P3.5 で置換・削除**（下記）。**恒久的に残すもの（別軸）**: `_unsafe_set_size`・`_unsafe_empty_capacity_unchecked`・`_unsafe_fill_size_unchecked`（size/capacity）・`_unsafe_get_bounds_unchecked`（bounds＝§6 の軸）・`_unsafe_force_unique`（安全な force-unique）。
  - **穴 release のデッドコード確認（要望）**: `mod`／total-functor `act` は punch を必ず plug するので PunchedArray を drop せず、その skip-idx release を**呼ばない**。PunchedArray を unbox 型にして release を drop 地点でインライン生成する実装なら、drop 地点の無いプログラムでは skip-idx release が**そもそも生成されない**（望ましい）。`emit_symbols`／IR を grep して `mod`/`act`(total) のみのプログラムで skip-idx release が出ないことを確認。型ごと無条件生成する実装なら LLVM/リンカ DCE で strip されることを最終バイナリで確認。generic `act`（失敗し得る functor は PunchedArray を drop し得る）では release は**実使用**なので大域 dead ではない（total path でのみ dead）。
  - **独立性・検証**: RC IR 導入前に単独で入る（挙動不変の内部最適化＝changelog 不要）。共有値テスト（2 箇所格納で破壊しないこと）＋回帰ベンチ（`benchmark/speedtest`・`fix-bench/batch`）で std PunchedArray の swap トリック除去分（generic `act`）の改善と非劣化を確認。assert ビルドで穴の二重解放を検出。
- **P1**: RC IR 型 ＋ AST→RC IR lowering（`generator.rs` から RC 抽出。名前は lowering が fresh 発番）＋ codegen 付け替え ＋ 全テスト再検証。**最大の山**。完了ゲート: `cargo test --release` 全最適化レベル（`FIX_MAX_OPT_LEVEL` max/basic/none、§1.6）で全通過・全ベンチでリグレッションなし・デバッグ情報一致（§1.6）を満たし、**ユーザに連絡して外部ライブラリテストを依頼してから次フェーズへ**。lowering は現 codegen の RC（move-out/last-use ＝既に最小 RC）を踏襲し、引数は全 `Own`（ベースライン）。source 関数の borrow 化・相殺（§2）は P2 で uniqueness と併せて入れる（前処理）。なお `InlineLLVM` の `OwnershipShape` 宣言（read getter は `Borrow`。§1.2）だけでも hot loop の array getter は借用扱いになり、ベンチの elision の主要部はそれで届く。
- **P2**: 前処理（borrow 化 §2.1・相殺 §2.2 で `OwnershipShape` を確定し `Retain` を削る）＋ uniqueness 解析（`Provenance` を追い resolve で uniqueness を得る）。read-only ログから始め arrayrw のループ `set` を `Unique`・共有される配列のテストを `Dynamic`（非 unique）と判定することを確認。
- **P3**: unique-check-elim（force-unique 除去 ＋ 特殊化）。特殊化キーは初版は全 `UniquenessShape`（関連末端への射影は後続の clone 削減）。**未到達 `RcFunc` の dead-function 除去（到達解析＋削除）と α 同値 `RcFunc` の併合（局所名だけ違う冗長 clone を潰す・§4.1）を同時に実装**——未使用/重複 clone を掃除しないと未到達・重複関数がバイナリに残る。arrayrw/fannkuch 計測、全テスト。
- **P3.5（§4 後の `*_uniqueness_unchecked` 掃除）**: **`*_uniqueness_unchecked` 族**（uniqueness チェックを飛ばして possibly-shared を黙って mutate する危険関数）は §4 が uniqueness を静的に扱えるので価値を失う → 全廃する。swap/punch は P0.7 で builtin 化済み。残るのは `_unsafe_set_bounds_uniqueness_unchecked_unreleased` の**未初期化 fill 利用**（push/append/map/reserve、265/307/383/401/689）——これを、上流の `_unsafe_force_unique`（§4 が elide）＋「未初期化スロット write」の狭い内部 primitive（uniqueness は上流で保証、名前から `uniqueness_unchecked` を落とす）へ置換して削除。手順: **安全版に置換 → §4 が overhead を消すことを確認 → 物理削除**（`#[deprecated]` で残さない）。§4 が unique を証明できない箇所はチェックが残る（安全側の正しい挙動＝precision は §2 で上げる）。`_unsafe_force_unique` 自体は安全（force-unique する）ので残す。狙い: possibly-shared を黙って壊す誤用しやすい unsafe を撲滅し、安全版と同速を §4 で担保。
- **P4**: reuse / 順序スケジューリング / 境界チェック除去 等。
- **（途中経過の性能に関する注意）**: `*_uniqueness_unchecked` の除去（P0.7 の swap/punch、P3.5 の未初期化 fill）は、**§4（P3）が force-unique を静的に飛ばすまでの間、一時的に性能を下げる**。特に **swap** は現状 uniqueness チェックを完全に飛ばしているので、force-unique フラグ付き builtin へ置換すると P0.7〜P3 の間はソートの各 swap に refcount チェックが 1 個乗る（`mod`/`act` は元々 `_forceunique` なのでほぼ中立）。**これは unsafe 除去による想定内の劣化で §4 で解消予定**。中間マイルストーンで benchmark を取るときは、劣化を「後で §4 が解消する分（unsafe 除去起因）」と「意図しない劣化」に**振り分ける**こと（commit hash＋どの unsafe をいつ外したかを記録し、§4 適用後に回復するかで判定）。回避したいなら P0.7 の abolish/置換を §4 と同じ P3 に寄せる選択肢もある（早期独立性と引き換え）。

## 8. リスク・未解決
- **P1 の codegen 付け替えの再検証コスト・範囲**が最大リスク（全プログラムに影響）。段階導入できるか（一部関数だけ RC IR 経由、等）も検討。
- uniqueness の precision は「`Retain` を入れないこと」で作る（borrow 化 §2.1・相殺 §2.2）。`Retain` は `Unique -> Dynamic` の一方向で回復しないので、`Retain` を減らせるかが要。
- ローカル名一意の**全変換での保存**（lowering は fresh 発番で構築的に一意。clone/特殊化は fresh 名発番で freshen）。
- getter（射影）の retain 有無・`Provenance` の不動点収束・threaded state・boxed の escape（`boxed_to_retained_ptr`）の RC IR での扱い。捕捉クロージャの捕捉 object は **boxed dynamic object** なので §3.1「boxed の中身は追わない」に従い**単一 `Boxed({Fresh})` 末端**として扱う（取り出しは `Dyn`・保守的で健全。§3.2 と一致）。捕捉ごとに `UnboxedAgg` の子として**透過追跡**するのは、クロージャ共有時に `Retain` が全捕捉子を `Dyn` 降格する別名健全性を要する**将来の精度拡張**（§3.1 の boxed 不追跡の例外）。
- 別名健全性は「別名を作る操作で対象を **`root` ごと** `Dynamic` にする」で担保（§3.2 の `Retain` 規則＝root ベース降格・§2.1 の root で共有）。`Retain(y)` は `root(y)` を共有する全末端を `Dyn` にするので、unboxed aggregate の `let y=t.@0; Retain(y)` でも `env[t]@[0]` を漏らさず再取り出し `t.@0` が `Fresh` に化けない（[4]）。変数キー降格では unboxed で source 末端を漏らし破壊するので不可。boxed 容器は getter が元々 `Dyn` を返す（中身不追跡）ので漏れない。

### InlineLLVM の `OwnershipShape`/`result_prov` 全件監査（67 件）
全 `LLVMGenerator` variant（`src/fixstd/builtin.rs`）を精読し、2 属性（`arg_ownership(i)`／`result_prov`）を割り当て可能か確認した。内訳の目安: **CLEAN ~41**（数値/比較/cast/literal＝unboxed in/out で宣言不要・結果 `Unboxed`、構築＝`Fresh`、read-only getter〔size/capacity/`union is`〕＝`Borrow`・結果 `Unboxed`、`empty`/`fill`＝`Fresh`、`DestructorMake`＝子を move-in し `Fresh`）、**NONTRIVIAL ~19**（下記(1)。割り当て可だが実装者が誤りやすい）、**DOESNT-FIT ~7**（下記(2)。2 属性で綺麗に表せない）。

**(1) 宣言を誤りやすい（P1 監査のチェックリスト）**:
- **force-unique 内包 → 結果 `Fresh`**（返るポインタが入力と同一でも一意保証）: `set`・`Array::force_unique`・struct `set_`/`mod_`(fu)・`plug_in`(fu)・`_unsafe_mutate_boxed_*`。
- **force-unique しない in-place → 結果 `Arg(i)`**（同じ・共有かもしれないポインタ。**`Fresh` にしない**）: `_unsafe_set_size`・`_unsafe_set_bounds_uniqueness_unchecked_unreleased`・`punch`(fu=false)・`plug_in`(fu=false)。後者2つは「古いスロットを release しない」内容 RC も伴い、呼び出し側の move-out 前提でのみ健全。
- **boxed 容器から要素/フィールドを取り出す getter → `Dyn`**（retain されるが容器と共有＝unique でない。**`Fresh` にしない**）: `_unsafe_get_bounds_unchecked`（要素 boxed 時）・boxed struct の `@field`・boxed union の `as_`。
- **container の boxed/unbox で結果が変わる**（一律に扱わない）: struct `@field`／union `as_` は boxed 容器 → `Dyn`、unbox 容器 → `Arg(i,[field])`（move-out・no-retain の素通し。retain されないので「retain 済みコピー」と誤ると二重計上）。**unbox getter の `OwnershipShape` は全 boxed 末端 `Own`**（[#F4]）: 取り出す field の末端は結果へ素通し（`root` が除外＝consume でない）だが、**取り出さない sibling 末端は op が内部で release ＝ consume する**（現 codegen `get_struct_fields` の unboxed 枝〔object.rs〕は「`ret` の field を retain せず、`ret` に無い field を release」）。so `arg_ownership` は全末端 Own・result は取り出し末端の `Arg(i,[field])`・sibling 末端は consume_sites 入り。sibling を `Borrow` 宣言すると (i) 実装が release しなければ sibling がリーク、(ii) release しつつ流入 param を borrow のまま残すと二重解放。「取り出し末端＝素通し・sibling＝consume・容器は Own」で一意に整合する。
- **`unsafe_is_unique` の value 部は passthrough `Arg(0)`**（refcount を**読む**だけで state も count も変えない＝`Fresh` を保つ）。**`mark_threaded`（InlineLLVM builtin `InlineLLVMMarkThreadedFunctionBody`, builtin.rs:5200）は `Own` 引数・結果 `Dyn` の値生成 op**——x を消費して threaded handle を返す（`id` 同型だが結果 `Dyn`＋state 副作用）。**operand が `Own`（＝借用不可、`set` と同様に状態変更は所有を要する）なのが要**: param を mark するだけの関数を Borrow にすると caller が threaded object への `Fresh` handle を残し「`is_unique ⟹ LOCAL`」補題が破れる（[5]）。Own なら caller が dual-use で `Retain`→`Dyn` になり補題回復。**`mark_global` も同形の値生成 op**だが user-callable でなく global 初期化点で codegen が発行（`gc.mark_global` generator.rs:1497。§8 の global 表現。builtin 不在＝[5] の穴は元々無いが一貫性で同形）。結果 `Dyn` は「降格」でなく「生成」（global 参照・`boxed_from_retained_ptr` と同じ Dyn 源）。
- **`union mod` の結果は分岐の phi = `{Fresh, Arg(1)}`**（match 枝＝新規構築、mismatch 枝＝入力 union 素通し）。
- **値未使用・型 witness のみ → arg `Borrow`・結果 `Ptr`＝`Unboxed`**: `get_retain_function_of_boxed_value`／`get_release_function_of_boxed_value`（引数の**型**だけ使い値は捨てる。返すのは helper 関数ポインタ＝rc 対象外）。
- **`union is` の RC idiom**: `get_scoped_obj_noretain`＋`if !used_later release`（実効 `Own` だが transient retain 無し）。`get_scoped_obj` ベースの `Own` パターンと形が違うので検査が見落としやすい（§8 分類A の read getter 群と同類）。

**(2) 2 属性で綺麗に表せない（opaque 化 or 別扱いが要る）**:
- **(a) タプル内の move-out 穴 → builtin `PunchedArray` で解消（決定・早期フェーズ）**: `_unsafe_get_linear_bounds_unchecked_unretained`（`mod`/`act` の中核）は `(Array a, a)` を返し、要素を **no-retain で move-out**（builtin.rs:1946）＝tuple.0 の配列は `idx` に**穴が空く**（stale スロット）。tuple.0 の型が平の `Array a` で**穴が型に出ない**ため、通常の配列デストラクタが `idx` も解放しにいき、tuple.1 と **idx 要素を二重に所有**するように見える（`Provenance` の各 boxed 末端＝所有参照、の前提を破る）。これが破綻の根本（症状としては「借用末端／refcount 共有」）。
  - std の `PunchedArray`（`unbox struct {_arr, _idx}`, std.fix:2648）は穴を**末尾へ swap＋@size 減で境界外へ追い出して** release-safe にしているが、`idx != 末尾`で**要素 move が 2 回余分**（drop され得る generic functor 対応のため）。
  - **解消: `PunchedArray` を builtin 抽象型化**（`{Array a, idx}`、専用デストラクタ／clone が **idx をスキップ**）。穴は idx に据え置き（**swap 無し・実行時ゼロコスト**、idx は unbox でレジスタ）、所有権が **PunchedArray＝idx 以外／element＝idx** にきれいに分割され、tuple.1 は正真正銘の所有末端・穴は型に出る → shape 解析が普通の boxed 型として扱え **(a) は CLEAN 化**。droppable なので最適化パス（total functor）と generic パスを一本化でき、std の swap 版を上位互換で置換、「linear 窓を触らない」壊れやすい不変条件も不要。punch/plug のみの抽象型に保つ（内側 `Array` を取り出させない）。
  - この置換で linear-get の unsafe 3 点のうち **(3) move-out の refcount プロトコルが型で安全化**され、残る unsafe は (1) 境界（＋非 forceunique 版の (2) 一意性）だけになる。§4 の force-unique 除去は punch 内で効くので不変。早期フェーズで入れる（§7）。
  - **PunchedArray が扱うのは穴 1 個**（`mod`/`act`）。`swap` のように**穴を 2 つ同時**に開ける op は単一 idx の PunchedArray に載らないが、**builtin `swap`（force_unique フラグ付き）として atomic 化**する（§7 P0.7）——穴を op 内部に閉じるので窓が露出せず、これも CLEAN・§4 除去対象（ソートで全 unchecked 化）。生 `_unsafe_get_linear_bounds_unchecked_unretained`/`_unsafe_swap` 系は廃止（§7 P0.7）＝**隠れ穴を作る composable primitive はどこにも残さない**ので、DOESNT-FIT (a) は完全消滅する。
- **(b) boxed 内部への生ポインタ**: `Array::_unsafe_get_ptr`（`Array a -> Ptr`, builtin.rs:2393）・`_get_boxed_ptr`（`a -> Ptr`, std.fix:938。[#R10-8b] 実名）は結果 `Ptr`（`Unboxed`）が**引数 boxed の内部バッファを alias**する。現 codegen は引数を last-use で release し得る＝返した `Ptr` が dangling しうる（`unsafe` op）。RC IR では引数を **`Borrow`**（呼び出し側が Ptr の生存中 配列を保持）と宣言するのが正しく、`Ptr` 結果は `Unboxed` で解析は alias を追わない——寿命義務はモデル外（呼び出し側責務）。「unboxed 結果が boxed 引数の内部を alias しその寿命に縛られる」は 2 属性の外。
- **(c) RC 追跡域の境界**: `boxed_to_retained_ptr`（boxed の 1 参照を生 `Ptr` へ escape。arg=`Own`、結果 `Ptr`=`Unboxed`＝生きた参照が帳簿外へ）／`boxed_from_retained_ptr`（生 `Ptr` の参照を boxed `a` へ materialize。結果=`Dyn`）。前者は「`Unboxed` の引数/結果が実は所有権を運ぶ」非対称で `Own`/`Borrow` 軸に載らない（§8 冒頭の boxed escape 未解決項目）。健全側には「to→arg `Own`・結果は追跡外／from→結果 `Dyn`」で倒す。
- **(d) `with_retained`**: 呼び出しをまたぐ意味的 retain。arg=`Own`/`Own` では意味が抜ける。**opaque のまま常に retain**（§8 の (B) の結論を再確認）。
- **(e) `fix`（`FixBody`）**: `free_vars` に Fix レベルの仮引数でない暗黙 capture `#CAP` が含まれ、それを合成 closure `fixf` に alias 格納して `f` が消費する（中間 closure のフロー）。RC IR では `LLVM(FixBody,[x,f,cap])` の全 `Own`＋内部 RC opaque で扱う（§8 の fix 項で既述）。per-Fix-arg だけ見ると `cap`（=`#CAP`）を見落とす点に注意。

**(3) 監査で裏取りできた既存 PLAN 決定**:
- unbox union の結果は「variant ごとの `UnboxedAgg`＋非活性 variant ⊥」で表す（§3.1/§3.3）。`make_union`/`union as`/`union mod` の unbox 版がこの表現を要求＝§3.3 で足した union 構築宣言が load-bearing と確認（tag 付き sum を positional product＋⊥ で表す点が肝）。
- `unsafe_is_unique` の value 部＝`Arg(0)` passthrough（§8）で `Fresh` を保つ、は妥当と確認。force-unique 内 clone を op に内包（§3.3/§4.2/§8）も確認。

### 決定事項・要確認
- **（決定）状態は変数ごとの `Provenance`（`State{env: Map<Var,Provenance>}`）、uniqueness は resolve して得る**（§3）: 各 boxed 末端の由来（`Fresh`/`Dyn`/`Arg`）を追い、`is_unique` は入力に resolve して `Unique` か見る。boxed 容器の中身は追わない（取り出しは `Dyn`）。`Fresh`/`Arg -> Dyn`（＝`Unique -> Dynamic`）への**降格**を起こすのは `Retain`（複製＝2つ目の参照）**のみ**で、`Dyn` は吸収状態（`unique_ptr`/`shared_ptr` 対応）。global 参照・`boxed_from_retained_ptr`・`mark_threaded`/`mark_global`（値生成 op、operand は `Own`＝借用不可・[5]）は末端を最初から `Dyn` として**生成**する（既存 `Fresh` の降格でなく Dyn 源）。分岐合流の join は §3.1 の集合 **union** で複数由来を保持し、`Dyn` への潰し込みは resolve の ⊔ が行う（join 自体は `Fresh` を降格しない）。move（`Let(x, Var(y))`）は由来を引き継ぐ。unboxed 集約は `UnboxedAgg` で子の由来を追う（`LoopState` 越しの配列など線形な受け渡しの精度）。
- **（決定）`Construct` ノードを設けず構築も `LLVM`**（§1.2）: 集約構築（struct/タプル/`ArrayLit`/union variant）は alloc 系 `LLVM` プリミティブ＋`Provenance`（引数をスロットへ move。結果は boxed 集約＝`Fresh`、unboxed 集約＝子の由来を担ぐ）で表す。InlineLLVM が効果を宣言する設計なので boxed alloc の Fresh も unboxed 集約が担ぐ子の由来も解析に伝わる（専用ノードを持たない。射影＝getter を専用ノード化しない方針の双対）。
- **（決定）boxed rc を `Unique | Dynamic` の 2 点で表す**（`CTRefCnt`、§3.1）: `alloc=Unique`、`Retain`->`Dynamic`（一方向）、`Release`・呼び出しは `Dynamic` を回復させない。これにより (a) **関数効果 ＝ 結果の由来 `Provenance`** で書ける（引数の生存は `OwnershipShape` が決める＝`Own`+`Unique` は last-use で dead、`Dynamic` は据え置き、`Borrow` は存続。§1.2）、(b) precision は borrow 化（§2.1）＋相殺（§2.2）で `Retain` を減らして作る、(c) 2 点束かつ単調遷移なので不動点は自明。`Provenance` は入力非依存なので `FuncRef` ごとに 1 つ（入力で分けるのは §4.1 の特殊化だけ）。
- **（決定）force-unique op の結果 `Provenance` は `Fresh` 固定（unchecked 化しても `Arg` 素通しにしない）**（§3.3/§4）: `set`/`swap`/`mod`(punch) 等は、入力が unique なら in-place で**入力と同一 object**を返し、shared なら clone で**別 object**を返す——結果の identity は入力依存だが、**uniqueness はどちらも `Unique` で入力非依存**。`Fresh` はこの入力非依存な uniqueness を表す。§4 で unchecked 版に差し替えた op は実際には入力を alias する（真の identity は `Arg(i)`）が、そこで provenance を `Arg(i)` に更新すると「checked=`Fresh`／unchecked=`Arg(i)`」＝§4 の除去判断に依存＝**入力依存**になり、§3↔§4 の反復不動点が要る（「provenance を 1 回計算・入力非依存」という設計が壊れる）。§4 は uniqueness しか読まず `Fresh` と `Arg(i)` は resolve 結果が同じなので**除去の得はゼロ**。identity/別名が効くのは §6（reuse・順序スケジューリング）で、そこは elim 後に「どの結果が入力を alias するか」を独立の局所パスで求めればよく、コア provenance に混ぜない。∴ `Fresh` 固定。
- **（決定）`RcRhs::Var(Var)`（move-bind）を持つ**（§1.2/§3.2）: `let y = x` を表せる。意味は move（`x` 消費・`y` が由来 `Provenance` を引き継ぐ、`Unique` も継ぐ）で rc 中立、それ自体は `Dynamic` トリガーでない。エイリアス（`x`,`y` 両方生存）は「後でも `x` を使う＝non-last use なので手前に `Retain`（`->Dynamic`）」で出る＝copy = `Retain` + move。copy propagation で消せる。
- **（決定）borrow 化を source 関数へ拡張**（§2.1）: lowering の all-`Own` を、読むだけの引数に限り `Own` -> `Borrow` へ**書き換える**。引数ごとの `OwnershipShape`（`UniquenessShape` 同型、末端 boxed に `Ownership`）を、消費の有無からコールグラフ上の最大不動点（初期 `Borrow`、消費で `Own` に降格）で決め、callee の内部 `Release` を落として呼び出し側へ出す（余る `Retain`/`Release` は §2.2 が相殺）。uniqueness 解析の前に走らせる。P2。
- **（決定）borrow 化の release 遅延はメモリ・トレードオフ——v1 の利得 routing で大部分解消**（§2.1・[#4]）: borrow 化は callee 内部の release を caller の呼び出し後へ移すので、値の解放が遅れて (a) ピークメモリ増（大きい借用 object ＋ callee が last-use 後に大量 alloc の例で最大 2 倍）、(b) release 直後の alloc による領域再利用の喪失、が起き得る。**v1 の利得 routing（§2.1）がこれを大部分潰す**: 「**所有かつ last-use**」の引数を渡す呼び出しは得が無いので `f_own` へ routing され、callee が**早期解放**する（build 型＝大きい借用 object を last-use で渡し callee が大 alloc、もピーク不変）。**残る遅延は混在 call の所有 last-use 引数だけ**（[#F5]: 同じ call に「**得あり引数（所有非 last-use、または借用値の受け渡し）**」があると call 全体が `f_borrow` へ行き、所有 last-use 引数も借用され解放がずれる）＝**per-param 精密化（§6）で消える将来課題**・v1 は許容（稀・net-zero・健全・clone は起きない）。**`Destructor` の dtor 遅延も同根で契約違反ではない**: 契約は「必ず解放される」（release は消えず遅れるだけ＝保存）と「`Destructor::borrow` の work 中は生存＝早すぎる解放をしない」（遅らせるのみで早めない＝保存）で、遅延はどちらも破らない。精密な dtor タイミングは opaque な `Destructor::borrow`/`borrow_io`（最適化対象外）を使う。なお baseline は正しい Fix コードで通常 clone しない（値は mutate 時点で unique）ので borrow 化は CPU（retain/release 削減＋is_unique 除去）の勝ち・メモリは中立〜微負。
- **（決定）Bool→union（P0.5、§7）**: std.fix 定義＋比較演算子の結果型＋FFI（Bool↔i8 tag、`_false`=0/`_true`=1）。`If`→`Match` desugar は P1 lowering 内。要確認: 比較 InlineLLVM の結果構築・`&&`/`||`/`not`・typecheck が union Bool で通るか（ビットは i8 不変）。**Bool は union 化後も debug/is_boolean で特別扱いを維持する**（`is_boolean()` が Bool-union tycon を認識・debug は `DW_ATE_BOOLEAN`〔object.rs:1715/1722〕→ P0 debug ベースライン不変。**素の union 化〔debug=union struct・ベースライン取り直し〕は却下**——レイアウトは i8 で変わらずデバッガ UX/外部ツール互換を保てるため）。`not`（**trait `Not`〔std.fix:3487〕の `Bool` インスタンス**——`impl Bool : Not { not = |x| BoolNegBody(x) }` 相当を Rust 側で builtin 登録〔`not_trait_instance_bool`, builtin.rs:6618〕・branchless `icmp`）・比較演算子（`Eq`/`LessThan` 等の `Bool` 系インスタンス）は **i8 tag を直接触る InlineLLVM をインスタンス本体に持つまま残し** **Match に脱糖しない**（Fix ソースは union tag を触れないので `Bool` インスタンスを source 実装〔match〕にすると分岐＝遅い。**Bool 用 `not` InlineLLVM は必須**）。
- **（決定）global 値の表現**: global 初期化を RC IR（init）として表し `mark_global` op（`Own` 引数・結果 `Dyn`）を init で発行（init 値を消費し global 値へ）。参照は atom で、解析は値の型どおり（boxed 末端は `Dynamic`）。program = top-level 関数集合 ＋ global init。現状の global 機構（lazy/eager・mark_global 発火点）は P1 実装時に確認。
- **（決定）lowering サブパス順**: AST 正規化（ANF 化 → lambda lift → `If`→`Match` desugar → destructure→getter → fresh 命名）→ 最後に last-use 解析＋明示 retain/release 挿入で RC IR 生成（形と名前が確定してから RC を載せる）。
- **（調査済み）RC site 監査の規模**: codegen の RC は `generator.rs` ~38・`builtin.rs` ~29（InlineLLVM `generate` 内部の release/retain）・`object.rs` ~21。builtin の 29 を「primitive 内 atomic（`make_array_unique` の clone-release 等、op 意味に内包）」「明示 `Release` 化すべきもの（引数を使用後に release 等）」「外部化できず宣言で残す内部 RC」に分類するのが P1 の主要監査。
- **（調査済み）`is_var_used_later` 依存の InlineLLVM**（`builtin.rs` 全10 site を分類。いずれも RC 判断のみで計算結果・挙動は used_later に非依存＝in-place/clone はランタイム refcount で決定）: **(A) 借用読み後 last-use なら引数 release**（`noretain` 読み＋`if !used_later release`）＝1855（配列要素 get）/2418（配列 ptr）/2489（get_size）/2540（get_capacity）/3873（union `is`）/4546・4648（retain 関数 ptr 取得）/4755（data ptr 取得）の 8。**(B) 呼び出しをまたぐ retain/release**（`with_retained`: `f(x)` の前後で x を retain→release し呼び出し中 x を生存）＝4206+4214。RC IR では (A) は容器引数を `Borrow`（借用）で宣言し、lowering の last-use 解析が容器の明示 `Release` を last-use に配置（getter が boxed 要素を retain するのは別効果）。(B)（`with_retained`）は **opaque な InlineLLVM のまま retain/release を内部に埋める**。この `Retain` は呼び出し中 x を shared に見せ f の in-place 変更を防ぐ**意味的** RC で、最適化で消えては困る＝外に出すメリットが無くリスク（相殺で消える）だけなので内部に残すのが正しい。used_later スキップは落として**常に retain**（内部 RC は codegen 時に used_later を見ない）。P1 の書き換え: (A) は used_later を `Borrow`＋lowering の last-use 解析へ移す、(B) は常に retain へ。どちらも `generate` は `is_var_used_later` を呼ばなくなる（grep 由来なので網羅監査）。
- **（要確認）各 InlineLLVM 引数の `OwnershipShape` と `Borrow` 化可否**: read-only op（§8 分類A）は既に `noretain`（借用的）なので `Borrow` に素直に対応。`Own` で retain してから読む引数があれば `Borrow` 化＋release 外出し＋相殺で速くなる（§6）。`Borrow` 化できない op もある。全件確認が要る。**`fix`（不動点コンビネータ）・bulk array 系**が `Borrow` 化できない候補（`loop` は InlineLLVM op でなく std の再帰関数なので対象外＝§3.2 の不動点で透過的に解析される）。P1 監査で各引数を「`Borrow` 化可／`Own` のまま（内部 RC を宣言で残す）」に分類する。
- **force-unique 内 clone の RC 境界**: `make_array_unique`/`make_struct_unique` の clone（共有時に deep copy ＋要素 retain）は op の atomic 意味に内包し、内部 RC は IR ノードに出さない（最適化対象でない共有パスのため）。引数 `OwnershipShape` のみ宣言。
- **（調査済み）ソースレベルの `unsafe_is_unique` 分岐**（std.fix 3 箇所）は §4 の対象外だが健全なので削除しない: (1) generic `Array::act`（`_unsafe_act_bounds_unchecked`、任意 functor）が `arr.unsafe_is_unique` で unique(punch)／shared(clone+set) を実行時分岐する。`optimize_act`（`src/optimization/optimize_act.rs`、既存・`enable_act_optimization`）が Identity/Const/Tuple2 の act body を force-unique op 版（`_unsafe_act_bounds_unchecked_{identity,const,tuple2}`）へ lowering 前に置換するので、ホットな act はこの分岐を通らず §4 の対象になる（`mod` は元から force-unique op 版）。それ以外の functor の act だけ分岐が残り、RC IR では `unsafe_is_unique` の結果 `Bool` 上の runtime `Match` に lower される（§3.2 が健全に扱い、§4 は差し替えない）。(2) `Destructor::mutate_unique_io`（FFI 資源の複製判断）・(3) `assert_unique`（デバッグ、assert ビルド §1.6）は配列/struct ホットパス外の正当用途で残す。削除は不適——(1) は op の途中でユーザ closure を呼ぶため単一 op に畳めないうえ、**is_unique 分岐が lazy clone を可能にしている**（shared 枝の clone は `set` 内＝f が成功したときだけ走り、**失敗系 functor で f がエラー抜けすれば clone ゼロ**／unique 枝は move-out で clone 無し）。これを `punch_fu` へ一律置換すると punch 時点で **eager clone** になり失敗系 monad で回帰するので**しない**。ホットな act は optimize_act が特殊化するので generic 分岐は非ホット fallback＝lazy clone を保つのが正（静的 unique の場合の check 除去は将来 §6 の「is_unique 静的解決」で両取り）。P1 監査: `unsafe_is_unique` の `result_prov` を passthrough（`Arg(0)`）と宣言して `Fresh` を保つ（精度）。unique-check-elim 実行時は act 最適化を有効にしておく。
- **（確認済み）高階イテレータの直接化は既存 decapturing に依存**: `loop`/`fold` 等は body を closure 引数で受け、その内部呼び出しは間接。uniqueness 特殊化（§4）が内部の force-unique op に届き、Provenance 解析（§3）が精度を出すには、この間接呼び出しが直接化されている必要がある。既存の **decapturing の closure specialization**（`src/optimization/decapturing.rs`、`enable_decapturing_optimization`。`inline` 後・`uncurry` 前＝RC IR lowering の前）が、body を焼き込んだ特殊版（`loop#lam` 等）を生成し内部呼び出しを直接化する（`pull_let` が適用範囲を広げる）。ベンチの `loop((0,arr), |…|)` は適用対象（lambda を直接渡す・自己呼び出しで同 index）。適用外の形（doc の制限: lambda をタプル等に入れて渡す等）は間接のまま残り、§3.2 の規則で結果 Dyn＝除去は効かない（健全・許容）。unique-check-elim を回すときは decapturing を有効にしておくのが前提。
- **（確認済み）`fix`（ローカル再帰の不動点コンビネータ）は RC IR で表現可能**: std `fix = |f| |x| FixBody`（`FixBody` は InlineLLVM、free vars `x,f,cap`）。lift で outer `|f|`／inner `|x|` の `RcFunc` になり、`fix(f)`=`Closure(inner,[f])`、本体は `LLVM(FixBody, [x,f,cap])`（全 `Own`）。FixBody は自己 funptr（codegen の `get_parent`）＋現 cap 再利用で `fixf=fix(f)` を作り `f(fixf)(x)` を呼ぶ（heap alloc なし、RC cycle 無し＝fixf→f だが f→fixf 無し）。内部 RC は宣言で残す（opaque・`Borrow` 化不可）ので fix 内再帰は解析から保守的に見える。`Closure(self)+App` への desugar は避ける（cap 再利用を失うため）。

## 9. フェーズ別テスト計画（検証入力）
各フェーズが「想定どおり動く」ことを、どの入力コードで・何を観測して確認するか（§7 のマイルストーンに対応）。統合テストは CLAUDE.md 規約（`main` から実際に実行・参照、tempdir へコピー、`Command::new("fix")` で `fix` 実行）で書く。

### 9.0 共通の検証基盤（複数フェーズで使う。P0/P1 で先に整える）
- **shared-value テスト（最重要の正しさパターン）**: 値を 2 箇所に格納（＝別名を作り共有に）→ 一方を in-place 系 op で mutate → **他方が壊れていないこと**を assert。`let a = [1,2,3]; let keep = (a, a); let a2 = a.set(0, 99); eval (keep.@0.@0, a2.@0)` が `(1, 99)`。clone-on-shared の健全性を突く（`set`/`mod`/`act`/`swap`/struct 系 全部に効く）。
- **assert ビルド（健全性）**: 「unique と判定した値が実行時に共有なら abort」するビルドモード。全テストをこれで走らせ、uniqueness 判定の誤りをゼロ検出で確認（§1.6/§5）。
- **除去の観測（IR/asm チェック）**: 一意文脈で `build_branch_by_is_unique` 由来の分岐が emit IR/asm から**消えている**こと、共有文脈で**残っている**ことを grep で確認。`fix` に IR/asm ダンプ経路が要る（既存の emit を利用）。
- **leak チェック（valgrind memcheck）**: `valgrind --leak-check=full` で RC バグ（漏れ・二重解放）を検出。**注意（要考慮）**: valgrind は exit 時のメモリを分類し、**Fix の global 値（`mark_global`）や global から到達可能な値は "still reachable" ＝ "definitely lost" に計上されない**——so 漏れたオブジェクトが global 到達可能だと "definitely lost" では捕まらない。→ リークテストは (a) 漏れる対象を **global に載せず**ローカルに閉じる（関数終了で本来解放されるべき形にする）、(b) 漏らす操作を**ループで反復**して definitely-lost を積み上げる／RSS 一定を確認、(c) 必要なら alloc/free カウントで補う。以下で「leak チェック」はこの手順を指す。
- **provenance/ownership ダンプ（P2 の照合用）**: 各関数の結果 `Provenance`・各変数末端の由来・各引数の `OwnershipShape` を出す debug 出力（`optimize_act` 等が使う `emit_symbols` と同様の仕組み）。P2 はこれで期待値照合する（この経路を作るのが P2 の前提作業）。
- **cachegrind 計測**: `fix-bench/batch/arrayrw{,_unsafe,_fn}`・`fannkuch`・ソートを commit hash 付きで instruction 数記録（§0 の目安表と比較）。
- **leak/double-free**: valgrind か assert ビルドで、boxed 要素の `mod`/`act`/`swap`・深い再帰・クロージャ捕捉を回すストレス。
- **RC IR validator（[#14]・debug ビルド限定の整形性検査）**: RC IR を書き換える全パス（§2.1 phase-2 patch・§2.2 相殺・§4.2 差し替え・将来 §6）の**直後**に走る `validate(RcProgram)`（debug ビルドのみ・release では無効）。検査: (i) 束縛名のグローバル一意（§1.1-3）、(ii) use-after-consume が無い（`Own` 位置で消費した変数がその後現れない）、(iii) 関数ごと・root ごとの**参照収支**（param の Own/Borrow・`Retain`・`Release`・consume の経路和が 0）、(iv) `Closure` 捕捉順と lifted 関数の射影順の一致（§1.2 不変条件）。(iii) は [#F1] のような patch の収支割れ（孤児 `Retain`＝over-retain リーク）を実装直後の任意入力で捕まえる——uniqueness assert ビルドは「unique 誤判定」しか見ず over-retain は見ない・valgrind は global 到達漏れを見逃す、という穴を埋める。P1 で作り以降の全パス後に常時走らせる。

### 9.1 P0（デバッグ情報ベースライン）
テスト自体が成果物。入力＝行構造が既知の小プログラム、`fix build -g` → `gdb -batch`（`break main.fix:N` → run → backtrace）。file:line 解決・停止・スタックの行情報（マングル名非依存）を assert。**現 main で通す**（P1 の比較基準）。

### 9.2 P0.5（Bool→union）
Bool の挙動・ビットが不変であることを突く。入力（`main` から eval して結果 assert）:
- 比較: `3 == 3`, `5 < 2`, `2 <= 2`, `1.0 < 2.0`。`&&`/`||`/`not`: `true && false`, `true || false`, `not(true)`。
- 分岐: `if b {..} else {..}`・`match b { true() => .. }`。
- FFI: Bool を i8 として C 関数へ渡す/受ける（tag `_false`=0/`_true`=1）。
- **debug 情報の Bool 不変（[A]）**: union 化後も Bool の debug 型が `DW_ATE_BOOLEAN`（union struct でない）＝§9.1 の P0 ベースラインと一致（`is_boolean()` が Bool-union tycon を認識）。gdb/llvm-dwarfdump で確認。
- **`not` が branchless（[A]）**: `not(b)`（`!b`）の emit asm/IR に**分岐が無い**（`icmp`/`sete` 相当のみ・`br` 無し）＝`Bool : Not` インスタンスが InlineLLVM 本体のまま（Match に脱糖されていない）。比較演算子も同様に branchless。
観測: 実行結果一致＋既存全テスト通過。性能中立（i8 のまま）。

### 9.3 P0.7（builtin PunchedArray + builtin swap）
`mod`/`act`/`swap` の正しさ・PunchedArray の drop 安全・リーク無しを突く。入力:
- 正しさ（unboxed/boxed 要素）: `[1,2,3].mod(1, |x| x+10)` == `[1,12,3]`、`[[1],[2]].mod(0, |x| x.push(9))`、`[1,2,3,4].swap(0,3)` == `[4,2,3,1]`、ソート結果。
- **shared-value**: 上記 9.0 の型を `mod`/`swap` でも（`keep` 側が不変であること）。
- **PunchedArray drop 安全（skip-idx release を突く核）**: 失敗し得る functor の generic `act`（`[[1],[2]].act(0, |_| Option::none())`）で **plug されず PunchedArray が drop** される経路 → リーク/二重解放なし（valgrind/assert）。
- **汎用 act の多重 plug（[1]・共有 PunchedArray で fu plug が走る核）**: **多要素 functor を返す** generic `act`——例 `[1,2,3].act(0, |x| [x+10, x+20])`（F=Array で 2 要素 → `map` が plug を 2 回呼ぶ）== `[[11,2,3],[21,2,3]]`。結果の各 Array が**独立**（idx を互いに汚さない＝共有 punched の force-unique〔skip-idx clone〕が走った証拠）。非 fu 実装だと両者が同一 buffer を alias して `[[21,2,3],[21,2,3]]`＝腐敗・二重解放になる。リーク/二重解放なし（valgrind）。
- **`pop_back` builtin（[#12]・shrink ＋末端の捨て安全）**: `pop_back : Array a -> Array a`（std.fix:362-367。**要素は返さず捨てる**——末尾を境界外に出し drop）を専用 builtin 化（[#R10-5]: shrink＋末端の解放を op 内に閉じ込め）するので、**boxed 要素**で突く: `[[1],[2],[3]].pop_back` == `[[1],[2]]` を unique 文脈（in-place shrink＋末端 drop）と shared 文脈（clone パスで捨て要素の参照収支が合う）で。観測: 残り配列の正しさ＋**捨てられる末尾要素のリーク/二重解放なし**（§9.0 leak・boxed 要素で valgrind 必須）。shrink 後の旧末端二重解放・clone パスの参照収支ずれは valgrind でのみ出る。
- **PunchedArray の mark skip-idx（[#13]・穴スロットを辿らない）**: `mark_threaded` の graph walk が PunchedArray の穴 idx（stale ポインタ）をスキップすること（穴を辿ると解放済み/無効 object の state を書きに行き UAF）。**まず Fix ソースから mark 経路（plug 前の PunchedArray を捕捉した closure が `mark_threaded` される文脈）を構成できるか確認**し、構成できれば統合テスト（valgrind/assert で UAF なし）、**構成不能なら P0.7 の項にその論証を書き、traverser 単体（Rust レベル）の mark work で skip-idx を検査する**テストで代替する（実装者に「存在し得ない Fix コードを書け」と求めない）。
観測: 実行結果＋shared-value＋leak。ソートベンチ（§4 前なので劣化は想定内＝§7 の注意書きどおり振り分け）。

### 9.4 P1（RC IR + lowering + codegen 付け替え）
挙動保存（無回帰）の**最大の検証**。入力＝**既存全テスト＋全ベンチ**（§1.6）＋ RC ストレス（boxed 捕捉クロージャ・深い再帰・ループ内 boxed・共有構造）。観測: `cargo test --release` を `FIX_MAX_OPT_LEVEL` max/basic/none で全通過／全ベンチ commit hash 比較で無回帰／9.1 の gdb テストでデバッグ情報一致／leak チェック／RC 挿入数・順序・解放挙動が現状一致（assert ビルド or RC ダンプ）。
- **tail 制御（tail-call elim 維持）の検証**（新 IR で tail 先読み `tail_of` の判定・phi 回避が壊れていないこと。§1.2 tail 先読み）:
  - **深い末尾再帰が stack overflow しない**: match arm が tail position の再帰（`sum`〔§2.1〕型や `loop`/`fold`）を大 N（例 1e8）で走らせ完走する＝各 tail 呼び出しが実際に tail-call elim されている。arm 内呼び出しでも、その match が tail position なら tail になること（`Let(x, Match{…App…}, Ret(x))` 型）を含める。**非 `-g` ビルドで走らせる**——`-g` は `set_tail_call` を付けない（generator.rs:1010）ので `tail` マーカー無しで overflow し得る（`-g` 時に overflow するのは既存挙動で回帰でない）。
  - **tail position の match が phi を生成しない**: 出力 asm/IR で、tail match の各 arm が `tail call`＋`ret`（merge/phi 無し）になっていることを確認（`call` の直後が `ret`）。複数 arm でも各 arm 独立に tail。
  - **非 tail の match は phi があってよい**: 結果を束縛して後続で使う形（`Let(x, Match, k)`、k が `Ret(Var(x))` でなく x を後で使用）は merge/phi でよく、arm 内呼び出しは tail でない——これも意図どおりであることを確認（tail 判定が rename 追従の先読み `tail_of`〔`let x=cexp; Ret(Var(x))`〕であって「直後に Ret」の flat 判定でないことの裏取り。`let z=Match; let w=Var(z); Ret(Var(w))` は tail・`let x=Match; …k で x 使用` は非 tail）。
  - **`tail_of` の負例（過大近似で誤コンパイルしない裏取り・`is_tail_cont` の `_ => false`）**: (a) `let r=App(g,[x]); eval assert(cond); Ret(Var(r))`（App と Ret の間に非 rename の文）、(b) `let z=Match(…); let n=z.@size; Ret(Var(z))`（match 結果 z を挟んで使用）が **非 tail** と判定されること——具体的に (1) codegen が継続（assert / `@size`）を落とさず emit する（tail 誤判定だと継続コードが消える誤コンパイル）、(2) routing が後続 `Release` を挿さない、(3) match は merge/phi を作る——を IR/実行で確認。`is_tail_cont` が rename 以外の文（実 op）を跨がず `false` を返すことの裏取り。

### 9.5 P2（borrow化 + 相殺 + provenance 解析）
解析が期待どおりの値を出すことを 9.0 の provenance/ownership ダンプで照合。入力と期待:
- `|x| let y = [x]; (y, y)` → 両末端 `(Dyn, Dyn)`（複製 Retain。§5）。`id(x)` 結果 `Arg(0,[])`。`set(i,v,arr)` 結果 `Fresh`。
- **多末端 sibling 独立（[3]・per-末端降格）**: `(arr1, arr2)` で arr1 だけ共有（片末端に `Retain`）→ **arr1=`Dyn`・arr2=`Fresh`**、後続 `arr2.set(..)` が elide される。全末端降格の実装だと arr2 も `Dyn` になり elide が落ちる＝回帰検出。
- **unboxed aggregate の別名降格（[4]・root ベース）**: `let t=([1,2,3],[4,5,6]); let y=t.@0; let t2=t.mod0(|a| a.set(0,99)); eval (y.@(0), t2.@0.@(0))` == `(1, 99)`。`y` が `t.@0` を共有するので `mod0` 内の `a.set` は clone すべき（in-place だと y が [99,2,3] に化ける）。**変数キー降格の実装だと破壊**（`Retain(y)` が `env[t]@[0]` を漏らし `a=t.@0` が Fresh→elide）＝root ベース（§3.2 の `Retain` 規則）の回帰検出。boxed 版（`Pair` box struct）は getter が Dyn を返すので元々壊れない（対照）。
- **片枝でのみ生じた Dyn を合流後へ持ち越す（[#8]・非 tail Match の env-join）**: **片方の arm でだけ別名/threaded を作り、合流後（k）に mutate** する入力。例: `let a = fill(3,0); let keep = if flag { [a] } else { [] }; let a2 = a.set(0,99); …`（`if` 脱糖で非 tail `Let(keep, Match, k)`。_true arm は `[a]` 構築で `Retain(a)`→`env[a]={Dyn}`、_false arm は a 不使用で `{Fresh}`）。観測: **合流後の `env[a]` が `Dynamic`**（provenance ダンプ）＝`join(Dyn, Fresh)` を全生存変数末端に pointwise 適用したこと。**結果値 keep の join だけ行い生存変数 a の env join を怠る実装**だと `env[a]` が `Fresh` のままになり、後続 `set` が誤 elide される（§9.6 で実行時破壊として検出）。(i) `Retain` 降格版と (ii) 片枝でだけ `mark_threaded` する版、boxed/unboxed 両 scrutinee で。
- **union-path RC ノードが非 active variant で壊れない（[#F1]・tag 安全）**: `o : Option (Array I64)` を分岐で作り（`if flag { some(fill(3,0)) } else { none() }`）**多重使用**（`let pre = g(o); match o { some(arr)=>…, none()=>… }`、`g` は borrow）し、**非 payload variant（none）で実行**（flag=false）＋ valgrind/ASan。**分解が union をまたいで `Retain(o, some::[])` を無条件 retain する実装**だと、none 経路で未初期化 payload スロット経由の refcount++ ＝ heap 破壊。**union で止める（tag 分岐）実装**なら none 経路は no-op で無害。§9 の他入力は「非 active variant 上の分解済み union-path RC ノード」を踏まないので、これが無いと F1 は silent。payload が boxed の union（`Option (Array I64)`）と unboxed の union（`Result I64 I64` 等）両方で。
- §3.4 arrayrw ループ → `loop` 結果 `{Fresh, Arg(0,[1])}`、`main` で `{Fresh}`＝Unique。§3.5 read-only `sum` → `arr` は `Arg` 素通し、`main` の set 前 `Fresh`＝Unique。同じ配列を共有してから渡す版 → `Dynamic`。
- borrow化: read-only 再帰 `sum` → `sum_borrow.arr = Borrow`（§2.1）・main の非 last-use 呼びが `sum_borrow` へ routing。`loop_fresh` の fresh tail 再帰は `loop_fresh_own` へ routing（`loop_fresh_borrow` は dead-func で刈られる）。相殺 → 借用呼び出しをまたぐ `Retain … Release` が消える。
- **borrow 降格 param の孤児 Retain 除去（[F1]・リーク検証）**: **borrow 化された param を非 last-use で borrow callee へ複数回渡す**関数——`sum2 = |arr| sum(arr,0,0) + sum(arr,0,0)`（`sum` は read-only 再帰・borrow 化される）——を大 N 反復で呼ぶ。lowering は 1 回目（非 last-use）の前に `Retain(arr)` を入れるが、`sum.arr=Borrow` ゆえ両 App は非消費で対応 Release が無い。観測: (a) **リーク無し**（§9.0 leak チェック。`arr` は global に載せずローカルに閉じ・反復で RSS 一定）＝phase-2 の remove が孤児 `Retain(arr)` を消したこと、(b) ownership ダンプで `sum2.arr=Borrow`、(c) IR ダンプに孤児 `Retain(arr)` が**残っていない**。remove が Release しか消さない実装（回帰）だと (a) の RSS が反復で線形増加で落ちる。多末端版（`|t| use(t.@0)+use(t.@0)` で `t=(borrow, own)` 混在）も 1 本。
- **相殺の first-Release ペアリング（[#F5]・二対の誤ペアで early-free UAF）**: 同一 object に **Retain/Release の対が 2 つ**乗る直線コード——`let arr = fill(…); let s1 = sum(arr,0,0); let s2 = sum(arr,0,0); arr.set(0, s1+s2)`（borrow 化後 `Retain; call; Release; Retain; call; Release; set(consume)`）——を valgrind/ASan で。**Retain#1 を Release#2 とペアにして消す実装**は `fill(rc1); call; Release(rc0＝解放); Retain(解放済み); call; set` ＝ **early-free UAF** だが**収支は合う**（§9.0 validator の参照収支 (iii) も uniqueness assert も素通り）ので sanitizer でのみ捕まる。加えて IR ダンプで「相殺後に残る各 Release がその object の最後の借用使用を post-dominate する」ことを確認。boxed 要素だと Retain/Release が非自明で効く。
- **global 初期化子も §2/§9 の対象（[#F8]・リーク検証）**: `RcGlobalInit` の body（param 無し関数と同扱い）が借用化された read-only 関数を呼ぶ入力（例: `g_arr : Array I64 = eval sum(fill(100,0), 0, 0) |> _ -> [];` 型で init 内にローカル配列と借用呼びを作る）→ init 内の借用呼びの後に caller-side Release が挿入され**ローカル配列がリークしない**（§9.0 leak・実行時 1 回だが assert/§9.0 validator で収支検査）。§2.1/§2.2 が globals を素通りする実装だと init 内の借用降格 callee が under-release でリーク。
- **混在 tail を `g_own` へ渡す借用引数の Retain（[#F1round8]・double-free 検証）**: `f_borrow` が tail で**借用値と所有値を一緒に** `g_own` へ渡す入力——`g = |a,b| a.@size + b.@size; f = |arr| g(arr, fill(3,0)); main = (let arr=fill(100,0); let s=f(arr); arr.set(0,s))`。観測: (a) **`arr` が二重解放/UAF しない**（valgrind/ASan）＝step 3-(ii-c) が `g_own` 呼び出し前に `Retain(arr)` を挿し `g_own` の release が `arr` を消さないこと、(b) IR ダンプに当該 `Retain(arr)` が存在。**Retain を欠く実装だと `g_own` が借用 `arr` を release → main の `set` が解放済みを触る**。返すケース（`g = |a,b| a`・`g_own` は `a` を release せず escape）も 1 本——同じ `Retain` で main が正しく 2 参照（rc=2）を得る（Retain 無しだと double-free）。
- **混在 shape の `g_borrow` の Own 位置への Release（[#1round9]・double-free 検証）**: **消費 param ＋ 読み取り param** の callee へ、**Own 位置に所有値・Borrow 位置に所有非 last-use 値**を渡す非 tail 呼び——`g = |a,b| (let s = b.@size; a.set(0,s))`（不動点で `g_borrow=(a:Own, b:Borrow)`）／`main = (let x=fill(10,0); let y=fill(5,0); let r=g(x,y); let t=y.@(0); …)`（y が得を供給し call は `g_borrow` へ・x は last-use で Own 位置 a へ）。観測: (a) **`x` が二重解放/UAF しない**（valgrind/ASan）＝step 3-(ii) が **Own 位置 a には Release を挿さない**（`own[g_borrow.a]==Own`）こと、(b) Borrow 位置 b には `Release(y)` が入り `Retain(y)` と相殺すること（ownership ダンプ＋IR grep）。**位置を見ずに「所有値なら全部 Release」する実装（round-8 の穴）だと、`g_borrow` が consume 済みの `x` を caller が再 release ＝ double-free**。§9.5 の他 borrow テストは全 param 借用可能でこの混在 shape を踏まない。
- **捕捉ありクロージャを借用系呼びに流す（[#2round9]・cap 二重解放検証）**: **クロージャを呼ぶだけの関数**へ**非 null cap のクロージャ**を渡す入力——`apply = |g, x| g(x); main = (let c=[1,2,3]; let g0 = |i| c.@(i); let arr=fill(10,0); let s=apply(g0, arr); arr.set(0,s))`（`g0` が `c` を捕捉＝cap 非 null）。観測: **`g0` の cap が二重解放しない**（valgrind/ASan）＝`consume_sites` が `App` の callee を数えて **`apply.g=Own`** になり、routing が Own 位置に所有 `g0` を渡して Release を挿さず、内部呼びの cap release 1 回だけになること（ownership ダンプで `apply.g=Own`・IR に caller 側 `Release(g0)` が**無い**）。**callee を数えない実装だと `apply.g=Borrow` 誤分類 → caller の挿入 `Release(g0)` が内部 release 済み cap を再解放 ＝ double-free**。§9 の他 closure テストは全て空捕捉（cap=null）でこれを踏まない。クロージャを 2 回呼ぶ版（`|g| (g(1), g(2))`・使用前 `Retain(g)` が各 callee-release と釣り合う）も 1 本。
- **コンテナ取り出し要素の retain-getter 形（[#R10-1round10]・間接デクリメント UAF 回帰）**: 取り出した要素の **last use が borrow-read**（read して捨てる）形を、(a) 捕捉値を read するクロージャを実際に呼ぶ（`g0 = |i| c.@(i)` の lifted body が `retain_get(cap); Release(cap); read; Release`）、(b) owned boxed-union の payload を match して read-drop（`match o { some(a) => a.get_size, none() => 0 }`、o は boxed union で match が last use）、で valgrind/ASan。観測: **UAF なし**＝取り出しが retain-getter（要素 retain を op 内）で lower され、`Release(cap)`/`Release(容器)` の間接デクリメントで要素が早期解放されないこと。**no-retain 取り出し＋別ノード明示 Retain で lower し §2.2 が Retain/Release を対消滅させる実装だと、コンテナ release が要素を道連れに解放して read が UAF**（§2.2「相殺は常に健全」の反例を retain-getter 形が構造的に防ぐ）。
- **union の borrow param の別名 Release 除去（[#6round9]・phase(i) の root ベース削除）**: **union 型の borrow param** を match して payload を read して捨てる関数を borrow 化し、caller が同じ値を後段 mutate する入力。(a) unboxed union（`Option (Array I64)` 等）——arm 内で payload を取り出す `Release`（root=param@(k::π)）を phase(i) が **root ベースで削除**すること、(b) boxed union——arm 内の容器 `Release(o)`（root=param）が削除され caller の (ii) Release と釣り合うこと。観測: **UAF/二重解放なし**（valgrind/ASan）。**変数名マッチで実装すると、別名の `Release(payload)`/`Release(o)` が残って過剰解放 → caller UAF**（§2.1 の phase(i) remove が明記する誤実装）。§9.5 の他 borrow テストは union の borrow param を含まずこれを踏まない。
- **unboxed union payload を match 結果で return する param の Own 化（[#R11-2]・arm Ret 消費 → caller UAF 回帰）**: **unboxed union の borrow 候補 param の payload（boxed）を、value match の arm 値として return** する関数を、caller が同じ union を**後段でも使い**（非 last-use）呼ぶ入力——`first_or : Option (Array I64) -> Array I64 -> Array I64 = |o, d| match o { some(a) => a, none() => d }` ／ `main = (let arr = fill(3,7); let o = some(arr); let r = first_or(o, empty(0)); let s = r.@(0); eval o.is_some; let t = r.@(1); …)`。観測: **UAF/二重解放なし**（valgrind/ASan）＝arm の `Ret(a)` を consume に数え `root(a)=o@(some::[])` で **`first_or.o=Own`** になり、payload move-out が正しく所有移譲されること（ownership ダンプで `first_or.o=Own`）。**arm Ret を consume に数えない（関数末尾 Ret しか見ない）実装だと `first_or.o=Borrow` 誤分類 → first_or_borrow が retain せず payload を返す → caller の `Release(o)` が payload を早期解放 → `r.@(1)` が UAF**。tail match（上）と、payload を一旦束ねてから返す非 tail match（`let z = match o {…}; some_use(z); z`）の両方で。
- **§2.2 の union payload consume 跨ぎ誤相殺（[#R12-1]・clamp_unit・load-bearing Retain の誤削除 → UAF 回帰）**: **unboxed union `o` を 2 回使う**（payload を match で move-out ＋ 借用呼びで read）ので lowering が load-bearing な `Retain(o,[])` を置く形——`g2 = |o, w| (match o { some(a)=>a.@size, none(_)=>0 }) + w.@size`（read-only → g2_borrow）／`f = |c| (let o = if c { some(fill(3,0)) } else { none() }; let z = match o { some(a)=>a, none(_)=>empty(0) }; let zz=[z]; let w=fill(4,0); let s=g2(o,w); let t=w.@(0); …)`——を **c=true** で valgrind/ASan。観測: **UAF なし**＝§2.2 が payload consume（キー `o@some::[]`）を **`clamp_unit` で `o@[]` に揃え**、whole-union `Retain(o,[])` を `needed` にして**誤相殺しない**こと（IR ダンプで `Retain(o,[])` が相殺後も残存）。**clamp しない実装だと consume が別バケットで見えず `Retain(o,[])`/`Release(o,[])` を対消滅 → z への move-out 後に g2 が解放間際の payload を read = UAF**（§2.2「相殺は無条件に健全」の反例）。§9.5 の他 union テストは「2 回使用＋payload move-out＋借用呼び」の組を踏まない。
- **非 tail match arm 内 App への Release 配置（[#F1round10]・[#3round9] を突く）**: **非 tail match の arm 末尾が borrow-routed App（`Let(t, App, Ret(Var(t)))`）** になる入力——`let r = if c { g(x) } else { 0 }; x.set(0, r)`（`g` は read-only で borrow 化・`x` は k で使用＝所有非 last-use → arm 内 App は非 tail・benefit あり → `g_borrow`）——を **c=true/false 両方**で valgrind/ASan 実行＋「Release が _true arm 内〔`Ret(Var(t))` の直前〕にある」IR grep。**Release を合流後 k（`Ret(Var(z))`）の先頭に置く誤実装**は c=false 経路（この経路に lowering Retain 無し）で `x` を早期解放 → `x.set` が UAF。c=true だけ踏むテストは Retain があってサイレント通過。
- **同一値の同一呼び内複数渡し（[#F2round10]・step-3 (ii) の per-出現 RC）**: 収支は「lowering が**出現ごと**に Retain・(ii) が**位置ごと**に Release/Retain」で初めて合う。(a) 非 tail: `g2 = |a,b| a.@size + b.@size`（両 Borrow）へ `let s=g2(x,x); x.set(0,s)`——(ii) が **Release x2** で rc が合う（1 個に dedup すると毎呼び +1 リーク・RSS 反復増）。(b) tail 混在: `f_borrow`（p Borrow）の `let r=App(g,[p,p,fresh]); Ret(Var(r))`（tail＋所有 fresh → `g_own`）——(ii) が **Retain(p) x2**（q=0,1 とも Own∧¬owns）で `g_own` の 2 param 独立 release と釣り合う（1 個に dedup すると過剰解放＝UAF）。valgrind＋IR grep（Release/Retain の個数）。[#F1round8]/[#10] は 1 回しか渡さずこれを踏まない。
- **`f_own` body の得あり呼びも RC 書き換えを受ける（[#F2]・リーク検証）**: escape-reachable な関数の body が**所有非 last-use 値を借用 callee へ渡す**入力——`h = |arr| (let s = sum(arr,0,0); arr.set(0,s))`（`sum` borrow 化）を値として間接呼びし（`h_own` が実行される）大 N 反復。観測: **リーク無し**（§9.0 leak・RSS 一定）＝`h_own`（f_own 版）でも `sum` 呼びが `sum_borrow` へ routing され**後続 Release が挿入**され、baseline の非 last-use 用 `Retain(arr)` と net-zero 相殺すること。**RC 書き換えを `f_borrow` だけに適用する実装（Reading A）だと `h_own` の Retain が孤児化して毎回 +1 リーク**。加えて `main`（param 無し＝f_own）の `sum` 呼びが `sum_borrow` へ行き §3.5 の elision が成立することも（Reading B の旗艦例失敗を検出）。
- **混在多引数 `f_borrow` 呼びの no-benefit 引数の Release（[#F5b]・リーク検証。[#F6round10]: 上の first-Release ペアリング [#F5] とは別物・id 改番）**: 1 つの call で「得あり引数（所有非 last-use）」と「得なし引数（**所有 last-use**）」が同居する入力——`let x=fill(…); let y=fill(…); let s=g2(x,y); x.set(0,s)`（`g2` は両引数を読むだけ・`y` はこの call が last-use）。call 全体が `g2_borrow` へ行き、**`y` の後続 `Release` は `y` の唯一の drop**（手前に Retain 無し・相殺相手無し）。観測: **`y` がリークしない**（valgrind・RSS 一定）＝step 3-(ii-b) が全所有引数に Release を挿すこと。**「routing を動機づけた引数だけ release する」誤実装だと `y` が漏れる**。ownership ダンプで `g2_borrow` 選択・IR grep で `Release(y)`。
- **escape-reachable ＋借用 callee の 2 版 routing（[1]/[F2]・リーク＆overflow 検証）**: escape-reachable な関数を**直接と間接の両方**から呼び、しかも body が**借用化される source 関数を呼ぶ**入力。例: `g = |arr| sum(arr, 0, 0)`（`sum` は read-only 再帰・借用化）を specializable な `loop`/`fold` に渡し（decapturing が直接化）＋値としても間接呼び（`let fs = [g]; fs.@(0)(arr)`）。観測: (a) **リーク／二重解放なし**（§9.0 leak・漏れ候補は global に載せずローカル・大 N 反復で RSS 一定）——間接呼びは `g_own` へ routing され、`g_own` は所有 `arr` を **`sum` の Own 版へ**渡して consume する（[F2] の穴を突く: `g_own` が借用 `sum` を呼ぶと `arr` がリーク or 非末尾化）、(b) **間接サイクルで閉じる形＋caller 所有 fresh を tail で渡す**（§9.6 [#2] の go/probe を read-only ラッパ経由にした版）を大 N で**完走**（overflow なし）、(c) IR/ownership ダンプで「間接呼び地点＝`g_own`・直接非 last-use 呼び＝`g_borrow`」の routing と両版の RC 整合を確認。**builtin だけ呼ぶラッパ（`|x| x.@size`）では [F2] を突かない**ので、借用 source 関数を呼ぶラッパにすること。escape しない直接専用関数だけが in-place borrow 化されることも確認。
- **条件付き素通し op の consume（[2]・二重解放検証）**: boxed `union mod`（`Result::mod_ok` 等、結果由来 `{Fresh, Arg}`）へ、**borrow 化され得る source param の union** を流す入力（match 枝＝op が旧 union を release、mismatch 枝＝素通し）を、match 枝・mismatch 枝の両方が走るように与える。観測: **二重解放／UAF なし**（§9.0 leak チェック＋sanitizer/assert）＝union param が `root`=producer 判定で **`Own` に降格**していること（ownership ダンプ）。加えて §2.2 で union mod をまたぐ `Retain … Release` が net-zero 誤除去されないこと。
- **mark_threaded の operand が Own（[5]・「is_unique⟹LOCAL」補題）**: param を `mark_threaded` する関数（`mark_it = |x| eval x.mark_threaded` 型）に配列を渡し、その後同じ配列を `set` する入力 → **set が elide されず checked**（mark_it へ Own で渡り caller が dual-use `Retain`→`Dyn`）。mark_it の param が Borrow に降格していない（ownership ダンプ）。assert ビルドで「unique 判定した値が実は threaded」を検出しない（threaded object を素で in-place しない）。
- **分岐 consume の相殺は枝ローカルで消さない（[#9]・★ の UAF）**: `Retain(x)` の後に**片枝で `Release(x)`（drop）・他枝で `x` を `Own` consume**（構築/escape へ渡す）に lower される入力——例 `|c, arr| if c { arr.@size } else { escape(arr) }`（`escape` は arr を Own 消費・`@size` 枝は borrow 後 drop、呼び出し前に非 last-use 用 `Retain(arr)`）——を **c=true/false の両方**で走らせる。観測: **UAF／二重解放なし**（§9.0 leak＋sanitizer/assert）、かつ相殺後も当該 `Retain` が **IR に残っている**（相殺ダンプ／IR grep）。**枝ローカルで `Retain`＋A 枝 `Release` を消す実装**（§2.2 ★）だと、consume 枝（c=false）で唯一参照を渡した後に caller が解放済みを触る＝UAF——**Release 枝（c=true）だけ踏むテストはサイレントに通る**ので両枝を必ず走らせる。
- **多重出現の一様扱い（[#10]・§2.1 の要検証を §9 へ転記）**: 同一値が複数箇所に出る形を stress-test。(a) **tail の fresh 二重渡し** `g(x,x)`（x は fresh）を**大 N の深い末尾再帰**で完走（＝所有引数の tail 呼びが routing で `g_own` へ行き全出現 consume＝tail 保持・leak なし・どちらへ routing されたか IR で確認）——所有 tail を `g_borrow` に回す実装は後続 Release で非末尾化し深い再帰で overflow。(b) `let y=(x,x); g(y)`（unboxed 集約末端の複製）・`[x,x]` の ownership ダンプ照合＋shared-value。(c) 入れ子集約 `((x,x), x)`。(d) **条件付き素通し op**（boxed `union mod` の結果 `{Fresh, Arg}`・複数 join）が `root`=producer で consume 判定に入り、流入 param が `Borrow` のまま残らない（match 枝 release で二重解放しない）ことも [2] とは別に単独で。

### 9.6 P3（unique-check-elim + 特殊化 + dead-func 除去）
§5 の全項目。入力と観測:
- **一意文脈で除去**: arrayrw ループ（threaded unique array の `set`/`mod`/`swap`）→ 分岐消滅（IR grep or cachegrind 減）。§5 マトリクス全セル（Array の set/mod/act(Id)/act(Tuple2)/swap、boxed struct field の set/mod/act）。
- **`is_unique ⟹ LOCAL` の実マルチスレッド E2E（[#F4]・データ競合検証）**: 「threaded な値は必ず `Dyn`」という健全性簡約を、**実際にスレッド境界をまたぐ**入力で突く（[5] は単一スレッドで param の Own 性しか見ない）。(a) 配列をスレッド API で送り、**受信側が mutate する間に送信側が自分のコピーを保持/read** する → 送信側のコピーが**壊れない**（assert ビルド＋valgrind drd/helgrind or ASan）。(b) 「**初回 checked・以降 unchecked**」: threaded array を受け取り初回 `set` で LOCAL 化（`build_branch_by_is_unique` の THREADED∧rc==1→acquire fence→`mark_local_one`〔generator.rs:816-858〕）してループ → IR で初回だけ checked・以降 unchecked。**回帰**: どこか（将来 op の `result_prov` が誤って `Fresh`・`mark_threaded` へ流れる末端の Retain 降格漏れ・`boxed_from_retained_ptr` 誤宣言）で threaded object の check を誤 elide すると、他スレッドから見える object を素で in-place ＝サイレントなデータ競合/破壊——どの §9 単一スレッド入力でも起こせないのでこれが必須。ランタイム側の LOCAL 降格は既に真（generator.rs 確認済み）なので、突くのは**解析側の不変条件**。
- **共有文脈で非除去**: 9.0 shared-value → 分岐残存（IR grep）＋他方不変。
- **分岐版 shared-value（[#8]・非 tail Match の env-join）**: 片枝でだけ別名/threaded を作り合流後に mutate する入力（§9.5 [#8] と同じ `let a=fill(3,0); let keep = if flag {[a]} else {[]}; let a2 = a.set(0,99); …`）を **flag=true/false の両方**で走らせる。観測: (a) **flag=true で `keep` の中身が不変**（in-place set が起きていない＝shared-value 破壊なし）、(b) `set` の unique 分岐が **IR に残存**（elide されていない）。**env-join を怠る実装**は合流後 a を Unique と誤判定して set を elide し、flag=true で `keep` を破壊する——**flag=false だけ踏むテストはサイレントに通る**ので両分岐を必ず走らせる。(i) `Retain` 別名版・(ii) 片枝 `mark_threaded` 版、boxed/unboxed scrutinee で。
- **初回 checked・以降 unchecked**: shared で入り初回 `set` で unique 化するループ → `loop@D -> loop@U` の 2 clone（IR に 2 clone、cachegrind で初回だけ高コスト）。
- **入れ子伝播**（§5）: タプル内配列 `loop((cnt, arr), …)`・struct 内配列・配列内 struct・union 内配列（`LoopState`）。
- **dead-func 除去**: 特殊化後に未到達 clone・元関数が消える（emit symbol 数／バイナリサイズが膨れない）。
- **相互再帰の α-併合（[#11]・coinductive refinement の正しさ）**: 相互再帰する配列更新ペア（例 even/odd で交互に `set` する `f`/`g`）を **unique 入力**と **shared 入力**の両方で呼ぶ。観測: (a) shared 側で **set 分岐が残存**（IR grep）＋**shared-value 不変**（誤併合で checked 版が unchecked 版に張り替わると shared 配列を in-place 破壊）、(b) symbol 数で「**α 同値な clone は 1 つに併合・非同値（checked/unchecked が違う）clone は分かれている**」ことを確認（`f@U/g@U` と `f@D/g@D` が別クラス）。§9.6 既存の dead-func・`loop@D->loop@U` は**自己**再帰なので、self 正規化だけで済まない**相互**再帰の U/D 混在を突く（§4.1 の partition refinement が checked/unchecked を取り違えて併合すると corruption）。
- **coinductive 併合の正例（[#7round9]・実際に併合が起きること）**: 上の [#11] は 4 clone が**非同値で分かれる**側（split・健全性）を突くが、**楽観初期化＋refinement が循環を越えて実際に併合する**正例も要る。**force-unique を含まない相互再帰ペア**（uniqueness が body に影響しない `f`/`g`——例: 交互に配列を**読むだけ**で返す）を Unique・Dynamic 両キーで呼び、**`f@U≡f@D`・`g@U≡g@D` が 1 つずつに併合**されることを emit symbol 数で assert。読み A（skeleton に clone 名を生で入れる誤実装）だと相互再帰が一切併合されず symbol 数が減らない——[#11] の split テストでは検出できないバイナリサイズ回帰なので、この正例が必須。
- **無限ループ（抜けない末尾再帰）を最適化が扱える（解析の停止性＋実行時 tail 保存）**: `break` しない末尾再帰ループ——unique array を毎反復 in-place `set` する `serve = |s| serve(s.set(i, …))` 型——を入力に:
  - (a) **コンパイルが有限時間で終わる**: 借用化・provenance・unique-check-elim・特殊化の各不動点がハングしない。generous な timeout 付き（例 数分）で `fix build` が正常終了することを assert（不動点が停止しなければハングして落ちる）。
  - (b) **実行 ~10s で末尾再帰が保たれる**: 非 `-g` ビルド（`-g` は `set_tail_call` を付けない、generator.rs:1010）をサブプロセスで起動し、~10s 走らせた後 (i) **生存**（tail が壊れて後続 `Release` が入り非末尾化すると数秒で stack overflow クラッシュ）・(ii) **RSS 一定**（スタック/ヒープが伸びない）を確認して kill。§9.4 は**終端する**深い再帰での tail 検証、ここは**終端しない**ループで解析停止性と tail 保存を突く。
  - (c) ループ本体の `set` の unique 分岐が消えている（IR grep／cachegrind）＝抜けないループでも in-loop elision が効く（§3.2）。
- **間接サイクルの tail を borrow 化が壊さない（[#2]・B 案の tail 全保護）**: **間接呼び出しで閉じる再帰**＋**caller 所有の fresh 値を tail で渡す**入力（decapturing が直接化できない形——closure を配列に入れて渡す `go = |n,acc| ( let ks=[|m,a| go(m,a)]; probe(ks.@(0), n, acc) )`／`probe = |k,n,acc| if n==0 {acc} else { k(n-1, acc+n) }`。加えて `probe` が borrow 可能な fresh を受け取る版も）を大 N（例 1e8）で走らせ**完走**（overflow しない）＝ routing が間接サイクルの所有 tail 呼びを `f_own` に落とし非末尾化しないこと。**回帰検出**: 「所有 tail を `f_borrow` に回す（または閉路のみ保護）」に戻すと、間接辺で閉じるこのサイクルの `go->probe` tail が非末尾化され数秒で overflow。baseline（現コンパイラ・borrow 化なし）は同入力を release で O(1) 完走する（実測。§9.4 の control）ので、この完走は borrow 化が baseline の tail を保ったことの確認。非 `-g`・RSS 一定も併せて観測。
- **span 保存（[#F9]・P2/P3 変換）**: 特殊化 clone・相殺・force-unique 差し替えが**実際に発火した**バイナリで、§9.1 の gdb/backtrace（または `llvm-dwarfdump` の行構造 assert）を再走。P1 のみで検証していた span 保存を P2/P3 でも突く——clone が default span のノードを出す等で DWARF が壊れると、最適化ビルドの backtrace/sanitizer レポートが後で壊れる（§1.1-5 の「全変換が span を保存」の実行時検査）。
- **健全性**: 全テストを assert ビルドで（unique 誤判定 abort）。
観測: IR/asm 分岐 grep／cachegrind（§0 目安の array 部を狙う）／shared-value／symbol・size／assert。

### 9.7 P3.5（`*_uniqueness_unchecked` 削除）
安全版＋§4 で同速・関数消滅を突く。入力: 同じホットパス（ソート・push/append/map）を安全版で → §4 が除去し**旧 unsafe 版と同速**（cachegrind 比較）。削除した関数への参照が無い（使えばコンパイルエラー）。全テスト＋shared-value。観測: cachegrind 同等／grep で不在／全通過。

### 9.8 P4（reuse / 順序スケジューリング / 境界チェック除去）
- reuse: `Release` 直後の alloc が再利用される小例（cachegrind で alloc 数減）。
- 順序スケジューリング: `f(arr.set(0,42), arr.@0)` を並べ替えて `set` が in-place 化（clone 消滅、cachegrind）。
- 境界チェック除去: `idx ∈ [0,size)` 証明で完全 unchecked、一意性除去と合成でベクトル化 → arrayrw が C 比 0.20x（§0）。
各 正しさ（実行結果）＋cachegrind。

## 10. 設計判断の根拠（却下した代替案・レビュー向け）
「素朴にはこうでは？」に対して**なぜその案を採らないか**を残す（同じ論点の蒸し返しを避けるため。将来の読み手に有用な根拠であり、経緯の記録ではない）。

- **borrow 化は 2 版（`f_own`/`f_borrow`）＋呼び出し地点 routing（単一版＋case B・閉路のみ保護・escape-clone は却下＝[#2]/[F2]）**（§2.1）: 初案は「単一の所有権を不動点で決め、閉路 tail で caller 所有末端を Own 固定（case B）」だったが 3 つ破綻した——(1) **閉路判定が間接呼び出しの動的サイクルを取りこぼす**（`Closure`/funptr は静的辺を張らないので、間接辺で閉じるサイクルが静的に非閉路に見え、cross-SCC と誤判定した tail を非末尾化して overflow。[#2]・実測: baseline は間接 tail サイクルを release で O(1)、generator.rs:1008-1010 が全 `App` を tail マーク）、(2) **単一版で Own 固定すると、その callee を非 tail で借用したい別 caller が借用利得を失う**（case B の周辺損失）、(3) **escape-reachable の all-Own original が borrow 化 callee を tail で呼ぶと [#2] を再現**（[F2]・post-hoc pin が不動点の tail 保護に間に合わない）。対処＝**2 版**（`f_own`＝全 Own・`f_borrow`＝借用化）を持ち**呼び出し地点ごとに routing**（間接→`f_own`／所有値の tail 呼び→`f_own`〔非末尾化しない〕／所有非 last-use・借用値→`f_borrow`〔借用利得〕）。tail 保護は閉路判定に非依存（間接サイクルも安全）・借用利得を非 tail 借用 caller で保つ・escape/間接も同じ routing（escape-clone・case B・大域不動点を 1 本に統一）。所有 last-use→`f_own` は #4 のメモリ遅延も大部分解消（早期解放）。dead-func が未使用版を刈る（片使いは 1 版）。**per-param 精密化**（引数ごとに Own/Borrow・最大 2^k 版で混在 call の残り #4 も消す）は v1 では作らず §6。
- **混在した所有値の tail 呼びは `f_own` に丸める（引数ごとに Own/Borrow を混ぜた版は v1 で作らない）**（§2.1）: 同一所有値が tail の複数位置に出る（`g(x,x)`）とき、`g_borrow` が一部 Own・一部 Borrow だと残る Borrow 位置に後続 `Release` が出て**非末尾化**する（その `Release` は §2.2 で消せない——間に Own consume が挟まる）。so routing は「所有引数ありの tail は `g_own`（全 Own）」に落とす（全出現 consume＝tail 保持。複数使用ぶんの `Retain` は lowering が供給・値は `Dynamic` だが同一 fresh の二重渡しはレア）。混在 tail（所有＋借用）も `g_own` へ丸め、借用値は呼び出し前に `Retain`（tail の前なので保たれる）。引数ごとに最適な版（最大 2^k）は v1 では作らない（§6 の per-param 精密化）。
- **§2.2 は consume をまたぐ `Retain`/`Release` を相殺しない（相殺の拡張は却下）**（§2.2）: 健全性の根本制約。間の consume が `Retain` の +1 を奪い `Release` は別参照を落とすので、両方消すと二重消費/UAF。これは欠陥でなく制約。上記の非末尾化は「相殺できるようにする」でなく「相殺対象を作らない」（tail の全 Own 降格）で回避する。
- **force-unique op の結果 `Provenance` は `Fresh` 固定（unchecked で `Arg` 素通しは却下）**（§8 決定事項に詳細）: 結果 identity は入力依存（unique→in-place で同一 object／shared→clone で別 object）だが uniqueness は入力非依存。`Fresh` が後者を捉える。`Arg` にすると provenance が §4 の除去判断に依存＝入力依存になり §3↔§4 の反復不動点が要る。§4 は uniqueness しか読まず両者 resolve 同一なので除去の得ゼロ。別名情報は §6 reuse 用に elim 後の別パスで。
- **`with_retained` は opaque に保つ（内部 retain の外出しは却下）**（§8(d)）: 内部 retain は「呼び出し中 x を shared に見せ in-place 破壊を防ぐ」意味的 RC。外出しすると §2.2 が net-zero として消し、`borrow_boxed`（＝`with_retained` で包む interior-pointer の安全策、§8(b)）経由で unique な x が破壊される。だから常に retain・不透明。
- **分岐合流の join は集合 union（Dyn 潰しは却下）**（§3.1）: `{Fresh, Arg}` を保持し、`Dyn` への潰し込みは resolve の ⊔ が行う。join で Dyn に潰すと arrayrw ループの結果が Dynamic になり elision が全滅する（§3.4）。
- **`Match` は値を産む cexp（`RcRhs`）・式の値は末尾 `Ret(Var)`・tail position は先読み（Match に束縛機能を持たせる案は却下・`Ret` は Var のみ）**（§1.2）: `let x = match…; k` を継続複製の指数爆発なく表すには Match が値を産む必要がある。Match に `Option<(result, 継続)>` を持たせる案は Let の束縛機能と重複するので却下。**Match を `RcRhs`（cexp）に置き、束縛は Let 一本・式の値は末尾 `Ret(Var(x))`**（教科書 ANF の `let x = <複合式> in …`）。`RcExpr`/`RcRhs` 分離の価値（全値に名前・順序固定・オペランド atom → RC/uniqueness の全パスが線形/木の走査）は保つ（App/LLVM の引数は Var のまま）。**`Ret` は Var のみ**〔[#R10-3round10]〕とし、App/Match/LLVM/Closure は必ず let 束縛して consume/provenance を `Let` 規則に一元化（Ret に複合 cexp を許すと各 pass の Ret 規則へ複製が要る）。**tail position**（tail-call elim・phi 回避・borrow/tail を駆動）は IR に符号化せず**先読み（rename 追従）`tail_of`**で決める（`let x=cexp; Ret(Var(x))` なら cexp が tail・Match なら各 arm が tail）——現 codegen の `tail` フラグと同じ挙動を先読みで駆動。tail の match は **phi を作らず各 arm 直接 return**（LLVM tail-call elim 維持）。代償: tail-ness が node 型に出ず `tail_of` 依存になる（§2.1 の一貫性不変条件で担保）。
- **escape-reachable 関数の間接呼び出しは `f_own` へ routing（全面 all-Own ピン留めは却下・2 版で直接の精度は保つ）**（§2.1・[F2]）: `Closure` ノード／funptr atom に現れて間接呼び出しされ得る関数は、間接呼び出し（callee 不明ゆえ固定 all-Own 規約）と直接呼び出しで規約が衝突する（borrow 化＝callee が release しない、を間接呼び出し側が観測できずリーク）。so **routing が間接呼び出しを `f_own` に固定**する。全面 all-Own（直接呼びも借用化を諦める）は精度を捨てるので却下——直接呼び出しは利得 routing で `f_own`/`f_borrow` を選び精度を保つ（2 版モデル）。これは **§4.1 [6]/[8] の「間接呼び出し側が観測できない変換〔呼び出し規約の変更／uniqueness 特殊化〕を escape-reachable な共有 funptr に適用しない」原則**の OwnershipShape 版（§4.1 は uniqueness 特殊化を間接に適用せず original を残す・§2.1 は間接を `f_own` に routing、と同型。旧「post-hoc に original をピン留めして clone」案は callee の tail 保護に間に合わず [F2] で破綻したので、routing に統一）。
- **除去条件は `is_unique` のみで足りる＝LOCAL は `is_unique` に吸収（別 state 解析は健全性に不要・粗近似 v1 も §6 前倒しも却下）**（§4）: threaded になる全経路（`mark_threaded` → 末端 `Dyn`／`boxed_from_retained_ptr` → `Dyn`）が末端を `Dyn` にし、容器越しに threaded になった内側 object も別 handle では `Dyn`（boxed 容器格納＝`Own` consume で handle 消滅／dual-use は Retain→`Dyn`／取り出し＝`Dyn`）。ゆえ **threaded 値を `Fresh`(is_unique) で持てない ⟹ `is_unique ⟹ LOCAL`**。マルチスレッドでも「初回 `set` の force-unique で threaded 解除→以降 unchecked」で動き、borrow-化も健全（borrow 中は自スレッドが rc を 1 保持＝他スレッドの release で dealloc されない・自スレッドは retain/release しない）。**前提**: `mark_threaded`/`mark_global` は **`Own` 引数・結果 `Dyn` の値生成 op**（x を消費し threaded/global handle を `Dyn` で返す）。**operand が `Own`（借用不可）**なのが要——param を mark する関数が Borrow になると caller が threaded object への `Fresh` handle を残し補題が破れる（[5]）。Own なら caller が dual-use で `Retain`→`Dyn`。`mark_global` は user-callable でなく global init 専用（[5] の穴なし・一貫性で同形）。
- **clone の冗長除去は「実質同一 `RcFunc` の併合パス」で行う（後ろ向き precompute 射影は後段）**（§4.1）: v1 は特殊化キーを全 `UniquenessShape` にし、force-unique に効かない末端違いで生じた **α 同値 clone**（名前グローバル一意〔§1.1-3〕ゆえ局所名だけ違う）を構造ハッシュで併合する（＋未到達 clone の dead-function 除去）。汎用で軽く、α 同値判定が素直。関連末端への射影（inter-procedural な後ろ向き precompute）は「そもそも冗長 clone を作らない」最適化として後で足す＝キー設計を初版から作り込まない。
- **`swap` は builtin 化し PunchedArray に載せない（単一 idx PunchedArray への相乗りは却下）**（§7 P0.7）: swap は i,j の穴を 2 つ同時に開けるので単一 idx の PunchedArray に載らない。builtin `swap`（force_unique フラグ）で atomic 化して §4 の除去対象にする。
- **PunchedArray は `unbox { Array a, I64 }` ＋専用 traverser（boxed 化・共有 array 分岐の改変は却下）**（§7 P0.7）: 内側 `Array a` は独立 refcount の boxed 値で、要素解放は rc==0 で型共有の array traverser を通る（idx が見えない・全配列共有）。ゆえ `build_traverse` の `Array` 分岐に skip-idx を足すのは不可で、`PunchedArray` 専用 traverser（idx を scope に持ち内側配列を skip-idx 解放）を追加する。boxed 化は実行時劣化なので不採用。
